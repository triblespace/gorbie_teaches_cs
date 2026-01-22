use std::sync::{OnceLock, RwLock};

use egui::RichText;

use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

mod booleans;
mod expressions;
mod if_else;
mod loops;
mod overview;
mod state;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Chapter {
    Overview,
    Expressions,
    Booleans,
    State,
    IfElse,
    Loops,
}

static CURRENT_CHAPTER: OnceLock<RwLock<Chapter>> = OnceLock::new();

fn chapter_lock() -> &'static RwLock<Chapter> {
    CURRENT_CHAPTER.get_or_init(|| RwLock::new(Chapter::Overview))
}

pub fn current_chapter() -> Chapter {
    *chapter_lock().read().expect("chapter lock poisoned")
}

pub fn set_chapter(chapter: Chapter) {
    *chapter_lock().write().expect("chapter lock poisoned") = chapter;
}

pub fn chapter_selector(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            ui.label(RichText::new("Teaching notebooks").heading());
            ui.add_space(6.0);
            ui.label("Pick a chapter to open.");
            ui.add_space(6.0);

            let mut selection = current_chapter();
            let mut toggle = widgets::ChoiceToggle::new(&mut selection).small();
            toggle = toggle.choice(Chapter::Overview, "0");
            toggle = toggle.choice(Chapter::Expressions, "1");
            toggle = toggle.choice(Chapter::Booleans, "2");
            toggle = toggle.choice(Chapter::State, "3");
            toggle = toggle.choice(Chapter::IfElse, "4");
            toggle = toggle.choice(Chapter::Loops, "5");
            ui.add(toggle);

            if selection != current_chapter() {
                set_chapter(selection);
            }
        });
    });
}

pub fn overview(nb: &mut NotebookCtx) {
    overview::overview(nb);
}

pub fn expressions(nb: &mut NotebookCtx) {
    expressions::expressions(nb);
}

pub fn booleans(nb: &mut NotebookCtx) {
    booleans::booleans(nb);
}

pub fn state(nb: &mut NotebookCtx) {
    state::state(nb);
}

pub fn if_else(nb: &mut NotebookCtx) {
    if_else::if_else(nb);
}

pub fn loops(nb: &mut NotebookCtx) {
    loops::loops(nb);
}
