use fuuka_bot::Config;

#[test]
fn config_test() -> anyhow::Result<()> {
    let config: Config = toml::from_str(r#"
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
enabled = false
r18 = false
token = "????????_XXXXXXXXXXXXXXXXXXXX"

[[pixiv.traps]]
rooms = ["!XXXXXXXXXXX:example.org"]
required-tags = []
target = ""

[[pixiv.traps]]
required-tags = []
target = ""

[[features]]
room = "!XXXXXXXXXXX:example.org"
jerryxiao = true
fortune = false
pixiv = false
pixiv-r18 = false

[stickers]
send-to = "!XXXXXXXXXXX:example.org"
"#)?;
    println!("Config: {config:#?}");
    Ok(())
}

#[test]
fn config_without_traps_test() -> anyhow::Result<()> {
    let config: Config = toml::from_str(r#"
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
enabled = false
r18 = false
token = "????????_XXXXXXXXXXXXXXXXXXXX"

[[features]]
room = "!XXXXXXXXXXX:example.org"
jerryxiao = true
fortune = false
pixiv = false
pixiv-r18 = false

[stickers]
send-to = "!XXXXXXXXXXX:example.org"
"#)?;
    println!("Config: {config:#?}");
    Ok(())
}
