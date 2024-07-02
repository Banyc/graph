use std::collections::VecDeque;

use slotmap::{new_key_type, SecondaryMap, SlotMap};

pub type NodeArray<T> = SlotMap<NodeIdx, T>;

new_key_type! {
    pub struct NodeIdx;
}

#[derive(Debug, Clone)]
pub struct Graph<T> {
    nodes: NodeArray<T>,
}
impl<T> Graph<T> {
    pub fn new(nodes: NodeArray<T>) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> &NodeArray<T> {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut NodeArray<T> {
        &mut self.nodes
    }
}

pub trait Node {
    fn children(&self) -> &[NodeIdx];
}

pub fn to_dot<T: Node>(graph: &Graph<T>) -> String {
    let mut dot = String::new();
    dot.push_str("digraph {\n");
    for (i, node) in graph.nodes() {
        for &child in node.children() {
            dot.push_str(&format!("\"{i:?}\" -> \"{child:?}\"\n"));
        }
    }
    dot.push('}');
    dot
}

/// Return the visited nodes in pre-order
///
/// A node can be visited more than once
pub fn depth_first_search<T: Node>(graph: &Graph<T>, starts: &[NodeIdx]) -> Vec<NodeIdx> {
    let mut in_stack = SecondaryMap::new();
    let mut stack = vec![];
    for &start in starts {
        stack.push(start);
        in_stack.insert(start, ());
    }
    let mut visit = vec![];
    while let Some(node) = stack.pop() {
        in_stack.remove(node);
        visit.push(node);
        for &child in graph.nodes().get(node).unwrap().children() {
            if in_stack.contains_key(child) {
                continue;
            }
            stack.push(child);
            in_stack.insert(child, ());
        }
    }
    visit
}

/// A node can be visited at most once
pub fn dependency_order<T: Node>(graph: &Graph<T>, starts: &[NodeIdx]) -> Vec<NodeIdx> {
    #[derive(Debug, Clone, Copy)]
    struct Edge {
        pub parent: Option<NodeIdx>,
        pub child: NodeIdx,
    }
    let mut pending_children = SecondaryMap::new();
    let mut visited = SecondaryMap::new();
    let mut stack = vec![];
    let mut visit = vec![];
    for &start in starts {
        stack.push(Edge {
            parent: None,
            child: start,
        });
    }
    while let Some(edge) = stack.pop() {
        let node = edge.child;
        if !pending_children.contains_key(node) {
            pending_children.insert(node, graph.nodes().get(node).unwrap().children().len());
        }
        if *pending_children.get(node).unwrap() == 0 {
            if !visited.contains_key(node) {
                visit.push(node);
                visited.insert(node, ());
            }
            let Some(parent) = edge.parent else {
                continue;
            };
            *pending_children.get_mut(parent).unwrap() -= 1;
            continue;
        }
        stack.push(edge);
        for &child in graph.nodes().get(node).unwrap().children() {
            if pending_children.contains_key(child) {
                assert_eq!(*pending_children.get(child).unwrap(), 0);
                *pending_children.get_mut(node).unwrap() -= 1;
                continue;
            }
            stack.push(Edge {
                parent: Some(node),
                child,
            });
        }
    }
    visit
}

#[derive(Debug)]
pub struct VisitParams<'a, T> {
    pub graph: &'a mut Graph<T>,
    pub node: NodeIdx,
}
#[derive(Debug, Clone)]
pub enum NextMove {
    Postpone,
    TerminateBranch,
    VisitChildren,
}

pub fn breath_first_search<T: Node>(
    graph: &mut Graph<T>,
    start: NodeIdx,
    visit: &mut impl FnMut(VisitParams<'_, T>) -> NextMove,
) {
    let mut in_queue = SecondaryMap::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    in_queue.insert(start, ());
    while let Some(node) = queue.pop_front() {
        in_queue.remove(node);
        let params = VisitParams { graph, node };
        let next_move = visit(params);
        match next_move {
            NextMove::Postpone => {
                queue.push_back(node);
                in_queue.insert(node, ());
                continue;
            }
            NextMove::TerminateBranch => continue,
            NextMove::VisitChildren => (),
        }
        for &child in graph.nodes().get(node).unwrap().children() {
            if in_queue.contains_key(child) {
                continue;
            }
            queue.push_back(child);
            in_queue.insert(child, ());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;

    use bumpalo::{collections::Vec as BumpVec, Bump};

    pub struct NodeA<'b> {
        pub children: BumpVec<'b, NodeIdx>,
    }
    impl Node for NodeA<'_> {
        fn children(&self) -> &[NodeIdx] {
            &self.children
        }
    }

    #[test]
    fn test_bfs() {
        let arena = Bump::new();
        let mut nodes = NodeArray::with_key();
        let node = NodeA {
            children: bumpalo::vec![in &arena;],
        };
        let node = nodes.insert(node);
        let node = NodeA {
            children: bumpalo::vec![in &arena; node],
        };
        let node = nodes.insert(node);
        let mut graph = Graph::new(nodes);

        let mut visit = |params: VisitParams<'_, NodeA>| {
            let _children = params.graph.nodes().get(params.node).unwrap();
            NextMove::VisitChildren
        };
        breath_first_search(&mut graph, node, &mut visit);
    }

    #[test]
    fn test_ref_cell() {
        let arena = Bump::new();
        let mut param_map: HashMap<String, RefCell<BumpVec<'_, f64>>> = HashMap::new();
        let params = bumpalo::vec![in &arena; 0., 1., -1.];
        let params = RefCell::new(params);
        param_map.insert("a".into(), params);
    }
}
