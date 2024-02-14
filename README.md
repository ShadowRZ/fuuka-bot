# Fuuka Bot

## User Agent

The bot consistently uses the following user agent template:

```text
fuuka-bot/<version> (https://github.com/ShadowRZ/fuuka-bot)
```

Where `<version>` is the running version of the bot.

## Usage

Copy `fuuka-bot.sample.toml` to `fuuka-bot.toml` and edit this file to your needs.

Then:

```
# Login to the homeserver and writes a credential.json file.
cargo run -- login
# Run the bot.
cargo run
```