//! ⭐⭐⭐ **NUMA SECÇÃO, TODAS AS CAIXAS COMEÇAM NO MESMO `x` — mesmo quando um nome é mais longo.**
//!
//! ⛔⛔ **Report do dono, 2026-09-15, com foto deste painel e uma seta na linha *Major every (px)*:**
//! *«a caixa recua quando na verdade o nome deveria criar as colunas»*.
//!
//! Medido nesse dia com o sistema de texto real, painel a `220`:
//!
//! | linha | o nome quer | a coluna que ela recebia | a caixa |
//! |---|---|---|---|
//! | `Cell size (px)` | `70,1` | `90,0` (a metade) | `x = 98,0` · `w = 84,0` |
//! | **`Major every (px)`** | **`92,2`** | **`92,2`** | **`x = 100,2` · `w = 81,8`** ⛔ |
//! | `Origin X (px)` | `70,7` | `90,0` | `x = 98,0` · `w = 84,0` |
//!
//! A coluna do nome podia **pedir emprestado** ao controlo o que ele não usava (spec §6) — e o
//! pedido era medido POR LINHA. ⇒ a linha mais comprida empurrava a caixa **dela** e mais nenhuma.
//!
//! # ⚠️ Porque este gate pinta o PAINEL e não chama a porta
//!
//! A porta ([`ph2d_editor_core::property_row`]) sempre soube responder: dê-lhe o nome da secção e
//! ela devolve uma coluna só. O defeito era da **fiação** — quem lhe passava o nome. *Um gate que
//! refaz a conta da porta mede a conta, não o produto* (a lição que o
//! `the_painter_borrows_the_slack_the_control_does_not_need` do Inspector já pagou). ⇒ a régua é o
//! rectângulo que o painel **REGISTOU** para cada campo.
//!
//! # ⚠️ O CONTROLO: a largura tem de estar no regime em que o defeito vivia
//!
//! Numa coluna larga a metade já chega a todos os nomes e as quatro caixas alinham **por
//! construção** — ali este gate passaria com o defeito dentro. Por isso ele estreita o painel até o
//! empréstimo acontecer, e **afirma que ele aconteceu**: a caixa começa à DIREITA de onde a metade
//! sozinha a poria. Sem essa metade, a mutação que devolve a coluna por-linha sobrevive.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_grid_snap::GridSnapPanel;
use ph2d_text::TextSystem;
use ph2d_tokens::Spacing;
use ph2d_vector::VectorScene;
use std::sync::Once;

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<GridSnapPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

/// Quanto o painel encolhe a partir da largura de omissão (`304`).
///
/// ⚠️ **Não é um gosto: é o que põe a metade da linha ABAIXO do nome mais comprido da secção.**
/// Com `-92` a linha mede `188`, a metade dá `86` e o `Major every (px)` pede mais do que isso —
/// que é exactamente a foto do dono. O gate afirma esse regime em vez de o supor.
const ESTREITA_EM: f32 = -92.0;

/// As quatro linhas da secção *Square*, na ordem em que o painel as pinta.
fn campos_da_seccao() -> [NodeId; 4] {
    [
        ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
        ph2d_editor_core::grid_snap::ids::GS_CFG_SPACING_MAJOR,
        ph2d_editor_core::grid_snap::ids::GS_CFG_ORIGIN_X,
        ph2d_editor_core::grid_snap::ids::GS_CFG_ORIGIN_Y,
    ]
}

#[test]
fn a_seccao_poe_todas_as_caixas_na_mesma_coluna() {
    ph2d_editor_core::test_support::ensure_panel_registry();
    ensure_typed_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(GridSnapPanel::ID, true);
    let viewport = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();

    // 1ª passagem: o painel persiste o rect de omissão.
    paint_hero_screen(&mut hero, viewport, &mut scene, &mut text);
    // 2ª: estreitado até ao regime do report.
    hero.store
        .set_panel_resize_delta(ids::GS_PANEL, ESTREITA_EM, 0.0);
    paint_hero_screen(&mut hero, viewport, &mut scene, &mut text);

    let painel = hero
        .store
        .panel_rect(ids::GS_PANEL)
        .expect("o painel da Grelha tem de publicar o rect dele");
    let caixas: Vec<(NodeId, Rect)> = campos_da_seccao()
        .into_iter()
        .map(|id| {
            let r = hero
                .hit_index
                .rect_for(id)
                .unwrap_or_else(|| panic!("a linha {id:?} nao foi pintada"));
            (id, r)
        })
        .collect();

    // ⭐ **A LEI**: as quatro caixas partilham a coluna.
    let (primeiro_id, primeira) = caixas[0];
    for (id, r) in &caixas[1..] {
        assert!(
            (r.x - primeira.x).abs() < 0.01 && (r.w - primeira.w).abs() < 0.01,
            "a caixa de {id:?} comeca em {:.2} com largura {:.2} e a de {primeiro_id:?} em {:.2} \
             com {:.2} — a coluna desta seccao esta esfarrapada",
            r.x,
            r.w,
            primeira.x,
            primeira.w
        );
    }

    // ⭐⭐ **O CONTROLO** — ver o doc do módulo: sem isto a mutação sobrevive numa coluna larga.
    let recuo = Spacing::Lg.px();
    let vao = Spacing::Md.px();
    let linha = painel.w - 2.0 * recuo;
    let so_a_metade = painel.x + recuo + (linha * 0.5 - vao) + vao;
    assert!(
        primeira.x > so_a_metade + 0.5,
        "a este painel ({:.1}) a metade da linha ja' chega a todos os nomes (caixa em {:.2}, a \
         metade poria-a em {so_a_metade:.2}) — o gate esta a medir o regime EASY e a mutacao \
         sobrevive. Aperte o `ESTREITA_EM`.",
        painel.w,
        primeira.x
    );
}
