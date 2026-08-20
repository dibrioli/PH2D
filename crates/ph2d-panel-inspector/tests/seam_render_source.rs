//! **A COSTURA da seção Render Source** — plano [`docs/Sprite_projeto/17`] §8, W6.
//!
//! O plano dizia *"testes de costura de comportamento para os três botões de Strategy… hoje há
//! **zero**"*. ⚠️ **A nota estava meia certa, e a metade errada importa:** o despacho JÁ tem teste
//! (`screens::hero::tests::…strategy…` afirma que `apply_event(Click(id))` levanta o
//! `InspectorSpriteSourceChange` certo). O que não existia é o que vem ANTES dele — que a seção
//! **pinte** aqueles botões e os **registe no índice de acerto**, para que um clique real lhes
//! chegue.
//!
//! Essa distinção não é académica: foi exatamente por a ignorar que eu escrevi hoje quatro testes
//! verdes sobre um `crop_region` **desligado** do chamador. *Um teste que chama `apply_event`
//! diretamente prova o handler e não prova a porta.*
//!
//! Estes testes correm o caminho inteiro que a shell corre — popular → publicar o snapshot →
//! **pintar** → ler o rectângulo pintado → despachar o clique — e afirmam:
//!
//! 1. os três botões de Strategy são pintados **e** hit-registered;
//! 2. clicar em cada um levanta exatamente a ação daquele botão;
//! 3. clicar no que já está ativo **não** levanta nada (não é uma edição);
//! 4. o `Reimport` **desativado não age** — a metade que um botão pintado a cinzento mais mente.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel};
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource, RequestedSpriteStrategy,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0xC0FF_EE00;

/// Um sprite de ATLAS, que é como todo sprite importado nasce — e por isso o estado de partida
/// certo para perguntar *"consigo mudar de estratégia?"*.
fn atlas_sprite(can_reimport: bool) -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        entity_bits: ENTITY,
        name: "fruits".into(),
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        sheet_label: None,
        source_pixels: Some((256, 256)),
        can_reimport,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: 1,
        vframes: 1,
        frame: 0,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: false,
        region_rect: [0.0; 4],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }
}

/// Popula, publica o snapshot e **pinta** — devolvendo os rectângulos que o painel de facto
/// registou. É o `paint` que separa este ficheiro dos testes de handler.
fn painted(
    info: InspectorSpriteInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_sprite(Some(info));
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 2000.0));
    (host, state, rects)
}

fn dispatch(host: &mut MockPanelHost, state: &mut InspectorState, id: ph2d_a11y::NodeId) -> bool {
    matches!(
        InspectorPanel::apply_event(state, host, WidgetEvent::Click(id)),
        EventOutcome::Consumed | EventOutcome::Observed,
    )
}

/// **(1) Os três botões existem no ecrã.**
///
/// ⚠️ Um botão que a seção não pinta — ou pinta e não regista — é indistinguível de um botão
/// partido: o ponteiro nunca lhe chega, e todo teste de handler continua verde. É a forma que este
/// repositório já viu quatro vezes (os pills de vetor, o Undo/Redo da barra, o *Use as Brush
/// Shape*, e o `[SHEET]` de ontem).
#[test]
fn the_three_strategy_buttons_are_painted_and_hit_registered() {
    let (_host, _state, rects) = painted(atlas_sprite(true));
    for (label, id) in [
        ("Atlas", ids::INSP_RENDER_STRATEGY_ATLAS),
        ("Individual", ids::INSP_RENDER_STRATEGY_INDIVIDUAL),
        ("Hand-packed", ids::INSP_RENDER_STRATEGY_HANDPACKED),
    ] {
        let hit = rects.iter().find(|(rid, _)| *rid == id);
        let Some((_, r)) = hit else {
            panic!(
                "o botao de Strategy '{label}' nao foi pintado nem registado — o ponteiro nunca \
                 lhe chega, e os testes de handler continuam verdes"
            );
        };
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "'{label}' registou um rectangulo de area zero: inalcancavel na pratica"
        );
    }
}

/// **(2) Cada botão levanta a SUA ação** — comparada por variante, para que trocar dois braços do
/// despacho reprove.
#[test]
fn each_strategy_button_raises_its_own_change() {
    for (id, want) in [
        (
            ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            RequestedSpriteStrategy::Individual,
        ),
        (
            ids::INSP_RENDER_STRATEGY_HANDPACKED,
            RequestedSpriteStrategy::HandPacked,
        ),
    ] {
        let (mut host, mut state, _) = painted(atlas_sprite(true));
        assert!(dispatch(&mut host, &mut state, id));
        let drained: Vec<EditorAction> = host.drained_actions();
        assert_eq!(
            drained,
            vec![EditorAction::InspectorSpriteSourceChange {
                entity_bits: ENTITY,
                strategy: want,
            }],
            "o botao {id:?} nao levantou a mudanca de estrategia dele"
        );
    }
}

/// **(3) Clicar no que já está ativo não é uma edição.**
///
/// ⚠️ Sem isto, um radio "confirma" a si próprio e cada clique distraído vira um passo de undo
/// sobre uma cena que não mudou.
#[test]
fn clicking_the_current_strategy_changes_nothing() {
    let (mut host, mut state, _) = painted(atlas_sprite(true));
    assert!(
        dispatch(&mut host, &mut state, ids::INSP_RENDER_STRATEGY_ATLAS),
        "o clique e' consumido (o botao existe), mesmo nao havendo edicao"
    );
    assert!(
        host.drained_actions().is_empty(),
        "clicar na estrategia JA' ativa levantou uma edicao"
    );
}

/// **(4) O `Reimport` desativado não age.**
///
/// ⚠️ É a metade em que um botão a cinzento mais mente: ele é pintado, é hit-registered (para
/// mostrar o cursor e o tooltip), e a única coisa que o impede de agir é um `&&` no handler. Um
/// controle *"desativado"* que ainda despacha é pior que um ativo — o artista vê o estado dizer
/// «não dá» e a cena mudar na mesma.
#[test]
fn a_disabled_reimport_does_not_act() {
    let (mut host, mut state, _) = painted(atlas_sprite(false));
    dispatch(&mut host, &mut state, ids::INSP_RENDER_SOURCE_REIMPORT);
    assert!(
        host.drained_actions().is_empty(),
        "o Reimport desativado levantou uma acao"
    );

    // Controle positivo: com `can_reimport = true` ele TEM de agir — senão o teste acima passaria
    // por o botao estar morto para toda a gente.
    let (mut host, mut state, _) = painted(atlas_sprite(true));
    dispatch(&mut host, &mut state, ids::INSP_RENDER_SOURCE_REIMPORT);
    assert_eq!(
        host.drained_actions(),
        vec![EditorAction::Reimport {
            entity_bits: ENTITY
        }],
        "controle: com can_reimport o botao tem de agir"
    );
}
