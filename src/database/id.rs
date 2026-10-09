use std::fmt::Debug;

use derive_more::{Display, From, Into};
use paste::paste;

pub(in crate::database) trait Id:
    Copy + Debug + From<usize> + Into<usize>
{
}

macro_rules! new_id {
    ($base:ident) => {
        paste! {
            #[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Display, From, Into)]
            pub(in crate::database) struct [<$base ID>](usize);

            impl Id for [<$base ID>] {}
        }
    };
}

new_id!(Node);
new_id!(NodeProperty);
new_id!(NodePropertyType);

new_id!(Edge);
new_id!(EdgeProperty);
new_id!(EdgePropertyType);

new_id!(Cluster);
