// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

pub mod and_bad {}

pub mod and_good {
    pub const VALUE: usize = 1;
}

pub mod or_named {}

pub mod or_content {
    pub const VALUE: usize = 1;
}

pub mod or_bad {}

pub mod must_be_empty {}

pub mod not_bad {
    pub const VALUE: usize = 1;
}
