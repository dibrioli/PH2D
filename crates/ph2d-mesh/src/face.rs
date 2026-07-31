//! A face — triângulo **ou** quadrilátero, num slot de tamanho fixo.
//!
//! Adaptado de `reference/sculptgl/src/mesh/MeshData.js` (`_facesABCD`) e
//! `Mesh.js` (`Utils.TRI_INDEX`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! ⚠️ **Por que quad e não só triângulo.** A GPU rasteriza triângulos, então a
//! tentação é guardar só triângulos. Mas a multiresolução planejada
//! (`docs/3D/04.3`) é **Catmull-Clark**, que produz quads a partir do nível 1 —
//! um núcleo só-triângulo obrigaria a triangular a cada nível e jogaria fora
//! exatamente a propriedade pela qual se escolhe Catmull-Clark. O SculptGL, que
//! tem multires, tomou a mesma decisão pelo mesmo motivo.
//!
//! O preço é 4 índices por face onde 3 bastariam para uma malha de triângulos —
//! **33% do buffer de índices**, e é o que a sonda `measure_memory` mede em vez
//! de a gente estimar.

/// O 4º índice de um **triângulo**. Um `u32` real nunca o alcança: seria uma
/// malha de 4 bilhões de vértices, ordens de grandeza acima do que a memória
/// do dispositivo permite (a sonda `measure_memory` dá o teto verdadeiro).
pub const TRI: u32 = u32::MAX;

/// Uma face da malha. `[a, b, c, TRI]` é um triângulo; `[a, b, c, d]` um quad.
///
/// `repr(transparent)` para que `&[Face]` seja `&[[u32; 4]]` ao bit — é assim
/// que ela sobe para o device sem uma conversão que poderia divergir.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face(pub [u32; 4]);

impl Face {
    #[must_use]
    pub const fn tri(a: u32, b: u32, c: u32) -> Self {
        Self([a, b, c, TRI])
    }

    #[must_use]
    pub const fn quad(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self([a, b, c, d])
    }

    #[must_use]
    pub const fn is_tri(&self) -> bool {
        self.0[3] == TRI
    }

    /// 3 para triângulo, 4 para quad.
    #[must_use]
    pub const fn vert_count(&self) -> usize {
        if self.is_tri() { 3 } else { 4 }
    }

    /// Os vértices, **sem** o sentinela. Este é o único jeito de percorrer uma
    /// face: quem indexa `f.0[3]` direto lê `TRI` num triângulo e escreve numa
    /// posição que não existe.
    #[must_use]
    pub fn verts(&self) -> &[u32] {
        &self.0[..self.vert_count()]
    }

    /// Quantos triângulos esta face vira ao rasterizar (1 ou 2).
    #[must_use]
    pub const fn tri_count(&self) -> usize {
        if self.is_tri() { 1 } else { 2 }
    }

    /// A face como triângulos, para o buffer de índices que a GPU consome.
    /// Um quad `(a,b,c,d)` vira `(a,b,c)` e `(a,c,d)` — a diagonal `a-c`, a
    /// mesma escolha do SculptGL, para que o desenho não dependa de qual
    /// consumidor triangulou.
    pub fn triangles(&self, out: &mut Vec<[u32; 3]>) {
        let [a, b, c, d] = self.0;
        out.push([a, b, c]);
        if d != TRI {
            out.push([a, c, d]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_triangle_never_exposes_the_sentinel() {
        let f = Face::tri(7, 8, 9);
        assert!(f.is_tri());
        assert_eq!(f.verts(), &[7, 8, 9]);
        assert_eq!(f.vert_count(), 3);
        assert_eq!(f.tri_count(), 1);
    }

    #[test]
    fn a_quad_is_two_triangles_sharing_the_a_c_diagonal() {
        let f = Face::quad(0, 1, 2, 3);
        assert!(!f.is_tri());
        assert_eq!(f.verts(), &[0, 1, 2, 3]);
        let mut tris = Vec::new();
        f.triangles(&mut tris);
        assert_eq!(tris, vec![[0, 1, 2], [0, 2, 3]]);
    }

    #[test]
    fn a_face_is_four_u32_to_the_bit_so_it_uploads_without_a_conversion() {
        assert_eq!(size_of::<Face>(), size_of::<[u32; 4]>());
        assert_eq!(align_of::<Face>(), align_of::<[u32; 4]>());
    }
}
