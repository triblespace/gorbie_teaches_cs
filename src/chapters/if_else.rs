use egui::text::LayoutJob;
use egui::RichText;
use egui::TextStyle;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chapters::Chapter;
use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

const CHAPTER: Chapter = Chapter::IfElse;

fn chapter_key(key: &'static str) -> (Chapter, &'static str) {
    (CHAPTER, key)
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
}

fn seed_from_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1)
}

struct PlannerState {
    raining: bool,
    temperature: i32,
}

impl Default for PlannerState {
    fn default() -> Self {
        Self {
            raining: false,
            temperature: 22,
        }
    }
}

struct StepperState {
    coins: i32,
    price: i32,
    step: usize,
}

impl Default for StepperState {
    fn default() -> Self {
        Self {
            coins: 6,
            price: 4,
            step: 0,
        }
    }
}

struct CodeStep {
    line: usize,
    coins: i32,
    status: Option<&'static str>,
    note: String,
}

struct Scenario {
    coins: i32,
    price: i32,
    can_buy: bool,
}

struct RandomPracticeState {
    rng: SimpleRng,
    scenario: Scenario,
    selection: Option<bool>,
}

impl Default for RandomPracticeState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let scenario = generate_scenario(&mut rng);
        Self {
            rng,
            scenario,
            selection: None,
        }
    }
}

impl RandomPracticeState {
    fn regenerate(&mut self) {
        self.scenario = generate_scenario(&mut self.rng);
        self.selection = None;
    }
}

fn generate_scenario(rng: &mut SimpleRng) -> Scenario {
    let coins = rng.gen_range_i32(0, 12);
    let price = rng.gen_range_i32(0, 12);
    Scenario {
        coins,
        price,
        can_buy: coins >= price,
    }
}

fn build_steps(coins: i32, price: i32) -> Vec<CodeStep> {
    let condition = coins >= price;
    let mut steps = Vec::new();
    steps.push(CodeStep {
        line: 0,
        coins,
        status: None,
        note: format!("Check coins >= price -> {condition}"),
    });

    if condition {
        let next_coins = coins.checked_sub(price).unwrap_or(coins);
        steps.push(CodeStep {
            line: 1,
            coins: next_coins,
            status: None,
            note: "Take the if branch and subtract the price.".to_string(),
        });
        steps.push(CodeStep {
            line: 2,
            coins: next_coins,
            status: Some("bought"),
            note: "Store the message.".to_string(),
        });
    } else {
        steps.push(CodeStep {
            line: 4,
            coins,
            status: Some("not enough"),
            note: "Take the else branch.".to_string(),
        });
    }

    steps
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

pub fn if_else(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "# Forks in the Road\n\\
                 An **if/else** statement lets your program choose between two paths.\n\\
                 We use it whenever a decision depends on a boolean."
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## A tiny story\n\\
                 You walk outside. If it is raining, you grab an umbrella.\n\\
                 Otherwise you keep walking.\n\\
                 That is exactly what `if/else` does: pick **one** path."
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## The if/else shape\n\\
                 ```text\n\\
                 if condition {{\n\\
                     do_this\n\\
                 }} else {{\n\\
                     do_that\n\\
                 }}\n\\
                 ```\n\\
                 The condition must be a boolean.\n\\
                 Only one branch runs."
            );
        });
    });

    nb.view(|ui| {
        note!(
            ui,
            "Only one branch runs.\n\\
             The other branch is skipped completely."
        );
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Conditions are booleans\n\\
                 The `condition` in an if/else must be **true** or **false**.\n\\
                 That means any boolean expression works here.\n\\
                 ```text\n\\
                 if true {{ ... }}\n\\
                 if (a and b) or not c {{ ... }}\n\\
                 ```"
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Comparisons create booleans\n\\
                 Comparisons like `>` or `==` produce a boolean.\n\\
                 That lets us use numbers inside if/else.\n\\
                 ```text\n\\
                 if apples > 3 {{ ... }}\n\\
                 if coins == price {{ ... }}\n\\
                 ```"
            );
        });
    });

    nb.state(
        &chapter_key("planner_state"),
        PlannerState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Plan your day").heading());
                ui.add_space(4.0);
                ui.label("Try different weather and see the plan change.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.add(widgets::ToggleButton::new(&mut state.raining, "Raining"));
                    ui.add_space(12.0);
                    ui.label("Temperature:");
                    ui.add(widgets::Slider::new(&mut state.temperature, 0..=40).text("C"));
                });

                let plan = if state.raining {
                    "Take an umbrella."
                } else if state.temperature >= 25 {
                    "Bring sunglasses."
                } else {
                    "Bring a light jacket."
                };

                ui.add_space(8.0);
                let code = [
                    "if raining {",
                    "    plan = \"umbrella\"",
                    "} else if temperature >= 25 {",
                    "    plan = \"sunglasses\"",
                    "} else {",
                    "    plan = \"jacket\"",
                    "}",
                ];
                code_frame(ui, highlight_line_job(ui, &code, None));
                ui.add_space(8.0);
                ui.label(format!("Plan: {plan}"));
            });
        },
    );

    nb.state(
        &chapter_key("stepper_state"),
        StepperState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Step through a decision").heading());
                ui.add_space(4.0);
                ui.label("Move through the decision one line at a time.");
                ui.add_space(6.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Coins:");
                    changed |= ui
                        .add(widgets::Slider::new(&mut state.coins, 0..=12))
                        .changed();
                    ui.add_space(12.0);
                    ui.label("Price:");
                    changed |= ui
                        .add(widgets::Slider::new(&mut state.price, 0..=12))
                        .changed();
                });
                if changed {
                    state.step = 0;
                }

                let steps = build_steps(state.coins, state.price);
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

                let lines = [
                    "if coins >= price {",
                    "    coins = coins - price",
                    "    status = \"bought\"",
                    "} else {",
                    "    status = \"not enough\"",
                    "}",
                ];

                let step = &steps[state.step];
                ui.add_space(8.0);
                code_frame(ui, highlight_line_job(ui, &lines, Some(step.line)));
                ui.add_space(6.0);
                ui.label(&step.note);
                let status = step.status.unwrap_or("(not set yet)");
                ui.label(format!(
                    "coins = {coins}, status = {status}",
                    coins = step.coins
                ));
            });
        },
    );

    nb.state(
        &chapter_key("random_practice_state"),
        RandomPracticeState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Random practice").heading());
                ui.add_space(6.0);
                ui.label("Decide which branch runs.");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New exercise")).clicked() {
                    state.regenerate();
                }

                ui.add_space(6.0);
                let coins = state.scenario.coins;
                let price = state.scenario.price;
                ui.label(format!("You have {coins} coins. The price is {price}."));
                ui.label("If coins >= price, you buy it. Otherwise you do not.");
                ui.add_space(6.0);

                let mut toggle = widgets::ChoiceToggle::new(&mut state.selection).small();
                toggle = toggle.choice(Some(true), "Buy");
                toggle = toggle.choice(Some(false), "Do not buy");
                ui.add(toggle);
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.scenario.can_buy => ui.label("Correct!"),
                    Some(_) => ui.label("Not quite. Try again."),
                    None => ui.label("Pick a branch."),
                }
            });
        },
    );

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Recap\n\\
                 - `if/else` chooses between two paths.\n\\
                 - The condition must be a boolean.\n\\
                 - Comparisons like `>` and `==` create booleans.\n\\
                 - Only one branch runs, and the other is skipped."
            );
        });
    });
}
