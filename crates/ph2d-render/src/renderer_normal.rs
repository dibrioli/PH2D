//! A **fachada das normais** (P14): os métodos do `SpriteRenderer` que registam, leem e localizam no
//! mundo os NORMAL MAPS das texturas individuais e das células do átlas — irmão de `renderer.rs` por
//! tecto de LOC.
//!
//! Corte mecânico: cada método saiu inteiro, verbatim, do `impl SpriteRenderer` de lá; nenhum
//! chamador muda de endereço (são métodos).

use super::*;

impl SpriteRenderer {
    /// ⭐ P14 — regista (`Some((w, h, rgba))`) ou tira (`None`) o NORMAL MAP da textura individual
    /// `texture_id` (rgb = n·½+½, Y para cima). Tem de ter o tamanho da cor: o mesmo `atlas_uv` lê
    /// então a mesma célula / o mesmo quadro nas duas. `Err` para outro tamanho ou id desconhecido.
    pub fn definir_normal(
        &mut self,
        texture_id: u32,
        normal: Option<(u32, u32, &[u8])>,
    ) -> Result<(), String> {
        let cor = self
            .individual
            .entries
            .get(&texture_id)
            .map(|e| (&e.texture, (e.width, e.height)));
        self.normais
            .definir(&self.gpu, &self.pipeline, cor, texture_id, normal)
    }

    /// ⭐ P14 — o NORMAL MAP de uma célula `key` do átlas (o Sprite do motor gravado vive no átlas):
    /// uma textura irmã do átlas recebe-o no MESMO retângulo da célula. Tamanho = o da célula.
    pub fn definir_normal_da_celula(
        &mut self,
        key: u32,
        normal: Option<(u32, u32, &[u8])>,
    ) -> Result<(), String> {
        self.normais
            .definir_celula(&self.gpu, &self.pipeline, &self.atlas, key, normal)
    }

    /// Os bytes do normal map da célula `key` (a cópia que o projeto grava), ou `None`.
    #[must_use]
    pub fn normal_da_celula(&self, key: u32) -> Option<&[u8]> {
        self.normais.bytes_da_celula(key)
    }

    /// A célula `key` do átlas tem normal registada?
    #[must_use]
    pub fn celula_tem_normal(&self, key: u32) -> bool {
        self.normais.tem_celula(key)
    }

    /// Quantas texturas têm normal registada (0 = nada alocado para normais).
    #[must_use]
    pub fn normais_alocadas(&self) -> usize {
        self.normais.len()
    }

    /// ⭐ P14 — as caixas no MUNDO (`[x0, y0, x1, y1]`) dos sprites da cena cuja imagem tem normal
    /// (textura individual com normal, ou recorte dentro de uma célula do átlas com normal). A luz
    /// liga o momento `M` só se alguma cruza a grade dela. Sem normal registada: nada, sem consulta.
    #[must_use]
    pub fn caixas_com_normal(&self, present: &mut PresentWorld) -> Vec<[f64; 4]> {
        let mut out = Vec::new();
        if self.normais.len() == 0 {
            return out;
        }
        let celulas: Vec<[f32; 4]> = self
            .normais
            .chaves_das_celulas()
            .map(|k| self.atlas.region_uv(k))
            .collect();
        let dentro = |uv: [f32; 4], c: &[f32; 4]| {
            let (u0, u1) = (uv[0].min(uv[2]), uv[0].max(uv[2]));
            let (v0, v1) = (uv[1].min(uv[3]), uv[1].max(uv[3]));
            u0 >= c[0] && v0 >= c[1] && u1 <= c[2] && v1 <= c[3]
        };
        let mut q = present.world_mut().query::<&RenderInstance>();
        for i in q.iter(present.world()) {
            let com = if i.texture_id == RenderInstance::ATLAS_TEXTURE_ID {
                celulas.iter().any(|c| dentro(i.atlas_uv, c))
            } else {
                self.normais.tem(i.texture_id)
            };
            if com {
                out.push(caixa_no_mundo(i));
            }
        }
        out
    }

    /// A textura `texture_id` tem normal registada?
    #[must_use]
    pub fn tem_normal(&self, texture_id: u32) -> bool {
        self.normais.tem(texture_id)
    }
}

/// A caixa no mundo do quad de uma instância (`world_pos + basis·(anchor + q·size)`, `q ∈ ±½`).
fn caixa_no_mundo(i: &RenderInstance) -> [f64; 4] {
    let mut c = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for q in [[-0.5_f32, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]] {
        let l = [
            i.anchor[0] + q[0] * i.size[0],
            i.anchor[1] + q[1] * i.size[1],
        ];
        let x = f64::from(i.world_pos[0] + l[0] * i.basis[0] + l[1] * i.basis[2]);
        let y = f64::from(i.world_pos[1] + l[0] * i.basis[1] + l[1] * i.basis[3]);
        c = [c[0].min(x), c[1].min(y), c[2].max(x), c[3].max(y)];
    }
    c
}
