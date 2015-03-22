#![crate_name = "rust_heaps"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#[cfg(test)]
extern crate test;

mod fib_node;
pub mod fibonacci_heap;

pub trait Heap<K, V> {
    type HeapEntry;

    fn find_min(&self) -> (K, V);
    fn delete_min(&mut self) -> (K, V);
    fn insert(&mut self, key: K, value: V) -> Self::HeapEntry;
    fn decrease_key(&mut self, entry: &Self::HeapEntry, delta: K);
    fn empty(&self) -> bool;
}

pub trait HeapExt {
    fn merge(mut self, mut other: Self) -> Self;
}

pub trait HeapDelete<K, V> {
    type HeapEntry;

    fn delete(&mut self, entry: Self::HeapEntry) -> (K, V);
}
