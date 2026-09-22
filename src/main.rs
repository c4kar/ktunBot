use ktunbot::bot::build_dispatcher;
use ktunbot::cache::AppCache;
use ktunbot::commands::Command;
use ktunbot::config::Config;
use ktunbot::db::Db;
use ktunbot::services::Services;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting ktunBot...");

    let config = Config::from_env();

    let cache = AppCache::new();
    let db = Db::connect(&config).await?;
    let svc = Services::new(config, cache, db);

    let bot = Bot::from_env();
    bot.set_my_commands(Command::bot_commands()).await?;

    tracing::info!("Bot started, beginning dispatch...");
    let mut dispatcher = build_dispatcher(bot, svc);
    dispatcher.dispatch().await;

    Ok(())
}
