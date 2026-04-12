use crate::ontology::monad::Monad;
use crate::geometry::polar_space::PolarCoord;
use crate::ontology::semantic_hash::generate_pure_hash;

/// Extrae todas las funciones/estructuras de un archivo de texto como Mónadas en bruto.
/// Por ahora simula la extracción buscando palabras clave like 'fn' o 'function'.
pub fn extract_raw_monads(source_code: &str, ring: u32) -> Vec<Monad> {
    let mut monads = Vec::new();
    
    // TODO: Implementar integración real con ast-grep
    // Simulamos identificando bloques por 'fn' o 'function'
    let blocks = source_code.split("\n").collect::<Vec<_>>();
    let mut current_block = String::new();
    let mut current_name = String::new();

    for (i, line) in blocks.iter().enumerate() {
        if line.contains("fn ") || line.contains("function ") {
            if !current_block.is_empty() {
                // Guardar anterior
                let hash = generate_pure_hash(&current_block);
                let coord = PolarCoord::new((ring as f64) * 100.0, (monads.len() as f64) * 45.0);
                monads.push(Monad::spawn(hash, current_name.clone(), coord, current_block.clone(), ring));
            }
            current_block = line.to_string();
            current_name = line.split_whitespace().nth(1).unwrap_or("unknown").to_string();
        } else {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    if !current_block.is_empty() {
        let hash = generate_pure_hash(&current_block);
        let coord = PolarCoord::new((ring as f64) * 100.0, (monads.len() as f64) * 45.0);
        monads.push(Monad::spawn(hash, current_name, coord, current_block, ring));
    }
    
    monads
}
