use std::fmt::Debug;

use derive_more::{Display, From};

use crate::database::{graph::IDIntoUSize, property_manager::{property::{PropertyFieldContents, PropertyStore}, type_registry::{FieldDescriptor, PropertyValidationError, TypeRegistry, ValidatedProperty}}};

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

    pub fn register_type(&mut self, name: String, fields: Vec<FieldDescriptor>) -> PropertyTypeId {
        let descriptor = self.type_registry.add_type(name, fields);
        self.property_store.add_type(descriptor)
    }

    pub fn add_node_property(&mut self, id: PropertyTypeId, field_contents: &[PropertyField]) -> Result<(), PropertyValidationError> {
        let validated = self.type_registry.validate_property(id, field_contents)?;
        self.property_store.add_property(id, &validated);
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct PropertyIdentifier<PropertyID, TypeID> {
    pub(super) id: PropertyID,
    pub(super) type_id: TypeID,
}

struct PropertyField {
    name: PropertyName,
    value: PropertyFieldContents,
}

#[derive(Clone, From, PartialEq, Eq, Debug, Display)]
struct PropertyName(String);

#[cfg(test)]
mod tests {
    use crate::database::{NodePropertyID, NodePropertyTypeID, property_manager::{FieldDescriptor, PropertyField, PropertyFieldContents, PropertyManager, PropertyValidationError, type_registry::FieldType}};

    fn get_manager() -> PropertyManager::<NodePropertyID, NodePropertyTypeID> {
        PropertyManager::<NodePropertyID, NodePropertyTypeID>::new()
    }

    #[test]
    fn import_type_test() {
        let mut manager = get_manager();
        let id = manager.register_type("test_type".into(), vec![
            FieldDescriptor {
                name: String::from("field1").into(),
                field_type: FieldType::Integer,
                nullable: false,
            },
            FieldDescriptor {
                name: String::from("field2").into(),
                field_type: FieldType::String,
                nullable: false,
            }
        ]);

        let result = manager.add_node_property(id, &[ 
            PropertyField {
                name: String::from("field1").into(),
                value: PropertyFieldContents::Integer(1),
            },

            PropertyField {
                name: String::from("field2").into(),
                value: PropertyFieldContents::String("test".into()),
            },
            
        ]);

        assert!(result.is_ok());
    }

    #[test]
    fn import_invalid_type_test() {
        let mut manager = get_manager();
        let id = manager.register_type("test_type".into(), vec![
            FieldDescriptor {
                name: String::from("field1").into(),
                field_type: FieldType::Integer,
                nullable: false,
            },
            FieldDescriptor {
                name: String::from("field2").into(),
                field_type: FieldType::String,
                nullable: false,
            }
        ]);

        let result = manager.add_node_property(id, &[ 
            PropertyField {
                name: String::from("field1").into(),
                value: PropertyFieldContents::Integer(1),
            },

            PropertyField {
                name: String::from("field2").into(),
                value: PropertyFieldContents::Integer(1),
            },
            
        ]);

        assert!(matches!(
            result,
            Err(PropertyValidationError::InvalidFieldType(FieldType::String, FieldType::Integer))
        ));
    }

    #[test]
    fn import_invalid_field_count_test() {
        let mut manager = get_manager();
        let id = manager.register_type("count_test".into(), vec![
            FieldDescriptor {
                name: String::from("a").into(),
                field_type: FieldType::Integer,
                nullable: false,
            },
            FieldDescriptor {
                name: String::from("b").into(),
                field_type: FieldType::String,
                nullable: false,
            },
            FieldDescriptor {
                name: String::from("c").into(),
                field_type: FieldType::Boolean,
                nullable: false,
            }
        ]);

        let result = manager.add_node_property(id, &[
            PropertyField {
                name: String::from("a").into(),
                value: PropertyFieldContents::Integer(1),
            },
            PropertyField {
                name: String::from("b").into(),
                value: PropertyFieldContents::String("x".into()),
            },
        ]);

        assert!(matches!(
            result,
            Err(PropertyValidationError::InvalidFieldAmmount(3, 2))
        ));
    }

    #[test]
    fn import_invalid_field_name_test() {
        let mut manager = get_manager();
        let id = manager.register_type("name_test".into(), vec![
            FieldDescriptor {
                name: String::from("field1").into(),
                field_type: FieldType::Integer,
                nullable: false,
            },
            FieldDescriptor {
                name: String::from("field2").into(),
                field_type: FieldType::String,
                nullable: false,
            }
        ]);

        let result = manager.add_node_property(id, &[
            PropertyField {
                name: String::from("field1").into(),
                value: PropertyFieldContents::Integer(1),
            },
            PropertyField {
                name: String::from("wrong_name").into(),
                value: PropertyFieldContents::String("test".into()),
            },
        ]);

        assert!(matches!(
            result,
            Err(PropertyValidationError::InvalidFieldName(expected, provided))
            if expected.to_string() == "field2" && provided.to_string() == "wrong_name"
        ));
    }
}
