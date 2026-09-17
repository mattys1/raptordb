use std::fmt::Debug;

use derive_more::{Display, From};

use crate::database::{graph::IDIntoUSize, property_manager::{property::{PropertyFieldContents, PropertyStore}, type_registry::{FieldDescriptor, PropertyValidationError, TypeDescriptor, TypeRegistry, ValidatedProperty}}};

pub(crate) mod type_registry;
pub(crate) mod property;

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

    pub fn register_type(&mut self, descriptor: TypeDescriptor) -> PropertyTypeId {
        let id = self.type_registry.add_type(descriptor);
        let descriptor = self.type_registry.get_type(id);
        self.property_store.add_type(descriptor)
    }

    pub fn add_property(&mut self, id: PropertyTypeId, field_contents: &[PropertyField]) -> Result<PropertyId, PropertyValidationError> {
        let validated = self.type_registry.validate_property(id, field_contents)?;
        Ok(self.property_store.add_property(id, &validated))
    }

    pub fn get_type_by_name(&self, name: &str) -> Option<PropertyTypeId> {
        self.type_registry.get_by_name(name)
    }

    pub fn describe(&self, id: PropertyTypeId) -> &TypeDescriptor {
        self.type_registry.get_type(id)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct PropertyIdentifier<PropertyID, TypeID> {
    pub(super) id: PropertyID,
    pub(super) type_id: TypeID,
}

pub(crate) struct PropertyField {
    name: PropertyName,
    value: PropertyFieldContents,
}

impl PropertyField {
    pub fn new(name: PropertyName, value: PropertyFieldContents) -> Self {
        Self { name, value }
    }
}

#[derive(Clone, From, PartialEq, Eq, Debug, Display)]
pub(crate) struct PropertyName(String);

#[cfg(test)]
mod tests {
    use crate::database::{NodePropertyID, NodePropertyTypeID, property_manager::{FieldDescriptor, PropertyField, PropertyFieldContents, PropertyManager, PropertyValidationError, type_registry::{FieldType, TypeDescriptor}}};

    fn get_manager() -> PropertyManager::<NodePropertyID, NodePropertyTypeID> {
        PropertyManager::<NodePropertyID, NodePropertyTypeID>::new()
    }

    #[test]
    fn import_type_test() {
        let mut manager = get_manager();
        let id = manager.register_type(TypeDescriptor {
            name: "test_type".into(),
            fields: vec![
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
            ]
        });

        let result = manager.add_property(id, &[ 
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
        fn import_invalid_type_test() {
            let mut manager = get_manager();
            let id = manager.register_type(TypeDescriptor {
                name: "test_type".into(),
                fields: vec![
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
                ]
            });

            let result = manager.add_property(id, &[ 
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
            fn import_invalid_field_count_test() {
                let mut manager = get_manager();
                let id = manager.register_type(TypeDescriptor {
                    name: "count_test".into(),
                    fields: vec![
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
                    ]
                });

                let result = manager.add_property(id, &[
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
                fn import_invalid_field_name_test() {
                    let mut manager = get_manager();
                    let id = manager.register_type(TypeDescriptor {
                        name: "name_test".into(),
                        fields: vec![
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
                        ]
                    });

                    let result = manager.add_property(id, &[
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

                assert!(matches!(
                result,
                Err(PropertyValidationError::InvalidFieldName(expected, provided))
                if expected.to_string() == "field2" && provided.to_string() == "wrong_name"
            ));
            }
        }
    }
}
