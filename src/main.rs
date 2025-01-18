use std::env;
use std::error::Error;
use config::AppConfig;
use open;

mod terminal;
mod config;
mod history;
mod repository;
mod watcher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // parse command line arguments first
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let command = &args[1];
        if command == "config" {
            let config_path = AppConfig::get_config_path()?;
            open::that(config_path.as_os_str())?;
            return Ok(());
        }
        println!("Usage: github-notify [config]");
        return Ok(());
    }

    // kick off the watcher
    let mut watcher = watcher::RepositoryWatcher::new()?;
    watcher.run().await?;

    Ok(())
}