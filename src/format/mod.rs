use std::sync::LazyLock;

use minijinja::Environment;

pub mod pixiv;

// TODO: Use a bot created environment
pub static ENVIRONMENT: LazyLock<Environment> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.add_template(
        "[fuuka-bot]/templates/pixiv/illust.txt",
        self::pixiv::illust::text::default(),
    )
    .unwrap();
    env.add_template(
        "[fuuka-bot]/templates/pixiv/illust.html",
        self::pixiv::illust::html::default(),
    )
    .unwrap();
    env
});
