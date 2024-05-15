# Configuration

The following content should be edited and saved to `fuuka-bot.toml` as you want.

```toml
# Admin user. (Optional)
admin-user = "@example:example.org"

[command]
# The command prefix.
prefix = "%%"

[matrix]
# Homeserver URL.
homeserver = "https://matrix.example.com"

# Service URLs.
[services]
# Hitokoto API.
hitokoto = "https://v1.hitokoto.cn"

[pixiv]
# Enable Pixiv support.
enabled = false
# Enable exporting R-18 content.
r18 = false
# The Pixiv PHPSESSID token.
token = "????????_XXXXXXXXXXXXXXXXXXXX"

# Room scoped traps.
[[pixiv.traps]]
rooms = ["!XXXXXXXXXXX:example.org"]
required-tags = []
target = ""

# Global traps.
[[pixiv.traps]]
required-tags = []
target = ""

[[features]]
room = ""
jerryxiao = true
fortune = false
pixiv = false
pixiv-r18 = false

[stickers]
# Where stickers are sent to.
send-to = "!XXXXXXXXXXX:example.org"

```

## Logging in

Run the bot, the bot should auto ask your credentials, note that **It is required for the homeserver to enable user/password login!**
