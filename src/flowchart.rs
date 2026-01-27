use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, TextStyle};

use GORBIE::themes;

#[derive(Clone, Copy, Debug)]
pub enum FlowchartNodeKind {
    Start,
    Decision,
    Action,
}

#[derive(Clone, Debug)]
pub struct FlowchartNode {
    pub rect: Rect,
    pub label: String,
    pub kind: FlowchartNodeKind,
    pub active: bool,
}

impl FlowchartNode {
    pub fn new(kind: FlowchartNodeKind, rect: Rect, label: impl Into<String>) -> Self {
        Self {
            rect,
            label: label.into(),
            kind,
            active: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

#[derive(Clone, Debug)]
pub struct FlowchartEdge {
    pub points: Vec<Pos2>,
    pub active: bool,
}

pub struct Flowchart {
    pub rect: Rect,
    pub nodes: Vec<FlowchartNode>,
    pub edges: Vec<FlowchartEdge>,
}

pub struct FlowchartStyle {
    pub font_id: FontId,
    pub text_color: Color32,
    pub node_corner_radius: f32,
    pub edge_corner_radius: f32,
    pub start_radius: f32,
    pub node_fill: Color32,
    pub active_node_fill: Color32,
    pub node_stroke: Stroke,
    pub active_edge_stroke: Stroke,
    pub inactive_edge_stroke: Stroke,
}

impl FlowchartStyle {
    pub fn from_ui(ui: &egui::Ui) -> Self {
        let background = ui.visuals().window_fill;
        let outline = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let active = themes::ral(2009);
        let inactive = themes::blend(background, outline, 0.55);
        let font_id = TextStyle::Monospace.resolve(ui.style());
        let edge_width: f32 = 2.5;
        Self {
            font_id,
            text_color: ui.visuals().text_color(),
            node_corner_radius: 0.0,
            edge_corner_radius: 9.0,
            start_radius: edge_width * 2.5,
            node_fill: background,
            active_node_fill: themes::blend(background, active, 0.12),
            node_stroke: Stroke::new(1.0, outline),
            active_edge_stroke: Stroke::new(edge_width, active),
            inactive_edge_stroke: Stroke::new(edge_width, inactive),
        }
    }
}

pub fn paint_flowchart(ui: &egui::Ui, chart: &Flowchart, style: &FlowchartStyle) {
    let painter = ui.painter_at(chart.rect);
    for edge in &chart.edges {
        let stroke = if edge.active {
            style.active_edge_stroke
        } else {
            style.inactive_edge_stroke
        };
        paint_polyline(&painter, &edge.points, stroke, style.edge_corner_radius);
    }

    for node in &chart.nodes {
        match node.kind {
            FlowchartNodeKind::Start => {
                let center = node.rect.center();
                let fill = if node.active {
                    style.active_edge_stroke.color
                } else {
                    style.inactive_edge_stroke.color
                };
                painter.circle_filled(center, style.start_radius, fill);
            }
            FlowchartNodeKind::Decision | FlowchartNodeKind::Action => {
                let fill = if node.active {
                    style.active_node_fill
                } else {
                    style.node_fill
                };
                painter.rect_filled(node.rect, style.node_corner_radius, fill);
                painter.rect_stroke(
                    node.rect,
                    style.node_corner_radius,
                    style.node_stroke,
                    egui::StrokeKind::Inside,
                );
                if node.active {
                    let inner_rect = node.rect.shrink(2.0);
                    if inner_rect.is_positive() {
                        painter.rect_stroke(
                            inner_rect,
                            style.node_corner_radius,
                            style.node_stroke,
                            egui::StrokeKind::Inside,
                        );
                    }
                }
                if !node.label.is_empty() {
                    painter.text(
                        node.rect.center(),
                        Align2::CENTER_CENTER,
                        &node.label,
                        style.font_id.clone(),
                        style.text_color,
                    );
                }
            }
        }
    }
}

fn paint_polyline(
    painter: &Painter,
    points: &[Pos2],
    stroke: Stroke,
    corner_radius: f32,
) {
    if points.len() < 2 {
        return;
    }
    if corner_radius <= 0.0 || points.len() < 3 {
        for segment in points.windows(2) {
            painter.line_segment([segment[0], segment[1]], stroke);
        }
        return;
    }

    let mut previous = points[0];
    for idx in 1..points.len() - 1 {
        let corner = points[idx];
        let next = points[idx + 1];

        let incoming = corner - previous;
        let outgoing = next - corner;
        let incoming_len = incoming.length();
        let outgoing_len = outgoing.length();
        if incoming_len <= 0.5 || outgoing_len <= 0.5 {
            painter.line_segment([previous, corner], stroke);
            previous = corner;
            continue;
        }

        let dir_in = incoming / incoming_len;
        let dir_out = outgoing / outgoing_len;
        let dot = (dir_in.dot(dir_out)).clamp(-1.0, 1.0);
        if dot.abs() >= 0.999 {
            painter.line_segment([previous, corner], stroke);
            previous = corner;
            continue;
        }

        let radius = corner_radius
            .min(incoming_len * 0.5)
            .min(outgoing_len * 0.5);
        if radius <= 0.5 {
            painter.line_segment([previous, corner], stroke);
            previous = corner;
            continue;
        }

        let entry = corner - dir_in * radius;
        let exit = corner + dir_out * radius;
        if previous.distance(entry) > 0.1 {
            painter.line_segment([previous, entry], stroke);
        }

        let angle = dot.acos();
        let handle = (4.0 / 3.0) * (angle / 4.0).tan() * radius;
        let ctrl1 = entry + dir_in * handle;
        let ctrl2 = exit - dir_out * handle;
        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            [entry, ctrl1, ctrl2, exit],
            false,
            Color32::TRANSPARENT,
            stroke,
        ));

        previous = exit;
    }

    let last = *points.last().expect("points checked above");
    painter.line_segment([previous, last], stroke);
}
