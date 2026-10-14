use nalgebra::{UnitComplex, Vector2};
use serde::{Deserialize, Serialize};
use crate::geometry::polar_space::PolarCoord;

/// Nivel discreto del anillo radial (0 = Núcleo/Contrato, 1 = Estable, 2 = Mutación/Experimental, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct RingLevel(pub u32);

/// Coordenada polar continua en el grupo de rotación SO(2) mediante UnitComplex.
/// Evita discontinuidades en 0 <-> 2π y elimina el gimbal lock modular.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PolarCoordinate {
    pub ring: RingLevel,
    pub radius_offset: f64,
    /// Vector unitario direccional en el plano complejo: re = cos(θ), im = sin(θ)
    pub sector: UnitComplex<f64>,
}

impl Default for PolarCoordinate {
    fn default() -> Self {
        Self {
            ring: RingLevel(0),
            radius_offset: 0.0,
            sector: UnitComplex::new(0.0),
        }
    }
}

impl PolarCoordinate {
    pub fn new(ring: u32, radius_offset: f64, angle_rad: f64) -> Self {
        Self {
            ring: RingLevel(ring),
            radius_offset: radius_offset.max(0.0),
            sector: UnitComplex::new(angle_rad),
        }
    }

    /// Distancia angular mínima (geodésica) entre dos Mónadas en el rango [0, π]
    pub fn angular_distance(&self, other: &Self) -> f64 {
        self.sector.angle_to(&other.sector).abs()
    }

    /// Proyección a espacio euclídeo cartesiano 2D para renderizado o indexación espacial (R-Tree)
    pub fn to_cartesian(&self, ring_spacing: f64) -> Vector2<f64> {
        let effective_r = (self.ring.0 as f64 * ring_spacing) + self.radius_offset;
        self.sector.transform_vector(&Vector2::new(effective_r, 0.0))
    }

    /// Ángulo en radianes normalizado en [0, 2π)
    pub fn angle_radians(&self) -> f64 {
        let mut a = self.sector.angle();
        if a < 0.0 {
            a += 2.0 * std::f64::consts::PI;
        }
        a
    }

    /// Ángulo en grados normalizado en [0, 360)
    pub fn angle_degrees(&self) -> f64 {
        self.angle_radians().to_degrees()
    }
}

impl From<&PolarCoord> for PolarCoordinate {
    fn from(pc: &PolarCoord) -> Self {
        let ring = (pc.r / 100.0).floor() as u32;
        let radius_offset = pc.r - (ring as f64 * 100.0);
        let angle_rad = pc.theta.to_radians();
        Self::new(ring, radius_offset, angle_rad)
    }
}

impl From<&PolarCoordinate> for PolarCoord {
    fn from(pc: &PolarCoordinate) -> Self {
        let effective_r = (pc.ring.0 as f64 * 100.0) + pc.radius_offset;
        let deg = pc.angle_degrees();
        PolarCoord::new(effective_r, deg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_so2_continuity_and_geodesic_distance() {
        // Puntos muy cercanos a través de la frontera 0 <-> 2π
        // coord1 a +5° (0.087 rad), coord2 a 355° (-5° / -0.087 rad)
        let c1 = PolarCoordinate::new(1, 0.0, 5.0_f64.to_radians());
        let c2 = PolarCoordinate::new(1, 0.0, 355.0_f64.to_radians());

        let dist = c1.angular_distance(&c2);
        let dist_deg = dist.to_degrees();

        // En punto flotante escalar convencional, |5 - 355| = 350° (discontinuidad).
        // En SO(2) con UnitComplex, la distancia geodésica es exactamente 10°.
        assert!((dist_deg - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_cartesian_projection() {
        let c = PolarCoordinate::new(1, 20.0, PI / 2.0); // 90 grados, radio 120.0
        let cart = c.to_cartesian(100.0);

        assert!((cart.x).abs() < 1e-6);
        assert!((cart.y - 120.0).abs() < 1e-6);
    }

    #[test]
    fn test_bidirectional_conversion_with_polar_coord() {
        let original = PolarCoord::new(250.0, 180.0);
        let so2 = PolarCoordinate::from(&original);
        let back = PolarCoord::from(&so2);

        assert_eq!(so2.ring.0, 2);
        assert_eq!(so2.radius_offset, 50.0);
        assert!((back.r - original.r).abs() < 1e-6);
        assert!((back.theta - original.theta).abs() < 1e-6);
    }
}
