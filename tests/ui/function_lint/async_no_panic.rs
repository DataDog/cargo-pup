//@compile-flags: --crate-name=test_async_no_panic

// Tests that NoPanic / NoUnwrap / NoIndexPanic rules work inside async
// function bodies. Rustc desugars async fns into coroutines, so the lint
// must follow into the coroutine body to find violations.

// ── should trigger NoUnwrap ──────────────────────────────────────────

async fn unwrap_option(x: Option<i32>) -> i32 {
    x.unwrap() //~ ERROR: Function may panic
}

async fn expect_result(x: Result<i32, &str>) -> i32 {
    x.expect("boom") //~ ERROR: Function may panic
}

// ── should trigger NoPanic ───────────────────────────────────────────

async fn explicit_panic() {
    panic!("async panic"); //~ ERROR: Function may panic
}

async fn uses_todo() -> i32 {
    todo!() //~ ERROR: Function may panic
}

// ── should trigger NoIndexPanic ──────────────────────────────────────

async fn index_slice(arr: &[i32]) -> i32 {
    arr[0] //~ ERROR: Function may panic
}

// ── should NOT trigger (safe alternatives) ───────────────────────────

async fn safe_unwrap_or(x: Option<i32>) -> i32 {
    x.unwrap_or(0)
}

async fn safe_match(x: Result<i32, &str>) -> i32 {
    match x {
        Ok(v) => v,
        Err(_) => -1,
    }
}

async fn safe_get(arr: &[i32]) -> Option<&i32> {
    arr.get(0)
}

// ── async methods in impl blocks ─────────────────────────────────────

struct Service;

impl Service {
    async fn method_unwraps(&self, x: Option<i32>) -> i32 {
        x.unwrap() //~ ERROR: Function may panic
    }

    async fn method_safe(&self, x: Option<i32>) -> i32 {
        x.unwrap_or(0)
    }
}

fn main() {}
