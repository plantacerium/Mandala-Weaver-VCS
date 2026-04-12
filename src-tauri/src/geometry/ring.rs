use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ring {
    pub level: u32,
    pub radius: f64,
    pub label: String,
}

impl Ring {
    /// Calcula el radio de expansión basado en el radio anterior y un delta de complejidad.
    pub fn calculate_expansion_radius(previous_radius: f64, complexity_delta: f64) -> f64 {
        // Crecimiento logarítmico o lineal basado en la complejidad
        previous_radius + 50.0 + (complexity_delta * 10.0)
    }
}
