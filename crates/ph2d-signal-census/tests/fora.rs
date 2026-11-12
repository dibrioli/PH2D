//! As origens de FORA do mundo (o grafo do Motion, a timeline, os estados de UI) entram no censo no
//! papel certo, e a impressão digital do quadro muda com elas.

use ph2d_anim::RationalTime;
use ph2d_ecs::World;
use ph2d_nodegraph::graph::Graph;
use ph2d_physics_ecs::SignalOnHit;
use ph2d_signal_census::{FORA, Fora, Hint, Role, census, fingerprint, fora};
use ph2d_timeline::TimelineDoc;
use ph2d_ui_state::{StateRole, StateSets, UiState};

fn n(doc: &str, field: &str) -> String {
    format!("{doc}.{field}")
}

fn grafo(nos: &[(&str, &str)]) -> Graph {
    let mut g = Graph::default();
    for (ty, nome) in nos {
        let id = g.add_node(*ty);
        g.set_text_param(id, fora::NAME_KEY, *nome);
    }
    g
}

fn timeline(sinal: &str) -> TimelineDoc {
    let mut t = TimelineDoc::new();
    let i = t.add_marker(RationalTime::from_seconds(1.0), "m");
    assert!(t.set_marker_signal(i, Some(sinal.into())));
    t
}

fn estados(sinal: &str) -> StateSets {
    let mut s = StateSets::default();
    s.set(7, UiState::new(StateRole::Default));
    let i = s.push_binding(7);
    s.set_binding_name(7, i, sinal.into());
    s
}

#[test]
fn as_constantes_sao_as_dos_nos() {
    assert_eq!(fora::NAME_KEY, ph2d_node_pulse_signal::NAME_KEY);
    assert_eq!(fora::NAME_KEY, ph2d_node_value_input::NAME_KEY);
}

#[test]
fn cada_origem_de_fora_e_contada_no_papel_dela() {
    let g = grafo(&[
        (fora::PULSE_SIGNAL, &n("MotionGraph", "pulse.signal.name")),
        (fora::VALUE_INPUT, &n("MotionGraph", "value.input.name")),
        // Controlo negativo: um nó de outro tipo com `name` não conta.
        ("motion.output", "nao_e_sinal"),
    ]);
    let t = timeline(&n("TimelineDoc", "markers.signal"));
    let s = estados(&n("StateSets", "bindings.name"));
    let painel = [n("AuthoredPanel", "rows.signal")];
    let f = Fora {
        motion: Some(&g),
        timeline: Some(&t),
        ui_states: Some(&s),
        painel: Some(&painel),
    };
    let c = census(&World::new(), &f);
    // Controlo positivo: um nome por linha, nem mais nem menos.
    assert_eq!(c.names.len(), FORA.len(), "nomes: {:?}", c.names.keys());
    assert_eq!(FORA.len(), 5);
    for o in FORA {
        let e = c.ends(&n(o.doc, o.field));
        let want = match o.role {
            Role::Emit => (1, 0),
            Role::Listen => (0, 1),
        };
        assert_eq!((e.emit, e.listen), want, "{}.{}", o.doc, o.field);
    }
    // Sem documentos, nada de fora.
    assert!(census(&World::new(), &Fora::default()).names.is_empty());
}

#[test]
fn o_on_hit_ouvido_so_pelo_motion_nao_diz_nobody() {
    let mut w = World::new();
    w.spawn(SignalOnHit("bump".into()));
    let vazio = census(&w, &Fora::default());
    assert_eq!(vazio.hint("bump", Role::Emit), Hint::Nobody);
    // A entrada do Motion apara o nome, como `input_of`.
    let g = grafo(&[(fora::VALUE_INPUT, " bump ")]);
    let f = Fora {
        motion: Some(&g),
        ..Fora::default()
    };
    assert_eq!(census(&w, &f).hint("bump", Role::Emit), Hint::Count(1));
    // E um `pulse.signal` só-espaços não grita (`collect_signals` salta-o).
    let mudo = grafo(&[(fora::PULSE_SIGNAL, "  ")]);
    let f = Fora {
        motion: Some(&mudo),
        ..Fora::default()
    };
    assert!(!census(&w, &f).names.contains_key("  "));
}

#[test]
fn a_impressao_digital_muda_com_cada_documento_de_fora() {
    let w = World::new();
    let (g1, g2) = (
        grafo(&[(fora::PULSE_SIGNAL, "a")]),
        grafo(&[(fora::PULSE_SIGNAL, "b")]),
    );
    let (t1, t2) = (timeline("a"), timeline("b"));
    let (s1, s2) = (estados("a"), estados("b"));
    let (p1, p2) = (["a".to_owned()], ["b".to_owned()]);
    let base = Fora {
        motion: Some(&g1),
        timeline: Some(&t1),
        ui_states: Some(&s1),
        painel: Some(&p1),
    };
    let f0 = fingerprint(&w, &base);
    assert_eq!(f0, fingerprint(&w, &base), "determinística");
    let variantes = [
        Fora {
            motion: Some(&g2),
            ..base
        },
        Fora {
            timeline: Some(&t2),
            ..base
        },
        Fora {
            ui_states: Some(&s2),
            ..base
        },
        Fora {
            painel: Some(&p2),
            ..base
        },
    ];
    for (i, v) in variantes.iter().enumerate() {
        assert_ne!(
            f0,
            fingerprint(&w, v),
            "variante {i} não mudou a impressão digital"
        );
    }
    // A porta do quadro republica quando só um documento de fora muda.
    let mut vezes = 0;
    ph2d_signal_census::when_changed(&w, &base, |_| vezes += 1);
    ph2d_signal_census::when_changed(&w, &base, |_| vezes += 1);
    ph2d_signal_census::when_changed(&w, &variantes[1], |c| {
        vezes += 1;
        assert_eq!(c.ends("b").emit, 1);
    });
    assert_eq!(vezes, 2);
}

/// ⭐ (GEF02 5.1) **Uma caixa que ouve a chave de um botão do painel do artista não diz «ninguém envia»**:
/// o painel é uma origem de fora (Emit). CONTROLO: sem o painel, «ninguém».
#[test]
fn o_botao_do_painel_do_artista_envia_para_quem_o_ouve() {
    let s = estados("jump");
    let sem = Fora {
        ui_states: Some(&s),
        ..Fora::default()
    };
    assert_eq!(
        census(&World::new(), &sem).hint("jump", Role::Listen),
        Hint::Nobody
    );
    let painel = ["jump".to_owned(), "mode/a".to_owned()];
    let com = sem.com_painel(&painel);
    let c = census(&World::new(), &com);
    assert_eq!(c.hint("jump", Role::Listen), Hint::Count(1));
    assert_eq!(
        c.ends("mode/a").emit,
        1,
        "a escolha de uma lista também envia"
    );
}

/// Quem ouve de FORA do mundo (doc Timeline 17 item 8): a entrada do Motion e o estado de UI ouvem
/// «land»; o pulso do Motion e o marcador da Timeline (que EMITEM) e outro nome não entram.
/// *Mutação: tirar o filtro `Role::Listen` do ramo `FORA` ⇒ o pulso e o marcador entram; tirar a
/// igualdade de nome ⇒ «jump» entra.*
#[test]
fn quem_ouve_de_fora_e_so_quem_ouve_aquele_nome() {
    use ph2d_signal_census::{Listener, Quem, listeners};
    let g = grafo(&[
        (fora::VALUE_INPUT, "land"),
        (fora::PULSE_SIGNAL, "land"),
        (fora::VALUE_INPUT, "jump"),
    ]);
    let t = timeline("land");
    let s = estados("land");
    let f = Fora {
        motion: Some(&g),
        timeline: Some(&t),
        ui_states: Some(&s),
        painel: None,
    };
    let doc = |d| Listener {
        object: None,
        what: Quem::Doc(d),
    };
    assert_eq!(
        listeners(&World::new(), &f, "land").list,
        vec![doc("MotionGraph"), doc("StateSets")]
    );
    assert_eq!(
        listeners(&World::new(), &f, "jump").list,
        vec![doc("MotionGraph")]
    );
}
