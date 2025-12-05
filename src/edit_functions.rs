use macroquad::prelude::*;
use crate::graph::Graph;
use crate::Nodes;

pub fn create_nodes_neighbors(graph: &mut Graph, nodes: &mut Nodes, mouse_pos: IVec2, radius: f32) {
  // Place a node if it won't overlap existing nodes. If hovering node, select it.
  if is_mouse_button_pressed(MouseButton::Left) {
    if graph.node_at(mouse_pos, radius).is_none() {
      nodes.selected = Some(graph.add_node(mouse_pos));
    }
    nodes.selected = nodes.hovering;
  }

  // Draw line from selected node to mouse
  if is_mouse_button_down(MouseButton::Left) {
    if let Some(node) = nodes.selected {
      let origin = graph.nodes.get(node).unwrap().position;
      draw_line(
        mouse_pos.x as f32, mouse_pos.y as f32,
        origin.x as f32, origin.y as f32,
        4., WHITE
      );
    }
  }

  // Assign neighbor if starts and ends on unique nodes
  if is_mouse_button_released(MouseButton::Left) {
    if let Some(node1) = nodes.selected && let Some(node2) = nodes.hovering && node1 != node2 {
      graph.add_connection(node1, node2);
    }
    nodes.selected = None;
  }

}

pub fn drag_nodes(graph: &mut Graph, nodes: &mut Nodes, mouse_pos: IVec2) {
  if is_mouse_button_pressed(MouseButton::Left) {
    nodes.selected = nodes.hovering;
  }
  if is_mouse_button_down(MouseButton::Left) {
    if let Some(dragging) = nodes.selected {
      graph.nodes.get_mut(dragging).unwrap().position = mouse_pos;
    }
  }
  if is_mouse_button_released(MouseButton::Left) {
    nodes.selected = None;
  }
}

pub fn remove_node(graph: &mut Graph, nodes: &mut Nodes) {
  if is_mouse_button_down(MouseButton::Left) {
    if let Some(remove) = nodes.hovering {
      graph.remove(remove);
    }
  }
}

pub fn modify(graph: &mut Graph, nodes: &mut Nodes, modify_val: &String, max: &String) {
  if let Ok(delta) = modify_val.parse::<i8>()
    && let Ok(max) = max.parse::<u8>()
      && let Some(node) = nodes.hovering
      && is_mouse_button_released(MouseButton::Left) {
        graph.clamped_update(node, delta, max);
  }
}

pub fn set(graph: &mut Graph, nodes: &mut Nodes, modify_val: &String, max: &String) {
  if let Ok(value) = modify_val.parse::<u8>()
    && let Ok(max) = max.parse::<u8>()
      && let Some(node) = nodes.hovering
      && is_mouse_button_down(MouseButton::Left) {
        graph.nodes.get_mut(node).unwrap().value = value.min(max);
  }
}
