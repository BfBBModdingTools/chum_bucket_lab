use druid::{
    im::vector,
    im::Vector,
    widget::{Button, Checkbox, Flex, Label, LineBreaking, List, Scroll},
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

#[derive(Data, Clone, Lens)]
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

fn ui_builder() -> impl Widget<AppState> {
    // build base panels
    let modlist_panel = Scroll::new(
        List::new(|| {
            Flex::row()
                .with_child(Checkbox::new("").lens(Mod::enabled))
                .with_child(Label::new(|data: &Mod, _env: &Env| data.name.clone()))
                .padding(LABEL_SPACING)
        })
        .lens(AppState::modlist),
    )
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
    let patch_button = Button::new(LocalizedString::new("Patch XBE"))
        .on_click(|_ctx, data: &mut AppState, _env| {
            // Temporarily cycle through mod list for info panel
            if let Some(index) = data.selected_mod {
                data.selected_mod = Some((index + 1) % data.modlist.len());
            } else {
                data.selected_mod = Some(0);
            }
        })
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
