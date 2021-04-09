pub mod data;
pub mod ui;

use druid::{im::Vector, AppLauncher, LocalizedString, WindowDesc};

use data::{AppState, Mod};
use std::fs;
use std::io::Write;

pub fn main() {
    let main_window = WindowDesc::new(ui::ui_builder)
        .title(LocalizedString::new("bfbb_modloader").with_placeholder("BfBB Modloader"));

    if let Ok(response) = reqwest::blocking::get(
        "https://raw.githubusercontent.com/SquareMan/bfbb_modloader/master/mods.json",
    ) {
        if let Ok(modlist_json) = response.text() {
            if let Ok(mut file) = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("mods.json")
            {
                match file.write_all(modlist_json.as_bytes()) {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
        }
    }

    // FIXME: More robust file I/O, any file error will currently result in a panic
    let modlist: Vec<Mod> = serde_json::from_reader(fs::File::open("mods.json").unwrap()).unwrap();
    let data = AppState::new(Vector::from(modlist));

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}
