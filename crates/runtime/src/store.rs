use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{Environment, ImportPath, Stack};

#[derive(Debug, Default)]
pub struct Store {
    modules: HashMap<PathBuf, RuntimeUnit>,
    main: RuntimeUnit,
    current: Option<PathBuf>,
    builtins: Option<Rc<Environment>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            modules: HashMap::new(),
            main: RuntimeUnit::default(),
            current: None,
            builtins: None,
        }
    }

    pub fn with_builtins(builtins: Environment) -> Self {
        let builtins = Rc::new(builtins);

        Store {
            modules: HashMap::new(),
            main: RuntimeUnit {
                stack: Stack::with_base(builtins.clone()),
                imports: vec![],
            },
            current: None,
            builtins: Some(builtins),
        }
    }

    pub fn switch_to_main(&mut self) {
        self.current = None;
    }

    pub fn switch_to_module(&mut self, path: PathBuf) {
        self.current = Some(path);
    }

    pub fn get_current(&self) -> &Stack {
        if let Some(path) = &self.current {
            match self.modules.get(path) {
                Some(m) => &m.stack,
                None => &self.main.stack,
            }
        } else {
            &self.main.stack
        }
    }

    pub fn get_current_mut(&mut self) -> &mut Stack {
        if let Some(path) = &self.current {
            match self.modules.get_mut(path) {
                Some(u) => &mut u.stack,
                None => &mut self.main.stack,
            }
        } else {
            &mut self.main.stack
        }
    }

    pub fn find_module(&self, path: &Path) -> Option<&Stack> {
        self.modules.get(path).map(|m| &m.stack)
    }

    pub fn add_import_to_current(&mut self, import: ImportPath) {
        if let Some(path) = &self.current {
            if let Some(module) = self.modules.get_mut(path) {
                module.imports.push(import);
            } else {
                self.create_module_helper(path.clone(), vec![import]);
            }
        } else {
            self.main.imports.push(import);
        }
    }

    pub fn create_module(&mut self, path: PathBuf) {
        self.create_module_helper(path, vec![]);
    }
}

impl Store {
    fn create_module_helper(
        &mut self,
        path: PathBuf,
        imports: Vec<ImportPath>,
    ) -> &mut RuntimeUnit {
        self.modules.entry(path).or_insert(RuntimeUnit {
            stack: if let Some(builtins) = &self.builtins {
                Stack::with_base(builtins.clone())
            } else {
                Stack::default()
            },
            imports,
        })
    }
}

#[derive(Debug, Default)]
struct RuntimeUnit {
    pub stack: Stack,
    pub imports: Vec<ImportPath>,
}
