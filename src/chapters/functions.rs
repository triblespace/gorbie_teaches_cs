use egui::text::LayoutJob;
use egui::RichText;
use egui::TextStyle;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chapters::Chapter;
use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

const CHAPTER: Chapter = Chapter::Functions;

fn chapter_key(key: &'static str) -> (Chapter, &'static str) {
    (CHAPTER, key)
}

struct FunctionMachineState {
    input: i32,
}

impl Default for FunctionMachineState {
    fn default() -> Self {
        Self { input: 3 }
    }
}

struct CallCounterState {
    input: i32,
    outputs: Vec<i32>,
}

impl Default for CallCounterState {
    fn default() -> Self {
        Self {
            input: 1,
            outputs: Vec::new(),
        }
    }
}

impl CallCounterState {
    fn push_output(&mut self, value: i32) {
        self.outputs.push(value);
        if self.outputs.len() > 5 {
            self.outputs.remove(0);
        }
    }
}

#[derive(Clone, Copy)]
enum FunctionKind {
    Double,
    AddTwo,
    Square,
}

impl FunctionKind {
    fn name(self) -> &'static str {
        match self {
            FunctionKind::Double => "double",
            FunctionKind::AddTwo => "add_two",
            FunctionKind::Square => "square",
        }
    }

    fn body(self) -> &'static str {
        match self {
            FunctionKind::Double => "n * 2",
            FunctionKind::AddTwo => "n + 2",
            FunctionKind::Square => "n * n",
        }
    }

    fn apply(self, value: i32) -> i32 {
        match self {
            FunctionKind::Double => value.checked_mul(2).unwrap_or(value),
            FunctionKind::AddTwo => value.checked_add(2).unwrap_or(value),
            FunctionKind::Square => value.checked_mul(value).unwrap_or(value),
        }
    }
}

struct FunctionQuestion {
    kind: FunctionKind,
    input: i32,
    output: i32,
}

struct PracticeState {
    rng: SimpleRng,
    question: FunctionQuestion,
    choices: Vec<i32>,
    selection: Option<i32>,
}

impl Default for PracticeState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let question = generate_question(&mut rng);
        let choices = build_choices(&mut rng, question.output);
        Self {
            rng,
            question,
            choices,
            selection: None,
        }
    }
}

impl PracticeState {
    fn regenerate(&mut self) {
        self.question = generate_question(&mut self.rng);
        self.choices = build_choices(&mut self.rng, self.question.output);
        self.selection = None;
    }
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 32) as u32
    }

    fn gen_range_i32(&mut self, min: i32, max: i32) -> i32 {
        let span = (max - min + 1) as u32;
        let value = self.next_u32() % span;
        min + value as i32
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        if values.len() <= 1 {
            return;
        }
        for i in (1..values.len()).rev() {
            let j = self.gen_range_i32(0, i as i32) as usize;
            values.swap(i, j);
        }
    }
}

fn seed_from_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1)
}

fn generate_question(rng: &mut SimpleRng) -> FunctionQuestion {
    let kind = match rng.gen_range_i32(0, 2) {
        0 => FunctionKind::Double,
        1 => FunctionKind::AddTwo,
        _ => FunctionKind::Square,
    };
    let input = rng.gen_range_i32(0, 9);
    let output = kind.apply(input);
    FunctionQuestion { kind, input, output }
}

fn build_choices(rng: &mut SimpleRng, answer: i32) -> Vec<i32> {
    let mut choices = vec![answer];
    while choices.len() < 4 {
        let delta = rng.gen_range_i32(-3, 3);
        if delta == 0 {
            continue;
        }
        let candidate = answer + delta;
        if candidate < 0 || candidate > 20 {
            continue;
        }
        if !choices.contains(&candidate) {
            choices.push(candidate);
        }
    }
    rng.shuffle(&mut choices);
    choices
}

fn code_frame(ui: &mut egui::Ui, job: LayoutJob) {
    let bg = ui.visuals().code_bg_color;
    let stroke = ui.visuals().widgets.inactive.bg_stroke;
    egui::Frame::group(ui.style())
        .fill(bg)
        .stroke(stroke)
        .inner_margin(egui::Margin::same(8))
        .corner_radius(10.0)
        .show(ui, |ui| {
            ui.label(job);
        });
}

fn code_job(ui: &egui::Ui, lines: &[String]) -> LayoutJob {
    let font = TextStyle::Monospace.resolve(ui.style());
    let format = egui::TextFormat::simple(font, ui.visuals().text_color());
    let mut job = LayoutJob::default();
    for (index, line) in lines.iter().enumerate() {
        job.append(line, 0.0, format.clone());
        if index + 1 < lines.len() {
            job.append("\n", 0.0, format.clone());
        }
    }
    job
}

fn question_code(question: &FunctionQuestion) -> Vec<String> {
    vec![
        format!("function {}(n) {{", question.kind.name()),
        format!("    {}", question.kind.body()),
        "}".to_string(),
        format!("result <- {}({})", question.kind.name(), question.input),
    ]
}

fn double_plus_one(input: i32) -> i32 {
    let doubled = input.checked_mul(2).unwrap_or(input);
    doubled.checked_add(1).unwrap_or(doubled)
}

fn step_output(input: i32) -> i32 {
    let base = input.checked_add(3).unwrap_or(input);
    base.checked_mul(2).unwrap_or(base)
}

pub fn functions(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        md!(
            ui,
            "# Functions as reusable steps\n\
             A **function** is a named recipe. It takes some input, follows steps,\n\
             and gives back a result. You can call the same function many times\n\
             instead of rewriting the same logic."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## A tiny story\n\
             You pack lunches for three friends. The steps are the same each time:\n\
             slice bread, add filling, wrap it up. You could repeat the steps by hand,\n\
             but it is easier to name the recipe once and reuse it."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
             "## Define and call\n\
             A function has a **name** and a **parameter**. The parameter is the input.\n\
             The last line is the result it gives back.\n\
             ```text\n\
             function double(n) {{\n\
                 n * 2\n\
             }}\n\
             result <- double(4)\n\
             ```\n\
             The call `double(4)` means: run the recipe with input `4`."
        );
    });

    nb.state(
        &chapter_key("function_machine_state"),
        FunctionMachineState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Function machine").heading());
                ui.add_space(4.0);
                ui.label("Slide the input and watch the output change.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Input:");
                    ui.add(widgets::Slider::new(&mut state.input, -6..=6));
                });

                let lines = [
                    "function double_plus_one(n) {".to_string(),
                    "    n * 2 + 1".to_string(),
                    "}".to_string(),
                ];
                ui.add_space(6.0);
                code_frame(ui, code_job(ui, &lines));

                let output = double_plus_one(state.input);
                ui.add_space(6.0);
                ui.label(format!("Output: {output}"));
                ui.label("Same input gives the same output every time.");
            });
        },
    );

    nb.state(
        &chapter_key("call_counter_state"),
        CallCounterState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Call it many times").heading());
                ui.add_space(4.0);
                ui.label("A function is reusable. Each call is a fresh run.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Input:");
                    ui.add(widgets::Slider::new(&mut state.input, 0..=6));
                    if ui.add(widgets::Button::new("Call")).clicked() {
                        let result = step_output(state.input);
                        state.push_output(result);
                    }
                    if ui.add(widgets::Button::new("Reset")).clicked() {
                        state.outputs.clear();
                    }
                });

                ui.add_space(6.0);
                if state.outputs.is_empty() {
                    ui.label("No calls yet.");
                } else {
                    ui.label("Recent results:");
                    for value in &state.outputs {
                        ui.label(format!("- {value}"));
                    }
                }
            });
        },
    );

    nb.state(
        &chapter_key("function_practice_state"),
        PracticeState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Quick practice").heading());
                ui.add_space(6.0);
                ui.label("What is the result of this function call?");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New exercise")).clicked() {
                    state.regenerate();
                }

                ui.add_space(6.0);
                let lines = question_code(&state.question);
                code_frame(ui, code_job(ui, &lines));
                ui.add_space(6.0);

                let mut toggle = widgets::ChoiceToggle::new(&mut state.selection).small();
                for choice in &state.choices {
                    toggle = toggle.choice(Some(*choice), choice.to_string());
                }
                ui.add(toggle);
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.question.output => ui.label("Correct!"),
                    Some(_) => ui.label("Not quite. Try again."),
                    None => ui.label("Pick an answer."),
                }
            });
        },
    );

    nb.view(|ui| {
        md!(
            ui,
            "## Recap\n\
             - A function is a named recipe.\n\
             - Inputs are called parameters.\n\
             - Calling a function runs the steps and gives a result.\n\
             - Reuse functions to avoid repeating the same work."
        );
    });
}
