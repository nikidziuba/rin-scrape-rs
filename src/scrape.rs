use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Write};
use std::{thread, time};
use thirtyfour::{*, prelude::*};
use std::process::{Command, Stdio, Child};
use serde_json::{Value};
use regex;
use terminal_link::Link;

// Info from steam store page, currently only title implemented

#[derive(Debug, Clone)]
pub struct SteamInfo {
    title: String,
    last_update: String,
    url: String
}

impl SteamInfo {
    pub fn title(&self) -> String { self.title.clone() }
    pub fn last_update(&self) -> String { self.last_update.clone() }
    pub fn url(&self) -> String { self.url.clone()}

}

// Remove first and last character from string
fn strip_f_l(value: String) -> String {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str().to_string()
}

pub struct SearchResult {
    thread_info: LinkText,
    author:  String,
    img_url: String,
    steam_links: Vec<SteamInfo>,
    dl_links: Vec<LinkText>

}
impl SearchResult {
    pub fn new(thread_info: LinkText, author:  String, img_url: String, steam_links: Vec<SteamInfo>, dl_links: Vec<LinkText>) -> SearchResult {
        SearchResult { thread_info, author, img_url, steam_links, dl_links }
    }

    pub fn thread_info(&self) -> LinkText { self.thread_info.clone() }
    pub fn author(&self) -> String { self.author.clone() }
    pub fn img_url(&self) -> String { self.img_url.clone() }
    pub fn steam_links(&self) -> Vec<SteamInfo> { self.steam_links.clone() }
    pub fn dl_links(&self) -> Vec<LinkText> { self.dl_links.clone() }

}




// A struct for storing link data (link:text)
#[derive(Clone)]
pub struct LinkText {
    link: String,
    text: String
}
impl LinkText {
    pub fn new(link: &str, text: &str) -> LinkText {
        LinkText { link: link.to_string(), text: text.to_string() }
    }
    pub fn link(&self) -> String { self.link.clone() }
    pub fn text(&self) -> String { self.text.clone() }
    pub fn to_hyper(&self) -> String {
        Link::new(&self.text(), &self.link()).to_string()
    }
}


// Initialize the WebDriver
pub async fn browser_init() -> WebDriverResult<(WebDriver, Child)> {
    let selenium = Command::new("geckodriver.exe")
        .stdout(Stdio::null())//disable output from the child
        .spawn()?;

    // Set the capabilities of Firefox
    let mut caps = DesiredCapabilities::firefox();
    caps.set_log_level(thirtyfour::common::capabilities::firefox::LogLevel::Fatal)?; // Disable non-fatal logs
    // caps.set_headless()?; // Set as headless
    caps.add("acceptInsecureCerts", true)?;
    
    // Connect to the browser
    let driver = WebDriver::new("http://127.0.0.1:4444", caps).await?;
    Ok((driver, selenium))
}



// Login to the site using creds from .env file
pub async fn login(driver: &WebDriver, username: &str, password: &str) -> WebDriverResult<()>{
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

// Search for the query
pub async fn search(driver: &WebDriver, query: &str) -> WebDriverResult<SearchResult>{
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

    // Get the url of the thread
    let url = &driver.current_url().await?.to_string();

    // And create a hyperlink with the thread title as text
    let thread_info: LinkText = LinkText::new(url, &title);



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


    
    
    // Get the url of the first image in post
    let img_url = post.find(By::Tag("img")).await?.attr("src").await?.expect("Couldnt find image url");

 

    
    let mut steam: Vec<SteamInfo> = Vec::new();

    for link in steam_links {
        let info = steam_info(&link).await?;
        steam.push(info);
    }





    Ok(SearchResult::new(thread_info, author, img_url, steam, dl_links))
}




// Used for downloading a file and saving it to TEMP dir 
pub async fn download_file(url: &str, tmp_dir: &Path) -> PathBuf {

    
    // Send a request for the file
    let file = reqwest::get(url).await.expect("URL doesn't exist");

    // Get file name of the file
    let fname = file
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

    // Full path of the downloaded file
    let file_path = tmp_dir.join(fname);




    // Create th file
    let mut dest = File::create(file_path.clone()).expect("Can't create file while downloading");
    // Get its contents from the link
    let content =  file.bytes().await.expect("Couldn't get bytes");
    // And write them to the file
    dest.write_all(&content).expect("Cant copy file to destination");

    return file_path //Return path of the file
}






// Get info from steam page
async fn steam_info(url: &str) -> Result<SteamInfo, std::io::Error> {

    let reg = regex::Regex::new(r"https://store\.steampowered\.com/app/([[:digit:]]+)*").unwrap();
    let appid =  &reg.captures(url).unwrap()[1];


    let res = reqwest::get(format!("http://api.steamcmd.net/v1/info/{}", appid)).await.expect("Couldnt get response from SteamCMD");

    let parsed: Value = serde_json::from_str(&res.text().await.unwrap())?;



    let title = strip_f_l(parsed["data"][appid]["common"]["name"].to_string());  
    
    let mut last_update = "0".to_string();


    for branches in parsed
            ["data"]
            [appid]
            ["depots"]
            ["branches"].as_object() {  
                for value in branches.values(){
                    let db_time = strip_f_l(value["timeupdated"].to_string()).parse::<i32>();
                    let time: i32 = match db_time {
                        Ok(res) => res,
                        Err(_) => 0
                    };
                    
                    if time > last_update.parse::<i32>().unwrap() {
                        last_update = time.to_string();
                    }
                }
    }
    let info = SteamInfo {
        title,
        last_update: last_update,
        url: url.to_string()
    };
    
    Ok(info)
}

