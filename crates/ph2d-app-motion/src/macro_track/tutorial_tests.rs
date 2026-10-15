//! Os gates do tutorial «fazer» do Motion (`docs/Macros/03_plano_W6.md` §4, W6a): a macro da cena `=3`
//! (as três ligações do vento e a largura final da caixa) sobre o grafo de um ALUNO — outro dono, outros
//! ids, nós de outro objecto ao lado.

use super::*;
use crate::macro_track::estado_de_teste;
use crate::macro_track::roteiro::CENA;
use crate::macro_track::smoke::{FAZER_DESDE, grava_desde, lugar_do_fazer};
use ph2d_panel_motion_graph::GraphIntent;

/// O objecto que gravou.
const PROFESSOR: u64 = 7;
/// O objecto do aluno.
const ALUNO: u64 = 42;

fn trilha() -> Track {
    let (b, _) = grava_desde(PROFESSOR, CENA, FAZER_DESDE, Some(lugar_do_fazer)).expect("grava");
    ph2d_macro::codec::read(&b)
        .expect("lê")
        .tracks
        .into_iter()
        .next()
        .expect("trilha")
}

fn vaga(tipo: &str) -> Vaga {
    Vaga {
        tipo: tipo.into(),
        ordem: 0,
    }
}

/// O aluno: um nó de OUTRO objecto à frente (o `next_id` do aluno não é o da gravação) e o grafo do
/// estado inicial pelo «Recomeçar».
fn aluno(track: &Track) -> MotionState {
    let mut m = estado_de_teste();
    let outro = m.doc.graph.add_node("field.box");
    m.doc.set_owner(outro, 99);
    recomeca(&mut m, ALUNO, track).expect("recomeça");
    m.history = ph2d_motion_doc::MotionHistory::new();
    m
}

fn faltas(m: &MotionState, c: &[Criterio]) -> Vec<bool> {
    avalia(m, ALUNO, c)
        .iter()
        .map(|e| e.falta.is_some())
        .collect()
}

/// O aluno liga pela porta do editor (o dreno do painel), como o rato faria.
fn liga(m: &mut MotionState, de: &str, pd: u16, para: &str, pp: u16) {
    m.history.begin(&m.doc);
    let s = retrato(m, ALUNO);
    let (a, b) = (s.nos[&vaga(de)].0, s.nos[&vaga(para)].0);
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    crate::motion_bridge::aplica_e_grava(
        m,
        &mut ph,
        &mut ph2d_editor_core::ToastQueue::default(),
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
        vec![GraphIntent::Connect {
            from_node: a,
            from_port: pd,
            to_node: b,
            to_port: pp,
        }],
    );
}

fn desfaz(m: &mut MotionState) {
    let prev = m.history.undo(&m.doc).expect("há passo");
    m.doc = prev;
}

#[test]
fn as_etapas_da_cena_fazer_sao_as_tres_ligacoes_e_a_largura() {
    let c = criterios(&trilha()).expect("critérios");
    let itens: Vec<&Item> = c.iter().flat_map(|c| &c.itens).collect();
    let fio = |de: &str, para: &str, pp: u16, atraso: bool| {
        Item::Liga(Fio {
            de: vaga(de),
            porta_de: 0,
            para: vaga(para),
            porta_para: pp,
            atraso,
        })
    };
    assert_eq!(c.len(), 4, "{itens:#?}");
    assert_eq!(itens.len(), 6, "{itens:#?}");
    assert_eq!(*itens[0], fio("field.box", "force.wind", 0, false));
    assert_eq!(*itens[1], fio("force.wind", "force.drag", 0, false));
    // A 3.ª etapa é UM gesto e três mudanças: o fio do aluno, e o editor muda o laço do integrador da
    // entrada dele para a cabeça da cadeia (a caixa) — a realimentação.
    assert_eq!(c[2].itens.len(), 3);
    assert_eq!(*itens[2], fio("force.drag", "motion.integrate", 1, false));
    assert_eq!(*itens[3], fio("motion.integrate", "field.box", 0, true));
    let Item::Liga(laco) = fio("motion.integrate", "motion.integrate", 1, true) else {
        unreachable!()
    };
    assert_eq!(*itens[4], Item::Desliga(laco));
    assert!(
        matches!(itens[5], Item::Valor { no, param, valor, antes }
            if *no == vaga("field.box") && param == "width" && mesmo_numero(*valor, 2.4)
                && mesmo_numero(*antes, 1.2)),
        "{:?}",
        itens[5]
    );
    assert!(
        c.windows(2).all(|w| w[0].k < w[1].k),
        "pela ordem da gravação"
    );
}

/// ⭐ Cada etapa só se cumpre quando o aluno a FAZ, o Ctrl+Z volta-a e o refazer avança-a; o nó de outro
/// objecto (uma caixa já lá) não conta.
#[test]
fn cada_etapa_so_se_cumpre_quando_feita_e_o_desfazer_a_volta() {
    let t = trilha();
    let c = criterios(&t).expect("critérios");
    let mut m = aluno(&t);
    assert_eq!(faltas(&m, &c), [true; 4]);

    liga(&mut m, "field.box", 0, "force.wind", 0);
    assert_eq!(faltas(&m, &c), [false, true, true, true]);
    desfaz(&mut m);
    assert_eq!(faltas(&m, &c), [true; 4], "desfazer volta o passo");
    let next = m.history.redo(&m.doc).expect("refaz");
    m.doc = next;
    assert_eq!(faltas(&m, &c), [false, true, true, true]);

    // Controlo: um fio ERRADO (a caixa ao arrasto) não cumpre a etapa do vento.
    liga(&mut m, "field.box", 0, "force.drag", 0);
    assert_eq!(faltas(&m, &c)[1], true);
    desfaz(&mut m);
}

/// ⭐ As três ligações feitas À MÃO pelo editor cumprem as três etapas — a 3.ª inclusive, que pede
/// também o laço que o EDITOR muda sozinho (o critério veio do que a gravação viu o editor fazer).
#[test]
fn as_ligacoes_feitas_a_mao_cumprem_as_etapas_e_so_a_largura_falta() {
    let t = trilha();
    let c = criterios(&t).expect("critérios");
    let mut m = aluno(&t);
    liga(&mut m, "field.box", 0, "force.wind", 0);
    liga(&mut m, "force.wind", 0, "force.drag", 0);
    liga(&mut m, "force.drag", 0, "motion.integrate", 1);
    assert_eq!(faltas(&m, &c), [false, false, false, true]);
}

/// ⭐ O balão aponta as DUAS portas do fio que falta, pelos ids do aluno.
#[test]
fn o_alvo_de_uma_ligacao_sao_as_portas_do_aluno() {
    let t = trilha();
    let c = criterios(&t).expect("critérios");
    let m = aluno(&t);
    let s = retrato(&m, ALUNO);
    let (caixa, vento) = (s.nos[&vaga("field.box")].0, s.nos[&vaga("force.wind")].0);
    let e = &avalia(&m, ALUNO, &c)[0];
    assert_eq!(
        e.falta,
        Some(PedidoDoTutorial::Liga {
            de: "field.box".into(),
            para: "force.wind".into()
        })
    );
    use ph2d_panel_motion_graph::alvo;
    assert_eq!(e.alvos, [alvo::saida(caixa, 0), alvo::entrada(vento, 0)]);
}

/// ⭐ Os nós casam por TIPO e ORDEM DE CRIAÇÃO (plano W6 §2): com DUAS caixas, o fio que a gravação
/// ligou à 2.ª só se cumpre na 2.ª caixa do aluno — a 1.ª, do mesmo tipo, não serve — e o balão
/// aponta-a. As fixturas da cena `=3` têm um nó por tipo e não o distinguem.
#[test]
fn com_dois_nos_do_mesmo_tipo_a_etapa_e_do_segundo() {
    use ph2d_panel_motion_graph::alvo;
    let nos = |m: &mut MotionState, dono: u64| {
        ["field.box", "field.box", "force.wind"].map(|t| {
            let id = m.doc.graph.add_node(t);
            m.doc.set_owner(id, dono);
            id
        })
    };
    let fio = |m: &mut MotionState, de: NodeId, para: NodeId| {
        m.doc
            .graph
            .connect(Edge {
                from: (de, 0),
                to: (para, 0),
                delayed: false,
            })
            .expect("liga");
    };
    // O professor liga a 2.ª caixa ao vento.
    let mut p = estado_de_teste();
    let [_, segunda, vento] = nos(&mut p, PROFESSOR);
    let antes = retrato(&p, PROFESSOR);
    fio(&mut p, segunda, vento);
    let c = [Criterio {
        k: 0,
        itens: delta(&antes, &retrato(&p, PROFESSOR)),
    }];
    // O aluno, com um nó de outro objecto à frente (outros ids).
    let mut m = estado_de_teste();
    let outro = m.doc.graph.add_node("field.box");
    m.doc.set_owner(outro, 99);
    let [a1, a2, av] = nos(&mut m, ALUNO);
    fio(&mut m, a1, av);
    let e = &avalia(&m, ALUNO, &c)[0];
    assert!(e.falta.is_some(), "a 1.ª caixa do aluno não é a gravada");
    assert_eq!(e.alvos, [alvo::saida(a2.0, 0), alvo::entrada(av.0, 0)]);
    m.doc.graph.disconnect(av, 0);
    fio(&mut m, a2, av);
    assert_eq!(faltas(&m, &c), [false], "a 2.ª caixa cumpre");
}

/// ⭐ «Mostra-me» cumpre a etapa, é UM passo de undo, e o desfazer devolve-a ao aluno.
#[test]
fn mostra_me_cumpre_a_etapa_e_o_desfazer_a_devolve() {
    let t = trilha();
    let c = criterios(&t).expect("critérios");
    let mut m = aluno(&t);
    for (i, crit) in c.iter().enumerate() {
        mostra(&mut m, ALUNO, crit).expect("mostra");
        let f = faltas(&m, &c);
        assert!(f[..=i].iter().all(|x| !x), "{i}: {f:?}");
        assert!(f[i + 1..].iter().all(|x| *x), "{i}: {f:?}");
    }
    desfaz(&mut m);
    assert_eq!(faltas(&m, &c), [false, false, false, true]);
}

/// ⭐ «Recomeçar» volta ao grafo inicial (o nó do outro objecto fica) e é um passo de undo.
#[test]
fn recomecar_volta_ao_inicio_e_o_desfazer_volta_ao_feito() {
    let t = trilha();
    let c = criterios(&t).expect("critérios");
    let mut m = aluno(&t);
    let inicio = retrato(&m, ALUNO);
    for crit in &c {
        mostra(&mut m, ALUNO, crit).expect("mostra");
    }
    recomeca(&mut m, ALUNO, &t).expect("recomeça");
    assert_eq!(faltas(&m, &c), [true; 4]);
    assert_eq!(retrato(&m, ALUNO).fios, inicio.fios);
    assert_eq!(m.doc.nodes_of(99).len(), 1, "o outro objecto fica");
    desfaz(&mut m);
    assert_eq!(faltas(&m, &c), [false; 4]);
}

/// Um VALOR cumpre-se pelo número que se VÊ e pelo de uma etapa posterior no mesmo param — salvo se
/// esse for o de antes (o estado não distingue «fez» de «não fez»).
#[test]
fn saltar_para_o_valor_final_cumpre_o_intermedio_mas_o_valor_de_antes_nao() {
    let no = vaga("field.box");
    let valor = |v: f32, a: f32| Item::Valor {
        no: no.clone(),
        param: "width".into(),
        valor: v,
        antes: a,
    };
    let crits = [
        Criterio {
            k: 0,
            itens: vec![valor(1.2, 1.0)],
        },
        Criterio {
            k: 1,
            itens: vec![valor(2.4, 1.2)],
        },
        Criterio {
            k: 2,
            itens: vec![valor(1.0, 2.4)],
        },
    ];
    let com = |w: f32| {
        let mut s = Retrato::default();
        s.valores.insert((no.clone(), "width".into()), (w, 1.0));
        s
    };
    assert!(cumpre(&crits[0].itens[0], &com(1.2), &crits[1..]));
    assert!(
        cumpre(&crits[0].itens[0], &com(2.4), &crits[1..]),
        "o final"
    );
    assert!(
        !cumpre(&crits[0].itens[0], &com(1.0), &crits[1..]),
        "o de antes"
    );
    assert!(
        cumpre(&crits[0].itens[0], &com(1.200_000_1), &crits[1..]),
        "o mesmo número no campo"
    );
    assert!(!cumpre(&crits[0].itens[0], &com(1.25), &crits[1..]));
}

/// ⭐ O editor traz à vista os nós da etapa UMA vez por etapa: no quadro seguinte, com o mesmo estado,
/// não volta a pedir (o aluno desloca à vontade); a etapa seguinte pede outra vez, com os nós dela.
#[test]
fn a_etapa_corrente_pede_ao_editor_os_nos_dela_uma_vez() {
    use ph2d_editor_core::macro_tutorial::TutorialDaFamilia;
    use ph2d_panel_motion_graph::take_frame_request;
    let t = trilha();
    let mut m = aluno(&t);
    let _ = take_frame_request();
    let s = retrato(&m, ALUNO);
    let n = |tipo: &str| s.nos[&vaga(tipo)].0;
    let avalia = |m: &mut MotionState| TutorialDaFamilia::avalia(m, ALUNO, &t, 1).expect("avalia");
    avalia(&mut m);
    assert_eq!(
        take_frame_request(),
        Some(vec![n("field.box"), n("force.wind")])
    );
    avalia(&mut m);
    assert_eq!(
        take_frame_request(),
        None,
        "mesmo estado, nenhum pedido novo"
    );
    liga(&mut m, "field.box", 0, "force.wind", 0);
    avalia(&mut m);
    assert_eq!(
        take_frame_request(),
        Some(vec![n("force.wind"), n("force.drag")])
    );
}

/// ⭐ A decisão do dono (15/10, opção «a»): o «Recomeçar» arruma, e a caixa e o vento — os dois nós
/// SOLTOS da cena — ficam lado a lado numa fila; empilhados, ficavam a 650 unidades e o vento caía
/// fora do editor ao zoom legível.
#[test]
fn os_nos_soltos_da_cena_ficam_lado_a_lado() {
    let m = aluno(&trilha());
    let s = retrato(&m, ALUNO);
    let lugar = |tipo: &str| m.doc.graph.pos(s.nos[&vaga(tipo)]).expect("lugar");
    let (caixa, vento) = (lugar("field.box"), lugar("force.wind"));
    assert_eq!(caixa.y, vento.y, "uma fila");
    assert!(caixa.x < vento.x, "pela ordem de criação");
}
