// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.


///
/// Implementators of this trait should be called
/// .*MyTraitImpl, and be private.
///
pub trait MyTrait {

    fn do_something_meaningful(&self);
}


///
/// I am poorly named and poorly scoped.
///
pub struct MyBadlyNamedThing {}
impl MyTrait for MyBadlyNamedThing {

    fn do_something_meaningful(&self) {
        todo!()
    }
}

///
/// I am well named and well scoped.
///
struct MyGoodlyNamedMyTraitImpl {}
impl MyTrait for MyGoodlyNamedMyTraitImpl {
    fn do_something_meaningful(&self) {
        todo!()
    }
}
