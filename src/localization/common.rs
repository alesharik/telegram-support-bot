use tracing::warn;
use crate::localization::{LocKey, sanitize};

#[derive(Clone)]
pub enum CommonMessages {
    VoiceMessagesNotSupported,
    PollsNotSupported,
    GamesNotSupported,
    InfoHeader {
        id: i64,
        first_name: Option<String>,
        last_name: Option<String>,
        lang: Option<String>,
    },
    Welcome,
    Faq,
    UserReply,
}

impl LocKey for CommonMessages {
    fn key(&self) -> String {
        match self {
            CommonMessages::VoiceMessagesNotSupported => "common.voiceMessagesNotSupported",
            CommonMessages::PollsNotSupported => "common.pollsNotSupported",
            CommonMessages::GamesNotSupported => "common.gamesNotSupported",
            CommonMessages::InfoHeader { .. } => "common.infoHeader",
            CommonMessages::Welcome => "common.welcome",
            CommonMessages::Faq => "common.faq",
            CommonMessages::UserReply => "common.userReply",
        }.to_string()
    }

    fn default_message(&self) -> String {
        match self {
            CommonMessages::VoiceMessagesNotSupported => "Voice messages not supported".to_string(),
            CommonMessages::GamesNotSupported => "Games not supported".to_string(),
            CommonMessages::PollsNotSupported => "Polls not supported".to_string(),
            CommonMessages::InfoHeader { .. } => "<b><a href=\"tg://user?id={id}\">{first_name} {last_name}</a></b>\n<b>Language: </b> {lang}\n".to_string(),
            CommonMessages::Welcome => "Welcome to support chat! Ask your questions here".to_string(),
            CommonMessages::Faq => "To contact support, send your message, video or file. You will receive support answer in this chat".to_string(),
            CommonMessages::UserReply => "Thank you for contacting us. We will answer as soon as possible.".to_string(),
        }
    }

    fn args(self) -> Option<Vec<(String, String)>> {
        match self {
            CommonMessages::VoiceMessagesNotSupported => None,
            CommonMessages::PollsNotSupported => None,
            CommonMessages::GamesNotSupported => None,
            CommonMessages::Welcome => None,
            CommonMessages::Faq => None,
            CommonMessages::UserReply => None,

            CommonMessages::InfoHeader { last_name, id, lang, first_name } => Some(vec![
                ("id".to_string(), id.to_string()),
                ("first_name".to_string(), sanitize(first_name.unwrap_or_default())),
                ("last_name".to_string(), sanitize(last_name.unwrap_or_default())),
                ("lang".to_string(), sanitize(lang.unwrap_or_default())),
            ])
        }
    }
}