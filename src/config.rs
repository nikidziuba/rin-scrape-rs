use std::{path::Path, io::Error};
use which;
use serde::{Serialize, Deserialize};

use crate::display::get_input;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    app_id : String,
    path: String,
    last_update: String, // epoch
    last_update_title: String, //title
    last_update_str: String // dd.mm.YYYY
}

impl AppConfig {
    pub fn app_id(&self) -> String { self.app_id.clone() }
    pub fn path(&self) -> String { self.path.clone() }
    pub fn last_update(&self) -> String { self.last_update.clone() }
    pub fn last_update_title(&self) -> String { self.last_update_title.clone() }
    pub fn last_update_str(&self) -> String { self.last_update_str.clone() }

    pub fn empty() -> AppConfig {
        AppConfig { app_id: "".to_string(), path: "".to_string(), last_update: "".to_string(), last_update_title: "".to_string(), last_update_str: "".to_string() }
    }
    pub fn from_file(path: &Path) -> Option<AppConfig> {
        if !path.exists() {
            return None;
        }
        let f = std::fs::File::open(path).unwrap();

        let loaded: AppConfig = serde_json::from_reader(f).unwrap();


        Some(loaded)
    }
    pub fn to_file(&self, path: &Path) -> Result<(), Error> {
        println!("{:?}", path.parent().unwrap());
        std::fs::create_dir_all(path.parent().unwrap())?;
        let f = std::fs::File::create(path)?;
        
        serde_json::to_writer_pretty(f, &self)?;


        Ok(())
    }
    pub fn new(app_id : String, path: String, last_update: String, last_update_title: String, last_update_str: String ) -> AppConfig {
        AppConfig { app_id, path, last_update, last_update_title, last_update_str }
    }
}


pub fn get_jd_path() -> String {
    // check for JD2_HOME in Env Vars
    let path = std::env::var("JD2_HOME");

    match path {
        Ok(x) => { 
            let jd = Path::new(&x);
            return jd.join("JDownloader2.exe").to_str().unwrap().to_string();
         },
        Err(_) => {}
    }

    //use which crate to find the binary
    let res = which::which("JDownloader2.exe");

    match res {
        Ok(x) => { return x.to_str().unwrap().to_string() },
        Err(_) => {}
    }

    // Or ust get user input
    let user_out = &get_input("Couldn't find JDownloader. Please enter its executable's path: ");
    let mut user_path = Path::new(user_out);

    while !user_path.exists() {
        println!("Path doesn't exist!");
        user_path = Path::new(user_out);

    }
    if user_path.is_dir() {
        let user_res = which::which_in("JDownloader2.exe", Some(user_path), std::env::current_dir().unwrap());

        match user_res {
            Ok(x) => { return x.to_str().unwrap().to_string() },
            Err(_) => {}
        }

    }
    user_path.to_str().unwrap().to_string()
}

// Create a config file for game and save it to a file
pub fn create_config() {
    let app_id = get_input("Steam AppId: ");
    let path = get_input("Absolute Path: ").replace("/", "\\");
    let last_update = "0".to_string();
    println!("SCS Format: \"{{Title}} | {{Last Update}}\"");
    let last_update_title = get_input("SCS Title: ");
    let last_update_str = "01.01.1970".to_string();

    let cfg = AppConfig::new(app_id, path.clone(), last_update, last_update_title, last_update_str);


    let file = Path::new(&path).join("app.dat");

    match cfg.to_file(file.as_path()) {
        Ok(_) => {},
        Err(x) => {
            println!("Error while creating file: {}", x);
            println!("Creating file in current directory");
            let cwd = std::env::current_dir().unwrap();
            let file = Path::new(&cwd).join("app.dat");
            cfg.to_file(file.as_path()).unwrap();
        }
    }

}