use std::collections::VecDeque;

struct WindowContent<T> {
    data: VecDeque<T>,
}

impl<T> WindowContent<T> {
    fn new(data: VecDeque<T>) -> Self {
        Self { data }
    }

    fn split_prefix(&mut self) -> Option<(T, &[T], &[T])> {
        // Pop the first/oldest element (owned).
        let owned = self.data.pop_front()?;

        // Borrow all remaining elements as slices.
        let (s1, s2) = self.data.as_slices();

        // Together, s1 then s2 is the suffix.
        Some((owned, s1, s2))
    }
}


fn main() {

    let mut window = WindowContent::new(VecDeque::from([0, 1, 2]));
    let (prefix, s1, s2) = window.split_prefix().unwrap();
    let suffix = s1.iter().chain(s2);

    println!("Prefix: {}", prefix);
    println!("Suffix: {:?}", suffix.collect::<Vec<_>>())

}