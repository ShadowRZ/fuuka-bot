# Configuration

The following content should be edited and saved to `fuuka-bot.toml` as you want.

```toml
# The command prefix.
command_prefix = "%%"
# Homeserver URL.
homeserver_url = "https://matrix.example.com"

# Service URLs. (Optional)
[services]
# Hitokoto API. (Optional)
hitokoto = "https://v1.hitokoto.cn"

# Features
# All fields are optional.
[features."!XXXXXXXXXXX:example.org"]
jerryxiao = true

# Configures sticker functions. (Optional)
[stickers]
# Room for uploading stickers. (Required)
sticker_room = "!XXXXXXXXXXX:example.org"
```

## Logging in

Run the bot, the bot should auto ask your credentials, note that **It is required for the homeserver to enable user/password login!**
