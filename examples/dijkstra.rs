/*
 *
 * Christopher Piraino
 *
 * Dijkstra's Algorithm in Rust
 *
 * Using Fibonacci heaps.
 */

extern crate core;
extern crate fibonacci_heap;
extern crate collections;
use fibonacci_heap::FibHeap;
use core::fmt;
use std::uint;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use collections::hash::Hash;
use collections::hash::sip::SipState;

static INFINITY: uint = uint::MAX;

struct Node {
    id: uint,
    edges: RefCell<Vec<Edge>>, // A node only holds edges were it is the source.
    previous: RefCell<Option<Rc<Node>>>,
    distance: Cell<uint>,
    visited: Cell<bool>
}

impl fmt::Show for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "\nNode "))
        try!(write!(f, "id: {} ", self.id));
        write!(f, "distance: {}", self.distance.get())
    }
}

impl Hash for Node {
    fn hash(&self, state: &mut SipState) {
        self.id.hash(state);
    }
}
impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

#[deriving(Show)]
struct Edge {
    source: Rc<Node>,
    target: Rc<Node>,
    cost: uint
}

fn shortest_path(graph: Vec<Rc<Node>>, start: &Rc<Node>, stop: &Rc<Node>) -> Vec<Rc<Node>> {
    let mut pq = FibHeap::new();
    start.distance.set(0);
    for n in graph.into_iter() {
        pq.insert(n.distance.get(), n);
    }

    while !pq.empty() {
        let (distance, node) = pq.delete_min();
        if node.id == stop.id {
            break;
        }
        node.visited.set(true);
        for e in node.edges.borrow().iter() {
            if !e.target.visited.get() {
               let new_dist = distance + e.cost;
                if new_dist < e.target.distance.get() {
                    let old_dist = e.target.distance.get();
                    {
                        e.target.distance.set(new_dist);
                        let mut prev = e.target.previous.borrow_mut();
                        *prev.deref_mut() = Some(node.clone());
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

fn construct_path(path: &mut Vec<Rc<Node>>, node: &Rc<Node>) {
    if node.distance.get() == 0 {
        path.push(node.clone());
        return
    }
    construct_path(path, &node.previous.borrow().clone().unwrap());
    path.push(node.clone());
}

fn main() {
    let n1 = Rc::new(Node {
        id: 1,
        edges: RefCell::new(Vec::new()),
        previous: RefCell::new(None),
        distance: Cell::new(INFINITY),
        visited: Cell::new(false)
    });
    let n2 = Rc::new(Node {
        id: 2,
        edges: RefCell::new(Vec::new()),
        previous: RefCell::new(None),
        distance: Cell::new(INFINITY),
        visited: Cell::new(false)
    });
    let n3 = Rc::new(Node {
        id: 3,
        edges: RefCell::new(Vec::new()),
        previous: RefCell::new(None),
        distance: Cell::new(INFINITY),
        visited: Cell::new(false)
    });
    let n4 = Rc::new(Node {
        id: 4,
        edges: RefCell::new(Vec::new()),
        previous: RefCell::new(None),
        distance: Cell::new(INFINITY),
        visited: Cell::new(false)
    });
    n1.edges.borrow_mut().push(Edge {
        source: n1.clone(),
        target: n2.clone(),
        cost: 1
    });
    n1.edges.borrow_mut().push(Edge {
        source: n1.clone(),
        target: n3.clone(),
        cost: 5
    });
    n2.edges.borrow_mut().push(Edge {
        source: n2.clone(),
        target: n3.clone(),
        cost: 3
    });
    n2.edges.borrow_mut().push(Edge {
        source: n2.clone(),
        target: n4.clone(),
        cost: 8
    });
    n3.edges.borrow_mut().push(Edge {
        source: n3.clone(),
        target: n4.clone(),
        cost: 2
    });
    n4.edges.borrow_mut().push(Edge {
        source: n4.clone(),
        target: n2.clone(),
        cost: 9
    });
    let n1c = n1.clone();
    let n4c = n4.clone();
    let graph = vec!(n1, n2, n3, n4);
    let shortest = shortest_path(graph, &n1c, &n4c);
    println!("Path: {}", shortest);
}
