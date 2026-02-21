use paste::paste;
use derive_more::Display;

pub(in crate::database) trait IDIntoUSize {
    fn as_usize(&self) -> usize;
    fn from_usize(id: usize) -> Self;
}

macro_rules! new_id {
    ($base:ident) => {
        paste! {
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Display)]
            pub(in crate::database) struct [<$base ID>](usize);
        }
    };

    ($base:ident, convert) => {
        paste! {

            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Display)]
            pub(in crate::database) struct [<$base ID>](usize);

            impl IDIntoUSize for [<$base ID>] {
                fn as_usize(&self) -> usize { self.0 }
                fn from_usize(id: usize) -> Self { [<$base ID>](id) }
            }            
        }
    };
}

// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Display)]
// #[display("{}", _0)] pub(super) struct Id<Tag>(usize, PhantomData<Tag>);

new_id!(Node, convert);
new_id!(NodeProperty, convert);
new_id!(NodePropertyType, convert);

new_id!(Edge, convert);
new_id!(EdgeProperty, convert);
new_id!(EdgePropertyType, convert);
