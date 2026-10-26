//! ⭐⭐⭐ **A lista de regras do cartão «Animation» está VIVA sob o dedo** (plano GameEngineFuncional 08).
//!
//! Cada gesto pelo ponteiro REAL (o `click_at`/`drag_at` do testkit cai no rectângulo que a pintura
//! registou): as linhas pintadas com o texto das condições e das animações, os dois menus de cada
//! regra, o ×, «+ Rule» e o arrasto pela pega `[n]` — e, sem Platform Player, o menu «When» sem poses.

use crate::seam_anim::{anim, clear, click, click_fresh, host_with};
use crate::seam_anim_card::{edits, pick_rule, with_character};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{AnimFieldEdit, AnimWhenInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_anim};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn pick_when(
    info: ph2d_editor_core::screens::hero::InspectorAnimInfo,
    rule: usize,
    opt: usize,
) -> Vec<EditorAction> {
    pick_rule(
        info,
        &ids::INSP_ANIM_RULE_WHEN,
        &ids::INSP_ANIM_RULE_WHEN_OPT,
        rule,
        opt,
    )
}

/// Arrasta a pega da regra `from` para o quarto de cima da regra `to`; devolve o barramento.
fn drag_grip(from: usize, to: usize) -> Vec<EditorAction> {
    let (mut host, mut st) = host_with(with_character(anim(false, false)));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let at = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
    };
    let a = at(ids::INSP_ANIM_RULE_GRIP[from]);
    let b = at(ids::INSP_ANIM_RULE_GRIP[to]);
    let events = host.drag_at(
        a.x + a.w * 0.5,
        a.y + a.h * 0.5,
        b.x + b.w * 0.5,
        b.y + b.h * 0.25,
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            ph2d_editor_core::interaction::WidgetEvent::PanelRowReparent { .. }
        )),
        "arrastar a pega não levantou um PanelRowReparent: {events:?}"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let out = host.drained_actions();
    clear();
    out
}

/// Os gestos das regras que a varredura do `seam_anim` (toda variante alcançável) ainda não tem.
pub(crate) fn rule_gestures() -> Vec<Vec<EditorAction>> {
    vec![
        pick_when(with_character(anim(true, false)), 1, 5),
        click_fresh(
            with_character(anim(true, false)),
            ids::INSP_ANIM_RULE_DEL[0],
        ),
        click_fresh(with_character(anim(true, false)), ids::INSP_ANIM_RULE_ADD),
        drag_grip(2, 0),
    ]
}

/// (i) Um personagem com as 4 regras derivadas pinta 4 linhas — com as poses e as animações escritas.
#[test]
fn um_personagem_pinta_as_quatro_regras_com_o_seu_texto() {
    let info = with_character(anim(false, false));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_anim(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let textos: Vec<String> = host
        .medindo_a_pintura::<InspectorPanel>(&mut st, VIEWPORT)
        .into_iter()
        .map(|m| m.texto)
        .collect();
    clear();
    for (i, table) in [
        ids::INSP_ANIM_RULE_GRIP,
        ids::INSP_ANIM_RULE_WHEN,
        ids::INSP_ANIM_RULE_PLAY,
        ids::INSP_ANIM_RULE_DEL,
    ]
    .iter()
    .enumerate()
    {
        for (r, &id) in table.iter().enumerate() {
            assert_eq!(
                rects.iter().any(|(n, _)| *n == id),
                r < 4,
                "tabela {i}, regra {r}: tem de haver uma linha por regra, e nenhuma a mais"
            );
        }
    }
    for want in [
        "Standing still",
        "Running",
        "Going up",
        "Falling",
        "walk",
        "attack",
        "1",
        "4",
    ] {
        assert!(
            textos.iter().any(|t| t == want),
            "«{want}» não foi pintado: {textos:?}"
        );
    }
}

/// (ii) Abrir o «When» da regra 2 e escolher «Key held: down» escreve a condição dessa regra.
#[test]
fn o_menu_when_escolhe_a_tecla() {
    let info = with_character(anim(false, false));
    let options = info.card.when_options();
    let j = options
        .iter()
        .position(|o| *o == AnimWhenInfo::Action("down".into()))
        .expect("«down» é uma opção");
    let (mut host, mut st) = host_with(info.clone());
    host.set_dropdown_open(ids::INSP_ANIM_RULE_WHEN[1], true);
    let textos: Vec<String> = host
        .medindo_a_pintura::<InspectorPanel>(&mut st, VIEWPORT)
        .into_iter()
        .map(|m| m.texto)
        .collect();
    clear();
    assert!(
        textos.iter().any(|t| t == "Key held: down"),
        "o popover não diz «Key held: down»: {textos:?}"
    );
    assert_eq!(
        edits(&pick_when(info, 1, j)),
        [AnimFieldEdit::RuleWhen(
            1,
            AnimWhenInfo::Action("down".into())
        )]
    );
}

/// (iii) O «Play» de uma regra escreve a animação dessa regra.
#[test]
fn o_menu_play_escolhe_a_animacao_da_regra() {
    assert_eq!(
        edits(&pick_rule(
            with_character(anim(false, false)),
            &ids::INSP_ANIM_RULE_PLAY,
            &ids::INSP_ANIM_RULE_PLAY_OPT,
            3,
            0,
        )),
        [AnimFieldEdit::RulePlay(3, "walk".into())]
    );
}

/// (iv) O × retira a SUA regra.
#[test]
fn o_x_retira_a_sua_regra() {
    for i in [0, 3] {
        assert_eq!(
            edits(&click_fresh(
                with_character(anim(false, false)),
                ids::INSP_ANIM_RULE_DEL[i]
            )),
            [AnimFieldEdit::RuleRemove(i as u8)]
        );
    }
}

/// (v) «+ Rule» acrescenta a 1.ª condição do menu e a 1.ª animação da lista.
#[test]
fn mais_regra_acrescenta_as_primeiras_opcoes() {
    let info = with_character(anim(false, false));
    let first = info.card.when_options()[0].clone();
    assert_eq!(first, AnimWhenInfo::Pose(0));
    assert_eq!(
        edits(&click_fresh(info, ids::INSP_ANIM_RULE_ADD)),
        [AnimFieldEdit::RuleAdd(first, "walk".into())]
    );
}

/// (vi) Arrastar a pega da regra 3 para cima da regra 1 move-a para o topo; para baixo também.
#[test]
fn arrastar_a_pega_reordena() {
    assert_eq!(edits(&drag_grip(2, 0)), [AnimFieldEdit::RuleMove(2, 0)]);
    assert_eq!(edits(&drag_grip(0, 3)), [AnimFieldEdit::RuleMove(0, 2)]);
}

/// A pega só arrasta: o clique parado nela não edita nada (e é consumido — não está morta).
#[test]
fn clicar_na_pega_nao_edita() {
    let (mut host, mut st) = host_with(with_character(anim(false, false)));
    let out = click(&mut host, &mut st, ids::INSP_ANIM_RULE_GRIP[1]);
    clear();
    assert!(edits(&out).is_empty(), "{out:?}");
}

/// (vii) Sem Platform Player o menu «When» não oferece poses: a 1.ª opção é a 1.ª tecla.
#[test]
fn sem_corpo_o_menu_when_nao_tem_poses() {
    let mut info = with_character(anim(false, false));
    info.card.has_character_body = false;
    info.card.character = None;
    assert!(
        info.card
            .when_options()
            .iter()
            .all(|o| !matches!(o, AnimWhenInfo::Pose(_)))
    );
    let (mut host, mut st) = host_with(info.clone());
    host.set_dropdown_open(ids::INSP_ANIM_RULE_WHEN[0], true);
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    let n = info.card.when_options().len();
    for (j, &id) in ids::INSP_ANIM_RULE_WHEN_OPT.iter().enumerate() {
        assert_eq!(rects.iter().any(|(r, _)| *r == id), j < n, "opção {j}");
    }
    assert_eq!(
        edits(&pick_when(info, 0, 0)),
        [AnimFieldEdit::RuleWhen(
            0,
            AnimWhenInfo::Action("left".into())
        )]
    );
}

/// (viii) Report do dono 25/10: num painel ESTREITO a lista do «When» tem a largura dos nomes, não a
/// do chip — e o clique na parte da opção que antes ficava FORA do chip escolhe-a.
#[test]
fn num_painel_estreito_a_lista_do_when_alarga_e_o_clique_fora_do_chip_escolhe() {
    const ESTREITO: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 240.0,
        h: 2400.0,
    };
    let info = with_character(anim(false, false));
    let j = info
        .card
        .when_options()
        .iter()
        .position(|o| *o == AnimWhenInfo::Action("down".into()))
        .expect("«down» é uma opção");
    let (mut host, mut st) = host_with(info);
    host.set_dropdown_open(ids::INSP_ANIM_RULE_WHEN[1], true);
    let rects = host.paint::<InspectorPanel>(&mut st, ESTREITO);
    let at = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
    };
    let chip = at(ids::INSP_ANIM_RULE_WHEN[1]);
    let band = at(ids::INSP_ANIM_RULE_WHEN_OPT[j]);
    assert!(
        band.w > chip.w,
        "a opção ficou na largura do chip: {} ≤ {}",
        band.w,
        chip.w
    );
    assert!(
        band.x >= ESTREITO.x && band.x + band.w <= ESTREITO.x + ESTREITO.w,
        "a lista saiu do painel: {band:?}"
    );
    // Um ponto da banda FORA da faixa do chip — à direita dele, ou à esquerda se a lista recuou.
    let x = if band.x + band.w > chip.x + chip.w {
        (chip.x + chip.w + band.x + band.w) * 0.5
    } else {
        (band.x + chip.x) * 0.5
    };
    let events = host.click_at(x, band.y + band.h * 0.5);
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let out = host.drained_actions();
    clear();
    assert_eq!(
        edits(&out),
        [AnimFieldEdit::RuleWhen(
            1,
            AnimWhenInfo::Action("down".into())
        )]
    );
}

/// (ix) Plano 09: com o agachar armado o menu «When» oferece «Crouching» e escolhê-la grava essa
/// condição; sem ele a opção não aparece (seria uma regra que nunca vale).
#[test]
fn agachado_so_se_oferece_com_o_agachar_armado() {
    use ph2d_editor_core::screens::hero::ANIM_POSE_CROUCH;
    let crouch = AnimWhenInfo::Pose(ANIM_POSE_CROUCH);
    let mut info = with_character(anim(false, false));
    info.card.can_crouch = false;
    assert!(!info.card.when_options().contains(&crouch));
    info.card.can_crouch = true;
    let j = info
        .card
        .when_options()
        .iter()
        .position(|o| *o == crouch)
        .expect("«Crouching» é uma opção");
    assert_eq!(
        edits(&pick_when(info, 0, j)),
        [AnimFieldEdit::RuleWhen(0, crouch)]
    );
}

/// (x) Plano 10: «Crouch-walking» segue a mesma porta — só com o agachar armado, e grava a sua condição.
#[test]
fn a_andar_agachado_so_se_oferece_com_o_agachar_armado() {
    use ph2d_editor_core::screens::hero::ANIM_POSE_CROUCH_WALK;
    let walk = AnimWhenInfo::Pose(ANIM_POSE_CROUCH_WALK);
    let mut info = with_character(anim(false, false));
    info.card.can_crouch = false;
    assert!(!info.card.when_options().contains(&walk));
    info.card.can_crouch = true;
    let j = info
        .card
        .when_options()
        .iter()
        .position(|o| *o == walk)
        .expect("«Crouch-walking» é uma opção");
    assert_eq!(
        edits(&pick_when(info, 0, j)),
        [AnimFieldEdit::RuleWhen(0, walk)]
    );
}
