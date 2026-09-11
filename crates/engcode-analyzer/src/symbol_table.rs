use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Database,
    Collection,
    Variable,
    Function,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub scope_level: usize,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    current_scope: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
            current_scope: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope += 1;
    }

    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }

    pub fn define(&mut self, name: String, symbol_type: SymbolType) {
        let symbol = Symbol {
            name: name.clone(),
            symbol_type,
            scope_level: self.current_scope,
        };
        self.scopes[self.current_scope].insert(name, symbol);
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        // Look from current scope up to global scope
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn exists(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_resolve() {
        let mut table = SymbolTable::new();
        table.define("MyDB".to_string(), SymbolType::Database);

        assert!(table.exists("MyDB"));
        let symbol = table.resolve("MyDB").unwrap();
        assert_eq!(symbol.symbol_type, SymbolType::Database);
    }

    #[test]
    fn test_scope_resolution() {
        let mut table = SymbolTable::new();
        table.define("global".to_string(), SymbolType::Variable);

        table.enter_scope();
        table.define("local".to_string(), SymbolType::Variable);

        assert!(table.exists("global"));
        assert!(table.exists("local"));

        table.exit_scope();
        assert!(table.exists("global"));
        assert!(!table.exists("local"));
    }
}
