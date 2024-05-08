use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolScope {
    Global,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol {
    pub scope: SymbolScope,
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    outer: Option<Box<SymbolTable>>,
    store: HashMap<String, Symbol>,
    num_definitions: u16,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    /// Encloses the current symbol table.
    pub fn enclose(&mut self) {
        let mut outer = Self::new();
        std::mem::swap(&mut outer, self);
        self.outer = Some(Box::new(outer));
    }

    /// Leaves the enclosure.
    pub fn leave(&mut self) {
        if let Some(outer) = self.outer.take() {
            let mut outer = *outer;
            std::mem::swap(&mut outer, self);
        }
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let scope = match self.outer {
            None => SymbolScope::Global,
            Some(_) => SymbolScope::Local,
        };

        let symbol = Symbol {
            scope,
            index: self.num_definitions,
        };

        self.store.insert(name, symbol);
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        if let Some(sym) = self.store.get(name) {
            return Some(sym);
        }

        match &self.outer {
            Some(outer) => outer.resolve(name),
            None => None,
        }
    }

    pub fn num_definitions(&self) -> usize {
        self.num_definitions as usize
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
