use std::ops::Sub;
use std::fmt::Debug;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};
use std::hash::Hash;
use fib_node::{FibNode};
use {Heap, HeapExt, HeapDelete};

#[derive(Clone)]
pub struct FibHeap<K,V> {
    // The minimum element is always contained at the top of the first root.
    roots: LinkedList<Rc<FibNode<K, V>>>,
    total: u32
}

impl<K: Ord + Debug + Clone + Sub<K, Output=K>,
V: Eq + PartialOrd + Debug + Clone> Heap<K, V>
for FibHeap<K, V> {
    type HeapEntry = Rc<FibNode<K, V>>;

    fn find_min(&self) -> (K, V) {
        match self.roots.front() {
            Some(min) => {
                (min.get_key().clone(), min.get_value().clone())
            },
            None => panic!("Fibonacci heap is empty")
        }
    }

    fn insert(&mut self, k: K, v: V) -> Rc<FibNode<K, V>> {
        let node = FibNode::new(k, v);
        let ret = node.clone();
        self.total += 1;
        self.insert_root(node);
        ret
    }

    fn delete_min(&mut self) -> (K, V) {
        match self.roots.pop_front() {
            None => panic!("Fibonacci heap is empty"),
            Some(min_entry) => {
                for c in min_entry.drain_children() {
                    c.set_parent(None);
                    self.insert_root(c);
                }
                // Linking Step
                self.consolidate();

                self.total = self.total - 1;
                min_entry.into_inner()
            }
        }
    }

    fn decrease_key(&mut self, node: &Rc<FibNode<K, V>>, delta: K) {
        // TODO: Figure out how to do this better.
        let new_node = node.clone();
        let key = new_node.get_key().clone();
        let new_key: K = key - delta;
        new_node.set_key(new_key);
        self.decreased_node(new_node);
    }

    fn empty(&self) -> bool {
        self.total == 0
    }
}

impl<K: Ord + Debug + Clone + Sub<K, Output=K>,
V: Eq + PartialOrd + Debug + Hash + Clone> HeapExt for FibHeap<K, V> {
    fn merge(mut self, mut other: FibHeap<K,V>) -> FibHeap<K, V> {
        let (smin, _) = self.find_min();
        let (omin, _) = other.find_min();

        if smin < omin {
            self.roots.append(&mut other.roots);
            self.total += other.total;
            self
        } else {
            other.roots.append(&mut self.roots);
            other.total += self.total;
            other
        }
    }
}

impl<K: Ord + Debug +Clone + Sub<K, Output=K>,
V: Eq + PartialOrd + Debug + Hash + Clone> HeapDelete<K, V>
for FibHeap<K, V> {
    type HeapEntry = Rc<FibNode<K, V>>;

    // This will essentially zero out the given value's key.
    // It is undefined behaviour if there is another zero value in the Heap.
    // TODO: Fix this and do it better
    fn delete(&mut self, node: Rc<FibNode<K, V>>) -> (K, V) {
        {
            let key = node.get_key().clone();
            self.decrease_key(&node, key);
        }
        self.delete_min()
    }
}

impl<K: Ord + Debug + Clone + Sub<K, Output=K>, V: Eq + PartialOrd + Debug + Clone> FibHeap<K, V> {
    pub fn new() -> FibHeap<K,V> {
        FibHeap { roots: LinkedList::new(), total: 0 }
    }

    fn decreased_node(&mut self, node: Rc<FibNode<K, V>>) {
        match node.get_parent() {
            Some(parent) => {
                let p = parent.clone().upgrade().expect("Parent has already been destroyed");
                if node < p {
                    let root = self.cut(parent.clone(), node);
                    self.insert_root(root);
                    self.cascading_cut(parent);
                }
            }
            None => {
                self.sort_roots();
                return
            }
        }
    }

    fn insert_root(&mut self, root: Rc<FibNode<K, V>>) {
        if self.roots.len() == 0 || *self.roots.front().unwrap() < root {
            self.roots.push_back(root);
        } else {
            self.roots.push_front(root);
        }
    }

    // TODO: This is horrible and inefficient.
    fn sort_roots(&mut self) {
        let r = self.roots.split_off(0);
        for n in r.into_iter() {
            self.insert_root(n);
        }
    }

    fn cut(&self, p: Weak<FibNode<K, V>>, child: Rc<FibNode<K, V>>) -> Rc<FibNode<K, V>> {
        let parent = p.upgrade().expect("Parent was already destroyed");
        let res = parent.remove_child(child.clone());
        assert!(res.is_ok());
        child.set_parent(None);
        child.set_marked(false);
        child
    }

    fn cascading_cut(&mut self, n: Weak<FibNode<K, V>>) {
        let node = n.upgrade().expect("Node was already destroyed");
        match node.get_parent() {
            Some(parent) => {
                if node.get_marked() {
                    let root = self.cut(parent.clone(), node);
                    self.insert_root(root);
                    self.cascading_cut(parent);
                } else {
                    node.set_marked(true);
                }
            }
            None => {
                return
            }
        }
    }

    fn consolidate(&mut self) {
        // The maximum rank of a FibHeap is O(log n).
        let log_n = (self.total as f64).log2() as u64 + 1;
        let mut rank_vec = vec!(None);
        rank_vec.resize(log_n as usize, None);
        loop {
            match self.roots.pop_front() {
                Some(node) => {
                    self.insert_by_rank(&mut rank_vec, node);
                }
                None => break
            }
        }
        for n in rank_vec.into_iter() {
            if n.is_some() {
                self.insert_root(n.unwrap());
            }
        }
    }

    fn link_and_insert(&self, rank_vec: &mut Vec<Option<Rc<FibNode<K, V>>>>,
                       root: Rc<FibNode<K, V>>, child: Rc<FibNode<K, V>>) {
        // We are only linking FibHeap roots, so they don't have parents.
        child.set_parent(Some(root.clone().downgrade()));
        child.set_marked(false);

        root.add_child(child);
        self.insert_by_rank(rank_vec, root);
    }

    fn insert_by_rank(&self, rank_vec: &mut Vec<Option<Rc<FibNode<K, V>>>>,
                      node: Rc<FibNode<K, V>>) {
        let rank = node.rank();
        if rank_vec[rank].is_none() {
            rank_vec[rank] = Some(node);
            return
        }

        rank_vec.push(None);
        let other = rank_vec.swap_remove(rank).unwrap();

        if node < other {
            self.link_and_insert(rank_vec, node, other);
        } else {
            self.link_and_insert(rank_vec, other, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use {Heap, HeapExt, HeapDelete};
    use fibonacci_heap::{FibHeap};

    #[test]
    fn fheap_insert() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        let one = fheap.insert(1, 1);
        let two = fheap.insert(2, 2);
        assert_eq!(one.get_key(), &1);
        assert_eq!(two.get_key(), &2);
        assert_eq!(fheap.total, 2);
        assert_eq!(fheap.roots.len(), 2);
    }

    #[test]
    fn fheap_find_min() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(2, 2);
        assert_eq!(fheap.find_min(), (1, 1));
    }

    #[test]
    fn fheap_merge() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(2, 2);
        let mut fheap1: FibHeap<u8, u8> = FibHeap::new();
        fheap1.insert(5, 5);
        fheap1.insert(0, 0);
        fheap1.insert(3, 3);

        fheap = fheap.merge(fheap1);
        assert_eq!(fheap.total, 6);
        assert_eq!(fheap.roots.len(), 6);
    }

    #[test]
    fn fheap_delete_min() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(2, 2);
        fheap.insert(3, 3);
        fheap.insert(4, 4);
        fheap.insert(5, 5);
        assert_eq!(fheap.find_min(), (1, 1));
        fheap.insert(0, 0);
        assert_eq!(fheap.find_min(), (0, 0));
        assert_eq!(fheap.delete_min(), (0, 0));
        assert_eq!(fheap.delete_min(), (1, 1));
        assert_eq!(fheap.delete_min(), (2, 2));
        fheap.delete_min();
        fheap.delete_min();
        fheap.delete_min();
        assert!(fheap.empty());
    }

    #[test]
    fn test_fheap_decrease_key() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(2, 2);
        let four = fheap.insert(4, 4);
        fheap.insert(0, 0);
        let five = fheap.insert(5, 5);
        fheap.delete_min();
        assert_eq!(fheap.roots.len(), 2);
        fheap.decrease_key(&four.clone(), 3);
        assert_eq!(four.clone().get_key(), &1);
        assert!(four.get_parent().is_none());
        assert_eq!(fheap.roots.len(), 3);
        fheap.decrease_key(&five, 5);
        assert_eq!(fheap.roots.len(), 3);
        assert_eq!(fheap.find_min(), (0, 5));
    }

    #[test]
    fn test_fheap_decrease_key_adding_to_empty_root() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        let four = fheap.insert(4, 4);
        fheap.insert(0, 0);
        fheap.delete_min();
        assert_eq!(fheap.roots.len(), 1);
        fheap.decrease_key(&four, 2);
        assert_eq!(four.get_key(), &2);
        assert!(four.get_parent().is_none());
    }

    #[test]
    fn test_fheap_cascading_cut() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(0, 0);
        fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(5, 5);
        fheap.insert(2, 2);
        fheap.insert(3, 3);
        let six = fheap.insert(6, 6);
        let seven = fheap.insert(7, 7);
        fheap.insert(18, 18);
        fheap.insert(9, 9);
        fheap.insert(11, 11);
        fheap.insert(15, 15);
        fheap.delete_min();
        assert_eq!(fheap.find_min(), (1, 1));
        assert_eq!(fheap.roots.len(), 3);
        fheap.decrease_key(&six, 4);
        assert_eq!(fheap.roots.len(), 4);
        fheap.decrease_key(&seven, 7);
        assert_eq!(fheap.roots.len(), 6);
    }

    #[test]
    fn test_fheap_delete() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        let one = fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(0, 0);
        let five = fheap.insert(5, 5);
        fheap.delete_min();
        fheap.delete(five);
        assert_eq!(fheap.roots.len(), 1);
        fheap.delete(one);
        assert_eq!(fheap.roots.len(), 1);
        assert_eq!(fheap.find_min(), (4, 4))
    }

    #[bench]
    fn bench_new(b: &mut Bencher) {
        b.iter(|| {
            let fheap: FibHeap<u8, u8> = FibHeap::new();
            assert_eq!(fheap.roots.len(), 0);
            assert!(fheap.empty());
        });
    }

    #[bench]
    fn bench_insert(b: &mut Bencher) {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        let mut n = 0;
        b.iter(|| {
            fheap.insert(n,n);
            n += 1;
        });
    }

    #[bench]
    fn bench_merge(b: &mut Bencher) {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(0, 0);
        fheap.insert(5, 5);
        fheap.insert(2, 2);
        fheap.insert(6, 6);
        fheap.insert(3, 3);
        fheap.insert(11, 11);
        let mut fheap1: FibHeap<u8, u8> = FibHeap::new();
        fheap1.insert(7, 7);
        fheap1.insert(10, 10);

        // TODO: How to do this better?
        b.iter(move || {
            fheap.clone().merge(fheap1.clone());
        });
    }

    #[bench]
    fn bench_delete_min(b: &mut Bencher) {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(0, 0);
        fheap.insert(5, 5);
        fheap.insert(2, 2);
        fheap.insert(6, 6);
        fheap.insert(3, 3);
        fheap.insert(12, 12);
        fheap.insert(11, 11);
        fheap.insert(13, 13);
        fheap.insert(14, 14);
        fheap.insert(15, 15);
        fheap.insert(16, 16);
        fheap.insert(17, 17);

        b.iter(|| {
            fheap.delete_min();
            fheap.insert(0, 0);
        });
    }

    #[bench]
    fn bench_decrease_key(b: &mut Bencher) {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(0, 0);
        fheap.insert(5, 5);
        fheap.insert(2, 2);
        fheap.insert(6, 6);
        fheap.insert(3, 3);
        let ten = fheap.insert(10, 10);
        fheap.insert(11, 11);
        fheap.insert(13, 13);
        fheap.insert(14, 14);
        fheap.insert(15, 15);
        fheap.insert(16, 16);
        fheap.insert(17, 17);
        fheap.insert(10, 10);

        b.iter(|| {
            fheap.decrease_key(&ten, 1);
        });
    }
}
