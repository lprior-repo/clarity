#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::types::Answer;

/// Graph node with owned data
#[derive(Clone, Debug)]
struct GraphNode {
    id: String,
    label: String,
    group: String,
    x: f64,
    y: f64,
}

/// Edge between nodes
#[derive(Clone, Debug)]
struct GraphEdge {
    from: String,
    to: String,
}

/// Node rendering data
#[derive(Clone, Debug)]
struct NodeRenderData {
    node: GraphNode,
    color: String,
    bg: String,
    text_y: f64,
}

/// Edge rendering data
#[derive(Clone, Debug)]
struct EdgeRenderData {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

/// Legend item data
#[derive(Clone, Debug)]
struct LegendItem {
    group: String,
    color: String,
}

/// Parse lines from text
fn parse_lines(text: Option<String>) -> Vec<String> {
    text.map(|t| {
        t.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect()
    })
    .unwrap_or_default()
}

/// Get answer value by step ID
fn get_val(answers: &[Answer], id: &str) -> Option<String> {
    answers
        .iter()
        .find(|a| a.step_id == id && a.value != "(skipped)")
        .map(|a| a.value.clone())
}

/// Group colors
fn group_color(group: &str) -> (String, String) {
    match group {
        "thesis" => ("hsl(221, 83%, 53%)".into(), "hsl(221, 83%, 53%, 0.15)".into()),
        "persona" => ("hsl(262, 83%, 58%)".into(), "hsl(262, 83%, 58%, 0.15)".into()),
        "scenario" => ("hsl(142, 71%, 45%)".into(), "hsl(142, 71%, 45%, 0.15)".into()),
        "usecase" => ("hsl(38, 92%, 50%)".into(), "hsl(38, 92%, 50%, 0.15)".into()),
        "task" => ("hsl(0, 72%, 51%)".into(), "hsl(0, 72%, 51%, 0.15)".into()),
        _ => ("hsl(0, 0%, 80%)".into(), "hsl(0, 0%, 80%, 0.15)".into()),
    }
}

/// Build graph nodes and edges from answers
fn build_graph_data(answers: &[Answer]) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    let problem = get_val(answers, "problem");
    let antithesis = get_val(answers, "antithesis");
    let solution = get_val(answers, "solution");

    if problem.is_some() {
        nodes.push(GraphNode {
            id: "problem".into(),
            label: "Problem".into(),
            group: "thesis".into(),
            x: 260.0,
            y: 60.0,
        });
    }

    if antithesis.is_some() {
        nodes.push(GraphNode {
            id: "antithesis".into(),
            label: "Antithesis".into(),
            group: "thesis".into(),
            x: 340.0,
            y: 60.0,
        });
        edges.push(GraphEdge {
            from: "problem".into(),
            to: "antithesis".into(),
        });
    }

    if solution.is_some() {
        nodes.push(GraphNode {
            id: "solution".into(),
            label: "Solution".into(),
            group: "thesis".into(),
            x: 300.0,
            y: 130.0,
        });
        if problem.is_some() {
            edges.push(GraphEdge {
                from: "problem".into(),
                to: "solution".into(),
            });
        }
        if antithesis.is_some() {
            edges.push(GraphEdge {
                from: "antithesis".into(),
                to: "solution".into(),
            });
        }
    }

    let persona = get_val(answers, "persona");
    if persona.is_some() {
        nodes.push(GraphNode {
            id: "persona".into(),
            label: "User".into(),
            group: "persona".into(),
            x: 140.0,
            y: 200.0,
        });
        if solution.is_some() {
            edges.push(GraphEdge {
                from: "solution".into(),
                to: "persona".into(),
            });
        }
    }

    let scenario = get_val(answers, "scenario");
    if scenario.is_some() {
        nodes.push(GraphNode {
            id: "scenario".into(),
            label: "North Star".into(),
            group: "scenario".into(),
            x: 460.0,
            y: 200.0,
        });
        if persona.is_some() {
            edges.push(GraphEdge {
                from: "persona".into(),
                to: "scenario".into(),
            });
        }
        if solution.is_some() {
            edges.push(GraphEdge {
                from: "solution".into(),
                to: "scenario".into(),
            });
        }
    }

    let use_cases = parse_lines(get_val(answers, "use-cases"));
    let uc_start = 300.0 - ((use_cases.len() as f64 - 1.0) * 70.0) / 2.0;

    for (i, uc) in use_cases.iter().enumerate() {
        let id = format!("uc-{}", i);
        let short = if uc.len() > 20 {
            format!("{}..", &uc[..18])
        } else {
            uc.clone()
        };
        nodes.push(GraphNode {
            id: id.clone(),
            label: short,
            group: "usecase".into(),
            x: uc_start + (i as f64) * 70.0,
            y: 300.0,
        });
        if scenario.is_some() {
            edges.push(GraphEdge {
                from: "scenario".into(),
                to: id,
            });
        }
    }

    let tasks = parse_lines(get_val(answers, "tasks"));
    let t_start = 300.0 - ((tasks.len() as f64 - 1.0) * 60.0) / 2.0;

    for (i, t) in tasks.iter().enumerate() {
        let id = format!("task-{}", i);
        let parts: Vec<&str> = t.splitn(2, ':').collect();
        let short = if parts.len() > 1 {
            parts[0].trim().to_string()
        } else {
            t[..t.len().min(14)].to_string()
        };
        nodes.push(GraphNode {
            id: id.clone(),
            label: short,
            group: "task".into(),
            x: t_start + (i as f64) * 60.0,
            y: 400.0,
        });

        if !use_cases.is_empty() {
            let uc_idx = i.min(use_cases.len() - 1);
            edges.push(GraphEdge {
                from: format!("uc-{}", uc_idx),
                to: id,
            });
        } else if scenario.is_some() {
            edges.push(GraphEdge {
                from: "scenario".into(),
                to: id,
            });
        }
    }

    (nodes, edges)
}

/// Build node render data
fn build_node_render_data(nodes: &[GraphNode]) -> Vec<NodeRenderData> {
    nodes
        .iter()
        .map(|node| {
            let (color, bg) = group_color(&node.group);
            NodeRenderData {
                node: node.clone(),
                color,
                bg,
                text_y: node.y + 32.0,
            }
        })
        .collect()
}

/// Build edge render data
fn build_edge_render_data(edges: &[GraphEdge], nodes: &[GraphNode]) -> Vec<EdgeRenderData> {
    edges
        .iter()
        .filter_map(|edge| {
            let from_node = nodes.iter().find(|n| n.id == edge.from)?;
            let to_node = nodes.iter().find(|n| n.id == edge.to)?;
            Some(EdgeRenderData {
                x1: from_node.x,
                y1: from_node.y,
                x2: to_node.x,
                y2: to_node.y,
            })
        })
        .collect()
}

/// Build legend items
fn build_legend_items() -> Vec<LegendItem> {
    ["thesis", "persona", "scenario", "usecase", "task"]
        .iter()
        .map(|group| LegendItem {
            group: String::from(*group),
            color: group_color(group).0,
        })
        .collect()
}

/// Render an edge line
fn render_edge(edge: &EdgeRenderData, idx: usize) -> Element {
    rsx! {
        line {
            key: "{idx}",
            x1: "{edge.x1}",
            y1: "{edge.y1}",
            x2: "{edge.x2}",
            y2: "{edge.y2}",
            stroke: "hsl(0, 0%, 20%)",
            "stroke-width": "1"
        }
    }
}

/// Render a node group
fn render_node(data: &NodeRenderData) -> Element {
    rsx! {
        g {
            key: "{data.node.id}",
            circle {
                cx: "{data.node.x}",
                cy: "{data.node.y}",
                r: "18",
                fill: "{data.bg}",
                stroke: "{data.color}",
                "stroke-width": "1.5"
            }
            text {
                x: "{data.node.x}",
                y: "{data.text_y}",
                "text-anchor": "middle",
                fill: "hsl(0, 0%, 80%)",
                "font-size": "10",
                "font-family": "system-ui, sans-serif",
                "{data.node.label}"
            }
        }
    }
}

/// Render a legend item
fn render_legend_item(item: &LegendItem) -> Element {
    rsx! {
        div { class: "flex items-center gap-1.5",
            span {
                class: "inline-block h-2.5 w-2.5 rounded-full",
                style: format!("background-color: {}", item.color)
            }
            span {
                class: "font-mono text-xs capitalize text-muted-foreground/60",
                "{item.group}"
            }
        }
    }
}

/// GraphVisualizer component - simplified SVG-based graph visualization
#[component]
pub fn GraphVisualizer(answers: Signal<Vec<Answer>>) -> Element {
    let answers_guard = answers.read();
    let (nodes, edges) = build_graph_data(&answers_guard);
    drop(answers_guard);

    if nodes.is_empty() {
        return rsx! {
            div { class: "flex h-full items-center justify-center",
                p { class: "text-sm text-muted-foreground/40", "Answer questions to see your plan graph grow" }
            }
        };
    }

    let node_render_data = build_node_render_data(&nodes);
    let edge_render_data = build_edge_render_data(&edges, &nodes);
    let legend_items = build_legend_items();

    let edge_elements: Vec<Element> = edge_render_data
        .iter()
        .enumerate()
        .map(|(idx, edge)| render_edge(edge, idx))
        .collect();

    let node_elements: Vec<Element> = node_render_data.iter().map(render_node).collect();

    let legend_elements: Vec<Element> = legend_items.iter().map(render_legend_item).collect();

    rsx! {
        div { class: "relative h-full w-full overflow-auto bg-[hsl(0,0%,2%)]",
            svg {
                view_box: "0 0 600 450",
                class: "w-full h-full",
                preserve_aspect_ratio: "xMidYMid meet",

                // Draw edges
                for edge in edge_elements.iter() {
                    {edge.clone()}
                }

                // Draw nodes
                for node in node_elements.iter() {
                    {node.clone()}
                }
            }

            // Legend
            div {
                class: "absolute bottom-3 left-3 flex flex-wrap gap-3",
                for item in legend_elements.iter() {
                    {item.clone()}
                }
            }
        }
    }
}
