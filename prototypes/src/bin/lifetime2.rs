struct Data {
    number: usize
}

struct TestStruct {
    data: Data
}

impl TestStruct {

    /// Used to demonstrate one can take an &self.data reference
    /// And Rust knows that it will not outlive the parent object
    pub fn consume(&self, consume_fn: fn(&Data) -> ()) {

        // &self.data is dropped after this function call
        // &self (from which &self.data is derived) has a longer lifetime than &self.data
        consume_fn(&self.data);
    }
}

fn print_data(data: &Data) {
    println!("Lifetime 2: {}", data.number);
}

fn test_multiple_borrow_vs_single_borrow(test_struct: TestStruct) -> TestStruct {

    // &test_struct is borrowed twice here. Each reference has a lifetime that spans the scope of each function call
    TestStruct::consume(&test_struct, print_data);
    TestStruct::consume(&test_struct, print_data);

    // Same reference is used for both function calls. The reference lasts until the end of this function scope.
    let r = &test_struct;
    TestStruct::consume(r, print_data);
    TestStruct::consume(r, print_data);

    test_struct
} // The reference r is dropped here

fn main() {

    let data = Data { number: 5};
    let mut test_struct = TestStruct { data };

    // You can pass by value here (no &print_data)
    // fn() is already a function pointer
    test_struct.consume(print_data);

    // Conceptually the same as above!
    TestStruct::consume(&test_struct, print_data);

    test_struct = test_multiple_borrow_vs_single_borrow(test_struct);


}