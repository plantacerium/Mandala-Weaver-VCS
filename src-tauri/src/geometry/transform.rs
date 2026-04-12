use crate::geometry::polar_space::PolarCoord;
use nalgebra::Vector2;

/// Convierte coordenadas polares a cartesianas (x, y) para el renderizado UI.
/// Retorna un Vector2 de nalgebra.
pub fn to_cartesian(coord: &PolarCoord) -> Vector2<f64> {
    let theta_rad = coord.theta.to_radians();
    let x = coord.r * theta_rad.cos();
    let y = coord.r * theta_rad.sin();
    Vector2::new(x, y)
}

/// Convierte cartesianas (x, y) a polares (r, theta).
pub fn from_cartesian(x: f64, y: f64) -> PolarCoord {
    let r = (x.powi(2) + y.powi(2)).sqrt();
    let theta = y.atan2(x).to_degrees();
    PolarCoord::new(r, theta)
}
