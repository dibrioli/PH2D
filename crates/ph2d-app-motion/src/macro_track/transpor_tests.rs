//! Os gates da trilha transposta (W18, `docs/Macros/16_W18_plano.md` §2.5 e §2.7): exportar leva só o
//! grafo do dono; o que toca outro dono recusa-se; aplicar faz nascer os nós no dono novo de OUTRO
//! projecto (outros ids) e, por cima, junta-os aos que ele já tinha.

use super::*;
use crate::macro_track::roteiro::CENA;
use crate::macro_track::smoke::{grava_gestos, grava_sobre};
use ph2d_macro::Event;

/// O objecto que gravou.
const DONO: u64 = 7;
/// O outro objecto Motion do mesmo projecto.
const OUTRO: u64 = 99;
/// O objecto que recebe a macro.
const NOVO: u64 = 123;
/// O gesto em que a gravação arma: os anteriores (5 nós) são o estado inicial, os seguintes criam 3.
const ARMA: usize = 15;

/// O projecto da gravação: dois nós de OUTRO objecto à frente (ids `0` e `1`, ligados).
fn projecto() -> MotionDoc {
    let mut d = MotionDoc::new();
    let a = d.graph.add_node("field.box");
    let b = d.graph.add_node("force.wind");
    d.set_owner(a, OUTRO);
    d.set_owner(b, OUTRO);
    d.graph
        .connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("liga");
    d
}

fn primeira(bytes: &[u8]) -> Track {
    ph2d_macro::codec::read(bytes)
        .expect("lê")
        .tracks
        .into_iter()
        .next()
        .expect("trilha")
}

/// A macro gravada no DONO, com o OUTRO ao lado.
fn gravada() -> Track {
    primeira(&grava_sobre(projecto(), DONO, CENA, ARMA).expect("grava").0)
}

fn empurra(t: &mut Track, p: &Passo) {
    let at = t.events.last().map_or(ph2d_macro::MacroTime(0), |e| e.t);
    t.events.push(Event {
        t: at,
        payload: p.codifica(),
    });
}

/// A gravada com um `Documento` no fim: o projecto INTEIRO de então (o que a testemunha grava depois
/// de um desfazer).
fn com_documento() -> Track {
    let mut t = gravada();
    let d = fim(&t).expect("toca");
    assert!(!d.nodes_of(OUTRO).is_empty(), "o documento leva o outro objecto");
    empurra(
        &mut t,
        &Passo::Documento {
            texto: d.to_text(),
            proximo_id: d.graph.next_id(),
            arestas: d.graph.edges().to_vec(),
        },
    );
    t
}

pub(crate) type Canon = (
    Vec<String>,
    Vec<(usize, u16, usize, u16, bool)>,
    Vec<(usize, String, usize, u16)>,
);

/// O grafo de `dono` a menos dos ids: cada nó pela ordem de id entre os do dono (o `import_owner` e os
/// criados guardam-na), os fios e os params dirigidos por essa ordem.
pub(crate) fn canon(d: &MotionDoc, dono: u64) -> Canon {
    let ids = d.nodes_of(dono);
    let r = |n: NodeId| ids.iter().position(|x| *x == n).expect("do dono");
    let g = &d.graph;
    let nos = ids
        .iter()
        .map(|n| {
            format!(
                "{} {:?} {:?} {:?} {} {:?}",
                g.node(*n).expect("nó").type_name,
                g.node_param_overrides(*n),
                g.node_text_param_overrides(*n),
                g.label(*n),
                g.node_bypassed(*n),
                g.pos(*n)
            )
        })
        .collect();
    let mut fios: Vec<_> = g
        .edges()
        .iter()
        .filter(|e| ids.contains(&e.to.0))
        .map(|e| (r(e.from.0), e.from.1, r(e.to.0), e.to.1, e.delayed))
        .collect();
    fios.sort_unstable();
    let mut dirigidos: Vec<_> = g
        .all_param_sources()
        .iter()
        .filter(|(n, _)| ids.contains(n))
        .flat_map(|(n, s)| s.iter().map(move |(k, (src, p))| (*n, k.clone(), *src, *p)))
        .map(|(n, k, src, p)| (r(n), k, r(src), p))
        .collect();
    dirigidos.sort_unstable();
    (nos, fios, dirigidos)
}

fn ini_de(t: &Track) -> MotionDoc {
    inicial::le(&t.initial).expect("inicial").doc
}

/// ⭐ O ficheiro exportado leva SÓ o grafo do dono — no inicial e no documento de cada passo —, e toca
/// até ao mesmo grafo do dono que a gravação.
#[test]
fn o_motion_exportado_leva_so_o_grafo_do_dono() {
    let t = com_documento();
    let antes = ini_de(&t);
    assert_eq!(antes.nodes_of(OUTRO).len(), 2, "a fixtura tem o outro objecto");
    assert_eq!(antes.nodes_of(DONO).len(), 5);

    let x = so_do_dono(&t).expect("exporta");
    assert_eq!(x.target, DONO);
    let ini = ini_de(&x);
    assert!(ini.nodes_of(OUTRO).is_empty(), "o outro objecto não viaja");
    assert_eq!(
        ini.graph.nodes().len(),
        5,
        "só os nós do dono: {:?}",
        ini.graph.nodes()
    );
    assert_eq!(canon(&ini, DONO), canon(&antes, DONO), "o grafo inicial do dono");
    let Ok(Passo::Documento { texto, .. }) =
        Passo::descodifica(&x.events.last().expect("passo").payload)
    else {
        panic!("o último passo é o documento");
    };
    let doc = MotionDoc::from_text(&texto).expect("lê");
    assert!(doc.nodes_of(OUTRO).is_empty(), "nem no documento do passo");

    let (a, b) = (fim(&t).expect("toca"), fim(&x).expect("toca a exportada"));
    let c = canon(&b, DONO);
    assert_eq!(c.0.len(), 8, "os 5 de antes e os 3 criados");
    assert_eq!(c, canon(&a, DONO), "o mesmo grafo no fim");
    assert!(b.nodes_of(OUTRO).is_empty());
}

/// ⛔ Um passo, ou um fio do estado inicial, que toca um nó de OUTRO objecto recusa-se pelo nome — nunca
/// se larga.
#[test]
fn um_passo_que_toca_outro_dono_recusa_se() {
    let t = gravada();
    let com = |p: Passo| {
        let mut x = t.clone();
        empurra(&mut x, &p);
        x
    };
    let param = com(Passo::Params {
        destino: Destino::No,
        intencoes: vec![IntencaoDeParam::SetParam {
            node: Alvo::Existente(0),
            param: "width".into(),
            value: 3.0,
        }],
    });
    assert_eq!(so_do_dono(&param), Err(ErroAoTranspor::DoutroDono(0)));
    let escrita = com(Passo::Escritas(vec![Escrita::Valor {
        node: Alvo::Existente(1),
        param: "strength".into(),
        valor: Some(2.0),
    }]));
    assert_eq!(so_do_dono(&escrita), Err(ErroAoTranspor::DoutroDono(1)));
    let fio = com(Passo::Grafo {
        nivel: None,
        ajuda: false,
        intencoes: vec![Intencao::Connect {
            from: Alvo::Existente(0),
            from_port: 0,
            to: Alvo::Criado(0),
            to_port: 0,
        }],
    });
    assert_eq!(so_do_dono(&fio), Err(ErroAoTranspor::DoutroDono(0)));
    assert_eq!(
        aplica(&fio, &MotionDoc::new(), NOVO).map(|_| ()),
        Err(ErroAoTranspor::DoutroDono(0))
    );

    // Um fio de fora no estado inicial: do outro objecto para um nó do dono.
    let mut ini = inicial::le(&t.initial).expect("inicial");
    let alvo = ini.doc.nodes_of(DONO)[0];
    ini.doc
        .graph
        .connect(Edge {
            from: (NodeId(1), 0),
            to: (alvo, 7),
            delayed: false,
        })
        .expect("liga");
    let mut de_fora = t.clone();
    de_fora.initial = inicial::escreve(&ini);
    assert_eq!(so_do_dono(&de_fora), Err(ErroAoTranspor::FioDeFora));
    assert_eq!(
        aplica(&de_fora, &MotionDoc::new(), NOVO).map(|_| ()),
        Err(ErroAoTranspor::FioDeFora)
    );
    // Controlo: a mesma trilha sem os passos de fora transpõe-se.
    assert!(so_do_dono(&t).is_ok());
}

/// ⭐ Aplicar como objecto NOVO noutro projecto (outros ids, outro `next_id`): o grafo do fim do dono
/// novo é o da gravação a menos dos ids, e o resto do projecto fica como estava.
#[test]
fn aplicar_novo_faz_nascer_os_nos_no_dono_novo() {
    let t = com_documento();
    let mut vivo = MotionDoc::new();
    let mut deles = Vec::new();
    for tipo in ["motion.grid", "source.shape", "force.drag"] {
        let n = vivo.graph.add_node(tipo);
        vivo.set_owner(n, 5);
        deles.push(n);
    }
    let fora = vivo.graph.add_node("motion.output");
    vivo.graph.remove_node(fora);
    vivo.graph.set_param(deles[0], "rows", 3.0);

    let (x, fim_doc) = aplica(&t, &vivo, NOVO).expect("aplica");
    assert_eq!(x.target, NOVO);
    let ini = ini_de(&x);
    assert_eq!(ini.nodes_of(5), deles, "o inicial é o projecto vivo");
    assert_eq!(ini.nodes_of(NOVO).len(), 5, "mais os nós da macro, do dono novo");
    assert!(ini.nodes_of(DONO).is_empty() && ini.nodes_of(OUTRO).is_empty());

    let gravado = canon(&fim(&t).expect("toca"), DONO);
    assert_eq!(canon(&fim_doc, NOVO), gravado, "o grafo da gravação, nos ids novos");
    assert_eq!(fim_doc.nodes_of(5), deles, "o resto do projecto fica");
    assert_eq!(
        fim_doc.graph.node_param_overrides(deles[0]),
        vivo.graph.node_param_overrides(deles[0])
    );
    assert_eq!(fim(&x).expect("toca a instalada"), fim_doc, "o fim é o da trilha instalada");
}

/// ⭐ Por cima de um objecto que já tem 2 nós, uma macro de 3: ficam 5, e os 2 de antes intactos.
#[test]
fn o_motion_por_cima_junta_os_nos() {
    let m = primeira(&grava_gestos(DONO, &CENA[..6], &[]).expect("grava").0);
    assert!(ini_de(&m).graph.nodes().is_empty(), "a macro cria os 3");

    let mut vivo = MotionDoc::new();
    let a = vivo.graph.add_node("field.box");
    let b = vivo.graph.add_node("force.wind");
    for n in [a, b] {
        vivo.set_owner(n, NOVO);
    }
    vivo.graph.set_param(a, "width", 2.0);
    let fio = Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    };
    vivo.graph.connect(fio).expect("liga");

    let (x, fim_doc) = aplica(&m, &vivo, NOVO).expect("aplica");
    let nos = fim_doc.nodes_of(NOVO);
    assert_eq!(nos.len(), 5, "{nos:?}");
    assert_eq!(&nos[..2], &[a, b], "os de antes ficam com os seus ids");
    assert_eq!(fim_doc.graph.node(a).expect("a").type_name, "field.box");
    assert_eq!(
        fim_doc.graph.node_param_overrides(a),
        vivo.graph.node_param_overrides(a)
    );
    assert!(fim_doc.graph.edges().contains(&fio), "e o fio entre eles");
    let tipos: Vec<_> = nos[2..]
        .iter()
        .map(|n| fim_doc.graph.node(*n).expect("nó").type_name.as_str())
        .collect();
    assert_eq!(tipos, ["source.shape", "motion.duplicator", "motion.output"]);
    let gravado = canon(&fim(&m).expect("toca"), DONO);
    assert_eq!(
        gravado.1.len(),
        2,
        "a macro liga forma → duplicador → saída"
    );
    let novos = fim_doc
        .graph
        .edges()
        .iter()
        .filter(|e| nos[2..].contains(&e.to.0))
        .count();
    assert_eq!(novos, 2, "os fios da macro, entre os nós dela");
    assert_eq!(x.target, NOVO);
}
