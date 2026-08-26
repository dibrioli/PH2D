//! Gates da cena `=9` — o ESTILO DO SINK (doc 89, folha 17).
//!
//! ⚠️ **Esta cena NÃO se coze num teste**, e a ausência é a decisão: o `source.object`
//! resolve um objecto pelo NOME numa cena de ECS viva, e a estrela dela só ganha entidade
//! depois do `vec_entities::sync` — num arnês headless a fonte emite vazio e toda medição
//! de posição leria `0` linhas, que passa por qualquer barra.
//!
//! O que se mede, então, é o que a cena **AUTORA**: o estilo de cada um dos oito sinks,
//! lido pela porta do produto (`ph2d_eval_motion::sink_style`) — a MESMA que as duas
//! rotas de render perguntam —, e a forma da cadeia de cada um. Um par que saísse igual
//! dos dois lados é uma fileira verde e muda, que é o modo de falha de uma cena de
//! conferência.
//!
//! ⚠️ **E as quatro fileiras não são a mesma espécie**: três mudam um PARAM do sink; a do
//! sub-UV muda um NÓ da cadeia, porque a célula é uma **coluna** e não um param. Um gate
//! que as tratasse igual teria de abrir uma excepção para uma delas — e uma excepção
//! dentro de um gate é onde a afirmação morre.

use super::{CHIP, build_sink_style_graph};
use ph2d_eval_motion::sink_style;
use ph2d_nodegraph::graph::{Graph, NodeId};

fn scene() -> (Graph, Vec<NodeId>) {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut g = Graph::new();
    let sinks = build_sink_style_graph(&mut g);
    g.validate(&reg).expect("a cena e' bem-tipada");
    (g, sinks)
}

/// Os tipos de nó que alimentam `sink`, subindo pelas arestas.
fn upstream_types(g: &Graph, sink: NodeId) -> Vec<String> {
    let mut seen = vec![sink];
    let mut i = 0;
    while i < seen.len() {
        let cur = seen[i];
        for e in g.edges() {
            if e.to.0 == cur && !seen.contains(&e.from.0) {
                seen.push(e.from.0);
            }
        }
        i += 1;
    }
    seen.iter()
        .filter_map(|id| g.nodes().iter().find(|n| n.id == *id))
        .map(|n| n.type_name.clone())
        .collect()
}

/// **A CENA MONTA OS OITO SINKS, e ela é bem-tipada.**
///
/// ⚠️ O `validate` é metade do gate: um param que o manifesto não declara faz o cook
/// recusar o GRAFO INTEIRO — foi o que derrubou três demos na integração do
/// `motion.color_ramp`.
#[test]
fn the_sink_style_scene_builds_four_pairs() {
    let (_, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
}

/// ⭐⭐⭐ **AS TRÊS FILEIRAS DE PARAM DIFEREM EM EXACTAMENTE UM CAMPO, e é o delas.**
///
/// ⚠️ **A metade que quase ficou de fora é o «exactamente UM»**: uma cena em que o lado
/// direito mudasse dois params ao mesmo tempo mostraria uma diferença — e o Enio não
/// saberia a qual dos dois atribuí-la. É a mesma lei que faz o `seed` ser o mesmo nos dois
/// lados de todo par desta conferência.
#[test]
fn every_param_row_changes_exactly_one_field_and_it_is_its_own() {
    let (g, sinks) = scene();
    let st = |k: usize| sink_style(&g, sinks[k]);
    // (fileira, qual campo) — 0 pivô · 1 sampling · 2 ordem.
    for (row, which) in [(0usize, 0usize), (2, 1), (3, 2)] {
        let (l, r) = (st(row * 2), st(row * 2 + 1));
        if row == 2 {
            // ⚠️ **A FILEIRA DO FILTRO É A EXCEPÇÃO, e ela tem motivo.** O valor de
            // omissão é `Project` — *o que o projecto disser* —, e um lado de um par
            // não pode ser uma resposta que muda com uma preferência: se o projecto
            // estivesse em `Nearest`, o par sairia IGUAL e mudo. Por isso o lado
            // esquerdo declara `Linear` explicitamente. É o mesmo cuidado do `seed`
            // partilhado: o que muda entre os dois lados tem de ser UMA coisa, e ela
            // tem de estar na cena e não nas preferências de quem a abre.
            let (f, _) = ph2d_render::RenderInstance::unpack_sampling(l.sampling);
            assert_eq!(f, 2, "o lado esquerdo do filtro declara `Linear`");
        } else {
            assert!(
                l.is_plain(),
                "fileira {row}: o lado ESQUERDO tem de ser o mundo de antes ({l:?})"
            );
        }
        let moved = [
            l.pivot != r.pivot,
            l.sampling != r.sampling,
            l.stream_order != r.stream_order,
            l.blend != r.blend,
        ];
        assert_eq!(
            moved.iter().filter(|m| **m).count(),
            1,
            "fileira {row}: o par tem de diferir em UM campo so' — {l:?} contra {r:?}"
        );
        assert!(moved[which], "fileira {row}: mexeu no campo errado ({r:?})");
    }
}

/// ⭐⭐ **A FILEIRA DO SUB-UV MUDA UM NÓ, NÃO UM PARAM** — e os dois sinks dela são
/// `PLAIN`, que é o ponto: a célula é uma **coluna de stream**, não uma decisão do sink.
#[test]
fn the_sub_uv_row_differs_by_a_node_and_both_its_sinks_are_plain() {
    let (g, sinks) = scene();
    for k in [2, 3] {
        assert!(
            sink_style(&g, sinks[k]).is_plain(),
            "o sub-UV nao e' um param do sink — os dois lados desenham PLAIN"
        );
    }
    assert!(
        !upstream_types(&g, sinks[2])
            .iter()
            .any(|t| t == "motion.sub_uv"),
        "o lado ESQUERDO mostra a arte inteira"
    );
    assert!(
        upstream_types(&g, sinks[3])
            .iter()
            .any(|t| t == "motion.sub_uv"),
        "o lado DIREITO tem de recortar a arte"
    );
}

/// **A fileira da ORDEM tem as DUAS texturas e a intercalação.**
///
/// ⚠️ Sem o `motion.sort` o stream sairia `A,A,A,B,B,B` — e aí a ordem das linhas e o
/// agrupamento por textura são a MESMA coisa, e o par sairia igual dos dois lados
/// **mesmo com o param certo**. É o gate que impede a fileira de ser verde e muda.
#[test]
fn the_sort_row_actually_interleaves_two_different_objects() {
    let (g, sinks) = scene();
    for k in [6, 7] {
        let types = upstream_types(&g, sinks[k]);
        assert_eq!(
            types.iter().filter(|t| *t == "source.object").count(),
            2,
            "a fileira da ordem precisa de DUAS fontes — uma textura so' nao mostra nada"
        );
        assert!(
            types.iter().any(|t| t == "motion.sort"),
            "sem o sort nao ha' alternancia"
        );
    }
    // E as duas fontes nomeiam objectos DIFERENTES (o nome é a referência inteira).
    let names: Vec<&String> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.object")
        .filter_map(|n| {
            g.node_text_param_overrides(n.id)
                .and_then(|m| m.get("object"))
        })
        .collect();
    assert!(
        names.iter().any(|n| n.as_str() == CHIP),
        "o segundo objecto tem de estar na cena, senao a mídia nao e' mista"
    );
}

/// **O FILTRO usa a MESMA célula dos dois lados.** Uma célula diferente faria o par
/// mostrar duas artes, e ninguém saberia a que atribuir a diferença.
#[test]
fn the_filter_row_magnifies_the_same_patch_on_both_sides() {
    let (g, sinks) = scene();
    let cell = |g: &Graph, k: usize| -> Vec<f32> {
        let mut ids: Vec<NodeId> = vec![sinks[k]];
        let mut i = 0;
        while i < ids.len() {
            let cur = ids[i];
            for e in g.edges() {
                if e.to.0 == cur && !ids.contains(&e.from.0) {
                    ids.push(e.from.0);
                }
            }
            i += 1;
        }
        ids.iter()
            .filter(|id| {
                g.nodes()
                    .iter()
                    .any(|n| n.id == **id && n.type_name == "motion.sub_uv")
            })
            .flat_map(|id| {
                ["cols", "rows", "cell", "stagger"].iter().map(move |p| {
                    g.node_param_overrides(*id)
                        .and_then(|m| m.get(*p))
                        .copied()
                        .unwrap_or(0.0)
                })
            })
            .collect()
    };
    assert_eq!(
        cell(&g, 4),
        cell(&g, 5),
        "os dois lados tem de recortar o MESMO pedaco"
    );
    assert!(
        !cell(&g, 4).is_empty(),
        "CONTROLE: a fileira do filtro recorta"
    );
}
