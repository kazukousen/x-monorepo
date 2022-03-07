use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Eq, Debug)]
pub struct Reference<T> {
    index: usize,
    _marker: PhantomData<T>,
}

impl<T> PartialEq for Reference<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Clone for Reference<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Reference<T> {}

impl<T> fmt::Display for Reference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ref({})", self.index)
    }
}

pub struct Allocator {
    objects: Vec<Box<dyn Any>>,
    free_slots: Vec<usize>,
    strings: HashMap<String, Reference<String>>,
}

impl Default for Allocator {
    fn default() -> Self {
        Self {
            objects: vec![],
            free_slots: vec![],
            strings: HashMap::new(),
        }
    }
}

struct Empty;

impl Allocator {
    pub fn alloc<T: Any>(&mut self, obj: T) -> Reference<T> {
        let entry: Box<dyn Any> = Box::new(obj);

        let index = match self.free_slots.pop() {
            Some(index) => index,
            None => {
                self.objects.push(entry);
                self.objects.len() - 1
            }
        };

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
        self.objects[reference.index].downcast_ref().unwrap()
    }

    fn free<T: Any>(&mut self, reference: &Reference<T>) {
        self.objects[reference.index] = Box::new(Empty);
        self.free_slots.push(reference.index);
    }
}
