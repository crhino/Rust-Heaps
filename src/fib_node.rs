use std::fmt::{Debug};
use std::cmp::Ordering;
use std::rc::{Rc, Weak};
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::collections::vec_deque::Drain;

pub struct FibNode<K, V> {
    inner: UnsafeCell<Inner<K, V>>,
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Ord for FibNode<K, V> {
    fn cmp(&self, other: &FibNode<K, V>) -> Ordering {
        unsafe { (*(self.inner.get())).cmp(&*other.inner.get()) }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialOrd for FibNode<K, V> {
    fn partial_cmp(&self, other: &FibNode<K, V>) -> Option<Ordering> {
        unsafe { (*(self.inner.get())).partial_cmp(&*other.inner.get()) }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialEq for FibNode<K, V> {
    fn eq(&self, other: &FibNode<K, V>) -> bool {
        unsafe { (*(self.inner.get())).eq(&*other.inner.get()) }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Eq for FibNode<K, V> {}

#[derive(Clone)]
pub struct Inner<K,V> {
    parent: Option<Weak<FibNode<K, V>>>,
    children: VecDeque<Rc<FibNode<K, V>>>,
    // Rank is the length of children
    marked: bool,
    key: K,
    value: V,
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Ord for Inner<K, V> {
    fn cmp(&self, other: &Inner<K, V>) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialOrd for Inner<K, V> {
    fn partial_cmp(&self, other: &Inner<K, V>) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialEq for Inner<K, V> {
    fn eq(&self, other: &Inner<K, V>) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Eq for Inner<K, V> {}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> FibNode<K,V> {
    pub fn new(key: K, value: V) -> Rc<FibNode<K,V>> {
        let inner = UnsafeCell::new(Inner::new(key, value));
        Rc::new(FibNode { inner: inner })
    }

    pub fn rank(&self) -> usize {
        unsafe { (*self.inner.get()).rank() }
    }

    pub fn add_child(&self, child: Rc<FibNode<K,V>>) {
        unsafe { (*self.inner.get()).add_child(child) }
    }

    pub fn remove_child(&self, child: Rc<FibNode<K,V>>)
        -> Result<Rc<FibNode<K,V>>, String> {
        unsafe { (*self.inner.get()).remove_child(child) }
    }

    pub fn set_marked(&self, mark: bool) {
        unsafe { (*self.inner.get()).set_marked(mark) }
    }

    pub fn get_marked(&self) -> bool {
        unsafe { (*self.inner.get()).get_marked() }
    }

    pub fn set_key(&self, key: K) {
        unsafe { (*self.inner.get()).set_key(key) }
    }

    pub fn set_parent(&self, parent: Option<Weak<FibNode<K,V>>>) {
        unsafe { (*self.inner.get()).set_parent(parent) }
    }

    pub fn get_parent(&self) -> Option<Weak<FibNode<K,V>>>{
        unsafe { (*self.inner.get()).get_parent() }
    }

    pub fn drain_children(&self) -> Drain<Rc<FibNode<K,V>>> {
        unsafe { (*self.inner.get()).drain_children() }
    }

    // Do this better, don't clone the thing.
    pub fn into_inner(&self) -> (K, V) {
        unsafe {
            let n = (*self.inner.get()).clone();
            n.into_inner()
        }
    }

    pub fn get_value(&self) -> &V {
        unsafe { (*self.inner.get()).get_value() }
    }

    pub fn get_key(&self) -> &K {
        unsafe { (*self.inner.get()).get_key() }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Inner<K,V> {
    pub fn new(key: K, value: V) -> Inner<K,V> {
        Inner {
            parent: None,
            children: VecDeque::new(),
            marked: false,
            key: key,
            value: value,
        }
    }

    pub fn rank(&self) -> usize {
        self.children.len()
    }

    pub fn add_child(&mut self, child: Rc<FibNode<K,V>>) {
        self.children.push_back(child);
    }

    // XXX: Better way to do this?
    pub fn remove_child(&mut self, child: Rc<FibNode<K,V>>)
        -> Result<Rc<FibNode<K,V>>, String> {
            for _ in (0..self.children.len()) {
                if *self.children.front().unwrap() == child {
                    return Ok(self.children.pop_front().unwrap())
                }
                let front = self.children.pop_front().unwrap();
                self.children.push_back(front);
            }
            Err(String::from_str("Could not find child {:?} in children"))
        }

    pub fn set_marked(&mut self, mark: bool) {
        self.marked = mark;
    }

    pub fn get_marked(&self) -> bool {
        self.marked
    }

    pub fn set_key(&mut self, key: K) {
        self.key = key;
    }

    pub fn set_parent(&mut self, parent: Option<Weak<FibNode<K,V>>>) {
        self.parent = parent;
    }

    pub fn get_parent(&self) -> Option<Weak<FibNode<K,V>>>{
        self.parent.clone()
    }

    pub fn drain_children(&mut self) -> Drain<Rc<FibNode<K,V>>> {
        self.children.drain()
    }

    pub fn into_inner(self) -> (K, V) {
        assert!(self.parent.is_none());
        assert_eq!(self.children.len(), 0);
        (self.key, self.value)
    }

    pub fn get_value(&self) -> &V {
        &self.value
    }

    pub fn get_key(&self) -> &K {
        &self.key
    }
}

#[cfg(test)]
mod test {
    use fib_node::{FibNode};

    #[test]
    fn node_test() {
        let node = FibNode::new(0u8, 0u8);
        let child = FibNode::new(1u8, 1u8);

        assert_eq!(node.get_key(), &0u8);
        assert_eq!(node.get_value(), &0u8);
        assert_eq!(node.get_value(), &0u8);
        assert_eq!(node.get_marked(), false);
        node.set_marked(true);
        assert_eq!(node.get_marked(), true);
        assert_eq!(node.rank(), 0);
        node.add_child(child);
        assert_eq!(node.rank(), 1);
    }

    #[test]
    fn parent_child_test() {
        let node = FibNode::new(1u8, 1u8);
        let root = node.clone();
        let child = FibNode::new(2u8, 2u8);
        child.set_parent(Some(root.clone().downgrade()));

        node.set_key(10u8);
        node.set_marked(true);
        let parent = child.get_parent().expect("Not a child");
        let parent = parent.upgrade().expect("Destroyed");

        assert!(root == parent);
        assert_eq!(root.get_key(), &10u8);
        assert_eq!(parent.get_marked(), true);
        assert_eq!(child.get_key(), &2u8);
    }

    #[test]
    fn remove_child_test() {
        let node = FibNode::new(0u8, 0u8);
        let child1 = FibNode::new(1u8, 1u8);
        let child2 = FibNode::new(2u8, 2u8);
        let child3 = FibNode::new(3u8, 3u8);
        let child4 = FibNode::new(4u8, 4u8);
        let child5 = FibNode::new(5u8, 5u8);

        node.add_child(child1.clone());
        node.add_child(child2.clone());
        node.add_child(child3.clone());
        node.add_child(child4.clone());

        assert_eq!(node.rank(), 4);
        let res = node.remove_child(child4);
        assert!(res.is_ok());
        let res = node.remove_child(child1);
        assert!(res.is_ok());
        let res = node.remove_child(child2);
        assert!(res.is_ok());
        let res = node.remove_child(child3);
        assert!(res.is_ok());
        let res = node.remove_child(child5);
        assert!(res.is_err());
    }
}
