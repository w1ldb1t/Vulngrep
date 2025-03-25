use crate::repository::GithubRepositoryError;
use crate::terminal::TerminalDisplay;
use config::AppConfig;
use open;
use std::env;
use std::error::Error;
use std::rc::Rc;

mod config;
mod history;
mod repository;
mod terminal;
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
        println!("Usage: vulngrep [config]");
        return Ok(());
    }

    let display = Rc::new(TerminalDisplay::new());

    // kick off the watcher
    let mut watcher = watcher::RepositoryWatcher::new(display.clone())?;
    match watcher.run().await {
        Ok(_) => (),
        Err(e)
            if e.downcast_ref::<GithubRepositoryError>()
                == Some(&GithubRepositoryError::InvalidToken) =>
        {
            display.display_error(e.to_string().as_str());
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
