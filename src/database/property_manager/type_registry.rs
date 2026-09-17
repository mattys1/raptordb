use std::{collections::HashMap, error::Error, fmt::{Debug, Display, Formatter}, usize};

use bitvec::field;
use derive_more::{Display, Error};

use crate::database::{graph::IDIntoUSize, property_manager::{PropertyField, PropertyFieldContents, PropertyName, type_registry}, store::Store};

// TODO: Support user-defined types?
#[derive(PartialEq, Debug, Display, Clone, Copy)]
pub(crate) enum FieldType {
    Integer,
    Float,
    String,
    Boolean,
}

#[derive(Debug, Clone)]
pub(crate) struct TypeDescriptor {
    pub(in crate::database) name: String,
    pub(in crate::database) fields: Vec<FieldDescriptor>,
}

impl TypeDescriptor {
    pub fn new(name: String, fields: Vec<FieldDescriptor>) -> Self {
        TypeDescriptor { name, fields }
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FieldDescriptor {
    pub(in crate::database) name: PropertyName,
    pub(in crate::database) field_type: FieldType,
    pub(in crate::database) nullable: bool,
}

impl FieldDescriptor {
   pub fn new(name: PropertyName, field_type: FieldType, nullable: bool) -> Self {
        FieldDescriptor { name, field_type, nullable }
    } 
}

pub(super) struct TypeRegistry<TypeId> {
    types: Store<TypeDescriptor, TypeId>,
    type_by_name: HashMap<String, TypeId>
}

impl<TypeId> TypeRegistry<TypeId> where TypeId: Copy + IDIntoUSize + Debug {
    pub fn new() -> Self {
        // TypeRegistry { types: Store::new() }
        TypeRegistry { types: Store::new(), type_by_name: HashMap::new() }
    }

    // TODO: make this return an id
    pub fn add_type(&mut self, descriptor: TypeDescriptor) -> TypeId {
        let id = TypeId::from_usize(self.type_by_name.len());
        self.type_by_name.insert(descriptor.name.clone(), id);
        self.types.add(descriptor);

        id
    }

    pub fn validate_property<'a>(&'a self, id: TypeId, fields: &'a [PropertyField]) -> Result<ValidatedProperty<'a>, PropertyValidationError> {
        ValidatedProperty::new(self, id, fields)
    }

    pub fn get_type(&self, id: TypeId) -> &TypeDescriptor {
        debug_assert!(self.types.exists(id), "Tried retrieving non existant type");

        self.types.get(id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<TypeId> {
        self.type_by_name.get(name).copied()
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
