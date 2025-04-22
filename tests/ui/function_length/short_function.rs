//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib
//@check-pass

// Configure a test lint with a 5-line maximum function length
// and have it apply to all code in this file
//# pup-config: |
//    short_function_test:
//      type: function_length
//      namespace: "^test$"
//      max_lines: 5
//      severity: Error

// All functions here should be below the 5 line limit

// This function is within the limit
fn acceptable_length() {
    let a = 1;
    let b = 2;
    let c = 3;
}

// Methods in impl blocks should also be checked
struct MyStruct;

impl MyStruct {
    // This method is within the limit
    fn acceptable_method(&self) {
        let a = 1;
        let b = 2;
        let c = 3;
    }
} 