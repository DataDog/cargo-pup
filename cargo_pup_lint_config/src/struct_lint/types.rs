// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::Severity;
use serde::{Deserialize, Serialize};

/// Specifies how to match structs for linting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructMatch {
    /// Match structs by name (exact name or regex pattern)
    Name(String),
    /// Match the name of an attribute that remains after macro expansion.
    ///
    /// Supported built-ins are `deprecated`, `doc`, `must_use`, `non_exhaustive`, and `repr`;
    /// arguments are unavailable. Custom and tool attributes can match if they survive expansion.
    /// Consumed attributes such as `derive` and attribute procedural macros cannot match. An active
    /// `cfg_attr` may match a supported resulting attribute, but not the original expression.
    HasAttribute(String),
    /// Match structs that implement a specific trait.
    ///
    /// Generic arguments are not part of the trait path. A generic trait matches when the
    /// struct implements it for any arguments; associated type values are not constrained.
    ImplementsTrait(String),
    /// Logical AND - both patterns must match
    AndMatches(Box<StructMatch>, Box<StructMatch>),
    /// Logical OR - either pattern must match
    OrMatches(Box<StructMatch>, Box<StructMatch>),
    /// Logical NOT - inverts the match
    NotMatch(Box<StructMatch>),
}

/// A complete struct lint definition with matching criteria and rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLint {
    pub name: String,
    pub matches: StructMatch,
    pub rules: Vec<StructRule>,
}

/// Rules that can be applied to structs matching specific criteria
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructRule {
    /// Enforces that the struct name matches the specified pattern
    MustBeNamed(String, Severity),
    /// Enforces that the struct name does not match the specified pattern
    MustNotBeNamed(String, Severity),
    /// Enforces that the struct has private visibility (not pub, not pub(crate), not pub(super))
    MustBePrivate(Severity),
    /// Enforces that the struct has public visibility (pub)
    MustBePublic(Severity),
    /// Enforces that the struct has pub(crate) visibility
    MustBePubCrate(Severity),
    /// Enforces that the struct implements a specific trait.
    ///
    /// Generic arguments are not part of the trait path. A generic trait is implemented when
    /// any valid arguments exist; associated type values are not constrained.
    ImplementsTrait(String, Severity),
    /// Logical AND - both rules must pass
    And(Box<StructRule>, Box<StructRule>),
    /// Logical OR - either rule must pass
    Or(Box<StructRule>, Box<StructRule>),
    /// Logical NOT - inverts the rule check
    Not(Box<StructRule>),
}
