/*
 *
 * Christopher Piraino
 *
 * Dijkstra's Algorithm in Rust
 *
 * Using Fibonacci heaps.
 */

// extern crate core;
extern crate rust_heaps;
use rust_heaps::fibonacci_heap::FibHeap;
use rust_heaps::{Heap};
use std::usize;
use std::rc::Rc;
use std::cmp::Ordering;
use std::cell::{RefMut, Ref, RefCell};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

static INFINITY: u64 = usize::MAX as u64;

#[derive(Clone, Debug)]
struct Node {
    inner: RefCell<InnerNode>,
}

#[derive(Clone, Debug)]
struct InnerNode {
    id: u64,
    edges: Vec<Edge>, // A node only holds edges where it is the source.
    previous: Option<Rc<Node>>,
    distance: u64,
    visited: bool,
}

impl Node {
    pub fn new(id: u64) -> Rc<Node> {
        let inner = RefCell::new(InnerNode {
            id: id,
            edges: Vec::new(),
            previous: None,
            distance: INFINITY,
            visited: false
        });
        Rc::new(Node { inner: inner })
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, InnerNode> {
        self.inner.borrow_mut()
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, InnerNode> {
        self.inner.borrow()
    }
}

impl Hash for Node {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        self.inner.borrow().id.hash(state);
    }
}
impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.inner.borrow().id == other.inner.borrow().id
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        self.inner.borrow().id.partial_cmp(&other.inner.borrow().id)
    }
}

#[derive(Debug, Clone)]
struct Edge {
    source: Rc<Node>,
    target: Rc<Node>,
    cost: u64
}

// impl fmt::Debug for Edge {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_struct("Edge")
//             .field("source", &self.source.borrow().id)
//             .field("target", &self.target.borrow().id)
//             .field("cost", &self.cost)
//             .finish()
//     }
// }

fn shortest_path<H: Heap<u64, Rc<Node>>>(pq: &mut H,
                                                  graph: Vec<Rc<Node>>,
                                                  start: Rc<Node>,
                                                  stop: Rc<Node>) -> Vec<Rc<Node>> {
    let mut node_map = HashMap::new();
    start.borrow_mut().distance = 0;
    for n in graph.into_iter() {
        let node = pq.insert(n.borrow().distance, n.clone());
        node_map.insert(n, node);
    }

    while !pq.empty() {
        let (distance, node) = pq.delete_min();
        if node.borrow().id == stop.borrow().id {
            break;
        }
        node.borrow_mut().visited = true;
        for e in node.borrow().edges.iter() {
            if !e.target.borrow().visited {
                let new_dist = distance + e.cost;
                if new_dist < e.target.borrow().distance {
                    let old_dist = e.target.borrow().distance;
                    {
                        e.target.borrow_mut().distance = new_dist;
                        let mut target = e.target.borrow_mut();
                        target.previous = Some(node.clone());
                    }
                    let fibnode = node_map.get(&e.target).unwrap();
                    pq.decrease_key(fibnode, old_dist - new_dist);
                }
            }
        }
    }

    let mut path = Vec::new();
    construct_path(&mut path, stop);
    path
}

fn construct_path(path: &mut Vec<Rc<Node>>, node: Rc<Node>) {
    if node.borrow().distance == 0 {
        path.push(node.clone());
        return
    }
    construct_path(path, node.borrow().previous.clone().unwrap());
    path.push(node.clone());
}

fn main() {
    let n1 = Node::new(1);
    let n2 = Node::new(2);
    let n3 = Node::new(3);
    let n4 = Node::new(4);
    n1.borrow_mut().edges.push(Edge {
        source: n1.clone(),
        target: n2.clone(),
        cost: 1
    });
    n1.borrow_mut().edges.push(Edge {
        source: n1.clone(),
        target: n3.clone(),
        cost: 5
    });
    n2.borrow_mut().edges.push(Edge {
        source: n2.clone(),
        target: n3.clone(),
        cost: 8
    });
    n2.borrow_mut().edges.push(Edge {
        source: n2.clone(),
        target: n4.clone(),
        cost: 3
    });
    n3.borrow_mut().edges.push(Edge {
        source: n3.clone(),
        target: n4.clone(),
        cost: 2
    });
    n4.borrow_mut().edges.push(Edge {
        source: n4.clone(),
        target: n2.clone(),
        cost: 9
    });
    let n1c = n1.clone();
    let n4c = n4.clone();
    let graph = vec!(n1, n2, n3, n4);
    let mut heap = FibHeap::new();
    let shortest = shortest_path(&mut heap, graph, n1c, n4c);
    // Shortest path is 1 -> 2 -> 4
    for n in shortest.iter() {
        println!("Node: {:?}", n.borrow().id);
    }
}
