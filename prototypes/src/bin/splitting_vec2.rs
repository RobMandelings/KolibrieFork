struct WindowContent<T> {
    data: Vec<T>,
}

impl<T> WindowContent<T> {
    fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    fn get_prefix(&mut self) -> Vec<T> {
        if self.data.is_empty() {
            return vec![];
        }

        // move out k elements from the front
        let prefix: Vec<T> = self.data.drain(0..3).collect();
        prefix
    }

    fn get_suffix(&self) -> &[T] {
        // Remaining elements form one contiguous suffix.
        let suffix: &[T] = &self.data;
        suffix
    }
}

fn main() {
    let mut window = WindowContent::new(vec![0, 1, 2, 3, 4, 5, 6]);
    let prefix = window.get_prefix();

    // An immutable reference is derived from immutable borrow, does that make a difference?
    let suffix = window.get_suffix();

    println!("Prefix: {:?}", prefix);
    println!("Suffix: {:?}", suffix);

}