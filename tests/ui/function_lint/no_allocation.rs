//@compile-flags: --crate-name=test_crate

// Test functions that should pass (no allocations)

fn pure_computation(x: i32, y: i32) -> i32 {
    x + y
}

fn stack_only(x: i32) -> [i32; 4] {
    [x, x + 1, x + 2, x + 3]
}

fn uses_references(x: &str) -> usize {
    x.len()
}

// Test functions that should fail (allocate)

fn allocates_box() -> Box<i32> {
    Box::new(42) //~ ERROR: Function allocates heap memory
}

fn allocates_vec_new() {
    let _v = Vec::<i32>::new(); //~ ERROR: Function allocates heap memory
}

fn allocates_string_new() {
    let _s = String::new(); //~ ERROR: Function allocates heap memory
}

fn allocates_to_string(x: i32) -> String {
    x.to_string() //~ ERROR: Function allocates heap memory
}

fn allocates_rc() -> std::rc::Rc<i32> {
    std::rc::Rc::new(42) //~ ERROR: Function allocates heap memory
}

fn allocates_arc() -> std::sync::Arc<i32> {
    std::sync::Arc::new(42) //~ ERROR: Function allocates heap memory
}

fn allocates_hashmap() {
    let _map = std::collections::HashMap::<i32, i32>::new(); //~ ERROR: Function allocates heap memory
}

// Transitive allocation tests

fn helper_allocates() -> Vec<i32> {
    let v = Vec::new(); //~ ERROR: Function allocates heap memory
    v
}

fn calls_allocating_function() -> usize {
    let v = helper_allocates(); //~ ERROR: Function allocates heap memory
    v.len()
}

// Method tests
struct MyStruct {
    value: i32,
}

impl MyStruct {
    fn no_alloc_method(&self) -> i32 {
        self.value * 2
    }

    fn alloc_method(&self) -> String {
        self.value.to_string() //~ ERROR: Function allocates heap memory
    }
}

fn main() {}
