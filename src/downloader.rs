use regex::Regex;
use terminal_size::terminal_size;
use thirtyfour::{WebDriver, By};
use crate::{scrape::SearchResult, config::{AppConfig, get_jd_path}, display::{center, update_table, epoch_to_date, get_input}};
use chrono::NaiveDate;
use std::{process, time::Duration};
use std::thread;


// Version Struct
#[derive(Clone)]
pub struct Version {
	title: String,
	last_update: String
}
impl Version {
	pub fn title(&self) -> String { self.title.clone() }
	pub fn last_update(&self) -> String { self.last_update.clone() }

}

// Update Struct

pub struct Update {
	from: Version,
	to: Version
}
impl Update {
	pub fn from(&self) -> Version { self.from.clone() }
	pub fn to(&self) -> Version { self.to.clone() }

}

// Convert date from dd.mm.YYYY to epoch 
fn date_to_epoch(date: String) -> String {
	let datetime = NaiveDate::parse_from_str(&date, "%d.%m.%Y").unwrap();
	let epoch = datetime.and_hms(0, 0, 0).timestamp().to_string();
	epoch
}

// Get links from privatebin
async fn get_privatebin(link: String, driver: &WebDriver) -> Vec<String> {
	let mut v: Vec<String> = Vec::new();

	driver.goto(link).await.unwrap();
	thread::sleep(Duration::from_secs_f32(1.0));

	let input = driver.find(By::Id("passworddecrypt")).await.unwrap();
	let mut button_opt = driver.find(By::XPath("/html/body/div[1]/div/div/div/form/button")).await;

    while match button_opt {
                Ok(_) => false,
                Err(_) => true} {
                    thread::sleep(Duration::from_secs_f32(0.2));
                    button_opt = driver.find(By::Name("btn btn-success btn-block")).await;
                }

	let button = button_opt.unwrap();
	input.send_keys("cs.rin.ru").await.unwrap();
	button.click().await.unwrap();

	let mut text_opt = driver.find(By::XPath(r#"//*[@id="prettyprint"]"#)).await;

    while match text_opt {
                Ok(_) => false,
                Err(_) => true} {
                    thread::sleep(Duration::from_secs_f32(0.2));
                    text_opt = driver.find(By::XPath(r#"//*[@id="prettyprint"]"#)).await;
                }

	let text = text_opt.unwrap();


	let links = text.find_all(By::Tag("a")).await.unwrap();

	

	for i in links {
		v.push(i.attr("href").await.unwrap_or(Some("".to_string())).unwrap());
	}

	v
}

// Check if link with last game title has changed last update date and create an Update Struct if so
pub fn check_update(sr: &SearchResult, cfg: &AppConfig) -> Option<Update> {

    let last_title = cfg.last_update_title();
	let last_date = date_to_epoch(cfg.last_update_str());

	let re = regex::Regex::new(r#"(?P<title>[[:ascii:]]+) \| (?P<date>[[:digit:]]{2}\.[[:digit:]]{2}\.[[:digit:]]{4})"#).unwrap();

	for i in sr.dl_links() {
		let text = i.text();

		let result = re.captures(&text).unwrap();
		let title_match =  result.name("title");
		let title = match title_match {
			Some(_) => { title_match.unwrap().as_str().to_string() }
			None => {"".to_string() } 
		};

		let date_match =  result.name("date");
		let date = match date_match {
			Some(_) => { date_to_epoch(date_match.unwrap().as_str().to_string()) }
			None => { "".to_string() }
		};
		if title == last_title && date != last_date {
			return Some(
				Update{
					from: Version { title: last_title, last_update: last_date },
					to: Version { title: title, last_update: date }
				}
			);
		}

	}
	return None;
	
}


// Ask user about updating the game
pub fn ask_update(info: &Update) -> bool{
	let txt = "There's an Update Available";
	center(txt, terminal_size::terminal_size().unwrap().0.0.into(), txt.len());
	println!("");
	println!("{}", update_table(info));

	loop {
		let ans = get_input("Do you want to download the update? y/n: ");
		match ans.to_lowercase().as_str() {
			"y" => { return true },
			"n" => { return false },
			s => { println!("Invalid response: '{s}', try again.")}
		}
	}
}
// Parse and download links, currently using JDownloader 2
pub async fn download_update(res: &SearchResult, update: &Update, driver: &WebDriver) {
	let dl_title = format!("{} | {}", update.to().title(), epoch_to_date(update.to().last_update()));
	println!("Updating: {}", dl_title);
	let mut dl_link = "".to_string();

	for link in res.dl_links() {
		if link.text() == dl_title {
			dl_link = link.link();
		}
	}



	let unsupported = vec![
		"filecrypt.cc"
	];
	let domain_re = Regex::new(r"(?i)^(?:https?://)?(?:[^@/\n]+@)?(?:www\.)?([^:/?\n]+)").unwrap();



	let domain_opt = domain_re.captures(&dl_link);
	let domain = match domain_opt {
		None => {
			let text = format!("Invalid link: {}", dl_link);
			center(&text, terminal_size().unwrap().0.0.into(), text.len());
			return ()
		}
		Some(x) => x.get(1).unwrap().as_str()
	};
	let mut dl_links: Vec<String> = Vec::new();
	if unsupported.contains(&domain) {
		println!("We don't support {domain} at this time, here's the link: {dl_link}");
		return ()
	}
	else if domain == "privatebin.rinuploads.org" {
		dl_links = get_privatebin(dl_link, driver).await;
	}
	else {
		dl_links = vec![dl_link];
	}
	
	
	dl_links.insert(0, "-add-link".to_string());
	
	let jd = get_jd_path();

	let _ = process::Command::new(jd)
		.args(dl_links)
		.spawn()
		.unwrap();

	



}