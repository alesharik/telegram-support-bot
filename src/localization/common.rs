use tracing::warn;
use crate::localization::LocKey;

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
}

impl LocKey for CommonMessages {
    fn key(&self) -> String {
        match self {
            CommonMessages::VoiceMessagesNotSupported => "common.voiceMessagesNotSupported",
            CommonMessages::PollsNotSupported => "common.pollsNotSupported",
            CommonMessages::GamesNotSupported => "common.gamesNotSupported",
            CommonMessages::InfoHeader { .. } => "common.infoHeader"
        }.to_string()
    }

    fn default_message(&self) -> String {
        match self {
            CommonMessages::VoiceMessagesNotSupported => "Voice messages not supported".to_string(),
            CommonMessages::GamesNotSupported => "Games not supported".to_string(),
            CommonMessages::PollsNotSupported => "Polls not supported".to_string(),
            CommonMessages::InfoHeader { .. } => "<b><a href=\"tg://user?id={id}\">{first_name} {last_name}</a></b>\n<b>Language: </b> {lang}\n".to_string(),
        }
    }

    fn args(self) -> Option<Vec<(String, String)>> {
        match self {
            CommonMessages::VoiceMessagesNotSupported => None,
            CommonMessages::PollsNotSupported => None,
            CommonMessages::GamesNotSupported => None,
            CommonMessages::InfoHeader { last_name, id, lang, first_name } => Some(vec![
                ("id".to_string(), id.to_string()),
                ("first_name".to_string(), sanitize(first_name.unwrap_or_default())),
                ("last_name".to_string(), sanitize(last_name.unwrap_or_default())),
                ("lang".to_string(), sanitize(lang.unwrap_or_default())),
            ])
        }
    }
}

fn sanitize(s: String) -> String {
    match sanitize_html::sanitize_str(&sanitize_html::rules::predefined::DEFAULT, &s) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to sanitize string {}: {:?}", s, e);
            String::new()
        }
    }
}