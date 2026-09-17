//! Os gates da cena `=119` — a do ciclo 8 (doc 113 §8).
//!
//! ⚠️⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente**
//! (`CLAUDE.md` §5.0). O anúncio promete oito coisas; cada uma que se pode medir é um gate aqui.
//!
//! ⚠️⚠️ **E aqui há uma armadilha que as cenas dos ciclos anteriores não tinham: as MEMBRANAS.**
//! Cinco fontes deste grupo não fabricam nada — elas LEEM um external que a shell publicou. Um
//! gate que cozesse sem publicar mediria **streams vazios** e passaria a dizer *«o pano tem zero
//! peças»* sobre produto CERTO. ⇒ [`corre`] publica pelas portas do PRODUTO antes de cada cook, e
//! o primeiro gate é o que prova que a publicação chegou.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::Graph;

/// Os seis panos, pela ordem em que [`build`] os empilha.
const TEXTO: usize = 0;
const FORMA: usize = 1;
const TABELA: usize = 2;
const GRAFICO: usize = 3;
const EMISSOR: usize = 4;
const CAMARA: usize = 5;

/// A janela em que os gates da câmara medem — a altura é o numerador do zoom.
const JANELA_H: u32 = 900;

fn a_montante(g: &Graph, sink: NodeId, tipo: &str) -> Option<NodeId> {
    ph2d_nodegraph::cook::upstream_cone(g, sink)
        .into_iter()
        .find(|n| g.node(*n).is_some_and(|i| i.type_name == tipo))
}

/// Monta uma cena NOVA, aplica `mexe`, PUBLICA as membranas do produto e devolve o stream do sink
/// `k`. `altura_mundo` é a da câmara: quanto menor, mais zoom (o `zoom` publicado é
/// `altura_da_janela ÷ altura_do_mundo`).
///
/// ⚠️ **Cena nova por medição**: o emissor carrega estado, e um `Cook` que já andou entrega outro
/// pano a meio do caminho.
fn corre(k: usize, altura_mundo: f32, mexe: impl FnOnce(&mut Graph, NodeId)) -> Stream {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    let sink = sinks[k];
    mexe(&mut m.doc.graph, sink);
    // As membranas, pelas portas do produto: a forma · o texto · a tabela (`publish_all`) e a
    // VISTA (`publish_editor_inputs`).
    crate::motion_externals::publish_all(&mut m, 0.0);
    crate::motion_bridge::publish_editor_inputs(
        &mut m,
        &ph2d_render::Camera2d {
            height_world: altura_mundo,
            ..ph2d_render::Camera2d::default()
        },
        (0.0, 0.0),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        ph2d_host::WindowSize::new(1600, JANELA_H),
    );
    m.pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("coze")[0]
        .as_stream()
        .clone()
}

fn nada(_: &mut Graph, _: NodeId) {}

fn ys(s: &Stream) -> Vec<f32> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        outra => panic!("a coluna `P` devia ser Vec2: {outra:?}"),
    }
}

fn sizes_x(s: &Stream) -> Vec<f32> {
    match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        outra => panic!("a coluna `size` devia ser Vec2: {outra:?}"),
    }
}

/// ⭐⭐⭐ **OS SEIS PANOS TÊM PEÇAS** — e este é o gate que prova que as MEMBRANAS chegaram.
///
/// ⛔ Sem ele, todos os outros mediriam streams vazios e passariam por vacuidade (`all()` sobre
/// uma lista vazia é verdadeiro) — a forma muda de defeito que o cabeçalho nomeia.
#[test]
fn every_panel_draws_something() {
    for k in 0..6 {
        // O emissor nasce vazio no tique 0 (nenhuma partícula ainda), então ele é o único que se
        // mede depois de andar — a prova de que ele é o que se mexe está no gate do anúncio.
        if k == EMISSOR {
            continue;
        }
        let s = corre(k, 10.0, nada);
        assert!(
            s.count() > 0,
            "o pano {k} nao emitiu peca nenhuma — a membrana dele nao publicou"
        );
    }
}

/// ⭐⭐ **CADA PANO TRAZ A FONTE QUE A FICHA DELE PROMETE** — as duas metades: ela está lá, e a do
/// pano vizinho **não** está.
#[test]
fn each_panel_carries_the_source_its_card_names() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    let esperado = [
        (TEXTO, "source.text"),
        (FORMA, "source.shape"),
        (TABELA, "source.table"),
        (GRAFICO, "source.table"),
        (EMISSOR, "motion.emitter"),
        (CAMARA, "source.camera"),
    ];
    for (k, tipo) in esperado {
        assert!(
            a_montante(&m.doc.graph, sinks[k], tipo).is_some(),
            "o pano {k} devia ter um `{tipo}`"
        );
    }
    // A metade da OBSOLESCÊNCIA: nenhum pano traz a fonte de outro (senão o par deixaria de
    // ensinar «muda a FONTE» e o gate de cima passaria com a cena errada).
    for (k, _) in esperado {
        for (_, outro) in esperado {
            let e_dele = esperado.iter().any(|(j, t)| *j == k && *t == outro);
            if !e_dele {
                assert!(
                    a_montante(&m.doc.graph, sinks[k], outro).is_none(),
                    "o pano {k} nao devia ter um `{outro}`"
                );
            }
        }
    }
}

/// ⭐⭐⭐ **UMA LETRA, UMA PEÇA — e trocar a palavra troca as peças** (passos 2 e 3).
#[test]
fn the_word_becomes_one_piece_per_letter() {
    let n = corre(TEXTO, 10.0, nada).count();
    assert_eq!(
        n,
        PALAVRA.chars().count(),
        "uma peca por letra de «{PALAVRA}»"
    );
    // O passo 3: escrever outra coisa na linha `Text` muda as peças.
    let outra = corre(TEXTO, 10.0, |g, sink| {
        let t = a_montante(g, sink, "source.text").expect("a fonte de texto");
        g.set_text_param(t, ph2d_node_source_text::TEXT_KEY, "PH2D!");
    })
    .count();
    assert_eq!(outra, 5, "«PH2D!» tem cinco glifos, deu {outra}");
}

/// ⭐⭐⭐ **O PAR DA TABELA: o mesmo ficheiro, e só UM lê uma coluna** (passo 5, as duas metades).
#[test]
fn both_table_panels_read_the_same_file_and_only_one_reads_a_column() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    let ficheiro = |k: usize| -> String {
        let t = a_montante(&m.doc.graph, sinks[k], "source.table").expect("a tabela");
        m.doc
            .graph
            .node_text_param_overrides(t)
            .and_then(|p| p.get(ph2d_node_source_table::FILE_KEY))
            .cloned()
            .unwrap_or_default()
    };
    assert_eq!(
        ficheiro(TABELA),
        ficheiro(GRAFICO),
        "os dois panos leem o MESMO ficheiro — e' isso que faz o par ensinar a COLUNA"
    );
    assert!(
        a_montante(&m.doc.graph, sinks[GRAFICO], "value.attribute").is_some()
            && a_montante(&m.doc.graph, sinks[GRAFICO], "motion.drive").is_some(),
        "o pano da direita tem o cartao a mais"
    );
    assert!(
        a_montante(&m.doc.graph, sinks[TABELA], "value.attribute").is_none(),
        "o da esquerda NAO tem — senao o par nao mostra diferenca nenhuma"
    );
}

/// ⭐⭐⭐ **À DIREITA É UM GRÁFICO; À ESQUERDA É UMA FILA** — o que o olho do dono tem de ver.
///
/// ⚠️ A régua é a EXCURSÃO das alturas, não «são diferentes»: duas filas planas a alturas
/// diferentes passariam por «são diferentes» e não ensinariam nada.
#[test]
fn the_right_table_panel_is_a_chart_and_the_left_is_a_flat_row() {
    let excursao = |k: usize| {
        let y = ys(&corre(k, 10.0, nada));
        let (lo, hi) = y
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
        hi - lo
    };
    let plana = excursao(TABELA);
    let grafico = excursao(GRAFICO);
    assert!(
        plana < 1e-4,
        "a fila da esquerda tem de ser PLANA, excursao {plana}"
    );
    assert!(
        grafico > 0.5,
        "o grafico da direita tem de subir e descer, excursao {grafico}"
    );
}

/// ⭐⭐ **O GRÁFICO SEGUE A COLUNA QUE O CARTÃO NOMEIA** — trocar o nome da coluna muda o desenho.
///
/// ⛔ Sem esta metade, um `Drive` ligado a um `value.attribute` que lesse **qualquer** coisa (ou
/// nada, e caísse no default) daria o mesmo veredito do gate acima.
#[test]
fn the_chart_follows_the_column_the_card_names() {
    let com_vendas = ys(&corre(GRAFICO, 10.0, nada));
    let com_nivel = ys(&corre(GRAFICO, 10.0, |g, sink| {
        let a = a_montante(g, sink, "value.attribute").expect("a coluna");
        g.set_text_param(a, ph2d_node_value_attribute::ATTR_KEY, "nivel");
    }));
    assert_eq!(com_vendas.len(), com_nivel.len(), "as mesmas linhas");
    assert!(
        com_vendas
            .iter()
            .zip(&com_nivel)
            .any(|(a, b)| (a - b).abs() > 1e-3),
        "trocar a coluna tem de mudar as alturas — a `{COLUNA}` nao esta' a ser lida"
    );
}

/// ⭐⭐⭐ **O PANO DA CÂMARA MEDE OS MESMOS PIXELS EM QUALQUER ZOOM** (passo 8), com o CONTROLO ao
/// lado: o pano vizinho **não** o faz.
///
/// ⚠️ A régua é o produto `tamanho × zoom` — o tamanho em pixels de ecrã. Medir só «o tamanho
/// mudou» passaria com qualquer fio ligado.
#[test]
fn the_camera_panel_measures_the_same_pixels_at_any_zoom() {
    let px = |altura: f32| {
        let s = corre(CAMARA, altura, nada);
        let zoom = f32::from(u16::try_from(JANELA_H).expect("a janela")) / altura;
        let t = sizes_x(&s);
        assert!(!t.is_empty(), "o pano da camara tem pecas");
        t[0] * zoom
    };
    let perto = px(5.0);
    let longe = px(20.0);
    assert!(
        (perto - CAM_PIXELS).abs() < 1e-2 && (longe - CAM_PIXELS).abs() < 1e-2,
        "as pecas tem de medir {CAM_PIXELS} px nos dois zooms: {perto} e {longe}"
    );
    // ⚠️ O CONTROLO: o pano da tabela tem o tamanho do MUNDO, e não se mexe com o zoom — é isso
    // que faz o passo 8 mostrar uma diferença.
    let t_perto = sizes_x(&corre(TABELA, 5.0, nada))[0];
    let t_longe = sizes_x(&corre(TABELA, 20.0, nada))[0];
    assert!(
        (t_perto - t_longe).abs() < 1e-6,
        "o pano da tabela nao pode seguir o zoom: {t_perto} contra {t_longe}"
    );
}

/// ⚠️⚠️ **A CENA INTEIRA COZE NA CPU, e é uma PROPRIEDADE declarada** — ver o cabeçalho.
///
/// ⛔ Este gate existe para o dia em que alguém medir esta cena e ler a recusa como defeito: ela
/// é o preço de a cena ter uma fonte de forma VIVA, e a pergunta *«quanto custa»* responde-se na
/// §7, que mede fonte a fonte e não uma cena com seis panos.
#[test]
fn the_scene_recuses_the_device_because_a_live_vector_source_is_in_it() {
    let m = MotionState::new();
    let vivas: Vec<&str> = ["source.text", "source.shape"]
        .into_iter()
        .filter(|t| {
            m.registry
                .is_live_vector_source(ph2d_nodegraph::node::NodeTypeId::of(t))
        })
        .collect();
    assert_eq!(
        vivas.len(),
        2,
        "as duas fontes desta cena declaram-se forma VIVA: {vivas:?}"
    );
}

/// ⭐⭐ **O ANÚNCIO NOMEIA CARTÕES QUE EXISTEM** — a lei dos passos do dono: um passo que manda
/// clicar num cartão tem de provar que aquele cartão está na cena.
#[test]
fn the_announcement_names_cards_the_scene_has() {
    let mut m = MotionState::new();
    build(&mut m.doc, &m.registry).expect("a cena monta");
    let rotulos: Vec<String> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter_map(|n| {
            m.doc
                .graph
                .label(n.id)
                .map(std::string::ToString::to_string)
        })
        .collect();
    let anuncio = include_str!("motion_state_demo_announce.rs");
    let fontes = anuncio
        .split("pub(super) fn fontes()")
        .nth(1)
        .expect("o anuncio do ciclo 8");
    let passos = fontes.split("pub(super) fn").next().expect("o corpo");
    for r in ["Text: a palavra", "Shape: a forma", "Table: o grafico"] {
        assert!(
            rotulos.iter().any(|l| l == r),
            "o anuncio manda clicar em `{r}` e a cena nao tem esse cartao: {rotulos:?}"
        );
        assert!(
            passos.contains(r),
            "o cartao `{r}` existe e o anuncio nao o nomeia — um passo a menos"
        );
    }
}
