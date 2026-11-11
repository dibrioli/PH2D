//! **As duas linhas de editor do cartão Jump** — «Show Jump Arc» e «Feel» (ficha
//! `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`). Irmão de `seam_player.rs` pelo tamanho;
//! todo clique passa pelo `click_at` REAL, nos cartões e fora deles.

use crate::seam_player::{VIEWPORT, click_real, expect, host_aberto, player, toggle_real};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{InspectorPlayerInfo, PlayerFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_player,
};

/// Corre `f` com os cartões ligados e desligados: as duas saídas têm de estar vivas.
fn nos_dois_modos(f: impl Fn(bool)) {
    for on in [true, false] {
        set_cards_on(on);
        f(on);
    }
    set_cards_on(false);
}

/// O clique REAL: nos cartões pela janela do produto (`pinta` + ponteiro), fora deles pelo arnês do irmão.
fn clique(info: InspectorPlayerInfo, id: NodeId, cartoes: bool, caixa: bool) -> Vec<EditorAction> {
    if !cartoes {
        return if caixa {
            toggle_real(info, id)
        } else {
            click_real(info, id)
        };
    }
    use ph2d_editor_core::panel::PanelHostInternal;
    let mut host = host_aberto();
    host.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let r = crate::o_cartao_da_fisica::pinta(&mut host, &mut state);
    let rect = crate::o_cartao_da_fisica::achado(&r, id)
        .unwrap_or_else(|| panic!("nos cartões o Jump nunca pintou {id:?}"));
    let out = crate::o_cartao_da_fisica::clica(&mut host, &mut state, rect);
    set_current_inspector_player(None);
    out
}

#[test]
fn a_caixa_show_jump_arc_chega_ao_barramento_nas_duas_direcoes() {
    nos_dois_modos(|c| {
        expect(
            &clique(player(), i::INSP_PLAYER_SHOW_ARC, c, true),
            PlayerFieldEdit::ShowJumpArc(true),
            "marcar Show Jump Arc",
        );
        let ligada = InspectorPlayerInfo {
            show_jump_arc: true,
            ..player()
        };
        expect(
            &clique(ligada, i::INSP_PLAYER_SHOW_ARC, c, true),
            PlayerFieldEdit::ShowJumpArc(false),
            "desmarcar Show Jump Arc",
        );
    });
}

/// Cada segmento levanta o SEU índice (Floaty 0 · Snappy 1 · Heavy 2) — clicar «Heavy» é `Feel(2)`.
#[test]
fn cada_segmento_do_feel_levanta_o_seu_indice() {
    nos_dois_modos(|c| {
        for (n, &id) in i::INSP_PLAYER_FEEL_IDS.iter().enumerate() {
            expect(
                &clique(player(), id, c, false),
                PlayerFieldEdit::Feel(n as u8),
                "clicar num segmento do Feel",
            );
        }
    });
}

/// O host assente na janela do produto, com o player `feel`.
fn assente(feel: Option<u8>) -> (ph2d_ui_testkit::MockPanelHost, InspectorState) {
    use ph2d_editor_core::panel::PanelHostInternal;
    let mut host = host_aberto();
    host.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(InspectorPlayerInfo { feel, ..player() }));
    let _ = crate::o_cartao_da_fisica::pinta(&mut host, &mut state);
    (host, state)
}

fn desenho(feel: Option<u8>) -> Vec<u32> {
    let (mut host, mut state) = assente(feel);
    let d = host.paint_draw_data::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_player(None);
    d
}

/// ⭐ **O seletor nunca mente:** cada `Some(n)` pinta um estado próprio, e `None` pinta um QUARTO, com
/// outra tinta (nenhum fundo aceso) — nunca um dos três por defeito.
#[test]
fn o_feel_acende_o_segmento_do_preset_e_com_none_nenhum() {
    set_cards_on(true);
    let estados = [None, Some(0), Some(1), Some(2)];
    let d: Vec<Vec<u32>> = estados.iter().map(|&f| desenho(f)).collect();
    assert_eq!(
        desenho(Some(1)),
        d[2],
        "a mesma escolha tem de pintar o mesmo"
    );
    for a in 0..d.len() {
        for b in a + 1..d.len() {
            assert_ne!(
                d[a], d[b],
                "{:?} e {:?} pintam igual: o seletor não diz qual preset está aceso",
                estados[a], estados[b]
            );
        }
    }
    // A mesma TINTA nos três presets (um aceso, dois apagados, só muda qual) e outra sem preset.
    let tinta = |v: &Vec<u32>| {
        let mut v = v.clone();
        v.sort_unstable();
        v
    };
    assert_eq!(
        tinta(&d[1]),
        tinta(&d[2]),
        "Floaty e Snappy acesos gastam tintas diferentes"
    );
    assert_eq!(
        tinta(&d[1]),
        tinta(&d[3]),
        "Floaty e Heavy acesos gastam tintas diferentes"
    );
    assert_ne!(
        tinta(&d[0]),
        tinta(&d[1]),
        "com None um segmento continua aceso: o seletor mente"
    );
    set_cards_on(false);
}

/// ⭐ **A dobra More do Walk (logo acima do cartão Jump) a abrir e a fechar** nunca põe um campo dela
/// por cima das linhas novas, e as duas aparecem inteiras em todo o quadro da animação.
#[test]
fn a_animar_a_dobra_acima_as_linhas_do_pulo_ficam_inteiras_e_por_baixo() {
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1600.0, 3200.0);
    let layout = || {
        HeroLayout::for_viewport_bands(
            janela,
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            DockSides::BOTH,
        )
    };
    let dobra = i::INSP_PLAYER_FOLD_WALK_MORE;
    let dentro = [
        i::INSP_PLAYER_AIR_ACCEL,
        i::INSP_PLAYER_BRAKE,
        i::INSP_PLAYER_MAX_SLOPE,
    ];
    let novas = [i::INSP_PLAYER_SHOW_ARC, i::INSP_PLAYER_FEEL_IDS[2]];
    set_cards_on(true);
    let mut falhas = Vec::new();
    let mut vistos = 0;
    for fechar in [true, false] {
        let mut host = crate::host_em_metros();
        host.store_mut()
            .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
        for id in i::INSP_PLAYER_FOLDS {
            host.set_collapsed(id, id != dobra);
        }
        let mut state = InspectorState::default();
        set_current_inspector_player(Some(player()));
        let assente = crate::o_cartao_da_fisica::pinta(&mut host, &mut state);
        let alturas: Vec<f32> = novas
            .iter()
            .map(|&id| {
                crate::o_cartao_da_fisica::achado(&assente, id)
                    .unwrap_or_else(|| panic!("a linha nova {id:?} não foi pintada"))
                    .h
            })
            .collect();
        for n in 1..=48 {
            let t = n as f32 * 0.02;
            let t = if fechar { 1.0 - t } else { t };
            host.set_collapsed(dobra, fechar);
            host.store_mut().set_section_open_live(dobra, t);
            let rects = host.paint_with_layout::<InspectorPanel>(&mut state, layout(), janela);
            let Some(topo) = crate::o_cartao_da_fisica::achado(&rects, novas[0]) else {
                falhas.push(format!("a {t:.2}: Show Jump Arc sumiu"));
                continue;
            };
            for (&id, &h) in novas.iter().zip(&alturas) {
                vistos += 1;
                let agora = crate::o_cartao_da_fisica::achado(&rects, id);
                if agora.is_none_or(|q| (q.h - h).abs() > 0.5) {
                    falhas.push(format!(
                        "a {t:.2}: a linha {id:?} está {agora:?} (assente: {h:.1})"
                    ));
                }
            }
            for &id in &dentro {
                if let Some(r) = crate::o_cartao_da_fisica::achado(&rects, id)
                    && r.y + r.h > topo.y + 0.5
                {
                    falhas.push(format!(
                        "a {t:.2}: o campo {id:?} (fim {:.1}) passa por cima de Show Jump Arc ({:.1})",
                        r.y + r.h,
                        topo.y
                    ));
                }
            }
        }
        set_current_inspector_player(None);
    }
    set_cards_on(false);
    assert!(vistos > 0, "a régua não viu linha nenhuma");
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
