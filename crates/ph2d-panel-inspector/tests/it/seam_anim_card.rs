//! ⭐⭐⭐ **O cartão «Animation» está VIVO sob o dedo** (plano GameEngineFuncional 07).
//!
//! Cada controlo novo é dirigido pelo ponteiro REAL (o `click_at` do testkit cai no rectângulo que a
//! pintura registou) e afirma a `AnimFieldEdit` que levanta: o ▶ da linha, «Edit Animations…»,
//! «Who Picks», a lista de regras (os menus «Play» são exactamente as animações da lista, e seguem um
//! renomear — os outros gestos das regras estão no `seam_anim_rules`),
//! «Run From», «Flip», «Playback», a animação escolhida e o grupo «Advanced» que nasce FECHADO. E as duas saídas: o Platform Player já não
//! pinta o «Auto Animate», e não há secção «Sprite Sheet».

use crate::seam_anim::{anim, clear, click, click_fresh, commit, host_with};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::screens::hero::{
    AnimCharacterInfo, AnimChooser, AnimFieldEdit, AnimRuleInfo, AnimWhenInfo, InspectorAnimInfo,
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

pub(crate) fn edits(actions: &[EditorAction]) -> Vec<AnimFieldEdit> {
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

/// A fixtura de um personagem no modo «Rules», com corpo: as 4 regras derivadas (parado · a correr ·
/// a subir · a cair) apontam para as duas animações, e o mapa de teclas tem `left` e `down`.
pub(crate) fn with_character(mut info: InspectorAnimInfo) -> InspectorAnimInfo {
    info.card.chooser = AnimChooser::Rules;
    info.card.has_character_body = true;
    info.card.character = Some(AnimCharacterInfo {
        run_from: 0.75,
        flip: true,
    });
    info.card.rules = ["walk", "attack", "walk", "attack"]
        .iter()
        .zip(0u8..)
        .map(|(play, i)| AnimRuleInfo {
            when: AnimWhenInfo::Pose(i),
            play: (*play).into(),
        })
        .collect();
    info.card.rule_actions = vec!["left".into(), "down".into()];
    info
}

/// Abre o menu `menu` da regra `rule` e clica na linha `opt` do popover; devolve as edições.
pub(crate) fn pick_rule(
    info: InspectorAnimInfo,
    menu: &[ph2d_a11y::NodeId],
    opts: &[ph2d_a11y::NodeId],
    rule: usize,
    opt: usize,
) -> Vec<EditorAction> {
    let (mut host, mut st) = host_with(info);
    host.set_dropdown_open(menu[rule], true);
    let out = click(&mut host, &mut st, opts[opt]);
    assert_eq!(
        host.dropdown_is_open(menu[rule]),
        Some(false),
        "escolher uma opção tem de fechar o menu"
    );
    clear();
    out
}

/// O menu «Play» da regra `rule`, opção `opt`.
fn pick_play(info: InspectorAnimInfo, rule: usize, opt: usize) -> Vec<EditorAction> {
    pick_rule(
        info,
        &ids::INSP_ANIM_RULE_PLAY,
        &ids::INSP_ANIM_RULE_PLAY_OPT,
        rule,
        opt,
    )
}

/// Os gestos do modo «Rules», para a varredura do `seam_anim`.
pub(crate) fn character_gestures() -> Vec<Vec<EditorAction>> {
    let mut out = vec![
        click_fresh(with_character(anim(true, false)), ids::INSP_ANIM_FLIP),
        commit(
            with_character(anim(true, false)),
            ids::INSP_ANIM_RUN_FROM,
            1.5,
        ),
        pick_play(with_character(anim(true, false)), 1, 0),
    ];
    out.extend(crate::seam_anim_rules::rule_gestures());
    out
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

/// ⭐ «Rules» vale sem Platform Player (teclas, factos de vida, sinais — plano 08).
#[test]
fn as_regras_oferecem_se_sem_platform_player() {
    let info = anim(false, false);
    assert!(!info.card.has_character_body);
    assert_eq!(
        edits(&click_fresh(info, ids::INSP_ANIM_CHOOSER[2])),
        [AnimFieldEdit::Chooser(AnimChooser::Rules.tag())]
    );
}

/// As regras só existem no modo «Rules».
#[test]
fn os_menus_das_regras_so_no_modo_rules() {
    for (info, want) in [
        (anim(false, false), false),
        (with_character(anim(false, false)), true),
    ] {
        let (mut host, mut st) = host_with(info);
        let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
        clear();
        for id in ids::INSP_ANIM_RULE_WHEN[..4]
            .iter()
            .chain(&ids::INSP_ANIM_RULE_PLAY[..4])
            .copied()
            .chain([
                ids::INSP_ANIM_RUN_FROM,
                ids::INSP_ANIM_FLIP,
                ids::INSP_ANIM_RULE_ADD,
            ])
        {
            assert_eq!(rects.iter().any(|(n, _)| *n == id), want, "{id:?}");
        }
    }
}

/// ⭐⭐ **O menu «Play» lista EXACTAMENTE as animações e segue um renomear** — escolher a 2.ª opção
/// escreve o nome dela; com a lista renomeada, a mesma opção escreve o nome NOVO.
#[test]
fn o_menu_play_lista_as_animacoes_e_segue_o_renomear() {
    let info = with_character(anim(false, false));
    let (mut host, mut st) = host_with(info.clone());
    host.set_dropdown_open(ids::INSP_ANIM_RULE_PLAY[0], true);
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    for (i, &id) in ids::INSP_ANIM_RULE_PLAY_OPT.iter().enumerate() {
        assert_eq!(
            rects.iter().any(|(n, _)| *n == id),
            i < info.rows.len(),
            "opção {i}: o menu tem de ter uma linha por animação, e nenhuma a mais"
        );
    }
    assert_eq!(
        edits(&pick_play(info.clone(), 2, 1)),
        [AnimFieldEdit::RulePlay(2, "attack".into())]
    );
    let mut renamed = info;
    renamed.rows[1].name = "golpe".into();
    for r in &mut renamed.card.rules {
        if r.play == "attack" {
            r.play = "golpe".into();
        }
    }
    assert_eq!(
        edits(&pick_play(renamed, 2, 1)),
        [AnimFieldEdit::RulePlay(2, "golpe".into())]
    );
}

#[test]
fn run_from_e_flip_escrevem_o_personagem() {
    // O «Run From» é uma velocidade do mundo: o campo mostra-a no seletor m/px (a porta `unidade_mundo`).
    let p = ph2d_editor_core::ProjectSettings::default();
    let no_ecra = ph2d_panel_inspector::unidade_mundo::guardada(ids::INSP_ANIM_RUN_FROM)
        .expect("de mundo")
        .no_ecra(1.5, p.display_unit, p.pixels_per_meter);
    assert_eq!(
        edits(&commit(
            with_character(anim(false, false)),
            ids::INSP_ANIM_RUN_FROM,
            no_ecra
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

/// Pinta a fixtura num host novo (com «Advanced» como o store o deixou) e devolve os rects.
fn painted(info: InspectorAnimInfo) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_anim(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    rects
}

fn has(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> bool {
    rects.iter().any(|(n, _)| *n == id)
}

/// ⭐ **«Advanced» nasce FECHADO, e «Playback» está à vista com ele fechado.** Abri-lo pelo cabeçalho
/// pinta os campos finos (Hold, a direcção fina, Repeat count), os sinais e as substituições do tocador.
#[test]
fn advanced_nasce_fechado_e_playback_esta_a_vista() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_anim(Some(anim(true, false)));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(has(&rects, ids::INSP_ANIM_ADVANCED));
    for id in [
        ids::INSP_ANIM_REWIND,
        ids::INSP_ANIM_PLAYING,
        ids::INSP_ANIM_FRAME_SCRUB,
        ids::INSP_ANIM_SPEED,
    ] {
        assert!(
            has(&rects, id),
            "«Playback» sem {id:?} com «Advanced» fechado"
        );
    }
    let fine = [
        ids::INSP_ANIM_HOLD_MS,
        ids::INSP_ANIM_DELAY_MS,
        ids::INSP_ANIM_REPEAT,
        ids::INSP_ANIM_DIR[0],
        ids::INSP_ANIM_SIGNAL_FINISH,
        ids::INSP_ANIM_SIGNAL_LOOP,
        ids::INSP_ANIM_DIR_OVERRIDE[0],
        ids::INSP_ANIM_LOOP_OVERRIDE[0],
    ];
    for id in fine {
        assert!(!has(&rects, id), "{id:?} pintado com «Advanced» fechado");
    }
    let _ = click(&mut host, &mut st, ids::INSP_ANIM_ADVANCED);
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clear();
    for id in fine {
        assert!(
            has(&rects, id),
            "abrir «Advanced» não pintou {id:?} — o grupo não está marcado dobrável ou perdeu-o"
        );
    }
}

/// ⭐ **⏸ só na linha que está no ecrã E a toca**, e clicá-lo PAUSA; ▶ na mesma linha parada RETOMA
/// (sem `SetCurrent` nem `Rewind`); ▶ noutra linha escolhe e toca.
#[test]
fn o_pause_mora_na_linha_que_toca_e_pausa_sem_rebobinar() {
    // `walk` (linha 0) está no ecrã a tocar.
    assert_eq!(
        edits(&click_fresh(anim(true, false), ids::INSP_ANIM_PLAY_ROW[0])),
        [AnimFieldEdit::Playing(false)]
    );
    assert_eq!(
        edits(&click_fresh(anim(false, false), ids::INSP_ANIM_PLAY_ROW[0])),
        [AnimFieldEdit::Playing(true)],
        "▶ na animação parada no ecrã retoma de onde parou — sem escolher de novo nem rebobinar"
    );
    assert_eq!(
        edits(&click_fresh(anim(true, false), ids::INSP_ANIM_PLAY_ROW[1])),
        [
            AnimFieldEdit::SetCurrent("attack".into()),
            AnimFieldEdit::Playing(true)
        ]
    );
    // E o ▶/⏸ da fileira «Playback» pergunta à cena.
    assert_eq!(
        edits(&click_fresh(anim(true, false), ids::INSP_ANIM_PLAYING)),
        [AnimFieldEdit::Playing(false)]
    );
}

/// ⭐ **O ⏸ PINTA-SE numa só linha.** Entre «a tocar» e «parada» só os ícones mudam: o ▶/⏸ de
/// «Playback» (um ícone, `d`) e o da linha no ecrã. Com a animação no ecrã fora da lista nenhuma
/// linha muda e a diferença é `d`; com ela na lista é `2d` — nem zero linhas, nem todas.
#[test]
fn o_pause_pinta_se_so_na_linha_que_toca() {
    let delta = |current: &str, rows: usize| -> i64 {
        let segs = |playing: bool| {
            let mut info = anim(playing, false);
            info.current = current.into();
            let extra = info.rows[1].clone();
            while info.rows.len() < rows {
                let mut r = extra.clone();
                r.name = format!("extra{}", info.rows.len());
                info.rows.push(r);
            }
            let mut host = MockPanelHost::with_panel::<InspectorPanel>();
            let mut st = InspectorState::default();
            set_current_inspector_anim(Some(info));
            let (_, segs) = host.paint_and_count_geometry::<InspectorPanel>(&mut st, VIEWPORT);
            clear();
            i64::from(segs)
        };
        segs(true) - segs(false)
    };
    let d = delta("ghost", 2);
    assert_ne!(d, 0, "o ▶/⏸ de «Playback» não mudou de ícone");
    assert_eq!(
        delta("walk", 2),
        2 * d,
        "a linha que toca não pintou ⏸ (ou pintou-o mais de uma)"
    );
    assert_eq!(delta("walk", 3), 2 * d, "mais linhas, o mesmo ⏸ único");
}

/// ⛔ **From/To só numa animação SEM lista de quadros** (report do dono: «From 0 To 31» numa animação
/// de 5 quadros). Com lista, a linha «N frames chosen in the editor» e nada de From/To.
#[test]
fn from_e_to_so_numa_animacao_sem_lista() {
    let range = painted(anim(false, false));
    assert!(has(&range, ids::INSP_ANIM_FROM) && has(&range, ids::INSP_ANIM_TO));
    let list = painted(with_frame_list(anim(false, false)));
    assert!(
        !has(&list, ids::INSP_ANIM_FROM),
        "From pintado sobre uma lista de quadros"
    );
    assert!(
        !has(&list, ids::INSP_ANIM_TO),
        "To pintado sobre uma lista de quadros"
    );
    assert!(has(&list, ids::INSP_ANIM_FPS) && has(&list, ids::INSP_ANIM_CYCLE[0]));
}

/// ⭐ **O ritmo é em QUADROS POR SEGUNDO** (a leitura do editor): o campo mostra `1000 / frame_ms` e
/// escrever `x` levanta `FrameMs(frame_ms_of_fps(x))` — a lei que `the_fps_law_is_the_engines` prende
/// à do motor.
#[test]
fn o_campo_de_quadros_por_segundo_escreve_o_frame_ms() {
    use ph2d_editor_core::screens::hero::InspectorAnimRow as R;
    for fps in [12.0, 24.0, 7.5] {
        assert_eq!(
            edits(&commit(anim(false, false), ids::INSP_ANIM_FPS, fps)),
            [AnimFieldEdit::FrameMs(0, R::frame_ms_of_fps(fps as f32))],
            "{fps} fps"
        );
    }
    assert_eq!(R::frame_ms_of_fps(12.0), 83);
    assert!(
        edits(&commit(anim(false, false), ids::INSP_ANIM_FPS, 0.0)).is_empty(),
        "zero quadros por segundo não é um ritmo"
    );
    let (host, _st) = host_with(anim(false, false));
    let shown = host.store().number_value(ids::INSP_ANIM_FPS);
    clear();
    assert_eq!(shown, Some(11.1), "walk tem 90 ms por quadro = 11,1 fps");
}

/// ⭐ **O ciclo é Repeat · Once · Ping-pong**: a POSIÇÃO do segmento é o `AnimCycleMode::tag`, e o
/// aceso é o `of_row` da animação (attack = vai-e-volta).
#[test]
fn o_ciclo_segmentado_escreve_o_modo() {
    use ph2d_editor_core::screens::hero::AnimCycleMode;
    for (i, m) in AnimCycleMode::ALL.iter().enumerate() {
        assert_eq!(
            edits(&click_fresh(anim(false, false), ids::INSP_ANIM_CYCLE[i])),
            [AnimFieldEdit::CycleMode(0, m.tag())],
            "{m:?}"
        );
    }
}

/// ⛔ «per-frame timing (imported)» mentia sobre tempos dados no editor: a frase diz só o facto, e só
/// quando ele é verdade; e «This frame» está SEMPRE pintado (apagado sem tempo próprio — ver
/// `seam_anim_this_frame`).
#[test]
fn o_tempo_proprio_diz_se_so_quando_existe() {
    let plain = painted(anim(false, false));
    assert!(
        has(&plain, ids::INSP_ANIM_FRAME_MS_THIS),
        "a linha não some"
    );
    let mut own = anim(false, false);
    own.rows[0].per_frame_ms = vec![0, 250, 0, 0];
    assert!(
        has(&painted(own), ids::INSP_ANIM_FRAME_MS_THIS),
        "o quadro 2 tem 250 ms"
    );
    let text = ph2d_i18n::tr("panel.inspector.anim_card.own_times");
    assert!(!text.contains("imported"), "{text}");
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

/// ⛔ **Não há secção «Sprite Sheet»** — e a grelha tem UM lugar: sob o seletor «Use», quando a imagem é
/// animada (plano GEF 15 D2); uma parada escolhe o quadro na miniatura e não mostra o corte.
#[test]
fn nao_ha_seccao_sprite_sheet_e_a_grelha_tem_um_so_lugar() {
    for anim_info in [None, Some(anim(false, false))] {
        let with_anim = anim_info.is_some();
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut st = InspectorState::default();
        crate::seam_image_use::publish_visibility(0xABCD_1234);
        ph2d_panel_inspector::set_current_inspector_sprite(Some(
            ph2d_editor_core::screens::hero::InspectorSpriteInfo {
                entity_bits: 0xABCD_1234,
                image_use: if with_anim {
                    ph2d_editor_core::InspectorImageUse::Animated
                } else {
                    ph2d_editor_core::InspectorImageUse::Still
                },
                ..crate::seam_render_source::atlas_sprite(false)
            },
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
                usize::from(with_anim),
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
    // A grelha vive sob o seletor «Use» de uma sprite animada (plano GEF 15 D2).
    crate::seam_image_use::publish_visibility(0xABCD_1234);
    ph2d_panel_inspector::set_current_inspector_sprite(Some(
        ph2d_editor_core::screens::hero::InspectorSpriteInfo {
            entity_bits: 0xABCD_1234,
            image_use: ph2d_editor_core::InspectorImageUse::Animated,
            hframes: 4,
            vframes: 2,
            ..crate::seam_render_source::atlas_sprite(false)
        },
    ));
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
    ph2d_panel_inspector::set_current_inspector_sprite(None);
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
    let hint = ph2d_i18n::tr("panel.inspector.anim_card.rules_hint");
    assert!(
        !medidos.iter().any(|m| m.texto == hint && !m.coube()),
        "«{hint}» saiu elidido numa linha"
    );
    // A frase mora entre «Who Picks» e a 1.ª regra, e ocupa as linhas que pede.
    let at = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
    };
    let chooser = at(ids::INSP_ANIM_CHOOSER[2]);
    let first_state = at(ids::INSP_ANIM_RULE_GRIP[0]);
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
