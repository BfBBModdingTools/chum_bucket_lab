use druid::{
    im::vector,
    im::Vector,
    widget::{Button, Checkbox, Flex, Label, List, Scroll},
};
use druid::{AppLauncher, Color, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

#[derive(Clone, Data, Lens)]
struct AppState {
    modlist: Vector<Mod>,
}

#[derive(Data, Clone, Lens)]
struct Mod {
    enabled: bool,
    name: String,
}

impl Mod {
    fn new(name: impl Into<String>) -> Mod {
        Mod {
            enabled: false,
            name: name.into(),
        }
    }
}

const PANEL_SPACING: f64 = 10.0;
const LABEL_SPACING: f64 = 5.0;
const BG_COLOR: Color = Color::grey8(0xa0);

pub fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("bfbb_modloader").with_placeholder("BfBB Modloader"));

    let modlist = vector![
        Mod::new("No Autosave"),
        Mod::new("Auto CB"),
        Mod::new("Mod 3"),
        Mod::new("Mod 4"),
    ];

    let data = AppState { modlist: modlist };
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

    let modinfo_panel = Label::new(LocalizedString::new("Information"))
        .center()
        .border(Color::WHITE, 1.0)
        .background(BG_COLOR);

    // Patch button
    let patch_button = Button::new(LocalizedString::new("Patch XBE"))
        .on_click(|_ctx, data: &mut Vector<Mod>, _env| {
            // Temporarily add a new mod to the list for UI testing
            data.push_back(Mod::new("Test Mod"));
        })
        .expand_width()
        .lens(AppState::modlist);

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
