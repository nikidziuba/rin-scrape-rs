use std::path::Path;
use thirtyfour::prelude::*;
use dotenv;
use home;


mod scrape;
use scrape::{login, search, browser_init};

mod display;
use display::{show_info};

mod downloader;
use downloader::{check_update, ask_update, download_update};

mod config;
use config::{AppConfig, create_config};
// Clear the screen
async fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    // Clear the terminal
    clear().await;
    // std::env::set_var("RUST_BACKTRACE", "1");
    // Get the cli arguments and make sure that there are 2 at minimum
    let args: Vec<String> = std::env::args().into_iter().map(|x| x.to_string()).collect();


    // Check if config is available 
    let cfg_opt = AppConfig::from_file(Path::new("./app.dat"));
    let cfg = cfg_opt.clone().unwrap_or(AppConfig::empty());

    let (query, cfg_loaded) = match cfg_opt {
        Some(_) => {
            (cfg.app_id(), true)
        },
        None => {
            if args.len() < 2 {
        
                println!("Usage:\nrin-scraper {{query}}\nQuery is the keyword that you want to search with, SteamAppID is recommended");
                return Ok(());
            }
            (args[1].clone(), false)
        }
    };

    if query.to_lowercase() == "createconfig" {
        create_config();
        return Ok(())
    }
    
    


    // Create a temp path 
    let tmp_dir = Path::new("./TEMP");
    if !tmp_dir.exists() {
        std::fs::create_dir("./TEMP/").unwrap();
    }
    // Load vars from .env file
    dotenv::dotenv().ok();
    dotenv::from_path(home::home_dir().unwrap()).ok();


    // Load the username and password
    let name = std::env::var("RIN_NAME").expect("RIN_NAME is not set. You have to specify your username in .env file");
    let pswd = std::env::var("RIN_PASS").expect("RIN_PASS is not set. You have to specify your username in .env file");


    let (driver,mut selenium) = browser_init().await?;
    
    
    
    // Login
    login(&driver, &name, &pswd).await?;


    // Search for query
    let s_res = search(&driver, &query).await?;


    // Show info from the search result
    show_info(&s_res, tmp_dir).await?;

    // Check for updates
    if cfg_loaded {
        let update = check_update(&s_res, &cfg);
        if let Some(updt) = update {
            if ask_update(&updt) {
                download_update(&s_res, &updt, &driver).await;
                cfg.to_file(Path::new("./app.dat")).expect("Error while saving config to file: ");
            }
        }

    }

    // Quit the driver
    driver.quit().await?;
    // Kill the geckodriver thread
    selenium.kill()?;



    //Remove the temp dir
    std::fs::remove_dir_all(tmp_dir).unwrap();


    Ok(())
}

