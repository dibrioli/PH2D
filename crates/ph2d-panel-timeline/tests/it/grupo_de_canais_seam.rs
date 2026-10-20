//! **A linha de GRUPO de canais (plano 35 §5.1) é pintada, registada, e um clique REAL nela chega**:
//! o losango do grupo seleciona as chaves dos membros, o ▶ mostra as linhas dos membros.
//!
//! O clique vai pelo canal de gestos (`TimelineSurface`): Down+Up no rect pintado, depois uma segunda
//! pintura (que corre `interact::process`) — o caminho do app.

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::ids::{timeline_group_key_id, timeline_group_twirl_id, timeline_twirl_id};
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_timeline::{
    AnimTarget, Interp, KeyId, KeyView, PropKind, TimelineIntent, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn track(target: u64, prop: PropKind) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop,
        entity: 9,
        missing: false,
        keys: vec![KeyView {
            id: KeyId::new(target * 100),
            t_seconds: 0.0,
            value: 0.0,
            interp: Interp::Linear,
            selected: false,
            roving: false,
        }],
        buffer_ghost: None,
        pre: ph2d_timeline::Extrap::Hold,
        post: ph2d_timeline::Extrap::Hold,
        expr: None,
    }
}

/// Uma rotação e a cor chapada do preenchimento (4 canais), todos com chave em 0.
fn snap() -> TimelineViewSnapshot {
    let cor = |target, c| {
        track(
            target,
            PropKind::Field(FieldKey::new(
                VETOR,
                v::TINTA_PREENCH + v::OFF_SOLIDA + c,
                0,
                0,
            )),
        )
    };
    TimelineViewSnapshot {
        fps: 60.0,
        tracks: vec![
            track(1, PropKind::Rotation),
            cor(2, 0),
            cor(3, 1),
            cor(4, 2),
            cor(5, 3),
        ],
        ..TimelineViewSnapshot::default()
    }
}

fn rect_of(
    regs: &[(ph2d_editor_core::ids::NodeId, Rect)],
    id: ph2d_editor_core::ids::NodeId,
) -> Option<Rect> {
    regs.iter().find(|(w, _)| *w == id).map(|(_, r)| *r)
}

fn tap(host: &mut MockPanelHost, r: Rect) {
    let ev = |kind, t| PointerEvent {
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    };
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, 0));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, 1));
}

/// Fechado, o grupo é UMA linha: o losango dele existe sob o rato e os ▶ dos membros não. Um clique
/// nele seleciona as 4 chaves da cor e só elas. (Mutação: não registar o hit em `paint_grupo` ⇒ o
/// `expect` dispara; tirar o braço `GroupKey` do despacho ⇒ zero seleções.)
#[test]
fn clicar_no_losango_do_grupo_seleciona_a_cor_inteira() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    set_current_timeline(Some(snap()));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        rect_of(&regs, timeline_twirl_id(3)).is_none(),
        "fechado, os membros não têm linha"
    );
    let r = rect_of(&regs, timeline_group_key_id(2, 0.0_f64.to_bits()))
        .expect("o losango do grupo foi pintado mas não registado (morto sob o rato)");
    let _ = drain_intents();

    tap(&mut host, r);
    let _ = host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    let mut sel: Vec<_> = drain_intents()
        .into_iter()
        .filter_map(|i| match i {
            TimelineIntent::AddToSelection(k) => Some(k.target.get()),
            _ => None,
        })
        .collect();
    sel.sort_unstable();
    assert_eq!(
        sel,
        vec![2, 3, 4, 5],
        "as 4 chaves da cor, e não a da rotação"
    );
    set_current_timeline(None);
}

/// O ▶ do grupo abre as linhas dos membros (cada uma com o seu ▶ do editor de curva): o primeiro
/// membro (R) não tem linha antes do clique e tem depois. (Mutação: tirar o braço `GroupTwirl` do
/// despacho ⇒ continua sem linha.)
#[test]
fn o_triangulo_do_grupo_mostra_os_membros() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    set_current_timeline(Some(snap()));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        rect_of(&regs, timeline_twirl_id(2)).is_none(),
        "fechado, o R não tem linha"
    );
    let r = rect_of(&regs, timeline_group_twirl_id(2)).expect("o ▶ do grupo não está registado");

    tap(&mut host, r);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        rect_of(&regs, timeline_twirl_id(2)).is_some(),
        "aberto, o R ganha a sua linha (e o seu ▶ do editor de curva)"
    );
    set_current_timeline(None);
}
