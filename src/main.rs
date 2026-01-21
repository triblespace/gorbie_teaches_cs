use GORBIE::prelude::*;

mod chapters;

#[notebook]
fn main(nb: &mut NotebookCtx) {
    let selection = chapters::current_chapter();
    chapters::chapter_selector(nb);

    match selection {
        chapters::Chapter::Overview => chapters::overview(nb),
        chapters::Chapter::Expressions => chapters::expressions(nb),
        chapters::Chapter::Booleans => chapters::booleans(nb),
        chapters::Chapter::State => chapters::state(nb),
        chapters::Chapter::IfElse => chapters::if_else(nb),
    }
}
