struct Data {
    number: usize
}

struct TestStruct {
    data: Data
}

fn print_data(test_struct: &TestStruct) {

    let test_ref = &test_struct;

    println!("{}", test_ref.data.number);
}

fn main() {

    let data = Data { number: 5};
    let test_struct = TestStruct { data };
    print_data(&test_struct);

}