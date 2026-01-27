use egui::text::LayoutJob;
use egui::RichText;
use egui::TextStyle;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chapters::Chapter;
use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

const CHAPTER: Chapter = Chapter::Loops;

fn chapter_key(key: &'static str) -> (Chapter, &'static str) {
    (CHAPTER, key)
}

struct LoopStep {
    line: usize,
    count: i32,
    note: String,
}

struct LoopStepperState {
    start: i32,
    limit: i32,
    step: usize,
}

impl Default for LoopStepperState {
    fn default() -> Self {
        Self {
            start: 0,
            limit: 4,
            step: 0,
        }
    }
}

struct LoopVisualState {
    total: i32,
    count: i32,
}

impl Default for LoopVisualState {
    fn default() -> Self {
        Self { total: 5, count: 0 }
    }
}

struct PracticeState {
    rng: SimpleRng,
    start: i32,
    limit: i32,
    answer: i32,
    choices: Vec<i32>,
    selection: Option<i32>,
}

impl Default for PracticeState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let (start, limit, answer) = generate_practice(&mut rng);
        let choices = build_choices(&mut rng, answer);
        Self {
            rng,
            start,
            limit,
            answer,
            choices,
            selection: None,
        }
    }
}

impl PracticeState {
    fn regenerate(&mut self) {
        let (start, limit, answer) = generate_practice(&mut self.rng);
        self.start = start;
        self.limit = limit;
        self.answer = answer;
        self.choices = build_choices(&mut self.rng, answer);
        self.selection = None;
    }
}

struct TerminationScenario {
    start: i32,
    limit: i32,
    delta: i32,
    condition: &'static str,
    stops: bool,
}

struct TerminationPracticeState {
    rng: SimpleRng,
    scenario: TerminationScenario,
    selection: Option<bool>,
}

impl Default for TerminationPracticeState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let scenario = pick_termination_scenario(&mut rng);
        Self {
            rng,
            scenario,
            selection: None,
        }
    }
}

impl TerminationPracticeState {
    fn regenerate(&mut self) {
        self.scenario = pick_termination_scenario(&mut self.rng);
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

fn build_steps(start: i32, limit: i32) -> Vec<LoopStep> {
    let mut steps = Vec::new();
    let mut count = start;
    steps.push(LoopStep {
        line: 0,
        count,
        note: format!("Set count to {count}."),
    });

    let mut safety = 0;
    loop {
        let condition = count < limit;
        steps.push(LoopStep {
            line: 1,
            count,
            note: format!("Check count < {limit} -> {condition}."),
        });
        if !condition {
            steps.push(LoopStep {
                line: 4,
                count,
                note: "Condition is false, so the loop stops.".to_string(),
            });
            break;
        }
        steps.push(LoopStep {
            line: 2,
            count,
            note: "Run the loop body once.".to_string(),
        });
        let next = count.checked_add(1).unwrap_or(count);
        steps.push(LoopStep {
            line: 3,
            count: next,
            note: "Increase count by 1.".to_string(),
        });
        count = next;
        safety += 1;
        if safety > 20 {
            steps.push(LoopStep {
                line: 4,
                count,
                note: "Stopped early to avoid an infinite loop.".to_string(),
            });
            break;
        }
    }
    steps
}

fn generate_practice(rng: &mut SimpleRng) -> (i32, i32, i32) {
    let start = rng.gen_range_i32(0, 5);
    let mut limit = rng.gen_range_i32(start + 2, start + 6);
    if limit > 12 {
        limit = start + 4;
    }
    let answer = limit - start;
    (start, limit, answer)
}

fn build_choices(rng: &mut SimpleRng, answer: i32) -> Vec<i32> {
    let mut choices = vec![answer];
    while choices.len() < 4 {
        let delta = rng.gen_range_i32(-3, 3);
        if delta == 0 {
            continue;
        }
        let candidate = answer + delta;
        if candidate < 0 || candidate > 12 {
            continue;
        }
        if !choices.contains(&candidate) {
            choices.push(candidate);
        }
    }
    rng.shuffle(&mut choices);
    choices
}

fn pick_termination_scenario(rng: &mut SimpleRng) -> TerminationScenario {
    const SCENARIOS: &[TerminationScenario] = &[
        TerminationScenario {
            start: 0,
            limit: 5,
            delta: 1,
            condition: "<",
            stops: true,
        },
        TerminationScenario {
            start: 0,
            limit: 5,
            delta: -1,
            condition: "<",
            stops: false,
        },
        TerminationScenario {
            start: 10,
            limit: 5,
            delta: -1,
            condition: ">",
            stops: true,
        },
        TerminationScenario {
            start: 10,
            limit: 5,
            delta: 1,
            condition: ">",
            stops: false,
        },
        TerminationScenario {
            start: 3,
            limit: 3,
            delta: 1,
            condition: "<",
            stops: true,
        },
        TerminationScenario {
            start: 3,
            limit: 3,
            delta: -1,
            condition: ">",
            stops: true,
        },
    ];
    let index = rng.gen_range_i32(0, (SCENARIOS.len() - 1) as i32) as usize;
    TerminationScenario {
        start: SCENARIOS[index].start,
        limit: SCENARIOS[index].limit,
        delta: SCENARIOS[index].delta,
        condition: SCENARIOS[index].condition,
        stops: SCENARIOS[index].stops,
    }
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

fn highlight_line_job(ui: &egui::Ui, lines: &[&str], highlight: Option<usize>) -> LayoutJob {
    let font = TextStyle::Monospace.resolve(ui.style());
    let normal = egui::TextFormat::simple(font.clone(), ui.visuals().text_color());
    let highlight_format = egui::TextFormat::simple(font, GORBIE::themes::ral(2009));
    let mut job = LayoutJob::default();
    for (index, line) in lines.iter().enumerate() {
        let format = if Some(index) == highlight {
            &highlight_format
        } else {
            &normal
        };
        job.append(line, 0.0, format.clone());
        if index + 1 < lines.len() {
            job.append("\n", 0.0, normal.clone());
        }
    }
    job
}

fn termination_code(ui: &egui::Ui, scenario: &TerminationScenario) -> LayoutJob {
    let op = if scenario.delta >= 0 { "+" } else { "-" };
    let delta = scenario.delta.abs();
    let lines = [
        format!("count <- {}", scenario.start),
        format!(
            "while count {} {} {{",
            scenario.condition, scenario.limit
        ),
        format!("    count <- count {} {}", op, delta),
        "}".to_string(),
    ];
    let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    highlight_line_job(ui, &line_refs, None)
}

pub fn loops(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        md!(
            ui,
            "# Loops and counting\n\
             A **loop** repeats a block of steps until a rule says to stop.\n\
             Counting gives the loop a clear goal and keeps it from running forever.\n\
             The loop checks a **condition**, runs the **body**, and then updates the count."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## A tiny story\n\
             You water three plants. Each plant needs one cup of water.\n\
             The steps are the same each time: pour water, move to the next plant.\n\
             A loop lets the computer repeat the steps and count how many are done.\n\
             When the count reaches **3**, you stop."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## The loop shape\n\
             A counting loop usually has three parts:\n\
             1. **Start** the counter.\n\
             2. **Check** the condition.\n\
             3. **Update** the counter.\n\
             ```text\n\
             count <- 0\n\
             while count < 5 {{\n\
                 do_work\n\
                 count <- count + 1\n\
             }}\n\
             ```"
        );
    });

    nb.state(
        &chapter_key("loop_visual_state"),
        LoopVisualState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Counting visual").heading());
                ui.add_space(4.0);
                ui.label("Each step runs the loop body once and fills one segment.");
                ui.add_space(6.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Total steps:");
                    changed |= ui
                        .add(widgets::Slider::new(&mut state.total, 1..=12))
                        .changed();
                    if ui.add(widgets::Button::new("Reset")).clicked() {
                        state.count = 0;
                    }
                    if ui
                        .add_enabled(state.count < state.total, widgets::Button::new("Step"))
                        .clicked()
                    {
                        state.count = state.count.saturating_add(1).min(state.total);
                    }
                });
                if changed && state.count > state.total {
                    state.count = state.total;
                }

                let progress = if state.total > 0 {
                    state.count as f32 / state.total as f32
                } else {
                    0.0
                };
                ui.add(
                    widgets::ProgressBar::new(progress)
                        .segments(state.total.max(1) as usize)
                        .text(format!("{}/{}", state.count, state.total)),
                );
            });
        },
    );

    nb.view(|ui| {
        note!(
            ui,
            "Common mistake: forgetting to update the counter.\n\
             If the counter never changes, the condition may stay true forever."
        );
    });

    nb.state(
        &chapter_key("loop_stepper_state"),
        LoopStepperState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Step through a loop").heading());
                ui.add_space(4.0);
                ui.label("Watch the counter grow one step at a time.");
                ui.add_space(6.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Start:");
                    changed |= ui
                        .add(widgets::Slider::new(&mut state.start, 0..=8))
                        .changed();
                    ui.add_space(12.0);
                    ui.label("Stop at:");
                    changed |= ui
                        .add(widgets::Slider::new(&mut state.limit, 1..=12))
                        .changed();
                });
                if state.limit <= state.start {
                    state.limit = state.start + 1;
                }
                if changed {
                    state.step = 0;
                }

                let steps = build_steps(state.start, state.limit);
                let max_step = steps.len().saturating_sub(1);
                if state.step > max_step {
                    state.step = max_step;
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(state.step > 0, widgets::Button::new("Prev"))
                        .clicked()
                    {
                        state.step = state.step.saturating_sub(1);
                    }
                    if ui
                        .add_enabled(state.step < max_step, widgets::Button::new("Next"))
                        .clicked()
                    {
                        state.step = (state.step + 1).min(max_step);
                    }
                    if ui.add(widgets::Button::new("Reset")).clicked() {
                        state.step = 0;
                    }
                    ui.add_space(6.0);
                    ui.label(format!(
                        "Step {step}/{max_step}",
                        step = state.step,
                        max_step = max_step
                    ));
                });

                let step = &steps[state.step];
                ui.add_space(8.0);
                let lines = [
                    format!("count <- {}", state.start),
                    format!("while count < {} {{", state.limit),
                    "    do_work".to_string(),
                    "    count <- count + 1".to_string(),
                    "}".to_string(),
                ];
                let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
                code_frame(ui, highlight_line_job(ui, &line_refs, Some(step.line)));
                ui.add_space(6.0);
                ui.label(&step.note);
                ui.label(format!("count = {}", step.count));
            });
        },
    );

    nb.state(
        &chapter_key("loop_termination_state"),
        TerminationPracticeState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Will it stop?").heading());
                ui.add_space(6.0);
                ui.label("Decide whether the loop eventually stops.");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New exercise")).clicked() {
                    state.regenerate();
                }
                ui.add_space(6.0);
                let job = termination_code(ui, &state.scenario);
                code_frame(ui, job);
                ui.add_space(6.0);

                let mut toggle = widgets::ChoiceToggle::new(&mut state.selection).small();
                toggle = toggle.choice(Some(true), "Stops");
                toggle = toggle.choice(Some(false), "Runs forever");
                ui.add(toggle);
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.scenario.stops => ui.label("Correct!"),
                    Some(_) => ui.label("Not quite. Watch how count changes."),
                    None => ui.label("Pick an answer."),
                }
            });
        },
    );

    nb.state(
        &chapter_key("loop_practice_state"),
        PracticeState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Quick practice").heading());
                ui.add_space(6.0);
                ui.label("How many times does the loop body run?");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New exercise")).clicked() {
                    state.regenerate();
                }

                ui.add_space(6.0);
                ui.label(format!("Start at {start}. Stop when count < {limit}.", start = state.start, limit = state.limit));
                ui.label("Each loop adds 1 to count.");
                ui.add_space(6.0);

                let mut toggle = widgets::ChoiceToggle::new(&mut state.selection).small();
                for choice in &state.choices {
                    toggle = toggle.choice(Some(*choice), choice.to_string());
                }
                ui.add(toggle);
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.answer => ui.label("Correct!"),
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
             - A loop repeats steps until a condition becomes false.\n\
             - Counting gives the loop a clear stop point.\n\
             - A counting loop has start, check, body, and update.\n\
             - If you forget the update, the loop can run forever."
        );
    });
}
