// Pre-implementation: Auto-import generation
// ==============================

use std::collections::HashSet;
use crate::ontology::monad::Monad;

/// Analyzes cross-references and generates import statements
pub struct ImportAnalyzer;

impl ImportAnalyzer {
    /// Extracts all references from monad content
    pub fn extract_references(content: &str) -> Vec<String> {
        let mut refs = Vec::new();
        
        let struct_pat = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9]*)\b").unwrap();
        for cap in struct_pat.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let name_str = name.as_str();
                if !Self::is_keyword(name_str) {
                    refs.push(name_str.to_string());
                }
            }
        }
        
        refs
    }
    
    fn is_keyword(name: &str) -> bool {
        matches!(name, "Self" | "String" | "Vec" | "Option" | "Result" | "Box" | "Rc" | "Arc" | "Cell" | "RefCell" | "Some" | "None" | "Ok" | "Err" | "true" | "false")
    }
    
    /// Generates use statements for a monad
    pub fn generate_imports(monad: &Monad, available: &[Monad]) -> Vec<String> {
        let refs = Self::extract_references(&monad.content);
        let available_names: HashSet<String> = available.iter()
            .map(|m| m.name.clone())
            .collect();
        
        refs.into_iter()
            .filter(|r| available_names.contains(r))
            .map(|r| format!("use crate::{};", Self::to_snake_case(&r)))
            .collect()
    }
    
    /// Convert to snake_case module path
    pub fn to_snake_case(name: &str) -> String {
        let mut result = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        }
        result
    }
    
    /// Generate mod statements for a collection of monads
    pub fn generate_mod_statements(monads: &[Monad]) -> Vec<String> {
        let mut modules: HashSet<String> = monads.iter()
            .map(|m| Self::to_snake_case(&m.name))
            .collect();
        
        let mut statements: Vec<String> = modules.into_iter()
            .map(|m| format!("pub mod {};", m))
            .collect();
        
        statements.sort();
        statements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::Monad;

    #[test]
    fn test_extract_references() {
        let content = "fn process_user(user: User) -> Result<Output, Error>";
        let refs = ImportAnalyzer::extract_references(content);
        assert!(refs.contains(&"User".to_string()));
        assert!(refs.contains(&"Output".to_string()));
        assert!(refs.contains(&"Error".to_string()));
    }
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(ImportAnalyzer::to_snake_case("UserService"), "user_service");
        assert_eq!(ImportAnalyzer::to_snake_case("APIClient"), "a_p_i_client");
    }
}