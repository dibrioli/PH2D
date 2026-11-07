//! ⭐ **A costura do cartão Animation** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)):
//! cada gesto entra como ponteiro no que está pintado e sai como pedido no barramento.

use crate::o_inspector_em_cartoes_por_tipo::{BITS, limpa, objeto, pinta, sprite};
use crate::seam_anim::anim;
use crate::seam_anim_card::edits;
use ph2d_a11y::NodeId;
use ph2d_editor_core::InspectorImageUse;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AnimFieldEdit, InspectorAnimInfo};
use ph2d_editor_core::widget::key_mark;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    CARD_LOOK_ANIMADA, CARD_ORDERING, InspectorPanel, InspectorState, cabeca_pintada, set_cards_on,
    set_current_inspector_anim, set_current_inspector_sprite,
};
use ph2d_ui_testkit::MockPanelHost;

fn fixa() -> InspectorAnimInfo {
    let mut a = anim(true, true);
    a.entity_bits = BITS;
    a
}

fn cena_com(a: InspectorAnimInfo) -> (MockPanelHost, InspectorState) {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_anim(Some(a));
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Animated)));
    (
        MockPanelHost::with_panel::<InspectorPanel>(),
        InspectorState::default(),
    )
}

fn cena() -> (MockPanelHost, InspectorState) {
    cena_com(fixa())
}

fn rect_de(r: &[(NodeId, Rect)], id: NodeId) -> Rect {
    r.iter()
        .rev()
        .find(|(i, q)| *i == id && q.w > 0.0)
        .map(|(_, q)| *q)
        .unwrap_or_else(|| panic!("{id:?} não pintou"))
}

fn pintou(r: &[(NodeId, Rect)], id: NodeId) -> bool {
    r.iter().any(|(i, q)| *i == id && q.w > 0.0)
}

fn ponteiro(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    r: Rect,
) -> (Vec<WidgetEvent>, Vec<EditorAction>) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut eventos = Vec::new();
    for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
        eventos.extend(host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        }));
    }
    for ev in &eventos {
        let _ = host.apply_panel_event::<InspectorPanel>(st, *ev);
    }
    (eventos, host.drained_actions())
}

fn clica(host: &mut MockPanelHost, st: &mut InspectorState, id: NodeId) -> Vec<AnimFieldEdit> {
    let r = rect_de(&pinta(host, st), id);
    let _ = host.drained_actions();
    edits(&ponteiro(host, st, r).1)
}

/// O que saiu do cartão não pinta em nenhum estado, nem com More aberto; o editor é quem os cobre.
/// *Mutação: tirar o `cards::active()` do `InspectorImageUse::Animated` ⇒ H/V voltam e fica vermelho.*
#[test]
fn o_corte_e_o_nome_sairam_do_cartao() {
    let (mut host, mut st) = cena();
    host.store_mut().set_collapsed(INSP_ANIM_ADVANCED, false);
    let r = pinta(&mut host, &mut st);
    for id in [
        INSP_ANIM_FROM,
        INSP_ANIM_TO,
        INSP_ANIM_FRAME_MS_THIS,
        INSP_ANIM_REWIND,
        INSP_ANIM_NAME,
        INSP_SPRITE_HFRAMES,
        INSP_SPRITE_VFRAMES,
    ] {
        assert!(!pintou(&r, id), "{id:?} continua no cartão");
    }
    for id in [INSP_ANIM_HOLD_MS, INSP_ANIM_DELAY_MS, INSP_ANIM_REPEAT] {
        assert!(pintou(&r, id), "{id:?} não está no More aberto");
    }
    limpa();
}

/// Add, Remove e Edit Animations são ícones do cabeçalho, e o clique real faz o que os botões largos faziam.
/// *Mutação: tirar um verbo de `cards_anim::proprio` ⇒ «não pintou».*
#[test]
fn os_verbos_moram_no_cabecalho() {
    let (mut host, mut st) = cena();
    let r = pinta(&mut host, &mut st);
    let linha = rect_de(&r, INSP_ANIM_ROW[0]);
    let add = rect_de(&r, INSP_ANIM_ADD);
    let (del, edit) = (
        rect_de(&r, INSP_ANIM_REMOVE),
        rect_de(&r, INSP_ANIM_OPEN_EDITOR),
    );
    assert!(add.y < linha.y && del.y == add.y && edit.y == add.y);
    assert!(add.x < del.x && del.x < edit.x, "Add · Remove · Edit");
    assert_eq!(
        clica(&mut host, &mut st, INSP_ANIM_ADD),
        [AnimFieldEdit::Add]
    );
    assert_eq!(
        clica(&mut host, &mut st, INSP_ANIM_REMOVE),
        [AnimFieldEdit::Remove(0)]
    );
    assert_eq!(
        clica(&mut host, &mut st, INSP_ANIM_OPEN_EDITOR),
        [AnimFieldEdit::OpenEditor]
    );
    limpa();
}

/// Sem animação na lista não há o que remover.
#[test]
fn sem_animacoes_nao_ha_remove() {
    let mut a = fixa();
    a.rows.clear();
    let (mut host, mut st) = cena_com(a);
    let r = pinta(&mut host, &mut st);
    assert!(pintou(&r, INSP_ANIM_ADD) && !pintou(&r, INSP_ANIM_REMOVE));
    limpa();
}

/// Frames/s e Speed dividem a linha, com o Cycle abaixo; a Speed continua a ser o canal do catálogo.
/// *Mutação: pintar uma linha entre os dois ⇒ os `y` diferem.*
#[test]
fn frames_por_segundo_e_speed_na_mesma_linha() {
    let (mut host, mut st) = cena();
    let r = pinta(&mut host, &mut st);
    let (fps, speed) = (rect_de(&r, INSP_ANIM_FPS), rect_de(&r, INSP_ANIM_SPEED));
    assert_eq!(fps.y, speed.y, "{fps:?} · {speed:?}");
    assert!(fps.x + fps.w <= speed.x, "{fps:?} · {speed:?}");
    assert!(rect_de(&r, INSP_ANIM_CYCLE[0]).y > speed.y);
    assert_eq!(
        key_mark::rotulo_de(INSP_ANIM_SPEED),
        Some("component.field.sprite_animator.3")
    );
    limpa();
}

/// Dois cliques na linha abrem o campo sobre o nome; digitar não grava; o Enter grava UM renomear.
/// *Mutação: o `TextChanged` passar a gravar ⇒ dois renomeios no barramento.*
#[test]
fn dois_cliques_renomeiam_e_o_enter_grava() {
    let (mut host, mut st) = cena();
    let linha = rect_de(&pinta(&mut host, &mut st), INSP_ANIM_ROW[1]);
    let (_, um) = ponteiro(&mut host, &mut st, linha);
    assert!(matches!(&edits(&um)[..], [AnimFieldEdit::SetCurrent(n)] if n == "attack"));
    let (ev, dois) = ponteiro(&mut host, &mut st, linha);
    assert!(
        ev.contains(&WidgetEvent::DoubleClick(INSP_ANIM_ROW[1])),
        "{ev:?}"
    );
    assert!(edits(&dois).is_empty(), "abrir o campo não grava: {dois:?}");
    let r = pinta(&mut host, &mut st);
    let campo = rect_de(&r, INSP_ANIM_NAME);
    assert_eq!(campo.y, rect_de(&r, INSP_ANIM_ROW[1]).y);
    assert_eq!(host.store().text(INSP_ANIM_NAME), Some("attack"));
    host.set_text(INSP_ANIM_NAME, "ataque");
    let _ =
        host.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::TextChanged(INSP_ANIM_NAME));
    assert!(edits(&host.drained_actions()).is_empty(), "digitar grava");
    let _ = host.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Submit(INSP_ANIM_NAME));
    assert_eq!(
        edits(&host.drained_actions()),
        [AnimFieldEdit::Rename(1, "ataque".into())]
    );
    assert!(
        !pintou(&pinta(&mut host, &mut st), INSP_ANIM_NAME),
        "o campo fecha"
    );
    limpa();
}

/// O Esc desiste, e clicar fora grava como no editor.
#[test]
fn esc_desiste_e_clicar_fora_grava() {
    let (mut host, mut st) = cena();
    let linha = rect_de(&pinta(&mut host, &mut st), INSP_ANIM_ROW[1]);
    let _ = ponteiro(&mut host, &mut st, linha);
    let _ = ponteiro(&mut host, &mut st, linha);
    host.set_text(INSP_ANIM_NAME, "nada");
    let _ = host.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Cancel(INSP_ANIM_NAME));
    let _ = pinta(&mut host, &mut st);
    assert!(
        edits(&host.drained_actions())
            .iter()
            .all(|e| !matches!(e, AnimFieldEdit::Rename(..)))
    );

    let _ = ponteiro(&mut host, &mut st, linha);
    let _ = ponteiro(&mut host, &mut st, linha);
    let _ = host.drained_actions();
    host.set_text(INSP_ANIM_NAME, "ataque");
    host.store_mut().set_focus(None);
    let _ = pinta(&mut host, &mut st);
    assert_eq!(
        edits(&host.drained_actions()),
        [AnimFieldEdit::Rename(1, "ataque".into())]
    );
    limpa();
}

/// A linha da miniatura diz o corte e, no piso do painel, nada se corta.
/// *Mutação: tirar `formas_com_o_corte` ⇒ a linha perde o `· 4 × 1`.*
#[test]
fn a_moldura_diz_o_corte_sem_cortar_no_piso() {
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_editor_core::widget::card_censo;
    let piso = ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN;
    for largura in [piso, 240.0, 320.0] {
        let (mut host, mut st) = cena();
        let _ = pinta_a_largura(&mut host, &mut st, largura);
        let ((_, medidos), censo) =
            card_censo::grava(|| elisao::medindo(|| pinta_a_largura(&mut host, &mut st, largura)));
        let cortados: std::collections::BTreeSet<_> = medidos
            .iter()
            .filter(|m| !m.coube())
            .map(|m| m.texto.clone())
            .collect();
        assert!(cortados.is_empty(), "a {largura}: {cortados:?}");
        assert_eq!(censo.textos_cortados(), Vec::<String>::new(), "a {largura}");
        assert!(cabeca_pintada().and_then(|c| c.tamanho).is_some());
    }
    limpa();
}

fn pinta_a_largura(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    largura: f32,
) -> Vec<(NodeId, Rect)> {
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1600.0, 2400.0);
    let bandas = ChromeBands {
        right_dock_w: largura,
        ..ChromeBands::DEFAULT
    };
    host.settle_section_folds();
    let mut r = Vec::new();
    for _ in 0..60 {
        let layout = HeroLayout::for_viewport_bands(
            janela,
            false,
            bandas,
            CenterSplit::None,
            DockSides::BOTH,
        );
        r = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || host.paint_with_layout::<InspectorPanel>(st, layout, janela),
        );
    }
    r
}

/// O Look nasce dobrado em Animated e aberto em Still; quem já abriu o Look continua com ele aberto.
/// *Mutação: semear a dobra do Look da Sprite (`INSP_LIVE_COLOR_SECTION`) ⇒ a Still dobra.*
#[test]
fn o_look_nasce_dobrado_so_em_animated() {
    let (mut host, mut st) = cena();
    let r = pinta(&mut host, &mut st);
    assert!(!pintou(&r, INSP_SPRITE_TINT_SWATCH), "Animated: dobrado");
    assert!(host.store().is_collapsed(CARD_LOOK_ANIMADA));

    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let r = pinta(&mut host, &mut st);
    assert!(pintou(&r, INSP_SPRITE_TINT_SWATCH), "Still: como antes");

    let (mut host, mut st) = cena();
    host.store_mut()
        .set_collapsed(core_ids::INSP_LIVE_COLOR_SECTION, false);
    let r = pinta(&mut host, &mut st);
    assert!(
        pintou(&r, INSP_SPRITE_TINT_SWATCH),
        "a escolha do artista fica"
    );
    limpa();
}

/// Os campos em uso, como no censo da Sprite: um grupo é um campo; o que pinta e não é campo é a moldura.
fn campos_do_animated() -> Vec<Vec<NodeId>> {
    let um = |v: &[NodeId]| v.iter().map(|id| vec![*id]).collect::<Vec<_>>();
    let mut c = crate::o_cartao_da_sprite::campos_da_sprite();
    c.extend(um(&[
        INSP_ANIM_ADD,
        INSP_ANIM_REMOVE,
        INSP_ANIM_OPEN_EDITOR,
        INSP_ANIM_PLAYING,
        INSP_ANIM_FRAME_SCRUB,
        INSP_ANIM_FPS,
        INSP_ANIM_SPEED,
        INSP_ANIM_HOLD_MS,
        INSP_ANIM_DELAY_MS,
        INSP_ANIM_REPEAT,
        INSP_ANIM_SIGNAL_FINISH,
        INSP_ANIM_SIGNAL_LOOP,
        INSP_ANIM_MARKER_ADD,
        INSP_ANIM_START_DD,
    ]));
    c.push(INSP_ANIM_ROW.to_vec());
    c.push(INSP_ANIM_PLAY_ROW.to_vec());
    c.push(INSP_ANIM_CYCLE.to_vec());
    c.push(INSP_ANIM_CHOOSER.to_vec());
    c
}

/// ⭐ A aceitação da ficha, contada no que está pintado (o método de `o_censo_dos_campos_da_sprite_em_uso`): à vista
/// com os outros cartões dobrados, e o total com tudo aberto. Meta do papel: 15 e 47, sem regras. A moldura muda com
/// quem acrescenta um campo.
#[test]
fn o_censo_dos_campos_do_animated_em_uso() {
    let (mut host, mut st) = cena();
    let conta = |r: &[(NodeId, Rect)]| {
        let tem = |id: &NodeId| r.iter().any(|(i, q)| i == id && q.w > 0.0);
        let campos = campos_do_animated();
        let vistos = campos.iter().filter(|g| g.iter().any(tem)).count();
        let moldura = r
            .iter()
            .filter(|(i, q)| q.w > 0.0 && !campos.iter().flatten().any(|c| c == i))
            .count();
        (vistos, moldura)
    };
    for id in [core_ids::INSP_LIVE_RENDER_SECTION, CARD_ORDERING] {
        host.store_mut().set_collapsed(id, true);
    }
    assert_eq!(conta(&pinta(&mut host, &mut st)), (17, 22), "à vista");
    for id in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        CARD_ORDERING,
        CARD_LOOK_ANIMADA,
        INSP_ANIM_ADVANCED,
        ph2d_panel_inspector::CARD_TRANSFORM_IMAGE,
    ] {
        host.store_mut().set_collapsed(id, false);
    }
    assert_eq!(conta(&pinta(&mut host, &mut st)), (48, 22), "no total");
    limpa();
}

fn com_inicio() -> InspectorAnimInfo {
    let mut a = fixa();
    a.card.starts_with = "walk".into();
    a
}

/// ⭐ O menu: a caixa está abaixo do Auto e o ponteiro real a clica; a lista abre pela porta do hero (`on_click`) com a
/// animação da caixa, e a linha dela escreve só o início. As opções pinta-as o hero por cima de tudo (a mesa não tem
/// esse trecho): aqui entra o clique dela, como no Ordering.
/// *Mutação: tirar o braço de `INSP_ANIM_START_OPT` no despacho ⇒ o clique não sai como edição.*
#[test]
fn o_menu_starts_with_abre_e_escolhe_pelo_ponteiro() {
    use ph2d_editor_core::panel::cards::dropdown;
    let (mut host, mut st) = cena_com(com_inicio());
    let r = pinta(&mut host, &mut st);
    let caixa = rect_de(&r, INSP_ANIM_START_DD);
    assert!(
        caixa.y > rect_de(&r, INSP_ANIM_CHOOSER[0]).y,
        "abaixo do Auto"
    );
    let (ev, _) = ponteiro(&mut host, &mut st, caixa);
    assert!(
        ev.contains(&WidgetEvent::Click(INSP_ANIM_START_DD)),
        "{ev:?}"
    );
    assert!(
        dropdown::on_click(INSP_ANIM_START_DD),
        "a caixa é uma lista declarada"
    );
    assert_eq!(dropdown::open(), Some(INSP_ANIM_START_DD));
    let _ = pinta(&mut host, &mut st);
    let _ = host.drained_actions();
    let _ = host
        .apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Click(INSP_ANIM_START_OPT[1]));
    assert_eq!(
        edits(&host.drained_actions()),
        [AnimFieldEdit::StartsWith("attack".into())]
    );
    limpa();
}

/// O ▶ noutra linha é prévia: muda o que toca e não pede para mudar o início.
/// *Mutação: o ▶ emitir `StartsWith` ⇒ vermelho.*
#[test]
fn o_play_da_linha_nao_escreve_o_inicio() {
    let (mut host, mut st) = cena_com(com_inicio());
    let e = clica(&mut host, &mut st, INSP_ANIM_PLAY_ROW[1]);
    assert_eq!(
        e,
        [
            AnimFieldEdit::SetCurrent("attack".into()),
            AnimFieldEdit::Playing(true)
        ]
    );
    limpa();
}

/// Script e Rules não têm o menu; os três botões dizem Auto · Script · Rules.
#[test]
fn so_o_auto_tem_o_menu() {
    use ph2d_editor_core::screens::hero::AnimChooser;
    assert_eq!(ph2d_i18n::tr("panel.inspector.anim_card.me"), "Auto");
    for quem in [AnimChooser::Script, AnimChooser::Rules] {
        let mut a = com_inicio();
        a.card.chooser = quem;
        let (mut host, mut st) = cena_com(a);
        assert!(
            !pintou(&pinta(&mut host, &mut st), INSP_ANIM_START_DD),
            "{quem:?}"
        );
    }
    limpa();
}

fn com_more_aberto(host: &mut MockPanelHost) {
    host.store_mut().set_collapsed(INSP_ANIM_ADVANCED, false);
}

/// Os textos que a pintura mandou para o censo.
fn textos(host: &mut MockPanelHost, st: &mut InspectorState) -> Vec<String> {
    use ph2d_editor_core::panel::cards::censo;
    censo::grava(|| pinta(host, st))
        .1
        .pecas
        .iter()
        .map(|p| p.nome.clone())
        .collect()
}

/// ⭐ Os campos do marcador saíram de More: sobra o botão, e o clique abre o editor, com ou sem marcador; o texto conta.
/// *Mutação: tirar o `cards::active()` de `apply_markers_event` ⇒ o clique volta a criar marcador no cartão.*
#[test]
fn o_marcador_no_cartao_e_o_resumo_que_abre_o_editor() {
    let (mut host, mut st) = cena();
    com_more_aberto(&mut host);
    let r = pinta(&mut host, &mut st);
    assert!(pintou(&r, INSP_ANIM_MARKER_ADD));
    assert!(!pintou(&r, INSP_ANIM_MARKER_FRAME[0]));
    assert!(textos(&mut host, &mut st).iter().any(|t| t == "Add marker"));
    assert_eq!(
        clica(&mut host, &mut st, INSP_ANIM_MARKER_ADD),
        [AnimFieldEdit::OpenEditor]
    );
    limpa();

    let mut a = fixa();
    for step in [1, 3] {
        a.rows[0]
            .markers
            .push(ph2d_editor_core::screens::hero::InspectorAnimMarker {
                step,
                signal: "step".into(),
            });
    }
    let (mut host, mut st) = cena_com(a);
    com_more_aberto(&mut host);
    let r = pinta(&mut host, &mut st);
    assert!(!pintou(&r, INSP_ANIM_MARKER_FRAME[0]));
    assert!(!pintou(&r, INSP_ANIM_MARKER_SIGNAL[0]));
    assert!(!pintou(&r, INSP_ANIM_MARKER_DEL[0]));
    assert!(textos(&mut host, &mut st).iter().any(|t| t == "2 markers"));
    assert_eq!(
        clica(&mut host, &mut st, INSP_ANIM_MARKER_ADD),
        [AnimFieldEdit::OpenEditor]
    );
    limpa();
}

fn facts(f: ph2d_panel_inspector::slot_image::SpriteFacts) {
    ph2d_panel_inspector::slot_image::set_current_sprite_facts(Some(
        ph2d_panel_inspector::slot_image::SpriteFacts {
            entity_bits: BITS,
            ..f
        },
    ));
}

fn verbo_da_imagem(a: &[EditorAction]) -> Vec<ph2d_editor_core::ImageUseEdit> {
    a.iter()
        .filter_map(|x| match x {
            EditorAction::ImageUseEdit { edit, .. } => Some(*edit),
            _ => None,
        })
        .collect()
}

/// ⭐ O Animated vazio: o convite pede a folha, oferece o herói e não há cartão Animation; o herói sai como edição.
/// *Mutação: tirar o `continue` de `presentes` ⇒ a lista da animação pinta no vazio.*
#[test]
fn o_vazio_do_animated_pede_a_folha_e_oferece_o_heroi() {
    let (mut host, mut st) = cena();
    facts(ph2d_panel_inspector::slot_image::SpriteFacts {
        sem_imagem: true,
        ..Default::default()
    });
    let r = pinta(&mut host, &mut st);
    assert!(pintou(&r, INSP_IMAGE_SAMPLE_HERO));
    assert!(!pintou(&r, INSP_IMAGE_PAINT_NEW), "pintar é da Sprite");
    assert!(pintou(&r, INSP_RENDER_TEXTURE_SLOT), "o alvo da escolha");
    assert!(!pintou(&r, INSP_ANIM_ROW[0]), "sem Animation");
    assert!(!pintou(&r, INSP_ANIM_ADD));
    let caixa = rect_de(&r, INSP_IMAGE_SAMPLE_HERO);
    let _ = host.drained_actions();
    let (_, a) = ponteiro(&mut host, &mut st, caixa);
    assert_eq!(
        verbo_da_imagem(&a),
        [ph2d_editor_core::ImageUseEdit::SampleHero]
    );
    limpa();
}

/// ⭐ O aviso de imagem trocada: o texto da ficha, com os números que chegam, e as duas saídas pelo ponteiro.
#[test]
fn o_aviso_de_imagem_trocada_diz_os_numeros_e_da_as_duas_saidas() {
    let (mut host, mut st) = cena();
    let r = pinta(&mut host, &mut st);
    assert!(!pintou(&r, INSP_IMAGE_CUT_AGAIN), "sem aviso, sem botões");
    facts(ph2d_panel_inspector::slot_image::SpriteFacts {
        corte_velho: Some(ph2d_editor_core::ImageCutNotice {
            grid: [8, 4],
            cell_px: [32, 32],
            animations: 6,
        }),
        ..Default::default()
    });
    assert_eq!(
        ph2d_i18n::tr_with(
            "card.insp.new_image_old_cut",
            &[("c", &8), ("r", &4), ("w", &32), ("h", &32), ("n", &6)]
        ),
        "New image, old cut: 8 × 4 makes cells of 32 × 32 px, and these 6 animations were made for another image."
    );
    for (id, verbo) in [
        (
            INSP_IMAGE_CUT_AGAIN,
            ph2d_editor_core::ImageUseEdit::CutAgain,
        ),
        (INSP_IMAGE_KEEP_CUT, ph2d_editor_core::ImageUseEdit::KeepCut),
    ] {
        let r = pinta(&mut host, &mut st);
        let caixa = rect_de(&r, id);
        let _ = host.drained_actions();
        let (_, a) = ponteiro(&mut host, &mut st, caixa);
        assert_eq!(verbo_da_imagem(&a), [verbo]);
    }
    limpa();
}
