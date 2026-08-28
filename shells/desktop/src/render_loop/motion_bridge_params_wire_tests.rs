//! As provas de **um fio, UM número** — o report do Enio de 2026-08-27.
//!
//! ⚠️ **A régua é a row que o painel de facto constrói** (`build_params_snapshot`), não a
//! `wire_face` sozinha. Uma sonda que perguntasse só à resolução ficaria verde com o
//! construtor a ignorá-la — que é exactamente a forma de gate vazio que a auditoria deste
//! módulo apanhou vinte e quatro vezes: *um gate sobre a DECLARAÇÃO está verde quando o
//! EXECUTOR a ignora.* As duas metades (face + faixa) são medidas no número final.

use super::super::build_params_snapshot;
use super::{WireFace, wire_face};
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_node_registry::ParamUnit;
use ph2d_panel_motion_params::ParamRow;

/// A row `Value` do `value.number` selecionado, já com a face aplicada (é o `.in_display`
/// que o construtor faz no push, então o que sai daqui é o que o artista lê).
fn value_row(motion: &MotionState) -> ph2d_panel_motion_params::ScalarRow {
    let snap = build_params_snapshot(motion, ProjectSettings::default())
        .expect("o `value.number` resolve");
    snap.rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Scalar(s) if s.name == "value" => Some(s.clone()),
            _ => None,
        })
        .expect("o modo Number mostra a row `value`")
}

/// **O DEFEITO REPORTADO, no número final: `0,94` no condutor lia `0,94` e no destino
/// `94 px`.**
///
/// Com a cura, a row do condutor lê `94 px` — o MESMO número e o MESMO sufixo que a row do
/// destino —, porque a face que ela veste é a do param que ela alimenta.
///
/// ⚠️ **O controlo está no mesmo teste**: sem o fio, a mesma row é o número cru e sem
/// sufixo. Sem essa metade, um gate que só afirmasse `94 px` passaria com a face colada no
/// nó (que seria a mentira oposta: um número sem destino não tem unidade).
#[test]
fn a_number_driving_a_length_reads_in_the_same_face_as_what_it_drives() {
    let mut motion = MotionState::new();
    let num = motion.doc.graph.add_node("value.number");
    motion.doc.graph.set_param(num, "value", 0.94);
    ph2d_panel_motion_graph::set_graph_selection(vec![num.0]);

    // O CONTROLO: sem fio nenhum, a magnitude não tem destino, logo não tem unidade.
    let bare = value_row(&motion);
    assert!(
        (bare.value - 0.94).abs() < 1e-6,
        "sem fio a row e' o numero cru, deu {}",
        bare.value
    );
    assert_eq!(bare.display.suffix, "", "sem fio nao ha sufixo nenhum");

    // E agora o fio, no param do report.
    let shape = motion.doc.graph.add_node("source.shape");
    motion
        .doc
        .graph
        .drive_param(shape, "size", (num, 0))
        .expect("o `size` aceita ser dirigido");

    let dressed = value_row(&motion);
    assert!(
        (dressed.value - 94.0).abs() < 1e-4,
        "0,94 m sao 94 px a 100 px/m, e a row do condutor tem de os dizer; deu {}",
        dressed.value
    );
    assert_eq!(dressed.display.suffix, "px");

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **A FAIXA SEGUE A FACE — e é a outra metade da cura.**
///
/// ⚠️ Com a face do destino e a faixa própria, o `Number` mostraria `94 px` num slider de
/// `±7500 px` (o `±75` dele convertido), onde a `size` inteira do destino cabe num décimo
/// do track. A row tem de alcançar a faixa que a row do DESTINO alcança — `0,05..10 m`,
/// que é `5..1000 px`.
#[test]
fn the_drivers_track_is_the_destinations_track() {
    let mut motion = MotionState::new();
    let num = motion.doc.graph.add_node("value.number");
    let shape = motion.doc.graph.add_node("source.shape");
    motion
        .doc
        .graph
        .drive_param(shape, "size", (num, 0))
        .expect("dirige");
    ph2d_panel_motion_graph::set_graph_selection(vec![num.0]);

    let row = value_row(&motion);
    assert!(
        (row.min - 5.0).abs() < 1e-3 && (row.max - 1000.0).abs() < 1e-3,
        "a faixa do `size` (0,05..10 m) em px e' 5..1000; deu {}..{}",
        row.min,
        row.max
    );
    // E o `±75` próprio do nó ficou para trás: ele é o default de quem não conduz nada.
    assert!(
        row.max < 7500.0,
        "a faixa propria convertida seria +-7500 px, que e' o defeito"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **DESTINOS QUE DISCORDAM DEIXAM A LACUNA — a cerca original, intacta.**
///
/// Um condutor que alimenta um comprimento **e** um ângulo não tem uma face; qualquer
/// escolha estaria certa metade do tempo, e a metade errada multiplicaria por
/// `pixels_per_meter`. É o caso exacto de que o doc do `ParamUnit::None` fala, e a resposta
/// continua a ser a dele.
#[test]
fn two_destinations_that_disagree_leave_the_number_bare() {
    let mut motion = MotionState::new();
    let num = motion.doc.graph.add_node("value.number");
    let shape = motion.doc.graph.add_node("source.shape");
    motion
        .doc
        .graph
        .drive_param(shape, "size", (num, 0))
        .expect("dirige o comprimento");
    // O `rotation` do mesmo nó é um `ParamWidget::Angle` — graus, nunca metros.
    motion
        .doc
        .graph
        .drive_param(shape, "rotation", (num, 0))
        .expect("dirige o angulo");

    assert_eq!(
        wire_face(&motion, num),
        None,
        "duas unidades nao fazem uma face"
    );

    ph2d_panel_motion_graph::set_graph_selection(vec![num.0]);
    let row = value_row(&motion);
    assert_eq!(row.display.suffix, "", "a lacuna e' a resposta honesta");
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **DOIS DESTINOS DA MESMA UNIDADE SOMAM AS FAIXAS, e não recusam.**
///
/// ⚠️ Discordância de UNIDADE é recusa; discordância de FAIXA não é. Dois comprimentos são
/// a mesma grandeza, e a união alcança os dois — recusar aqui faria o segundo fio APAGAR a
/// cura que o primeiro trouxe, que é o defeito a fingir-se de prudência.
#[test]
fn two_destinations_of_one_unit_union_their_ranges() {
    let mut motion = MotionState::new();
    let num = motion.doc.graph.add_node("value.number");
    let a = motion.doc.graph.add_node("source.shape");
    let b = motion.doc.graph.add_node("source.shape");
    motion.doc.graph.drive_param(a, "size", (num, 0)).unwrap();
    motion.doc.graph.drive_param(b, "size", (num, 0)).unwrap();

    let face = wire_face(&motion, num).expect("duas vezes a mesma unidade e' uma unidade");
    assert_eq!(face.unit, ParamUnit::Length);
    assert_eq!(
        face.range,
        Some((0.05, 10.0, 0.05)),
        "a uniao de duas faixas iguais e' a propria faixa"
    );
}

/// **UM CONDUTOR DE UM CONDUTOR RECUSA.**
///
/// ⚠️ **A 1.ª redacção deste teste justificava a recusa com um CICLO, e a medição refutou-a:**
/// o `Graph::drive_param` já recusa fechar ciclo (`EdgeError::WouldCycle`), tal como um
/// `connect` — um param dirigido *é* uma dependência. Então a resolução recursiva terminaria,
/// e a recusa não é sobre terminação.
///
/// Ela é sobre o que o artista LÊ: com recursão, a face de um nó viria de um destino a três
/// saltos, sem nada na tela a dizê-lo — e cada salto multiplicaria as maneiras de a resposta
/// ser ambígua sem que a ambiguidade aparecesse em lado nenhum. *Uma face que se resolve num
/// salto pode ser apontada; uma que se resolve numa cadeia é adivinhação com procedimento.*
#[test]
fn a_driver_whose_destination_is_itself_a_driver_refuses() {
    let mut motion = MotionState::new();
    let a = motion.doc.graph.add_node("value.number");
    let b = motion.doc.graph.add_node("value.number");
    motion.doc.graph.drive_param(b, "value", (a, 0)).unwrap();
    assert_eq!(wire_face(&motion, a), None);

    // ⭐ E o CICLO nem chega a existir — o documento recusa-o, que é a garantia independente
    // desta. Sem esta metade eu continuaria a atribuir a terminação a uma recusa que não a
    // compra.
    assert!(
        motion.doc.graph.drive_param(a, "value", (b, 0)).is_err(),
        "o grafo tem de recusar fechar o ciclo, como faria num `connect`"
    );

    // E a cadeia LONGA (b conduz c) continua a recusar no 1.º salto, não no fundo dela.
    let c = motion.doc.graph.add_node("source.shape");
    motion.doc.graph.drive_param(c, "size", (b, 0)).unwrap();
    assert_eq!(
        wire_face(&motion, a),
        None,
        "a face de `a` nao atravessa `b` ate' ao `size` de `c`"
    );
    assert!(
        wire_face(&motion, b).is_some(),
        "mas `b`, que conduz o `size` directamente, veste-se"
    );
}

/// **UM NÓ QUE NÃO CONDUZ NADA NÃO TEM FACE** — a terceira recusa, e o estado em que todo
/// `value.number` nasce.
#[test]
fn a_driver_of_nothing_has_no_face() {
    let mut motion = MotionState::new();
    let num = motion.doc.graph.add_node("value.number");
    assert_eq!(wire_face(&motion, num), None);
    // Uma ARESTA não é um param dirigido: ligar a saída a uma PORTA não veste ninguém, e é
    // essa a fronteira que a cerca do `ParamUnit::None` guarda (uma coluna pode ser
    // qualquer coisa).
    let math = motion.doc.graph.add_node("value.math");
    motion
        .doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (num, 0),
            to: (math, 0),
            delayed: false,
        })
        .expect("liga a porta");
    assert_eq!(
        wire_face(&motion, num),
        None,
        "uma porta nao declara unidade nenhuma"
    );
}

/// **A face resolvida nunca é uma PERGUNTA** — `FromWire` e `FromChannel` são perguntas, e
/// deixá-las sair daqui poria uma delas no lugar de uma resposta no `display_face`.
#[test]
fn the_resolved_face_is_never_itself_a_question() {
    let mut motion = MotionState::new();
    let num = motion.doc.graph.add_node("value.number");
    let shape = motion.doc.graph.add_node("source.shape");
    motion
        .doc
        .graph
        .drive_param(shape, "size", (num, 0))
        .unwrap();
    let WireFace { unit, .. } = wire_face(&motion, num).expect("resolve");
    assert!(!matches!(
        unit,
        ParamUnit::FromWire | ParamUnit::FromChannel
    ));
}

/// **A LEI VALE PARA O `value.lfo` TAMBÉM, e ele é o nó do report original.**
///
/// ⚠️ *"preciso usar LFO:Offset"* foi a frase que abriu o `value.number` — o artista ia buscar
/// o `offset` de uma LFO com amplitude zero porque não havia um número. O `offset` dele tem a
/// MESMA confusão de faces, e a cura é a mesma declaração.
///
/// ⚠️ **Os DOIS params, e o par é o ponto:** a saída é `w·amplitude + offset`, então declarar só
/// um deles seria meia unidade (há gate de homogeneidade em `ph2d-node-registry-init`). Aqui
/// mede-se a outra ponta — que a declaração CHEGA à row que o artista lê.
#[test]
fn the_lfo_wears_the_face_of_what_it_drives_too() {
    let mut motion = MotionState::new();
    let lfo = motion.doc.graph.add_node("value.lfo");
    let shape = motion.doc.graph.add_node("source.shape");
    motion
        .doc
        .graph
        .drive_param(shape, "size", (lfo, 0))
        .expect("o `size` aceita fio");
    ph2d_panel_motion_graph::set_graph_selection(vec![lfo.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default()).expect("resolve");
    let row = |name: &str| {
        snap.rows
            .iter()
            .find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == name => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("a row `{name}` existe"))
    };
    for name in ["amplitude", "offset"] {
        assert_eq!(
            row(name).display.suffix,
            "px",
            "`{name}` conduz um comprimento, logo le-se em px"
        );
    }
    // ⚠️ **E o CONTROLE: o `period` NÃO se converte.** Ele são segundos, e uma face de
    // comprimento ali multiplicaria um tempo por `pixels_per_meter` — o defeito que o
    // `FromChannel` regista como `±90` virando `±9000`.
    assert_eq!(row("period").display.suffix, "s");

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
