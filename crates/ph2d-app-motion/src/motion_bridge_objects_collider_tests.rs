//! Os gates do colisor de um objecto da cena (doc 115 W4 → §33): a lei da caixa (agora no
//! `Collide` do `source.object`), a membrana que deixou de declarar, o passe que se arma sozinho, e
//! a migração dos documentos que ligavam o `Collide` no `Output`.
//!
//! ⚠️⚠️ **Toda fixtura da lei tem `size ≠ 1`, e isso é a lei do ficheiro e não um gosto.** A lei
//! errada que a §10.1 do doc 115 escreveu — declarar `size / 2` em vez do quadrado unitário — dá
//! **exactamente a mesma resposta** quando `size = [1, 1]`, porque a porta da leitura multiplica
//! pelo `size`. *Um corpus no ponto neutro de uma conversão não testa essa conversão.*

use ph2d_contact::Forma;
use ph2d_node_source_object::MEIA_DO_OBJECTO;
use ph2d_node_source_object::colisor::com_colisor;
use ph2d_nodegraph::attr::{
    BOUNCE_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column,
    FRICTION_COLUMN, INV_INERTIA_COLUMN, Stream,
};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const COLUNAS_DO_COLISOR: [&str; 3] =
    [COLLIDER_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN];

/// A caixa de MUNDO que a linha `i` de `s` declara, pela porta do produto.
fn caixa(s: &Stream, i: usize) -> Option<([f32; 2], [f32; 2])> {
    let c = ph2d_contact::colisores(s)?.get(i).copied().flatten()?;
    match c.forma {
        Forma::Caixa { meia, .. } => Some((meia, c.desvio)),
        Forma::Disco(_) => None,
    }
}

/// Uma folha de grupo, com o tamanho já composto (é o que o `resolve_leaf` entrega).
fn folha(p: [f32; 2], size: [f32; 2], rot_deg: f32) -> super::super::LeafInstance {
    super::super::LeafInstance {
        p,
        rot_deg,
        size,
        tint: BRANCO,
        uv: UV,
        tid: 7,
        gid: 0,
    }
}

/// ⭐⭐⭐ **A MEMBRANA NÃO DECLARA COLISOR NENHUM** (doc 115 §33) — nos TRÊS construtores da
/// aparência. Um objecto só colide quando o artista liga o `Collide` do cartão dele; se a membrana
/// declarasse, todo objecto da cena colidiria sem o pedir — e a cerca do dispositivo (que pergunta
/// pelos botões) não o veria.
#[test]
fn a_membrana_nao_declara_colisor_nenhum() {
    let medidos = [
        (
            "sprite/ladrilho",
            super::super::appearance_tile([4.0, 1.0], BRANCO, UV, 7, false),
        ),
        (
            "vector vivo",
            super::super::appearance_vector([4.0, 1.0], BRANCO, 3),
        ),
        (
            "grupo",
            super::super::group_stream(&[folha([0.0, 0.0], [4.0, 1.0], 0.0)]),
        ),
    ];
    assert_eq!(medidos.len(), 3, "o piso: TRES construtores de aparencia");
    for (nome, s) in &medidos {
        assert_eq!(s.count(), 1, "`{nome}`: o controlo — a corrente existe");
        for c in COLUNAS_DO_COLISOR {
            assert!(
                s.get(c).is_none(),
                "`{nome}` publicou `{c}` — a declaracao e' do Collide do source.object"
            );
        }
        assert!(ph2d_contact::colisores(s).is_none());
    }
}

/// ⭐⭐⭐ **A LEI, medida pela porta que a lê: a caixa de mundo de um objecto é `size / 2`** — e a
/// declaração que a produz é o quadrado UNITÁRIO, porque [`ph2d_contact::declarado`] converte de
/// geometria para mundo multiplicando pelo `size` da própria linha.
///
/// ⛔ **É este gate que mata a redacção `ph2d_collider_box = size / 2`**: com ela, um objecto de
/// `4 × 1` declararia `[2; 0,5]` e a porta devolveria `[8; 0,5]`.
#[test]
fn a_caixa_de_mundo_de_um_objecto_e_metade_do_tamanho_dele() {
    for size in [[4.0, 1.0], [0.5, 0.5], [2.0, 3.0], [1.0, 1.0]] {
        let s = com_colisor(super::super::appearance_tile(size, BRANCO, UV, 7, false));
        let (meia, desvio) = caixa(&s, 0).expect("o Collide declara uma CAIXA");
        assert_eq!(
            meia,
            [size[0] / 2.0, size[1] / 2.0],
            "a caixa de mundo tem de ser metade do quadro desenhado (size = {size:?})"
        );
        assert_eq!(
            desvio,
            [0.0, 0.0],
            "o quadro de um objecto e' centrado no P"
        );
    }
}

/// ⭐⭐ **OS TRÊS MÉDIOS**, pela porta do cartão — o grupo entra com uma folha **escalada**, o
/// caminho por onde a conversão de unidades erraria sem ninguém ver.
#[test]
fn o_collide_do_objecto_declara_a_caixa_nos_tres_medios() {
    let esperado = [2.0, 0.5];
    let medidos = [
        (
            "sprite/ladrilho",
            com_colisor(super::super::appearance_tile(
                [4.0, 1.0],
                BRANCO,
                UV,
                7,
                false,
            )),
        ),
        (
            "vector vivo",
            com_colisor(super::super::appearance_vector([4.0, 1.0], BRANCO, 3)),
        ),
        (
            "grupo",
            com_colisor(super::super::group_stream(&[folha(
                [0.0, 0.0],
                [4.0, 1.0],
                0.0,
            )])),
        ),
    ];
    for (nome, s) in &medidos {
        let (meia, _) =
            caixa(s, 0).unwrap_or_else(|| panic!("`{nome}` nao declarou forma nenhuma"));
        assert_eq!(meia, esperado, "`{nome}`");
    }
}

/// ⚠️ **Uma folha do grupo leva a ORIENTAÇÃO dela**, e a caixa é orientada com ela.
#[test]
fn a_caixa_de_uma_folha_rodada_roda_com_ela() {
    let s = com_colisor(super::super::group_stream(&[folha(
        [0.0, 0.0],
        [4.0, 1.0],
        90.0,
    )]));
    let c = ph2d_contact::colisores(&s).expect("declara")[0].expect("uma caixa");
    let Forma::Caixa { meia, eixo } = c.forma else {
        panic!("uma folha e' uma caixa, nunca um disco");
    };
    assert_eq!(meia, [2.0, 0.5], "as meias sao da folha, nao do angulo");
    assert!(
        eixo[0].abs() < 1e-6 && (eixo[1] - 1.0).abs() < 1e-6,
        "a 90 graus o eixo x da caixa aponta para cima: {eixo:?}"
    );
}

/// ⛔⛔ **AS QUATRO AUSÊNCIAS, cada uma uma lei** — o desvio (a arte é centrada), o raio (um objecto
/// é um quadro), o material e a inércia (o cartão não os autora).
#[test]
fn o_collide_do_objecto_declara_a_caixa_e_mais_nada() {
    let s = com_colisor(super::super::appearance_tile(
        [4.0, 1.0],
        BRANCO,
        UV,
        7,
        false,
    ));
    assert!(s.get(COLLIDER_BOX_COLUMN).is_some(), "a caixa, essa, vai");
    for c in [
        COLLIDER_OFFSET_COLUMN,
        COLLIDER_COLUMN,
        FRICTION_COLUMN,
        BOUNCE_COLUMN,
        INV_INERTIA_COLUMN,
    ] {
        assert!(
            s.get(c).is_none(),
            "o Collide do objecto escreveu `{c}` — a AUSENCIA e' a declaracao"
        );
    }
}

/// ⚠️ **Uma corrente vazia sai como entrou.** Uma coluna de zero linhas não é uma declaração.
#[test]
fn uma_corrente_vazia_nao_declara_nada() {
    let saida = com_colisor(Stream::new(0));
    assert_eq!(saida.count(), 0);
    assert!(saida.get(COLLIDER_BOX_COLUMN).is_none());
}

/// ⚠️ A constante é `0,5` e **não** `1`, que é a convenção da `source.shape` (geometria em raio 1).
#[test]
fn a_meia_declarada_e_o_quadrado_unitario() {
    assert_eq!(MEIA_DO_OBJECTO, [0.5, 0.5]);
}

/// O tipo da coluna é o que a porta exige — um `Vec2` do comprimento da nuvem.
#[test]
fn a_coluna_e_um_vec2_do_comprimento_da_nuvem() {
    let s = com_colisor(super::super::group_stream(&[
        folha([0.0, 0.0], [2.0, 2.0], 0.0),
        folha([3.0, 0.0], [1.0, 1.0], 0.0),
    ]));
    match s.get(COLLIDER_BOX_COLUMN) {
        Some(Column::Vec2(v)) => assert_eq!(v.len(), 2),
        outro => panic!("a caixa tem de ser um Vec2 por folha: {outro:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// doc 115 §33 — O PASSE AUTOMÁTICO sobre objectos, pela rota da CPU
// ─────────────────────────────────────────────────────────────────────────────

/// `source.object "Bola"` (tamanho `2`) carimbado em DOIS pontos a `1` de distância — as cópias
/// entram uma na outra em metade do lado. Devolve `(estado, saída)`.
fn duas_bolas(collide: Option<f32>) -> (crate::motion_state::MotionState, NodeId) {
    let mut m = crate::motion_state::MotionState::new();
    let g = &mut m.doc.graph;
    let obj = g.add_node("source.object");
    g.set_text_param(obj, ph2d_node_source_object::OBJECT_PARAM, "Bola");
    if let Some(v) = collide {
        g.set_param(obj, ph2d_node_source_object::COLLIDE_PARAM, v);
    }
    let grade = g.add_node("motion.grid");
    for (p, v) in [("rows", 1.0), ("cols", 2.0), ("gap_x", 1.0)] {
        g.set_param(grade, p, v);
    }
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    for (de, para) in [(obj, (dup, 0)), (grade, (dup, 1)), (dup, (out, 0))] {
        g.connect(Edge {
            from: (de, 0),
            to: para,
            delayed: false,
        })
        .expect("liga");
    }
    m.pump.cook.set_external(
        "Bola".to_string(),
        super::super::appearance_tile([2.0, 2.0], BRANCO, UV, 7, false),
    );
    (m, out)
}

fn xs(s: &Stream) -> Vec<f32> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        _ => panic!("a saida tem de trazer P"),
    }
}

/// ⭐⭐⭐ **OBJECTOS SEM O BOTÃO: nenhuma coluna de colisor, e o passe não faz NADA** (doc 115
/// §33, gate (a)) — o quadro é o de sempre, ao bit, e é isto que mantém a rota do dispositivo
/// aberta (o irmão da cerca: `uma_peca_que_declara_recusa_o_dispositivo`).
#[test]
fn objectos_sem_o_botao_nao_declaram_e_o_passe_e_identidade() {
    let (mut m, out) = duas_bolas(None);
    let cozido = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, out, 0.0)
        .expect("coze")[0]
        .as_stream()
        .clone();
    assert_eq!(cozido.count(), 2, "o controlo: as duas copias existem");
    for c in COLUNAS_DO_COLISOR {
        assert!(
            cozido.get(c).is_none(),
            "sem o botao a corrente trouxe `{c}`"
        );
    }
    let (separado, r) = ph2d_eval_motion::o_que_o_sink_desenha(&m.doc.graph, out, &cozido);
    assert!(
        separado.is_none(),
        "sem declaracao o passe tem de ser identidade"
    );
    assert_eq!(r.pecas, 0);
}

/// ⭐⭐⭐ **O `Collide` DO OBJECTO declara a caixa e o passe SEPARA na CPU, sem botão no Output**
/// (doc 115 §33, gate (b)) — a rota do cartão até ao que se desenha.
#[test]
fn o_collide_do_objecto_separa_as_copias_sozinho() {
    let (mut m, out) = duas_bolas(Some(1.0));
    let cozido = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, out, 0.0)
        .expect("coze")[0]
        .as_stream()
        .clone();
    let antes = xs(&cozido);
    assert!(
        (antes[1] - antes[0]).abs() < 1.5,
        "o controlo: as copias tem de nascer sobrepostas: {antes:?}"
    );
    assert!(
        cozido.get(COLLIDER_BOX_COLUMN).is_some(),
        "o botao ligado tem de declarar a caixa"
    );
    let separado = ph2d_eval_motion::o_que_o_sink_desenha(&m.doc.graph, out, &cozido)
        .0
        .expect("com o Collide do objecto o passe separa — sem tocar no Output");
    let depois = xs(&separado);
    assert!(
        (depois[1] - depois[0]).abs() > 1.9,
        "duas caixas de lado 2 tem de ficar a ~2 de distancia: {antes:?} -> {depois:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// doc 115 §33 — A MIGRAÇÃO do `Collide` do Output
// ─────────────────────────────────────────────────────────────────────────────

/// `objecto → dup ← grelha`, `dup → saída`, mais uma forma a montante e um objecto SOLTO (que não
/// chega à saída). Devolve `(grafo, saída, objecto ligado, objecto solto, forma)`.
fn documento_antigo(collide_da_saida: Option<f32>) -> (Graph, NodeId, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let obj = g.add_node("source.object");
    let solto = g.add_node("source.object");
    let forma = g.add_node("source.shape");
    let dup = g.add_node("motion.duplicator");
    let mistura = g.add_node("motion.combine");
    let out = g.add_node("motion.output");
    for (de, para) in [
        (obj, (dup, 0)),
        (dup, (mistura, 0)),
        (forma, (mistura, 1)),
        (mistura, (out, 0)),
    ] {
        g.connect(Edge {
            from: (de, 0),
            to: para,
            delayed: false,
        })
        .expect("liga");
    }
    if let Some(v) = collide_da_saida {
        g.set_param(out, "collide", v);
    }
    (g, out, obj, solto, forma)
}

fn collide_de(g: &Graph, n: NodeId) -> Option<f32> {
    g.node_param_overrides(n)
        .and_then(|p| p.get(ph2d_node_source_object::COLLIDE_PARAM))
        .copied()
}

/// ⭐⭐⭐ **O documento que ligava o `Collide` no Output abre com os OBJECTOS a montante ligados, e o
/// override legado some** — os objectos soltos e as formas ficam como estavam (as formas já tinham
/// o botão delas, e era ele que as fazia declarar).
#[test]
fn a_migracao_liga_os_objectos_a_montante_e_apaga_o_legado() {
    let (mut g, out, obj, solto, forma) = documento_antigo(Some(1.0));
    assert_eq!(crate::migra_o_collide_da_saida(&mut g), 1);
    assert_eq!(collide_de(&g, obj), Some(1.0), "o objecto a montante liga");
    assert_eq!(
        collide_de(&g, solto),
        None,
        "o objecto solto nao chega a saida"
    );
    assert_eq!(collide_de(&g, forma), None, "a forma tem o botao dela");
    assert!(
        g.node_param_overrides(out)
            .and_then(|p| p.get("collide"))
            .is_none(),
        "o override legado tem de sair — o Output ja nao o declara"
    );
    // ⭐ O documento migrado VALIDA: o legado era um `UnknownParam`.
    let reg = crate::motion_state::MotionState::new().registry;
    let violacoes = g.validate(&reg).err().unwrap_or_default();
    assert!(
        !violacoes
            .iter()
            .any(|v| matches!(v, ph2d_nodegraph::graph::Violation::UnknownParam { .. })),
        "{violacoes:?}"
    );
}

/// ⚠️ **Desligado, nenhum objecto muda — mas o legado sai na mesma** (ele é um param que o nó já
/// não declara).
#[test]
fn a_migracao_com_o_legado_desligado_so_apaga_o_override() {
    let (mut g, out, obj, _, _) = documento_antigo(Some(0.0));
    assert_eq!(crate::migra_o_collide_da_saida(&mut g), 0);
    assert_eq!(collide_de(&g, obj), None);
    assert!(g.node_param_overrides(out).is_none(), "o override saiu");
}

/// ⚠️ **IDEMPOTENTE**, e um documento novo (sem legado) passa intacto, ao bit.
#[test]
fn a_migracao_e_idempotente_e_nao_toca_num_documento_novo() {
    let (mut g, _, obj, _, _) = documento_antigo(Some(1.0));
    assert_eq!(crate::migra_o_collide_da_saida(&mut g), 1);
    let depois = g.clone();
    assert_eq!(crate::migra_o_collide_da_saida(&mut g), 0, "a 2.a corrida");
    assert_eq!(g, depois);
    assert_eq!(collide_de(&g, obj), Some(1.0));

    let (mut novo, ..) = documento_antigo(None);
    let antes = novo.clone();
    assert_eq!(crate::migra_o_collide_da_saida(&mut novo), 0);
    assert_eq!(novo, antes, "um documento sem legado nao muda um bit");
}

/// ⭐⭐ **A PORTA DE CARREGAMENTO migra** — o `MotionState::load_text` (o Ctrl+O do projecto) entrega
/// o documento já migrado. Um gate que chamasse só a função afirmaria a lei, nunca que o produto a
/// usa.
#[test]
fn o_carregamento_do_projecto_migra_o_collide_da_saida() {
    let (g, _, obj, _, _) = documento_antigo(Some(1.0));
    let mut doc = ph2d_motion_doc::MotionDoc::new();
    doc.graph = g;
    let texto = doc.to_text();
    let mut m = crate::motion_state::MotionState::new();
    m.load_text(&texto).expect("o documento antigo abre");
    assert_eq!(collide_de(&m.doc.graph, obj), Some(1.0));
    assert!(
        m.doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.output")
            .all(|n| m
                .doc
                .graph
                .node_param_overrides(n.id)
                .and_then(|p| p.get("collide"))
                .is_none()),
        "o legado nao pode sobreviver ao carregamento"
    );
}
