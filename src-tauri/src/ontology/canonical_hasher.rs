use ast_grep_core::Node;
use ast_grep_core::tree_sitter::StrDoc;
use ast_grep_language::SupportLang;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalHasherError {
    #[error("Estructura sintáctica no válida en el nodo")]
    InvalidStructure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SemanticIdentity {
    /// Hash de la interfaz: nombre + genéricos + tipos de entrada + tipo de salida
    pub contract_hash: String,
    /// Hash del cuerpo lógico normalizado (invariante a formato y comentarios)
    pub implementation_hash: String,
    /// Representación canónica legible del contrato para depuración y SurrealDB
    pub canonical_signature: String,
}

impl SemanticIdentity {
    /// Determina si dos Mónadas pueden intercambiarse directamente en una receta de destilación
    pub fn is_drop_in_compatible(&self, other: &Self) -> bool {
        !self.contract_hash.is_empty() && self.contract_hash == other.contract_hash
    }
}

/// Nodos sintácticos que aportan ruido visual pero cero valor semántico
fn is_trivia_or_comment(kind: &str) -> bool {
    matches!(kind, "line_comment" | "block_comment" | "comment")
}

/// Alimenta recursivamente un blake3::Hasher con la estructura pura del AST
pub fn hash_ast_canonical<D: ast_grep_core::Doc>(node: &Node<D>, hasher: &mut blake3::Hasher) {
    let kind = node.kind();
    if is_trivia_or_comment(&kind) {
        return;
    }

    let children: Vec<_> = node.children().collect();

    if children.is_empty() {
        // Es un nodo hoja (token real: identificador, literal, operador)
        let token_text = node.text();
        let trimmed = token_text.trim();
        if !trimmed.is_empty() {
            hasher.update(b"TOKEN:");
            hasher.update(kind.as_bytes());
            hasher.update(b":");
            hasher.update(trimmed.as_bytes());
            hasher.update(b";");
        }
    } else {
        // Es un nodo compuesto: registramos su apertura y cierre estructural
        hasher.update(b"OPEN:");
        hasher.update(kind.as_bytes());
        hasher.update(b"{");
        for child in children {
            hash_ast_canonical(&child, hasher);
        }
        hasher.update(b"}");
    }
}

fn is_valid_symbol_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split("::").all(|part| {
        !part.is_empty() && part.chars().all(|c| c.is_alphanumeric() || c == '_')
    })
}

pub fn is_common_builtin_symbol(s: &str) -> bool {
    matches!(
        s,
        "Ok" | "Err" | "Some" | "None" | "println" | "eprintln" | "format" | "panic"
        | "assert" | "assert_eq" | "assert_ne" | "todo" | "unimplemented" | "unreachable" | "vec"
        | "clone" | "to_string" | "unwrap" | "unwrap_or" | "unwrap_or_default" | "unwrap_or_else"
        | "expect" | "as_str" | "as_ref" | "into" | "from" | "iter" | "collect" | "len" | "is_empty"
        | "push" | "push_str" | "insert" | "get" | "map" | "and_then" | "filter" | "find"
        | "print" | "range" | "enumerate" | "isinstance" | "str" | "int" | "float" | "list" | "dict" | "set"
        | "log" | "warn" | "error" | "info" | "parseInt" | "parseFloat" | "String" | "Number" | "Boolean" | "Array" | "Object"
        | "make" | "append" | "recover" | "close"
    )
}

pub fn is_rust_prelude_symbol(s: &str) -> bool {
    is_common_builtin_symbol(s)
}

/// Extrae llamadas a otros símbolos dentro de un subárbol AST (Rust, TS/JS, Python, Go, C/C++, Java, C#)
pub fn extract_outbound_calls<D: ast_grep_core::Doc>(body_node: &Node<D>, self_name: &str) -> BTreeSet<String> {
    let mut calls = BTreeSet::new();

    fn record_symbol(sym: &str, self_name: &str, calls: &mut BTreeSet<String>) {
        let trimmed = sym.trim();
        let tail = trimmed
            .rsplit(|c| c == '.' || c == ':')
            .next()
            .unwrap_or(trimmed)
            .trim();
        if is_valid_symbol_identifier(tail)
            && tail != self_name
            && !is_common_builtin_symbol(tail)
        {
            calls.insert(tail.to_string());
        }
    }

    fn walk_calls<D: ast_grep_core::Doc>(node: &Node<D>, self_name: &str, calls: &mut BTreeSet<String>) {
        let k = node.kind();
        let k_ref: &str = &k;
        match k_ref {
            "call_expression" | "call" | "invocation_expression" => {
                if let Some(func_expr) = node.field("function") {
                    let func_kind = func_expr.kind();
                    let fk: &str = &func_kind;
                    if fk == "identifier" || fk == "scoped_identifier" {
                        let text = func_expr.text().to_string();
                        if is_valid_symbol_identifier(text.trim())
                            && text.trim() != self_name
                            && !is_common_builtin_symbol(text.trim())
                        {
                            calls.insert(text.trim().to_string());
                        } else {
                            record_symbol(&text, self_name, calls);
                        }
                    }
                }
            }
            _ => {}
        }
        for child in node.children() {
            walk_calls(&child, self_name, calls);
        }
    }

    walk_calls(body_node, self_name, &mut calls);
    calls
}

/// Analiza un nodo de función de Rust y extrae su identidad semántica canónica
pub fn extract_rust_function_identity(fn_node: &Node<StrDoc<SupportLang>>) -> Result<(SemanticIdentity, BTreeSet<String>), CanonicalHasherError> {
    let name_node = fn_node
        .field("name")
        .ok_or(CanonicalHasherError::InvalidStructure)?;
    let symbol_name = name_node.text().trim().to_string();

    // 1. CONTRACT HASH: nombre + genéricos + tipos de parámetros + retorno
    let mut contract_hasher = blake3::Hasher::new_derive_key("MANDALA_WEAVER_CONTRACT_V1");
    let mut canonical_sig = format!("fn {}", symbol_name);

    contract_hasher.update(b"NAME:");
    contract_hasher.update(symbol_name.as_bytes());

    if let Some(type_params) = fn_node.field("type_parameters") {
        contract_hasher.update(b"GENERICS:");
        hash_ast_canonical(&type_params, &mut contract_hasher);
        canonical_sig.push_str(type_params.text().trim());
    }

    canonical_sig.push('(');
    if let Some(params_node) = fn_node.field("parameters") {
        let mut param_types_str = Vec::new();
        for child in params_node.children() {
            match child.kind().as_ref() {
                "parameter" => {
                    // Solo hasheamos el campo `type` del parámetro, ignorando el nombre local
                    if let Some(ty_node) = child.field("type") {
                        contract_hasher.update(b"PARAM_TYPE:");
                        hash_ast_canonical(&ty_node, &mut contract_hasher);
                        param_types_str.push(ty_node.text().trim().to_string());
                    }
                }
                "self_parameter" => {
                    contract_hasher.update(b"SELF_PARAM:");
                    hash_ast_canonical(&child, &mut contract_hasher);
                    param_types_str.push(child.text().trim().to_string());
                }
                _ => {}
            }
        }
        canonical_sig.push_str(&param_types_str.join(", "));
    }
    canonical_sig.push(')');

    contract_hasher.update(b"RETURN:");
    if let Some(ret_node) = fn_node.field("return_type") {
        hash_ast_canonical(&ret_node, &mut contract_hasher);
        let ret_text = ret_node.text().trim().to_string();
        if ret_text.starts_with("->") {
            canonical_sig.push_str(&format!(" {}", ret_text));
        } else {
            canonical_sig.push_str(&format!(" -> {}", ret_text));
        }
    } else {
        contract_hasher.update(b"UNIT_RETURN");
        canonical_sig.push_str(" -> ()");
    }

    // 2. IMPLEMENTATION HASH: cuerpo de la función normalizado (sin trivia)
    let body_node = fn_node
        .field("body")
        .ok_or(CanonicalHasherError::InvalidStructure)?;

    let mut impl_hasher = blake3::Hasher::new_derive_key("MANDALA_WEAVER_IMPL_V1");
    hash_ast_canonical(&body_node, &mut impl_hasher);

    // 3. Puertos de salida (símbolos invocados dentro del cuerpo)
    let outbound_calls = extract_outbound_calls(&body_node, &symbol_name);

    let identity = SemanticIdentity {
        contract_hash: contract_hasher.finalize().to_hex().to_string(),
        implementation_hash: impl_hasher.finalize().to_hex().to_string(),
        canonical_signature: canonical_sig,
    };

    Ok((identity, outbound_calls))
}

/// Extractor canónico políglota para funciones, métodos, clases y contratos en cualquier lenguaje
pub fn extract_generic_identity<D: ast_grep_core::Doc>(node: &Node<D>, symbol_name: &str) -> (SemanticIdentity, BTreeSet<String>) {
    let mut impl_hasher = blake3::Hasher::new_derive_key("MANDALA_WEAVER_IMPL_V1");
    if let Some(body) = node.field("body") {
        hash_ast_canonical(&body, &mut impl_hasher);
    } else {
        hash_ast_canonical(node, &mut impl_hasher);
    }
    let impl_hash = impl_hasher.finalize().to_hex().to_string();

    let mut contract_hasher = blake3::Hasher::new_derive_key("MANDALA_WEAVER_CONTRACT_V1");
    contract_hasher.update(b"NAME:");
    contract_hasher.update(symbol_name.as_bytes());

    let mut canonical_sig = if let Some(params) = node.field("parameters") {
        contract_hasher.update(b"PARAMS:");
        hash_ast_canonical(&params, &mut contract_hasher);
        let p_text = params.text().trim().to_string();
        if p_text.starts_with('(') {
            format!("fn {}{}", symbol_name, p_text)
        } else {
            format!("fn {}({})", symbol_name, p_text)
        }
    } else {
        format!("sym {}", symbol_name)
    };

    if let Some(ret) = node.field("return_type").or_else(|| node.field("result")) {
        contract_hasher.update(b"RETURN:");
        hash_ast_canonical(&ret, &mut contract_hasher);
        let r_text = ret.text().trim().trim_start_matches(':').trim_start_matches("->").trim().to_string();
        if !r_text.is_empty() {
            canonical_sig.push_str(&format!(" -> {}", r_text));
        }
    }

    let contract_hash = contract_hasher.finalize().to_hex().to_string();
    let outbound_calls = extract_outbound_calls(node, symbol_name);

    let identity = SemanticIdentity {
        contract_hash,
        implementation_hash: impl_hash,
        canonical_signature: canonical_sig,
    };

    (identity, outbound_calls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast_grep_core::AstGrep;

    #[test]
    fn test_canonical_contract_and_impl_invariance() {
        let r_lang = SupportLang::Rust;

        let code_ring0 = r#"
            /// Calcula la tensión radial del nodo
            pub fn calcular_tension( masa: f64, radio: f64 ) -> f64 {
                // Comentario que debe ser ignorado por blake3
                let factor = normalizar(radio);
                masa * factor
            }
        "#;

        let code_ring1 = r#"
            pub fn calcular_tension(masa: f64, radio: f64) -> f64 {
                let factor = normalizar(radio); /* bloque comentado */
                masa * factor
            }
        "#;

        let code_ring2 = r#"
            pub fn calcular_tension(m_kg: f64, r_metros: f64) -> f64 {
                let factor = normalizar_rapido(r_metros);
                (m_kg * factor).powi(2)
            }
        "#;

        let grep0 = AstGrep::new(code_ring0, r_lang);
        let grep1 = AstGrep::new(code_ring1, r_lang);
        let grep2 = AstGrep::new(code_ring2, r_lang);

        let fn0 = grep0.root().children().find(|n| n.kind() == "function_item").unwrap();
        let fn1 = grep1.root().children().find(|n| n.kind() == "function_item").unwrap();
        let fn2 = grep2.root().children().find(|n| n.kind() == "function_item").unwrap();

        let (id0, calls0) = extract_rust_function_identity(&fn0).unwrap();
        let (id1, _calls1) = extract_rust_function_identity(&fn1).unwrap();
        let (id2, calls2) = extract_rust_function_identity(&fn2).unwrap();

        // 1. Invarianza total entre ring 0 y ring 1 ante formato y comentarios
        assert_eq!(id0.contract_hash, id1.contract_hash);
        assert_eq!(id0.implementation_hash, id1.implementation_hash);

        // 2. ring 0 y ring 2 tienen distintos nombres de parámetros pero MISMO contrato
        assert_eq!(id0.contract_hash, id2.contract_hash);
        assert!(id0.is_drop_in_compatible(&id2));
        assert_eq!(id0.canonical_signature, "fn calcular_tension(f64, f64) -> f64");

        // 3. Su implementación es distinta porque el cuerpo cambió
        assert_ne!(id0.implementation_hash, id2.implementation_hash);

        // 4. Se detectan las llamadas salientes hacia otros contratos
        assert!(calls0.contains("normalizar"));
        assert!(calls2.contains("normalizar_rapido"));
    }
}
