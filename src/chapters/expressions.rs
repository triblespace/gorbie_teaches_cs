use crate::chapters::Chapter;
use egui::text::LayoutJob;
use egui::RichText;
use egui::TextStyle;
use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};
use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

const CHAPTER: Chapter = Chapter::Expressions;

fn chapter_key(key: &'static str) -> (Chapter, &'static str) {
    (CHAPTER, key)
}

struct RandomExerciseState {
    rng: SimpleRng,
    exercise: Exercise,
    choices: Vec<i64>,
    selection: Option<i64>,
}

impl Default for RandomExerciseState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let exercise = generate_exercise(&mut rng);
        let choices = build_choices(&mut rng, exercise.answer);
        Self {
            rng,
            exercise,
            choices,
            selection: None,
        }
    }
}

impl RandomExerciseState {
    fn regenerate(&mut self) {
        self.exercise = generate_exercise(&mut self.rng);
        self.choices = build_choices(&mut self.rng, self.exercise.answer);
        self.selection = None;
    }
}

struct Exercise {
    expr: Expr,
    answer: i64,
}

struct TreeExerciseState {
    rng: SimpleRng,
    expr: Expr,
    feedback: Option<String>,
}

impl Default for TreeExerciseState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let expr = generate_tree_expr(&mut rng);
        Self {
            rng,
            expr,
            feedback: None,
        }
    }
}

impl TreeExerciseState {
    fn regenerate(&mut self) {
        self.expr = generate_tree_expr(&mut self.rng);
        self.feedback = None;
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

    fn gen_range_i64(&mut self, min: i64, max: i64) -> i64 {
        let span = (max - min + 1) as u64;
        let value = self.next_u32() as u64 % span;
        min + value as i64
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        if values.len() <= 1 {
            return;
        }
        for i in (1..values.len()).rev() {
            let j = self.gen_range_i64(0, i as i64) as usize;
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

fn build_choices(rng: &mut SimpleRng, answer: i64) -> Vec<i64> {
    let mut choices = vec![answer];
    while choices.len() < 4 {
        let delta = rng.gen_range_i64(-5, 5);
        if delta == 0 {
            continue;
        }
        let candidate = answer + delta;
        if candidate < 0 {
            continue;
        }
        if !choices.contains(&candidate) {
            choices.push(candidate);
        }
    }
    rng.shuffle(&mut choices);
    choices
}

fn generate_exercise(rng: &mut SimpleRng) -> Exercise {
    for _ in 0..200 {
        let expr = random_expr(rng, 0, 3);
        if matches!(expr.kind, ExprKind::Num(_)) {
            continue;
        }
        if count_ops(&expr) < 2 {
            continue;
        }
        if let Ok(answer) = eval_expr(&expr) {
            if (0..=99).contains(&answer) {
                return Exercise { expr, answer };
            }
        }
    }
    Exercise {
        expr: Expr {
            kind: ExprKind::Add(
                Box::new(Expr {
                    kind: ExprKind::Mul(Box::new(Expr::num(2)), Box::new(Expr::num(3))),
                }),
                Box::new(Expr::num(1)),
            ),
        },
        answer: 7,
    }
}

fn eval_expr(expr: &Expr) -> Result<i64, String> {
    match &expr.kind {
        ExprKind::Num(value) => Ok(*value),
        ExprKind::Neg(inner) => eval_expr(inner)?
            .checked_neg()
            .ok_or_else(|| "Overflow".to_string()),
        ExprKind::Add(left, right) => eval_expr(left)?
            .checked_add(eval_expr(right)?)
            .ok_or_else(|| "Overflow".to_string()),
        ExprKind::Sub(left, right) => eval_expr(left)?
            .checked_sub(eval_expr(right)?)
            .ok_or_else(|| "Overflow".to_string()),
        ExprKind::Mul(left, right) => eval_expr(left)?
            .checked_mul(eval_expr(right)?)
            .ok_or_else(|| "Overflow".to_string()),
    }
}

fn count_ops(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Num(_) => 0,
        ExprKind::Neg(inner) => 1 + count_ops(inner),
        ExprKind::Add(left, right) | ExprKind::Sub(left, right) | ExprKind::Mul(left, right) => {
            1 + count_ops(left) + count_ops(right)
        }
    }
}

fn expr_to_string(expr: &Expr) -> String {
    render_expr_with_highlight(expr, None).0
}

fn random_expr(rng: &mut SimpleRng, depth: u8, max_depth: u8) -> Expr {
    let use_number = depth >= max_depth || rng.gen_range_i64(0, 4) == 0;
    if use_number {
        let value = rng.gen_range_i64(1, 9);
        return Expr::num(value);
    }

    let roll = rng.gen_range_i64(0, 4);
    if roll == 3 {
        let inner = random_expr(rng, depth + 1, max_depth);
        return Expr {
            kind: ExprKind::Neg(Box::new(inner)),
        };
    }

    let left = random_expr(rng, depth + 1, max_depth);
    let right = random_expr(rng, depth + 1, max_depth);
    let kind = match roll {
        0 => ExprKind::Add(Box::new(left), Box::new(right)),
        1 => ExprKind::Sub(Box::new(left), Box::new(right)),
        _ => ExprKind::Mul(Box::new(left), Box::new(right)),
    };
    Expr { kind }
}

fn generate_tree_expr(rng: &mut SimpleRng) -> Expr {
    for _ in 0..120 {
        let expr = random_expr(rng, 0, 3);
        if matches!(expr.kind, ExprKind::Num(_)) {
            continue;
        }
        if let Ok(value) = eval_expr(&expr) {
            if (-50..=50).contains(&value) {
                return expr;
            }
        }
    }
    Expr::num(1)
}

struct ExpressionState {
    input: String,
    step: usize,
    rng: SimpleRng,
}

impl Default for ExpressionState {
    fn default() -> Self {
        Self {
            input: "(3 * 2) + 2".to_string(),
            step: 0,
            rng: SimpleRng::new(seed_from_time()),
        }
    }
}

#[derive(Clone)]
enum ExprKind {
    Num(i64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
struct Expr {
    kind: ExprKind,
}

impl Expr {
    fn num(value: i64) -> Self {
        Self {
            kind: ExprKind::Num(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PathStep {
    Unary,
    Left,
    Right,
}

struct Step {
    expr: Expr,
    highlight: Option<Vec<PathStep>>,
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        let expr = self.parse_sum()?;
        self.skip_ws();
        if self.pos < self.input.len() {
            return Err(format!("Unexpected input at position {}", self.pos + 1));
        }
        Ok(expr)
    }

    fn parse_sum(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_product()?;
        loop {
            self.skip_ws();
            if self.consume(b'+') {
                let right = self.parse_product()?;
                node = Expr {
                    kind: ExprKind::Add(Box::new(node), Box::new(right)),
                };
            } else if self.consume(b'-') {
                let right = self.parse_product()?;
                node = Expr {
                    kind: ExprKind::Sub(Box::new(node), Box::new(right)),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_product(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_factor()?;
        loop {
            self.skip_ws();
            if self.consume(b'*') {
                let right = self.parse_factor()?;
                node = Expr {
                    kind: ExprKind::Mul(Box::new(node), Box::new(right)),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.consume(b'-') {
            let inner = self.parse_factor()?;
            return Ok(Expr {
                kind: ExprKind::Neg(Box::new(inner)),
            });
        }
        if self.consume(b'(') {
            let inner = self.parse_sum()?;
            self.skip_ws();
            if !self.consume(b')') {
                return Err(format!("Expected ')' at position {}", self.pos + 1));
            }
            return Ok(inner);
        }
        self.parse_number()
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        let start = self.pos;
        let mut value: i64 = 0;
        while let Some(byte) = self.peek() {
            if !byte.is_ascii_digit() {
                break;
            }
            self.pos += 1;
            let digit = (byte - b'0') as i64;
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or_else(|| "Number too large".to_string())?;
        }
        if self.pos == start {
            return Err(format!("Expected a number at position {}", self.pos + 1));
        }
        Ok(Expr::num(value))
    }

    fn skip_ws(&mut self) {
        while let Some(byte) = self.peek() {
            if !byte.is_ascii_whitespace() {
                break;
            }
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

fn parse_expression(input: &str) -> Result<Expr, String> {
    let mut parser = Parser::new(input);
    parser.parse_expression()
}

fn as_num(expr: &Expr) -> Option<i64> {
    match expr.kind {
        ExprKind::Num(value) => Some(value),
        _ => None,
    }
}

fn is_reducible(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Num(_) => false,
        ExprKind::Neg(inner) => as_num(inner).is_some(),
        ExprKind::Add(left, right) | ExprKind::Sub(left, right) | ExprKind::Mul(left, right) => {
            as_num(left).is_some() && as_num(right).is_some()
        }
    }
}

fn eval_reducible(expr: &Expr) -> Result<i64, String> {
    match &expr.kind {
        ExprKind::Num(value) => Ok(*value),
        ExprKind::Neg(inner) => {
            let value = as_num(inner).ok_or_else(|| "Expected a number".to_string())?;
            value.checked_neg().ok_or_else(|| "Overflow".to_string())
        }
        ExprKind::Add(left, right) => {
            let left = as_num(left).ok_or_else(|| "Expected a number".to_string())?;
            let right = as_num(right).ok_or_else(|| "Expected a number".to_string())?;
            left.checked_add(right)
                .ok_or_else(|| "Overflow".to_string())
        }
        ExprKind::Sub(left, right) => {
            let left = as_num(left).ok_or_else(|| "Expected a number".to_string())?;
            let right = as_num(right).ok_or_else(|| "Expected a number".to_string())?;
            left.checked_sub(right)
                .ok_or_else(|| "Overflow".to_string())
        }
        ExprKind::Mul(left, right) => {
            let left = as_num(left).ok_or_else(|| "Expected a number".to_string())?;
            let right = as_num(right).ok_or_else(|| "Expected a number".to_string())?;
            left.checked_mul(right)
                .ok_or_else(|| "Overflow".to_string())
        }
    }
}

fn find_reducible(expr: &Expr) -> Option<Vec<PathStep>> {
    match &expr.kind {
        ExprKind::Num(_) => None,
        ExprKind::Neg(inner) => find_reducible(inner)
            .map(|mut path| {
                path.insert(0, PathStep::Unary);
                path
            })
            .or_else(|| {
                if is_reducible(expr) {
                    Some(Vec::new())
                } else {
                    None
                }
            }),
        ExprKind::Add(left, right) | ExprKind::Sub(left, right) | ExprKind::Mul(left, right) => {
            find_reducible(left)
                .map(|mut path| {
                    path.insert(0, PathStep::Left);
                    path
                })
                .or_else(|| {
                    find_reducible(right).map(|mut path| {
                        path.insert(0, PathStep::Right);
                        path
                    })
                })
                .or_else(|| {
                    if is_reducible(expr) {
                        Some(Vec::new())
                    } else {
                        None
                    }
                })
        }
    }
}

fn reduce_at(expr: Expr, path: &[PathStep]) -> Result<Expr, String> {
    if path.is_empty() {
        return Ok(Expr::num(eval_reducible(&expr)?));
    }

    let (head, tail) = path.split_first().ok_or("Invalid path")?;
    match (head, expr.kind) {
        (PathStep::Unary, ExprKind::Neg(inner)) => Ok(Expr {
            kind: ExprKind::Neg(Box::new(reduce_at(*inner, tail)?)),
        }),
        (PathStep::Left, ExprKind::Add(left, right)) => Ok(Expr {
            kind: ExprKind::Add(Box::new(reduce_at(*left, tail)?), right),
        }),
        (PathStep::Right, ExprKind::Add(left, right)) => Ok(Expr {
            kind: ExprKind::Add(left, Box::new(reduce_at(*right, tail)?)),
        }),
        (PathStep::Left, ExprKind::Sub(left, right)) => Ok(Expr {
            kind: ExprKind::Sub(Box::new(reduce_at(*left, tail)?), right),
        }),
        (PathStep::Right, ExprKind::Sub(left, right)) => Ok(Expr {
            kind: ExprKind::Sub(left, Box::new(reduce_at(*right, tail)?)),
        }),
        (PathStep::Left, ExprKind::Mul(left, right)) => Ok(Expr {
            kind: ExprKind::Mul(Box::new(reduce_at(*left, tail)?), right),
        }),
        (PathStep::Right, ExprKind::Mul(left, right)) => Ok(Expr {
            kind: ExprKind::Mul(left, Box::new(reduce_at(*right, tail)?)),
        }),
        _ => Err("Invalid reduction path".to_string()),
    }
}

fn expr_at_path<'a>(expr: &'a Expr, path: &[PathStep]) -> Option<&'a Expr> {
    if path.is_empty() {
        return Some(expr);
    }
    let (head, tail) = path.split_first()?;
    match (head, &expr.kind) {
        (PathStep::Unary, ExprKind::Neg(inner)) => expr_at_path(inner, tail),
        (PathStep::Left, ExprKind::Add(left, _))
        | (PathStep::Left, ExprKind::Sub(left, _))
        | (PathStep::Left, ExprKind::Mul(left, _)) => expr_at_path(left, tail),
        (PathStep::Right, ExprKind::Add(_, right))
        | (PathStep::Right, ExprKind::Sub(_, right))
        | (PathStep::Right, ExprKind::Mul(_, right)) => expr_at_path(right, tail),
        _ => None,
    }
}

fn build_steps(expr: Expr) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    let mut current = expr;
    loop {
        if let Some(path) = find_reducible(&current) {
            steps.push(Step {
                expr: current.clone(),
                highlight: Some(path.clone()),
            });
            current = reduce_at(current, &path)?;
        } else {
            steps.push(Step {
                expr: current.clone(),
                highlight: None,
            });
            break;
        }
    }
    Ok(steps)
}

fn render_expr_with_highlight(
    expr: &Expr,
    highlight: Option<&[PathStep]>,
) -> (String, Vec<Range<usize>>) {
    let mut text = String::new();
    let mut highlight_range = None;
    let highlight_enabled = highlight.is_some();
    render_expr(
        expr,
        highlight.unwrap_or(&[]),
        highlight_enabled,
        &mut text,
        &mut highlight_range,
    );
    let ranges = highlight_range.into_iter().collect();
    (text, ranges)
}

fn render_expr(
    expr: &Expr,
    highlight_path: &[PathStep],
    highlight_enabled: bool,
    out: &mut String,
    highlight_range: &mut Option<Range<usize>>,
) {
    let start = out.len();
    match &expr.kind {
        ExprKind::Num(value) => {
            out.push_str(&value.to_string());
        }
        ExprKind::Neg(inner) => {
            out.push_str("(-");
            let (child_path, child_highlight): (&[PathStep], bool) =
                match highlight_path.split_first() {
                    Some((PathStep::Unary, rest)) => (rest, highlight_enabled),
                    _ => (&[], false),
                };
            render_expr(inner, child_path, child_highlight, out, highlight_range);
            out.push(')');
        }
        ExprKind::Add(left, right) => {
            out.push('(');
            let (left_path, left_highlight, right_path, right_highlight): (
                &[PathStep],
                bool,
                &[PathStep],
                bool,
            ) = match highlight_path.split_first() {
                Some((PathStep::Left, rest)) => (rest, highlight_enabled, &[], false),
                Some((PathStep::Right, rest)) => (&[], false, rest, highlight_enabled),
                _ => (&[], false, &[], false),
            };
            render_expr(left, left_path, left_highlight, out, highlight_range);
            out.push_str(" + ");
            render_expr(right, right_path, right_highlight, out, highlight_range);
            out.push(')');
        }
        ExprKind::Sub(left, right) => {
            out.push('(');
            let (left_path, left_highlight, right_path, right_highlight): (
                &[PathStep],
                bool,
                &[PathStep],
                bool,
            ) = match highlight_path.split_first() {
                Some((PathStep::Left, rest)) => (rest, highlight_enabled, &[], false),
                Some((PathStep::Right, rest)) => (&[], false, rest, highlight_enabled),
                _ => (&[], false, &[], false),
            };
            render_expr(left, left_path, left_highlight, out, highlight_range);
            out.push_str(" - ");
            render_expr(right, right_path, right_highlight, out, highlight_range);
            out.push(')');
        }
        ExprKind::Mul(left, right) => {
            out.push('(');
            let (left_path, left_highlight, right_path, right_highlight): (
                &[PathStep],
                bool,
                &[PathStep],
                bool,
            ) = match highlight_path.split_first() {
                Some((PathStep::Left, rest)) => (rest, highlight_enabled, &[], false),
                Some((PathStep::Right, rest)) => (&[], false, rest, highlight_enabled),
                _ => (&[], false, &[], false),
            };
            render_expr(left, left_path, left_highlight, out, highlight_range);
            out.push_str(" * ");
            render_expr(right, right_path, right_highlight, out, highlight_range);
            out.push(')');
        }
    }
    let end = out.len();
    if highlight_enabled && highlight_path.is_empty() {
        *highlight_range = Some(start..end);
    }
}

fn path_in_subtree(path: &[PathStep], subtree: &[PathStep]) -> bool {
    path.len() >= subtree.len() && path[..subtree.len()] == *subtree
}

struct NodeDraw {
    label: String,
    depth: usize,
    x: i32,
    highlight: bool,
    children: Vec<usize>,
    path: Vec<PathStep>,
}

struct NodeLayout {
    rect: egui::Rect,
    label: String,
    highlight: bool,
    children: Vec<usize>,
    path: Vec<PathStep>,
}

fn build_nodes(
    expr: &Expr,
    depth: usize,
    path: &mut Vec<PathStep>,
    highlight_path: Option<&[PathStep]>,
    nodes: &mut Vec<NodeDraw>,
    next_leaf_x: &mut i32,
) -> usize {
    let highlight = highlight_path.map_or(false, |sub| path_in_subtree(path, sub));
    let (label, children, x) = match &expr.kind {
        ExprKind::Num(value) => {
            let x = *next_leaf_x;
            *next_leaf_x += 1;
            (value.to_string(), Vec::new(), x)
        }
        ExprKind::Neg(inner) => {
            path.push(PathStep::Unary);
            let child = build_nodes(inner, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = nodes[child].x;
            ("-".to_string(), vec![child], x)
        }
        ExprKind::Add(left, right) => {
            path.push(PathStep::Left);
            let left_idx = build_nodes(left, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            path.push(PathStep::Right);
            let right_idx = build_nodes(right, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = (nodes[left_idx].x + nodes[right_idx].x) / 2;
            ("+".to_string(), vec![left_idx, right_idx], x)
        }
        ExprKind::Sub(left, right) => {
            path.push(PathStep::Left);
            let left_idx = build_nodes(left, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            path.push(PathStep::Right);
            let right_idx = build_nodes(right, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = (nodes[left_idx].x + nodes[right_idx].x) / 2;
            ("-".to_string(), vec![left_idx, right_idx], x)
        }
        ExprKind::Mul(left, right) => {
            path.push(PathStep::Left);
            let left_idx = build_nodes(left, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            path.push(PathStep::Right);
            let right_idx = build_nodes(right, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = (nodes[left_idx].x + nodes[right_idx].x) / 2;
            ("*".to_string(), vec![left_idx, right_idx], x)
        }
    };

    let index = nodes.len();
    nodes.push(NodeDraw {
        label,
        depth,
        x,
        highlight,
        children,
        path: path.clone(),
    });
    index
}

fn build_tree_layout(
    ui: &egui::Ui,
    expr: &Expr,
    highlight_path: Option<&[PathStep]>,
) -> (Vec<NodeLayout>, egui::Vec2, egui::FontId) {
    let mut nodes = Vec::new();
    let mut next_leaf_x = 0;
    let mut path = Vec::new();
    let _root = build_nodes(
        expr,
        0,
        &mut path,
        highlight_path,
        &mut nodes,
        &mut next_leaf_x,
    );

    let max_label_len = nodes
        .iter()
        .map(|node| node.label.chars().count())
        .max()
        .unwrap_or(1);
    let min_x = nodes.iter().map(|node| node.x).min().unwrap_or(0);
    let max_x = nodes.iter().map(|node| node.x).max().unwrap_or(0);
    let max_depth = nodes.iter().map(|node| node.depth).max().unwrap_or(0);

    let font_id = TextStyle::Monospace.resolve(ui.style());
    let (char_width, row_height) = ui.fonts_mut(|fonts| {
        let width = fonts.glyph_width(&font_id, '0');
        let height = fonts.row_height(&font_id);
        (width.max(1.0), height.max(1.0))
    });
    let node_padding = egui::vec2((char_width * 0.6).max(4.0), (row_height * 0.2).max(2.0));
    let node_width = max_label_len as f32 * char_width + node_padding.x * 2.0;
    let node_height = row_height + node_padding.y * 2.0;
    let col_gap = (char_width * 2.0).max(8.0);
    let row_gap = (row_height * 0.8).max(8.0);
    let col_spacing = node_width + col_gap;
    let row_spacing = node_height + row_gap;

    let layout_width = node_width + (max_x - min_x) as f32 * col_spacing;
    let layout_height = node_height + max_depth as f32 * row_spacing;

    let mut layouts = Vec::with_capacity(nodes.len());
    for node in &nodes {
        let x_center = node_width / 2.0 + (node.x - min_x) as f32 * col_spacing;
        let y_center = node_height / 2.0 + node.depth as f32 * row_spacing;
        let rect = egui::Rect::from_center_size(
            egui::pos2(x_center, y_center),
            egui::vec2(node_width, node_height),
        );
        layouts.push(NodeLayout {
            rect,
            label: node.label.clone(),
            highlight: node.highlight,
            children: node.children.clone(),
            path: node.path.clone(),
        });
    }

    (layouts, egui::vec2(layout_width, layout_height), font_id)
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

fn highlight_formats(ui: &egui::Ui) -> (egui::TextFormat, egui::TextFormat) {
    let font = TextStyle::Monospace.resolve(ui.style());
    let normal = egui::TextFormat::simple(font.clone(), ui.visuals().text_color());
    let highlight = egui::TextFormat::simple(font, GORBIE::themes::ral(2009));
    (normal, highlight)
}

fn append_highlighted_line(
    job: &mut LayoutJob,
    line: &str,
    ranges: &[Range<usize>],
    normal: &egui::TextFormat,
    highlight: &egui::TextFormat,
) {
    let mut cursor = 0;
    for range in ranges {
        let start = range.start.min(line.len());
        let end = range.end.min(line.len());
        if start > cursor {
            job.append(&line[cursor..start], 0.0, normal.clone());
        }
        if end > start {
            job.append(&line[start..end], 0.0, highlight.clone());
        }
        cursor = end;
    }
    if cursor < line.len() {
        job.append(&line[cursor..], 0.0, normal.clone());
    }
}

fn highlighted_job(ui: &egui::Ui, line: &str, ranges: &[Range<usize>]) -> LayoutJob {
    let (normal, highlight) = highlight_formats(ui);
    let mut job = LayoutJob::default();
    append_highlighted_line(&mut job, line, ranges, &normal, &highlight);
    job
}

fn draw_tree(ui: &mut egui::Ui, expr: &Expr, highlight_path: Option<&[PathStep]>) {
    let (mut layouts, desired, font_id) = build_tree_layout(ui, expr, highlight_path);
    let (rect, _response) = ui.allocate_at_least(desired, egui::Sense::hover());
    let mut origin = rect.min;
    if rect.width() > desired.x {
        origin.x += (rect.width() - desired.x) / 2.0;
    }
    if rect.height() > desired.y {
        origin.y += (rect.height() - desired.y) / 2.0;
    }

    for layout in &mut layouts {
        layout.rect = layout.rect.translate(origin.to_vec2());
    }

    let highlight_color = GORBIE::themes::ral(2009);
    let line_color = ui.visuals().widgets.inactive.bg_stroke.color;
    let line_width = ui.visuals().widgets.inactive.bg_stroke.width.max(1.0);
    let line_stroke = |highlight| {
        egui::Stroke::new(
            line_width,
            if highlight {
                highlight_color
            } else {
                line_color
            },
        )
    };
    let text_color = ui.visuals().text_color();
    let painter = ui.painter();

    for layout in &layouts {
        for child_idx in &layout.children {
            let child = &layouts[*child_idx];
            let highlight = layout.highlight && child.highlight;
            let stroke = line_stroke(highlight);
            let start = layout.rect.center_bottom() + egui::vec2(0.0, stroke.width / 2.0);
            let end = child.rect.center_top() - egui::vec2(0.0, stroke.width / 2.0);
            let mid_y = (start.y + end.y) / 2.0;
            let points = vec![
                start,
                egui::pos2(start.x, mid_y),
                egui::pos2(end.x, mid_y),
                end,
            ];
            painter.add(egui::Shape::line(points, stroke));
        }
        let stroke = line_stroke(layout.highlight);
        painter.rect(
            layout.rect,
            egui::CornerRadius::same(4),
            ui.visuals().code_bg_color,
            stroke,
            egui::StrokeKind::Inside,
        );
        let color = if layout.highlight {
            highlight_color
        } else {
            text_color
        };
        let galley = ui
            .fonts_mut(|fonts| fonts.layout_no_wrap(layout.label.clone(), font_id.clone(), color));
        let text_pos = layout.rect.center() - galley.size() / 2.0;
        painter.galley(text_pos, galley, text_color);
    }
}

fn draw_tree_interactive(
    ui: &mut egui::Ui,
    expr: &Expr,
    next_path: Option<&[PathStep]>,
) -> Option<Vec<PathStep>> {
    let (mut layouts, desired, font_id) = build_tree_layout(ui, expr, None);
    for layout in &mut layouts {
        layout.highlight = next_path.map_or(false, |path| path == layout.path);
    }

    let (rect, _response) = ui.allocate_at_least(desired, egui::Sense::hover());
    let mut origin = rect.min;
    if rect.width() > desired.x {
        origin.x += (rect.width() - desired.x) / 2.0;
    }
    if rect.height() > desired.y {
        origin.y += (rect.height() - desired.y) / 2.0;
    }

    for layout in &mut layouts {
        layout.rect = layout.rect.translate(origin.to_vec2());
    }

    let highlight_color = GORBIE::themes::ral(2009);
    let line_color = ui.visuals().widgets.inactive.bg_stroke.color;
    let line_width = ui.visuals().widgets.inactive.bg_stroke.width.max(1.0);
    let line_stroke = |highlight| {
        egui::Stroke::new(
            line_width,
            if highlight {
                highlight_color
            } else {
                line_color
            },
        )
    };
    let text_color = ui.visuals().text_color();
    let painter = ui.painter();
    let mut clicked = None;

    for layout in &layouts {
        for child_idx in &layout.children {
            let child = &layouts[*child_idx];
            let highlight = layout.highlight && child.highlight;
            let stroke = line_stroke(highlight);
            let start = layout.rect.center_bottom() + egui::vec2(0.0, stroke.width / 2.0);
            let end = child.rect.center_top() - egui::vec2(0.0, stroke.width / 2.0);
            let mid_y = (start.y + end.y) / 2.0;
            let points = vec![
                start,
                egui::pos2(start.x, mid_y),
                egui::pos2(end.x, mid_y),
                end,
            ];
            painter.add(egui::Shape::line(points, stroke));
        }

        let id = ui.make_persistent_id(("tree-exercise-node", &layout.path));
        let response = ui.interact(layout.rect, id, egui::Sense::click());
        if response.clicked() {
            clicked = Some(layout.path.clone());
        }

        let stroke = line_stroke(layout.highlight);
        painter.rect(
            layout.rect,
            egui::CornerRadius::same(4),
            ui.visuals().code_bg_color,
            stroke,
            egui::StrokeKind::Inside,
        );
        let color = if layout.highlight {
            highlight_color
        } else {
            text_color
        };
        let galley = ui
            .fonts_mut(|fonts| fonts.layout_no_wrap(layout.label.clone(), font_id.clone(), color));
        let text_pos = layout.rect.center() - galley.size() / 2.0;
        painter.galley(text_pos, galley, text_color);
    }

    clicked
}

pub fn expressions(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        md!(
            ui,
            "# Hello, expressions\n\
             An **expression** is a little sentence that describes the world.\n\
             Expressions let us draw conclusions or answer questions by applying simple rules\n\
             either by hand or with a computer.\n\n\
             An expression can be as simple as a **constant** value like `3`.\n\
             Or you can build larger expressions from smaller ones using symbols like\n\
             `+`, `-`, or `*`. We call those symbols **operations**.\n\n\
             Examples:\n\
             - `3`\n\
             - `3 + 1`\n\
             - `(10 - 4)`\n\
             - `(3 * 2) + 2`\n\
             - `-(4 + 1) * 3`\n\n\
             Expressions can be *evaluated*, which means turning them into a single value.\n\
             That final value is what the expression *means*.\n\n"
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## A tiny story\n\
             Imagine two baskets of apples.\n\
             Each basket holds 3 apples, and we have 2 baskets.\n\
             So we can write `3 * 2` and get **6**.\n\n\
             Now imagine there are 2 extra apples on the table:\n\
             - First, multiply the baskets: `3 * 2`.\n\
             - Then add the extras: `(3 * 2) + 2`.\n\n\
             By describing the situation with an expression, we can evaluate it to find out how many apples there are in total."
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## The rules of evaluation\n\
             When an expression has several operations, there are rules:\n\
             - Parentheses first: `(3 + 2) * 4` evaluates the part inside `()` first.\n\
             - Left-to-right when the precedence is the same: `8 - 3 - 2` means `(8 - 3) - 2`.\n\
             - Inside-out: evaluate the deepest expression before outer ones.\n\
             - Multiplication before addition or subtraction: `3 + 2 * 4` means `3 + (2 * 4)`.\n\
             - Unary minus sticks to the number or parentheses: `-(3 + 2)`.\n\n\
             These rules are called **precedence** (what happens first) and\n\
             **associativity** (how ties are grouped).\n\
             You do not need to memorize the names, just the rules."
        );
    });

    nb.view(|ui| {
        note!(
            ui,
            "In general it is much more important to understand and remember the concepts, than to remember the names!\n\
             But you will encounter them in more advanced math later, \
             where they can be useful to understand and communicate new concepts faster."
        );
    });

    nb.state(
        &chapter_key("expression_state"),
        ExpressionState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Step through an expression").heading());
                ui.add_space(4.0);
                ui.label("Use numbers, +, -, *, parentheses, and unary minus.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Expression:");
                    let response = ui.add(widgets::TextField::singleline(&mut state.input));
                    if response.changed() {
                        state.step = 0;
                    }
                    if ui.add(widgets::Button::new("Random")).clicked() {
                        let expr = generate_tree_expr(&mut state.rng);
                        state.input = expr_to_string(&expr);
                        state.step = 0;
                    }
                });

                let expr = match parse_expression(&state.input) {
                    Ok(expr) => expr,
                    Err(error) => {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("Parse error: {error}"))
                                .color(ui.visuals().error_fg_color),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("Tip: check parentheses or a missing number/operator.")
                                .color(ui.visuals().weak_text_color()),
                        );
                        return;
                    }
                };

                let steps = match build_steps(expr) {
                    Ok(steps) => steps,
                    Err(error) => {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("Evaluation error: {error}"))
                                .color(ui.visuals().error_fg_color),
                        );
                        return;
                    }
                };

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
                    ui.label(format!("Step {}/{}", state.step, max_step));
                });

                ui.add_space(8.0);
                let step = &steps[state.step];
                let (expression, expression_ranges) =
                    render_expr_with_highlight(&step.expr, step.highlight.as_deref());
                code_frame(ui, highlighted_job(ui, &expression, &expression_ranges));

                ui.add_space(6.0);
                ui.label("Tree view:");
                ui.add_space(4.0);
                draw_tree(ui, &step.expr, step.highlight.as_deref());
                ui.add_space(6.0);
                if step.highlight.is_some() {
                    ui.label("The highlighted part is what you can evaluate next.");
                } else {
                    ui.label("Fully evaluated.");
                }
            });
        },
    );

    nb.state(
        &chapter_key("tree_exercise_state"),
        TreeExerciseState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Tree practice").heading());
                ui.add_space(6.0);
                ui.label("Click a box to evaluate it in the right order (left to right).");
                ui.label("Keep going until the whole tree becomes one number.");
                ui.add_space(6.0);
                let mut show_hint = false;
                ui.horizontal(|ui| {
                    if ui.add(widgets::Button::new("New tree")).clicked() {
                        state.regenerate();
                    }
                    let hint_response = ui.add(widgets::Button::new("Hold for hint"));
                    let hint_keyboard = hint_response.has_focus()
                        && ui.input(|input| {
                            input.key_down(egui::Key::Enter) || input.key_down(egui::Key::Space)
                        });
                    show_hint = hint_response.is_pointer_button_down_on() || hint_keyboard;
                    if hint_response.clicked() {
                        state.feedback = None;
                    }
                });
                ui.add_space(6.0);

                let next_path = find_reducible(&state.expr);
                let highlight_path = if show_hint {
                    next_path.as_deref()
                } else {
                    None
                };
                let done = next_path.is_none();

                let (expression, expression_ranges) =
                    render_expr_with_highlight(&state.expr, highlight_path);
                code_frame(ui, highlighted_job(ui, &expression, &expression_ranges));
                ui.add_space(6.0);

                let clicked = draw_tree_interactive(ui, &state.expr, highlight_path);
                if !done {
                    if let Some(path) = clicked {
                        if next_path.as_ref().map_or(false, |next| next == &path) {
                            match reduce_at(state.expr.clone(), &path) {
                                Ok(expr) => {
                                    state.expr = expr;
                                    state.feedback = None;
                                }
                                Err(error) => {
                                    state.feedback = Some(format!("Oops: {error}"));
                                }
                            }
                        } else {
                            let feedback = expr_at_path(&state.expr, &path).and_then(|expr| {
                                if matches!(expr.kind, ExprKind::Num(_)) {
                                    Some("Constants already have a value.".to_string())
                                } else {
                                    None
                                }
                            });
                            state.feedback = Some(feedback.unwrap_or_else(|| {
                                "Not yet. Work left-to-right; if there is no deeper expression, move up to the next level.".to_string()
                            }));
                        }
                    }
                }

                ui.add_space(6.0);
                if let Some(value) = as_num(&state.expr) {
                    ui.label(format!("All done! Value = {value}."));
                }
                if let Some(feedback) = &state.feedback {
                    ui.label(feedback);
                }
            });
        },
    );

    nb.state(
        &chapter_key("random_exercise_state"),
        RandomExerciseState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Random practice").heading());
                ui.add_space(6.0);
                ui.label("Generate a new expression and evaluate it.");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New exercise")).clicked() {
                    state.regenerate();
                }
                ui.add_space(6.0);
                let expression = expr_to_string(&state.exercise.expr);
                code_frame(ui, highlighted_job(ui, &expression, &[]));
                ui.add_space(6.0);
                let mut toggle = widgets::ChoiceToggle::new(&mut state.selection).small();
                for choice in &state.choices {
                    toggle = toggle.choice(Some(*choice), choice.to_string());
                }
                ui.add(toggle);
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.exercise.answer => ui.label("Correct!"),
                    Some(_) => ui.label("Not quite. Try another answer or generate a new one."),
                    None => ui.label("Pick an answer."),
                }
            });
        },
    );

    nb.view(|ui| {
        md!(
            ui,
            "## What just happened\n\
             Expressions are little machines that turn inputs into values.\n\
             You can use their results anywhere a number is needed.\n\n\
             Next up: **Hello, state** shows how to *store* a value in a named box."
        );
    });
}
