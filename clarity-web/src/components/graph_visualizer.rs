#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::lattice::effects::trace_effects;
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
        "effect-root" => ("hsl(173, 80%, 40%)".into(), "hsl(173, 80%, 40%, 0.15)".into()),
        "effect-leaf" => ("hsl(280, 70%, 55%)".into(), "hsl(280, 70%, 55%, 0.15)".into()),
        "effect" => ("hsl(200, 70%, 50%)".into(), "hsl(200, 70%, 50%, 0.15)".into()),
        _ => ("hsl(0, 0%, 80%)".into(), "hsl(0, 0%, 80%, 0.15)".into()),
    }
}

/// Build graph nodes and edges from effects analysis
fn build_effects_graph(answers: &[Answer]) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    // Extract solution text for effects analysis
    let solution_text: String = answers
        .iter()
        .filter(|a| {
            let step = a.step_id.to_lowercase();
            step.contains("solution") || step.contains("approach")
        })
        .map(|a| a.value.as_str())
        .collect::<Vec<_>>()
        .join(". ");

    if solution_text.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let effects_output = trace_effects(&solution_text);

    // Convert effects dependency nodes/edges to graph format
    let nodes: Vec<GraphNode> = effects_output
        .nodes
        .iter()
        .enumerate()
        .map(|(i, dep_node)| {
            let x = 300.0 + ((i as f64 - effects_output.nodes.len() as f64 / 2.0) * 100.0);
            let y = 200.0;

            GraphNode {
                id: dep_node.id.clone(),
                label: dep_node.label.clone(),
                group: if dep_node.is_root {
                    "effect-root".to_string()
                } else if dep_node.is_leaf {
                    "effect-leaf".to_string()
                } else {
                    "effect".to_string()
                },
                x,
                y,
            }
        })
        .collect();

    let edges: Vec<GraphEdge> = effects_output
        .edges
        .iter()
        .map(|dep_edge| GraphEdge {
            from: dep_edge.from.clone(),
            to: dep_edge.to.clone(),
        })
        .collect();

    (nodes, edges)
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

    // Try effects-based graph first, fall back to basic graph
    let effects_output = trace_effects(&answers_guard
        .iter()
        .filter(|a| {
            let step = a.step_id.to_lowercase();
            step.contains("solution") || step.contains("approach")
        })
        .map(|a| a.value.as_str())
        .collect::<Vec<_>>()
        .join(". "));

    let (effects_nodes, effects_edges) = build_effects_graph(&answers_guard);
    let (nodes, edges, use_effects) = if !effects_nodes.is_empty() {
        (effects_nodes, effects_edges, true)
    } else {
        let (basic_nodes, basic_edges) = build_graph_data(&answers_guard);
        (basic_nodes, basic_edges, false)
    };

    let warnings = effects_output.warnings;

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
    let legend_items = if use_effects {
        vec![
            LegendItem { group: "Causal Root".to_string(), color: group_color("effect-root").0 },
            LegendItem { group: "Causal Link".to_string(), color: group_color("effect").0 },
            LegendItem { group: "Outcome".to_string(), color: group_color("effect-leaf").0 },
        ]
    } else {
        build_legend_items()
    };

    let edge_elements: Vec<Element> = edge_render_data
        .iter()
        .enumerate()
        .map(|(idx, edge)| render_edge(edge, idx))
        .collect();

    let node_elements: Vec<Element> = node_render_data.iter().map(render_node).collect();

    let legend_elements: Vec<Element> = legend_items.iter().map(render_legend_item).collect();

    rsx! {
        div { class: "flex h-full w-full flex-col",
            // Header showing graph type
            div { class: "shrink-0 border-b border-border px-4 py-2",
                div { class: "flex items-center justify-between",
                    span {
                        class: "text-xs font-medium uppercase tracking-wider text-muted-foreground/70",
                        if use_effects { "Cusal Dependency Graph" } else { "Planning Graph" }
                    }
                    if !warnings.is_empty() {
                        span {
                            class: "flex items-center gap-1 rounded-full bg-chart-4/10 px-2 py-0.5 text-xs text-chart-4",
                            "{warnings.len()} warning(s)"
                        }
                    }
                }
            }

            // Graph area
            div { class: "relative flex-1 overflow-auto bg-[hsl(0,0%,2%)]",
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

            // Warnings section
            if !warnings.is_empty() {
                div { class: "shrink-0 border-t border-border bg-chart-4/5 px-4 py-2",
                    div { class: "space-y-1",
                        for warning in warnings.iter() {
                            div { class: "flex items-start gap-2 text-xs text-chart-4",
                                svg {
                                    width: "12",
                                    height: "12",
                                    view_box: "0 0 12 12",
                                    fill: "none",
                                    class: "shrink-0 mt-0.5",
                                    path {
                                        d: "M6 1C3.2 1 1 3.2 1 6C1 8.8 3.2 11 6 11C8.8 11 11 8.8 11 6C11 3.2 8.8 1 6 1ZM6 8.5C5.4 8.5 5 8.1 5 7.5C5 6.9 5.4 6.5 6 6.5C6.6 6.5 7 6.9 7 7.5C7 8.1 6.6 8.5 6 8.5ZM6 5.5C5.4 5.5 5 5.1 5 4.5V3C5 2.4 5.4 2 6 2C6.6 2 7 2.4 7 3V4.5C7 5.1 6.6 5.5 6 5.5Z",
                                        fill: "currentColor"
                                    }
                                }
                                "{warning}"
                            }
                        }
                    }
                }
            }
        }
    }
}
