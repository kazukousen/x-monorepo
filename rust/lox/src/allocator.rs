use crate::function::Closure;
use crate::vm::CallFrame;
use crate::{Function, Value};
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::mem;

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

impl<T> hash::Hash for Reference<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

struct Empty;

pub trait Trace {
    fn trace(&self, allocator: &mut Allocator);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Trace for Empty {
    fn trace(&self, _: &mut Allocator) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Trace for String {
    fn trace(&self, _: &mut Allocator) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Trace for Function {
    fn trace(&self, allocator: &mut Allocator) {
        allocator.mark_object(self.name);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Trace for Closure {
    fn trace(&self, allocator: &mut Allocator) {
        allocator.mark_object(self.func_id);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct ObjHeader {
    is_marked: bool,
    obj: Box<dyn Trace>,
}

impl ObjHeader {
    fn empty() -> Self {
        Self {
            is_marked: false,
            obj: Box::new(Empty {}),
        }
    }
}

pub struct Allocator {
    objects: Vec<ObjHeader>,
    free_slots: Vec<usize>,
    gray_stack: VecDeque<usize>,
    strings: HashMap<String, Reference<String>>,
}

impl Default for Allocator {
    fn default() -> Self {
        Self {
            objects: vec![],
            free_slots: vec![],
            gray_stack: VecDeque::new(),
            strings: HashMap::new(),
        }
    }
}

impl Allocator {

    pub fn should_gc(&self) -> bool {
        true
    }

    pub fn alloc<T: Trace + 'static>(&mut self, obj: T) -> Reference<T> {
        let index = match self.free_slots.pop() {
            Some(index) => index,
            None => {
                self.objects.push(ObjHeader {
                    obj: Box::new(obj),
                    is_marked: false,
                });
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
        self.objects[reference.index]
            .obj
            .as_any()
            .downcast_ref()
            .unwrap()
    }

    fn free(&mut self, index: usize) {
        self.objects[index] = ObjHeader::empty();
        self.free_slots.push(index);
    }

    pub fn collect_garbage(&mut self) {
        self.trace_references();
        self.sweep();
    }

    pub fn mark_value(&mut self, v: Value) {
        match v {
            Value::String(id) => self.mark_object(id),
            Value::Closure(id) => self.mark_object(id),
            Value::Function(id) => self.mark_object(id),
            _ => (),
        }
    }

    pub fn mark_object<T: Any>(&mut self, v: Reference<T>) {
        if self.objects[v.index].is_marked {
            return;
        }

        self.objects[v.index].is_marked = true;
        self.gray_stack.push_back(v.index);
    }

    fn mark_table(&mut self, table: &Table) {
        for (&k, &v) in table.iter() {
            self.mark_object(k);
            self.mark_value(v);
        }
    }

    fn trace_references(&mut self) {
        while let Some(index) = self.gray_stack.pop_back() {
            self.blacken_object(index);
        }
    }

    fn blacken_object(&mut self, i: usize) {
        let header = mem::replace(&mut self.objects[i], ObjHeader::empty());

        header.obj.trace(self);

        self.objects[i] = header;
    }

    fn sweep(&mut self) {
        for i in 0..self.objects.len() {
            if self.objects[i].is_marked {
                self.objects[i].is_marked = false;
            } else {
                self.free(i);
            }
        }
    }
}

pub type Table = HashMap<Reference<String>, Value>;
