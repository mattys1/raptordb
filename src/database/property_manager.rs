use std::fmt::Debug;

use crate::database::{graph::IDIntoUSize, property_manager::{property::{PropertyFieldContents, PropertyStore}, type_registry::{FieldDescriptor, TypeRegistry}}};

mod type_registry;
mod property;

pub(super) struct PropertyManager<PropertyId, PropertyTypeId> {
    type_registry: TypeRegistry<PropertyTypeId>,
    property_store: PropertyStore<PropertyId, PropertyTypeId>
}

impl <PropertyId, PropertyTypeId> PropertyManager<PropertyId, PropertyTypeId> where
    PropertyId: IDIntoUSize + Copy + Debug,
    PropertyTypeId: IDIntoUSize + Copy + Debug {
    pub fn new() -> Self {
        Self { type_registry: TypeRegistry::new(), property_store: PropertyStore::new() }
    }

    pub fn register_type(&mut self, fields: Vec<FieldDescriptor>) -> PropertyTypeId {
        let descriptor = self.type_registry.add_type("test_type".into(), fields);

        self.property_store.add_type(descriptor)
    }

    pub fn add_node_property(&mut self, id: PropertyTypeId, field_contents: &[PropertyFieldContents]) {
        self.property_store.add_property(id, field_contents);
    }
}

#[cfg(test)]
mod tests {
    use crate::database::{NodePropertyID, NodePropertyTypeID, property_manager::{FieldDescriptor, PropertyFieldContents, PropertyManager, type_registry::FieldType}};

    #[test]
    fn import_type_test() {
        let mut manager = PropertyManager::<NodePropertyID, NodePropertyTypeID>::new();
        let id = manager.register_type(vec![
            FieldDescriptor {
                name: "field1".into(),
                field_type: FieldType::Integer,
                nullable: false,
            },
            FieldDescriptor {
                name: "field2".into(),
                field_type: FieldType::String,
                nullable: false,
            }
        ]);

        manager.add_node_property(id, &[
            PropertyFieldContents::Integer(1),
            PropertyFieldContents::String("test".into()),
        ]);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct PropertyIdentifier<PropertyID, TypeID> {
    pub(super) id: PropertyID,
    pub(super) type_id: TypeID,
}
