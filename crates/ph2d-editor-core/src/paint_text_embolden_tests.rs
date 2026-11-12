//! ⭐ O *stem darkening* chega à TINTA: os `glyph_runs` da codificação levam o `FontEmbolden`
//! só no preset que o pede, também dentro do `com_peso_exato`, e os avanços não mudam.

use crate::paint::resolve;
use crate::paint::{paint_text, set_text_rendering};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, TextRendering, Theme};
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

/// A tinta mais escura que a superfície de referência não alarga, nem num tema escuro.
#[test]
fn dark_ink_is_never_emboldened() {
    crate::published::set_ui_theme(Theme::Dark);
    let (a, _) = pinta(TextRendering::CrispPlusDarkened, false, ESCURA);
    assert!(!a.is_empty() && a.iter().all(|a| *a == 0.0), "{a:?}");
}

/// ⭐ Dono 10/11 («para a UI também não faz diferença»): num tema escuro TODA a tinta do chrome mais
/// clara que o `BgElev` alarga — também o Text3 e o Accent (`Y < 0,5`), que a lei velha deixava de
/// fora; num tema claro nenhuma, nem a branca.
#[test]
fn on_a_dark_theme_every_chrome_ink_lighter_than_the_surface_is_emboldened() {
    let px = f64::from(ph2d_tokens::STEM_DARKENING_PX);
    for theme in [Theme::Dark, Theme::Forge, Theme::Oled] {
        crate::published::set_ui_theme(theme);
        for t in [
            ColorToken::Text1,
            ColorToken::Text2,
            ColorToken::Text3,
            ColorToken::Accent,
        ] {
            let (a, _) = pinta(TextRendering::CrispPlusDarkened, false, resolve(t, theme));
            assert!(
                !a.is_empty() && a.iter().all(|a| *a == px),
                "{theme:?} {t:?}: {a:?}"
            );
        }
    }
    for theme in [Theme::Light, Theme::Sunstone] {
        crate::published::set_ui_theme(theme);
        for ink in [resolve(ColorToken::Text1, theme), CLARA] {
            let (a, _) = pinta(TextRendering::CrispPlusDarkened, false, ink);
            assert!(
                !a.is_empty() && a.iter().all(|a| *a == 0.0),
                "{theme:?}: {a:?}"
            );
        }
    }
}

/// Com o fundo conhecido decide o CONTRASTE: a tinta mais clara que o fundo alarga (mesmo com
/// `Y < 0,5`), a mais escura não (mesmo com `Y > 0,5`), e o `0` não alarga nada.
#[test]
fn with_a_known_background_the_lighter_ink_is_emboldened() {
    use crate::paint::font_embolden_on;
    let roxo = Color::from_rgb8(0x9d, 0x7c, 0xd8);
    let cinza = Color::from_rgb8(0xe0, 0xe0, 0xe0);
    assert_eq!(
        font_embolden_on(0.1, roxo, ESCURA).amount.xx,
        0.1_f32 as f64
    );
    assert_eq!(font_embolden_on(0.1, cinza, CLARA).amount.xx, 0.0);
    assert_eq!(font_embolden_on(0.1, ESCURA, CLARA).amount.xx, 0.0);
    assert_eq!(font_embolden_on(0.0, CLARA, ESCURA).amount.xx, 0.0);
}

/// O texto RODADO (rótulos verticais) recebe o mesmo escurecimento do preset, e só nele.
#[test]
fn rotated_text_is_emboldened_only_in_the_darkened_preset() {
    crate::published::set_ui_theme(Theme::Dark);
    let corre = |mode| {
        let (mut ts, mut scene) = (TextSystem::without_system_fonts(), VectorScene::new());
        set_text_rendering(mode);
        crate::paint::paint_text_rotated_ccw(
            &mut ts,
            &mut scene,
            "Hierarchy",
            40.0,
            200.0,
            PX,
            f32::INFINITY,
            CLARA,
        );
        set_text_rendering(TextRendering::default());
        let r = &scene.inner().encoding().resources;
        let a: Vec<f64> = r
            .glyph_runs
            .iter()
            .map(|g| g.font_embolden.amount.xx)
            .collect();
        a
    };
    let px = f64::from(ph2d_tokens::STEM_DARKENING_PX);
    let (com, sem) = (
        corre(TextRendering::CrispPlusDarkened),
        corre(TextRendering::CrispHeavyPlus),
    );
    assert!(!com.is_empty() && com.iter().all(|a| *a == px), "{com:?}");
    assert!(sem.iter().all(|a| *a == 0.0), "{sem:?}");
}
