use druid::{
    im::vector,
    im::Vector,
    widget::{Button, Flex, Label, List, Scroll},
};
use druid::{AppLauncher, Color, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

#[derive(Clone, Default, Data, Lens)]
struct AppState {
    modlist: Vector<Mod>,
}

#[derive(Default, Data, Clone)]
struct Mod {
    name: String,
}

const PANEL_SPACING: f64 = 10.0;
const LABEL_SPACING: f64 = 5.0;
const BG_COLOR: Color = Color::grey8(0xa0);

pub fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("bfbb_modloader").with_placeholder("BfBB Modloader"));

    let modlist = vector![
        Mod {
            name: "No Autosave".to_string()
        },
        Mod {
            name: "Auto CB".to_string()
        },
        Mod {
            name: "Mod 3".to_string()
        },
        Mod {
            name: "Mod 4".to_string()
        },
    ];

    let data = AppState { modlist: modlist };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppState> {
    // build base panels
    let modlist_panel = Scroll::new(List::new(|| {
        Label::new(|item: &Mod, _env: &_| item.name.clone()).padding(LABEL_SPACING)
    }))
    .vertical()
    .border(Color::WHITE, 1.0)
    .expand()
    .background(BG_COLOR)
    .lens(AppState::modlist);

    let modinfo_panel = Label::new(LocalizedString::new("Information"))
        .center()
        .border(Color::WHITE, 1.0)
        .background(BG_COLOR);

    // Patch button
    let patch_button = Button::new(LocalizedString::new("Patch XBE"))
        .on_click(|_ctx, data: &mut Vector<Mod>, _env| {
            // Temporarily add a new mod to the list for UI testing
            data.push_back(Mod {
                name: "Test Mod".to_string(),
            });
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
