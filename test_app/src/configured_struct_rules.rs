// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

pub trait RequiredTrait {}
pub trait ForbiddenTrait {}

#[repr(C)]
pub struct AppReprWrongName;

#[derive(Debug)]
pub struct AppReprDerivedOnlyWrongName;

#[cfg_attr(any(), repr(C))]
pub struct AppReprInactiveWrongName;

#[cfg_attr(all(), repr(C))]
pub struct AppReprActiveWrongName;

pub struct AppNeedsTraitMissing;

pub struct AppNeedsTraitPresent;
impl RequiredTrait for AppNeedsTraitPresent {}

pub struct AppMustAvoidTraitBad;
impl ForbiddenTrait for AppMustAvoidTraitBad {}

pub struct AppMustAvoidTraitGood;

pub struct AppCompositeOrBad;
struct AppCompositeOrPrivate;
pub struct AppCompositeOrAllowed;

pub struct AppMatcherNegationBad;

pub struct AppMatcherNegationImplemented;
impl RequiredTrait for AppMatcherNegationImplemented {}
