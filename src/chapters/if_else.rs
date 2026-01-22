use egui::text::LayoutJob;
use egui::Align2;
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

struct FlowchartIntroState {
    condition: bool,
}

impl Default for FlowchartIntroState {
    fn default() -> Self {
        Self { condition: false }
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

struct Action {
    label: &'static str,
    code: &'static [&'static str],
    display: &'static str,
}

impl Action {
    fn new(label: &'static str, code: &'static [&'static str], display: &'static str) -> Self {
        Self {
            label,
            code,
            display,
        }
    }
}

struct Condition<Ctx> {
    label: &'static str,
    code: &'static str,
    eval: fn(&Ctx) -> bool,
}

impl<Ctx> Condition<Ctx> {
    fn new(label: &'static str, code: &'static str, eval: fn(&Ctx) -> bool) -> Self {
        Self { label, code, eval }
    }
}

enum DecisionTail<Ctx> {
    Action(Action),
    Next(Box<Decision<Ctx>>),
}

struct Decision<Ctx> {
    condition: Condition<Ctx>,
    yes: Action,
    no: DecisionTail<Ctx>,
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

fn planner_is_raining(state: &PlannerState) -> bool {
    state.raining
}

fn planner_is_hot(state: &PlannerState) -> bool {
    state.temperature >= 25
}

fn flowchart_intro_condition(state: &FlowchartIntroState) -> bool {
    state.condition
}

fn stepper_can_buy(state: &StepperState) -> bool {
    state.coins >= state.price
}

fn flowchart_intro_decision() -> Decision<FlowchartIntroState> {
    Decision {
        condition: Condition::new("condition?", "condition", flowchart_intro_condition),
        yes: Action::new("do_this", &["do_this"], "do_this"),
        no: DecisionTail::Action(Action::new("do_that", &["do_that"], "do_that")),
    }
}

fn plan_decision() -> Decision<PlannerState> {
    Decision {
        condition: Condition::new("raining?", "raining", planner_is_raining),
        yes: Action::new("umbrella", &["plan = \"umbrella\""], "Take an umbrella."),
        no: DecisionTail::Next(Box::new(Decision {
            condition: Condition::new(
                "temperature >= 25?",
                "temperature >= 25",
                planner_is_hot,
            ),
            yes: Action::new("sunglasses", &["plan = \"sunglasses\""], "Bring sunglasses."),
            no: DecisionTail::Action(Action::new(
                "jacket",
                &["plan = \"jacket\""],
                "Bring a light jacket.",
            )),
        })),
    }
}

fn stepper_decision() -> Decision<StepperState> {
    Decision {
        condition: Condition::new("coins >= price?", "coins >= price", stepper_can_buy),
        yes: Action::new(
            "buy",
            &["coins = coins - price", "status = \"bought\""],
            "bought",
        ),
        no: DecisionTail::Action(Action::new(
            "do not buy",
            &["status = \"not enough\""],
            "not enough",
        )),
    }
}

fn decision_chain<'a, Ctx>(
    decision: &'a Decision<Ctx>,
) -> (Vec<(&'a Condition<Ctx>, &'a Action)>, &'a Action) {
    let mut steps = Vec::new();
    let mut current = decision;
    loop {
        steps.push((&current.condition, &current.yes));
        match &current.no {
            DecisionTail::Action(action) => return (steps, action),
            DecisionTail::Next(next) => current = next,
        }
    }
}

fn decision_selected_index<Ctx>(decision: &Decision<Ctx>, ctx: &Ctx) -> usize {
    let (steps, _) = decision_chain(decision);
    for (idx, (condition, _)) in steps.iter().enumerate() {
        if (condition.eval)(ctx) {
            return idx;
        }
    }
    steps.len()
}

fn decision_selected_action<'a, Ctx>(decision: &'a Decision<Ctx>, ctx: &Ctx) -> &'a Action {
    let (steps, else_action) = decision_chain(decision);
    for (condition, action) in steps {
        if (condition.eval)(ctx) {
            return action;
        }
    }
    else_action
}

fn decision_code_lines<Ctx>(decision: &Decision<Ctx>) -> Vec<String> {
    let (steps, else_action) = decision_chain(decision);
    let mut lines = Vec::new();
    for (idx, (condition, action)) in steps.iter().enumerate() {
        if idx == 0 {
            lines.push(format!("if {} {{", condition.code));
        } else {
            lines.push(format!("}} else if {} {{", condition.code));
        }
        for line in action.code {
            lines.push(format!("    {line}"));
        }
    }
    lines.push("} else {".to_string());
    for line in else_action.code {
        lines.push(format!("    {line}"));
    }
    lines.push("}".to_string());
    lines
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

fn text_width(ui: &egui::Ui, text: &str, font_id: &egui::FontId) -> f32 {
    ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_string(), font_id.clone(), ui.visuals().text_color())
            .size()
            .x
    })
}

fn paint_polyline(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    stroke: egui::Stroke,
    corner_radius: f32,
) {
    if points.len() < 2 {
        return;
    }
    for segment in points.windows(2) {
        painter.line_segment([segment[0], segment[1]], stroke);
    }
    if corner_radius > 0.0 && points.len() > 2 {
        for corner in &points[1..points.len() - 1] {
            painter.circle_filled(*corner, corner_radius, stroke.color);
        }
    }
}

fn paint_if_else_flowchart<Ctx>(
    ui: &mut egui::Ui,
    decision: &Decision<Ctx>,
    ctx: &Ctx,
) {
    let (steps, else_action) = decision_chain(decision);
    if steps.is_empty() {
        return;
    }
    let width = ui.available_width().max(240.0);
    let background = ui.visuals().window_fill;
    let outline = ui.visuals().widgets.noninteractive.bg_stroke.color;
    let inactive = GORBIE::themes::blend(background, outline, 0.55);
    let accent = GORBIE::themes::ral(2009);
    let active_stroke = egui::Stroke::new(2.0, accent);
    let inactive_stroke = egui::Stroke::new(2.0, inactive);
    let neutral_stroke = egui::Stroke::new(2.0, outline);

    let font_id = TextStyle::Monospace.resolve(ui.style());
    let mut action_label_width = text_width(ui, else_action.label, &font_id);
    let mut condition_label_width: f32 = 0.0;
    for (condition, action) in &steps {
        action_label_width = action_label_width.max(text_width(ui, action.label, &font_id));
        condition_label_width = condition_label_width.max(text_width(ui, condition.label, &font_id));
    }
    let value_width = text_width(ui, "(false)", &font_id);
    condition_label_width = condition_label_width.max(value_width);

    let action_box_w = (action_label_width + 24.0).clamp(96.0, width * 0.45);
    let action_box_h = 28.0;
    let condition_box_w = (condition_label_width + 28.0).clamp(120.0, width * 0.55);
    let condition_box_h = 40.0;
    let start_r = 7.0;
    let row_gap = 28.0;
    let top_padding = 8.0;
    let gap_to_condition = 16.0;
    let bottom_padding = 8.0;
    let chosen = decision_selected_index(decision, ctx);

    let height = (top_padding
        + start_r * 2.0
        + gap_to_condition
        + steps.len().saturating_sub(1) as f32 * (condition_box_h + row_gap)
        + condition_box_h
        + bottom_padding)
        .max(140.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let center_x = rect.center().x;
    let top = rect.top() + top_padding;
    let start_center = egui::pos2(center_x, top + start_r);
    let first_condition_center_y =
        start_center.y + start_r + gap_to_condition + condition_box_h / 2.0;
    let max_dx = (rect.width() / 2.0 - action_box_w / 2.0 - 6.0).max(0.0);
    let min_dx = condition_box_w / 2.0 + action_box_w / 2.0 + 8.0;
    let branch_dx = if min_dx <= max_dx { min_dx } else { max_dx };

    let corner_radius = 3.0;
    let mut last_condition_box = egui::Rect::NOTHING;
    let mut last_left_box = egui::Rect::NOTHING;

    painter.add(egui::Shape::circle_filled(start_center, start_r, background));
    painter.add(egui::Shape::circle_stroke(start_center, start_r, neutral_stroke));

    for (idx, (condition, action)) in steps.iter().enumerate() {
        let condition_center_y =
            first_condition_center_y + idx as f32 * (condition_box_h + row_gap);
        let condition_box = egui::Rect::from_center_size(
            egui::pos2(center_x, condition_center_y),
            egui::vec2(condition_box_w, condition_box_h),
        );
        let right_center = egui::pos2(center_x + branch_dx, condition_center_y);
        let right_box =
            egui::Rect::from_center_size(right_center, egui::vec2(action_box_w, action_box_h));

        let condition_top = egui::pos2(condition_box.center().x, condition_box.top());
        let condition_bottom = egui::pos2(condition_box.center().x, condition_box.bottom());
        let condition_right = egui::pos2(condition_box.right(), condition_box.center().y);
        let right_left = egui::pos2(right_box.left(), right_box.center().y);
        if idx == 0 {
            let start_bottom = egui::pos2(center_x, start_center.y + start_r);
            painter.line_segment([start_bottom, condition_top], neutral_stroke);
        }

        let yes_path = [condition_right, right_left];
        let yes_stroke = if chosen == idx {
            active_stroke
        } else {
            inactive_stroke
        };
        paint_polyline(&painter, &yes_path, yes_stroke, corner_radius);

        if idx + 1 < steps.len() {
            let next_center_y =
                first_condition_center_y + (idx + 1) as f32 * (condition_box_h + row_gap);
            let next_top = egui::pos2(center_x, next_center_y - condition_box_h / 2.0);
            let no_path = [condition_bottom, next_top];
            let no_stroke = if chosen > idx {
                active_stroke
            } else {
                inactive_stroke
            };
            paint_polyline(&painter, &no_path, no_stroke, 0.0);
        }

        painter.rect_filled(condition_box, 6.0, background);
        painter.rect_stroke(condition_box, 6.0, neutral_stroke, egui::StrokeKind::Inside);

        let yes_fill = if chosen == idx {
            GORBIE::themes::blend(background, accent, 0.12)
        } else {
            background
        };
        painter.rect_filled(right_box, 6.0, yes_fill);
        painter.rect_stroke(right_box, 6.0, yes_stroke, egui::StrokeKind::Inside);

        let value = if (condition.eval)(ctx) { "true" } else { "false" };
        painter.text(
            condition_box.center(),
            Align2::CENTER_CENTER,
            format!("{}\n({value})", condition.label),
            font_id.clone(),
            ui.visuals().text_color(),
        );
        painter.text(
            right_box.center(),
            Align2::CENTER_CENTER,
            action.label,
            font_id.clone(),
            ui.visuals().text_color(),
        );

        last_condition_box = condition_box;
        last_left_box = egui::Rect::from_center_size(
            egui::pos2(center_x - branch_dx, condition_center_y),
            egui::vec2(action_box_w, action_box_h),
        );
    }

    let else_path = [
        egui::pos2(last_condition_box.left(), last_condition_box.center().y),
        egui::pos2(last_left_box.right(), last_left_box.center().y),
    ];
    let else_stroke = if chosen >= steps.len() {
        active_stroke
    } else {
        inactive_stroke
    };
    paint_polyline(&painter, &else_path, else_stroke, corner_radius);
    let else_fill = if chosen >= steps.len() {
        GORBIE::themes::blend(background, accent, 0.12)
    } else {
        background
    };
    painter.rect_filled(last_left_box, 6.0, else_fill);
    painter.rect_stroke(last_left_box, 6.0, else_stroke, egui::StrokeKind::Inside);
    painter.text(
        last_left_box.center(),
        Align2::CENTER_CENTER,
        else_action.label,
        font_id,
        ui.visuals().text_color(),
    );
}

pub fn if_else(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "# Forks in the Road\n\
                 Programs often face choices: **if** something is true, do one thing,\n\
                 **else** do something different. An `if/else` is the tool for those choices.\n\
                 It always picks **one** path, never both.\n\
                 This lets you turn real-world questions into clear, testable rules."
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## A tiny story\n\
                 You walk outside and ask a simple question: *Is it raining?*\n\
                 If yes, you grab an umbrella. If no, you keep walking.\n\
                 The question is the **condition**, and the umbrella/keep-walking\n\
                 are the two **branches**. A decision picks **one** branch."
            );
        });
    });

    nb.state(
        &chapter_key("flowchart_intro_state"),
        FlowchartIntroState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                md!(
                    ui,
                    "## A flowchart first\n\
                     A flowchart is a picture of a decision.\n\
                     The box asks a yes/no question, and the arrows show the two paths.\n\
                     You follow the arrow that matches the answer and ignore the other.\n\
                     Flip the condition below and watch the highlighted path change."
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Condition:");
                    ui.add(
                        widgets::ChoiceToggle::binary(&mut state.condition, "false", "true")
                            .small(),
                    );
                });
                ui.add_space(6.0);
                let decision = flowchart_intro_decision();
                paint_if_else_flowchart(ui, &decision, state);
            });
        },
    );

    nb.state(
        &chapter_key("planner_state"),
        PlannerState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Plan your day (flowchart)").heading());
                ui.add_space(4.0);
                ui.label("Try different weather and see the plan change.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.add(widgets::ToggleButton::new(&mut state.raining, "Raining"));
                    ui.add_space(12.0);
                    ui.label("Temperature:");
                    ui.add(widgets::Slider::new(&mut state.temperature, 0..=40).text("C"));
                });

                let decision = plan_decision();
                let plan = decision_selected_action(&decision, state).display;

                ui.add_space(8.0);
                ui.label(format!("Plan: {plan}"));
                ui.add_space(8.0);
                ui.label("Flowchart:");
                ui.add_space(4.0);
                paint_if_else_flowchart(ui, &decision, state);
            });
        },
    );

    nb.view(|ui| {
        note!(
            ui,
            "Only one branch runs.\n\
             The other branch is skipped completely.\n\
             This makes the program predictable: exactly one path is taken."
        );
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Why this matters\n\
                 If/else lets you **guard** actions. You can check a rule before you act.\n\
                 That means safer programs: only spend coins if you have enough,\n\
                 only open the door if the code is correct, only send a message if it is valid.\n\
                 Decisions help your program match how the real world works."
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Writing it as code\n\
                 The flowchart above turns into `if/else` code like this:\n\
                 ```text\n\
                 if condition {{\n\
                     do_this\n\
                 }} else {{\n\
                     do_that\n\
                 }}\n\
                 ```\n\
                 The condition must be a boolean. The lines inside the braces form a block.\n\
                 Only one block runs, so your program takes one clear path."
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Conditions are booleans\n\
                 The `condition` in an if/else must be **true** or **false**.\n\
                 That means any boolean expression works here.\n\
                 You can use variables, comparisons, and logic operators to build a condition.\n\
                 ```text\n\
                 if true {{ ... }}\n\
                 if (a and b) or not c {{ ... }}\n\
                 ```"
            );
        });
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Comparisons create booleans\n\
                 Comparisons like `>` or `==` produce a boolean.\n\
                 That lets us use numbers inside if/else.\n\
                 Read them as questions: *Is apples greater than 3? Is coins equal to price?*\n\
                 ```text\n\
                 if apples > 3 {{ ... }}\n\
                 if coins == price {{ ... }}\n\
                 ```"
            );
        });
    });

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

                let step = &steps[state.step];
                ui.add_space(8.0);
                let decision = stepper_decision();
                let code_lines = decision_code_lines(&decision);
                let code_refs: Vec<&str> =
                    code_lines.iter().map(String::as_str).collect();
                code_frame(ui, highlight_line_job(ui, &code_refs, Some(step.line)));
                ui.add_space(6.0);
                ui.label(&step.note);
                let status = step.status.unwrap_or("(not set yet)");
                ui.label(format!(
                    "coins = {coins}, status = {status}",
                    coins = step.coins
                ));
                ui.add_space(8.0);
                ui.label("Flowchart view:");
                ui.add_space(4.0);
                paint_if_else_flowchart(ui, &decision, state);
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
        note!(
            ui,
            "Common mistake: forgetting the `else`.\n\
             If you only write `if`, nothing happens when the condition is false.\n\
             That can be okay, but make sure it is intentional."
        );
    });

    nb.view(|ui| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            md!(
                ui,
                "## Recap\n\
                 - `if/else` chooses between two paths based on a question.\n\
                 - The condition must be a boolean (true/false).\n\
                 - Comparisons like `>` and `==` create booleans you can test.\n\
                 - Only one branch runs; the other is skipped.\n\
                 - Flowcharts and code are two views of the same decision."
            );
        });
    });
}
