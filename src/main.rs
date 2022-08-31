use std::path::Path;
use thirtyfour::prelude::*;
use dotenv::dotenv;

mod scrape;
use scrape::{login, search, browser_init};

mod display;
use display::{show_info};

// Clear the screen
async fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}


#[tokio::main]
async fn main() -> WebDriverResult<()> {
    // Clear the terminal
    clear().await;
    
    // Get the cli arguments and make sure that there are 2 at minimum
    let args: Vec<String> = std::env::args().into_iter().map(|x| x.to_string()).collect();
    if args.len() < 2 {
        println!("Usage:\nrin-scraper {{query}}\nQuery is the keyword that you want to search with, SteamAppID is recommended");
        return Ok(());
    }
    let query: &str = &args[1];


    // Create a temp path 
    let tmp_dir = Path::new("./TEMP");
    if !tmp_dir.exists() {
        std::fs::create_dir("./TEMP/").unwrap();
    }
    // Load vars from .env file
    dotenv().ok();


    // Load the username and password
    let name = std::env::var("RIN_NAME").expect("RIN_NAME is not set. You have to specify your username in .env file");
    let pswd = std::env::var("RIN_PASS").expect("RIN_PASS is not set. You have to specify your username in .env file");


    let (driver,mut selenium) = browser_init().await?;
    
    
    
    // Login
    login(&driver, &name, &pswd).await?;


    // Search for query
    let s_res = search(&driver, query).await?;

    // Show info from the search result
    show_info(s_res, tmp_dir).await?;

    // Quit the driver
    driver.quit().await?;
    // Kill the geckodriver thread
    selenium.kill()?;

    //Remove the temp dir
    std::fs::remove_dir_all(tmp_dir).unwrap();


    Ok(())
}

