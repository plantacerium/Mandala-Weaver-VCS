use crate::geometry::polar_space::PolarCoord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monad {
    pub id: String,        // Hash semántico
    pub coord: PolarCoord, // Ubicación radial
    pub content: String,   // Código fuente o metadatos
    pub name: String,      // Nombre de la función/entidad
    pub ring: u32,         // Nivel de expansión
}

impl Monad {
    pub fn spawn(id: String, name: String, coord: PolarCoord, content: String, ring: u32) -> Self {
        Self {
            id,
            coord,
            content,
            name,
            ring,
        }
    }
}
