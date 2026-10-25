//! ⭐⭐⭐ **O cartão «Animation» está VIVO sob o dedo** (plano GameEngineFuncional 07).
//!
//! Cada controlo novo é dirigido pelo ponteiro REAL (o `click_at` do testkit cai no rectângulo que a
//! pintura registou) e afirma a `AnimFieldEdit` que levanta: o ▶ da linha, «Edit Animations…»,
//! «Who Picks», os quatro menus de estado (exactamente as animações da lista, e seguem um renomear),
//! «Run From», «Flip» e o grupo «More» que nasce FECHADO. E as duas saídas: o Platform Player já não
//! pinta o «Auto Animate», e não há secção «Sprite Sheet».

use crate::seam_anim::{anim, clear, click, click_fresh, commit, host_with};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::screens::hero::{
    AnimCharacterInfo, AnimChooser, AnimFieldEdit, InspectorAnimInfo,
};
use ph2d_editor_core::screens::layout::ChromeBands;
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

fn edits(actions: &[EditorAction]) -> Vec<AnimFieldEdit> {
    actions
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorAnimEdit { edit, .. } => Some(edit.clone()),
            _ => None,
        })
        .collect()
}

/// A fixtura com a animação a tocar (`walk`, a 1.ª) feita LISTA de quadros: 2 · 0 · 2 · 3.
pub(crate) fn with_frame_list(mut info: InspectorAnimInfo) -> InspectorAnimInfo {
    info.rows[0].frames = vec![2, 0, 2, 3];
    info.frame = 2;
    info.card.step = 2;
    info
}

/// A fixtura no modo «The Character», com corpo; os estados apontam para as duas animações.
pub(crate) fn with_character(mut info: InspectorAnimInfo) -> InspectorAnimInfo {
    info.card.chooser = AnimChooser::Character;
    info.card.has_character_body = true;
    info.card.character = Some(AnimCharacterInfo {
        names: [
            "walk".into(),
            "attack".into(),
            "walk".into(),
            "attack".into(),
        ],
        run_from: 0.75,
        flip: true,
    });
    info
}

/// Abre o menu do estado `state` e clica na opção `opt`; devolve as edições.
fn pick_state(info: InspectorAnimInfo, state: usize, opt: usize) -> Vec<EditorAction> {
    let (mut host, mut st) = host_with(info);
    host.set_dropdown_open(ids::INSP_ANIM_STATE_PICK[state], true);
    let out = click(&mut host, &mut st, ids::INSP_ANIM_STATE_OPT[opt]);
    assert_eq!(
        host.dropdown_is_open(ids::INSP_ANIM_STATE_PICK[state]),
        Some(false),
        "escolher uma opção tem de fechar o menu"
    );
    clear();
    out
}

/// Os três gestos do modo «The Character», para a varredura do `seam_anim`.
pub(crate) fn character_gestures() -> Vec<Vec<EditorAction>> {
    vec![
        click_fresh(with_character(anim(true, false)), ids::INSP_ANIM_FLIP),
        commit(
            with_character(anim(true, false)),
            ids::INSP_ANIM_RUN_FROM,
            1.5,
        ),
        pick_state(with_character(anim(true, false)), 1, 0),
    ]
}

#[test]
fn o_play_de_uma_linha_escolhe_e_poe_a_tocar() {
    assert_eq!(
        edits(&click_fresh(anim(false, false), ids::INSP_ANIM_PLAY_ROW[1])),
        [
            AnimFieldEdit::SetCurrent("attack".into()),
            AnimFieldEdit::Playing(true)
        ]
    );
}

#[test]
fn editar_animacoes_pede_o_editor() {
    assert_eq!(
        edits(&click_fresh(anim(false, false), ids::INSP_ANIM_OPEN_EDITOR)),
        [AnimFieldEdit::OpenEditor]
    );
}

/// ⚠️ A POSIÇÃO do segmento é o `AnimChooser::tag` — trocar o array escreveria outra escolha.
#[test]
fn quem_escolhe_cada_segmento_escreve_a_sua_escolha() {
    let mut info = anim(false, false);
    info.card.has_character_body = true;
    for (i, c) in AnimChooser::ALL.iter().enumerate() {
        assert_eq!(
            edits(&click_fresh(info.clone(), ids::INSP_ANIM_CHOOSER[i])),
            [AnimFieldEdit::Chooser(c.tag())],
            "{c:?}"
        );
    }
}

/// ⛔ Sem Platform Player «The Character» não pega: o clique não emite nada (a cena recusá-lo-ia).
#[test]
fn o_personagem_nao_se_oferece_sem_platform_player() {
    let info = anim(false, false);
    assert!(!info.card.has_character_body);
    assert!(edits(&click_fresh(info, ids::INSP_ANIM_CHOOSER[2])).is_empty());
}

/// Os menus só existem no modo «The Character».
#[test]
fn os_menus_de_estado_so_no_modo_do_personagem() {
    for (info, want) in [
        (anim(false, false), false),
        (with_character(anim(false, false)), true),
    ] {
        let (mut host, mut st) = host_with(info);
        let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
        clear();
        for id in ids::INSP_ANIM_STATE_PICK
            .into_iter()
            .chain([ids::INSP_ANIM_RUN_FROM, ids::INSP_ANIM_FLIP])
        {
            assert_eq!(rects.iter().any(|(n, _)| *n == id), want, "{id:?}");
        }
    }
}

/// ⭐⭐ **O menu lista EXACTAMENTE as animações e segue um renomear** — escolher a 2.ª opção escreve o
/// nome dela; com a lista renomeada, a mesma opção escreve o nome NOVO.
#[test]
fn o_menu_de_estado_lista_as_animacoes_e_segue_o_renomear() {
    let info = with_character(anim(false, false));
    let (mut host, mut st) = host_with(info.clone());
    host.set_dropdown_open(ids::INSP_ANIM_STATE_PICK[0], true);
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    for (i, &id) in ids::INSP_ANIM_STATE_OPT.iter().enumerate() {
        assert_eq!(
            rects.iter().any(|(n, _)| *n == id),
            i < info.rows.len(),
            "opção {i}: o menu tem de ter uma linha por animação, e nenhuma a mais"
        );
    }
    assert_eq!(
        edits(&pick_state(info.clone(), 2, 1)),
        [AnimFieldEdit::CharacterState(2, "attack".into())]
    );
    let mut renamed = info;
    renamed.rows[1].name = "golpe".into();
    renamed.card.character.as_mut().unwrap().names[1] = "golpe".into();
    assert_eq!(
        edits(&pick_state(renamed, 2, 1)),
        [AnimFieldEdit::CharacterState(2, "golpe".into())]
    );
}

#[test]
fn run_from_e_flip_escrevem_o_personagem() {
    assert_eq!(
        edits(&commit(
            with_character(anim(false, false)),
            ids::INSP_ANIM_RUN_FROM,
            1.5
        )),
        [AnimFieldEdit::CharacterRunFrom(1.5)]
    );
    // O contrário do que está DESENHADO (`flip: true` ⇒ desmarcar).
    assert_eq!(
        edits(&click_fresh(
            with_character(anim(false, false)),
            ids::INSP_ANIM_FLIP
        )),
        [AnimFieldEdit::CharacterFlip(false)]
    );
}

/// ⭐ **«More» nasce FECHADO** e abre-se pelo cabeçalho: o tocador fino só aparece depois.
#[test]
fn o_grupo_more_nasce_fechado_e_abre_pelo_cabecalho() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_anim(Some(anim(true, false)));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rects.iter().any(|(n, _)| *n == ids::INSP_ANIM_MORE));
    assert!(!rects.iter().any(|(n, _)| *n == ids::INSP_ANIM_SPEED));
    let _ = click(&mut host, &mut st, ids::INSP_ANIM_MORE);
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    assert!(
        rects.iter().any(|(n, _)| *n == ids::INSP_ANIM_SPEED),
        "o cabeçalho de «More» não abre — o grupo não está marcado dobrável"
    );
}

/// Numa LISTA de quadros o arrasto da barra escolhe o PASSO — a célula 2 aparece duas vezes.
#[test]
fn numa_lista_a_barra_escolhe_o_passo() {
    let (mut host, mut st) = host_with(with_frame_list(anim(true, false)));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_ANIM_FRAME_SCRUB)
        .map(|(_, r)| *r)
        .expect("a barra não foi pintada");
    for ev in host.click_at(r.x + r.w * 0.99, r.y + r.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let out = host.drained_actions();
    clear();
    assert_eq!(edits(&out), [AnimFieldEdit::SetStep(3)]);
}

/// ⛔ **O Platform Player já não pinta o «Auto Animate»** — nenhum dos ids do bloco antigo.
#[test]
fn o_platform_player_ja_nao_pinta_as_animacoes() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    ph2d_panel_inspector::set_current_inspector_player(Some(crate::seam_player::player()));
    host.open_all_sections();
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    ph2d_panel_inspector::set_current_inspector_player(None);
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION),
        "a fixtura não pintou o cartão Platform Player — o gate não mediria nada"
    );
    for old in [
        "insp_player_anim_off",
        "insp_player_anim_on",
        "insp_player_anim_idle",
        "insp_player_anim_run",
        "insp_player_anim_jump",
        "insp_player_anim_fall",
        "insp_player_anim_run_from",
        "insp_player_anim",
        "insp_player_anim_flip_off",
        "insp_player_anim_flip_on",
    ] {
        let id = ph2d_tool_registry::hash_node_id(old);
        assert!(!rects.iter().any(|(n, _)| *n == id), "{old} ainda pintado");
    }
}

/// ⛔ **Não há secção «Sprite Sheet»** — e a grelha não se perdeu: numa sprite sem animação ela está
/// no Render Source; com animação, no cartão «Animation». Uma só vez em cada caso.
#[test]
fn nao_ha_seccao_sprite_sheet_e_a_grelha_tem_um_so_lugar() {
    for anim_info in [None, Some(anim(false, false))] {
        let with_anim = anim_info.is_some();
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut st = InspectorState::default();
        ph2d_panel_inspector::set_current_inspector_sprite(Some(
            crate::seam_render_source::atlas_sprite(false),
        ));
        set_current_inspector_anim(anim_info);
        host.open_all_sections();
        let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
        ph2d_panel_inspector::set_current_inspector_sprite(None);
        clear();
        assert!(
            !rects
                .iter()
                .any(|(n, _)| *n == ph2d_editor_core::ids::INSP_LIVE_SHEET_SECTION),
            "a secção «Sprite Sheet» voltou a ser pintada"
        );
        for id in [ids::INSP_SPRITE_HFRAMES, ids::INSP_SPRITE_VFRAMES] {
            assert_eq!(
                rects.iter().filter(|(n, _)| *n == id).count(),
                1,
                "{id:?} (com animação: {with_anim})"
            );
        }
        // Centrado e espelho continuam na sprite.
        assert!(rects.iter().any(|(n, _)| *n == ids::INSP_SPRITE_CENTERED));
        assert!(rects.iter().any(|(n, _)| *n == ids::INSP_SPRITE_FLIP_X));
    }
}

/// ⭐ Report do dono (foto 01): «Show sheet on…» e a frase de «Who Picks» («The character picks by
/// what he does: stand…») saíam cortados. No modo «The Character», com grelha, à largura do dock da
/// foto: o nome cabe na coluna e a frase QUEBRA em várias linhas — nenhuma das duas se elide.
#[test]
fn o_cartao_nao_corta_o_nome_da_grelha_nem_a_frase_de_quem_escolhe() {
    // O dock de fábrica e o da foto (a janela de 1600 com o Inspector estreitado).
    for w in [ph2d_tokens::INSPECTOR_W_PX, 290.0, 270.0] {
        // LITERAL-PX-OK: larguras do dock
        card_fits_at(w);
    }
}

fn card_fits_at(w: f32) {
    let mut info = with_character(anim(false, false));
    info.card.hframes = 4;
    info.card.vframes = 2;
    // ⚠️ Pela pintura do REGISTO e com o Transform ao lado: a coluna dos nomes é UMA por painel
    // (`ColunaDoPainel`), e quem a prende são as fileiras de dois campos das outras secções — o
    // cartão sozinho teria a coluna toda para si e não cortaria nada.
    let (mut host, mut st) = host_with(info);
    ph2d_panel_inspector::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [2000.0, 800.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let mut painel = ph2d_editor_core::panel::ErasedPanel::new::<InspectorPanel>();
    let bandas = ChromeBands {
        right_dock_w: w,
        ..ChromeBands::DEFAULT
    };
    let wide = Rect {
        w: 1600.0, // LITERAL-PX-OK: a janela do teste
        ..VIEWPORT
    };
    let medidos = host.medindo_a_pintura_do_registo_com_bandas(&mut painel, wide, bandas);
    let dock = Rect { w, ..VIEWPORT };
    let rects = host.paint::<InspectorPanel>(&mut st, dock);
    ph2d_panel_inspector::set_current_inspector_transform(None);
    clear();
    let label = ph2d_i18n::tr("panel.inspector.sprite_sheet.show_sheet_on_canvas");
    let seen: Vec<_> = medidos.iter().filter(|m| m.texto == label).collect();
    assert!(
        !seen.is_empty(),
        "«{label}» não foi pintado — o gate não mediria nada"
    );
    assert!(
        seen.iter().all(|m| m.coube()),
        "«{label}» saiu cortado a {w} px: {:?}",
        seen.iter()
            .map(|m| (&m.pintado, m.largura))
            .collect::<Vec<_>>()
    );
    let hint = ph2d_i18n::tr("panel.inspector.anim_card.character_hint");
    assert!(
        !medidos.iter().any(|m| m.texto == hint && !m.coube()),
        "«{hint}» saiu elidido numa linha"
    );
    // A frase mora entre «Who Picks» e o 1.º menu de estado, e ocupa as linhas que pede.
    let at = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
    };
    let chooser = at(ids::INSP_ANIM_CHOOSER[2]);
    let first_state = at(ids::INSP_ANIM_STATE_PICK[0]);
    let font = ph2d_tokens::TypeToken::Sm.px();
    assert!(
        first_state.y - (chooser.y + chooser.h) >= 2.0 * font, // LITERAL-PX-OK: duas linhas
        "a frase tinha de quebrar em mais de uma linha a {} px",
        dock.w
    );
}

/// ⭐ Report do dono: o editor escolhia outra animação (ou criava uma pelo «+») e o cartão continuava
/// a realçar «idle». Quando a animação no ecrã muda, a linha aberta do cartão vai para ela.
#[test]
fn a_linha_aberta_do_cartao_segue_a_animacao_no_ecra() {
    let (mut host, mut st) = host_with(anim(false, false));
    assert_eq!(st.anim_selected, 0, "a fixtura abre em «walk»");
    let mut moved = anim(false, false);
    moved.current = "attack".into();
    set_current_inspector_anim(Some(moved));
    let _ = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    assert_eq!(st.anim_selected, 1, "o editor pôs «attack» no ecrã");
}
