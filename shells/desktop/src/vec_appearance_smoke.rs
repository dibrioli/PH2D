//! ⭐⭐⭐ **A FORMA TEM OPACIDADE E MISTURA PRÓPRIAS** — `PH2D_VEC_APPEARANCE_SMOKE=1`.
//!
//! # O que a cena fecha
//!
//! Até 2026-09-05 uma forma vectorial não tinha nem uma nem outra: o painel oferecia duas rows
//! `Opacity` que são o **alfa da tinta que a ferramenta tem na mão** (elas semeiam a forma seguinte
//! e re-vestem a selecção), e mistura não havia em lado nenhum — `grep blend_mode` nas crates do
//! vector dava vazio. É o item 2 do estudo 42, e a metade que faltava ao report do Enio de 04/09
//! (*"o painel não mostra..."* — não havia o que mostrar).
//!
//! # A cena (unidades de mundo, ~±9)
//!
//! Uma **faixa de fundo** com três blocos de cor (frio · quente · claro) e, por cima dela, uma
//! fileira de discos — cada um com um **modo de mistura** diferente, na ordem em que o dropdown os
//! oferece. Mais dois quadrados iguais à direita: o de cima **opaco**, o de baixo a **40 %**.
//!
//! ⚠️ **O fundo é load-bearing:** um modo de mistura só existe contra o que está por baixo. Sobre
//! o fundo da tela todos os modos desenham a mesma coisa, e a cena ensinaria que a feature não
//! funciona.
//!
//! # O que provar
//!
//! 1. **Os discos não são todos iguais** — cada um compõe-se com a faixa de outra maneira (o
//!    `Multiply` escurece, o `Screen`/`Add` clareiam, o `Difference` inverte).
//! 2. **O quadrado de baixo é translúcido** e o de cima não, com a MESMA cor autorada: a
//!    opacidade é do OBJECTO, e não da tinta.
//! 3. **O painel edita as duas** — pegue a ferramenta Vector, clique numa forma, e a seção
//!    **Appearance** aparece com *Opacity* e *Blend*. Arrastar o slider muda a forma na hora;
//!    escolher outro modo na lista muda como ela se compõe.
//! 4. **Ctrl+Z desfaz** o arrasto inteiro num passo só.
//!
//! ⚠️ Se a linha `[vec-appearance-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_vec_scene::{BlendMode, Opacity, ShapeKind};

/// A faixa de fundo: três blocos, para os modos terem contra o que compor.
const FUNDO: [([f64; 2], [f64; 2], [u8; 3]); 3] = [
    ([-9.0, -1.5], [-3.0, 4.5], [40, 70, 160]),
    ([-3.0, -1.5], [3.0, 4.5], [190, 90, 40]),
    ([3.0, -1.5], [9.0, 4.5], [225, 225, 230]),
];

/// Os modos que a fileira demonstra — os que mais se lêem sobre uma faixa de cor.
///
/// ⚠️ **Uma amostra, não a lista**: a lista de verdade é a que o dropdown oferece (derivada da
/// tradução para o Vello), e repeti-la aqui seria uma segunda cópia dela a envelhecer.
const FILEIRA: [BlendMode; 5] = [
    BlendMode::Normal,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Add,
    BlendMode::Difference,
];

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn vec_appearance_smoke(&mut self) {
        if self.vec_appearance_smoke_done || std::env::var_os("PH2D_VEC_APPEARANCE_SMOKE").is_none()
        {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.vec_appearance_smoke_done = true;

        let scene = &mut self.gfx.as_mut().expect("gfx").vec_scene;
        for (a, b, rgb) in FUNDO {
            scene.push_path(crate::build_smoke::shape(
                ShapeKind::Rectangle,
                a,
                b,
                &[],
                rgb,
            ));
        }
        // A fileira de discos, um por modo, da esquerda para a direita.
        for (i, modo) in FILEIRA.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let x = -8.0 + i as f64 * 3.4;
            let id = scene.push_path(crate::build_smoke::shape(
                ShapeKind::Ellipse,
                [x, 0.0],
                [x + 3.0, 3.0],
                &[],
                [235, 215, 90],
            ));
            if let Some(p) = scene.path_mut(id) {
                p.blend = *modo;
            }
        }
        // O par da OPACIDADE: a mesma cor autorada, um opaco e um a 40 %.
        for (i, op) in [1.0_f32, 0.4].into_iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let y = -5.5 + i as f64 * -3.4;
            let id = scene.push_path(crate::build_smoke::shape(
                ShapeKind::Rectangle,
                [-2.0, y],
                [2.0, y + 3.0],
                &[],
                [90, 200, 140],
            ));
            if let Some(p) = scene.path_mut(id) {
                p.opacity = Opacity::new(op);
            }
        }

        eprintln!(
            "[vec-appearance-smoke] faixa de fundo + {} discos (um por modo de mistura: {}) e dois \
             quadrados iguais, o de baixo a 40%%. PEGUE a ferramenta Vector e clique numa forma: a \
             secao Appearance mostra Opacity e Blend DELA.",
            FILEIRA.len(),
            FILEIRA
                .iter()
                .map(|m| m.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}
