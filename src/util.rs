pub struct CyclicIterator<'a, T> {
    items: &'a [T],
    index: usize,
}

impl<'a, T> CyclicIterator<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self { items, index: 0 }
    }
}

impl<'a, T> Iterator for CyclicIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            None
        } else {
            let item = &self.items[self.index];
            self.index = (self.index + 1) % self.items.len();
            Some(item)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Class, Config};
    use crate::simulation::{Rotation, Rotations};

    #[test]
    fn test_cyclic_on_vector() {
        let test_data = vec![1, 2, 3, 4, 5];
        let mut cyclic_iter = CyclicIterator::new(&test_data);
        assert_eq!(cyclic_iter.next(), Some(&1));
        assert_eq!(cyclic_iter.next(), Some(&2));
        assert_eq!(cyclic_iter.next(), Some(&3));
        assert_eq!(cyclic_iter.next(), Some(&4));
        assert_eq!(cyclic_iter.next(), Some(&5));
        assert_eq!(cyclic_iter.next(), Some(&1));
        assert_eq!(cyclic_iter.next(), Some(&2));
        assert_eq!(cyclic_iter.next(), Some(&3));
        assert_eq!(cyclic_iter.next(), Some(&4));
        assert_eq!(cyclic_iter.next(), Some(&5));
    }

    #[test]
    fn test_cyclic_on_rotation() {
        let test_rotation = Rotation::get_rotation(Class::Warlock, &Config::default());
        let mut cyclic_iter = CyclicIterator::new(&test_rotation.skills);
        assert_eq!(cyclic_iter.next().unwrap().name, "Lich Form");
        assert_eq!(cyclic_iter.next().unwrap().name, "Profane Spirit");
        assert_eq!(cyclic_iter.next().unwrap().name, "Engulfing Darkness");
        assert_eq!(cyclic_iter.next().unwrap().name, "Explosive Plaque");
        assert_eq!(cyclic_iter.next().unwrap().name, "Venom Bolt");
        assert_eq!(cyclic_iter.next().unwrap().name, "Lich Form");
        assert_eq!(cyclic_iter.next().unwrap().name, "Profane Spirit");
    }
}
