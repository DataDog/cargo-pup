use std::collections::HashMap;

const MY_CONST: u32 = 42;

pub fn my_fn(a: u32, b: u32) -> u32 {
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    println!("MY_CONST: {}", MY_CONST);
    a + b
}

pub struct MyStruct {}

impl MyStruct {
    pub fn do_something(&self) {
        println!("{:?}", my_fn(1, 2));
        let m: HashMap<String, String> = HashMap::new();
    }
}
