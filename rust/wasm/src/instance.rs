use super::{FuncType, Local, Module, ValueType};
use crate::exports::ExportDesc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};

#[derive(Clone)]
struct ModuleInstanceRef(Rc<ModuleInstance>);

impl Deref for ModuleInstanceRef {
    type Target = ModuleInstance;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct ModuleInstance {
    funcs: RefCell<Vec<FunctionInstanceRef>>,
    exports: RefCell<HashMap<String, External>>,
}

impl Default for ModuleInstance {
    fn default() -> Self {
        Self {
            funcs: RefCell::new(Vec::new()),
            exports: RefCell::new(HashMap::new()),
        }
    }
}

impl ModuleInstance {
    pub fn func_by_name(&self, name: &str) -> Option<FunctionInstanceRef> {
        if let External::Func(ref func) = self.exports.borrow().get(name)? {
            return Some(func.clone());
        }
        None
    }

    fn func_by_index(&self, idx: u32) -> Option<FunctionInstanceRef> {
        self.funcs.borrow().get(idx as usize).cloned()
    }

    fn push_func(&self, func: FunctionInstanceRef) {
        self.funcs.borrow_mut().push(func);
    }

    fn insert_export(&self, name: String, external: External) {
        self.exports.borrow_mut().insert(name, external);
    }
}

impl ModuleInstanceRef {
    pub fn instantiate(module: Module) -> Self {
        let instance = ModuleInstanceRef(Rc::new(ModuleInstance::default()));

        // TODO: resolve imports
        // TODO: resolve globals

        instance.resolve_functions(&module);

        instance.resolve_exports(&module);

        instance
    }

    fn resolve_functions(&self, module: &Module) {
        let imported_function_num = 0 as u32; // TODO

        let funcs = module
            .function_section()
            .map(|fs| fs.entries())
            .unwrap_or(&[]);
        let codes = module.code_section().map(|cs| cs.entries()).unwrap_or(&[]);

        if funcs.len() != codes.len() {
            todo!()
        }

        let func_names = match module.function_names() {
            Some(func_names) => func_names,
            None => HashMap::new(),
        };

        for (code_idx, (&type_idx, code)) in funcs.iter().zip(codes.iter()).enumerate() {
            // resolve name section
            let name = match func_names.get(&(code_idx as u32 + imported_function_num)) {
                Some(name) => name.clone(),
                None => "unknown".to_string(),
            };

            let f = FunctionInstance {
                module: Rc::downgrade(&self.0),
                name,
                signature: module
                    .type_section()
                    .map(|ts| ts.get_func_type(type_idx))
                    .expect("Due to validation type should exists")
                    .into(),
                body: code.body().to_vec(),
                locals: code.locals().to_vec(),
            };

            self.push_func(FunctionInstanceRef::build(f));
        }
    }

    fn resolve_exports(&self, module: &Module) {
        for export in module
            .export_section()
            .map(|es| es.entries())
            .unwrap_or(&[])
            .iter()
        {
            let external = match export.desc() {
                ExportDesc::Func(ref idx) => External::Func(self.func_by_index(*idx).expect("")),
                ExportDesc::Table(_) => External::Table,
                ExportDesc::Memory(_) => External::Memory,
                ExportDesc::Global(_) => External::Global,
            };

            self.insert_export(export.name().to_string(), external);
        }
    }
}

#[derive(Clone)]
pub struct FunctionInstanceRef(Rc<FunctionInstance>);

impl FunctionInstanceRef {
    fn build(instance: FunctionInstance) -> Self {
        Self(Rc::new(instance))
    }
}

pub struct FunctionInstance {
    name: String,
    signature: Signature,
    body: Vec<u8>,
    locals: Vec<Local>,
    module: Weak<ModuleInstance>,
}

impl FunctionInstanceRef {
    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn locals(&self) -> &[Local] {
        &self.0.locals
    }

    pub fn body(&self) -> &[u8] {
        &self.0.body
    }
}

struct Signature {
    params: Vec<ValueType>,
    result: Option<ValueType>,
}

impl From<&FuncType> for Signature {
    fn from(ft: &FuncType) -> Self {
        Self {
            params: ft.params().iter().cloned().collect(),
            result: ft.results().first().cloned(),
        }
    }
}

enum External {
    Func(FunctionInstanceRef),
    Table,
    Memory,
    Global,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{decode_file, Cursor},
        ModuleInstanceRef,
    };

    #[test]
    fn test_module_instance() {
        let module = decode_file("./fib.wasm").expect("should be decoded");
        let instance = ModuleInstanceRef::instantiate(module);

        let func = instance.func_by_name("fib").expect("should be exists");
        assert_eq!("fib", func.name());
        assert_eq!(1, func.locals().len());
    }
}
