use serde::{Deserialize, Serialize};

/// Representa una coordenada exacta en el Mandala (Espacio Polar).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PolarCoord {
    pub r: f64,
    pub theta: f64, // Ángulo en grados [0, 360)
}

impl PolarCoord {
    /// Crea una nueva coordenada normalizando el ángulo entre 0 y 360 grados.
    pub fn new(r: f64, theta: f64) -> Self {
        let normalized_theta = ((theta % 360.0) + 360.0) % 360.0;
        Self { r, theta: normalized_theta }
    }

    /// Calcula la distancia espacial "orbital" entre dos coordenadas.
    /// Usamos la fórmula de la distancia en coordenadas polares:
    /// d = sqrt(r1^2 + r2^2 - 2*r1*r2*cos(theta1 - theta2))
    pub fn distance_to(&self, other: &PolarCoord) -> f64 {
        let r1 = self.r;
        let r2 = other.r;
        let d_theta = (self.theta - other.theta).to_radians();
        
        (r1.powi(2) + r2.powi(2) - 2.0 * r1 * r2 * d_theta.cos()).sqrt()
    }
}
