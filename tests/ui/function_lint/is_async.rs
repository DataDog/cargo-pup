// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_is_async
//@compile-flags: --crate-type lib

// This test verifies that the FunctionLint's IsAsync matcher works correctly

// Async function that should trigger the lint
async fn async_function() { //~ ERROR: Function 'async_function' is forbidden by lint rule
}

// Async function with return type
async fn async_with_return() -> String { //~ ERROR: Function 'async_with_return' is forbidden by lint rule
    "hello".to_string()
}

// Regular function that should NOT trigger the lint
fn sync_function() {
}

// Another regular function with return type
fn sync_with_return() -> String {
    "hello".to_string()
}

// Async method in impl block
struct TestStruct;

impl TestStruct {
    async fn async_method(&self) { //~ ERROR: Function 'async_method' is forbidden by lint rule
    }
    
    fn sync_method(&self) {
    }
}

// Async functions in traits
trait AsyncTrait {
    async fn trait_async_method(&self);
    
    fn trait_sync_method(&self);
}

impl AsyncTrait for TestStruct {
    async fn trait_async_method(&self) { //~ ERROR: Function 'trait_async_method' is forbidden by lint rule
    }
    
    fn trait_sync_method(&self) {
    }
}