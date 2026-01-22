use crate::chapters::Chapter;
use egui::RichText;
use std::time::{SystemTime, UNIX_EPOCH};
use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

const CHAPTER: Chapter = Chapter::State;

fn chapter_key(key: &'static str) -> (Chapter, &'static str) {
    (CHAPTER, key)
}

struct PracticeState {
    rng: SimpleRng,
    start: i32,
    ops: Vec<Op>,
    result: i32,
    choices: Vec<i32>,
    selection: Option<i32>,
}

impl Default for PracticeState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let (start, ops, result) = generate_practice(&mut rng);
        let choices = build_choices(&mut rng, result);
        Self {
            rng,
            start,
            ops,
            result,
            choices,
            selection: None,
        }
    }
}

impl PracticeState {
    fn regenerate(&mut self) {
        let (start, ops, result) = generate_practice(&mut self.rng);
        self.start = start;
        self.ops = ops;
        self.result = result;
        self.choices = build_choices(&mut self.rng, result);
        self.selection = None;
    }
}

#[derive(Clone, Copy)]
enum Op {
    Add(i32),
    Sub(i32),
    Mul(i32),
}

impl Op {
    fn apply(self, value: i32) -> Option<i32> {
        match self {
            Op::Add(amount) => value.checked_add(amount),
            Op::Sub(amount) => value.checked_sub(amount),
            Op::Mul(amount) => value.checked_mul(amount),
        }
    }

    fn update_line(self, name: &str, arrow: &str) -> String {
        match self {
            Op::Add(amount) => format!("{name} {arrow} {name} + {amount}"),
            Op::Sub(amount) => format!("{name} {arrow} {name} - {amount}"),
            Op::Mul(amount) => format!("{name} {arrow} {name} * {amount}"),
        }
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

fn generate_practice(rng: &mut SimpleRng) -> (i32, Vec<Op>, i32) {
    for _ in 0..200 {
        let start = rng.gen_range_i32(2, 9);
        let op_count = rng.gen_range_i32(2, 4);
        let mut ops = Vec::with_capacity(op_count as usize);
        for _ in 0..op_count {
            let op = match rng.gen_range_i32(0, 2) {
                0 => Op::Add(rng.gen_range_i32(1, 4)),
                1 => Op::Sub(rng.gen_range_i32(1, 4)),
                _ => Op::Mul(rng.gen_range_i32(2, 3)),
            };
            ops.push(op);
        }

        let mut value = start;
        let mut ok = true;
        for op in &ops {
            if let Some(next) = op.apply(value) {
                value = next;
            } else {
                ok = false;
                break;
            }
            if value < 0 || value > 99 {
                ok = false;
                break;
            }
        }

        if ok {
            return (start, ops, value);
        }
    }

    let ops = vec![Op::Add(2), Op::Mul(2)];
    (3, ops, 10)
}

fn build_choices(rng: &mut SimpleRng, answer: i32) -> Vec<i32> {
    let mut choices = vec![answer];
    while choices.len() < 4 {
        let delta = rng.gen_range_i32(-6, 6);
        if delta == 0 {
            continue;
        }
        let candidate = answer + delta;
        if !(0..=99).contains(&candidate) {
            continue;
        }
        if !choices.contains(&candidate) {
            choices.push(candidate);
        }
    }
    rng.shuffle(&mut choices);
    choices
}

pub fn state(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        md!(
            ui,
            "# Hello, state\n\
             A **variable** is a named place that holds a value over time.\n\
             The *place* stays the same, the *value* can change.\n\
             The current value in the place is its **state**.\n\n\
             If expressions are new, start with **Hello, expressions** first.\n\n\
             The *name* tells us which place we mean.\n\
             The *value* is what is inside the place.\n\
             We can change the value as the story changes.\n\n\
             We update a variable with a left arrow (←).\n\
             The right side is an expression we evaluate.\n\
             The left side is the place that gets the new value."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## A tiny story\n\
             We have a place called `apples`.\n\
             At the start, the place has **3** apples.\n\n\
             If we add one apple, the number grows.\n\
             If we take one apple, the number shrinks.\n\n\
             The place stays the same.\n\
             Only the value inside changes.\n\
             This is why we use state: the world changes and we need to remember it."
        );
    });

    nb.view(|ui| {
        let arrow = "\u{2190}";
        md!(
            ui,
            "## Assignment and update\n\
             We *introduce* a variable by giving it a name and a starting value.\n\
             Then we update it by writing a new value into the same place.\n\n\
             ```text\n\
             apples {arrow} 3\n\
             apples {arrow} apples + 1\n\
             ```\n\n\
             Read this as: “put 3 into the apples place, then add 1.”\n\
             The second line is **self-referential**: it uses `apples` to compute\n\
             the new value for `apples`.\n\
             The right side is evaluated first, using the current value.\n\
             Then we store the result in the same place."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## Some values stay fixed\n\
             Not everything should change. Sometimes we want a **constant** value\n\
             that stays the same while other values move around.\n\
             Constants make programs easier to understand because the rule never shifts.\n\
             We will use fixed values more in the Rust track."
        );
    });

    nb.state(&chapter_key("immutability_demo"), 2_i32, |ui, count| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            ui.label(RichText::new("A fixed rule, a changing value").heading());
            ui.add_space(6.0);

            let limit = 5;
            ui.label(format!("limit = {limit} (fixed)"));
            ui.label(format!("count = {count} (changes)"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.add(widgets::Button::new("+1")).clicked() {
                    *count = count.saturating_add(1);
                }
                if ui
                    .add_enabled(*count > 0, widgets::Button::new("-1"))
                    .clicked()
                {
                    *count = count.saturating_sub(1);
                }
                if ui.add(widgets::Button::new("reset")).clicked() {
                    *count = 2;
                }
            });

            ui.add_space(6.0);
            if *count >= limit {
                ui.label("The rule says: stop when count reaches the limit.");
            } else {
                ui.label("The rule stays the same even while count changes.");
            }
        });
    });

    let apples = nb.state(&chapter_key("apples"), 3_i32, |ui, value| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            ui.label(RichText::new("Try changing the value.").heading());
            ui.add_space(6.0);

            ui.label(RichText::new(format!("apples = {value}")).heading());
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.add(widgets::Button::new("+1")).clicked() {
                    *value = value.saturating_add(1);
                }
                if ui
                    .add_enabled(*value > 0, widgets::Button::new("-1"))
                    .clicked()
                {
                    *value = value.saturating_sub(1);
                }
                if ui.add(widgets::Button::new("double")).clicked() {
                    *value = value.saturating_mul(2);
                }
                if ui.add(widgets::Button::new("reset")).clicked() {
                    *value = 3;
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Set value:");
                ui.add(
                    widgets::NumberField::new(value)
                        .speed(1.0)
                        .min_decimals(0)
                        .max_decimals(0),
                );
            });

            if *value == 0 {
                ui.add_space(6.0);
                ui.label("We cannot go below zero apples.");
            }
        });
    });

    nb.state(&chapter_key("assignment_step"), 0_usize, |ui, step| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            let max_step = 3_usize;
            if *step > max_step {
                *step = max_step;
            }

            let arrow = "\u{2190}";
            let lines = [
                format!("apples {arrow} 3"),
                format!("apples {arrow} apples + 1"),
                format!("apples {arrow} apples - 1"),
                format!("apples {arrow} apples * 2"),
            ];

            ui.label(RichText::new("Step through the updates").heading());
            ui.add_space(4.0);
            ui.label("Use the buttons to move the marker.");
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.add(widgets::Button::new("Prev")).clicked() {
                    *step = step.saturating_sub(1);
                }
                if ui.add(widgets::Button::new("Next")).clicked() {
                    *step = (*step + 1).min(max_step);
                }
                if ui.add(widgets::Button::new("Reset")).clicked() {
                    *step = 0;
                }
                ui.add_space(6.0);
                let step_value = *step;
                ui.label(format!("Step {step_value}/{max_step}"));
            });

            ui.add_space(8.0);
            let mut code = String::new();
            for (index, line) in lines.iter().enumerate() {
                let marker = if index == *step { "> " } else { "  " };
                code.push_str(marker);
                code.push_str(line);
                if index + 1 < lines.len() {
                    code.push('\n');
                }
            }
            let results = [3, 4, 3, 6];
            let result = results[*step];
            let mut values = String::new();
            for (index, value) in results.iter().enumerate() {
                let marker = if index == *step { "> " } else { "  " };
                let line = format!("{marker}line {}: {}", index + 1, value);
                values.push_str(&line);
                if index + 1 < results.len() {
                    values.push('\n');
                }
            }
            widgets::markdown(
                ui,
                &format!(
                    "```text\n{code}\n```\n\n\
                     Result after this line: **{result}**\n\n\
                     Values so far:\n\
                     ```text\n{values}\n```\n\n\
                     The arrow ({arrow}) means \"update the box\".\n\
                     The name stays the same. The value changes."
                ),
            );
        });
    });

    nb.state(
        &chapter_key("practice_state"),
        PracticeState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                let arrow = "\u{2190}";
                ui.label(RichText::new("Random practice").heading());
                ui.add_space(6.0);
                ui.label("Apply the updates in order, then choose the final value.");
                ui.label("State is just the current value in the place.");
                ui.label("Each line uses the current value and writes back a new one.");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New sequence")).clicked() {
                    state.regenerate();
                }
                ui.add_space(6.0);

                let mut lines = Vec::with_capacity(state.ops.len() + 1);
                lines.push(format!("apples {arrow} {}", state.start));
                for op in &state.ops {
                    lines.push(op.update_line("apples", arrow));
                }
                let code = lines.join("\n");
                widgets::markdown(ui, &format!("```text\n{code}\n```"));

                ui.add_space(6.0);
                let mut toggle = widgets::ChoiceToggle::new(&mut state.selection).small();
                for choice in &state.choices {
                    toggle = toggle.choice(Some(*choice), choice.to_string());
                }
                ui.add(toggle);
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.result => ui.label("Correct!"),
                    Some(_) => ui.label("Not quite. Try another answer."),
                    None => ui.label("Pick an answer."),
                }
            });
        },
    );

    nb.view(move |ui| {
        let value = apples.read(ui);
        md!(
            ui,
            "## What just happened\n\
             A variable keeps its value until you change it.\n\
             Buttons change the value, so the number updates.\n\n\
             Current value: **{value}**"
        );
    });
}
