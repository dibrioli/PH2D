//! ⭐⭐⭐ **UM CAMPO NUMÉRICO NUNCA É PINTADO MAIS ESTREITO DO QUE ELE PRÓPRIO DECLARA.**
//!
//! ⛔⛔ **É uma ORDEM DO DONO, e ela estava escrita no código desde 2026-05-24** — no doc do
//! [`ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX`]:
//!
//! > *«Panel layouts that scale chip width with available space … must clamp the chip below this
//! > floor. User feedback: "não permita que a caixa seja redimencionada para menor que isso".»*
//!
//! ⚠️⚠️ **E a linha de propriedade que nasceu em 2026-09-14 violava-a em metade do curso do dock.**
//! O piso que ela nomeava era `ICON_BTN_SIZE_PX + Spacing::Lg` = `48`, com o doc a dizer que `36`
//! era *«a largura da coluna do stepper de um `NumberInput`»* — e a coluna do stepper é
//! [`ph2d_editor_core::widget::number_input::stepper_width`], que dá `clamp(0,6 × altura, 16, 22)`.
//! *O piso dizia de que recurso era e estava errado sobre ele* (`CLAUDE.md` §0.0).
//!
//! Medido no produto (o rect REGISTADO do campo `Float Height` da §14), antes da cura:
//!
//! | painel | campo pintado | piso declarado |
//! |---|---|---|
//! | `220` (o mínimo do dock) | `48,00` | `72` ⛔ |
//! | `245` | `59,38` | `72` ⛔ |
//! | `280` | `94,38` | `72` ✅ |
//!
//! # ⚠️ Por que a régua é o RECT REGISTADO e não a porta
//!
//! Um gate que chamasse `property_row_columns` e comparasse com o piso estaria a refazer a conta do
//! produto — e a mutação que troca o piso mudaria os dois lados ao mesmo tempo. Aqui pinta-se o
//! painel a sério e lê-se o que ele **entregou ao `HitIndex`**: é o mesmo rect que recebe o dedo.

use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_player};
use ph2d_ui_testkit::MockPanelHost;

/// ⭐ **O curso que o artista alcança**, não uma largura escolhida: o dock vai de
/// [`ph2d_tokens::PANEL_MIN_W_PX`] a `720`
/// (`ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN`..`DOCK_W_MAX`).
///
/// ⛔ **A escada NÃO inclui `200`**, que a redacção anterior media como *«o pior caso plausível»*:
/// o dock não desce abaixo de `220`, logo aquilo era uma largura que **nenhum arrasto produz** —
/// *uma régua calibrada fora do curso mede um programa que não existe*.
const LARGURAS: &[f32] = &[
    220.0, 232.0, 245.0, 260.0, 273.3, 288.0, 304.0, 400.0, 720.0,
];

fn campo_registado(painel_w: f32, id: ph2d_a11y::NodeId) -> f32 {
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        w: 2400.0,
        h: 8000.0,
    };
    let mut layout = HeroLayout::for_viewport(viewport);
    layout.inspector = Rect {
        x: viewport.w - painel_w,
        y: 0.0,
        w: painel_w,
        h: 8000.0,
    };
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(crate::seam_player::player()));
    let rects = host.paint_with_layout::<InspectorPanel>(&mut state, layout, viewport);
    set_current_inspector_player(None);
    rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| r.w)
        .unwrap_or_else(|| panic!("o campo nao foi pintado a' largura {painel_w}"))
}

#[test]
fn a_field_is_never_narrower_than_its_owner_declared() {
    let piso = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;
    // ⚠️ **Dois campos, e não um.** Um só mediria a aritmética de UMA row; estes dois vivem em
    //    cartões diferentes da §14, logo entram na porta com `x`/`w` diferentes.
    let campos: [(&str, ph2d_a11y::NodeId); 2] = [
        (
            "Float Height (§14)",
            ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT,
        ),
        (
            "Jump Height (§14)",
            ph2d_panel_inspector::ids::INSP_PLAYER_JUMP_HEIGHT,
        ),
    ];
    let mut medidos = 0usize;
    let mut estreitos = Vec::new();
    for painel in LARGURAS {
        for (nome, id) in campos {
            let w = campo_registado(*painel, id);
            medidos += 1;
            if w + 0.01 < piso {
                estreitos.push(format!(
                    "{nome} a' largura de painel {painel}: {w:.2} px, abaixo do piso de {piso}"
                ));
            }
        }
    }
    assert_eq!(
        medidos,
        LARGURAS.len() * campos.len(),
        "piso de populacao: a varredura mediu {medidos} celulas"
    );
    assert!(
        estreitos.is_empty(),
        "{} campo(s) pintado(s) abaixo do que o dono mandou:\n  {}\n\n\
         O piso da coluna do rotulo e' `NUMBER_INPUT_MIN_W_PX` — ver `property_label_col_w_for`.",
        estreitos.len(),
        estreitos.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO da régua acima** — ela tem de saber ver um campo estreito, senão passa sobre um
/// painel que deixou de pintar o campo.
///
/// ⚠️ *Uma fixtura sem o fenómeno mede silêncio*: aqui confirma-se que a largura do campo **responde
/// à largura do painel** (nas duas pontas do curso ela tem de ser diferente) e que na ponta larga
/// ela está **bem acima** do piso — se as duas leituras fossem iguais, a varredura estaria a medir
/// uma constante.
#[test]
fn the_ruler_can_see_the_field_follow_the_panel() {
    let estreito = campo_registado(220.0, ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT);
    let largo = campo_registado(720.0, ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT);
    assert!(
        largo > estreito + 100.0,
        "o campo mede {estreito:.1} a 220 e {largo:.1} a 720 — a regua esta' a medir uma constante"
    );
    assert!(
        (estreito - ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX).abs() < 0.01,
        "a 220 (o minimo do dock) o campo devia estar EXACTAMENTE no piso e mede {estreito:.2} — \
         se ficou acima, a coluna do rotulo deixou de pedir emprestado o que podia"
    );
}
