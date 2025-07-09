//@compile-flags: --crate-name test_builder_style
//@compile-flags: --crate-type lib

// This test verifies builder-style lints: "with_" methods returning `Self` are forbidden,
// while "set_" methods returning `&mut Self` are allowed.

pub struct WidgetBuilder {
    val: i32,
}

impl WidgetBuilder {
    // This should trigger the MustNotExist rule
    pub fn with_val(mut self, val: i32) -> Self { //~ ERROR: Function 'with_val' is forbidden by lint rule
        self.val = val;
        self
    }

    // This is the preferred style and should compile cleanly
    pub fn set_val(&mut self, val: i32) -> &mut Self {
        self.val = val;
        self
    }

    // Name matches the forbidden prefix but uses an allowed return type (&mut Self) – should be OK
    pub fn with_val_ref(&mut self, val: i32) -> &mut Self {
        self.val = val;
        self
    }

    // Opposite rule: name starts with "set_" but returns Self – should trigger error
    pub fn set_val_value(self, val: i32) -> Self { //~ ERROR: Function 'set_val_value' is forbidden by lint rule
        Self { val }
    }

    // Control method that matches neither rule
    pub fn touch(&self) {
        let _ = self.val;
    }
} 