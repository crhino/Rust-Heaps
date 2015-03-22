use std::fmt::{Debug};
use std::cmp::Ordering;
use std::mem;
use std::collections::VecDeque;
use std::collections::vec_deque::Drain;

#[derive(Clone, Debug)]
pub struct FibNode<K, V> {
    inner: *mut _FibNode<K, V>,
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Ord for FibNode<K, V> {
    fn cmp(&self, other: &FibNode<K, V>) -> Ordering {
        unsafe { (*(self.inner)).cmp(&*other.inner) }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialOrd for FibNode<K, V> {
    fn partial_cmp(&self, other: &FibNode<K, V>) -> Option<Ordering> {
        unsafe { (*(self.inner)).partial_cmp(&*other.inner) }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialEq for FibNode<K, V> {
    fn eq(&self, other: &FibNode<K, V>) -> bool {
        unsafe { (*(self.inner)).eq(&*other.inner) }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Eq for FibNode<K, V> {}

#[derive(Clone, Debug)]
pub struct _FibNode<K,V> {
    parent: Option<FibNode<K, V>>,
    children: VecDeque<FibNode<K, V>>,
    // Rank is the length of children
    marked: bool,
    key: K,
    value: V,
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Ord for _FibNode<K, V> {
    fn cmp(&self, other: &_FibNode<K, V>) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialOrd for _FibNode<K, V> {
    fn partial_cmp(&self, other: &_FibNode<K, V>) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> PartialEq for _FibNode<K, V> {
    fn eq(&self, other: &_FibNode<K, V>) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> Eq for _FibNode<K, V> {}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> FibNode<K,V> {
    pub fn new(key: K, value: V) -> FibNode<K,V> {
        let node = _FibNode {
            parent: None,
            children: VecDeque::new(),
            marked: false,
            key: key,
            value: value,
        };
        let inner = unsafe { mem::transmute(Box::new(node)) };
        FibNode { inner: inner }
    }

    pub fn from_mut_ptr(ptr: *mut _FibNode<K, V>) -> FibNode<K, V> {
        FibNode { inner: ptr }
    }

    pub fn get_mut_ptr(&self) -> *mut _FibNode<K, V> {
        self.inner.clone()
    }

    pub fn rank(&self) -> usize {
        unsafe { (*self.inner).rank() }
    }

    pub fn add_child(&self, child: FibNode<K,V>) {
        unsafe { (*self.inner).add_child(child) }
    }

    pub fn remove_child(&self, child: FibNode<K,V>)
        -> Result<FibNode<K,V>, String> {
        unsafe { (*self.inner).remove_child(child) }
        }

    pub fn set_marked(&mut self, mark: bool) {
        unsafe { (*self.inner).set_marked(mark) }
    }

    pub fn get_marked(&self) -> bool {
        unsafe { (*self.inner).get_marked() }
    }

    pub fn set_key(&mut self, key: K) {
        unsafe { (*self.inner).set_key(key) }
    }

    pub fn set_parent(&mut self, parent: Option<FibNode<K,V>>) {
        unsafe { (*self.inner).set_parent(parent) }
    }

    pub fn get_parent(&self) -> Option<FibNode<K,V>>{
        unsafe { (*self.inner).get_parent() }
    }

    pub fn root(&self) -> bool {
        unsafe { (*self.inner).root() }
    }

    pub fn children_drain(&mut self) -> Drain<FibNode<K,V>> {
        unsafe { (*self.inner).children_drain() }
    }

    // TODO: Fix this so it actually consumes everything
    pub fn into_inner(self) -> (K, V) {
        unsafe {
            let node = (*self.inner).clone();
            node.into_inner()
        }
    }

    pub fn get_value(&self) -> &V {
        unsafe { (*self.inner).get_value() }
    }

    pub fn get_key(&self) -> &K {
        unsafe { (*self.inner).get_key() }
    }
}

impl<K: Clone + Ord + Debug, V: Eq + Clone + PartialOrd + Debug> _FibNode<K,V> {
    pub fn new(key: K, value: V) -> _FibNode<K,V> {
        _FibNode {
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

    pub fn add_child(&mut self, child: FibNode<K,V>) {
        self.children.push_back(child);
    }

    // XXX: Better way to do this?
    pub fn remove_child(&mut self, child: FibNode<K,V>)
        -> Result<FibNode<K,V>, String> {
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

    pub fn set_parent(&mut self, parent: Option<FibNode<K,V>>) {
        self.parent = parent;
    }

    pub fn get_parent(&self) -> Option<FibNode<K,V>>{
        self.parent.clone()
    }

    pub fn root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn children_drain(&mut self) -> Drain<FibNode<K,V>> {
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
        let mut node = FibNode::new(0u8, 0u8);
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
        let mut node = FibNode::new(1u8, 1u8);
        let root = node.clone();
        let mut child = FibNode::new(2u8, 2u8);
        child.set_parent(Some(root.clone()));

        node.set_key(10u8);
        node.set_marked(true);
        let parent = child.get_parent().expect("Not a child");

        assert_eq!(root, parent);
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
