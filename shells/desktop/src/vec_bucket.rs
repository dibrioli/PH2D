//! ⭐⭐⭐ **O BALDE na shell** (plano 40) — a costura entre o ponteiro e a lei
//! ([`ph2d_vec_fill`]).
//!
//! # As duas coisas que esta camada decide
//!
//! 1. **QUE contornos entram na rede** — os visíveis do documento, cozidos e assados no MUNDO (a
//!    convenção do `apply_vec_boolean`). A lei não sabe o que é uma cena.
//! 2. **QUANDO a rede é reconstruída.** ⚠️⚠️ **Medido, e é o que decide o desenho desta camada:**
//!
//!    | contornos | arcos | montar a rede | achar a face |
//!    |---|---|---|---|
//!    | 4 | 8 | `0,06 ms` | `0,01 ms` |
//!    | 10 | 136 | `0,72 ms` | `0,05 ms` |
//!    | 20 | 280 | **`3,80 ms`** | `0,08 ms` |
//!    | 40 | 628 | **`26,3 ms`** | `0,18 ms` |
//!    | 80 | 1293 | **`188 ms`** | `0,35 ms` |
//!
//!    ⇒ **montar por quadro está REFUTADO** (o orçamento é 16,7 ms e ele estoura aos ~20 traços),
//!    e **achar a face por quadro é de graça**. Por isso a rede é **guardada** e só se refaz quando
//!    a geometria muda — o que, com o balde na mão, é raro: não há gizmo neste modo, então o
//!    documento só muda quando o próprio balde deposita uma forma (ou num undo).
//!
//! ⚠️ **A chave do cache é o CONTEÚDO** (uma soma sobre as âncoras), e não a contagem de caminhos:
//! mover uma forma não muda quantas há, e um cache que não visse isso acenderia uma face onde já
//! não há linha nenhuma.

use ph2d_vec_fill::Rede;
use ph2d_vec_scene::{VecPath, VecScene, VecVertex, VecXforms, trim_tool};

/// A rede guardada, com a chave do documento que a produziu.
pub(crate) struct BucketCache {
    chave: u64,
    rede: Rede,
}

/// **Os contornos VISÍVEIS do documento, no MUNDO** — o universo em que o balde procura faces.
///
/// ⛔ Os escondidos ficam de fora: uma linha que não se vê não pode cercar uma região que o artista
/// aponta — ele estaria a preencher contra uma parede invisível.
fn contornos_mundo(
    scene: &VecScene,
    xforms: &VecXforms,
    oculto: &dyn Fn(u64) -> bool,
) -> Vec<(Vec<VecVertex>, bool)> {
    let mut out = Vec::new();
    for p in scene.paths() {
        if oculto(p.id) {
            continue;
        }
        let mut cozido = p.cooked().into_owned();
        ph2d_vec_scene::bake_xform(&mut cozido, &ph2d_vec_scene::xform_of(xforms, p.id));
        for c in trim_tool::contours_of(&cozido) {
            if c.verts.len() >= 2 {
                out.push((c.verts.clone(), c.closed));
            }
        }
    }
    out
}

/// A chave do documento: o conteúdo das âncoras, e não a contagem.
fn chave(contornos: &[(Vec<VecVertex>, bool)]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (verts, closed) in contornos {
        h = h.wrapping_mul(0x0100_0000_01b3) ^ u64::from(*closed);
        for v in verts {
            for x in [v.anchor[0], v.anchor[1], v.out_handle[0], v.out_handle[1]] {
                h = h.wrapping_mul(0x0100_0000_01b3) ^ x.to_bits();
            }
        }
    }
    h
}

impl crate::App {
    /// ⭐⭐⭐ **Recalcula a face sob o cursor** — uma vez por quadro, ao lado do realce do Trim.
    ///
    /// ⚠️ **Fora do modo Balde ele é LIMPO**, e não apenas não-actualizado: uma região a arder
    /// depois de trocar de ferramenta prometeria um preenchimento que nenhum clique faria.
    pub(crate) fn refresh_bucket_hover(&mut self, pointer: (f32, f32)) {
        if self.vec_draw_config.mode != ph2d_tool_vector::DrawMode::Bucket {
            self.vec_bucket_face = None;
            self.vec_bucket_cache = None; // a rede não sobrevive à troca de ferramenta
            return;
        }
        let Some(world) = self.vec_world_at(pointer) else {
            self.vec_bucket_face = None;
            return;
        };
        let Some(gfx) = self.gfx.as_ref() else {
            self.vec_bucket_face = None;
            return;
        };
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        let vista = crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
        let contornos = contornos_mundo(&gfx.vec_scene, &xf, &|id| vista.is_hidden(id));
        let k = chave(&contornos);
        if self.vec_bucket_cache.as_ref().is_none_or(|c| c.chave != k) {
            self.vec_bucket_cache = Some(BucketCache {
                chave: k,
                rede: ph2d_vec_fill::rede(&contornos),
            });
        }
        let Some(cache) = self.vec_bucket_cache.as_ref() else {
            self.vec_bucket_face = None;
            return;
        };
        self.vec_bucket_face = cache
            .rede
            .face_em(world)
            .map(|f| cache.rede.geometria(&f))
            .filter(|g| g.len() >= 2)
            .map(|verts| VecPath {
                verts,
                closed: true,
                ..VecPath::default()
            });
    }

    /// **A tinta que o balde deposita** — a corrente da ferramenta.
    ///
    /// ⚠️ **`alpha == 0` significa SEM preenchimento** neste app (a convenção que o Shape tool usa
    /// ao fechar uma forma), e um balde que a ignorasse depositaria formas invisíveis.
    pub(crate) fn bucket_paint(&self) -> Option<ph2d_vec_scene::Rgba8> {
        let f = self.vec_pen.style().fill;
        (f.a != 0).then_some(f)
    }

    /// ⭐⭐⭐ **DEPOSITA a face que está acesa.** `true` se algo nasceu.
    ///
    /// ⚠️ **A geometria vem do ESTADO DO QUADRO** (`vec_bucket_face`), e não de um cálculo feito
    /// aqui: o que o artista vê aceso é literalmente o que fica. Recalcular no clique abriria a
    /// porta para o cursor ter andado um pixel entre o desenho e o gesto.
    ///
    /// ⚠️ **A forma nasce ATRÁS de tudo** (`insert_path(0, …)`): ela é o fundo da região, e por
    /// cima dela têm de continuar a ver-se as linhas que a cercam. ⛔ Ao topo, ela tapá-las-ia — e
    /// o artista veria o desenho desaparecer sob a própria tinta.
    pub(crate) fn apply_bucket(&mut self) -> bool {
        let Some(tinta) = self.bucket_paint() else {
            eprintln!(
                "[ph2d-vec] balde: o preenchimento corrente e' transparente — escolha uma cor"
            );
            return false;
        };
        let Some(face) = self.vec_bucket_face.clone() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let nova = VecPath {
            fill: Some(ph2d_vec_scene::Paint::solid(tinta)),
            ..face
        };
        let id = gfx.vec_scene.insert_path(0, nova);
        self.vec_pen.select(Some(id));
        // A rede mudou (há uma forma a mais): o cache cai, e o próximo quadro reconstrói.
        self.vec_bucket_cache = None;
        self.vec_bucket_face = None;
        true
    }
}

#[cfg(test)]
#[path = "vec_bucket_tests.rs"]
mod tests;
