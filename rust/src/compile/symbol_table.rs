use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolScope {
    Global,
    Local,
    Free,
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
    free_symbols: Vec<Symbol>,
    num_definitions: u16,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            outer: None,
            store: HashMap::new(),
            free_symbols: vec![],
            num_definitions: 0,
        }
    }

    /// Encloses the current symbol table.
    pub fn enclose(&mut self) {
        let mut outer = Self::new();
        std::mem::swap(&mut outer, self);
        self.outer = Some(Box::new(outer));
    }

    /// Leaves the enclosure. Return free symbols.
    pub fn leave(&mut self) -> Vec<Symbol> {
        if let Some(outer) = self.outer.take() {
            let mut outer = *outer;
            std::mem::swap(&mut outer, self);

            outer.free_symbols
        } else {
            vec![]
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

    fn define_free(&mut self, original: Symbol, name: String) -> Symbol {
        self.free_symbols.push(original);

        let symbol = Symbol {
            scope: SymbolScope::Free,
            index: self.free_symbols.len() as u16 - 1,
        };

        self.store.insert(name, symbol);
        symbol
    }

    pub fn resolve(&mut self, name: &str) -> Option<Symbol> {
        if let Some(sym) = self.store.get(name) {
            return Some(*sym);
        }

        match &mut self.outer {
            Some(outer) => {
                let symbol = outer.resolve(name)?;

                if symbol.scope == SymbolScope::Global {
                    return Some(symbol);
                }

                let symbol = self.define_free(symbol, name.to_string());
                Some(symbol)
            }
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

#[cfg(test)]
mod test {
    use crate::compile::symbol_table::{Symbol, SymbolScope};

    use super::SymbolTable;

    #[test]
    fn resolve_free() {
        let mut table = SymbolTable::new();
        table.define("a".to_string());
        table.define("b".to_string());

        // First local
        table.enclose();
        table.define("c".to_string());
        table.define("d".to_string());
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Global,
                index: 0,
            }),
            table.resolve("a")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Global,
                index: 1,
            }),
            table.resolve("b")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Local,
                index: 0,
            }),
            table.resolve("c")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Local,
                index: 1,
            }),
            table.resolve("d")
        );
        assert_eq!(table.free_symbols, []);

        // Second local
        table.enclose();
        table.define("e".to_string());
        table.define("f".to_string());
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Global,
                index: 0,
            }),
            table.resolve("a")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Global,
                index: 1,
            }),
            table.resolve("b")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Free,
                index: 0,
            }),
            table.resolve("c")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Free,
                index: 1,
            }),
            table.resolve("d")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Local,
                index: 0,
            }),
            table.resolve("e")
        );
        assert_eq!(
            Some(Symbol {
                scope: SymbolScope::Local,
                index: 1,
            }),
            table.resolve("f")
        );
        assert_eq!(
            vec![
                Symbol {
                    scope: SymbolScope::Local,
                    index: 0,
                },
                Symbol {
                    scope: SymbolScope::Local,
                    index: 1,
                },
            ],
            table.free_symbols
        )
    }
}
