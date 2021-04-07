use druid::widget::{Flex, Label};
use druid::{AppLauncher, Color, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

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
    Flex::row()
        .with_flex_child(
            Label::new("Modlist").center().border(Color::WHITE, 1.0),
            3.0,
        )
        .with_spacer(10.0)
        .with_flex_child(
            Flex::column()
                .with_flex_child(
                    Label::new("Information").center().border(Color::WHITE, 1.0),
                    1.0,
                )
                .with_spacer(10.0)
                .with_child(Label::new("Patch XBE").center().border(Color::WHITE, 1.0)),
            2.0,
        )
        .padding(10.0)
}
