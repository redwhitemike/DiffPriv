/// based on https://stackoverflow.com/questions/53755017/can-i-randomly-sample-from-a-hashset-efficiently
/// used to efficiently retrieve observed values in the Laplace noiser
use std::collections::HashSet;

#[derive(Default, Clone)]
pub struct VecSet<T> {
    set: HashSet<T>,
    vec: Vec<T>,
}

impl<T> VecSet<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn insert(&mut self, elem: T) {
        assert_eq!(self.set.len(), self.vec.len());
        let was_new = self.set.insert(elem.clone());
        if was_new {
            self.vec.push(elem);
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }
}
