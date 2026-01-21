use egui::text::LayoutJob;
use egui::RichText;
use egui::TextStyle;
use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chapters::Chapter;
use GORBIE::cards::{with_padding, DEFAULT_CARD_PADDING};
use GORBIE::prelude::*;

const CHAPTER: Chapter = Chapter::Booleans;

fn chapter_key(key: &'static str) -> (Chapter, &'static str) {
    (CHAPTER, key)
}

struct ExpressionState {
    input: String,
    step: usize,
    rng: SimpleRng,
}

impl Default for ExpressionState {
    fn default() -> Self {
        Self {
            input: "not (true and false) or true".to_string(),
            step: 0,
            rng: SimpleRng::new(seed_from_time()),
        }
    }
}

struct RandomExerciseState {
    rng: SimpleRng,
    exercise: Exercise,
    selection: Option<bool>,
}

impl Default for RandomExerciseState {
    fn default() -> Self {
        let mut rng = SimpleRng::new(seed_from_time());
        let exercise = generate_exercise(&mut rng);
        Self {
            rng,
            exercise,
            selection: None,
        }
    }
}

impl RandomExerciseState {
    fn regenerate(&mut self) {
        self.exercise = generate_exercise(&mut self.rng);
        self.selection = None;
    }
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

struct Exercise {
    expr: Expr,
    answer: bool,
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

#[derive(Clone)]
enum ExprKind {
    Bool(bool),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
struct Expr {
    kind: ExprKind,
}

impl Expr {
    fn boolean(value: bool) -> Self {
        Self {
            kind: ExprKind::Bool(value),
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
        let expr = self.parse_or()?;
        self.skip_ws();
        if self.pos < self.input.len() {
            return Err(format!("Unexpected input at position {}", self.pos + 1));
        }
        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_and()?;
        loop {
            self.skip_ws();
            if self.consume_word("or") || self.consume_bytes(b"||") {
                let right = self.parse_and()?;
                node = Expr {
                    kind: ExprKind::Or(Box::new(node), Box::new(right)),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_unary()?;
        loop {
            self.skip_ws();
            if self.consume_word("and") || self.consume_bytes(b"&&") {
                let right = self.parse_unary()?;
                node = Expr {
                    kind: ExprKind::And(Box::new(node), Box::new(right)),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.consume_word("not") || self.consume_bytes(b"!") {
            let inner = self.parse_unary()?;
            return Ok(Expr {
                kind: ExprKind::Not(Box::new(inner)),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.consume_bytes(b"(") {
            let expr = self.parse_or()?;
            self.skip_ws();
            if !self.consume_bytes(b")") {
                return Err("Expected ')'".to_string());
            }
            return Ok(expr);
        }
        if let Some(value) = self.consume_bool() {
            return Ok(Expr::boolean(value));
        }
        Err(format!("Expected true/false at position {}", self.pos + 1))
    }

    fn consume_bool(&mut self) -> Option<bool> {
        if self.consume_word("true") || self.consume_word("yes") || self.consume_word("on") {
            return Some(true);
        }
        if self.consume_word("false") || self.consume_word("no") || self.consume_word("off") {
            return Some(false);
        }
        None
    }

    fn skip_ws(&mut self) {
        while let Some(byte) = self.peek() {
            if byte.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn consume_bytes(&mut self, bytes: &[u8]) -> bool {
        if self.input.get(self.pos..self.pos + bytes.len()) == Some(bytes) {
            self.pos += bytes.len();
            true
        } else {
            false
        }
    }

    fn consume_word(&mut self, word: &str) -> bool {
        let bytes = word.as_bytes();
        if self.input.get(self.pos..self.pos + bytes.len()) != Some(bytes) {
            return false;
        }
        let next = self.input.get(self.pos + bytes.len()).copied();
        if let Some(byte) = next {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                return false;
            }
        }
        self.pos += bytes.len();
        true
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }
}

fn parse_expression(input: &str) -> Result<Expr, String> {
    let mut parser = Parser::new(input);
    parser.parse_expression()
}

fn as_bool(expr: &Expr) -> Option<bool> {
    match expr.kind {
        ExprKind::Bool(value) => Some(value),
        _ => None,
    }
}

fn is_reducible(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Bool(_) => false,
        ExprKind::Not(inner) => as_bool(inner).is_some(),
        ExprKind::And(left, right) | ExprKind::Or(left, right) => {
            as_bool(left).is_some() && as_bool(right).is_some()
        }
    }
}

fn eval_reducible(expr: &Expr) -> Result<bool, String> {
    match &expr.kind {
        ExprKind::Bool(value) => Ok(*value),
        ExprKind::Not(inner) => {
            let value = as_bool(inner).ok_or_else(|| "Expected a boolean".to_string())?;
            Ok(!value)
        }
        ExprKind::And(left, right) => {
            let left = as_bool(left).ok_or_else(|| "Expected a boolean".to_string())?;
            let right = as_bool(right).ok_or_else(|| "Expected a boolean".to_string())?;
            Ok(left && right)
        }
        ExprKind::Or(left, right) => {
            let left = as_bool(left).ok_or_else(|| "Expected a boolean".to_string())?;
            let right = as_bool(right).ok_or_else(|| "Expected a boolean".to_string())?;
            Ok(left || right)
        }
    }
}

fn find_reducible(expr: &Expr) -> Option<Vec<PathStep>> {
    match &expr.kind {
        ExprKind::Bool(_) => None,
        ExprKind::Not(inner) => find_reducible(inner)
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
        ExprKind::And(left, right) | ExprKind::Or(left, right) => find_reducible(left)
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
            }),
    }
}

fn reduce_at(expr: Expr, path: &[PathStep]) -> Result<Expr, String> {
    if path.is_empty() {
        return Ok(Expr::boolean(eval_reducible(&expr)?));
    }

    let (head, tail) = path.split_first().ok_or("Invalid path")?;
    match (head, expr.kind) {
        (PathStep::Unary, ExprKind::Not(inner)) => Ok(Expr {
            kind: ExprKind::Not(Box::new(reduce_at(*inner, tail)?)),
        }),
        (PathStep::Left, ExprKind::And(left, right)) => Ok(Expr {
            kind: ExprKind::And(Box::new(reduce_at(*left, tail)?), right),
        }),
        (PathStep::Right, ExprKind::And(left, right)) => Ok(Expr {
            kind: ExprKind::And(left, Box::new(reduce_at(*right, tail)?)),
        }),
        (PathStep::Left, ExprKind::Or(left, right)) => Ok(Expr {
            kind: ExprKind::Or(Box::new(reduce_at(*left, tail)?), right),
        }),
        (PathStep::Right, ExprKind::Or(left, right)) => Ok(Expr {
            kind: ExprKind::Or(left, Box::new(reduce_at(*right, tail)?)),
        }),
        _ => Err("Invalid path".to_string()),
    }
}

fn expr_at_path<'a>(expr: &'a Expr, path: &[PathStep]) -> Option<&'a Expr> {
    if path.is_empty() {
        return Some(expr);
    }
    let (head, tail) = path.split_first()?;
    match (head, &expr.kind) {
        (PathStep::Unary, ExprKind::Not(inner)) => expr_at_path(inner, tail),
        (PathStep::Left, ExprKind::And(left, _)) | (PathStep::Left, ExprKind::Or(left, _)) => {
            expr_at_path(left, tail)
        }
        (PathStep::Right, ExprKind::And(_, right)) | (PathStep::Right, ExprKind::Or(_, right)) => {
            expr_at_path(right, tail)
        }
        _ => None,
    }
}

fn build_steps(expr: Expr) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    let mut current = expr;
    loop {
        let highlight = find_reducible(&current);
        steps.push(Step {
            expr: current.clone(),
            highlight: highlight.clone(),
        });
        let Some(path) = highlight else { break };
        current = reduce_at(current, &path)?;
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
        ExprKind::Bool(value) => {
            if *value {
                out.push_str("true");
            } else {
                out.push_str("false");
            }
        }
        ExprKind::Not(inner) => {
            out.push_str("not ");
            let (child_path, child_highlight): (&[PathStep], bool) =
                match highlight_path.split_first() {
                    Some((PathStep::Unary, rest)) => (rest, highlight_enabled),
                    _ => (&[], false),
                };
            render_expr(inner, child_path, child_highlight, out, highlight_range);
        }
        ExprKind::And(left, right) => {
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
            out.push_str(" and ");
            render_expr(right, right_path, right_highlight, out, highlight_range);
            out.push(')');
        }
        ExprKind::Or(left, right) => {
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
            out.push_str(" or ");
            render_expr(right, right_path, right_highlight, out, highlight_range);
            out.push(')');
        }
    }
    let end = out.len();
    if highlight_enabled && highlight_path.is_empty() {
        *highlight_range = Some(start..end);
    }
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
        ExprKind::Bool(value) => {
            let x = *next_leaf_x;
            *next_leaf_x += 1;
            (
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
                Vec::new(),
                x,
            )
        }
        ExprKind::Not(inner) => {
            path.push(PathStep::Unary);
            let child = build_nodes(inner, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = nodes[child].x;
            ("not".to_string(), vec![child], x)
        }
        ExprKind::And(left, right) => {
            path.push(PathStep::Left);
            let left_idx = build_nodes(left, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            path.push(PathStep::Right);
            let right_idx = build_nodes(right, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = (nodes[left_idx].x + nodes[right_idx].x) / 2;
            ("and".to_string(), vec![left_idx, right_idx], x)
        }
        ExprKind::Or(left, right) => {
            path.push(PathStep::Left);
            let left_idx = build_nodes(left, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            path.push(PathStep::Right);
            let right_idx = build_nodes(right, depth + 1, path, highlight_path, nodes, next_leaf_x);
            path.pop();
            let x = (nodes[left_idx].x + nodes[right_idx].x) / 2;
            ("or".to_string(), vec![left_idx, right_idx], x)
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

fn path_in_subtree(path: &[PathStep], subtree: &[PathStep]) -> bool {
    path.len() >= subtree.len() && path[..subtree.len()] == *subtree
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

        let id = ui.make_persistent_id(("bool-tree-node", &layout.path));
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

fn count_ops(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Bool(_) => 0,
        ExprKind::Not(inner) => 1 + count_ops(inner),
        ExprKind::And(left, right) | ExprKind::Or(left, right) => {
            1 + count_ops(left) + count_ops(right)
        }
    }
}

fn eval_expr(expr: &Expr) -> Result<bool, String> {
    match &expr.kind {
        ExprKind::Bool(value) => Ok(*value),
        ExprKind::Not(inner) => Ok(!eval_expr(inner)?),
        ExprKind::And(left, right) => Ok(eval_expr(left)? && eval_expr(right)?),
        ExprKind::Or(left, right) => Ok(eval_expr(left)? || eval_expr(right)?),
    }
}

fn expr_to_string(expr: &Expr) -> String {
    render_expr_with_highlight(expr, None).0
}

fn random_expr(rng: &mut SimpleRng, depth: u8, max_depth: u8) -> Expr {
    let use_literal = depth >= max_depth || rng.gen_range_i32(0, 3) == 0;
    if use_literal {
        let value = rng.gen_range_i32(0, 1) == 1;
        return Expr::boolean(value);
    }

    let roll = rng.gen_range_i32(0, 2);
    if roll == 0 {
        let inner = random_expr(rng, depth + 1, max_depth);
        return Expr {
            kind: ExprKind::Not(Box::new(inner)),
        };
    }

    let left = random_expr(rng, depth + 1, max_depth);
    let right = random_expr(rng, depth + 1, max_depth);
    let kind = if rng.gen_range_i32(0, 1) == 0 {
        ExprKind::And(Box::new(left), Box::new(right))
    } else {
        ExprKind::Or(Box::new(left), Box::new(right))
    };
    Expr { kind }
}

fn generate_exercise(rng: &mut SimpleRng) -> Exercise {
    for _ in 0..200 {
        let expr = random_expr(rng, 0, 3);
        if matches!(expr.kind, ExprKind::Bool(_)) {
            continue;
        }
        if count_ops(&expr) < 2 {
            continue;
        }
        if let Ok(answer) = eval_expr(&expr) {
            return Exercise { expr, answer };
        }
    }
    Exercise {
        expr: Expr {
            kind: ExprKind::And(
                Box::new(Expr::boolean(true)),
                Box::new(Expr::boolean(false)),
            ),
        },
        answer: false,
    }
}

fn generate_tree_expr(rng: &mut SimpleRng) -> Expr {
    for _ in 0..200 {
        let expr = random_expr(rng, 0, 3);
        if matches!(expr.kind, ExprKind::Bool(_)) {
            continue;
        }
        if count_ops(&expr) < 2 {
            continue;
        }
        return expr;
    }
    Expr::boolean(true)
}

pub fn booleans(nb: &mut NotebookCtx) {
    nb.view(|ui| {
        md!(
            ui,
            "# To Bool or Not to Bool\n\\
             A **boolean** is a value with two options.\n\\
             It answers a yes/no question.\n\\
             We will write booleans as `true` and `false`.\n\n\\
             Common pairs that mean the same idea:\n\\
             - yes / no\n\\
             - on / off\n\\
             - true / false\n\\
             - bit (0/1)\n\\
             - thumbs up / thumbs down\n\\
             - open / closed\n\\
             - pass / fail"
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## Why booleans\n\\
             Booleans let us ask questions and make decisions.\n\\
             They are the simplest way to describe a condition.\n\n\\
             Examples:\n\\
             - Is the light on?\n\\
             - Is the number bigger than 10?\n\\
             - Did the user press the button?"
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## Boolean operations\n\\
             We can combine booleans using three simple operations:\n\\
             - **not** flips a value.\n\\
             - **and** needs both sides to be true.\n\\
             - **or** needs at least one side to be true.\n\n\\
             ```text\n\\
             not true  -> false\n\\
             true and false -> false\n\\
             true or false  -> true\n\\
             ```"
        );
    });

    nb.view(|ui| {
        md!(
            ui,
            "## Rules of evaluation\n\\
             When a boolean expression has several operations, there are rules:\n\\
             - Parentheses first: `(true or false) and true`.\n\\
             - Deepest expression first: evaluate the innermost parentheses first.\n\\
             - not before and before or.\n\\
             - Left-to-right when the precedence is the same.\n\n\\
             These rules are called **precedence** and **associativity**.\n\\
             You do not need to memorize the names, just the rules."
        );
    });

    nb.state(
        &chapter_key("expression_state"),
        ExpressionState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Step through a boolean expression").heading());
                ui.add_space(4.0);
                ui.label("Use true/false, and/or/not, and parentheses.");
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
                            RichText::new("Tip: check parentheses or a missing true/false.")
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

    nb.state(&chapter_key("tree_exercise_state"), TreeExerciseState::default(), |ui, state| {
        with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
            ui.label(RichText::new("Tree practice").heading());
            ui.add_space(6.0);
            ui.label("Click a box to evaluate it in the right order (left to right).");
            ui.label("Keep going until the whole tree becomes one value.");
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
                            if matches!(expr.kind, ExprKind::Bool(_)) {
                                Some("Booleans already have a value.".to_string())
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
            if let Some(value) = as_bool(&state.expr) {
                ui.label(format!("All done! Value = {value}."));
            }
            if let Some(feedback) = &state.feedback {
                ui.label(feedback);
            }
        });
    });

    nb.state(
        &chapter_key("random_exercise_state"),
        RandomExerciseState::default(),
        |ui, state| {
            with_padding(ui, DEFAULT_CARD_PADDING, |ui| {
                ui.label(RichText::new("Random practice").heading());
                ui.add_space(6.0);
                ui.label("Evaluate the expression, then choose true or false.");
                ui.add_space(6.0);
                if ui.add(widgets::Button::new("New exercise")).clicked() {
                    state.regenerate();
                }
                ui.add_space(6.0);

                let expression = expr_to_string(&state.exercise.expr);
                code_frame(ui, highlighted_job(ui, &expression, &[]));

                ui.add_space(6.0);
                ui.add(
                    widgets::ChoiceToggle::new(&mut state.selection)
                        .choice(Some(true), "true")
                        .choice(Some(false), "false")
                        .small(),
                );
                ui.add_space(4.0);
                match state.selection {
                    Some(value) if value == state.exercise.answer => ui.label("Correct!"),
                    Some(_) => ui.label("Not quite. Try another answer."),
                    None => ui.label("Pick an answer."),
                }
            });
        },
    );

    nb.view(|ui| {
        md!(
            ui,
            "## What just happened\n\\
             Booleans capture yes/no answers.\n\\
             We can combine them with not, and, and or.\n\\
             Evaluation rules help us compute the final true/false.\n\n\\
             Next up: **Hello, state** uses values that can change over time."
        );
    });
}
