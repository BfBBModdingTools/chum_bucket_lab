pub mod data;
pub mod linker;
pub mod ui;

use druid::{im::Vector, AppLauncher, LocalizedString, WindowDesc};

use data::{AppData, Mod};
use std::io::{Error, ErrorKind, Write};
use std::{env, fs};

const WINDOW_TITLE: &str = "bfbb_modloader";

#[derive(Clone, Debug)]
struct Config {
    check_update: bool,
}
impl Config {
    const DEFAULT_CONFIG: Config = Config { check_update: true };
    const OPTION_UPDATE: &'static str = "--update";

    fn new(args: &[String]) -> Self {
        if args.len() < 3 {
            return Config::DEFAULT_CONFIG.to_owned();
        }

        if &args[1] != Config::OPTION_UPDATE {
            return Config::DEFAULT_CONFIG.to_owned();
        }

        match &args[2].parse::<bool>() {
            Err(_) => Config::DEFAULT_CONFIG.to_owned(),
            Ok(b) => Config { check_update: *b },
        }
    }
}

pub fn main() {
    //TEMPORARY
    let mut xbe = linker::load_xbe(std::fs::File::open("baserom/default.xbe").unwrap()).unwrap();
    let _ = linker::add_test_section(&mut xbe);
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output/default.xbe")
        .unwrap();
    let _ = output.write(&xbe.serialize().unwrap());

    // Get config from command line args
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    let main_window = WindowDesc::new(ui::ui_builder)
        .title(LocalizedString::new(WINDOW_TITLE).with_placeholder("BfBB Modloader"));

    if config.check_update {
        update_modlist();
    }

    //TODO: Error prompt when this fails
    let modlist = match parse_modlist() {
        Err(_) => {
            println!("Failed to parse modlist");
            Vec::new()
        }
        Ok(list) => list,
    };

    AppLauncher::with_window(main_window)
        .delegate(ui::Delegate)
        .use_simple_logger()
        .launch(AppData::new(Vector::from(modlist)))
        .expect("launch failed");
}

fn update_modlist() {
    match reqwest::blocking::get(data::URL_MODLIST) {
        Err(_) => println!("Failed to retrieve modslist from internet"), // TODO: Interent connectivity error
        Ok(response) => match response.text() {
            Err(_) => println!("Failed to convert HTTP response to text"), // TODO: Not sure when this happens
            Ok(modlist_json) => match fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(data::PATH_MODLIST)
            {
                Err(_) => println!("Failed to write updated file to disk"), // TODO: File access error
                Ok(mut file) => match file.write_all(modlist_json.as_bytes()) {
                    Ok(_) => (),
                    Err(_) => (),
                },
            },
        },
    }
}

fn parse_modlist() -> std::io::Result<Vec<Mod>> {
    // TODO: Consider if saving the modlist locally is even necessary
    // Note: Keeping a local copy enables the app to still function even
    // if we fail to download latest version even though the user has a valid
    // internet connection
    let file = fs::File::open(data::PATH_MODLIST)?;

    match serde_json::from_reader::<_, Vec<Mod>>(file) {
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)), //Failed to deserialize file
        Ok(modlist) => Ok(modlist),
    }
}
