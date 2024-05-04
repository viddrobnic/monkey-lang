use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolScope {
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol {
    pub scope: SymbolScope,
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    store: HashMap<String, Symbol>,
    num_definitions: u16,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let symbol = Symbol {
            scope: SymbolScope::Global,
            index: self.num_definitions,
        };

        self.store.insert(name, symbol);
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.store.get(name)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
