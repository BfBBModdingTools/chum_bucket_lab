use druid::{
    widget::{Button, Checkbox, Flex, Label, LineBreaking, List, ListIter, Scroll},
    AppDelegate, DelegateCtx, EventCtx, Handled,
};
use druid::{
    Color, Command, Data, Env, FileDialogOptions, FileSpec, Lens, LocalizedString, Target, Widget,
    WidgetExt,
};

use crate::data::{self, PATH_ROM};
use crate::data::{AppData, Mod, Patch, Rom};

const PANEL_SPACING: f64 = 10.0;
const LABEL_SPACING: f64 = 5.0;
const BG_COLOR: Color = Color::grey8(0x80);

impl ListIter<(AppData, Mod)> for AppData {
    fn for_each(&self, mut cb: impl FnMut(&(AppData, Mod), usize)) {
        for (i, item) in self.modlist.iter().enumerate() {
            cb(&(self.clone(), item.to_owned()), i)
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut (AppData, Mod), usize)) {
        let mut new_data = Vec::new();
        let mut self_clone = self.clone();

        for (i, item) in self.modlist.iter_mut().enumerate() {
            let mut data = (self_clone.clone(), item.clone());
            cb(&mut data, i);

            if !data.0.selected_mod.same(&self_clone.selected_mod) {
                self_clone = data.0;
            }

            if !data.1.same(item) {
                new_data.push((data.1, i));
            }
        }

        self.clone_from(&self_clone);
        for (m, i) in new_data.iter() {
            self.modlist.get_mut(*i).unwrap().enabled = m.enabled;
        }
    }

    fn data_len(&self) -> usize {
        self.modlist.data_len()
    }
}

struct EnabledLens;

impl Lens<(AppData, Mod), bool> for EnabledLens {
    fn with<R, F: FnOnce(&bool) -> R>(&self, data: &(AppData, Mod), f: F) -> R {
        f(&data.1.enabled)
    }

    fn with_mut<R, F: FnOnce(&mut bool) -> R>(&self, data: &mut (AppData, Mod), f: F) -> R {
        f(&mut data.1.enabled)
    }
}

pub fn ui_builder() -> impl Widget<AppData> {
    // build mod panel
    let modlist_panel = Scroll::new(List::new(|| {
        Flex::row()
            .with_child(Checkbox::new("").lens(EnabledLens))
            .with_child(
                Label::new(|(_, item): &(AppData, Mod), _env: &Env| item.name.clone()).on_click(
                    |_, (list, item): &mut (AppData, Mod), _| {
                        list.selected_mod = list.modlist.index_of(item);
                    },
                ),
            )
            .padding(LABEL_SPACING)
    }))
    .vertical()
    .on_click(|_, data: &mut AppData, _| {
        // TODO: This is called when a label is clicked,
        // but luckily that occurs after this.
        data.selected_mod = None;
    })
    .border(Color::WHITE, 1.0)
    .expand()
    .background(BG_COLOR);

    // build information panel for selected mod
    let modinfo_panel = Label::new(|data: &AppData, _env: &Env| {
        if let Some(index) = data.selected_mod {
            if let Some(m) = data.modlist.get(index) {
                return format! {"Name: {}\nAuthor: {}\n\n{}", m.name, m.author, m.description};
            }
        }
        data.response.to_owned()
    })
    .with_line_break_mode(LineBreaking::WordWrap)
    .expand()
    .padding(LABEL_SPACING)
    .border(Color::WHITE, 1.0)
    .background(BG_COLOR);

    // Patch button
    let patch_button = Button::new(LocalizedString::new("Patch XBE"))
        .on_click(patch_button_on_click)
        .expand_width();

    // Arrange panels
    Flex::row()
        .with_flex_child(modlist_panel, 3.0)
        .with_spacer(PANEL_SPACING)
        .with_flex_child(
            Flex::column()
                .with_flex_child(modinfo_panel, 1.0)
                .with_spacer(PANEL_SPACING)
                .with_child(patch_button),
            2.0,
        )
        .padding(PANEL_SPACING)
}

fn set_response(data: &mut AppData, response: impl Into<String>) {
    let response = response.into();
    println!("RESPONSE: {}", response);

    data.selected_mod = None;
    data.response = response;
}

fn patch_button_on_click(ctx: &mut EventCtx, data: &mut AppData, _: &Env) {
    if !std::path::Path::new(data::PATH_ROM).is_file() {
        let types = vec![FileSpec::new("Xbox Executable", &["xbe"])];
        let options = FileDialogOptions::new()
            .allowed_types(types)
            .button_text("Import")
            .title("Import Clean ROM");

        ctx.submit_command(Command::new(
            druid::commands::SHOW_OPEN_PANEL,
            options,
            Target::Auto,
        ));
        return;
    }

    apply_enabled_mods(data);
}

fn apply_enabled_mods(data: &mut AppData) {
    let modlist = data.modlist.clone();
    let enabled_mods = modlist.iter().filter(|i| i.enabled).collect::<Vec<&Mod>>();

    if enabled_mods.is_empty() {
        set_response(data, "No mods selected");
        return;
    }

    match Rom::new() {
        Err(e) => set_response(data, e.to_string()),
        Ok(mut rom) => {
            for m in enabled_mods {
                // Download Patch
                match m.download() {
                    Err(_) => {
                        set_response(data, format!("Failed to download {}", m.name));
                    }
                    Ok(patch_bytes) => {
                        let mut patch = Patch::new(patch_bytes);
                        match patch.apply_to(&mut rom) {
                            Err(_) => {
                                set_response(data, format!("Failed to apply {}", m.name));
                            }
                            Ok(_) => (),
                        }
                    }
                }
            }

            // Write out modified rom
            match rom.export() {
                Err(_) => set_response(data, "Failed to export patched rom!"),
                Ok(_) => set_response(data, "Successfully patched ROM."),
            }
        }
    }
}

pub struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppData,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(druid::commands::OPEN_FILE) {
            if let Err(_) = std::fs::create_dir_all("baserom") {
                set_response(data, "Failed to make baserom directory");
                return Handled::Yes;
            }
            if let Err(_) = std::fs::copy(file_info.path(), data::PATH_ROM) {
                set_response(data, "Failed to copy rom");
                return Handled::Yes;
            }

            if let Ok(bytes) = std::fs::read(data::PATH_ROM) {
                if Rom::verify_hash(&bytes) != true {
                    set_response(data, "The imported file is not correct.");
                    let _ = std::fs::remove_file(PATH_ROM);
                } else {
                    apply_enabled_mods(data);
                }
            }
            return Handled::Yes;
        }

        Handled::No
    }
}
