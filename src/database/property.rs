use crate::database::store::Store;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct PropertyIdentifier<PropertyID, TypeID> {
    pub(super) id: PropertyID,
    pub(super) type_id: TypeID,
}

struct Property<T> {
    contents: T
}
