//! **NEM TODOS AO MESMO TEMPO** (`PH2D_GPU_COOK_DEMO=112`) — a cena de smoke do ciclo 4
//! ([doc 107](../../docs/Motion%20Nodes/107_ciclo_4_foco_os_campos.md)).
//!
//! ## O que se vê
//!
//! Um pano de peças iguais, e **uma mancha delas maior que as outras**. O que decide quais é um
//! `motion.falloff` — o campo mais simples da casa —, e ele nasce **fora do centro do pano** de
//! propósito.
//!
//! ## ⚠️ Por que o campo está deslocado, e por que isso é a cena inteira
//!
//! Com o campo centrado no pano, mexer no `Center X` só o faz **sair** — e a cena ensinaria que
//! um campo é um interruptor. Fora do centro, **arrastar a alça no canvas varre a mancha pelo
//! pano**, que é a coisa que o grupo inteiro existe para fazer. É a mesma armadilha que a cena
//! `=111` do ciclo 3 documenta: *uma cena que só mostra o caso em que as respostas coincidem é
//! uma cena que ensina que a escolha não importa.*
//!
//! ## O que ela está de facto a demonstrar
//!
//! 1. **O `motion.falloff` tem ALÇA DE CANVAS** (W1) — ele era o terceiro campo espacial e o
//!    único sem uma, sendo o que o artista encontra primeiro. Arrastar `Center X`/`Center Y` num
//!    slider é caçar a posição.
//! 2. **A forma decide a mancha**: `Circle` dá um disco, `Rect` um quadrado, `Linear` uma rampa.
//!    ⚠️ E o `Rotation` **aparece no cartão** nas duas que têm direcção — num círculo ele está
//!    escondido porque não move um texel.
//! 3. **O `field.remap` reescreve a mancha sem lhe tocar na forma** — a mesma zona, outra
//!    dureza de borda.
//! 4. **Os cartões dos campos espaciais falam a mesma língua** (W2): `Placement` e `Falloff`,
//!    no `field.box` como no `field.radial_sweep`.
//!
//! ⚠️ **Se o pano inteiro crescer por igual**, o campo não está a ser lido — e o sintoma é o do
//! `motion.scale` a ignorar a coluna `falloff`, não o do campo.

use crate::motion::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// O passo entre elementos, em unidades de MUNDO — o mesmo da cena `=111`, que é o que faz um
/// pano ler como pano em vez de um borrão.
const PASSO: f32 = 0.03;
/// O tamanho de cada peça (o sink desenha um quad de tamanho `1`).
const PECA: f32 = 0.02;
/// 80 × 80 = 6 400 peças.
const LADO: f32 = 80.0;
/// A largura do pano, DERIVADA — nunca digitada duas vezes.
const LARGURA: f32 = LADO * PASSO;

/// O raio do campo: **um quarto da largura do pano**, o maior que ainda deixa ver o pano por
/// fora da mancha nos quatro lados quando ela está no centro.
const RAIO: f32 = LARGURA * 0.25;

/// Onde o campo nasce — ver o cabeçalho. ⚠️ **É `0,3 ×` a largura**, o suficiente para a mancha
/// ficar claramente de um lado e ainda inteira dentro do pano (com `0,5` ela sairia pela borda e
/// o artista veria meia mancha, que ensina outra coisa).
const DESLOCAMENTO: f32 = LARGURA * 0.3;

/// Quanto a mancha cresce. `1` seria a identidade; `2,2` é grande o bastante para se ver a
/// mancha de relance e pequeno o bastante para as peças não se fundirem num borrão.
const CRESCIMENTO: f32 = 2.2;

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![
        Caption::new([DESLOCAMENTO, RAIO * 1.4], "arraste ESTA alca"),
        Caption::new(
            [-LARGURA * 0.5, -LARGURA * 0.62],
            "Falloff ▸ Shape: Circle -> Rect",
        ),
    ]
}

/// `grid → falloff → remap → scale → output`.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", LADO);
    g.set_param(grid, "cols", LADO);
    g.set_param(grid, "gap_x", PASSO);
    g.set_param(grid, "gap_y", PASSO);

    // ⭐ O CAMPO, fora do centro — a razão de existir desta cena. `Circle` é o default, e o
    // passo 3 do smoke troca-o por `Rect` para o `Rotation` aparecer no cartão.
    let campo = g.add_node("motion.falloff");
    g.set_param(campo, "center_x", DESLOCAMENTO);
    g.set_param(campo, "center_y", 0.0);
    g.set_param(campo, "radius", RAIO);

    // A CURVA da mancha — a mesma zona, outra dureza de borda. Nasce no neutro para o passo do
    // smoke ter o que mexer.
    let remap = g.add_node("field.remap");

    // Quem LÊ o campo. ⚠️ O `motion.scale` lê a coluna `falloff` como `1 + (amount − 1)·f`, então
    // fora da mancha ele é a identidade AO BIT e a peça fica exactamente como o `fit` a deixou.
    let cresce = g.add_node("motion.scale");
    g.set_param(cresce, "amount", CRESCIMENTO);

    // A peça — sem isto o pano é um borrão sólido.
    let fit = g.add_node("motion.scale");
    g.set_param(fit, "amount", PECA);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, fit, campo, remap, cresce, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                x: 40.0 + i as f32 * 172.0,
                y: 140.0,
            },
        );
    }

    for (from, to) in [
        (grid, fit),
        (fit, campo),
        (campo, remap),
        (remap, cresce),
        (cresce, out),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    g.validate(reg).ok()?;
    Some(vec![out])
}
