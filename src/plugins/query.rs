use crate::call::Call;
use crate::encoding::{Decode, Encode};
use crate::orga;
use crate::query::Query as QueryTrait;
use crate::Result;
use educe::Educe;
use std::cell::RefCell;

#[orga(skip(Query))]
#[state(transparent)]
// TODO: #[call(transparent)]
pub struct QueryPlugin<T> {
    inner: RefCell<T>,
}

#[derive(Clone, Encode, Decode, Educe)]
#[educe(Debug)]
pub enum Query<T: QueryTrait + Call> {
    Inner(T::Query),
    CallSimulation(T::Call),
    RawKey(Vec<u8>),
}

impl<T> QueryTrait for QueryPlugin<T>
where
    T: QueryTrait + Call,
{
    type Query = Query<T>;

    fn query(&self, query: Self::Query) -> Result<()> {
        match query {
            Query::Inner(inner) => self.inner.borrow().query(inner),
            Query::CallSimulation(call) => self.inner.borrow_mut().call(call),
            Query::RawKey(_key) => {
                // TODO
                Ok(())
            }
        }
    }
}
// TODO: abci method passthroughs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call::build_call;
    use crate::call::FieldCall;
    use crate::query::FieldQuery;
    use crate::state::State;

    #[derive(State, FieldCall, Default, Debug)]
    struct Bloop {
        #[call]
        pub app: Intermediate,
    }

    #[derive(State, FieldCall, Default, Debug)]
    pub struct Intermediate {
        #[call]
        pub baz: Baz,
        #[call]
        pub foo: Foo,
    }

    #[orga]
    #[derive(Debug)]
    pub struct Foo {
        pub a: u32,
        pub b: u32,
        pub c: (u32, u32),
        pub d: Baz,
    }

    #[orga]
    impl Foo {
        #[call]
        pub fn inc_a(&mut self, n: u32) -> Result<()> {
            self.a += n;

            Ok(())
        }
    }

    #[orga]
    #[derive(Debug)]
    pub struct MyApp {
        #[call]
        pub foo: Foo,
    }

    #[orga]
    #[derive(Debug)]
    pub struct Baz {
        beep: u32,
        boop: u8,
    }

    #[orga]
    impl Baz {
        #[call]
        pub fn inc_beep(&mut self, n: u32) -> Result<()> {
            self.beep += n;

            Ok(())
        }

        #[call]
        pub fn other_baz_method(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[derive(State, FieldCall, Default, FieldQuery)]
    pub struct Bleep {
        pub a: u32,
        pub b: u64,
    }

    #[orga]
    impl Bleep {
        #[query]
        fn my_query(&self, n: u32) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn call_sim() -> Result<()> {
        let mut bloop = Bloop::default();
        let client = &mut bloop;
        let call_one = build_call!(client.app.baz.inc_beep(10));
        let client = &mut bloop;
        let call_two = build_call!(client.app.baz.inc_beep(15));

        dbg!(&bloop);
        bloop.call(call_one)?;
        bloop.call(call_two)?;
        dbg!(&bloop);
        assert_eq!(bloop.app.baz.beep, 25);
        Ok(())
    }
}
