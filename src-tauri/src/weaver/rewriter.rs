use ast_grep_core::{AstGrep, Doc, Node};
use ast_grep_language::SupportLang;

/// Reescritor higiénico de AST con ast-grep para inyección de mónadas puente
pub struct AstSynthesizer;

impl AstSynthesizer {
    /// Renombra el identificador de declaración de la función (ej. `normalizar` -> `normalizar__bridge_a1b2c3d4`)
    pub fn rename_function_declaration(source: &str, new_fn_name: &str) -> String {
        let grep = AstGrep::new(source, SupportLang::Rust);
        let root = grep.root();

        fn find_fn_name_range<D: Doc>(node: &Node<D>) -> Option<std::ops::Range<usize>> {
            if node.kind() == "function_item" {
                if let Some(name_node) = node.field("name") {
                    let r = name_node.range();
                    return Some(r.start..r.end);
                }
            }
            for child in node.children() {
                if let Some(r) = find_fn_name_range(&child) {
                    return Some(r);
                }
            }
            None
        }

        if let Some(range) = find_fn_name_range(&root) {
            let mut mutated = source.to_string();
            mutated.replace_range(range, new_fn_name);
            return mutated;
        }

        source.to_string()
    }

    /// Reescribe todas las llamadas `target_callee(...)` dentro del cuerpo de un invocador
    /// para que apunten a `aliased_bridge_name(...)` sin tocar variables ni comentarios.
    pub fn redirect_call_sites(
        caller_source: &str,
        target_callee: &str,
        aliased_bridge_name: &str,
    ) -> String {
        let grep = AstGrep::new(caller_source, SupportLang::Rust);
        let root = grep.root();
        let mut byte_ranges = Vec::new();

        fn collect_call_ranges<D: Doc>(
            node: &Node<D>,
            target_callee: &str,
            ranges: &mut Vec<std::ops::Range<usize>>,
        ) {
            if node.kind() == "call_expression" {
                if let Some(func_expr) = node.field("function") {
                    if func_expr.text().trim() == target_callee {
                        let r = func_expr.range();
                        ranges.push(r.start..r.end);
                    }
                }
            }
            for child in node.children() {
                collect_call_ranges(&child, target_callee, ranges);
            }
        }

        collect_call_ranges(&root, target_callee, &mut byte_ranges);

        // Ordenamos de mayor a menor offset para reemplazar sin desplazar los rangos anteriores
        byte_ranges.sort_by(|a, b| b.start.cmp(&a.start));

        let mut mutated = caller_source.to_string();
        for range in byte_ranges {
            mutated.replace_range(range, aliased_bridge_name);
        }

        mutated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_function_declaration() {
        let src = "pub fn normalizar_adapter(r: f64) -> f64 { r }";
        let renamed = AstSynthesizer::rename_function_declaration(src, "normalizar__bridge_12345678");
        assert_eq!(renamed, "pub fn normalizar__bridge_12345678(r: f64) -> f64 { r }");
    }

    #[test]
    fn test_redirect_call_sites_ignores_comments_and_strings() {
        let src = r#"
            pub fn calcular(x: f64) -> f64 {
                // comentario con normalizar(x) no debe cambiar
                let s = "normalizar(x) en string";
                let val = normalizar(x) + normalizar(x * 2.0);
                val
            }
        "#;
        let redirected = AstSynthesizer::redirect_call_sites(src, "normalizar", "normalizar__bridge_a1b2");
        assert!(redirected.contains("// comentario con normalizar(x) no debe cambiar"));
        assert!(redirected.contains("\"normalizar(x) en string\""));
        assert!(redirected.contains("normalizar__bridge_a1b2(x)"));
        assert!(redirected.contains("normalizar__bridge_a1b2(x * 2.0)"));
    }
}
