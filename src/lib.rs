#![crate_name = "rust_heaps"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
mod fib_node;
pub mod fibonacci_heap;

pub trait Heap<K, V> {
    fn find_min(&self) -> (K, V);
    fn delete_min(&mut self) -> (K, V);
    fn insert(&mut self, key: K, value: V);
    fn decrease_key(&mut self, value: V, delta: K);
    fn empty(&self) -> bool;
}

pub trait HeapExt<K, V> {
    fn merge(&mut self, other: Self);
}

pub trait HeapDelete<K, V> {
    fn delete(&mut self, value: V) -> (K, V);
}
