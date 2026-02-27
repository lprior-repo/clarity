use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DependencyGraph {
  adjacency: HashMap<String, Vec<String>>,
  nodes: HashSet<String>,
}

impl DependencyGraph {
  #[must_use]
  pub fn new() -> Self {
    Self {
      adjacency: HashMap::new(),
      nodes: HashSet::new(),
    }
  }

  pub fn add_node(&mut self, node: String) {
    self.nodes.insert(node.clone());
    self.adjacency.entry(node).or_default();
  }

  pub fn add_edge(&mut self, from: String, to: String) {
    self.nodes.insert(from.clone());
    self.nodes.insert(to.clone());
    self.adjacency.entry(from).or_default().push(to.clone());
    self.adjacency.entry(to).or_default();
  }

  #[must_use]
  pub fn detect_cycles(&self) -> Option<Vec<String>> {
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    let mut path = Vec::new();

    for node in &self.nodes {
      if !visited.contains(node) {
        if let Some(cycle) =
          self.dfs_cycle_detect(node, &mut visited, &mut recursion_stack, &mut path)
        {
          return Some(cycle);
        }
      }
    }

    None
  }

  fn dfs_cycle_detect(
    &self,
    node: &str,
    visited: &mut HashSet<String>,
    recursion_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
  ) -> Option<Vec<String>> {
    visited.insert(node.to_string());
    recursion_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = self.adjacency.get(node) {
      for neighbor in neighbors {
        if !visited.contains(neighbor) {
          if let Some(cycle) = self.dfs_cycle_detect(neighbor, visited, recursion_stack, path) {
            return Some(cycle);
          }
        } else if recursion_stack.contains(neighbor) {
          let cycle_start = path.iter().position(|entry| entry == neighbor)?;
          return Some(path[cycle_start..].to_vec());
        }
      }
    }

    recursion_stack.remove(node);
    path.pop();
    None
  }

  #[must_use]
  pub fn topological_sort(&self) -> Option<Vec<String>> {
    let mut in_degree: HashMap<String, usize> =
      self.nodes.iter().map(|node| (node.clone(), 0)).collect();

    for deps in self.adjacency.values() {
      for dep in deps {
        if let Some(count) = in_degree.get_mut(dep) {
          *count += 1;
        }
      }
    }

    let mut queue: Vec<String> = in_degree
      .iter()
      .filter(|(_, degree)| **degree == 0)
      .map(|(node, _)| node.clone())
      .collect();

    let mut result = Vec::new();
    while let Some(node) = queue.pop() {
      result.push(node.clone());
      if let Some(neighbors) = self.adjacency.get(&node) {
        for neighbor in neighbors {
          if let Some(degree) = in_degree.get_mut(neighbor) {
            *degree -= 1;
            if *degree == 0 {
              queue.push(neighbor.clone());
            }
          }
        }
      }
    }

    if result.len() == self.nodes.len() {
      result.reverse();
      Some(result)
    } else {
      None
    }
  }
}

impl Default for DependencyGraph {
  fn default() -> Self {
    Self::new()
  }
}
