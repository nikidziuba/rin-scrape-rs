use std::path::Path;
use crate::scrape::{SteamInfo, SearchResult, LinkText, download_file};
use terminal_link::Link;
use chrono::{prelude::*};
use terminal_size::{terminal_size, Width, Height};
use std::path::PathBuf;
use termimage;
use image::GenericImageView;

// 2 hyperlinks and lenghts of texts
struct LinkPair {
    link1: String,
    len1: usize,
    link2: String,
    len2: usize
}

impl LinkPair {
    fn new(one: String, two: String, len1: usize, len2: usize) -> LinkPair{
        LinkPair { link1: one, link2: two, len1, len2 }
    }
    fn link1(&self) -> String { self.link1.clone() }
    fn link2(&self) -> String { self.link2.clone() }
    fn len1(&self) -> usize { self.len1.clone() }
    fn len2(&self) -> usize { self.len2.clone() }


}

// Convert epoch number to date in desired format
pub async fn epoch_to_date(time: String) -> String {
    let timestamp = time.parse::<i64>().unwrap();
    
    
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    

    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    datetime.format("%d.%m.%Y").to_string()
}



// Show image to the terminal
pub async fn show_image(url: &str, tmp_dir: &Path) {
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




// Create a table line from two strings
async fn table_line(s1: String, l1: usize, s2: String, l2: usize, width: usize, wall: String) -> String {
    
    // Padding used for centering
    let padding1 = (width - l1) / 2;
    let padding2 = (width - l2) / 2;

    // Prevention of wrong centering - i32 rounds to 0 in division
    let padding1_2 = match (width - l1) % 2 == 0{
        true => { padding1 },
        false => { padding1 + 1}
    };
    let padding2_2 = match (width - l2) % 2 == 0{
        true => { padding2 },
        false => { padding2 + 1}
    };
    // | s1 | s2 |
    let line = format!("{wall}{pad1}{s1}{pad1_2}{wall}{pad2}{s2}{pad2_2}{wall}",
        pad1 = " ".repeat(padding1),
        pad1_2 = " ".repeat(padding1_2),
        pad2 = " ".repeat(padding2), 
        pad2_2 = " ".repeat(padding2_2));

    line // return the line
} 
// Create and print a table from steam and download links
async fn link_table(steam_links: Vec<SteamInfo>, dl_links: Vec<LinkText>) {
    // Pairs of steam and download links
    let mut pairs: Vec<LinkPair> = Vec::new();

    match steam_links.len() == dl_links.len() {
        // If there is the same amount of links in both vectors ( most cases )
        true => {
            for i in 0..steam_links.len() {
                let st_text = format!("{} | {}", &steam_links[i].title(), epoch_to_date(steam_links[i].last_update()).await);
                let st_link = Link::new(&st_text, &steam_links[i].url()).to_string();
                pairs.push(LinkPair::new(st_link, dl_links[i].to_hyper().to_string(), st_text.len(), dl_links[i].text().len()));
            }
        }

        false => {// If not, check wich vector has more links and fill the other item in pair with " "
            match steam_links.len() > dl_links.len() {
                true => {
                    for i in 0..dl_links.len() {
                        let st_text = format!("{} | {}", &steam_links[i].title(), epoch_to_date(steam_links[i].last_update()).await);
                        let st_link = Link::new(&st_text, &steam_links[i].url()).to_string();
                        pairs.push(LinkPair::new(st_link, dl_links[i].to_hyper().to_string(), st_text.len(), dl_links[i].text().len()));

                    }
                    for i in dl_links.len()..steam_links.len() {
                        let st_text = format!("{} | {}", &steam_links[i].title(), epoch_to_date(steam_links[i].last_update()).await);
                        let st_link = Link::new(&st_text, &steam_links[i].url()).to_string();
                        pairs.push(LinkPair::new(st_link, " ".to_string(), st_text.len(), " ".len()));
                    }
                }
                false => {
                    for i in 0..steam_links.len() {
                        let st_text = format!("{} | {}", &steam_links[i].title(), epoch_to_date(steam_links[i].last_update()).await);
                        let st_link = Link::new(&st_text, &steam_links[i].url()).to_string();
                        pairs.push(LinkPair::new(st_link, dl_links[i].to_hyper().to_string(), st_text.len(), dl_links[i].text().len()));
                    }
                    for i in steam_links.len()..dl_links.len() {
                        pairs.push(LinkPair::new(" ".to_string(), dl_links[i].to_hyper().to_string(), " ".len(), dl_links[i].text().len()));
                    }
                }
            }
        }
    }

    // Table Creation
    let corner = "+".to_string();
    let floor = "─".to_string();
    let wall = "|".to_string();
    let width = terminal_size().unwrap().0.0 as usize / 2 - 1;
    // +───────+────────+
    let pause = format!("{corner}{ceil}{corner}{ceil}{corner}", ceil = floor.to_string().repeat(width)); 
    
    // Table headers
    let mut table = format!("{pause}\n{info}\n{pause}", 
        info = table_line("Steam Links".to_string(), "Steam Links".len(), "Download Links".to_string(), "Download Links".len(), width, wall.clone()).await
    );
    // Createa a line for each pair
    for pair in pairs {
        table = format!("{table}\n{next_line}",
        next_line = table_line(pair.link1(), pair.len1(), pair.link2(), pair.len2(), width, wall.clone()).await
        );
    }
    // Add a 'floor' at the end
    table = format!("{table}\n{pause}");

    //Print it
    println!("{}", table);


}


async fn center(text: &str, width: usize, len: usize) {
    // text: the text to print out
    // width: current width of the terminal
    // len: lenght of the text that you want to center, used to center the hyperlinks
    println!("{pad}{text}{pad}", pad = " ".repeat( ( ( width - len ) / 2 ) as usize ));
}


// Show info - this is the "main" function of file
pub async fn show_info(info: SearchResult, tmp_dir: &Path) -> Result<(), std::io::Error>{
    let steam_links = info.steam_links();
    let dl_links = info.dl_links();

    let width = terminal_size().unwrap().0.0 as usize;
    // Print the image
    show_image(&info.img_url(), tmp_dir).await;
    println!("");
    center(&info.thread_info().to_hyper(), width, info.thread_info().text().len()).await;
    println!("");
    link_table(steam_links, dl_links).await;




    Ok(())
}

