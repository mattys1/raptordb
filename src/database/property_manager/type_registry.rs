use std::{collections::HashMap, fmt::Debug, usize};

use crate::database::{graph::IDIntoUSize, store::Store};

// TODO: Support user-defined types?
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
    pub(super) name: String,
    pub(super) field_type: FieldType,
    pub(super) nullable: bool,
}

pub(super) struct TypeRegistry<TypeId> {
    // types: Store<TypeDescriptor<TypeId>, TypeId>,
    type_by_name: HashMap<String, TypeDescriptor<TypeId>>
}

impl<TypeId> TypeRegistry<TypeId> where TypeId: Copy + IDIntoUSize + Debug {
    pub fn new() -> Self {
        // TypeRegistry { types: Store::new() }
        TypeRegistry { type_by_name: HashMap::new() }
    }

    pub fn add_type(&mut self, name: String, fields: Vec<FieldDescriptor>) -> &TypeDescriptor<TypeId> {
        let id = TypeId::from_usize(self.type_by_name.len()); // Simple ID generation strategy
        let descriptor = TypeDescriptor { id, fields };
        self.type_by_name.insert(name.clone(), descriptor);

        self.type_by_name.get(&name).unwrap() 
    }
}
