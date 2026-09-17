// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_attributes_and_configured_rules
//@compile-flags: --crate-type lib

pub trait RequiredTrait {}
pub trait GenericRequiredTrait<T> {}
pub trait ForbiddenTrait {}

// Matches inert attributes retained after expansion.
#[repr(C)]
pub struct ReprWrongName; //~ ERROR: Struct must match pattern '^ReprAllowed', found 'ReprWrongName'

// Expansion consumes derive attributes, so they cannot satisfy HasAttribute.
#[derive(Debug)]
pub struct ReprDerivedOnlyWrongName;

#[cfg_attr(any(), repr(C))]
pub struct ReprInactiveWrongName;

#[cfg_attr(all(), repr(C))]
pub struct ReprActiveWrongName; //~ ERROR: Struct must match pattern '^ReprAllowed', found 'ReprActiveWrongName'

// Requires a trait.
pub struct NeedsTraitMissing; //~ ERROR: Struct 'NeedsTraitMissing' must implement trait matching 'test_attributes_and_configured_rules::RequiredTrait'

pub struct NeedsTraitPresent;
impl RequiredTrait for NeedsTraitPresent {}

// Requires a generic trait.
pub struct NeedsGenericTraitMissing; //~ ERROR: Struct 'NeedsGenericTraitMissing' must implement trait matching 'test_attributes_and_configured_rules::GenericRequiredTrait'

pub struct NeedsGenericTraitPresent;
impl GenericRequiredTrait<String> for NeedsGenericTraitPresent {}

// Forbids a generic trait.
pub struct MustAvoidGenericTraitBad; //~ ERROR: Struct 'MustAvoidGenericTraitBad' must not implement trait matching 'test_attributes_and_configured_rules::GenericRequiredTrait'
impl GenericRequiredTrait<u8> for MustAvoidGenericTraitBad {}

pub struct MustAvoidGenericTraitGood;

// Forbids a trait.
pub struct MustAvoidTraitBad; //~ ERROR: Struct 'MustAvoidTraitBad' must not implement trait matching 'test_attributes_and_configured_rules::ForbiddenTrait'
impl ForbiddenTrait for MustAvoidTraitBad {}

pub struct MustAvoidTraitGood;

// Tests Or.
pub struct CompositeOrBad; //~ ERROR: Struct must match pattern '^AllowedComposite', found 'CompositeOrBad'
//~^ ERROR: Struct 'CompositeOrBad' has pub visibility, but must be private

struct CompositeOrPrivate;
pub struct AllowedCompositeOr;

// Tests a negated trait matcher.
pub struct MatcherNegationBad; //~ ERROR: Struct must match pattern '^MatcherNegationAllowed', found 'MatcherNegationBad'

pub struct MatcherNegationImplemented;
impl RequiredTrait for MatcherNegationImplemented {}
