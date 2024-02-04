# Telegram support bot

### Features
- forwards anything
- synchronizes all message changes
- manages chats within superchat
- anonymizes staff
- localization support
- user notes (for keeping context)

### Commands
#### User
- `/start` - print welcome message
- `/help` - print help message
- `/faq` - print FAQ message

#### Staff
- `/setnote a b` - set note `a` with value `b` for user
- `/notes` - get all user notes
- `/delnote a` - delete note `a`

### Example config
```toml
[database]
type = "Sqlite"
path = "db.sqlite"

[telegram]
token = "bot token"
superchat = "staff superchat"
```

### TODO
- [ ] sync reactions between chats
- [ ] documentation