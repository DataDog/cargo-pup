pub struct WidgetBuilder {
    width: u32,
    height: u32,
}

impl WidgetBuilder {
    // BAD: Consuming builder – should be caught by the lint (returns `Self`)
    pub fn with_width(self, width: u32) -> Self {
        Self { width, ..self }
    }

    // BAD: Consumes self but uses a `set_` prefix – also forbidden
    pub fn set_height(self, height: u32) -> Self {
        Self { height, ..self }
    }

    // GOOD: Reference-based setter – allowed (`&mut self` → `&mut Self`)
    pub fn set_width(&mut self, width: u32) -> &mut Self {
        self.width = width;
        self
    }

    // GOOD: Reference-based "with_" method – allowed
    pub fn with_height(&mut self, height: u32) -> &mut Self {
        self.height = height;
        self
    }
} 