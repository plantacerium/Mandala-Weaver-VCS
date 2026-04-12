use crate::geometry::polar_space::PolarCoord;

/// Detecta si dos coordenadas están demasiado cerca (umbral de colisión).
pub fn detect_overlap(a: &PolarCoord, b: &PolarCoord, threshold: f64) -> bool {
    a.distance_to(b) < threshold
}

/// Desplaza ligeramente una coordenada en su órbita para evitar superposición.
/// Incrementa theta lo suficiente para superar el umbral.
pub fn resolve_orbital_shift(coord: &mut PolarCoord, shift_degrees: f64) {
    coord.theta += shift_degrees;
    // Normalizar
    coord.theta = ((coord.theta % 360.0) + 360.0) % 360.0;
}
