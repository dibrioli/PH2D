//! ⭐ O *stem darkening* chega à TINTA: os `glyph_runs` da codificação levam o `FontEmbolden`
//! só no preset que o pede, também dentro do `com_peso_exato`, e os avanços não mudam.

use crate::paint::{paint_text, set_text_rendering};
use ph2d_text::TextSystem;
use ph2d_tokens::TextRendering;
use ph2d_vector::{Color, Glyph, VectorScene};

/// Corpo da letra do chrome, abaixo do limiar do alargamento inteiro.
const PX: f32 = 11.0;
const CLARA: Color = Color::WHITE;
const ESCURA: Color = Color::BLACK;

/// (alargamento de cada corrida, glifos posicionados) de uma frase pintada em `mode`.
fn pinta(mode: TextRendering, exato: bool, ink: Color) -> (Vec<f64>, Vec<Glyph>) {
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    set_text_rendering(mode);
    let mut f = || {
        paint_text(
            &mut ts,
            &mut scene,
            "self.sped is never set",
            0.0,
            0.0,
            PX,
            f32::INFINITY,
            ink,
        )
    };
    if exato {
        ph2d_text::com_peso_exato(f);
    } else {
        f();
    }
    set_text_rendering(TextRendering::default());
    let r = &scene.inner().encoding().resources;
    let a = r
        .glyph_runs
        .iter()
        .map(|g| g.font_embolden.amount.xx)
        .collect();
    (a, r.glyphs.clone())
}

#[test]
fn the_darkened_preset_emboldens_every_run_and_keeps_the_advances() {
    let (fabrica, gf) = pinta(TextRendering::CrispHeavyPlus, false, CLARA);
    let (escuro, ge) = pinta(TextRendering::CrispPlusDarkened, false, CLARA);
    assert!(!escuro.is_empty());
    assert!(
        fabrica.iter().all(|a| *a == 0.0),
        "fábrica sem alargamento: {fabrica:?}"
    );
    let px = f64::from(ph2d_tokens::STEM_DARKENING_PX);
    assert!(escuro.iter().all(|a| *a == px), "{escuro:?}");
    let pos = |g: &[Glyph]| g.iter().map(|q| (q.id, q.x, q.y)).collect::<Vec<_>>();
    assert_eq!(pos(&gf), pos(&ge), "o alargamento não mexe nos avanços");
}

/// ⚠️ O `com_peso_exato` tira o reforço de PESO; a borda continua (a aba de scripts pinta lá).
#[test]
fn the_exact_weight_scope_keeps_the_stem_darkening() {
    let (a, _) = pinta(TextRendering::CrispPlusDarkened, true, CLARA);
    assert!(!a.is_empty() && a.iter().all(|a| *a > 0.0), "{a:?}");
}

/// Só a letra CLARA: a escura (o tema Light) fica como estava — a medição não a aproxima.
#[test]
fn dark_ink_is_never_emboldened() {
    let (a, _) = pinta(TextRendering::CrispPlusDarkened, false, ESCURA);
    assert!(!a.is_empty() && a.iter().all(|a| *a == 0.0), "{a:?}");
}
