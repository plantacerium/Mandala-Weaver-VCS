use serde::{Deserialize, Serialize};
use crate::ontology::monad::Monad;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constellation {
    pub ring_level: u32,
    pub monads: Vec<Monad>,
}

impl Constellation {
    pub fn new(ring_level: u32) -> Self {
        Self {
            ring_level,
            monads: Vec::new(),
        }
    }

    pub fn validate_harmony(&self) -> Result<(), String> {
        // En un futuro, aquí se validará que no haya colisiones no resueltas
        // o incoherencias sintácticas en el anillo.
        Ok(())
    }
}
