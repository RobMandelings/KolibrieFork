struct Test {
    var: String,
}

impl Test {
    fn get_var(&self) -> &String {
        &self.var
    }
}

fn main() {

    let t = Test { var: "hello".to_string() };
    let r = t.get_var();

}