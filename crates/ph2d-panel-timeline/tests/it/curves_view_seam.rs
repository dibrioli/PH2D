//! ⭐ **«Keys | Curves»** (doc 17, item 5; maquetes B1/B2): o segmentado pintado na barra troca a área das
//! faixas por UM grafo, e volta; a chave arrasta-se no grafo grande num só passo de undo; o olho esconde a
//! curva; fora da aba Keys e no clássico o botão não existe. Gesto real sobre rects pintados.

use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, Vista, ids};
use ph2d_timeline::{
    AnimTarget, Interp, KeyId, KeyView, PropKind, TimelineIntent, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 700.0);
const Y: u64 = 7;
const X: u64 = 8;

fn kv(id: u64, t: f64, v: f32) -> KeyView {
    KeyView {
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp: Interp::Linear,
        selected: false,
        roving: false,
    }
}

fn faixa(target: u64, prop: PropKind, keys: Vec<KeyView>) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop,
        entity: 1,
        missing: false,
        keys,
        buffer_ghost: None,
        pre: ph2d_timeline::Extrap::Hold,
        post: ph2d_timeline::Extrap::Hold,
        expr: None,
    }
}

fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 24.0,
        view_length_seconds: 2.0,
        view_length_explicit: true,
        tracks: vec![
            faixa(
                Y,
                PropKind::TranslationY,
                vec![kv(1, 0.0, 0.0), kv(2, 0.5, 96.0), kv(3, 1.0, 0.0)],
            ),
            faixa(
                X,
                PropKind::TranslationX,
                vec![kv(4, 0.0, 0.0), kv(5, 1.0, 60.0)],
            ),
        ],
        ..TimelineViewSnapshot::default()
    }
}

fn mesa(cartoes: bool) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    set_current_timeline(Some(snap()));
    let mut m = Mesa::nova(TimelinePanelState::default());
    m.assenta_em(JANELA);
    let _ = drain_intents();
    let _ = m.host.drained_actions();
    m
}

fn pinta(m: &mut Mesa<TimelinePanel>) {
    m.rects = m.host.paint::<TimelinePanel>(&mut m.st, JANELA);
}

fn tem(m: &Mesa<TimelinePanel>, id: ph2d_a11y::NodeId) -> bool {
    m.rect(id).is_some()
}

/// Clicar o «Curves» PINTADO troca as chaves pelas âncoras do grafo grande; clicar «Keys» volta.
/// *Mutação: `route` que não muda a vista ⇒ vermelho.*
#[test]
fn clicar_curves_troca_a_area_pelo_grafo_e_keys_volta() {
    let mut m = mesa(true);
    assert!(
        tem(&m, ids::timeline_key_hit_id(Y, 2)),
        "Keys: a chave está na faixa"
    );
    assert!(
        !tem(&m, ids::timeline_anchor_hit_id(Y, 2)),
        "Keys: sem grafo aberto"
    );
    m.clica(ids::TIMELINE_VIEW_CURVES);
    pinta(&mut m);
    assert_eq!(m.st.vista, Vista::Curves);
    assert!(
        tem(&m, ids::timeline_anchor_hit_id(Y, 2)),
        "Curves: a âncora da principal"
    );
    assert!(
        !tem(&m, ids::timeline_key_hit_id(Y, 2)),
        "Curves: zero losangos"
    );
    assert!(
        !tem(&m, ids::timeline_anchor_hit_id(X, 5)),
        "a secundária não se edita"
    );
    m.clica(ids::TIMELINE_VIEW_KEYS);
    pinta(&mut m);
    assert_eq!(m.st.vista, Vista::Keys);
    assert!(tem(&m, ids::timeline_key_hit_id(Y, 2)));
}

/// Arrastar a âncora pintada no grafo grande muda a chave dentro de UM `BeginEdit`/`EndEdit`.
#[test]
fn arrastar_a_chave_no_grafo_grande_e_um_passo_de_undo() {
    let mut m = mesa(true);
    m.clica(ids::TIMELINE_VIEW_CURVES);
    pinta(&mut m);
    let _ = drain_intents();
    let r = m
        .rect(ids::timeline_anchor_hit_id(Y, 2))
        .expect("âncora pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    m.ponteiro(PointerKind::Down, cx, cy);
    pinta(&mut m);
    m.ponteiro(PointerKind::Move, cx, cy + 10.0);
    pinta(&mut m);
    m.ponteiro(PointerKind::Move, cx, cy + 30.0);
    pinta(&mut m);
    m.ponteiro(PointerKind::Up, cx, cy + 30.0);
    pinta(&mut m);
    let it = drain_intents();
    let n = |f: fn(&TimelineIntent) -> bool| it.iter().filter(|i| f(i)).count();
    assert_eq!(n(|i| matches!(i, TimelineIntent::BeginEdit)), 1, "{it:?}");
    assert_eq!(n(|i| matches!(i, TimelineIntent::EndEdit)), 1, "{it:?}");
    let ultimo = it.iter().rev().find_map(|i| match i {
        TimelineIntent::SetKeyValue { key, value, .. } if *key == KeyId::new(2) => Some(*value),
        _ => None,
    });
    match ultimo {
        Some(ph2d_timeline::AnimValue::Float(v)) => {
            assert!(v < 96.0, "puxar para baixo baixa o valor: {v}")
        }
        o => panic!("sem SetKeyValue da chave 2: {o:?}"),
    }
}

/// O olho de uma linha, clicado, esconde a curva dela; a principal passa à seguinte.
#[test]
fn o_olho_esconde_a_curva() {
    let mut m = mesa(true);
    m.clica(ids::TIMELINE_VIEW_CURVES);
    pinta(&mut m);
    m.clica(ids::timeline_curve_eye_id(Y));
    pinta(&mut m);
    assert_eq!(m.st.curvas_escondidas, vec![Y]);
    assert!(
        !tem(&m, ids::timeline_anchor_hit_id(Y, 2)),
        "a escondida não se desenha"
    );
    assert!(
        tem(&m, ids::timeline_anchor_hit_id(X, 5)),
        "a outra vira a principal"
    );
    m.clica(ids::timeline_curve_eye_id(Y));
    pinta(&mut m);
    assert!(tem(&m, ids::timeline_anchor_hit_id(Y, 2)));
}

/// Fora da aba Keys e no clássico o segmentado não existe; o olho só existe em Curves.
#[test]
fn fora_de_keys_e_no_classico_nao_ha_botao() {
    let mut m = mesa(true);
    assert!(tem(&m, ids::TIMELINE_VIEW_CURVES));
    assert!(
        !tem(&m, ids::timeline_curve_eye_id(Y)),
        "em Keys o ícone é o da propriedade"
    );
    m.clica(ids::TIMELINE_VIEW_CURVES);
    m.clica(ids::TIMELINE_TAB_ARRANGE);
    m.assenta_em(JANELA);
    assert!(!tem(&m, ids::TIMELINE_VIEW_CURVES));
    assert_eq!(m.st.vista, Vista::Keys, "mudar de aba volta a Keys");
    let c = mesa(false);
    assert!(!tem(&c, ids::TIMELINE_VIEW_KEYS));
    assert!(!tem(&c, ids::TIMELINE_VIEW_CURVES));
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Outro clipe na área, ou a lista de Containers, volta a Keys (como mudar de aba).
/// *Mutação: `volta_a_keys` que só olha a lista ⇒ vermelho.*
#[test]
fn mudar_de_clipe_ou_abrir_a_lista_volta_a_keys() {
    let com_clipes = |ativo: usize| TimelineViewSnapshot {
        clips: vec!["Idle".into(), "Run".into()],
        active_clip: ativo,
        ..snap()
    };
    let mut m = mesa(true);
    set_current_timeline(Some(com_clipes(0)));
    pinta(&mut m);
    m.clica(ids::TIMELINE_VIEW_CURVES);
    pinta(&mut m);
    assert_eq!(m.st.vista, Vista::Curves, "o mesmo clipe fica em Curves");
    set_current_timeline(Some(com_clipes(1)));
    pinta(&mut m);
    assert_eq!(m.st.vista, Vista::Keys, "outro clipe volta a Keys");
    assert!(tem(&m, ids::timeline_key_hit_id(Y, 2)));
    m.clica(ids::TIMELINE_VIEW_CURVES);
    pinta(&mut m);
    assert_eq!(m.st.vista, Vista::Curves);
    m.clica(ids::TIMELINE_TAB_CONTAINERS);
    pinta(&mut m);
    assert_eq!(
        m.st.vista,
        Vista::Keys,
        "a lista de Containers volta a Keys"
    );
    ferramenta::force_cards_on_this_thread(None);
}
