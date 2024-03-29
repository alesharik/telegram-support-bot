mod utils;

use std::sync::Arc;
use serde::Deserialize;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{BotCommandScope, InputFile, MediaKind, MessageId, MessageKind, ParseMode, ReactionEmoji, ReactionType, Recipient, ThreadId};
use crate::database::{Database, InsertMessageEntity, InsertNoteEntity, InsertUserEntity, MessageType, UserEntity};
use crate::localization::{CommonMessages, LocalizationBundle, sanitize};
use crate::telegram::utils::MessageBuilder;

#[derive(Deserialize, Debug, Clone)]
pub struct TelegramConfig {
    pub token: String,
    pub superchat: i64,
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum UserCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "print welcome message.")]
    Start,
    #[command(description = "show FAQ.")]
    Faq,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum SupportCommand {
    #[command(description = "Set note for user", parse_with = "split")]
    Setnote { key: String, value: String },
    #[command(description = "Get all notes for user")]
    Notes,
    #[command(description = "Delete note")]
    Delnote { key: String },
}

pub async fn run(config: TelegramConfig, db: Box<dyn Database + 'static>, loc: LocalizationBundle) -> anyhow::Result<()> {
    use teloxide::utils::command::BotCommands;

    let bot = Bot::new(config.token.clone());

    let superchat = ChatId(config.superchat);

    bot.set_my_commands(UserCommand::bot_commands())
        .scope(BotCommandScope::AllPrivateChats)
        .await?;
    bot.set_my_commands(SupportCommand::bot_commands())
        .scope(BotCommandScope::Chat { chat_id: Recipient::Id(superchat) })
        .await?;

    Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(Update::filter_message()
                .branch(dptree::filter(|m: Message| { m.chat.is_private() })
                    .branch(Update::filter_message().filter_command::<UserCommand>().endpoint(user_cmd))
                    .branch(Update::filter_message()).endpoint(user_msg))
                .branch(dptree::filter(move |m: Message| { m.chat.id == superchat })
                    .branch(Update::filter_message().filter_command::<SupportCommand>().endpoint(superchat_cmd))
                    .branch(Update::filter_message()).endpoint(superchat_msg)))
            .branch(Update::filter_edited_message()
                .branch(dptree::filter(|m: Message| { m.chat.is_private() })
                    .branch(Update::filter_message().filter_command::<UserCommand>().endpoint(noop))
                    .branch(Update::filter_message()).endpoint(user_update))
                .branch(dptree::filter(move |m: Message| { m.chat.id == superchat })
                    .branch(Update::filter_message().filter_command::<SupportCommand>().endpoint(noop))
                    .branch(Update::filter_message()).endpoint(superchat_update)))
    )
        .dependencies(dptree::deps![config, Arc::new(db), Arc::new(loc)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn noop() -> HandlerResult {
    Ok(())
}
async fn update_user_info_msg(bot: &Bot, mut entity: UserEntity, cfg: TelegramConfig, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> Result<UserEntity, Box<dyn std::error::Error + Send + Sync>> {
    let bot = bot.parse_mode(ParseMode::Html);
    let mut msg = loc.localize(None, CommonMessages::InfoHeader {
        lang: entity.lang_code.clone(),
        last_name: entity.last_name.clone().map(|s| s.to_string()),
        first_name: entity.first_name.clone().map(|s| s.to_string()),
        id: entity.telegram_id,
    });
    for note in db.get_notes(&entity).await? {
        msg = format!("{}<b>{}: </b><code>{}</code>\n", msg, sanitize(note.key), sanitize(note.value));
    }

    if let Some(id) = entity.info_message {
        bot.edit_message_text(ChatId(cfg.superchat), MessageId(id as i32), &msg)
            .await?;
        Ok(entity)
    } else {
        let msg = bot.send_message(ChatId(cfg.superchat), &msg).message_thread_id(ThreadId(MessageId(entity.topic as i32))).await?;
        bot.pin_chat_message(ChatId(cfg.superchat), msg.id).await?;
        entity.info_message = Some(msg.id.0 as i64);
        db.update_user(entity.clone()).await?;
        Ok(entity)
    }
}

async fn user_cmd(bot: Bot, msg: Message, loc: Arc<LocalizationBundle>, cmd: UserCommand) -> HandlerResult {
    use teloxide::utils::command::BotCommands;

    let user_lang = msg.from().and_then(|u| u.language_code.clone());
    match cmd {
        UserCommand::Help => bot.send_message(msg.chat.id, UserCommand::descriptions().to_string()).await?,
        UserCommand::Start => bot.send_message(msg.chat.id, loc.localize(user_lang, CommonMessages::Welcome)).await?,
        UserCommand::Faq => bot.send_message(msg.chat.id, loc.localize(user_lang, CommonMessages::Faq)).await?,
    };
    Ok(())
}

async fn superchat_cmd(bot: Bot, msg: Message, loc: Arc<LocalizationBundle>, cfg: TelegramConfig, db: Arc<Box<dyn Database>>, cmd: SupportCommand) -> HandlerResult {
    let Some(topic) = msg.thread_id else {
        return Ok(());
    };
    let Some(user) = db.get_user_by_topic(topic.0.0 as i64).await? else {
        return Ok(());
    };

    match cmd {
        SupportCommand::Setnote { key, value } => {
            db.save_note(InsertNoteEntity { user_id: user.id, key: key.trim().to_string(), value: value.trim().to_string() }).await?;
            update_user_info_msg(&bot, user, cfg.clone(), db.clone(), loc.clone()).await?;
            bot.send_message(ChatId(cfg.superchat), "Note saved")
                .message_thread_id(topic)
                .await?;
        }
        SupportCommand::Delnote { key } => {
            db.delete_note(&user, key.trim()).await?;
            update_user_info_msg(&bot, user, cfg.clone(), db.clone(), loc.clone()).await?;
            bot.parse_mode(ParseMode::Html)
                .send_message(ChatId(cfg.superchat), "Note deleted")
                .message_thread_id(topic)
                .await?;
        }
        SupportCommand::Notes => {
            let mut msg = "User notes:\n".to_string();
            for note in db.get_notes(&user).await? {
                msg = format!("{}\n<b>{}:</b> <code>{}</code>", msg, note.key, note.value);
            }
            bot.parse_mode(ParseMode::Html)
                .send_message(ChatId(cfg.superchat), msg)
                .message_thread_id(topic)
                .await?;
        }
    };
    Ok(())
}

async fn user_msg(bot: Bot, msg: Message, cfg: TelegramConfig, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> HandlerResult {
    let user = match db.get_user_by_tg_id(UserId(msg.chat.id.0 as u64)).await? {
        None => {
            let name = format!("#T {} {}", msg.chat.first_name().unwrap_or(""), msg.chat.last_name().unwrap_or(""));
            let topic = bot.create_forum_topic(ChatId(cfg.superchat), name, 16766590, "").await?;
            let entity = InsertUserEntity {
                telegram_id: msg.chat.id.0,
                topic: topic.thread_id.0.0 as i64,
                info_message: None,
                first_name: msg.chat.first_name().map(|s| s.to_string()),
                last_name: msg.chat.last_name().map(|s| s.to_string()),
                lang_code: msg.from().and_then(|l| l.language_code.clone()),
            };
            let en = db.insert_user(entity).await?;
            bot.edit_forum_topic(ChatId(cfg.superchat), topic.thread_id)
                .name(format!("#T{:#06} {} {}", en.id, msg.chat.first_name().unwrap_or(""), msg.chat.last_name().unwrap_or("")))
                .await?;
            update_user_info_msg(&bot, en, cfg.clone(), db.clone(), loc.clone()).await?
        }
        Some(user) => if user.info_message.is_none() {
            update_user_info_msg(&bot, user, cfg.clone(), db.clone(), loc.clone()).await?
        } else {
            user
        },
    };
    let tx = match msg.kind {
        MessageKind::Common(ref a) => match a.media_kind.clone() {
            MediaKind::Animation(obj) => {
                MessageBuilder::new(bot.send_animation(ChatId(cfg.superchat), InputFile::file_id(obj.animation.file.id)))
                    .with(obj.caption, |o, v| { v.caption(o) })
                    .build()
                    .caption_entities(obj.caption_entities)
                    .has_spoiler(obj.has_media_spoiler)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Audio(audio) => {
                MessageBuilder::new(bot.send_audio(ChatId(cfg.superchat), InputFile::file_id(audio.audio.file.id)))
                    .with(audio.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(audio.caption_entities)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Contact(contact) => {
                MessageBuilder::new(bot.send_contact(ChatId(cfg.superchat), contact.contact.phone_number, contact.contact.first_name))
                    .with(contact.contact.last_name, |o, v| v.last_name(o))
                    .with(contact.contact.vcard, |o, v| v.vcard(o))
                    .build()
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Document(doc) => {
                MessageBuilder::new(bot.send_document(ChatId(cfg.superchat), InputFile::file_id(doc.document.file.id)))
                    .with(doc.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(doc.caption_entities)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Venue(v) => {
                MessageBuilder::new(bot.send_venue(ChatId(cfg.superchat), v.venue.location.latitude, v.venue.location.longitude, v.venue.title, v.venue.address))
                    .with(v.venue.foursquare_id, |o, v| v.foursquare_id(o))
                    .with(v.venue.foursquare_type, |o, v| v.foursquare_type(o))
                    .with(v.venue.google_place_id, |o, v| v.google_place_id(o))
                    .with(v.venue.google_place_type, |o, v| v.google_place_type(o))
                    .build()
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Location(loc) => {
                MessageBuilder::new(bot.send_location(ChatId(cfg.superchat), loc.location.latitude, loc.location.longitude))
                    .with(loc.location.horizontal_accuracy, |o, v| v.horizontal_accuracy(o))
                    .with(loc.location.live_period, |o, v| v.live_period(o.seconds()))
                    .with(loc.location.heading, |o, v| v.heading(o))
                    .with(loc.location.proximity_alert_radius, |o, v| v.proximity_alert_radius(o))
                    .build()
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Photo(p) => {
                let Some(photo) = p.photo.iter().max_by_key(|a| a.width * a.height) else {
                    return Ok(())
                };
                MessageBuilder::new(bot.send_photo(ChatId(cfg.superchat), InputFile::file_id(photo.file.id.clone())))
                    .with(p.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(p.caption_entities)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Sticker(s) => {
                bot.send_sticker(ChatId(cfg.superchat), InputFile::file_id(s.sticker.file.id))
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Text(t) => {
                bot.send_message(ChatId(cfg.superchat), t.text)
                    .entities(t.entities)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Video(v) => {
                MessageBuilder::new(bot.send_video(ChatId(cfg.superchat), InputFile::file_id(v.video.file.id)))
                    .with(v.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(v.caption_entities)
                    .has_spoiler(v.has_media_spoiler)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::VideoNote(v) => {
                bot.send_video_note(ChatId(cfg.superchat), InputFile::file_id(v.video_note.file.id))
                    .length(v.video_note.length)
                    .duration(v.video_note.duration.seconds())
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }
            MediaKind::Voice(v) => {
                MessageBuilder::new(bot.send_voice(ChatId(cfg.superchat), InputFile::file_id(v.voice.file.id)))
                    .with(v.caption, |c, v| v.caption(c))
                    .build()
                    .caption_entities(v.caption_entities)
                    .message_thread_id(ThreadId(MessageId(user.topic as i32)))
                    .await?
            }

            MediaKind::Game(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::GamesNotSupported)).await?;
                return Ok(());
            }
            MediaKind::Poll(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::PollsNotSupported)).await?;
                return Ok(());
            }
            _ => return Ok(())
        }
        _ => return Ok(())
    };
    db.insert_message(InsertMessageEntity::incoming(&user, &msg, tx.id)).await?;
    bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|l| l.language_code.clone()), CommonMessages::UserReply)).await?;
    Ok(())
}

async fn superchat_msg(bot: Bot, msg: Message, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> HandlerResult {
    let Some(topic) = msg.thread_id else {
        return Ok(());
    };
    let Some(user) = db.get_user_by_topic(topic.0.0 as i64).await? else {
        return Ok(());
    };
    let uid = UserId(user.telegram_id as u64);
    let tx = match msg.kind {
        MessageKind::Common(ref a) => match a.media_kind.clone() {
            MediaKind::Animation(obj) => {
                MessageBuilder::new(bot.send_animation(uid, InputFile::file_id(obj.animation.file.id)))
                    .with(obj.caption, |o, v| { v.caption(o) })
                    .build()
                    .caption_entities(obj.caption_entities)
                    .has_spoiler(obj.has_media_spoiler)
                    .await?
            }
            MediaKind::Audio(audio) => {
                MessageBuilder::new(bot.send_audio(uid, InputFile::file_id(audio.audio.file.id)))
                    .with(audio.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(audio.caption_entities)
                    .await?
            }
            MediaKind::Contact(contact) => {
                MessageBuilder::new(bot.send_contact(uid, contact.contact.phone_number, contact.contact.first_name))
                    .with(contact.contact.last_name, |o, v| v.last_name(o))
                    .with(contact.contact.vcard, |o, v| v.vcard(o))
                    .build()
                    .await?
            }
            MediaKind::Document(doc) => {
                MessageBuilder::new(bot.send_document(uid, InputFile::file_id(doc.document.file.id)))
                    .with(doc.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(doc.caption_entities)
                    .await?
            }
            MediaKind::Venue(v) => {
                MessageBuilder::new(bot.send_venue(uid, v.venue.location.latitude, v.venue.location.longitude, v.venue.title, v.venue.address))
                    .with(v.venue.foursquare_id, |o, v| v.foursquare_id(o))
                    .with(v.venue.foursquare_type, |o, v| v.foursquare_type(o))
                    .with(v.venue.google_place_id, |o, v| v.google_place_id(o))
                    .with(v.venue.google_place_type, |o, v| v.google_place_type(o))
                    .build()
                    .await?
            }
            MediaKind::Location(loc) => {
                MessageBuilder::new(bot.send_location(uid, loc.location.latitude, loc.location.longitude))
                    .with(loc.location.horizontal_accuracy, |o, v| v.horizontal_accuracy(o))
                    .with(loc.location.live_period, |o, v| v.live_period(o.seconds()))
                    .with(loc.location.heading, |o, v| v.heading(o))
                    .with(loc.location.proximity_alert_radius, |o, v| v.proximity_alert_radius(o))
                    .build()
                    .await?
            }
            MediaKind::Photo(p) => {
                let Some(photo) = p.photo.iter().max_by_key(|a| a.width * a.height) else {
                    return Ok(())
                };
                MessageBuilder::new(bot.send_photo(uid, InputFile::file_id(photo.file.id.clone())))
                    .with(p.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(p.caption_entities)
                    .await?
            }
            MediaKind::Sticker(s) => {
                bot.send_sticker(uid, InputFile::file_id(s.sticker.file.id))
                    .await?
            }
            MediaKind::Text(t) => {
                bot.send_message(uid, t.text)
                    .entities(t.entities)
                    .await?
            }
            MediaKind::Video(v) => {
                MessageBuilder::new(bot.send_video(uid, InputFile::file_id(v.video.file.id)))
                    .with(v.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(v.caption_entities)
                    .has_spoiler(v.has_media_spoiler)
                    .await?
            }
            MediaKind::VideoNote(v) => {
                bot.send_video_note(uid, InputFile::file_id(v.video_note.file.id))
                    .length(v.video_note.length)
                    .duration(v.video_note.duration.seconds())
                    .await?
            }
            MediaKind::Voice(v) => {
                MessageBuilder::new(bot.send_voice(uid, InputFile::file_id(v.voice.file.id)))
                    .with(v.caption, |c, v| v.caption(c))
                    .build()
                    .caption_entities(v.caption_entities)
                    .await?
            }

            MediaKind::Game(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::GamesNotSupported)).await?;
                return Ok(());
            }
            MediaKind::Poll(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::PollsNotSupported)).await?;
                return Ok(());
            }
            _ => return Ok(())
        }
        _ => return Ok(())
    };
    db.insert_message(InsertMessageEntity::outgoing(&user, &msg, tx.id)).await?;
    bot.set_message_reaction(msg.chat.id, msg.id, vec![ReactionType::emoji(ReactionEmoji::Lightning)]).await?;
    Ok(())
}

async fn user_update(bot: Bot, edited: Message, cfg: TelegramConfig, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> HandlerResult {
    let Some(user) = db.get_user_by_tg_id(UserId(edited.chat.id.0 as u64)).await? else {
        return Ok(())
    };
    let Some(msg) = db.get_message(&user, MessageType::Incoming, edited.id.0 as i64).await? else {
        return user_msg(bot, edited, cfg, db, loc).await;
    };
    let mid = MessageId(msg.tx_msg_id as i32);
    let original = msg.rx_message()?;
    if original.caption() != edited.caption() || original.caption_entities() != edited.caption_entities() {
        MessageBuilder::new(bot.edit_message_caption(ChatId(cfg.superchat), mid))
            .with(edited.caption(), |c, v| v.caption(c))
            .with(edited.caption_entities(), |c, v| v.caption_entities(c.iter().cloned().collect::<Vec<_>>()))
            .build()
            .await?;
    }
    if original.text() != edited.text() {
        if let Some(text) = edited.text() {
            bot.edit_message_text(ChatId(cfg.superchat), mid, text).await?;
        }
    }
    if original.location() != edited.location() {
        if let Some(location) = edited.location() {
            bot.edit_message_live_location(ChatId(cfg.superchat), mid, location.latitude, location.longitude).await?;
        }
    }
    Ok(())
}

async fn superchat_update(bot: Bot, edited: Message, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> HandlerResult {
    let Some(topic) = edited.thread_id else {
        return Ok(());
    };
    let Some(user) = db.get_user_by_topic(topic.0.0 as i64).await? else {
        return Ok(());
    };
    let Some(msg) = db.get_message(&user, MessageType::Outgoing, edited.id.0 as i64).await? else {
        return superchat_msg(bot, edited, db, loc).await;
    };
    let mid = MessageId(msg.tx_msg_id as i32);
    let uid = UserId(user.telegram_id as u64);
    let original = msg.rx_message()?;
    if original.caption() != edited.caption() || original.caption_entities() != edited.caption_entities() {
        MessageBuilder::new(bot.edit_message_caption(uid, mid))
            .with(edited.caption(), |c, v| v.caption(c))
            .with(edited.caption_entities(), |c, v| v.caption_entities(c.iter().cloned().collect::<Vec<_>>()))
            .build()
            .await?;
    }
    if original.text() != edited.text() {
        if let Some(text) = edited.text() {
            bot.edit_message_text(uid, mid, text).await?;
        }
    }
    if original.location() != edited.location() {
        if let Some(location) = edited.location() {
            bot.edit_message_live_location(uid, mid, location.latitude, location.longitude).await?;
        }
    }
    Ok(())
}