use std::{collections::HashMap, error::Error, fmt::{Debug, Display, Formatter}, usize};

use bitvec::field;
use derive_more::{Display, Error};

use crate::database::{graph::IDIntoUSize, property_manager::{PropertyField, PropertyFieldContents, PropertyName, type_registry}, store::Store};

// TODO: Support user-defined types?
#[derive(PartialEq, Debug, Display, Clone, Copy)]
pub(super) enum FieldType {
    Integer,
    Float,
    String,
    Boolean,
}

pub(super) struct TypeDescriptor<TypeId> {
    id: TypeId,
    fields: Vec<FieldDescriptor>,
}

impl<TypeId> TypeDescriptor<TypeId> {
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

pub(super) struct FieldDescriptor {
    pub(super) name: PropertyName,
    pub(super) field_type: FieldType,
    pub(super) nullable: bool,
}

pub(super) struct TypeRegistry<TypeId> {
    types: Store<TypeDescriptor<TypeId>, TypeId>,
    type_by_name: HashMap<String, TypeId>
}

impl<TypeId> TypeRegistry<TypeId> where TypeId: Copy + IDIntoUSize + Debug {
    pub fn new() -> Self {
        // TypeRegistry { types: Store::new() }
        TypeRegistry { types: Store::new(), type_by_name: HashMap::new() }
    }

    // TODO: make this return an id
    pub fn add_type(&mut self, name: String, fields: Vec<FieldDescriptor>) -> &TypeDescriptor<TypeId> {
        let id = TypeId::from_usize(self.type_by_name.len()); // Simple ID generation strategy
        let descriptor = TypeDescriptor { id, fields };
        self.type_by_name.insert(name.to_string(), id);
        self.types.add(descriptor);

        self.types.get(*self.type_by_name.get(&name).unwrap()) 
    }

    pub fn validate_property<'a>(&'a self, id: TypeId, fields: &'a [PropertyField]) -> Result<ValidatedProperty<'a>, PropertyValidationError> {
        ValidatedProperty::new(self, id, fields)
    }
}


pub(super) struct ValidatedProperty<'a> {
    fields: &'a [PropertyField] 
}

impl <'a> ValidatedProperty<'a> {
    fn new<TypeId>(registry: &TypeRegistry<TypeId>, id: TypeId, fields: &'a [PropertyField]) -> Result<Self, PropertyValidationError> where TypeId: IDIntoUSize + Copy + Debug {
        let type_descriptor = registry.types.get(id);

        if fields.len() != type_descriptor.field_count() {
            return Err(PropertyValidationError::InvalidFieldAmmount(type_descriptor.field_count(), fields.len()));
        }

        for (idx, field) in fields.iter().enumerate() {
            if field.name != type_descriptor.fields[idx].name {
                return Err(PropertyValidationError::InvalidFieldName(type_descriptor.fields[idx].name.clone(), field.name.clone()))
            }

            match field.value {
                PropertyFieldContents::Integer(_) => {
                    if type_descriptor.fields[idx].field_type != FieldType::Integer {
                        return Err(PropertyValidationError::InvalidFieldType(type_descriptor.fields[idx].field_type, FieldType::Integer));
                    }
                },
                PropertyFieldContents::Float(_) => {
                    if type_descriptor.fields[idx].field_type != FieldType::Float {
                        return Err(PropertyValidationError::InvalidFieldType(type_descriptor.fields[idx].field_type, FieldType::Float));
                    }
                },
                PropertyFieldContents::String(_) => {
                    if type_descriptor.fields[idx].field_type != FieldType::String {
                        return Err(PropertyValidationError::InvalidFieldType(type_descriptor.fields[idx].field_type, FieldType::String));
                    }
                },
                PropertyFieldContents::Boolean(_) => {
                    if type_descriptor.fields[idx].field_type != FieldType::Boolean {
                        return Err(PropertyValidationError::InvalidFieldType(type_descriptor.fields[idx].field_type, FieldType::Boolean));
                    }
                },
            }
        }

        Ok(ValidatedProperty { fields })
    }

    pub fn fields(&self) -> &[PropertyField] {
        self.fields
    }
}

#[derive(Debug, Display)]
pub(super) enum PropertyValidationError {
    #[display("Invalid field name - in type: {}, provided: {}", _0.to_string(), _1.to_string())]
    InvalidFieldName(PropertyName, PropertyName),
    #[display("Invalid field type - in type: {}, provided: {}", _0, _1)]
    InvalidFieldType(FieldType, FieldType),
    #[display("Invalid field ammount - in type: {}, provided: {}", _0, _1)]
    InvalidFieldAmmount(usize, usize)
}

impl Error for PropertyValidationError {}
