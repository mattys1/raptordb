use std::{fmt::{self, Debug}, marker::Copy};

use derive_more::{Display, derive};

use crate::database::{graph::IDIntoUSize, property_manager::{PropertyField, ValidatedProperty, type_registry::TypeDescriptor}, store::Store};

pub(super) struct PropertyStore<PropertyId, PropertyTypeId> {
    items: Store<Properties<PropertyId>, PropertyTypeId>
}

impl <PropertyId, PropertyTypeId> PropertyStore<PropertyId, PropertyTypeId> where
    PropertyTypeId: IDIntoUSize + Copy + Debug,
    PropertyId: IDIntoUSize + Debug + Copy {
    pub fn new() -> Self {
        PropertyStore { items: Store::new() }
    }

    pub fn add_type(&mut self, type_descriptor: &TypeDescriptor) -> PropertyTypeId {
        self.items.add(Properties::new::<PropertyTypeId>(type_descriptor))
    }

    pub fn add_property(&mut self, type_id: PropertyTypeId, property: &ValidatedProperty) -> PropertyId {
        let fields = &mut self.items.get_mut(type_id).fields;

        if fields.is_empty() {
            for _ in 0..property.fields().len() {
                fields.push(Store::new());
            }
        }

        debug_assert_eq!(property.fields().len(), fields.len(), "REMOVE: property len doesnt equal type len");

        for (idx, field) in fields.iter_mut().enumerate() {
            field.add(property.fields()[idx].value.clone());
        }

        PropertyId::from_usize(self.items.get(type_id).property_count())
    }
}

#[derive(Clone, Debug, Display)]
pub(crate) enum PropertyFieldContents {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

struct Properties<PropertyId> {
    fields: Vec<Store<PropertyFieldContents, PropertyId>>
}

impl<PropertyId> Properties<PropertyId> {
    fn new<PropertyTypeId>(type_descriptor: &TypeDescriptor) -> Self {
        Properties { fields: Vec::with_capacity(type_descriptor.field_count()) }
    }

    fn property_count(&self) -> usize {
        if self.fields.is_empty() {
            0
        } else {
            debug_assert!(!self.fields.iter().any(|field| field.len() != self.fields[0].len()));

            self.fields[0].len()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::property_manager::{TypeRegistry, type_registry::TypeDescriptor};

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct DummyId(usize);

    impl IDIntoUSize for DummyId {
        fn from_usize(val: usize) -> Self { DummyId(val) }
        fn as_usize(&self) -> usize { self.0 }
    }

    #[test]
    fn test_add_property_returns_correct_id() {
        let type_desc = TypeDescriptor {
            name: "dummy".into(),
            fields: vec![],
        };
        let mut store = PropertyStore::<DummyId, DummyId>::new();
        let type_id = store.add_type(&type_desc);

        // Create a TypeRegistry and validate the property
        let mut registry = TypeRegistry::<DummyId>::new();
        let reg_type_id = registry.add_type(type_desc.clone());
        let validated_property = registry.validate_property(reg_type_id, &[]).unwrap();

        let property_id = store.add_property(type_id, &validated_property);

        // Since this is the first property, expect id 0
        assert_eq!(property_id, DummyId(0));
    }
}
