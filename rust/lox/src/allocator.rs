use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Eq, PartialEq, Debug)]
pub struct Reference<T> {
    index: usize,
    _marker: PhantomData<T>,
}

impl<T> Clone for Reference<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Reference<T> {}

impl<T> fmt::Display for Reference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reference")
            .field("index", &self.index)
            .finish()
    }
}

pub struct Allocator {
    objects: Vec<Option<Box<dyn Any>>>,
    strings: HashMap<String, Reference<String>>,
}

impl Default for Allocator {
    fn default() -> Self {
        Self {
            objects: vec![],
            strings: HashMap::new(),
        }
    }
}

impl Allocator {
    fn alloc<T: Any>(&mut self, obj: T) -> Reference<T> {
        let entry: Option<Box<dyn Any>> = Some(Box::new(obj));
        self.objects.push(entry);
        let index = self.objects.len() - 1;

        Reference {
            index,
            _marker: PhantomData,
        }
    }

    pub fn new_string(&mut self, name: String) -> Reference<String> {
        if let Some(&reference) = self.strings.get(&name) {
            return reference;
        };

        let reference = self.alloc(name.clone());
        self.strings.insert(name, reference);

        reference
    }

    pub fn deref<T: Any>(&self, reference: &Reference<T>) -> &T {
        self.objects[reference.index]
            .as_ref()
            .unwrap()
            .downcast_ref()
            .unwrap()
    }

    fn free<T: Any>(&mut self, reference: &Reference<T>) {
        self.objects[reference.index] = None;
    }
}
