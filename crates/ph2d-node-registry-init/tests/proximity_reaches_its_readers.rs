//! **A CADEIA QUE O `motion.proximity` EXISTE PARA SERVIR** (doc 89, folha 03 linhas 42 e 61).
//!
//! Os gates da própria crate provam a LEI (a fracção, a contagem, a independência de ordem) e
//! são **cegos ao catálogo**: eles não sabem dizer se a coluna que o nó escreve é a que alguém
//! lê, nem se os três modos do *Push Apart* do C4D — **Push · Scale · Hide** — de facto saem
//! por composição. Esta crate é a única que vê os 124 nós de uma vez, e é aqui que a wave se
//! prova.
//!
//! ⚠️ **Cada gate traz o CONTROLE ao lado**, e ele é a metade que importa: *a sobreposição
//! sumiu* é satisfeito por qualquer coisa que encolha tudo a zero, enquanto *ela ESTAVA lá e
//! deixou de estar* nomeia exactamente o buraco que a wave fecha.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Os canais do `motion.drive`. Eles são `pub(crate)` na crate dele, então o número vive aqui
/// como literal — e o que impede um literal errado de passar em silêncio é o próprio gate
/// afirmar a CONSEQUÊNCIA (o `size` encolheu, a contagem caiu), nunca o número.
const CH_SIZE: f32 = 3.0;
/// Idem — `channel_column` mapeia `5 => "falloff"`.
const CH_FALLOFF: f32 = 5.0;
/// `Combine::Set` (`1 => Combine::Set` no `channel.rs`).
const COMBINE_SET: f32 = 1.0;
/// `Combine::Multiply`.
const COMBINE_MULTIPLY: f32 = 2.0;
/// `motion.cull`: `MODE_FALLOFF` = 1 (mantém quem tem `falloff ≥ amount`; `invert` troca).
const CULL_MODE_FALLOFF: f32 = 1.0;

/// O raio de disco de toda esta fixture. Contra o passo de `0,25` da grade abaixo ele dá
/// `min_dist = 0,6`: quase todo par nasce sobreposto.
const RADIUS: f32 = 0.3;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, from_port: u16, to: NodeId, to_port: u16) {
    g.connect(Edge {
        from: (from, from_port),
        to: (to, to_port),
        delayed: false,
    })
    .expect("edge");
}

/// Uma grade APERTADA: passo `0,25` contra `min_dist = 0,6`. É a fixture que CONTÉM o
/// fenómeno — numa grade folgada as três cadeias abaixo seriam no-ops e os gates ficariam
/// verdes sem medir nada.
fn crowded(g: &mut Graph) -> NodeId {
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 6.0);
    g.set_param(src, "cols", 6.0);
    g.set_param(src, "gap_x", 0.25);
    g.set_param(src, "gap_y", 0.25);
    src
}

/// Uma nuvem **MISTA**: alguns pares amontoados, outros folgados.
///
/// ⚠️ Ela existe porque a `crowded` **não contém o fenómeno do Hide**: numa grade uniforme de
/// passo `0,25` contra `min_dist = 0,6` TODO elemento toca alguém, então o modo Hide apaga a
/// cena inteira e o gate não consegue distinguir *"apaga quem se sobrepõe"* de *"apaga tudo"*.
/// Um `motion.scatter` de 40 pontos em `4×4` tem espaçamento médio na ordem do diâmetro, logo
/// nasce com as duas espécies — e é determinístico (`seed`).
fn mixed(g: &mut Graph) -> NodeId {
    let src = g.add_node("motion.scatter");
    g.set_param(src, "count", 40.0);
    g.set_param(src, "width", 4.0);
    g.set_param(src, "height", 4.0);
    g.set_param(src, "seed", 3.0);
    src
}

fn proximity(g: &mut Graph, input: NodeId) -> NodeId {
    let n = g.add_node("motion.proximity");
    g.set_param(n, "radius", RADIUS);
    wire(g, input, 0, n, 0);
    n
}

/// `<stream> → value.attribute(<coluna>)`, pelo canal do picker: o nome é TEXT PARAM.
fn read_column_as_value(g: &mut Graph, input: NodeId, column: &str) -> NodeId {
    let attr = g.add_node("value.attribute");
    g.set_param(attr, "mode", 0.0);
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, column);
    wire(g, input, 0, attr, 0);
    attr
}

fn cook(g: &Graph, reg: &NodeRegistry, target: NodeId) -> Stream {
    let mut c = Cook::new();
    c.cook(g, reg, target, 0.0).expect("cooks")[0]
        .as_stream()
        .clone()
}

fn scalars(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **OS DOIS CANAIS DEIXAM DE DEVOLVER ZEROS.**
///
/// ⚠️ O controle é a metade que prova a wave: sem este nó, *"colore pela densidade"* e
/// *"esconde quem se sobrepõe"* — duas frases ordinárias de motion graphics — devolvem um
/// campo de zeros numa nuvem visivelmente amontoada, e o artista não tem como distinguir isso
/// de ter digitado o nome errado.
#[test]
fn the_neighbourhood_channels_were_zeros_and_this_node_fills_them() {
    let reg = registry();
    for column in ["neighbours", "overlap"] {
        let mut g = Graph::new();
        let src = crowded(&mut g);
        let control = read_column_as_value(&mut g, src, column);
        let field = scalars(&cook(&g, &reg, control), "v");
        assert!(!field.is_empty(), "a fixture tem elementos");
        assert!(
            field.iter().all(|v| *v == 0.0),
            "CONTROLE: sem o `motion.proximity` o canal `{column}` tem de ser zeros -- se ja \
             tinha valores, a fixture nao contem o fenomeno que a wave fecha"
        );

        let mut g = Graph::new();
        let src = crowded(&mut g);
        let prox = proximity(&mut g, src);
        let with = read_column_as_value(&mut g, prox, column);
        let field = scalars(&cook(&g, &reg, with), "v");
        let live = field.iter().filter(|v| **v > 0.0).count();
        assert!(
            live > field.len() / 2,
            "com o no', a maioria dos elementos tem `{column}`; {live} de {}",
            field.len()
        );
    }
}

/// **O MODO *SCALE* DO C4D É UMA COMPOSIÇÃO** — e a folha 03 (linha 61) prescrevia um param
/// `mode` dentro do `motion.collide`, enquanto o doc 63 §2.2 propunha um nó novo
/// (`motion.push_apart`). Com a vizinhança publicada, as duas propostas caem:
///
/// `proximity → value.attribute(Overlap) → value.map_range(0..1 → 1..0) → motion.drive(Size,
/// Multiply)`
///
/// ⚠️ O oráculo é uma SEGUNDA medição: o `overlap` que a cadeia leu descreve os tamanhos de
/// ANTES, então perguntar a ele se a wave funcionou seria ler o diagnóstico em vez do doente.
#[test]
fn the_scale_mode_is_a_composition_and_it_removes_every_overlap() {
    let reg = registry();

    // CONTROLE: a mesma nuvem, medida sem a cadeia de encolhimento.
    let mut g = Graph::new();
    let src = crowded(&mut g);
    let prox = proximity(&mut g, src);
    let before = scalars(&cook(&g, &reg, prox), "overlap");
    let overlapping = before.iter().filter(|v| **v > 0.0).count();
    assert!(
        overlapping > before.len() / 2,
        "CONTROLE: a nuvem TEM de nascer sobreposta; {overlapping} de {}",
        before.len()
    );

    // A cadeia.
    let mut g = Graph::new();
    let src = crowded(&mut g);
    let prox = proximity(&mut g, src);
    let attr = read_column_as_value(&mut g, prox, "overlap");
    // `1 − x` num nó: a faixa de entrada `0..1` mapeada para a saída `1..0`.
    let inv = g.add_node("value.map_range");
    g.set_param(inv, "in_lo", 0.0);
    g.set_param(inv, "in_hi", 1.0);
    g.set_param(inv, "out_lo", 1.0);
    g.set_param(inv, "out_hi", 0.0);
    wire(&mut g, attr, 0, inv, 0);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", CH_SIZE);
    g.set_param(drive, "mode", COMBINE_MULTIPLY);
    wire(&mut g, prox, 0, drive, 0);
    wire(&mut g, inv, 0, drive, 1);
    // A SEGUNDA medição, sobre os tamanhos novos.
    let after_n = proximity(&mut g, drive);
    let after = scalars(&cook(&g, &reg, after_n), "overlap");

    assert_eq!(after.len(), before.len(), "o Scale nao apaga ninguem");
    for (i, v) in after.iter().enumerate() {
        assert!(
            *v <= 1e-6,
            "o elemento {i} continua sobreposto por {v} depois do Scale"
        );
    }
    // ⚠️ E o CONTROLE do próprio Scale: os discos encolheram o JUSTO, não a nada — sem esta
    // metade, um `drive` que zerasse todo `size` passaria no laço acima.
    let sizes = match cook(&g, &reg, drive).get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        other => panic!("o `motion.drive(Size)` tem de escrever a coluna `size`: {other:?}"),
    };
    let smallest = sizes.iter().fold(f32::MAX, |a, s| a.min(s[0].abs()));
    assert!(
        smallest > 0.0,
        "nenhum disco pode encolher a nada; o menor veio {smallest}"
    );
}

/// **E O MODO *HIDE* TAMBÉM** — `proximity → value.attribute(Overlap) → motion.drive(Falloff,
/// Set) → motion.cull(Falloff, invert)`: quem se sobrepõe SAI da cena.
///
/// ⚠️ Repare que este é o modo que muda a CONTAGEM, que nesta engine é a família do
/// `StreamOp::Compact` com lei de contagem própria (ADR-0136). Enfiá-lo num solver de posição
/// teria alargado o `motion.collide` para *mover · redimensionar · apagar*; o `motion.cull` já
/// é o nó que apaga, e ele nem sabe que este nó existe.
#[test]
fn the_hide_mode_is_a_composition_and_it_drops_the_overlapping() {
    let reg = registry();

    let mut g = Graph::new();
    let src = mixed(&mut g);
    let prox = proximity(&mut g, src);
    let attr = read_column_as_value(&mut g, prox, "overlap");
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", CH_FALLOFF);
    g.set_param(drive, "mode", COMBINE_SET);
    wire(&mut g, prox, 0, drive, 0);
    wire(&mut g, attr, 0, drive, 1);
    let cull = g.add_node("motion.cull");
    g.set_param(cull, "mode", CULL_MODE_FALLOFF);
    // Mantém quem tem `falloff ≥ amount` — INVERTIDO, ou seja quem tem quase zero.
    g.set_param(cull, "amount", 1e-4);
    g.set_param(cull, "invert", 1.0);
    wire(&mut g, drive, 0, cull, 0);

    let full = cook(&g, &reg, prox).count();
    let kept = cook(&g, &reg, cull);
    assert!(
        kept.count() < full,
        "CONTROLE: a nuvem apertada tinha de perder elementos; {} de {full} sobreviveram",
        kept.count()
    );
    assert!(kept.count() > 0, "e nao pode perder TODOS: sobrou zero");

    // Os sobreviventes são exactamente os que não tocavam ninguém — re-medidos, dão zero.
    let after = proximity(&mut g, cull);
    let overlap = scalars(&cook(&g, &reg, after), "overlap");
    assert_eq!(overlap.len(), kept.count());
    assert!(
        overlap.iter().all(|v| *v == 0.0),
        "quem sobreviveu ao Hide nao pode estar sobreposto: {overlap:?}"
    );
}

/// **A LEI DE TOQUE É A DO `motion.collide`, e este gate mede as duas.**
///
/// Partilhar a lei não é zelo: é o que faz *o que o `proximity` reporta como sobreposto* ser
/// *o que o `collide` empurraria*. Duas definições de "tocar" na mesma biblioteca divergiriam
/// e o artista veria o Scale encolher um disco que o Push deixou em paz.
///
/// ⚠️ O oráculo não é comparar constantes — é varrer o `radius` e achar **o degrau em que cada
/// um dispara**. Um gate que lesse `r_i + r_j` dos dois fontes seria um espelho da lei; este
/// pergunta ao PRODUTO.
#[test]
fn the_touch_law_is_the_collides_own() {
    let reg = registry();
    // Uma fila com passo 0,5: o primeiro contacto acontece quando `2·r > 0,5`, ou seja
    // `r > 0,25`. A varredura passa por lá em passos de 0,01.
    let line = |g: &mut Graph| {
        let src = g.add_node("motion.grid");
        g.set_param(src, "rows", 1.0);
        g.set_param(src, "cols", 5.0);
        g.set_param(src, "gap_x", 0.5);
        g.set_param(src, "gap_y", 0.5);
        src
    };

    let mut first_prox: Option<usize> = None;
    let mut first_collide: Option<usize> = None;
    for step in 0..40 {
        let r = 0.05 + 0.01 * step as f32;

        let mut g = Graph::new();
        let src = line(&mut g);
        let prox = g.add_node("motion.proximity");
        g.set_param(prox, "radius", r);
        wire(&mut g, src, 0, prox, 0);
        let touches = scalars(&cook(&g, &reg, prox), "neighbours")
            .iter()
            .any(|c| *c > 0.0);
        if touches && first_prox.is_none() {
            first_prox = Some(step);
        }

        let mut g = Graph::new();
        let src = line(&mut g);
        let base = match cook(&g, &reg, src).get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("a grade emite `P`"),
        };
        let col = g.add_node("motion.collide");
        g.set_param(col, "radius", r);
        g.set_param(col, "iterations", 1.0);
        g.set_param(col, "strength", 1.0);
        wire(&mut g, src, 0, col, 0);
        let moved = match cook(&g, &reg, col).get("P") {
            Some(Column::Vec2(v)) => v
                .iter()
                .zip(&base)
                .any(|(a, b)| (a[0] - b[0]).abs() > 1e-6 || (a[1] - b[1]).abs() > 1e-6),
            _ => panic!("o collide emite `P`"),
        };
        if moved && first_collide.is_none() {
            first_collide = Some(step);
        }
    }

    assert_eq!(
        first_prox, first_collide,
        "os dois nos tem de comecar a agir no MESMO raio -- `proximity` no passo {first_prox:?} \
         e `collide` no passo {first_collide:?}"
    );
    assert!(
        first_prox.is_some(),
        "a varredura tem de ATRAVESSAR o limiar; nenhum dos dois disparou"
    );
}
