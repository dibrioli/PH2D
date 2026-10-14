//! **Gate 3 do plano** (`docs/Macros/01_plano_onda1.md` §5.3): a macro do Motion tocada no palco
//! reproduz AO BIT, quadro a quadro, o que o artista viu (modo fiel); o modo explicativo dá o
//! documento final cozido da semente; o documento autorado não se toca.
//!
//! A fixture é a sessão de um artista com as 24 intenções da sonda (FORMAS — `source.shape` →
//! `motion.duplicator` — e SIMULAÇÃO com campo — `motion.integrate` ⇄ `field.box` → `force.wind` →
//! `force.drag`), e ela EXIGE o fenómeno: edições com o relógio a tocar, uma pausa com edições no
//! mesmo tique, desfazer e refazer, um recuo e um salto do playhead, um gizmo que escreve um param,
//! um nó arrastado (sem re-cozer) e a bomba suja sem edição. Sem os tiques a reprodução cai noutro
//! sítio (o controlo `sem_os_tiques_a_reproducao_cai_noutro_sitio`; a sonda mediu `3,59`).

use super::passo::Passo;
use super::tocar::{ErroDaMacro, Modo, TocadorDoMotion};
use crate::motion_bridge::quadro::{MaoDoEditor, o_quadro};
use crate::motion_state::MotionState;
use ph2d_macro::{MacroPlayer, MacroRecorder, MacroTime, RefRect};
use ph2d_nodegraph::graph::{NodeId, Pos};
use ph2d_panel_motion_graph::{GraphIntent, push_intent};

const DT: f64 = 1.0 / 60.0;

/// Um gesto da macro da sonda. Os nós referem-se por ORDEM DE CRIAÇÃO (slot).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Gesto {
    Add(&'static str),
    Liga(usize, u16, usize, u16),
    Param(usize, &'static str, f32),
}

pub(crate) const FORMA: usize = 0;
pub(crate) const GRADE: usize = 1;
pub(crate) const DUP: usize = 2;
pub(crate) const SAIDA: usize = 3;
pub(crate) const INTEG: usize = 4;
pub(crate) const VENTO: usize = 5;
pub(crate) const CAIXA: usize = 6;
pub(crate) const ARRASTO: usize = 7;

/// A macro de tutorial: 8 nós, 24 gestos.
pub(crate) const MACRO: &[Gesto] = &[
    Gesto::Add("source.shape"),
    Gesto::Param(FORMA, "size", 0.35),
    Gesto::Add("motion.grid"),
    Gesto::Param(GRADE, "rows", 6.0),
    Gesto::Param(GRADE, "cols", 6.0),
    Gesto::Param(GRADE, "gap_x", 0.5),
    Gesto::Param(GRADE, "gap_y", 0.5),
    Gesto::Add("motion.duplicator"),
    Gesto::Liga(FORMA, 0, DUP, 0),
    Gesto::Liga(GRADE, 0, DUP, 1),
    Gesto::Add("motion.output"),
    Gesto::Liga(DUP, 0, SAIDA, 0),
    Gesto::Add("motion.integrate"),
    Gesto::Liga(GRADE, 0, INTEG, 0),
    Gesto::Liga(INTEG, 0, DUP, 1),
    Gesto::Add("force.wind"),
    Gesto::Liga(VENTO, 0, INTEG, 1),
    Gesto::Add("field.box"),
    Gesto::Param(CAIXA, "width", 1.0),
    Gesto::Liga(CAIXA, 0, VENTO, 0),
    Gesto::Add("force.drag"),
    Gesto::Liga(VENTO, 0, ARRASTO, 0),
    Gesto::Liga(ARRASTO, 0, INTEG, 1),
    Gesto::Param(VENTO, "strength", 6.0),
];

/// A intenção que o painel emitiria para o gesto `i`, com os ids da sessão.
pub(crate) fn intent_de(g: Gesto, slots: &[NodeId], i: usize) -> GraphIntent {
    match g {
        Gesto::Add(t) => GraphIntent::AddNode {
            type_name: t,
            x: i as f32 * 120.0,
            y: 0.0,
        },
        Gesto::Liga(a, ap, b, bp) => GraphIntent::Connect {
            from_node: slots[a].0,
            from_port: ap,
            to_node: slots[b].0,
            to_port: bp,
        },
        Gesto::Param(n, k, v) => GraphIntent::SetParam {
            node: slots[n].0,
            param: k,
            value: v,
        },
    }
}

/// O que se VÊ, ao bit: cada instância (quad e vector) com posição, tamanho e o resto.
pub(crate) fn retrato(m: &MotionState) -> Vec<u32> {
    let mut v = Vec::new();
    for i in &m.pump.instances {
        v.extend(i.world_pos.iter().chain(&i.size).map(|f| f.to_bits()));
    }
    v.push(u32::MAX);
    for i in &m.pump.vector_instances {
        v.push(i.geometry_id);
        v.extend(
            i.world_pos
                .iter()
                .chain(&i.size)
                .chain(&i.basis)
                .chain(&i.tint)
                .map(|f| f.to_bits()),
        );
    }
    v
}

/// O maior desvio de posição entre dois retratos do mesmo grafo.
fn desvio(a: &MotionState, b: &MotionState) -> f32 {
    a.pump
        .vector_instances
        .iter()
        .zip(&b.pump.vector_instances)
        .map(|(p, q)| {
            (p.world_pos[0] - q.world_pos[0])
                .abs()
                .max((p.world_pos[1] - q.world_pos[1]).abs())
        })
        .fold(0.0f32, f32::max)
}

/// Um `MotionState` vazio, a cozer na CPU, sem nada nas filas.
pub(crate) fn estado() -> MotionState {
    super::estado_de_teste()
}

/// O que o artista faz entre dois quadros (além dos gestos da macro).
#[derive(Clone, Copy, Debug)]
enum Acao {
    Gesto(usize),
    Pausa,
    Toca,
    /// O playhead salta para este tique.
    Vai(u64),
    Desfaz,
    Refaz,
    /// Um gizmo de canvas escreve a altura da caixa (como o do campo: `set_param` + `mark_dirty`).
    Gizmo(f32),
    /// Arrastar um nó: o documento muda, o cozimento não.
    MoveNo,
    /// A bomba suja sem edição (o LOD da forma).
    Suja,
}

/// A sessão: `(quadro, ação)`, por ordem.
fn roteiro() -> Vec<(u32, Acao)> {
    let mut r = Vec::new();
    for i in 0..=10 {
        r.push((3 + 3 * i as u32, Acao::Gesto(i)));
    }
    r.push((36, Acao::Pausa));
    for (k, i) in (11..=14).enumerate() {
        r.push((38 + 2 * k as u32, Acao::Gesto(i)));
    }
    r.push((46, Acao::Desfaz));
    r.push((48, Acao::Refaz));
    r.push((50, Acao::Vai(10)));
    r.push((50, Acao::Toca));
    for (k, i) in (15..=19).enumerate() {
        r.push((53 + 3 * k as u32, Acao::Gesto(i)));
    }
    r.push((68, Acao::Gizmo(0.6)));
    r.push((70, Acao::MoveNo));
    r.push((72, Acao::Suja));
    r.push((74, Acao::Vai(40)));
    for (k, i) in (20..=23).enumerate() {
        r.push((77 + 3 * k as u32, Acao::Gesto(i)));
    }
    // Um recuo DEPOIS da última edição (tique 52): o scrub restaura do anel um estado que carrega a
    // história desde ela — o palco só o repete se limpou o anel quando o produto o limpou.
    r.push((120, Acao::Vai(70)));
    // Uma escrita de fora DEPOIS da última edição (tique 75) e um recuo para antes dela (72): o
    // produto limpou o anel, logo re-simula; um palco que não o limpasse restauraria o tique 72 de
    // antes do gizmo.
    r.push((125, Acao::Gizmo(0.4)));
    r.push((135, Acao::Vai(72)));
    // A bomba suja sem edição logo depois de um quadro que chegou sujo (a edição do quadro 86): a
    // gravação só a vê se fechou o estado da bomba DEPOIS de cozer.
    r.push((87, Acao::Suja));
    r
}

const QUADROS: u32 = 150;

/// A sessão gravada.
pub(super) struct Gravada {
    inicial: Vec<u8>,
    /// As cargas de cada quadro, pela ordem.
    quadros: Vec<Vec<Vec<u8>>>,
    /// O que se viu em cada quadro.
    pub(super) retratos: Vec<Vec<u32>>,
    pub(super) fim: MotionState,
    /// Quantos quadros a placa cozeu.
    pub(super) na_placa: usize,
}

/// **O artista**: cada quadro passa pelas portas do produto — o dreno do grafo, o de params, o quadro.
fn sessao() -> Gravada {
    sessao_em(None)
}

/// [`sessao`], a cozer na `placa` quando há uma.
pub(super) fn sessao_em(placa: Option<&ph2d_gpu::GpuContext>) -> Gravada {
    let mut m = estado();
    m.gpu_enabled = placa.is_some();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let store = ph2d_editor_core::interaction::WidgetStore::default();
    let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
    let mut ph = ph2d_core::Playhead::new(DT);
    ph.play();
    let mut tick: u64 = 0;
    let inicial = m.gravacao_arma().expect("arma");
    let roteiro = roteiro();
    let mut slots: Vec<NodeId> = Vec::new();
    let (mut quadros, mut retratos, mut na_placa) = (Vec::new(), Vec::new(), 0);
    for q in 0..QUADROS {
        let antes: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
        for (_, a) in roteiro.iter().filter(|(f, _)| *f == q) {
            match *a {
                Acao::Gesto(i) => push_intent(intent_de(MACRO[i], &slots, i)),
                Acao::Pausa => ph.pause(),
                Acao::Toca => ph.play(),
                Acao::Vai(t) => tick = t,
                Acao::Desfaz => {
                    let prev = m.history.undo(&m.doc).expect("há o que desfazer");
                    m.doc = prev;
                    m.pump.mark_dirty();
                }
                Acao::Refaz => {
                    let next = m.history.redo(&m.doc).expect("há o que refazer");
                    m.doc = next;
                    m.pump.mark_dirty();
                }
                Acao::Gizmo(h) => {
                    m.doc.graph.set_param(slots[CAIXA], "height", h);
                    m.pump.mark_dirty();
                }
                Acao::MoveNo => {
                    let p = m.doc.graph.pos(slots[SAIDA]).expect("a saída existe");
                    m.doc.graph.set_pos(
                        slots[SAIDA],
                        Pos {
                            x: p.x + 40.0,
                            y: p.y,
                        },
                    );
                }
                Acao::Suja => m.pump.mark_dirty_keeping_ring(),
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "um tique cabe num f64")]
        ph.seek(tick as f64 * DT);
        crate::motion_bridge::apply_graph_intents(&mut m, &mut ph, &mut toasts, &mut split);
        crate::motion_bridge::params::apply_param_edits_for_tests(&mut m, &store, &mut toasts);
        o_quadro(&mut m, placa, &ph, DT, MaoDoEditor::do_painel());
        na_placa += usize::from(m.gpu_live);
        retratos.push(retrato_visto(&m, placa));
        quadros.push(m.gravacao_passos());
        let mut novos: Vec<NodeId> = m
            .doc
            .graph
            .nodes()
            .iter()
            .map(|n| n.id)
            .filter(|n| !antes.contains(n))
            .collect();
        novos.sort_by_key(|n| n.0);
        slots.extend(novos);
        if ph.is_playing() {
            tick += 1;
        }
    }
    let resto = m.gravacao_para();
    assert!(resto.is_empty(), "nada mudou depois do último quadro");
    assert_eq!(slots.len(), 8, "a macro cria oito nós");
    assert!(
        retratos.last().is_some_and(|r| r.len() > 1),
        "a cena cozeu VAZIA — a comparação ao bit seria sobre nada"
    );
    Gravada {
        inicial,
        quadros,
        retratos,
        fim: m,
        na_placa,
    }
}

/// O que se VÊ, ao bit, também quando a placa cozeu o quadro: as cópias que ela escreveu.
pub(super) fn retrato_visto(m: &MotionState, placa: Option<&ph2d_gpu::GpuContext>) -> Vec<u32> {
    let Some(gpu) = placa.filter(|_| m.gpu_live) else {
        return retrato(m);
    };
    let mut v: Vec<u32> = m
        .gpu_cook
        .instances()
        .map(|i| ph2d_gpu_cook::read_instances(gpu, i))
        .unwrap_or_default()
        .iter()
        .flat_map(|i| i.world_pos.iter().chain(&i.size).map(|f| f.to_bits()))
        .collect();
    v.push(u32::MAX);
    if let Some(f) = m.gpu_cook.formas() {
        v.extend(ph2d_gpu_cook::read_formas(gpu, f));
    }
    v
}

fn eh_quadro(c: &[u8]) -> bool {
    matches!(Passo::descodifica(c), Ok(Passo::Quadro { .. }))
}

/// A sessão pelo núcleo: o gravador (um evento por carga, no tempo do quadro), o ficheiro escrito e
/// lido, e o tocador do núcleo a despachar.
pub(super) fn pelo_nucleo(g: &Gravada) -> ph2d_macro::MacroDoc {
    let mut rec = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
    let tr = rec.open_track(super::FAMILIA, 0, g.inicial.clone());
    for (q, cargas) in g.quadros.iter().enumerate() {
        for c in cargas {
            rec.push(tr, MacroTime::from_ms(q as u64 * 16).0, c.clone())
                .expect("tempo não decrescente");
        }
    }
    let bytes = ph2d_macro::codec::write(&rec.finish());
    ph2d_macro::codec::read(&bytes).expect("o ficheiro lê-se")
}

/// Toca `doc` no palco; depois de cada quadro, `cada(k, palco)`.
pub(super) fn toca(
    doc: &ph2d_macro::MacroDoc,
    modo: Modo,
    placa: Option<&ph2d_gpu::GpuContext>,
    mut cada: impl FnMut(usize, &MotionState),
) -> Result<TocadorDoMotion, ErroDaMacro> {
    let tr = &doc.tracks[0];
    let mut t = TocadorDoMotion::novo(&tr.initial, modo)?;
    let mut k = 0;
    let mut player = MacroPlayer::new(doc);
    player.advance_to(doc, doc.duration(), |_, _, ev| {
        t.aplica(&ev.payload, placa)?;
        if eh_quadro(&ev.payload) {
            cada(k, t.palco());
            k += 1;
        }
        Ok::<(), ErroDaMacro>(())
    })?;
    assert!(player.is_finished(doc));
    Ok(t)
}

#[test]
fn o_modo_fiel_reproduz_cada_quadro_que_o_artista_viu_ao_bit() {
    let g = sessao();
    let autorado = g.fim.doc.to_text();
    let doc = pelo_nucleo(&g);
    let mut vistos = 0;
    let t = toca(&doc, Modo::Fiel, None, |k, palco| {
        assert_eq!(
            retrato(palco),
            g.retratos[k],
            "o quadro {k} do palco não é o que o artista viu"
        );
        vistos += 1;
    })
    .expect("a macro toca até ao fim");
    assert_eq!(
        vistos, QUADROS as usize,
        "um quadro do palco por quadro gravado"
    );
    assert_eq!(
        t.palco().doc,
        g.fim.doc,
        "o documento do palco é o da sessão"
    );
    assert_eq!(
        t.palco().pump.last_cooked_tick(),
        g.fim.pump.last_cooked_tick()
    );
    assert_eq!(
        g.fim.doc.to_text(),
        autorado,
        "tocar não toca o documento autorado"
    );
    // Um palco que deixasse intenções nas filas do painel guiaria o editor no quadro seguinte.
    assert!(
        crate::drain_param_intents().is_empty(),
        "o palco deixou edições de param na fila"
    );
    assert!(
        ph2d_panel_motion_graph::drain_intents().is_empty(),
        "o palco deixou intenções do grafo na fila"
    );
}

#[test]
fn a_fixture_contem_cada_especie_de_passo() {
    let g = sessao();
    let mut especies = std::collections::BTreeSet::new();
    for c in g.quadros.iter().flatten() {
        especies.insert(match Passo::descodifica(c).expect("lê-se") {
            Passo::Grafo { .. } => "grafo",
            Passo::Params { .. } => "params",
            Passo::Escritas(_) => "escritas",
            Passo::Documento { .. } => "documento",
            Passo::Toque(t) if t.editou => "toque que edita",
            Passo::Toque(_) => "toque que só suja",
            Passo::Quadro { a_tocar: false, .. } => "quadro parado",
            Passo::Quadro { .. } => "quadro a tocar",
            Passo::Recusada(_) => "recusada",
            Passo::Desconhecido(_) => "desconhecido",
        });
    }
    for e in [
        "grafo",
        "params",
        "escritas",
        "documento",
        "toque que edita",
        "toque que só suja",
        "quadro parado",
        "quadro a tocar",
    ] {
        assert!(
            especies.contains(e),
            "a fixture não tem «{e}»: {especies:?}"
        );
    }
    assert!(!especies.contains("recusada") && !especies.contains("desconhecido"));
}

#[test]
fn sem_os_tiques_a_reproducao_cai_noutro_sitio() {
    // CONTROLO: os mesmos passos com o relógio congelado (cada quadro no tique 0, parado) — o que uma
    // macro sem o tique da cena faria. Quadro a quadro, a reprodução tem de cair noutro sítio.
    let g = sessao();
    let mut t = TocadorDoMotion::novo(&g.inicial, Modo::Fiel).expect("o inicial lê-se");
    let (mut k, mut diferem, mut maior) = (0, 0, 0.0f32);
    for c in g.quadros.iter().flatten() {
        match Passo::descodifica(c) {
            Ok(Passo::Quadro { dt, sujou, .. }) => {
                let congelado = Passo::Quadro {
                    tempo: 0.0,
                    a_tocar: false,
                    dt,
                    sujou,
                };
                t.aplica(&congelado.codifica(), None)
                    .expect("o quadro coze");
                let r = retrato(t.palco());
                if r != g.retratos[k] {
                    diferem += 1;
                }
                for (p, q) in posicoes(&r).iter().zip(posicoes(&g.retratos[k])) {
                    maior = maior.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs());
                }
                k += 1;
            }
            _ => t.aplica(c, None).expect("o passo aplica-se"),
        }
    }
    eprintln!("sem os tiques: {diferem} de {k} quadros diferem · max |Δpos| = {maior:.4}");
    assert!(
        diferem * 2 > k,
        "sem os tiques só {diferem} de {k} quadros diferem — a fixture não exerce o relógio"
    );
    assert!(
        maior > 0.1,
        "o desvio sem os tiques é pequeno demais: {maior}"
    );
}

/// As posições das instâncias vectoriais de um [`retrato`] (13 palavras cada, depois do marco).
fn posicoes(r: &[u32]) -> Vec<[f32; 2]> {
    let ini = r
        .iter()
        .position(|w| *w == u32::MAX)
        .map_or(r.len(), |i| i + 1);
    r[ini..]
        .as_chunks::<13>()
        .0
        .iter()
        .map(|c| [f32::from_bits(c[1]), f32::from_bits(c[2])])
        .collect()
}

#[test]
fn o_modo_explicativo_da_o_documento_final_cozido_da_semente_ao_bit() {
    let g = sessao();
    let doc = pelo_nucleo(&g);
    let t = toca(&doc, Modo::Explicativo, None, |_, _| {}).expect("a macro toca até ao fim");
    // A construção de uma vez: o documento final, cozido da semente no último tique.
    let mut b = estado();
    b.doc = g.fim.doc.clone();
    b.pump.define_a_lei(g.fim.pump.a_lei());
    let fim = doc.tracks[0]
        .events
        .iter()
        .rev()
        .find_map(|e| match Passo::descodifica(&e.payload) {
            Ok(Passo::Quadro {
                tempo, a_tocar, dt, ..
            }) => Some((tempo, a_tocar, dt)),
            _ => None,
        })
        .expect("há quadros");
    let mut ph = ph2d_core::Playhead::new(fim.2);
    ph.seek(fim.0);
    if !fim.1 {
        ph.pause();
    }
    o_quadro(&mut b, None, &ph, fim.2, MaoDoEditor::default());
    assert!(!b.pump.vector_instances.is_empty());
    assert_eq!(
        retrato(t.palco()),
        retrato(&b),
        "o explicativo não é o documento final cozido da semente (max |Δpos| {:.6})",
        desvio(t.palco(), &b)
    );
}

#[test]
fn as_24_intencoes_de_uma_vez_sao_o_explicativo_da_sessao_so_com_elas() {
    // A medição da sonda, agora gate: passo a passo no explicativo = tudo num lote, ao bit.
    let mut m = estado();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let store = ph2d_editor_core::interaction::WidgetStore::default();
    let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
    let mut ph = ph2d_core::Playhead::new(DT);
    ph.play();
    let inicial = m.gravacao_arma().expect("arma");
    let mut slots: Vec<NodeId> = Vec::new();
    let mut cargas = Vec::new();
    for (i, gst) in MACRO.iter().enumerate() {
        let antes: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
        push_intent(intent_de(*gst, &slots, i));
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        ph.seek((i as u64 * 4) as f64 * DT);
        crate::motion_bridge::apply_graph_intents(&mut m, &mut ph, &mut toasts, &mut split);
        crate::motion_bridge::params::apply_param_edits_for_tests(&mut m, &store, &mut toasts);
        o_quadro(&mut m, None, &ph, DT, MaoDoEditor::do_painel());
        cargas.extend(m.gravacao_passos());
        slots.extend(
            m.doc
                .graph
                .nodes()
                .iter()
                .map(|n| n.id)
                .filter(|n| !antes.contains(n)),
        );
    }
    let mut t = TocadorDoMotion::novo(&inicial, Modo::Explicativo).expect("o inicial lê-se");
    for c in &cargas {
        t.aplica(c, None).expect("o passo aplica-se");
    }
    let mut b = estado();
    for (i, gst) in MACRO.iter().enumerate() {
        push_intent(intent_de(*gst, &slots, i));
    }
    let mut ph_b = ph2d_core::Playhead::new(DT);
    #[expect(clippy::cast_precision_loss, reason = "um tique")]
    ph_b.seek(((MACRO.len() as u64 - 1) * 4) as f64 * DT);
    ph_b.play();
    crate::motion_bridge::apply_graph_intents(&mut b, &mut ph_b, &mut toasts, &mut split);
    crate::motion_bridge::params::apply_param_edits_for_tests(&mut b, &store, &mut toasts);
    o_quadro(&mut b, None, &ph_b, DT, MaoDoEditor::default());
    assert_eq!(t.palco().doc.to_text(), b.doc.to_text(), "o mesmo grafo");
    assert_eq!(
        retrato(t.palco()),
        retrato(&b),
        "o explicativo é a construção de uma vez"
    );
}

/// ⭐ **A macro EM CENA segue o relógio da Timeline** (item 5 da W4, [`super::em_cena`]): a seguir
/// quadro a quadro, cada retrato é o que o artista viu; e um RECUO dá ao bit o retrato de tocar do
/// zero até lá (o scrub para trás recomeça do estado inicial gravado).
#[test]
fn a_macro_em_cena_segue_o_relogio_e_recuar_e_tocar_do_zero_ate_la() {
    use super::em_cena::MacroEmCena;
    let g = sessao();
    let bytes = ph2d_macro::codec::write(&pelo_nucleo(&g));
    let mundo = crate::motion_mundo::MundoDaCena::default();
    let em = |q: u32| f64::from(q) * 0.016;
    let nova = || {
        MacroEmCena::de(&bytes, Modo::Fiel)
            .expect("lê-se")
            .expect("tem a trilha do Motion")
    };

    let mut seguida = nova();
    for q in 0..QUADROS {
        seguida.segue(em(q), &mundo, None).expect("toca");
        assert_eq!(
            retrato(seguida.palco()),
            g.retratos[q as usize],
            "o quadro {q} da macro em cena não é o que o artista viu"
        );
    }

    let meio = QUADROS / 2;
    let mut direta = nova();
    direta.segue(em(meio), &mundo, None).expect("toca");
    assert_ne!(
        retrato(seguida.palco()),
        retrato(direta.palco()),
        "controlo: o fim e o meio têm de diferir, senão o recuo abaixo não mede nada"
    );
    seguida.segue(em(meio), &mundo, None).expect("recua");
    assert_eq!(
        retrato(seguida.palco()),
        retrato(direta.palco()),
        "recuar até ao meio não dá o retrato de tocar do zero até lá"
    );
    assert_eq!(retrato(direta.palco()), g.retratos[meio as usize]);
}
