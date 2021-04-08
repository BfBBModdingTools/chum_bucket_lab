use druid::{
    im::vector,
    im::Vector,
    widget::{Button, Checkbox, Flex, Label, LineBreaking, List, ListIter, Scroll},
};
use druid::{AppLauncher, Color, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

#[derive(Clone, Data, Lens)]
struct AppState {
    modlist: Vector<Mod>,
    selected_mod: Option<usize>,
}

impl AppState {
    fn new(modlist: Vector<Mod>) -> AppState {
        AppState {
            modlist,
            selected_mod: None,
        }
    }
}

// TOOD: Consider not needing PartialEq
#[derive(Data, Clone, PartialEq, Lens)]
struct Mod {
    enabled: bool,
    name: String,
    author: String,
    description: String,
}

impl Mod {
    fn new(name: impl Into<String>) -> Mod {
        Mod {
            enabled: false,
            name: name.into(),
            author: "".to_owned(),
            description: "".to_owned(),
        }
    }

    fn set_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    fn set_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

const PANEL_SPACING: f64 = 10.0;
const LABEL_SPACING: f64 = 5.0;
const BG_COLOR: Color = Color::grey8(0xa0);

pub fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("bfbb_modloader").with_placeholder("BfBB Modloader"));

    let modlist = vector![
        Mod::new("No Autosave").set_author("TheCoolSquare").set_description("Prevents Autosave functionality from ever being enabled. blah blah blahblah blah blahblah blah blah\n\nblah"),
        Mod::new("Auto CB").set_author("fusecv & daft7"),
        Mod::new("Mod 3"),
        Mod::new("Mod 4"),
    ];

    let data = AppState::new(modlist);
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

impl ListIter<(AppState, Mod)> for AppState {
    fn for_each(&self, mut cb: impl FnMut(&(AppState, Mod), usize)) {
        for (i, item) in self.modlist.iter().enumerate() {
            cb(&(self.clone(), item.to_owned()), i)
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut (AppState, Mod), usize)) {
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

impl Lens<(AppState, Mod), bool> for EnabledLens {
    fn with<R, F: FnOnce(&bool) -> R>(&self, data: &(AppState, Mod), f: F) -> R {
        f(&data.1.enabled)
    }

    fn with_mut<R, F: FnOnce(&mut bool) -> R>(&self, data: &mut (AppState, Mod), f: F) -> R {
        f(&mut data.1.enabled)
    }
}

fn ui_builder() -> impl Widget<AppState> {
    // build mod panel
    let modlist_panel = Scroll::new(List::new(|| {
        Flex::row()
            .with_child(Checkbox::new("").lens(EnabledLens))
            .with_child(
                Label::new(|(_, item): &(AppState, Mod), _env: &Env| item.name.clone()).on_click(
                    |_, (list, item): &mut (AppState, Mod), _| {
                        list.selected_mod = list.modlist.index_of(item);
                    },
                ),
            )
            .padding(LABEL_SPACING)
    }))
    .vertical()
    .border(Color::WHITE, 1.0)
    .expand()
    .background(BG_COLOR);

    // build information panel for selected mod
    let modinfo_panel = Label::new(|data: &AppState, _env: &Env| {
        if let Some(index) = data.selected_mod {
            if let Some(m) = data.modlist.get(index) {
                return format! {"Name: {}\nAuthor: {}\n\n{}", m.name, m.author, m.description};
            }
        }
        "".to_string()
    })
    .with_line_break_mode(LineBreaking::WordWrap)
    .expand()
    .padding(LABEL_SPACING)
    .border(Color::WHITE, 1.0)
    .background(BG_COLOR);

    // Patch button
    let patch_button = Button::new(LocalizedString::new("Patch XBE")).expand_width();

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
