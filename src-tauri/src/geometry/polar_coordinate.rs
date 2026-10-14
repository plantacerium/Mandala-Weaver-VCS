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

/// Proporción áurea conjugada (1 / φ = φ - 1 = (√5 - 1) / 2)
pub const PHI_CONJUGATE: f64 = 0.618_033_988_749_895;

/// Calcula el ancho de banda radial adaptativo en función del radio base del anillo,
/// el espacio entre anillos (ring_gap) y la cantidad de mónadas en dicho anillo.
/// 
/// Considera que a mayor tamaño de anillo (R), la circunferencia es mayor pero
/// los anillos exteriores en proyectos grandes concentran la mayoría del código,
/// por lo que modula la banda sin invadir los corredores entre anillos adyacentes.
pub fn compute_adaptive_band_width(
    ring_radius: f64,
    ring_gap: f64,
    total_in_ring: usize,
) -> f64 {
    if total_in_ring <= 36 {
        return 0.0;
    }

    let perimeter = 2.0 * std::f64::consts::PI * ring_radius;
    let linear_density = total_in_ring as f64 / perimeter.max(1.0); // nodos por pixel

    // Densidad ideal: ~0.07 nodos/pixel (al menos 14px entre centros de nodos)
    let congestion_factor = (linear_density / 0.07).max(1.0);

    // Ancho deseado proporcional a la congestión
    let desired_band = (congestion_factor - 1.0) * 18.0;

    // Cota de seguridad: no más del 55% del espacio entre anillos (para evitar colisión con
    // anillos adyacentes) ni más del 28% del propio radio del anillo (protege anillos internos).
    let max_safe_band = (ring_gap * 0.55).min(ring_radius * 0.28);

    desired_band.min(max_safe_band).max(0.0)
}

/// Distribuye un nodo en una banda anular continua [-band_width/2, +band_width/2]
/// utilizando la sucesión cuasi-aleatoria de baja discrepancia de Weyl (Proporción Áurea).
///
/// Dos nodos contiguos en el perímetro angular (índices i e i+1) saltan a alturas radiales
/// opuestas, transformando colisiones lineales 1D en una dispersión 2D suave y continua.
pub fn compute_jittered_radius_offset(index_in_ring: usize, band_width: f64) -> f64 {
    if band_width <= 0.0 {
        return 0.0;
    }
    let normalized = ((index_in_ring as f64) * PHI_CONJUGATE).fract();
    (normalized - 0.5) * band_width
}

impl PolarCoordinate {
    pub fn new(ring: u32, radius_offset: f64, angle_rad: f64) -> Self {
        Self {
            ring: RingLevel(ring),
            radius_offset,
            sector: UnitComplex::new(angle_rad),
        }
    }

    /// Distancia angular mínima (geodésica) entre dos Mónadas en el rango [0, π]
    pub fn angular_distance(&self, other: &Self) -> f64 {
        self.sector.angle_to(&other.sector).abs()
    }

    /// Proyección a espacio euclídeo cartesiano 2D para renderizado o indexación espacial (R-Tree)
    pub fn to_cartesian(&self, ring_spacing: f64) -> Vector2<f64> {
        let effective_r = ((self.ring.0 as f64 * ring_spacing) + self.radius_offset).max(0.0);
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

    #[test]
    fn test_adaptive_band_width_scaling() {
        let ring_gap = 70.0;
        // Anillos con baja densidad (<= 36 nodos) mantienen banda 0.0
        let band_small = compute_adaptive_band_width(190.0, ring_gap, 22);
        assert_eq!(band_small, 0.0);

        // Anillo exterior de alta densidad (R=260, N=313)
        let band_crowded = compute_adaptive_band_width(260.0, ring_gap, 313);
        assert!(band_crowded > 15.0);
        // Debe respetar la cota de seguridad máxima (<= 55% del ring_gap = 38.5px)
        assert!(band_crowded <= ring_gap * 0.55);
    }

    #[test]
    fn test_weyl_golden_ratio_dispersion() {
        let band_width = 30.0;
        let o0 = compute_jittered_radius_offset(0, band_width);
        let o1 = compute_jittered_radius_offset(1, band_width);
        let o2 = compute_jittered_radius_offset(2, band_width);

        // Todos deben estar estrictamente acotados en [-band_width/2, +band_width/2]
        assert!(o0 >= -15.0 && o0 <= 15.0);
        assert!(o1 >= -15.0 && o1 <= 15.0);
        assert!(o2 >= -15.0 && o2 <= 15.0);

        // Puntos angulares contiguos (0 y 1) deben saltar radialmente para no chocar
        let diff = (o1 - o0).abs();
        assert!(diff > 10.0, "Los nodos contiguos deben separarse radialmente");
    }
}
