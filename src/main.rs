use druid::widget::{Align, Label};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WindowDesc};

#[derive(Clone, Default, Data, Lens)]
struct AppState {}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("bfbb_modloader").with_placeholder("BfBB Modloader"));
    let data = AppState::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppState> {
    let label = Label::new("Placeholder Text");

    Align::centered(label)
}
