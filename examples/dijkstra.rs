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
use std::fmt;
use std::uint;
use std::rc::Rc;
use std::cell::{RefCell};
use std::hash::Hash;
use std::hash::sip::SipState;

static INFINITY: u64 = uint::MAX as u64;

#[deriving(Clone, Show)]
struct Node {
    id: u64,
    edges: Vec<Edge>, // A node only holds edges were it is the source.
    previous: Option<Rc<RefCell<Node>>>,
    distance: u64,
    visited: bool,
}

impl fmt::Show for RefCell<Node> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Node ( "));
        try!(write!(f, "id: {}, ", self.borrow().id));
        write!(f, "distance: {} )", self.borrow().distance)
    }
}

impl Hash for RefCell<Node> {
    fn hash(&self, state: &mut SipState) {
        self.borrow().id.hash(state);
    }
}
impl Eq for RefCell<Node> {}

impl PartialEq for RefCell<Node> {
    fn eq(&self, other: &RefCell<Node>) -> bool {
        self.borrow().id == other.borrow().id
    }
}

impl PartialOrd for RefCell<Node> {
    fn partial_cmp(&self, other: &RefCell<Node>) -> Option<Ordering> {
        self.borrow().id.partial_cmp(&other.borrow().id)
    }
}

#[deriving(Show, Clone)]
struct Edge {
    source: Rc<RefCell<Node>>,
    target: Rc<RefCell<Node>>,
    cost: u64
}

fn shortest_path<H: Heap<u64, Rc<RefCell<Node>>>>(pq: &mut H,
                 graph: Vec<Rc<RefCell<Node>>>,
                 start: Rc<RefCell<Node>>,
                 stop: Rc<RefCell<Node>>) -> Vec<Rc<RefCell<Node>>> {
    start.borrow_mut().distance = 0;
    for n in graph.into_iter() {
        pq.insert(n.borrow().distance, n);
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
                    pq.decrease_key(e.target.clone(), old_dist - new_dist);
                }
            }
        }
    }

    let mut path = Vec::new();
    construct_path(&mut path, stop);
    path
}

fn construct_path(path: &mut Vec<Rc<RefCell<Node>>>, node: Rc<RefCell<Node>>) {
    if node.borrow().distance == 0 {
        path.push(node.clone());
        return
    }
    construct_path(path, node.borrow().previous.clone().unwrap());
    path.push(node.clone());
}

fn main() {
    let n1 = Rc::new(RefCell::new(Node {
        id: 1,
        edges: Vec::new(),
        previous: None,
        distance: INFINITY,
        visited: false
    }));
    let n2 = Rc::new(RefCell::new(Node {
        id: 2,
        edges: Vec::new(),
        previous: None,
        distance: INFINITY,
        visited: false
    }));
    let n3 = Rc::new(RefCell::new(Node {
        id: 3,
        edges: Vec::new(),
        previous: None,
        distance: INFINITY,
        visited: false
    }));
    let n4 = Rc::new(RefCell::new(Node {
        id: 4,
        edges: Vec::new(),
        previous: None,
        distance: INFINITY,
        visited: false
    }));
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
        cost: 3
    });
    n2.borrow_mut().edges.push(Edge {
        source: n2.clone(),
        target: n4.clone(),
        cost: 8
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
    println!("Path: {}", shortest);
}
