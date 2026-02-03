//@compile-flags: --crate-name=test_no_panic

// Test functions that should pass (no panic paths)

fn pure_computation(x: i32, y: i32) -> i32 {
    x + y
}

fn uses_unwrap_or(x: Option<i32>) -> i32 {
    x.unwrap_or(0)
}

fn uses_unwrap_or_else(x: Option<i32>) -> i32 {
    x.unwrap_or_else(|| 42)
}

fn uses_unwrap_or_default(x: Option<i32>) -> i32 {
    x.unwrap_or_default()
}

fn uses_result_ok(x: Result<i32, &str>) -> Option<i32> {
    x.ok()
}

fn uses_result_err(x: Result<i32, &str>) -> Option<&str> {
    x.err()
}

fn uses_if_let(x: Option<i32>) -> i32 {
    if let Some(v) = x {
        v
    } else {
        0
    }
}

fn uses_match(x: Result<i32, &str>) -> i32 {
    match x {
        Ok(v) => v,
        Err(_) => -1,
    }
}

// Test functions that should fail (may panic)

fn uses_unwrap(x: Option<i32>) -> i32 {
    x.unwrap() //~ ERROR: Function may panic
}

fn uses_expect(x: Option<i32>) -> i32 {
    x.expect("expected a value") //~ ERROR: Function may panic
}

fn uses_result_unwrap(x: Result<i32, &str>) -> i32 {
    x.unwrap() //~ ERROR: Function may panic
}

fn uses_result_expect(x: Result<i32, &str>) -> i32 {
    x.expect("expected ok") //~ ERROR: Function may panic
}

fn uses_result_unwrap_err(x: Result<i32, &str>) -> &str {
    x.unwrap_err() //~ ERROR: Function may panic
}

fn uses_result_expect_err(x: Result<i32, &str>) -> &str {
    x.expect_err("expected error") //~ ERROR: Function may panic
}

// Note: MIR-level analysis cannot distinguish between panic!(), unreachable!(), unimplemented!(),
// todo!(), and assert!() macros - they all compile to similar underlying panic functions.
// All are detected by the NoPanic rule.

// Transitive panic tests

fn helper_panics(x: Option<i32>) -> i32 {
    x.unwrap() //~ ERROR: Function may panic
}

fn calls_panicking_function(x: Option<i32>) -> i32 {
    helper_panics(x) //~ ERROR: Function may panic
}

// Method tests
struct MyStruct {
    value: Option<i32>,
}

impl MyStruct {
    fn safe_method(&self) -> i32 {
        self.value.unwrap_or(0)
    }

    fn panicking_method(&self) -> i32 {
        self.value.unwrap() //~ ERROR: Function may panic
    }
}

// NoPanic tests (explicit panic!() calls)

fn uses_panic_macro() {
    panic!("explicit panic"); //~ ERROR: Function may panic
}

fn uses_panic_with_format() {
    panic!("panic with {}", "formatting"); //~ ERROR: Function may panic
}

// Other panic macros (all caught by NoPanic rule)

fn uses_unreachable() -> i32 {
    unreachable!() //~ ERROR: Function may panic
}

fn uses_unreachable_with_message() -> i32 {
    unreachable!("should not reach here") //~ ERROR: Function may panic
}

fn uses_unimplemented() -> i32 {
    unimplemented!() //~ ERROR: Function may panic
}

fn uses_todo() -> i32 {
    todo!() //~ ERROR: Function may panic
}

fn uses_todo_with_message() -> i32 {
    todo!("implement this later") //~ ERROR: Function may panic
}

fn uses_assert(x: i32) {
    assert!(x > 0); //~ ERROR: Function may panic
}

fn uses_assert_eq(x: i32, y: i32) {
    assert_eq!(x, y); //~ ERROR: Function may panic
}

fn uses_assert_ne(x: i32, y: i32) {
    assert_ne!(x, y); //~ ERROR: Function may panic
}

// NoIndexPanic tests

fn uses_slice_index(arr: &[i32]) -> i32 {
    arr[0] //~ ERROR: Function may panic
}

fn uses_array_index(arr: [i32; 3], idx: usize) -> i32 {
    arr[idx] //~ ERROR: Function may panic
}

// Safe alternatives (should NOT trigger errors)

fn safe_slice_get(arr: &[i32]) -> Option<&i32> {
    arr.get(0)
}

fn safe_slice_first(arr: &[i32]) -> Option<&i32> {
    arr.first()
}

fn main() {}
