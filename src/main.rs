pub mod data;
pub mod ui;

use druid::{im::Vector, AppLauncher, LocalizedString, WindowDesc};

use data::{AppState, Mod};
use std::fs;
use std::io::Write;

const WINDOW_TITLE: &str = "bfbb_modloader";

pub fn main() {
    let main_window = WindowDesc::new(ui::ui_builder)
        .title(LocalizedString::new(WINDOW_TITLE).with_placeholder("BfBB Modloader"));

    match reqwest::blocking::get(data::URL_MODLIST) {
        Err(_) => println!("Failed to retrieve modslist from internet"), // TODO: Interent connectivity error
        Ok(response) => match response.text() {
            Err(_) => println!("Failed to convert HTTP response to text"), // TODO: Not sure when this happens
            Ok(modlist_json) => match fs::OpenOptions::new()
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

    // TODO: Consider if saving the modlist locally is even necessary
    // Note: Keeping a local copy enables the app to still function even
    // if we fail to download latest version even though the user has a valid
    // internet connection
    match fs::File::open(data::PATH_MODLIST) {
        Err(_) => println! {"Failed to open file"}, // TODO: Failed to open file error
        Ok(file) => {
            match serde_json::from_reader::<_, Vec<Mod>>(file) {
                Err(_) => println!("File corrupted"), // TODO: Failed to deserialize file error
                Ok(modlist) => {
                    AppLauncher::with_window(main_window)
                        .use_simple_logger()
                        .launch(AppState::new(Vector::from(modlist)))
                        .expect("launch failed");
                }
            }
        }
    }
}
