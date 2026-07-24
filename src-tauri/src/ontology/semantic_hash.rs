/// Extracts meaningful tokens from source code for AST-aware hashing.
/// Strips comments, whitespace, and string literal contents while preserving
/// structural tokens (identifiers, keywords, operators, delimiters).
fn extract_ast_tokens(content: &str) -> String {
    let mut tokens = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // Skip line comments
        if c == '/' && i + 1 < len && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Skip block comments
        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }

        // Skip hash comments (Rust, Python, etc.)
        if c == '#' && i + 1 < len && chars[i + 1] == '!' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '#' && (i == 0 || chars[i - 1] != '!') {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Skip string literal contents but keep delimiters for structure
        if c == '"' || c == '\'' {
            let quote = c;
            i += 1;
            while i < len && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < len {
                i += 1; // skip closing quote
            }
            tokens.push(quote);
            tokens.push(quote);
            continue;
        }

        // Skip whitespace (collapse multiple into single space)
        if c.is_whitespace() {
            if !tokens.ends_with(' ') && !tokens.is_empty() {
                tokens.push(' ');
            }
            i += 1;
            continue;
        }

        tokens.push(c);
        i += 1;
    }

    // Final trim
    tokens.trim().to_string()
}

/// Generates a semantic hash that is resilient to whitespace and comment changes
/// while being sensitive to structural code differences.
///
/// Uses AST-aware token extraction to strip comments and normalize whitespace
/// before hashing, so the hash only reflects meaningful code structure.
pub fn generate_pure_hash(content: &str) -> String {
    let token_content = extract_ast_tokens(content);
    let hash = blake3::hash(token_content.as_bytes());
    hash.to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_ignores_whitespace_changes() {
        let a = "fn foo() { return 1; }";
        let b = "fn   foo()  {  return 1;  }";
        assert_eq!(generate_pure_hash(a), generate_pure_hash(b));
    }

    #[test]
    fn hash_ignores_comments() {
        let a = "fn foo() { return 1; }";
        let b = "fn foo() { /* block */ return 1; // line\n}";
        assert_eq!(generate_pure_hash(a), generate_pure_hash(b));
    }

    #[test]
    fn hash_detects_semantic_changes() {
        let a = "fn foo() { return 1; }";
        let b = "fn foo() { return 2; }";
        assert_ne!(generate_pure_hash(a), generate_pure_hash(b));
    }
}
