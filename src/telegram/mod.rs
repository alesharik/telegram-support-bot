mod utils;

use std::sync::Arc;
use serde::Deserialize;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MediaKind, MessageId, MessageKind, ParseMode};
use crate::database::{Database, InsertUserEntity, UserEntity};
use crate::localization::{CommonMessages, LocalizationBundle};
use crate::telegram::utils::MessageBuilder;

#[derive(Deserialize, Debug, Clone)]
pub struct TelegramConfig {
    pub token: String,
    pub superchat: i64,
    pub managers: Vec<i64>,
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(config: TelegramConfig, db: Box<dyn Database + 'static>, loc: LocalizationBundle) -> anyhow::Result<()> {
    let bot = Bot::new(config.token.clone());

    let superchat = ChatId(config.superchat);
    Dispatcher::builder(
        bot,
        Update::filter_message()
            .branch(dptree::filter(|m: Message| { m.chat.is_private() }).endpoint(user_msg))
            .branch(dptree::filter(move |m: Message| { m.chat.id == superchat }).endpoint(superchat_msg))
    )
        .dependencies(dptree::deps![config, Arc::new(db), Arc::new(loc)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn update_user_info_msg(bot: &Bot, user_msg: &Message, mut entity: UserEntity, cfg: TelegramConfig, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> Result<UserEntity, Box<dyn std::error::Error + Send + Sync>> {
    let bot = bot.parse_mode(ParseMode::Html);
    let header = loc.localize(user_msg.from().and_then(|u| u.language_code.clone()), CommonMessages::InfoHeader {
        lang: user_msg.from().and_then(|u| u.language_code.clone()),
        last_name: user_msg.chat.last_name().map(|s| s.to_string()),
        first_name: user_msg.chat.first_name().map(|s| s.to_string()),
        id: user_msg.chat.id.0,
    });

    if let Some(id) = entity.info_message {
        bot.edit_message_text(ChatId(cfg.superchat), MessageId(id as i32), &header)
            .await?;
        Ok(entity)
    } else {
        let msg = bot.send_message(ChatId(cfg.superchat), &header).message_thread_id(entity.topic as i32).await?;
        bot.pin_chat_message(ChatId(cfg.superchat), msg.id).await?;
        entity.info_message = Some(msg.id.0 as i64);
        db.update_user(entity.clone()).await?;
        Ok(entity)
    }
}

async fn user_msg(bot: Bot, msg: Message, cfg: TelegramConfig, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> HandlerResult {
    let user = match db.get_user_by_tg_id(UserId(msg.chat.id.0 as u64)).await? {
        None => {
            let name = format!("#T {} {}", msg.chat.first_name().unwrap_or(""), msg.chat.last_name().unwrap_or(""));
            let topic = bot.create_forum_topic(ChatId(cfg.superchat), name, 16766590, "").await?;
            let entity = InsertUserEntity { telegram_id: msg.chat.id.0, topic: topic.message_thread_id as i64, info_message: None };
            let en = db.insert_user(entity).await?;
            bot.edit_forum_topic(ChatId(cfg.superchat), topic.message_thread_id)
                .name(format!("#T{:#06} {} {}", en.id, msg.chat.first_name().unwrap_or(""), msg.chat.last_name().unwrap_or("")))
                .await?;
            update_user_info_msg(&bot, &msg, en, cfg.clone(), db.clone(), loc.clone()).await?
        }
        Some(user) => if user.info_message.is_none() {
            update_user_info_msg(&bot, &msg, user, cfg.clone(), db.clone(), loc.clone()).await?
        } else {
            user
        },
    };
    match msg.kind {
        MessageKind::Common(ref a) => match a.media_kind.clone() {
            MediaKind::Animation(obj) => {
                MessageBuilder::new(bot.send_animation(ChatId(cfg.superchat), InputFile::file_id(obj.animation.file.id)))
                    .with(obj.caption, |o, v| { v.caption(o) })
                    .build()
                    .caption_entities(obj.caption_entities)
                    .has_spoiler(obj.has_media_spoiler)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Audio(audio) => {
                MessageBuilder::new(bot.send_audio(ChatId(cfg.superchat), InputFile::file_id(audio.audio.file.id)))
                    .with(audio.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(audio.caption_entities)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Contact(contact) => {
                MessageBuilder::new(bot.send_contact(ChatId(cfg.superchat), contact.contact.phone_number, contact.contact.first_name))
                    .with(contact.contact.last_name, |o, v| v.last_name(o))
                    .with(contact.contact.vcard, |o, v| v.vcard(o))
                    .build()
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Document(doc) => {
                MessageBuilder::new(bot.send_document(ChatId(cfg.superchat), InputFile::file_id(doc.document.file.id)))
                    .with(doc.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(doc.caption_entities)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Venue(v) => {
                MessageBuilder::new(bot.send_venue(ChatId(cfg.superchat), v.venue.location.latitude, v.venue.location.longitude, v.venue.title, v.venue.address))
                    .with(v.venue.foursquare_id, |o, v| v.foursquare_id(o))
                    .with(v.venue.foursquare_type, |o, v| v.foursquare_type(o))
                    .with(v.venue.google_place_id, |o, v| v.google_place_id(o))
                    .with(v.venue.google_place_type, |o, v| v.google_place_type(o))
                    .build()
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Location(loc) => {
                MessageBuilder::new(bot.send_location(ChatId(cfg.superchat), loc.location.latitude, loc.location.longitude))
                    .with(loc.location.horizontal_accuracy, |o, v| v.horizontal_accuracy(o))
                    .with(loc.location.live_period, |o, v| v.live_period(o))
                    .with(loc.location.heading, |o, v| v.heading(o))
                    .with(loc.location.proximity_alert_radius, |o, v| v.proximity_alert_radius(o))
                    .build()
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Photo(p) => {
                let Some(photo) = p.photo.iter().max_by_key(|a| a.width * a.height) else {
                    return Ok(())
                };
                MessageBuilder::new(bot.send_photo(ChatId(cfg.superchat), InputFile::file_id(photo.file.id.clone())))
                    .with(p.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(p.caption_entities)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Sticker(s) => {
                bot.send_sticker(ChatId(cfg.superchat), InputFile::file_id(s.sticker.file.id))
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Text(t) => {
                bot.send_message(ChatId(cfg.superchat), t.text)
                    .entities(t.entities)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::Video(v) => {
                MessageBuilder::new(bot.send_video(ChatId(cfg.superchat), InputFile::file_id(v.video.file.id)))
                    .with(v.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(v.caption_entities)
                    .has_spoiler(v.has_media_spoiler)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }
            MediaKind::VideoNote(v) => {
                bot.send_video_note(ChatId(cfg.superchat), InputFile::file_id(v.video_note.file.id))
                    .length(v.video_note.length)
                    .duration(v.video_note.duration)
                    .message_thread_id(user.topic as i32)
                    .await?;
            }

            MediaKind::Game(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::GamesNotSupported)).await?;
            }
            MediaKind::Poll(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::PollsNotSupported)).await?;
            }
            MediaKind::Voice(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::VoiceMessagesNotSupported)).await?;
            }
            _ => return Ok(())
        }
        _ => return Ok(())
    }
    Ok(())
}

async fn superchat_msg(bot: Bot, msg: Message, db: Arc<Box<dyn Database>>, loc: Arc<LocalizationBundle>) -> HandlerResult {
    let Some(topic) = msg.thread_id else {
        return Ok(());
    };
    let Some(user) = db.get_user_by_topic(topic as i64).await? else {
        return Ok(());
    };
    let uid = UserId(user.telegram_id as u64);
    match msg.kind {
        MessageKind::Common(ref a) => match a.media_kind.clone() {
            MediaKind::Animation(obj) => {
                MessageBuilder::new(bot.send_animation(uid, InputFile::file_id(obj.animation.file.id)))
                    .with(obj.caption, |o, v| { v.caption(o) })
                    .build()
                    .caption_entities(obj.caption_entities)
                    .has_spoiler(obj.has_media_spoiler)
                    .await?;
            }
            MediaKind::Audio(audio) => {
                MessageBuilder::new(bot.send_audio(uid, InputFile::file_id(audio.audio.file.id)))
                    .with(audio.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(audio.caption_entities)
                    .await?;
            }
            MediaKind::Contact(contact) => {
                MessageBuilder::new(bot.send_contact(uid, contact.contact.phone_number, contact.contact.first_name))
                    .with(contact.contact.last_name, |o, v| v.last_name(o))
                    .with(contact.contact.vcard, |o, v| v.vcard(o))
                    .build()
                    .await?;
            }
            MediaKind::Document(doc) => {
                MessageBuilder::new(bot.send_document(uid, InputFile::file_id(doc.document.file.id)))
                    .with(doc.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(doc.caption_entities)
                    .await?;
            }
            MediaKind::Venue(v) => {
                MessageBuilder::new(bot.send_venue(uid, v.venue.location.latitude, v.venue.location.longitude, v.venue.title, v.venue.address))
                    .with(v.venue.foursquare_id, |o, v| v.foursquare_id(o))
                    .with(v.venue.foursquare_type, |o, v| v.foursquare_type(o))
                    .with(v.venue.google_place_id, |o, v| v.google_place_id(o))
                    .with(v.venue.google_place_type, |o, v| v.google_place_type(o))
                    .build()
                    .await?;
            }
            MediaKind::Location(loc) => {
                MessageBuilder::new(bot.send_location(uid, loc.location.latitude, loc.location.longitude))
                    .with(loc.location.horizontal_accuracy, |o, v| v.horizontal_accuracy(o))
                    .with(loc.location.live_period, |o, v| v.live_period(o))
                    .with(loc.location.heading, |o, v| v.heading(o))
                    .with(loc.location.proximity_alert_radius, |o, v| v.proximity_alert_radius(o))
                    .build()
                    .await?;
            }
            MediaKind::Photo(p) => {
                let Some(photo) = p.photo.iter().max_by_key(|a| a.width * a.height) else {
                    return Ok(())
                };
                MessageBuilder::new(bot.send_photo(uid, InputFile::file_id(photo.file.id.clone())))
                    .with(p.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(p.caption_entities)
                    .await?;
            }
            MediaKind::Sticker(s) => {
                bot.send_sticker(uid, InputFile::file_id(s.sticker.file.id))
                    .await?;
            }
            MediaKind::Text(t) => {
                bot.send_message(uid, t.text)
                    .entities(t.entities)
                    .await?;
            }
            MediaKind::Video(v) => {
                MessageBuilder::new(bot.send_video(uid, InputFile::file_id(v.video.file.id)))
                    .with(v.caption, |o, v| v.caption(o))
                    .build()
                    .caption_entities(v.caption_entities)
                    .has_spoiler(v.has_media_spoiler)
                    .await?;
            }
            MediaKind::VideoNote(v) => {
                bot.send_video_note(uid, InputFile::file_id(v.video_note.file.id))
                    .length(v.video_note.length)
                    .duration(v.video_note.duration)
                    .await?;
            }

            MediaKind::Game(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::GamesNotSupported)).await?;
            }
            MediaKind::Poll(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::PollsNotSupported)).await?;
            }
            MediaKind::Voice(_) => {
                bot.send_message(msg.chat.id, loc.localize(msg.from().and_then(|u| u.language_code.clone()), CommonMessages::VoiceMessagesNotSupported)).await?;
            }
            _ => return Ok(())
        }
        _ => return Ok(())
    }
    Ok(())
}