use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq)]
pub enum BlockScope {
    If,
    Else,
    Function,
    Global,
}

#[derive(Clone)]
pub struct ScopeTracker(Rc<RefCell<Vec<BlockScope>>>);

impl ScopeTracker {
    pub fn new() -> ScopeTracker {
        ScopeTracker(Rc::new(RefCell::new(vec![BlockScope::Global])))
    }

    pub fn get(&self) -> RefMut<Vec<BlockScope>> {
        self.0.as_ref().borrow_mut()
    }

    pub fn guard(&self, scope: BlockScope) -> ScopeGuard {
        ScopeGuard::new(self.clone(), scope)
    }

    pub fn has_scope(&self, scope: BlockScope) -> bool {
        self.get().contains(&scope)
    }
}

pub struct ScopeGuard {
    stack: ScopeTracker,
}

impl ScopeGuard {
    pub fn new(stack: ScopeTracker, scope: BlockScope) -> ScopeGuard {
        stack.get().push(scope);
        ScopeGuard { stack }
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        self.stack.get().pop();
    }
}
