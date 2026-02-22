use std::{fmt::{self, Debug}, marker::Copy};

use derive_more::derive;

use crate::database::{graph::IDIntoUSize, property_manager::type_registry::TypeDescriptor, store::Store};

pub(super) struct PropertyStore<PropertyId, PropertyTypeId> {
    items: Store<Properties<PropertyId>, PropertyTypeId>
}

impl <PropertyId, PropertyTypeId> PropertyStore<PropertyId, PropertyTypeId> where
    PropertyTypeId: IDIntoUSize + Copy + Debug,
    PropertyId: IDIntoUSize + Debug + Copy {
    pub fn new() -> Self {
        PropertyStore { items: Store::new() }
    }

    pub fn add_type(&mut self, type_descriptor: &TypeDescriptor<PropertyTypeId>) -> PropertyTypeId {
        self.items.add(Properties::new(type_descriptor))
    }

    pub fn add_property(&mut self, type_id: PropertyTypeId, property: &[PropertyFieldContents]) {
        let fields = &mut self.items.get_mut(type_id).fields;

        if fields.is_empty() {
            for _ in 0..property.len() {
                fields.push(Store::new());
            }
        }

        debug_assert_eq!(property.len(), fields.len(), "REMOVE: property len doesnt equal type len");

        for (idx, field) in fields.iter_mut().enumerate() {
            field.add(property[idx].clone());
        }
    }
}

#[derive(Clone)]
pub(super) enum PropertyFieldContents {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

struct Properties<PropertyId> {
    fields: Vec<Store<PropertyFieldContents, PropertyId>>
}

impl<PropertyId> Properties<PropertyId> {
    fn new<PropertyTypeId>(type_descriptor: &TypeDescriptor<PropertyTypeId>) -> Self
    where
        PropertyTypeId: IDIntoUSize + Copy + Debug,
    {
        Properties { fields: Vec::with_capacity(type_descriptor.field_count()) }
    }
}

