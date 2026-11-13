//! ⭐ Dono 10/11 (foto): o cartão da Timeline estreitado a ~400 px pintava a ⟷ da duração FORA da
//! borda. Nada da Timeline se pinta fora do cartão; a ⟷ cabe na área de tempo — presa à borda com o
//! ▸ quando o fim fica além da vista à mão —, e a vista enquadrada re-enquadra ao estreitar.

use ph2d_editor_core::interaction::TIMELINE_EDGE_R;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{
    AnimTarget, Extrap, Interp, KeyId, KeyView, PropKind, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::caixas_pintadas;
use ph2d_ui_testkit::mesa::Mesa;

const PAINEL: ph2d_a11y::NodeId = ph2d_editor_core::ids::TIMELINE_PANEL;
const JANELA: Rect = Rect::new(0.0, 0.0, 1930.0, 1040.0);
const LARGURA: f32 = 400.0;
/// A régua mede pela linha central; meio traço `Heavy` de folga.
const TOLERANCIA: f32 = 2.0;
const FIM_S: f64 = 4.0;

fn documento() -> TimelineViewSnapshot {
    let chave = |n: u64, t: f64| KeyView {
        id: KeyId::new(n),
        t_seconds: t,
        value: 0.0,
        interp: Interp::Linear,
        selected: false,
        roving: false,
    };
    TimelineViewSnapshot {
        fps: 24.0,
        time_seconds: 1.0,
        view_length_seconds: FIM_S,
        view_length_explicit: true,
        keys_mode: true,
        tracks: vec![TrackView {
            target: AnimTarget::new(1),
            prop: PropKind::TranslationX,
            entity: 7,
            missing: false,
            keys: vec![chave(1, 0.5), chave(2, 3.5)],
            buffer_ghost: None,
            pre: Extrap::Hold,
            post: Extrap::Hold,
            expr: None,
        }],
        ..TimelineViewSnapshot::default()
    }
}

fn cartao(mesa: &Mesa<TimelinePanel>) -> Rect {
    mesa.host
        .store()
        .panel_rect(PAINEL)
        .expect("o cartão pintou")
}

/// A mesa no cartão LARGO, estreitado pela borda direita até `LARGURA` e assentado.
fn mesa_estreita() -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(true));
    set_current_timeline(Some(documento()));
    let mut mesa = Mesa::nova(TimelinePanelState::default());
    mesa.assenta_em(JANELA);
    let r = cartao(&mesa);
    let store = mesa.host.store_mut();
    let base = store.floating_card_offsets(PAINEL);
    let d = (LARGURA - r.w, 0.0);
    store.resize_floating_card(PAINEL, base, (r.w, r.h), TIMELINE_EDGE_R, d, (0.0, 0.0));
    store.mark_floating_card_settled();
    mesa.assenta_em(JANELA);
    mesa
}

fn fecha() {
    set_current_timeline(None);
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}

fn dentro(r: Rect, de: Rect, tol: f32) -> bool {
    r.x >= de.x - tol
        && r.y >= de.y - tol
        && r.x + r.w <= de.x + de.w + tol
        && r.y + r.h <= de.y + de.h + tol
}

/// Tudo o que a Timeline pintou fica no cartão; devolve (a área de tempo, o alvo da ⟷).
fn prova_que_nada_sai(mesa: &mut Mesa<TimelinePanel>) -> (Rect, Rect) {
    let cena = mesa
        .host
        .paint_vector_scene::<TimelinePanel>(&mut mesa.st, JANELA);
    let c = cartao(mesa);
    assert!((c.w - LARGURA).abs() < 1.0, "o cartão estreitou: {c:?}");
    let caixas = caixas_pintadas(&cena);
    assert!(caixas.len() > 50, "a régua leu a cena: {}", caixas.len());
    // A sombra do cartão (o único primitivo que ENVOLVE o cartão inteiro) é dele, por fora de propósito.
    let sombra = |r: &Rect| dentro(c, *r, 0.0);
    // E os controlos de tocar FLUTUAM acima do cartão de propósito (dono 13/11): a fila do ⏮ ao ●,
    // com o fundo que os lê sobre o canvas, colada por cima da moldura.
    let regs = mesa.host.registos_da_ultima_pintura();
    let rect = |id| regs.iter().find(|(w, _)| *w == id).map(|(_, r)| *r);
    let (ini, rec) = (
        rect(ids::TIMELINE_GO_START).expect("o ⏮ flutua"),
        rect(ids::TIMELINE_AUTOKEY).expect("o ● flutua"),
    );
    assert!(
        ini.y + ini.h <= c.y,
        "os controlos não flutuam acima do cartão"
    );
    let pad = ph2d_editor_core::panel::cards::banda::metrics::SEG_PAD;
    let fila = Rect::new(
        ini.x - pad,
        ini.y - pad,
        rec.x + rec.w - ini.x + pad * 2.0,
        ini.h + pad * 2.0,
    );
    let fora: Vec<_> = caixas
        .iter()
        .filter(|r| !sombra(r) && !dentro(**r, c, TOLERANCIA) && !dentro(**r, fila, 0.5))
        .collect();
    assert!(fora.is_empty(), "pintado fora do cartão {c:?}: {fora:?}");
    let fita = rect(ids::TIMELINE_LANES).expect("a fita registou-se");
    let seta = rect(ids::TIMELINE_DUR_HANDLE).expect("a ⟷ registou-se");
    assert!(
        seta.x >= fita.x && seta.x + seta.w <= fita.x + fita.w,
        "o alvo da ⟷ {seta:?} sai da área de tempo {fita:?}"
    );
    (fita, seta)
}

#[test]
fn no_cartao_estreito_nada_se_pinta_fora_e_o_fim_fica_a_vista() {
    let mut mesa = mesa_estreita();
    let (fita, seta) = prova_que_nada_sai(&mut mesa);
    let px = f64::from(fita.w) / mesa.st.view_span_s;
    let fim = fita.x + ((FIM_S - mesa.st.view_start_s) * px) as f32;
    let folga = (FIM_S * f64::from(ph2d_tokens::chrome::TIMELINE_ENQUADRE_MARGEM) * px) as f32;
    fecha();
    assert!(
        fim + folga <= fita.x + fita.w,
        "re-enquadrou: o fim {fim} e a folga {folga} cabem até {}",
        fita.x + fita.w
    );
    assert!(
        seta.x > fim,
        "a ⟷ à direita do fim, solta: {seta:?} × {fim}"
    );
}

#[test]
fn com_a_vista_a_mao_a_seta_fica_presa_na_borda() {
    let mut mesa = mesa_estreita();
    mesa.st.px_per_s *= 4.0;
    mesa.st.view_start_s = 0.0;
    mesa.assenta_em(JANELA);
    let (fita, seta) = prova_que_nada_sai(&mut mesa);
    let px = f64::from(fita.w) / mesa.st.view_span_s;
    let fim = fita.x + ((FIM_S - mesa.st.view_start_s) * px) as f32;
    fecha();
    let direita = fita.x + fita.w;
    assert!(
        fim > direita,
        "a vista à mão deixou o fim além: {fim} > {direita}"
    );
    assert!(
        direita - (seta.x + seta.w) < 1.0,
        "a ⟷ presa na borda direita: {seta:?} × {direita}"
    );
}

/// Arrastar a ⟷ presa continua a mudar a duração, e a leitura do arrasto também fica no cartão.
#[test]
fn a_seta_presa_arrasta_a_duracao_e_a_leitura_fica_dentro() {
    let mut mesa = mesa_estreita();
    mesa.st.px_per_s *= 4.0;
    mesa.st.view_start_s = 0.0;
    mesa.assenta_em(JANELA);
    let (_, seta) = prova_que_nada_sai(&mut mesa);
    let _ = drain_intents();
    let (x, y) = (seta.x + seta.w * 0.5, seta.y + seta.h * 0.5);
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x - 6.0, y));
    ev.extend(mesa.ponteiro(PointerKind::Move, x - 30.0, y));
    let _ = mesa.aplica(ev);
    let (_, durante) = prova_que_nada_sai(&mut mesa);
    let pedidos = drain_intents();
    fecha();
    assert_eq!(durante, seta, "o mesmo alvo durante o arrasto");
    assert!(
        pedidos.iter().any(|i| format!("{i:?}").contains("Length")),
        "o arrasto pediu uma duração nova: {pedidos:?}"
    );
}
