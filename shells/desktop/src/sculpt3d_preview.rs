//! **QUANDO O PREVIEW DO PADRÃO É RECALCULADO** — a política, separada do que
//! ele É.
//!
//! Filho (`#[path]`) de [`super`] pelo motivo dos irmãos: ele alcança os campos
//! privados da cena. O corte é entre *que valor cada vértice tem* (o kernel, na
//! `ph2d-sculpt3d`, que não sabe o que é um frame) e *em que frame vale a pena
//! perguntar de novo* — que é a única coisa que mora aqui.
//!
//! # Por que existe uma política, e não um recálculo por frame
//!
//! Medido pela porta do produto (`measure_alpha_over_a_whole_mesh`), avaliar o
//! padrão sobre a malha inteira custa:
//!
//! | malha | Noise | Scales/Pores/Cracks | Strata | Scratches |
//! |---|---|---|---|---|
//! | 96×144 (13,7k) | 1,23 ms | **2,33** | 1,25 | 0,12 |
//! | 533×800 (426k) | 37,9 | **63,7** | 36,4 | 3,8 |
//!
//! ⚠️ **64 ms são quatro quadros.** Um preview que recalculasse a malha inteira
//! a cada movimento do mouse faria o traço engasgar quanto maior a peça — que é
//! literalmente o defeito que o `upload.rs` da `ph2d-mesh-render` existe para
//! não ter, um canal ao lado. A lei é a mesma: **o custo de um gesto é função da
//! PEGADA, nunca do documento.**
//!
//! Daí as duas rotas, e elas não são duas respostas à mesma pergunta:
//!
//! * **a CHAVE mudou** (o artista escolheu outro padrão, outra escala, outro
//!   eixo) — o campo inteiro é outro, e não há janela que o descreva;
//! * **um dab MOVEU vértices** — o padrão é lido na POSIÇÃO do vértice, então
//!   os exatos vértices que se moveram são os que o preview deixou de descrever,
//!   e a janela é a mesma que a posição já sobe.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::Brush;

/// **O que faz o campo inteiro ser outro.**
///
/// ⚠️ **A contagem de vértices ENTRA**, e não é higiene: uma subdivisão ou um
/// remesh trocam a topologia sem tocar num knob do pincel, e um preview do
/// comprimento antigo não é *um pouco* velho — ele descreve vértices que não
/// existem mais. O renderizador o recusa (`preview_of`), mas recusar depois de
/// já não desenhar é tarde: aqui ele é RECOMPUTADO.
///
/// ⚠️ **A `strength` e o RAIO ficam de fora, e é a decisão do kernel:** o
/// preview mostra o CAMPO, não um dab — o falloff e a força pertencem ao gesto,
/// que ainda não aconteceu.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct PreviewKey {
    alpha: Option<ph2d_sculpt3d::Alpha>,
    scale: f32,
    az: u16,
    elev: u16,
    /// Se o verbo é freado pela máscara — o `paints_mask` do kernel, que decide
    /// se o padrão aparece sobre barro protegido.
    gated: bool,
    verts: usize,
}

impl PreviewKey {
    pub(crate) fn of(brush: &Brush, mesh: &Mesh) -> Self {
        Self {
            alpha: brush.alpha,
            scale: brush.alpha_scale,
            az: brush.alpha_az_deg,
            elev: brush.alpha_elev_deg,
            gated: !brush.verb.paints_mask(),
            verts: mesh.vert_count(),
        }
    }
}

/// O preview de uma peça e a chave que o produziu.
#[derive(Default)]
pub(crate) struct PreviewState {
    /// Um valor por vértice, ou **VAZIO** quando não há padrão armado — vazio é
    /// o que o renderizador lê como *desarmado*, e um vetor de zeros custaria
    /// 4 B por vértice para dizer a mesma coisa.
    pub(crate) values: Vec<f32>,
    key: Option<PreviewKey>,
    /// O preview inteiro precisa subir? ⚠️ **Ele é distinto do `dirty` da
    /// geometria**: um giro de eixo não move um vértice, então a janela do
    /// upload parcial estaria VAZIA e o barro continuaria com o padrão anterior.
    pub(crate) whole_dirty: bool,
}

impl PreviewState {
    /// **Põe o preview em dia**, pela rota mais barata que responde certo.
    ///
    /// `moved` são os vértices que o dab tocou desde a última vez — a MESMA
    /// lista que a posição sobe.
    pub(crate) fn refresh(&mut self, mesh: &Mesh, brush: &Brush, armed: bool, moved: &[u32]) {
        if !armed || brush.alpha.is_none() {
            // ⚠️ **Desarmar LIMPA, e a limpeza tem de subir.** Um preview que
            // ficasse no vetor continuaria no device: o artista desmarcaria a
            // caixa e o barro seguiria tingido, o que se lê como *"o botão não
            // faz nada"*.
            if !self.values.is_empty() {
                self.values.clear();
                self.whole_dirty = true;
            }
            self.key = None;
            return;
        }
        let key = PreviewKey::of(brush, mesh);
        if self.key != Some(key) {
            ph2d_sculpt3d::preview_into(mesh, brush, &mut self.values);
            self.key = Some(key);
            self.whole_dirty = true;
            return;
        }
        if !moved.is_empty() {
            ph2d_sculpt3d::preview_verts(mesh, brush, moved, &mut self.values);
        }
    }
}

#[cfg(test)]
#[path = "sculpt3d_preview_tests.rs"]
mod tests;
