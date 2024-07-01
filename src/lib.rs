use std::collections::VecDeque;

use slotmap::{new_key_type, SecondaryMap, SlotMap};

new_key_type! {
    pub struct NodeIdx;
}

#[derive(Debug, Clone)]
pub struct Graph<T> {
    nodes: SlotMap<NodeIdx, T>,
}
impl<T> Graph<T> {
    pub fn new(nodes: SlotMap<NodeIdx, T>) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> &SlotMap<NodeIdx, T> {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut SlotMap<NodeIdx, T> {
        &mut self.nodes
    }
}

pub trait Node {
    fn children(&self) -> &[NodeIdx];
}

/// Return the visited nodes in pre-order
///
/// Each node is visited at most once
pub fn depth_first_search<T: Node>(graph: &Graph<T>, start: NodeIdx) -> Vec<NodeIdx> {
    let mut met = SecondaryMap::new();
    let mut stack = vec![];
    stack.push(start);
    met.insert(start, ());
    let mut visit = vec![];
    while let Some(node) = stack.pop() {
        visit.push(node);
        for &child in graph.nodes().get(node).unwrap().children() {
            if met.contains_key(child) {
                continue;
            }
            stack.push(child);
            met.insert(child, ());
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
    Noop,
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
            NextMove::Noop => continue,
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
        let mut nodes = SlotMap::with_key();
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
}
