use druid::widget::{Button, Checkbox, Flex, Label, LineBreaking, List, ListIter, Scroll};
use druid::{Color, Data, Env, Lens, LocalizedString, Widget, WidgetExt};

use crate::data::{AppState, Mod};

const PANEL_SPACING: f64 = 10.0;
const LABEL_SPACING: f64 = 5.0;
const BG_COLOR: Color = Color::grey8(0xa0);

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

pub fn ui_builder() -> impl Widget<AppState> {
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
