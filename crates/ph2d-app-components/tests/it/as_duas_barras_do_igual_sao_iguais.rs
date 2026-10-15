//! ⭐⭐ **As duas barras do `=` têm o mesmo pico e um vão entre elas** — no preset de fábrica
//! (`CrispHeavyPlus`), a `s = 1,0`, em cada fonte da interface e na mono (CodeEditor 15/10).
//!
//! Report do dono: *«o traço superior parece mais grosso»*. Causa: `hint` OFF no preset e, com ele
//! ON, o `prep` inerte das fontes embutidas desviava o skrifa para o interpretador (no-op). A régua
//! é a da sonda `mede_as_duas_barras_do_igual` (tinta por linha de píxel). Sem placa (CI) não mede.

use super::texto_na_placa::{Letra, estilo_do_dono, placa, rasteriza};
use ph2d_text::TextSystem;
use ph2d_tokens::{TextRendering, UiFont, UiTextStyle};

/// Os picos das duas barras: a linha mais escura e a mais escura a ≥ 2 linhas dela.
fn picos(p: &[f32]) -> (f32, f32) {
    let arg = |f: &dyn Fn(usize) -> bool| {
        (0..p.len())
            .filter(|&r| f(r))
            .max_by(|&a, &b| p[a].total_cmp(&p[b]))
            .unwrap()
    };
    let r1 = arg(&|_| true);
    let r2 = arg(&|r| r.abs_diff(r1) >= 2);
    (p[r1.min(r2)], p[r1.max(r2)])
}

/// Medido 15/10 (tinta por linha de píxel do `=`, `s = 1,0`, preset de fábrica):
///
/// | letra | hint OFF (antes) | Δpico | hint ON + sem `prep` (agora) | Δpico |
/// |---|---|---|---|---|
/// | Inter | `5.20 / 5.49` | 5,2 % | `[4.65 6.07 · 4.74 6.07]` | 0 % |
/// | NotoSans | `5.88 / 4.22` | 28,3 % | `[0.74 5.88 1.84 · 0.92 5.88 1.93]` | 0 % |
/// | Atkinson | `6.02 / 5.54` | 7,9 % | `[6.12 3.72 3.91 6.12]` | 0 % |
/// | dono (Noto light small) | `3.42 / 4.01` | 14,7 % | `[5.18 0.84 5.27 0.92]` | 1,6 % |
/// | mono 12 px | `4.52 / 4.92` | 8,2 % | `[5.16 · 5.16]` | 0 % |
/// | mono 13 px | `3.75 / 4.45` | 15,7 % | `[5.59 · 5.59]` | 0 % |
///
/// A tolerância fica acima do maior resíduo de agora (1,6 %: as pontas da barra não caem no píxel
/// em x) e abaixo do menor defeito de antes (5,2 %). ⚠️ A Atkinson a `wght` 800 / 11 px não tem
/// linha de vão nem antes nem agora (o vão dela é menor que um píxel): o gate mede os picos, não o vão.
const TOL_PICO: f32 = 0.03;

#[test]
fn as_duas_barras_do_igual_tem_o_mesmo_pico_no_preset_de_fabrica() {
    let Some(mut placa) = placa((200, 40)) else {
        println!("sem adaptador: as barras do = não foram medidas");
        return;
    };
    let mut ts = TextSystem::without_system_fonts();
    let mut letras: Vec<(String, Letra)> = UiFont::ALL
        .iter()
        .map(|&font| {
            let estilo = UiTextStyle {
                font,
                ..Default::default()
            };
            (format!("{font:?}"), Letra::Ui(estilo))
        })
        .collect();
    letras.push((
        "dono (Noto light small)".into(),
        Letra::Ui(estilo_do_dono()),
    ));
    letras.push(("mono 12px".into(), Letra::Mono(12.0)));
    letras.push(("mono 13px".into(), Letra::Mono(13.0)));
    let mut falhas = Vec::new();
    for (nome, letra) in letras {
        let alfa = rasteriza(
            &mut placa,
            &mut ts,
            letra,
            TextRendering::CrispHeavyPlus,
            1.0,
            "=",
            (40.0, 30.0),
            None,
        );
        let linhas = alfa.linhas();
        let (sup, inf) = picos(&linhas);
        let delta = (sup - inf).abs() / sup.max(inf);
        let perfil: Vec<f32> = linhas.into_iter().filter(|&v| v > 0.0).collect();
        println!("{nome}: {perfil:.2?} Δ {:.1} %", 100.0 * delta);
        if delta > TOL_PICO {
            falhas.push(format!("{nome}: {perfil:.2?} (Δ {:.1} %)", 100.0 * delta));
        }
    }
    assert!(falhas.is_empty(), "o = tem barras desiguais: {falhas:#?}");
}
