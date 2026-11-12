//! **O container aberto NO LUGAR** (doc 17 item 10): o mapa de cada volta de uma instância, o que
//! o snapshot publica (a aberta e as irmãs da MESMA lane, em tempo de cena) e a edição embrulhada
//! (`TimelineIntent::NoLugar`), convertida cena → container num sítio só.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_timeline::{
    PropKind, StackHost, StripId, StripLoop, StripSource, TimelineIntent as I, TimelineState,
    TimelineViewSnapshot, Volta, apply_intent, instance_map,
};

const EPS: f64 = 1e-9;

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

/// Fixa a instância `id` da cena em `[t0, t1)` lendo `[0, 2)` do container, no modo `modo`.
fn fixa(st: &mut TimelineState, lane: usize, id: StripId, t0: f64, t1: f64, modo: StripLoop) {
    let s = st.doc.strip_in_mut(StackHost::Document, lane, id).unwrap();
    (s.t_start, s.t_end, s.src_in, s.src_out) = (t0, t1, 0.0, 2.0);
    s.speed = 2.0 / (t1 - t0);
    s.loop_mode = modo;
}

struct Cena {
    st: TimelineState,
    c: usize,
    /// A aberta `[1, 2)` (velocidade 2), a irmã `[5, 6)` na mesma lane, a outra irmã na lane 1.
    a: StripId,
    b: StripId,
    outra: StripId,
    /// A fita de dentro: `[0.5, 1.5)` no tempo do container.
    x: StripId,
}

fn cena() -> Cena {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    for (t, v) in [(0.0, 0.0), (2.0, 10.0)] {
        doc.upsert_key(
            1,
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let c = doc.add_container("Run & jump".into());
    let dentro = StackHost::Container(c);
    doc.add_lane_in(dentro, "Body".into()).unwrap();
    let x = doc
        .add_strip_to(dentro, 0, StripSource::Clip(0), 0.5, 1.5)
        .unwrap();
    doc.add_lane("A".into()).unwrap();
    doc.add_lane("B".into()).unwrap();
    let src = StripSource::Container(u16::try_from(c).unwrap());
    let host = StackHost::Document;
    let a = doc.add_strip_to(host, 0, src, 1.0, 2.0).unwrap();
    let b = doc.add_strip_to(host, 0, src, 5.0, 6.0).unwrap();
    let outra = doc.add_strip_to(host, 1, src, 10.0, 11.0).unwrap();
    fixa(&mut st, 0, a, 1.0, 2.0, StripLoop::Once);
    fixa(&mut st, 0, b, 5.0, 6.0, StripLoop::Once);
    fixa(&mut st, 1, outra, 10.0, 11.0, StripLoop::Once);
    Cena {
        st,
        c,
        a,
        b,
        outra,
        x,
    }
}

fn inner_x(st: &TimelineState, c: usize, x: StripId) -> (f64, f64, f64) {
    let s = st.doc.containers()[c].stack[0]
        .strips
        .iter()
        .find(|s| s.id == x)
        .unwrap();
    (s.t_start, s.t_end, s.ease_in)
}

fn snap(st: &mut TimelineState) -> TimelineViewSnapshot {
    let ph = Playhead::new(1.0 / 60.0);
    let mut s = TimelineViewSnapshot::default();
    s.rebuild(st, &ph, false);
    s
}

fn wrapped(st: &mut TimelineState, a: StripId, intent: I) {
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        st,
        &mut ph,
        I::NoLugar {
            instancia: a,
            intent: Box::new(intent),
        },
    );
}

#[test]
fn no_lugar_o_mapa_da_instancia_da_cada_volta() {
    let mut k = cena();
    // Velocidade 2 (a fixture): uma volta, meio segundo de cena por segundo do container.
    let m = instance_map(&k.st.doc, k.a).unwrap();
    assert_eq!((m.container, m.lane, m.t0, m.t1), (k.c, 0, 1.0, 2.0));
    assert_eq!(
        m.voltas,
        vec![Volta {
            t0: 1.0,
            t1: 2.0,
            u0: 0.0,
            u1: 2.0,
            reversed: false
        }]
    );
    assert!(near(m.escala(), 0.5) && near(m.local(1.5), 1.0) && near(m.scene(1.0), 1.5));
    // Velocidade 1, em 1 s: identidade deslocada.
    fixa(&mut k.st, 0, k.a, 1.0, 3.0, StripLoop::Once);
    let m = instance_map(&k.st.doc, k.a).unwrap();
    assert!(near(m.escala(), 1.0) && near(m.local(2.5), 1.5));
    // Loop de 2 voltas e meia a velocidade 1: três voltas, a última cortada.
    fixa(&mut k.st, 0, k.a, 1.0, 3.0, StripLoop::Loop);
    k.st.doc
        .strip_in_mut(StackHost::Document, 0, k.a)
        .unwrap()
        .t_end = 6.0;
    let m = instance_map(&k.st.doc, k.a).unwrap();
    let v: Vec<_> = m
        .voltas
        .iter()
        .map(|v| (v.t0, v.t1, v.u0, v.u1, v.reversed))
        .collect();
    assert_eq!(
        v,
        vec![
            (1.0, 3.0, 0.0, 2.0, false),
            (3.0, 5.0, 0.0, 2.0, false),
            (5.0, 6.0, 0.0, 1.0, false)
        ]
    );
    // PingPong: a segunda volta lê de trás para a frente.
    k.st.doc
        .strip_in_mut(StackHost::Document, 0, k.a)
        .unwrap()
        .loop_mode = StripLoop::PingPong;
    let m = instance_map(&k.st.doc, k.a).unwrap();
    assert_eq!(
        m.voltas[1],
        Volta {
            t0: 3.0,
            t1: 5.0,
            u0: 2.0,
            u1: 0.0,
            reversed: true
        }
    );
    assert!(near(m.voltas[1].scene(1.5), 3.5));
    // Uma fita de clipe não é instância.
    let clip = k.st.doc.containers()[k.c].stack[0].strips[0].id;
    assert!(instance_map(&k.st.doc, clip).is_none());
}

#[test]
fn no_lugar_o_snapshot_publica_a_aberta_e_as_irmas_da_mesma_lane() {
    let mut k = cena();
    assert!(
        snap(&mut k.st).no_lugar.is_none(),
        "nada aberto, nada publicado"
    );
    k.st.no_lugar = Some(k.a);
    let v = snap(&mut k.st).no_lugar.expect("a instância abre");
    assert_eq!((v.instancia, v.lane, v.container), (k.a, 0, k.c));
    assert_eq!((v.nome.as_str(), v.janela), ("Run & jump", (1.0, 2.0)));
    assert_eq!(v.lanes.len(), 1);
    let fitas: Vec<_> = v.lanes[0]
        .strips
        .iter()
        .map(|s| {
            (
                s.instancia,
                s.aberta,
                s.view.id,
                s.view.t_start,
                s.view.t_end,
            )
        })
        .collect();
    assert_eq!(
        fitas,
        vec![(k.a, true, k.x, 1.25, 1.75), (k.b, false, k.x, 5.25, 5.75)],
        "a aberta em tempo de cena, a irmã da mesma lane esmaecida, nada da lane 1 ({:?})",
        k.outra
    );
    // Na aba Keys, ou dentro de um container, não há «no lugar».
    let ph = Playhead::new(1.0 / 60.0);
    let mut s = TimelineViewSnapshot::default();
    s.rebuild(&mut k.st, &ph, true);
    assert!(s.no_lugar.is_none());
}

#[test]
fn no_lugar_mover_na_cena_cai_no_tempo_do_container_e_a_irma_acompanha() {
    let mut k = cena();
    k.st.no_lugar = Some(k.a);
    // Arrastada para o segundo 1,5 da cena = o segundo 1,0 do container (velocidade 2).
    let mv = I::MoveStrip {
        lane: 0,
        to_lane: 0,
        id: k.x,
        t_start: 1.5,
    };
    wrapped(&mut k.st, k.a, mv);
    assert!(
        near(inner_x(&k.st, k.c, k.x).0, 1.0),
        "{:?}",
        inner_x(&k.st, k.c, k.x)
    );
    let v = snap(&mut k.st).no_lugar.unwrap();
    let irma = v.lanes[0]
        .strips
        .iter()
        .find(|s| s.instancia == k.b)
        .unwrap();
    assert!(
        near(irma.view.t_start, 5.5),
        "a irmã é a mesma peça: {}",
        irma.view.t_start
    );
    // Uma DURAÇÃO de cena divide-se pela escala: 0,25 s de cena = 0,5 s do container.
    let ease = I::SetStripEase {
        lane: 0,
        id: k.x,
        edge: 0,
        seconds: 0.25,
    };
    wrapped(&mut k.st, k.a, ease);
    assert!(
        near(inner_x(&k.st, k.c, k.x).2, 0.5),
        "{:?}",
        inner_x(&k.st, k.c, k.x)
    );
    assert!(k.st.edit_path.is_empty(), "o caminho volta ao que era");
}

#[test]
fn no_lugar_o_loop_do_container_converte() {
    let mut k = cena();
    let lp = I::SetContainerLoop {
        container: 99,
        range: Some((1.25, 1.5)),
        ping_pong: true,
    };
    wrapped(&mut k.st, k.a, lp);
    assert_eq!(k.st.doc.container_loop(k.c), (Some((0.5, 1.0)), true));
    k.st.no_lugar = Some(k.a);
    assert_eq!(
        snap(&mut k.st).no_lugar.unwrap().loop_cena,
        Some((1.25, 1.5))
    );
}

#[test]
fn no_lugar_uma_intencao_fora_da_familia_nao_muda_nada() {
    let mut k = cena();
    let antes = k.st.doc.clone();
    wrapped(&mut k.st, k.a, I::SetSceneLength { len: Some(9.0) });
    let mv = I::MoveStrip {
        lane: 0,
        to_lane: 0,
        id: k.x,
        t_start: 1.5,
    };
    let aninhada = I::NoLugar {
        instancia: k.a,
        intent: Box::new(mv),
    };
    wrapped(&mut k.st, k.a, aninhada);
    // Uma instância que não existe também não.
    let mv = I::MoveStrip {
        lane: 0,
        to_lane: 0,
        id: k.x,
        t_start: 1.5,
    };
    wrapped(&mut k.st, k.x, mv);
    assert_eq!(k.st.doc, antes);
}

#[test]
fn no_lugar_o_undo_desfaz_a_edicao_de_dentro() {
    let mut k = cena();
    let mv = I::MoveStrip {
        lane: 0,
        to_lane: 0,
        id: k.x,
        t_start: 1.5,
    };
    wrapped(&mut k.st, k.a, mv);
    assert!(near(inner_x(&k.st, k.c, k.x).0, 1.0));
    assert!(k.st.undo());
    assert!(near(inner_x(&k.st, k.c, k.x).0, 0.5));
}
