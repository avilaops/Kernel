use crate::vec3::Vec3;

/// Axis-Aligned Bounding Box (AABB)
/// Caixa delimitadora alinhada aos eixos, útil para detecção de colisão
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Cria um AABB vazio (invertido) que pode ser expandido
    pub const EMPTY: Aabb = Aabb {
        min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    /// Cria um AABB a partir de pontos mínimo e máximo
    #[inline]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Cria um AABB a partir de um ponto central e tamanho
    #[inline]
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Cria um AABB que engloba todos os pontos fornecidos
    #[inline]
    pub fn from_points(points: &[Vec3]) -> Self {
        let mut aabb = Self::EMPTY;
        for &point in points {
            aabb = aabb.expand_to_include_point(point);
        }
        aabb
    }

    /// Retorna o centro do AABB
    #[inline]
    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Retorna o tamanho (dimensões) do AABB
    #[inline]
    pub fn size(self) -> Vec3 {
        self.max - self.min
    }

    /// Retorna a semi-extensão (metade do tamanho)
    #[inline]
    pub fn half_extents(self) -> Vec3 {
        self.size() * 0.5
    }

    /// Retorna o volume do AABB
    #[inline]
    pub fn volume(self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Retorna a área da superfície do AABB
    #[inline]
    pub fn surface_area(self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.y * size.z + size.z * size.x)
    }

    /// Verifica se o AABB contém um ponto
    #[inline]
    pub fn contains_point(self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Verifica se este AABB contém completamente outro AABB
    #[inline]
    pub fn contains_aabb(self, other: Aabb) -> bool {
        self.min.x <= other.min.x
            && self.max.x >= other.max.x
            && self.min.y <= other.min.y
            && self.max.y >= other.max.y
            && self.min.z <= other.min.z
            && self.max.z >= other.max.z
    }

    /// Verifica se este AABB intersecta com outro AABB
    #[inline]
    pub fn intersects(self, other: Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Retorna a interseção entre dois AABBs, ou None se não houver interseção
    #[inline]
    pub fn intersection(self, other: Aabb) -> Option<Aabb> {
        if !self.intersects(other) {
            return None;
        }

        Some(Aabb {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        })
    }

    /// Expande o AABB para incluir um ponto
    #[inline]
    pub fn expand_to_include_point(self, point: Vec3) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    /// Expande o AABB para incluir outro AABB
    #[inline]
    pub fn expand_to_include_aabb(self, other: Aabb) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Expande o AABB por uma quantidade uniforme em todas as direções
    #[inline]
    pub fn expand(self, amount: f32) -> Self {
        let expansion = Vec3::splat(amount);
        Self {
            min: self.min - expansion,
            max: self.max + expansion,
        }
    }

    /// Expande o AABB por quantidades diferentes em cada direção
    #[inline]
    pub fn expand_by_vec(self, amount: Vec3) -> Self {
        Self {
            min: self.min - amount,
            max: self.max + amount,
        }
    }

    /// Retorna o ponto mais próximo dentro do AABB a um ponto dado
    #[inline]
    pub fn closest_point(self, point: Vec3) -> Vec3 {
        point.clamp(self.min, self.max)
    }

    /// Retorna a distância ao quadrado de um ponto ao AABB
    #[inline]
    pub fn distance_squared_to_point(self, point: Vec3) -> f32 {
        let closest = self.closest_point(point);
        point.distance_squared(closest)
    }

    /// Retorna a distância de um ponto ao AABB
    #[inline]
    pub fn distance_to_point(self, point: Vec3) -> f32 {
        self.distance_squared_to_point(point).sqrt()
    }

    /// Testa interseção com um raio
    /// Retorna (t_min, t_max) se houver interseção, onde t são os parâmetros do raio
    #[inline]
    pub fn intersect_ray(self, ray_origin: Vec3, ray_dir: Vec3) -> Option<(f32, f32)> {
        let inv_dir = Vec3::new(1.0 / ray_dir.x, 1.0 / ray_dir.y, 1.0 / ray_dir.z);

        let t1 = (self.min - ray_origin) * inv_dir;
        let t2 = (self.max - ray_origin) * inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_enter = t_min.x.max(t_min.y).max(t_min.z);
        let t_exit = t_max.x.min(t_max.y).min(t_max.z);

        if t_enter <= t_exit && t_exit >= 0.0 {
            Some((t_enter.max(0.0), t_exit))
        } else {
            None
        }
    }

    /// Retorna os 8 vértices do AABB
    #[inline]
    pub fn vertices(self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// Verifica se o AABB é válido (min <= max)
    #[inline]
    pub fn is_valid(self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }

    /// Verifica se o AABB está vazio (volume zero ou negativo)
    #[inline]
    pub fn is_empty(self) -> bool {
        !self.is_valid() || self.volume() <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_center() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        let center = aabb.center();
        assert_eq!(center, Vec3::ONE);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!aabb.contains_point(Vec3::new(1.5, 0.5, 0.5)));
    }

    #[test]
    fn test_aabb_intersects() {
        let aabb1 = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let aabb2 = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        let aabb3 = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));

        assert!(aabb1.intersects(aabb2));
        assert!(!aabb1.intersects(aabb3));
    }

    #[test]
    fn test_aabb_expand() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let expanded = aabb.expand(0.5);
        assert_eq!(expanded.min, Vec3::new(-0.5, -0.5, -0.5));
        assert_eq!(expanded.max, Vec3::new(1.5, 1.5, 1.5));
    }

    #[test]
    fn test_aabb_from_points() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(-1.0, 1.0, 2.0),
        ];
        let aabb = Aabb::from_points(&points);
        assert_eq!(aabb.min, Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_ray_intersection() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let ray_origin = Vec3::new(-1.0, 0.5, 0.5);
        let ray_dir = Vec3::new(1.0, 0.0, 0.0);

        let result = aabb.intersect_ray(ray_origin, ray_dir);
        assert!(result.is_some());

        let (t_min, t_max) = result.unwrap();
        assert!(t_min >= 0.0 && t_max >= t_min);
    }
}
