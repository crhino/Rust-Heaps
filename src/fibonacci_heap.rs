use std::num::Float;
use std::fmt::Show;
use std::collections::DList;
use std::hash::Hash;
use std::collections::HashMap;
use fib_node::{FibEntryType, FibEntry};
use {Heap, HeapExt, HeapDelete};

#[deriving(Clone)]
pub struct FibHeap<K,V> {
    // A hash table for O(1) access to entries. The value is the key.
    hash_table: HashMap<V, FibEntryType<K,V>>,
    // The minimum element is always contained at the top of the first root.
    roots: DList<FibEntryType<K,V>>,
    total: int
}

impl<K: Ord + Show + Clone + Sub<K,K>,
V: Eq + PartialOrd + Show + Hash + Clone> Heap<K, V>
for FibHeap<K, V> {
    fn find_min(&self) -> (K, V) {
        match self.roots.front() {
            Some(n) => (n.borrow().get_key().clone(), n.borrow().value().clone()),
            None => panic!("Fibonacci heap is empty")
        }
    }

    fn insert(&mut self, k: K, v: V) {
        let vhash = v.clone();
        let node = FibEntry::new(k, v);
        let hashnode = node.clone();
        self.insert_root(node);
        self.hash_table.insert(vhash, hashnode);
        self.total += 1;
    }

    fn delete_min(&mut self) -> (K, V) {
        match self.roots.pop_front() {
            None => panic!("Fibonacci heap is empty"),
            Some(mut min_node) => {
                for c in min_node.children_into_iter() {
                    c.set_parent(None);
                    self.insert_root(c);
                }
                // Linking Step
                self.consolidate();

                self.total = self.total - 1;
                self.hash_table.remove(min_node.borrow().value());
                min_node.into_inner()
            }
        }
    }

    fn decrease_key(&mut self, value: V, delta: K) {
        let node = self.hash_table[value].clone();
        let key = node.borrow().get_key().clone();
        node.set_key(key - delta);
        self.decreased_node(node);
    }

    fn empty(&self) -> bool {
        self.total == 0
    }
}

impl<K: Ord + Show + Clone + Sub<K,K>,
V: Eq + PartialOrd + Show + Hash + Clone> HeapExt<K, V>
for FibHeap<K, V> {
    fn merge(&mut self, other: FibHeap<K,V>) {
        self.roots.merge(other.roots, |s, o| { s < o });

        for (k, v) in other.hash_table.into_iter() {
            self.hash_table.insert(k, v);
        }

        self.total += other.total;
    }
}

impl<K: Ord + Show +Clone + Sub<K,K>,
V: Eq + PartialOrd + Show + Hash + Clone> HeapDelete<K, V>
for FibHeap<K, V> {
    // This will essentially zero out the given value's key.
    // It is undefined behaviour if there is another zero value in the Heap.
    fn delete(&mut self, value: V) -> (K, V) {
        {
            let node = self.hash_table[value].clone();
            let key = node.borrow().get_key().clone();
            self.decrease_key(value, key);
        }
        self.delete_min()
    }
}

impl<K: Ord + Show + Clone + Sub<K,K>, V: Eq + PartialOrd + Show + Hash + Clone> FibHeap<K, V> {
    pub fn new() -> FibHeap<K,V> {
        FibHeap { hash_table: HashMap::new(), roots: DList::new(), total: 0 }
    }

    fn decreased_node(&mut self, node: FibEntryType<K, V>) {
        match node.get_parent() {
            Some(parent) => {
                if node < parent {
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

    fn insert_root(&mut self, root: FibEntryType<K, V>) {
        if self.roots.len() == 0 || *self.roots.front().unwrap() < root {
            self.roots.push_back(root);
        } else {
            self.roots.push_front(root);
        }
    }

    // XXX: Should I be using a DList for this data structure?
    fn sort_roots(&mut self) {
        if self.roots.len() == 0 {
            return
        }

        let mut min_node = self.roots.pop_front().unwrap();
        for _ in range(0, self.roots.len()) {
            if *self.roots.front().unwrap() < min_node {
                self.roots.push_back(min_node);
                min_node = self.roots.pop_front().unwrap();
                // Put the recently added node at front so that it will properly rotate backward.
                self.roots.rotate_forward();
            }
            self.roots.rotate_backward()
       }
       self.roots.push_front(min_node);
    }

    fn cut(&self, parent: FibEntryType<K, V>, child: FibEntryType<K, V>) -> FibEntryType<K, V> {
        let res = parent.remove_child(child.clone());
        assert!(res.is_ok());
        child.set_parent(None);
        child.set_marked(false);
        child
    }

    fn cascading_cut(&mut self, node: FibEntryType<K, V>) {
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
        let log_n = (self.total as f64).log2() as uint + 1;
        let mut rank_vec = Vec::from_fn(log_n, |_| -> Option<FibEntryType<K,V>> { None });
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

    fn link_and_insert(&self, rank_vec: &mut Vec<Option<FibEntryType<K,V>>>,
                       root: FibEntryType<K,V>, child: FibEntryType<K,V>) {
        // We are only linking FibHeap roots, so they don't have parents.
        child.set_parent(Some(root.clone()));
        child.set_marked(false);

        root.add_child(child);
        self.insert_by_rank(rank_vec, root);
    }

    fn insert_by_rank(&self, rank_vec: &mut Vec<Option<FibEntryType<K,V>>>,
                      node: FibEntryType<K,V>) {
        let rank = node.rank();
        if rank_vec[rank].is_none() {
            rank_vec[rank] = Some(node);
            return
        }

        rank_vec.push(None);
        let other = rank_vec.swap_remove(rank).unwrap().unwrap();

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
    use fib_node::{FibEntry};

    #[test]
    fn fheap_insert() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(2, 2);
        let one = fheap.hash_table.get(&1).clone().expect("Value 1 not found");
        let two = fheap.hash_table.get(&2).clone().expect("Value 2 not found");
        assert_eq!(one.borrow().get_key(), &1);
        assert_eq!(two.borrow().get_key(), &2);
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

        fheap.merge(fheap1);
        assert_eq!(fheap.total, 6);
        assert_eq!(fheap.roots.len(), 6);
        assert_eq!(fheap.hash_table.len(), 6);
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
        fheap.insert(4, 4);
        let four = fheap.hash_table[4].clone();
        fheap.insert(0, 0);
        fheap.insert(5, 5);
        fheap.delete_min();
        assert_eq!(fheap.roots.len(), 2);
        fheap.decrease_key(4, 3);
        assert_eq!(four.borrow().get_key(), &1);
        assert!(four.get_parent().is_none());
        assert_eq!(fheap.roots.len(), 3);
        fheap.decrease_key(5, 5);
        assert_eq!(fheap.roots.len(), 3);
        assert_eq!(fheap.find_min(), (0, 5));
    }

    #[test]
    fn test_fheap_decrease_key_adding_to_empty_root() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(4, 4);
        let four = fheap.hash_table[4].clone();
        fheap.insert(0, 0);
        fheap.delete_min();
        assert_eq!(fheap.roots.len(), 1);
        fheap.decrease_key(4, 2);
        assert_eq!(four.borrow().get_key(), &2);
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
        fheap.insert(6, 6);
        fheap.insert(7, 7);
        fheap.insert(18, 18);
        fheap.insert(9, 9);
        fheap.insert(11, 11);
        fheap.insert(15, 15);
        fheap.delete_min();
        assert_eq!(fheap.find_min(), (1, 1));
        assert_eq!(fheap.roots.len(), 3);
        fheap.decrease_key(6, 4);
        assert_eq!(fheap.roots.len(), 4);
        fheap.decrease_key(7, 7);
        assert_eq!(fheap.roots.len(), 6);
    }

    #[test]
    fn test_fheap_delete() {
        let mut fheap: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(1, 1);
        fheap.insert(4, 4);
        fheap.insert(0, 0);
        fheap.insert(5, 5);
        fheap.delete_min();
        fheap.delete(5);
        assert_eq!(fheap.roots.len(), 1);
        fheap.delete(1);
        assert_eq!(fheap.roots.len(), 1);
        assert_eq!(fheap.find_min(), (4, 4))
    }

    #[bench]
    fn bench_new(b: &mut Bencher) {
        b.iter(|| {
            let fheap: FibHeap<u8, u8> = FibHeap::new();
            assert_eq!(fheap.roots.len(), 0);
            assert_eq!(fheap.hash_table.len(), 0);
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
        let fheap1: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(7, 7);
        fheap.insert(10, 10);

        b.iter(|| {
            fheap.merge(fheap1.clone());
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
        fheap.insert(10, 10);
        fheap.insert(11, 11);
        fheap.insert(13, 13);
        fheap.insert(14, 14);
        fheap.insert(15, 15);
        fheap.insert(16, 16);
        fheap.insert(17, 17);
        let fheap1: FibHeap<u8, u8> = FibHeap::new();
        fheap.insert(10, 10);

        b.iter(|| {
            fheap.decrease_key(10, 9);
            fheap.merge(fheap1.clone());
        });
    }
}
