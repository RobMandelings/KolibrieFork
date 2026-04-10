struct WindowContent<T> {
    data: Vec<T>,
}

impl<T> WindowContent<T> {
    fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    fn split_prefix(&mut self) -> Option<(Vec<T>, &[T])> {
        if self.data.is_empty() {
            return None;
        }

        // move out k elements from the front
        let prefix: Vec<T> = self.data.drain(0..3).collect();

        // Remaining elements form one contiguous suffix.
        let suffix: &[T] = &self.data;
        Some((prefix, suffix))
    }
}

fn main() {
    let mut window = WindowContent::new(vec![0, 1, 2, 3, 4, 5, 6]);
    let (prefix, suffix) = window.split_prefix().unwrap();

    // window.data.insert(4, 4);

    println!("Prefix: {:?}", prefix);
    println!("Suffix: {:?}", suffix);

}