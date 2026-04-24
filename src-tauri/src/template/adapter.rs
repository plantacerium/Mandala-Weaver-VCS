use crate::ontology::monad::Monad;
use crate::template::{AdapterConfig, AdapterType};

pub struct AdapterEngine;

impl AdapterEngine {
    pub fn apply_adapters(monads: &[Monad], adapters: &[AdapterConfig]) -> String {
        let mut source = String::new();

        let imports: Vec<_> = adapters
            .iter()
            .filter_map(|a| {
                if let AdapterType::Import { module } = &a.adapter_type {
                    Some(format!("use {};\n", module))
                } else {
                    None
                }
            })
            .collect();

        let import_count = imports.len();
        for import in imports {
            source.push_str(&import);
        }

        if import_count > 0 {
            source.push('\n');
        }

        let wrappers: Vec<_> = adapters
            .iter()
            .filter(|a| matches!(a.adapter_type, AdapterType::Wrapper { .. }))
            .cloned()
            .collect();

        for monad in monads {
            let mut content = if let Some(alias) = Self::get_alias(adapters, &monad.name) {
                monad.content.replace(&monad.name, &alias)
            } else {
                monad.content.clone()
            };

            if !wrappers.is_empty() {
                let temp_monad = Monad {
                    content,
                    ..monad.clone()
                };
                content = Self::apply_wrappers(&temp_monad, &wrappers);
            }

            source.push_str(&content);
            source.push_str("\n\n");
        }

        source
    }

    pub fn resolve_with_adapters(monads: &[Monad], adapters: &[AdapterConfig]) -> String {
        if adapters.is_empty() {
            let mut source = String::new();
            for monad in monads {
                source.push_str(&monad.content);
                source.push_str("\n\n");
            }
            source
        } else {
            Self::apply_adapters(monads, adapters)
        }
    }

    fn get_alias<'a>(adapters: &'a [AdapterConfig], name: &str) -> Option<String> {
        adapters
            .iter()
            .find(|a| a.from == name)
            .and_then(|a| {
                if let AdapterType::Alias { ref new_name } = a.adapter_type {
                    Some(new_name.clone())
                } else {
                    None
                }
            })
    }

    pub fn apply_wrappers(monad: &Monad, wrappers: &[AdapterConfig]) -> String {
        let mut content = monad.content.clone();

        for adapter in wrappers {
            if let AdapterType::Wrapper { before, after } = &adapter.adapter_type {
                if adapter.from == monad.name || adapter.from == "*" {
                    content = format!("{}{}{}", before, content, after);
                }
            }
        }

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;

    fn make_test_monad(name: &str) -> Monad {
        Monad::spawn(
            format!("hash_{}", name),
            name.to_string(),
            PolarCoord::new(100.0, 45.0),
            format!("fn {}() {{ }}", name),
            1,
        )
    }

    #[test]
    fn test_imports_injected() {
        let monads = vec![make_test_monad("foo")];
        let adapters = vec![AdapterConfig {
            name: "std_import".to_string(),
            from: "foo".to_string(),
            to: "std".to_string(),
            adapter_type: AdapterType::Import {
                module: "std::collections::HashMap".to_string(),
            },
        }];

        let source = AdapterEngine::apply_adapters(&monads, &adapters);
        assert!(source.contains("use std::collections::HashMap"));
    }

    #[test]
    fn test_alias_applied() {
        let monads = vec![make_test_monad("old_name")];
        let adapters = vec![AdapterConfig {
            name: "alias".to_string(),
            from: "old_name".to_string(),
            to: "new_name".to_string(),
            adapter_type: AdapterType::Alias {
                new_name: "new_name".to_string(),
            },
        }];

        let source = AdapterEngine::apply_adapters(&monads, &adapters);
        assert!(source.contains("fn new_name()"));
    }

    #[test]
    fn test_wrapper_applied() {
        let monad = make_test_monad("inner");
        let adapters = vec![AdapterConfig {
            name: "wrap".to_string(),
            from: "inner".to_string(),
            to: "wrapped".to_string(),
            adapter_type: AdapterType::Wrapper {
                before: "pre(".to_string(),
                after: ")".to_string(),
            },
        }];

        let result = AdapterEngine::apply_wrappers(&monad, &adapters);
        assert!(result.starts_with("pre("));
        assert!(result.ends_with(")"));
    }
}