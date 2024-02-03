use crate::localization::LocKey;

#[derive(Clone)]
pub enum CommonMessages {
    VoiceMessagesNotSupported,
    PollsNotSupported,
    GamesNotSupported,
}

impl LocKey for CommonMessages {
    fn key(&self) -> String {
        match self {
            CommonMessages::VoiceMessagesNotSupported => "common.voiceMessagesNotSupported",
            CommonMessages::PollsNotSupported => "common.pollsNotSupported",
            CommonMessages::GamesNotSupported => "common.gamesNotSupported"
        }.to_string()
    }

    fn default_message(&self) -> String {
        match self {
            CommonMessages::VoiceMessagesNotSupported => "Voice messages not supported".to_string(),
            CommonMessages::GamesNotSupported => "Games not supported".to_string(),
            CommonMessages::PollsNotSupported => "Polls not supported".to_string(),
        }
    }

    fn args(self) -> Option<Vec<(String, String)>> {
        None
    }
}