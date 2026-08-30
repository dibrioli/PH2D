//! Gates da **membrana das fitas** — a metade da shell do modo `Branches`.
//!
//! ⚠️ **As quatro condições de UI não servem aqui**: isto não é um widget, é uma MEMBRANA. A
//! pergunta é a do `source.shape`: *a shell publica sob a chave que o nó lê?* Um par de chaves
//! divergentes não dá erro nenhum — dá uma planta invisível.

use super::publish;
use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

/// Uma planta que bifurca, no modo pedido.
fn plant(geometry: i32) -> (MotionState, ph2d_nodegraph::graph::NodeId) {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node(ls::MANIFEST.name);
    state
        .doc
        .graph
        .set_param(n, ls::param::GEOMETRY, geometry as f32);
    // Gramática explícita: o modo guiado deriva a sua, e um gate que dependesse dela mediria
    // duas coisas ao mesmo tempo.
    state
        .doc
        .graph
        .set_param(n, ls::param::MODE, ls::MODE_GRAMMAR as f32);
    state.doc.graph.set_text_param(n, ls::AXIOM_PARAM, "F");
    state
        .doc
        .graph
        .set_text_param(n, ls::RULES_PARAM, "F -> F[+F]F[-F]F");
    (state, n)
}

fn published(state: &MotionState, key: &str) -> Option<usize> {
    state
        .pump
        .cook
        .externals()
        .get(key)
        .map(|e| e.value.count())
}

/// A chave que a shell usa, lida pela MESMA porta que o `eval` usa.
fn key_of(state: &mut MotionState, n: ph2d_nodegraph::graph::NodeId) -> String {
    let resolved = super::super::motion_externals::resolved_params(state, n, 0.0, &ls::MANIFEST);
    let texts = state.doc.graph.node_text_param_overrides(n);
    let text = |k: &str| texts.and_then(|m| m.get(k)).cloned().unwrap_or_default();
    ls::ribbon_key(
        |name: &str| resolved.get(name).copied().unwrap_or(0.0),
        &text(ls::AXIOM_PARAM),
        &text(ls::RULES_PARAM),
    )
}

/// ⭐⭐⭐ **UMA planta é UMA instância, com UMA geometria.**
///
/// ⚠️⚠️ **A lei apertou depois do *"ficamos com 4 fps"* (Enio, 2026-08-30).** Ela era *"menos
/// fitas que ossos"* — verdadeira e frouxa: a 1.ª redacção publicava **uma instância por RAMO**,
/// cada uma com geometria distinta, e o renderer tesselava as `3 124` **todo o quadro** (o cache
/// dele é por `geometry_id` e por quadro). Menos que os ossos, e na mesma inutilizável.
///
/// ⇒ a afirmação passa a ser o NÚMERO EXACTO: `1`. É também a leitura mais fiel do report que
/// abriu esta wave — *"não crescem como um objeto só"*.
#[test]
fn a_plant_in_branches_mode_publishes_fewer_ribbons_than_it_has_bones() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let ribbons = published(&state, &key).expect("a shell tem de publicar sob a chave do no'");
    assert_eq!(
        ribbons, 1,
        "uma planta tem de sair como UMA instância — {ribbons} seria uma tesselação por ramo, \
         todo o quadro"
    );

    // Quantos ossos a mesma planta tem, pela porta do próprio nó.
    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    assert!(
        ribbons < sk.count(),
        "{ribbons} fitas para {} ossos — isso é uma fita por retângulo",
        sk.count()
    );
}

/// ⭐⭐ **Cada fita leva uma GEOMETRIA de verdade.**
///
/// ⚠️ Um `geometry_id` de `0` é o «nada» do lowering: publicar contagem certa com ids vazios
/// desenharia coisa nenhuma e passaria no gate de cima.
#[test]
fn every_published_ribbon_carries_a_real_geometry_handle() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let ext = state.pump.cook.externals().get(&key).expect("publicado");
    let Some(Column::Scalar(ids)) = ext.value.get("geometry_id") else {
        panic!("a fita tem de carregar `geometry_id`");
    };
    assert!(!ids.is_empty());
    assert!(
        ids.iter().all(|h| *h > 0.0),
        "há fitas com handle vazio — elas não desenham: {ids:?}"
    );
}

/// ⭐ **O modo antigo continua intocado** — decisão do Enio (*"não quero eliminar o modo
/// atual"*).
///
/// A shell não publica nada, e o nó emite o esqueleto de sempre.
#[test]
fn segments_mode_publishes_nothing_and_keeps_the_old_skeleton() {
    let (mut state, n) = plant(ls::GEOMETRY_SEGMENTS);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    assert!(
        published(&state, &key).is_none(),
        "o modo Segments não pode publicar fitas"
    );
}

/// ⭐⭐⭐ **O default do nó É `Branches`** — a ordem do dono, medida no manifesto e não na
/// memória de ninguém.
#[test]
fn a_node_dropped_from_the_palette_is_born_in_branches_mode() {
    let d = ls::MANIFEST
        .params
        .iter()
        .find(|s| s.name == ls::param::GEOMETRY)
        .expect("o param existe")
        .default;
    assert_eq!(
        d.round() as i32,
        ls::GEOMETRY_BRANCHES,
        "o default tem de ser Branches (Enio, 2026-08-30)"
    );
    // ⚠️ E o VALOR de `Segments` continua a ser `0`: um documento salvo guarda o índice.
    assert_eq!(ls::GEOMETRY_SEGMENTS, 0);
}

/// ⛔⛔⛔ **UMA PLANTA QUE NÃO MUDOU NÃO CONSTRÓI NADA** — o gate que nasceu do *"ficamos com
/// 4 fps"* (Enio, 2026-08-30).
///
/// A membrana tinha o memo certo e **não o usava**: chamava o construtor da fita e só depois o
/// entregava ao `intern`, que não o teria chamado. Cada quadro re-corria o varrimento booleano
/// de todos os ramos de todas as plantas.
///
/// ⚠️ **A régua é uma CONTAGEM, não um relógio** — de propósito. Um gate de tempo entra na
/// família de flakes de recurso sob fan-out do `CLAUDE.md` §5.0; o número de geometrias
/// guardadas é determinístico e diz exactamente a mesma coisa: *se nada mudou, nada se
/// construiu*.
///
/// ⚠️ E corre a VARREDURA entre as duas publicações, que é a segunda metade: sem o
/// `handle_for` a marcar as chaves como vivas, o fim do quadro apagaria as geometrias que estão
/// a ser desenhadas e a reconstrução voltava por outra porta — com o memo intacto.
#[test]
fn republishing_an_unchanged_plant_builds_no_geometry_and_survives_the_sweep() {
    let (mut state, _n) = plant(ls::GEOMETRY_BRANCHES);

    let before = super::ribbons_built();
    publish(&mut state, 0.0);
    let built = state.shape_store.len();
    let first_pass = super::ribbons_built() - before;
    assert!(built > 0, "a 1.ª publicação tem de construir as fitas");
    assert!(first_pass > 0, "a 1.ª publicação tem de CONSTRUIR fitas");
    let dropped = state.shape_store.sweep();
    assert!(
        dropped.is_empty(),
        "a varredura do 1.º quadro apagou {} geometrias que acabaram de ser pedidas",
        dropped.len()
    );

    let before_second = super::ribbons_built();
    publish(&mut state, 0.0);
    let second_pass = super::ribbons_built() - before_second;
    assert_eq!(
        second_pass, 0,
        "a 2.ª publicação da MESMA planta CONSTRUIU {second_pass} fitas (a 1.ª construiu \
         {first_pass}) — o memo está lá e não está a ser usado"
    );
    assert_eq!(
        state.shape_store.len(),
        built,
        "e nada de novo foi guardado"
    );
    let dropped = state.shape_store.sweep();
    assert!(
        dropped.is_empty(),
        "a varredura do 2.º quadro apagou {} geometrias ainda em uso — falta marcá-las vivas",
        dropped.len()
    );
    assert_eq!(state.shape_store.len(), built, "nada se perdeu no caminho");
}

/// ⛔⛔ **UMA PLANTA GRANDE SAI INTEIRA** — o report do Enio de 2026-08-30 (*"9841 ramos passam
/// do tecto de 4096 — a planta sai cortada"*).
///
/// ⚠️ **O gate mede a AUSÊNCIA de um segundo tecto**, e a barra não é um número escolhido: é a
/// contagem que a decomposição devolve. Um corte a `N` ramos passaria despercebido em toda
/// planta pequena e mutilaria exactamente as grandes — que são as que alguém constrói para ver
/// se o motor aguenta.
///
/// ⚠️ E o limite a sério fica NOMEADO no lado do nó (`MAX_MODULES`), que é onde ele foi medido.
#[test]
fn a_big_plant_is_published_whole_and_no_second_ceiling_clips_it() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    // Seis gerações desta gramática dão ~15 k ramos — bem acima do tecto que foi removido.
    state.doc.graph.set_param(n, ls::param::GENERATIONS, 6.0);
    let before = super::ribbons_built();
    publish(&mut state, 0.0);
    let built = super::ribbons_built() - before;

    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    let want = ls::branch::branches(
        &super::v2(&sk, "P"),
        &super::v1(&sk, "parent"),
        &super::v2(&sk, "size"),
        &super::v1(&sk, "sym"),
        0.0,
    )
    .len();
    assert!(
        want > 4096,
        "a fixtura tem de ser MAIOR que o tecto removido: {want}"
    );
    assert_eq!(
        built, want,
        "a membrana construiu {built} fitas de {want} — alguma coisa está a cortar a planta"
    );
}

/// O número de voltas (`NonZero`) de um ponto na geometria composta — o mesmo critério com que
/// o renderer a preenche.
fn winding(path: &ph2d_vec_scene::VecPath, q: [f64; 2]) -> i32 {
    let mut w = 0i32;
    let contours = std::iter::once((path.verts.as_slice(), path.closed))
        .chain(path.subpaths.iter().map(|c| (c.verts.as_slice(), c.closed)));
    for (verts, _closed) in contours {
        let n = verts.len();
        for i in 0..n {
            let a = verts[i].anchor;
            let b = verts[(i + 1) % n].anchor;
            // Regra clássica da meia-recta para o número de voltas.
            if a[1] <= q[1] {
                if b[1] > q[1] {
                    let cross = (b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1]);
                    if cross > 0.0 {
                        w += 1;
                    }
                }
            } else if b[1] <= q[1] {
                let cross = (b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1]);
                if cross < 0.0 {
                    w -= 1;
                }
            }
        }
    }
    w
}

/// ⛔⛔⛔ **NENHUMA FENDA NA JUNÇÃO** — o report do Enio de 2026-08-30 (*"no quarto exemplo, com
/// Custom, pequenas fendas"*), medido.
///
/// ⚠️ **A régua é a COBERTURA, não a contagem de contornos.** Um gate que só contasse os discos
/// ficaria verde com o disco no sítio errado ou com raio zero. Este pergunta o que o olho
/// pergunta: *este ponto está pintado?* — pelo mesmo critério (`NonZero`) com que o renderer o
/// preenche.
///
/// A afirmação é a propriedade que o disco compra: **todo ponto a menos de `w/2` da junção está
/// coberto**. É exactamente o que uma cunha por cobrir viola, e a sonda varre um anel inteiro de
/// direcções para não depender de adivinhar de que lado a cunha caiu.
#[test]
fn no_wedge_is_left_uncovered_where_a_branch_meets_its_parent() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    // ⚠️ Quatro gerações, não cinco: a sonda é `O(sondas × vértices)` e a fixtura grande punha
    // o gate em 18 s. `624` ramos já dão `124` juntas, que é população de sobra para a lei.
    state.doc.graph.set_param(n, ls::param::GENERATIONS, 4.0);
    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    let bs = ls::branch::branches(
        &super::v2(&sk, "P"),
        &super::v1(&sk, "parent"),
        &super::v2(&sk, "size"),
        &super::v1(&sk, "sym"),
        0.0,
    );
    let origin = bs[0].points[0];
    let path = super::plant_geometry(&bs, origin).expect("a planta tem geometria");

    let joints: Vec<_> = bs.iter().filter(|b| b.joins_parent).collect();
    assert!(
        joints.len() > 100,
        "a fixtura tem de ter juntas: {}",
        joints.len()
    );
    let mut naked = 0usize;
    for b in &joints {
        let (p0, w0) = (b.points[0], b.widths[0]);
        let r = f64::from(w0) * 0.5 * 0.6;
        for k in 0..16 {
            let a = std::f64::consts::TAU * f64::from(k) / 16.0;
            let q = [
                f64::from(p0[0] - origin[0]) + r * a.cos(),
                f64::from(p0[1] - origin[1]) + r * a.sin(),
            ];
            if winding(&path, q) == 0 {
                naked += 1;
            }
        }
    }
    assert_eq!(
        naked,
        0,
        "{naked} sondas de {} caíram em cima de uma FENDA — a cunha entre as duas pontas não \
         está coberta",
        joints.len() * 16
    );
}

/// Uma planta cuja gramática pousa um `J` em cada ponta, com o nome pedido no slot pedido.
fn plant_with_leaves(names: [&str; 3]) -> (MotionState, ph2d_nodegraph::graph::NodeId) {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    state.doc.graph.set_param(n, ls::param::GENERATIONS, 3.0);
    state.doc.graph.set_text_param(n, ls::AXIOM_PARAM, "F");
    // Cada ponta ganha as TRÊS letras, para uma fixtura só exercitar os três slots.
    state
        .doc
        .graph
        .set_text_param(n, ls::RULES_PARAM, "F -> F[+F[JKM]]F[-F[JKM]]");
    for (i, name) in names.iter().enumerate() {
        if !name.is_empty() {
            state.doc.graph.set_text_param(n, ls::LEAF_PARAMS[i], *name);
        }
    }
    (state, n)
}

/// Publica um objecto nomeado com a aparência que o `publish_objects` publicaria.
fn publish_object(state: &mut MotionState, name: &str, texture_id: u32) {
    state.pump.cook.set_external(
        name.to_string(),
        super::super::motion_bridge::appearance_tile(
            [2.0, 3.0],
            [1.0, 1.0, 1.0, 1.0],
            [0.25, 0.25, 0.75, 0.75],
            texture_id,
        ),
    );
}

fn column_v1(state: &MotionState, key: &str, col: &str) -> Vec<f32> {
    match state
        .pump
        .cook
        .externals()
        .get(key)
        .map(|e| e.value.get(col))
    {
        Some(Some(Column::Scalar(v))) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐⭐⭐ **A LETRA PLANTA O OBJECTO** — o report do Enio de 2026-08-29 (*"deveríamos ter um modo
/// de escolher o objeto que será exposto em cada fase"*).
///
/// ⚠️ A afirmação é a do PRODUTO: as linhas publicadas têm de trazer a **textura daquele
/// objecto**, e uma por âncora. Um gate que só contasse linhas passaria com folhas invisíveis.
#[test]
fn a_named_letter_plants_that_objects_appearance_at_every_anchor() {
    let (mut state, n) = plant_with_leaves(["folha", "", ""]);
    publish_object(&mut state, "folha", 7);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);

    let tex = column_v1(&state, &key, "texture_id");
    let geom = column_v1(&state, &key, "geometry_id");
    // A linha 0 é a PLANTA (geometria vectorial); as outras são as folhas.
    assert!(geom[0] > 0.0, "a linha 0 tem de ser a planta");
    let leaves = tex
        .iter()
        .skip(1)
        .filter(|t| (**t - 7.0).abs() < 0.5)
        .count();
    assert!(
        leaves > 0,
        "nenhuma folha plantada — texturas publicadas: {tex:?}"
    );
    assert!(
        geom.iter().skip(1).all(|g| *g == 0.0),
        "uma folha não pode levar geometria vectorial: {geom:?}"
    );
}

/// ⭐⭐ **AS TRÊS LETRAS SÃO TRÊS SLOTS, e cada uma planta o SEU objecto.**
///
/// ⚠️ **É o gate que apanha a ordem trocada.** `LEAF_PARAMS` e `LEAF_SYMBOLS` são duas listas
/// emparelhadas por índice; trocar a ordem numa só faria a flor nascer onde o artista pediu
/// folha — e a contagem total ficaria igual, então só medir "há folhas" não veria nada.
#[test]
fn each_of_the_three_letters_plants_its_own_object() {
    let (mut state, n) = plant_with_leaves(["j_obj", "k_obj", "m_obj"]);
    for (name, tid) in [("j_obj", 11u32), ("k_obj", 22), ("m_obj", 33)] {
        publish_object(&mut state, name, tid);
    }
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let tex = column_v1(&state, &key, "texture_id");
    // A gramática pousa `JKM` em cada sítio, nessa ordem, então as folhas publicadas têm de ser
    // a repetição de `[11, 22, 33]`.
    //
    // ⛔⛔ **A 1.ª redacção deste gate perguntava se as três texturas APARECEM, e a mutação que
    // troca `J` com `K` SOBREVIVEU** — trocadas, as três continuam a aparecer. *Um teste de
    // PERTENÇA não vê uma permutação; o que a vê é a SEQUÊNCIA.* E o doc dele dizia, em voz
    // alta, que apanhava a ordem trocada.
    let leaves: Vec<f32> = tex.iter().skip(1).copied().collect();
    assert!(leaves.len() >= 6, "poucas folhas: {leaves:?}");
    assert_eq!(
        leaves.len() % 3,
        0,
        "cada sítio pousa as três letras: {leaves:?}"
    );
    for (i, t) in leaves.iter().enumerate() {
        let want = [11.0, 22.0, 33.0][i % 3];
        assert!(
            (t - want).abs() < 0.5,
            "a folha {i} devia ser a textura {want} e é {t} — as letras e os params estão \
             emparelhados pelo ÍNDICE, e a ordem trocou: {leaves:?}"
        );
    }
}

/// ⭐ **Uma letra SEM nome não planta nada** — e um nome que ninguém publicou também não.
///
/// ⚠️ *Não adivinha e não falha*: um nome pode ser escrito antes de a forma existir, e o quadro
/// seguinte tenta de novo. O que não pode é nascer um quad branco no sítio da folha.
#[test]
fn an_unnamed_or_unpublished_letter_plants_nothing() {
    for names in [["", "", ""], ["nao_existe", "", ""]] {
        let (mut state, n) = plant_with_leaves(names);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let geom = column_v1(&state, &key, "geometry_id");
        assert_eq!(
            geom.len(),
            1,
            "só a planta devia estar publicada, e vieram {} linhas ({names:?})",
            geom.len()
        );
    }
}

/// ⛔⛔ **O nome posto numa letra que a gramática não emite tem de ser DITO.**
///
/// Report do Enio (2026-08-30): *"só apareceu em seu exemplo, ao trocar o tipo de árvore não
/// aparece mais"*. Os moldes de planta trazem `J`, mas **uma gramática escrita à mão pode não
/// trazer letra nenhuma** — e aí o campo fica cheio, nada nasce, e o artista não tem como saber
/// porquê. *Um controlo com valor lá dentro e efeito nenhum parece ligado: é a pior espécie de
/// morto.*
///
/// ⚠️ **A metade que se gateia é a DECISÃO, não o canal** — o aviso sai no `stderr`, que um teste
/// não lê; por isso a lei vive numa função pura ([`super::unanswered_slots`]) e é ela que se mede.
#[test]
fn a_letter_with_a_name_and_no_anchor_is_reported() {
    let anchor = |slot: usize| super::Anchor {
        p: [0.0, 0.0],
        rot: 0.0,
        slot,
    };
    let names = |a: &str, b: &str, c: &str| [a.to_string(), b.to_string(), c.to_string()];

    // Nome posto, letra ausente da gramática ⇒ acusa.
    assert_eq!(
        super::unanswered_slots(&names("folha", "", ""), &[]),
        vec![0],
        "um nome sem ancora nenhuma tem de ser acusado"
    );
    // A letra existe ⇒ cala.
    assert!(
        super::unanswered_slots(&names("folha", "", ""), &[anchor(0)]).is_empty(),
        "com a ancora la' o aviso seria ruido"
    );
    // ⚠️ **Por SLOT, nunca «há âncoras?»** — uma gramática com `J` e um nome em `K` é exactamente
    // o caso do report, e uma régua que só perguntasse «esta planta tem âncoras?» ficaria muda.
    assert_eq!(
        super::unanswered_slots(&names("folha", "flor", ""), &[anchor(0)]),
        vec![1],
        "o slot que tem ancora cala e o que nao tem acusa"
    );
    // Campo vazio nunca acusa: não pedir objecto nenhum é o estado normal.
    assert!(super::unanswered_slots(&names("", "", ""), &[]).is_empty());
}
