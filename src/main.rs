use std::path::{Path, PathBuf};
use std::{thread, time};
use thirtyfour::prelude::*;
use thirtyfour;
use std::process::{Command, Stdio};
use terminal_link::Link;
use termimage;
use std::io::Write;
use std::fs::File;
use image::GenericImageView;
use terminal_size::{Width, Height, terminal_size};
use dotenv::dotenv;

// A struct for storing link data (link:text)
struct LinkText {
    link: String,
    text: String
}
impl LinkText {
    fn new(link: &str, text: &str) -> LinkText {
        LinkText { link: link.to_string(), text: text.to_string() }
    }
    // fn link(&self) -> String { self.link.clone() }
    // fn text(&self) -> String { self.text.clone() }
}

// Info from steam store page, currently only title implemented
struct SteamInfo {
    title: String
}

impl SteamInfo {
    fn title(&self) -> String { self.title.clone() }
}

// Used for downloading a file, in this case image and saving it to TEMP dir 
async fn download_file(url: &str, tmp_dir: &Path) -> PathBuf {
    
    // Send a request for the image
    let image = reqwest::get(url).await.expect("URL doesn't exist");

    // Get file name of the image
    let fname = image
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

    // Full path of the downloaded image
    let img_path = tmp_dir.join(fname);




    // Create th file
    let mut dest = File::create(img_path.clone()).expect("Can't create file while downloading");
    // Get its contents from the link
    let content =  image.bytes().await.expect("Couldn't get bytes");
    // And write them to the file
    dest.write_all(&content).expect("Cant copy file to destination");

    return img_path //Return path of the file
}

// Show image to the terminal
async fn show_image(url: &str, tmp_dir: &Path) {
    // Download the image
    let image = download_file(&url, &tmp_dir).await; 

    let term_image = (String::new(), PathBuf::from(image)); 
    // Get format of the image
    let format = termimage::ops::guess_format(&term_image).expect("Couldnt find the file format");

    // Load the image to memory
    let loaded_img = termimage::ops::load_image(&term_image, format).expect("Couldn't load the file");

    // Get size of the terminal
    let term_size: (u32, u32) = {
        let size = terminal_size();
        if let Some((Width(w), Height(h))) = size {
            (w as u32, h as u32)
        } else {
            panic!("Unable to get terminal size")
        }
    };

    // Create a resized image to fit it in the terminal
    let img_s = termimage::ops::image_resized_size(loaded_img.dimensions(), term_size, true);
    let resized = termimage::ops::resize_image(&loaded_img, img_s);


    // Write it in true color
    termimage::ops::write_ansi_truecolor(&mut std::io::stdout(), &resized);
       

}



// Login to the site using creds from .env file
async fn login(driver: &WebDriver, username: &str, password: &str) -> WebDriverResult<()>{
    // Load the login page
    driver.goto("https://cs.rin.ru/forum/ucp.php?mode=login").await?;
    // Here there's a security check that creates important cookies, that's why geckodriver is used

    // Wait until it can find login form
    let mut login_form = driver.find(By::Name("username")).await; 
    while match login_form {
                Ok(_) => false,
                Err(_) => true} {
                    thread::sleep(time::Duration::from_secs_f32(0.2));
                    login_form = driver.find(By::Name("username")).await;
                }
    // Find the forms and Login button
    let login_form2 = login_form.unwrap();
    let pass_form = driver.find(By::Name("password")).await?;
    let login_button = driver.find(By::Name("login")).await?;

    // Enter login info and click the button
    login_form2.send_keys(username).await?;
    pass_form.send_keys(password).await?;
    thread::sleep(time::Duration::from_secs_f32(1.0));

    login_button.click().await?;
    
    Ok(())

}


// Clear the screen
async fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

// Print text to the terminal centered
async fn center(text: &str, width: usize, len: usize) {
    // text: the text to print out
    // width: current width of the terminal
    // len: lenght of the text that you want to center, used to center the hyperlinks
    println!("{}{}\n", " ".repeat( ( width - len  ) / 2 ) , text);
}

// Get information from steam store page
async fn steam_info(url: &str, driver: &WebDriver) -> Result<SteamInfo, WebDriverError> {
    driver.goto(url).await?;
    // Find the title
    let title_result = driver.find(By::XPath("//*[@id=\"appHubAppName\"]")).await;

    let title: String;
    // Check if the result is ok, when its not fallback to default text 
    match title_result {
        Ok(_) => title = title_result.unwrap().text().await?,
        Err(_) => title = "Steam Link".to_string()
    };

    // Go back to the previous page
    driver.back().await.unwrap();
    // Return the Steam info
    return Ok(SteamInfo {
        title
    });
  
}
// Search for the query
async fn search(driver: &WebDriver, query: &str, tmp_dir: &Path, width: usize) -> WebDriverResult<()>{
    //Search for the query in text from first post of threads on SCS forum (id 22)
    driver.goto(format!("https://cs.rin.ru/forum/search.php?keywords={}&terms=any&author=&fid%5B%5D=22&sc=1&sf=firstpost&sk=t&sd=d&sr=topics&st=0&ch=300&t=0&submit=Search", query)).await?;
    
    // Get the first thread and enter it
    let first = driver.find(By::XPath("/html/body/table/tbody/tr/td/div[2]/form/table[2]/tbody/tr[2]/td[3]/a")).await?;
    first.click().await?;

    thread::sleep(time::Duration::from_secs_f32(0.5));

    // Get the first post
    let post = driver.find(By::XPath("/html/body/table/tbody/tr/td/div[2]/div[2]/table[3]/tbody/tr[3]/td[2]/table/tbody/tr/td/div[1]")).await?;

    // Get its title,
    let title = driver.find(By::XPath("/html/body/table/tbody/tr/td/div[2]/div[1]/h2/a")).await?.text().await?;
    //  author,
    let author = driver.find(By::XPath("/html/body/table/tbody/tr/td/div[2]/div[2]/table[3]/tbody/tr[3]/td[1]/table/tbody/tr[1]/td")).await?.text().await?;
    //  and links
    let link_elems = post.find_all(By::Tag("a")).await?;


    let mut steam_links: Vec<String> = Vec::new();
    let mut dl_links: Vec<LinkText> = Vec::new();
    

    // for each extracted links
    for link in link_elems {
        let href = link.attr("href").await?.unwrap();

        // Get the steam links
        if href.contains("store.steampowered.com") {
            steam_links.push(href.clone());
        }
        // And the download links
        if href.contains("privatebin.rinuploads.org") || href.contains("drive.google.com") || href.contains("filecrypt.cc") {
            dl_links.push(LinkText::new(&href.clone(), &link.text().await?));
        }
    };


    
    // Chek if the author is a bot
    if author != "Upload-Crew [Bot]" {
        let text = format!("Post wasn't created by the bot, here's the link: {}", driver.current_url().await?);
        center(&text, width, text.len()).await;
        return Ok(());
    }
    // Get the url of the first image in post
    let img_url = post.find(By::Tag("img")).await?.attr("src").await?.expect("Couldnt find image url");

    //Print it to the terminal
    show_image(&img_url, tmp_dir).await;

    // Get the url of the thread
    let url = &driver.current_url().await?.to_string();
    // And create a hyperlink with the thread title as text
    let page_link: Link = Link::new(&title, url);
    // Print it out
    center(&page_link.to_string(), width as usize, title.len()).await;
    println!("");
    let text = "{ Steam Links }";
    center(text, width as usize, text.len()).await;
    println!("");
    // Print the steam links as hyperlinks
    for link in steam_links {
        let title = steam_info(&link, driver).await?.title();

        center(
            &Link::new(&title, &link).to_string(),
             width as usize,
              title.len()
            ).await;

    }

    println!("");
    let text = "Found download links:";
    center(text, width as usize, text.len()).await;
    // Print the download links as hyperlinks
    let mut n = 1;
    for link in dl_links {
        let text = format!("[{}] {}", n, link.text);
        center(
            &Link::new(&text, &link.link).to_string(),
            width as usize,
            text.len()
        ).await;
        n += 1;
    }
    


    Ok(())
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

    // Get the width of the terminal
    let width: u16 = terminal_size().unwrap().0.0;

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



    // Launch the geckodriver server on localhost
    let mut selenium = Command::new("geckodriver.exe")
        .stdout(Stdio::null())//disable output from the child
        .spawn()?;

    // Set the capabilities of Firefox
    let mut caps = DesiredCapabilities::firefox();
    caps.set_log_level(thirtyfour::common::capabilities::firefox::LogLevel::Fatal)?; // Disable non-fatal logs
    caps.set_headless()?; // Set as headless
    
    // Connect to the browser
    let driver = WebDriver::new("http://127.0.0.1:4444", caps).await?;
    
    // Login
    login(&driver, &name, &pswd).await?;

    // Search for query
    search(&driver, query, tmp_dir, width as usize).await?;

    // Quit the driver
    driver.quit().await?;
    // Kill the geckodriver thread
    selenium.kill()?;

    //Remove the temp dir
    std::fs::remove_dir_all(tmp_dir).unwrap();


    Ok(())
}
