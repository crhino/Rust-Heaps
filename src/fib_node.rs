use std::fmt;
use std::fmt::{Show};
use std::rc::{try_unwrap, Rc};
use std::cell::RefCell;
use std::collections::DList;
use std::collections::dlist::IntoIter;

pub type FibEntryType<K,V> = Rc<RefCell<FibNode<K,V>>>;

pub trait FibEntry<K, V> {
    fn new(key: K, value: V) -> FibEntryType<K,V>;
    fn rank(&self) -> uint;
    fn add_child(&self, child: FibEntryType<K, V>);
    fn remove_child(&self, child: FibEntryType<K, V>)
    -> Result<FibEntryType<K,V>, String>;
    fn set_marked(&self, mark: bool);
    fn get_marked(&self) -> bool;
    fn set_key(&self, key: K);
    fn set_parent(&self, parent: Option<FibEntryType<K,V>>);
    fn get_parent(&self) -> Option<FibEntryType<K,V>>;
    fn root(&self) -> bool;
    fn children_into_iter(&mut self) -> IntoIter<FibEntryType<K, V>>;
    fn into_inner(self) -> (K, V);
}

#[deriving(Clone)]
struct FibNode<K,V> {
    parent: Option<FibEntryType<K,V>>,
    children: DList<FibEntryType<K, V>>,
    // Rank is the length of children
    marked: bool,
    key: K,
    value: V
}

impl<K: Show, V: Show> Show for FibNode<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "FibNode ( "));
        if self.parent.is_none() {
            try!(write!(f, "parent: ( None ), "));
        } else {
            try!(write!(f, "parent ( key: {}, value: {} ), ",
                        self.parent.clone().unwrap().borrow().get_key(),
                        self.parent.clone().unwrap().borrow().value()));
        }
        try!(write!(f, "children: {}, ", self.children));
        try!(write!(f, "marked: {}, ", self.marked));
        try!(write!(f, "key: {}, ", self.key));
        write!(f, "value: {} )", self.value)
    }
}

impl<K: Show, V: Show> Show for FibEntryType<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl<K: Ord, V: PartialOrd> PartialOrd for FibEntryType<K, V> {
    fn partial_cmp(&self, other: &FibEntryType<K, V>) -> Option<Ordering> {
        self.borrow().get_key().partial_cmp(other.borrow().get_key())
    }
}

impl<K: Ord, V: PartialOrd> PartialEq for FibEntryType<K, V> {
    fn eq(&self, other: &FibEntryType<K, V>) -> bool {
        self.borrow().value().eq(other.borrow().value())
    }
}

impl<K: Ord, V: PartialOrd> Eq for FibEntryType<K, V> {}

impl<K: Ord, V: PartialOrd> Ord for FibEntryType<K, V> {
    fn cmp(&self, other: &FibEntryType<K, V>) -> Ordering {
        self.borrow().get_key().cmp(other.borrow().get_key())
    }
}

impl<K: Ord + Show, V: PartialOrd + Show> FibEntry<K,V> for FibEntryType<K,V> {
    fn new(key: K, value: V) -> FibEntryType<K,V> {
        Rc::new(RefCell::new(FibNode {
            parent: None,
            children: DList::new(),
            marked: false,
            key: key,
            value: value,
        }))
    }

    fn rank(&self) -> uint {
        self.borrow().children.len()
    }

    fn add_child(&self, child: FibEntryType<K, V>) {
        self.borrow_mut().children.insert_ordered(child);
    }

    // XXX: Better way to do this?
    fn remove_child(&self, child: FibEntryType<K, V>)
        -> Result<FibEntryType<K,V>, String> {
            let mut borrow = self.borrow_mut();
            let children = &mut borrow.children;

            for _ in range(0, children.len()) {
                if *children.front().unwrap() == child {
                    return Ok(children.pop_front().unwrap())
                }
                children.rotate_backward();
            }
            Err(format!("Child {} was not found in children", child))
        }

    fn set_marked(&self, mark: bool) {
        self.borrow_mut().marked = mark;
    }

    fn get_marked(&self) -> bool {
        self.borrow().marked
    }

    fn set_key(&self, key: K) {
        self.borrow_mut().key = key;
    }

    fn set_parent(&self, parent: Option<FibEntryType<K,V>>) {
        self.borrow_mut().parent = parent;
    }

    fn get_parent(&self) -> Option<FibEntryType<K,V>>{
        self.borrow().parent.clone()
    }

    fn root(&self) -> bool {
        self.borrow().parent.is_none()
    }

    fn children_into_iter(&mut self) -> IntoIter<FibEntryType<K, V>> {
        let mut borrow = self.borrow_mut();
        let children = borrow.children.clone();
        borrow.children = DList::new();
        children.into_iter()
    }

    fn into_inner(self) -> (K, V) {
        match try_unwrap(self) {
            Ok(node) => {
                let inner = node.into_inner();
                (inner.key, inner.value)
            },
            Err(rc) => panic!("{} is still shared", rc)
        }
    }
}

impl<K, V> FibNode<K, V> {
    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn get_key(&self) -> &K {
        &self.key
    }
}

#[cfg(test)]
mod test {
    use fib_node::{FibEntry};

    #[test]
    fn node_test() {
        let node = FibEntry::new(0u8, 0u8);
        let child = FibEntry::new(1u8, 1u8);

        assert_eq!(node.borrow().get_key(), &0u8);
        assert_eq!(node.borrow().value(), &0u8);
        assert_eq!(node.get_marked(), false);
        node.set_marked(true);
        assert_eq!(node.get_marked(), true);
        assert_eq!(node.rank(), 0);
        node.add_child(child);
        assert_eq!(node.rank(), 1);
    }

    #[test]
    fn parent_child_test() {
        let node = FibEntry::new(1u8, 1u8);
        let root = node.clone();
        let child = FibEntry::new(2u8, 2u8);
        child.set_parent(Some(root.clone()));

        node.set_key(10u8);
        node.set_marked(true);
        let parent = child.get_parent().expect("Not a child");

        assert_eq!(root, parent);
        assert_eq!(root.borrow().get_key(), &10u8);
        assert_eq!(parent.get_marked(), true);
        assert_eq!(child.borrow().get_key(), &2u8);
    }

    #[test]
    fn remove_child_test() {
        let node = FibEntry::new(0u8, 0u8);
        let child1 = FibEntry::new(1u8, 1u8);
        let child2 = FibEntry::new(2u8, 2u8);
        let child3 = FibEntry::new(3u8, 3u8);
        let child4 = FibEntry::new(4u8, 4u8);
        let child5 = FibEntry::new(5u8, 5u8);

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
