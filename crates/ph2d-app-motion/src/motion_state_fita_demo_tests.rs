//! O gate da cena `=139` (doc 132 §2, P2): monta, e com o tecto do roteiro a fita dobra em segundos.

use super::*;
use crate::motion_state::MotionState;
use ph2d_eval_motion::{Historia, Razao};

/// O tecto do gate: a dobra tem de chegar ≥ 1 s antes de o relógio parar ([`PARA_EM_S`]), para a
/// faixa parada ter largura na régua.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "segundos inteiros"
)]
const MAX_TIQUES: u64 = ((PARA_EM_S - 1.0) * 60.0) as u64;

/// ⭐ **A CENA MONTA E, COM O TECTO PEQUENO, DOBRA EM MENOS DE [`MAX_TIQUES`]** — pelo caminho da
/// CPU do produto (publica, escopos, bomba). Os gatilhos `sinal_*` são o que a enche: sem eles
/// (só a `batida`) a mesma fita não dobra no mesmo tempo (o CONTROLO).
#[test]
fn a_cena_monta_e_com_o_tecto_pequeno_dobra_em_segundos() {
    let corre = |com_sinais: bool| -> Option<u64> {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let c = monta(&mut m.doc, &m.registry).expect("a cena monta");
        if !com_sinais {
            let sinais: Vec<NodeId> = m
                .doc
                .graph
                .nodes()
                .iter()
                .filter(|n| {
                    m.doc
                        .graph
                        .node_text_param_overrides(n.id)
                        .and_then(|t| t.get(ph2d_node_value_input::NAME_KEY))
                        .is_some_and(|s| s.starts_with("sinal_"))
                })
                .map(|n| n.id)
                .collect();
            assert_eq!(sinais.len(), GATILHOS);
            for n in sinais {
                assert!(m.doc.graph.remove_node(n));
            }
        }
        m.sinks = vec![c.saida];
        m.pump.define_a_lei(true);
        let tecto = ph2d_eval_motion::razao::tecto_da_fita(Some(TECTO_DO_ROTEIRO), None);
        m.pump.define_o_razao_da_fita(Razao::new(tecto));
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        for t in 0..MAX_TIQUES {
            #[expect(clippy::cast_precision_loss, reason = "um tique")]
            let s = t as f64 / 60.0;
            crate::motion_externals::publish_all(&mut m, s);
            let sinks = m.sinks.clone();
            m.pump.advance_or_scrub_scoped(
                &m.doc.graph,
                &m.registry,
                &sinks,
                t,
                |t| t as f64 / 60.0,
                m.default_uv_rect,
                m.default_size,
                &scopes,
            );
            assert!(m.pump.last_error().is_none(), "{:?}", m.pump.last_error());
            if m.pump.fita().historia_em(0) != Historia::Verdade {
                return Some(t);
            }
        }
        None
    };
    let t = corre(true).expect("com o tecto do roteiro a fita tem de dobrar em 4 s");
    eprintln!("cena 139: a fita dobrou no tique {t}");
    assert!(
        corre(false).is_none_or(|sem| sem > t),
        "CONTROLO: os gatilhos `sinal_*` são o que enche a fita"
    );
}

/// ⭐ **O RELÓGIO PÁRA AOS [`PARA_EM_S`] E A FAIXA FICA NA RÉGUA** — a ferramenta toca o relógio do
/// editor ao entrar, e sem a paragem a faixa seguia-o para fora do ecrã (foto do coordenador, 13/10:
/// 25 s depois de abrir, a faixa em `[22, 25]` s numa régua de 0 a 6). Pela porta do `dispatch`
/// ([`crate::motion_demo_legend::encena_a_paragem`]), quadro a quadro como o produto; e o CONTROLO:
/// sem a cena montada, o relógio não pára.
#[test]
fn o_relogio_da_cena_para_aos_cinco_segundos() {
    let corre = |monta_a_cena: bool| {
        if monta_a_cena {
            let mut doc = MotionDoc::new();
            let mut reg = NodeRegistry::new();
            ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
            assert!(build(&mut doc, &reg).is_some(), "a cena monta");
        }
        let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
        ph.play();
        for _ in 0..1_500 {
            ph.advance();
            crate::motion_demo_legend::encena_a_paragem(&mut ph);
        }
        ph
    };
    let ph = corre(true);
    assert!(!ph.is_playing(), "a cena pára o relógio");
    assert!(
        (ph.time() - PARA_EM_S).abs() < 1e-9,
        "parou em {}",
        ph.time()
    );
    // Uma vez só: tocar de novo é do artista.
    let mut de_novo = ph;
    de_novo.play();
    for _ in 0..60 {
        de_novo.advance();
        crate::motion_demo_legend::encena_a_paragem(&mut de_novo);
    }
    assert!(de_novo.is_playing() && de_novo.time() > PARA_EM_S + 0.9);
    let sem = corre(false);
    assert!(
        sem.is_playing() && sem.time() > 20.0,
        "CONTROLO: sem a cena, nada pára"
    );
}

/// O que o arrasto REAL da régua empurra no barramento: o painel da Timeline, numa vista
/// `[0, 10)` s na aba `Keys`, com a régua posta em `frac` (o caminho do
/// `keys_ruler_moves_the_editor_clock_seam` da `ph2d-panel-timeline`).
fn o_arrasto_da_regua(frac: f32) -> ph2d_editor_core::action_bus::EditorAction {
    use ph2d_panel_timeline::{TimelinePanel, ids::TIMELINE_RULER};
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<TimelinePanel>();
    let mut painel = ph2d_panel_timeline::state::TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 10.0,
        tab: ph2d_panel_timeline::tab::Tab::Keys,
        ..ph2d_panel_timeline::state::TimelinePanelState::default()
    };
    host.set_slider_value(TIMELINE_RULER, frac);
    host.apply_panel_event::<TimelinePanel>(
        &mut painel,
        ph2d_editor_core::interaction::WidgetEvent::ValueChanged(TIMELINE_RULER),
    );
    let mut acoes = host.drained_actions();
    assert_eq!(acoes.len(), 1, "a régua empurra UM evento: {acoes:?}");
    acoes.remove(0)
}

/// ⭐ **`PH2D_MOTION_FITA_SCRUB` É O ARRASTO DA RÉGUA** (a foto do tutorial 14): o evento que o
/// ajudante empurra é, ao byte, o que o painel empurra quando o artista arrasta a régua para o
/// mesmo segundo — o shell fá-lo `Scrub` no relógio da vista. Dispara só DEPOIS da paragem (o
/// relógio parado aos [`PARA_EM_S`]) e UMA vez. CONTROLOS: a tocar, e parado antes da paragem,
/// não dispara; outra cena ou um valor que não é segundo, não arma.
#[test]
fn o_scrub_da_foto_e_o_arrasto_da_regua_depois_da_paragem() {
    let segundo = 1.25; // = 0,125 de uma vista de 10 s: exacto em f32 e em f64.
    let regua = o_arrasto_da_regua(0.125);
    assert_eq!(
        o_arrasto(segundo),
        regua,
        "o mesmo evento, no mesmo barramento"
    );

    let mut auto = ScrubAuto::novo(segundo);
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    assert!(auto.passo(&ph).is_none(), "parado no 0, antes da paragem");
    let mut doc = MotionDoc::new();
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    assert!(
        build(&mut doc, &reg).is_some(),
        "a cena monta e pede a paragem"
    );
    ph.play();
    let mut disparos = Vec::new();
    for _ in 0..600 {
        ph.advance();
        crate::motion_demo_legend::encena_a_paragem(&mut ph);
        if let Some(a) = auto.passo(&ph) {
            disparos.push((ph.time(), a));
        }
    }
    assert_eq!(disparos.len(), 2, "UMA vez cada: {disparos:?}");
    let (quando, acao) = disparos.remove(0);
    assert!(
        (quando - PARA_EM_S).abs() < 1e-9,
        "depois da paragem: {quando}"
    );
    assert_eq!(acao, Passo::Arrasta(regua), "o arrasto da régua");
    assert_eq!(disparos[0].1, Passo::Enquadra, "e depois o `F` no cartão");
    // O cartão da foto existe na cena.
    assert!(doc.graph.nodes().iter().any(|n| {
        doc.graph
            .node_text_param_overrides(n.id)
            .and_then(|t| t.get(ph2d_node_value_input::NAME_KEY))
            .is_some_and(|s| s == CARTAO_DA_FOTO)
    }));

    assert_eq!(segundo_do_ambiente(Some("1.25"), Some("139")), Some(1.25));
    assert_eq!(segundo_do_ambiente(Some("1.25"), Some("141")), None);
    assert_eq!(segundo_do_ambiente(Some("x"), Some("139")), None);
    assert_eq!(segundo_do_ambiente(Some("-1"), Some("139")), None);
    assert_eq!(segundo_do_ambiente(None, Some("139")), None);
}

/// ⭐ **DEPOIS DO ARRASTO PARA ANTES DA FAIXA, A FAIXA E O AVISO FICAM** (o passo 3 do tutorial 14,
/// fotografado: a faixa sumia e o cartão calava-se ao fim de UM quadro). A `=139` pelo quadro do
/// PRODUTO: toca até à paragem, pára, vai ao segundo 1, e os quadros seguintes no MESMO tique
/// re-cozem (sujos, como a placa coze todo quadro) — a fita fica com a faixa de antes e o cartão
/// continua a dizer que mostra a base. Nas duas rotas (a da placa onde há adapter).
#[test]
fn depois_do_arrasto_para_antes_da_faixa_a_faixa_e_o_aviso_ficam() {
    let rota = |gpu: Option<&ph2d_gpu::GpuContext>| {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let c = monta(&mut m.doc, &m.registry).expect("a cena monta");
        m.sinks = vec![c.saida];
        m.gpu_enabled = gpu.is_some();
        let tecto = ph2d_eval_motion::razao::tecto_da_fita(Some(TECTO_DO_ROTEIRO), None);
        m.pump.define_o_razao_da_fita(Razao::new(tecto));
        let quadro = |m: &mut MotionState, ph: &ph2d_core::Playhead| match gpu {
            Some(g) => crate::motion_bridge::quadro::coze_o_quadro(m, g, ph, 1.0 / 60.0),
            None => crate::motion_bridge::quadro::coze_o_quadro_sem_placa(m, ph, 1.0 / 60.0),
        };
        let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
        ph.play();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "tiques"
        )]
        for k in 0..=(PARA_EM_S * 60.0) as u64 {
            #[expect(clippy::cast_precision_loss, reason = "um tique")]
            ph.seek(k as f64 / 60.0);
            quadro(&mut m, &ph);
        }
        ph.pause();
        quadro(&mut m, &ph);
        let faixa = m.pump.fita().faixa().expect("gravou");
        let (base, _) = faixa;
        assert!(base > 60, "a fita dobrou depois do segundo 1: {faixa:?}");
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        let aviso = ph2d_i18n::tr_with(
            crate::motion_historia::CHAVE_DA_QUEIXA,
            &[("s", &format!("{:.2}", base as f64 / 60.0))],
        );
        ph.seek(1.0);
        for q in 0..4 {
            quadro(&mut m, &ph);
            assert_eq!(
                m.pump.fita().faixa(),
                Some(faixa),
                "quadro {q}: a fita intacta"
            );
            assert_eq!(
                m.queixa_da_historia.as_deref(),
                Some(aviso.as_str()),
                "quadro {q}: o cartão continua a avisar"
            );
            m.pump.mark_dirty_keeping_ring();
        }
    };
    rota(None);
    if let Ok(g) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
        rota(Some(&g));
    }
}
