//! ⭐⭐⭐ **O DIVISOR É DA PEÇA INTEIRA E A MARCHA ANDA AOS PEDAÇOS** — a sonda que mede se isso é
//! uma alavanca ou uma ilusão.
//!
//! # A pergunta
//!
//! Um deformador divide o campo pelo pior factor de Lipschitz **da peça toda**, e a marcha paga
//! `σ×` mais passos em todo lado por causa do pior sítio. ⚠️ Mas o traçado já **especializa a árvore
//! por ladrilho × fatia de profundidade** (W56e) — e a fita dessa região só é avaliada **dentro**
//! dela. ⇒ o divisor dela podia ser o da região, não o da peça.
//!
//! ⭐ **E é seguro por construção:** dividir um campo por uma constante positiva **não move o zero
//! nem a normal** (`∇(f/σ)/‖∇(f/σ)‖ = ∇f/‖∇f‖`). Duas regiões vizinhas com divisores diferentes
//! desenham exactamente a mesma superfície — o que muda é só o tamanho do passo.
//!
//! ⛔ **A recusa que existe responde a OUTRA pergunta:** a de 2026-09-01 mediu **octantes do
//! envelope** (`0,0416` contra `0,0405`, ganho nenhum). Um octante é `1/8` da peça; uma região de
//! marcha é `~1/200` dela, e a diferença entre as duas perguntas é a razão de esta sonda existir.
//!
//! # ⛔⛔⛔ O VEREDITO (2026-09-02): REFUTADA, por DOIS mecanismos
//!
//! Peça `[Bend, Twist, Taper]` sobre uma barra `0,3 × 0,3 × 0,9`, divisor da peça inteira `271,2`:
//!
//! | granularidade | regiões | pior | mediana | média | ganho na média | relógio |
//! |---|---:|---:|---:|---:|---:|---:|
//! | octante (a recusa de 01/09) | `8` | `306,5` | `306,5` | `299,9` | **`0,90×`** | `26 ms` |
//! | `1/32` da peça | `32` | `348,3` | `263,8` | `275,0` | `0,99×` | `105 ms` |
//! | ~ladrilho × fatia | `256` | `448,4` | `150,0` | `179,5` | `1,51×` | `856 ms` |
//! | `TILE = 24` num quadro de `320` | `676` | `470,4` | `137,0` | `171,6` | `1,58×` | **`2 176 ms`** |
//!
//! ⛔ **1. O RELÓGIO.** O bound é uma ramificação-e-poda de `3,2 ms` por chamada; à granularidade que
//! de facto compra alguma coisa são `676` chamadas por quadro = **`2,2 s`**, contra um orçamento de
//! `16,7 ms`. Mesmo com a cache de fitas a cortar para `~44` compilações por quadro seriam `140 ms`
//! — `8×` o quadro inteiro — para comprar `1,58×` no tamanho do passo. *Uma optimização que custa
//! mais do que o trabalho que poupa não é uma optimização.*
//!
//! ⭐⭐⭐ **2. E O DIVISOR NÃO É MONÓTONO NO DOMÍNIO — uma REGIÃO pode ser PIOR que a peça inteira.**
//! O pior octante pede `306,5` contra os `271,2` da peça, e a pior região de ladrilho pede `470,4`.
//! O mecanismo é geométrico: **o estiramento de uma deformação cresce com a distância ao eixo dela,
//! e a bola da peça está CENTRADA nesse eixo** — ela ganha por simetria o que uma sub-bola do bordo
//! não tem. Medido por deformador sozinho, o efeito é maior onde a lei é mais não-linear:
//!
//! | pilha | peça inteira | pior região | média | ganho |
//! |---|---:|---:|---:|---:|
//! | `[Bend]` | `3,29` | **`10,00`** | `5,14` | `0,64×` |
//! | `[Twist]` | `3,09` | `2,65` | `2,19` | `1,41×` |
//! | `[Taper]` | `2,39` | `2,35` | `2,18` | `1,10×` |
//! | `[Bend, Twist]` | `21,40` | **`48,06`** | `34,11` | `0,63×` |
//!
//! ⇒ *«dividir para conquistar» supõe que a resposta do todo é o máximo das respostas das partes.
//! Aqui não é: a resposta do todo usa uma simetria que as partes não têm*, e para o `Bend` e para
//! o par `[Bend, Twist]` localizar **piora** — a média fica acima do número da peça inteira.
//!
//! ⚠️ **Fica válido o que a sonda também diz:** o ORÇAMENTO de passos continua a ser governado pelo
//! pior, e o CUSTO pela média — são duas contas diferentes sobre o mesmo número, e quem reabrir isto
//! tem de dizer qual das duas está a atacar.
//!
//! ```text
//! cargo test -p ph2d-field-eval --release --test the_divisor_is_global_but_the_march_is_local -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::bounds::{self, Ball};
use ph2d_field_eval::hybrid::Registry;

const HALF: [f32; 3] = [0.30, 0.30, 0.90];
const BANDA: (f32, f32, f32) = (-2.0, 2.0, 0.1);

fn mods() -> Vec<Unary> {
    use ph2d_field::mods::{BEND_AXIS, TAPER_AXIS, TWIST_AXIS};
    vec![
        Unary::Bend {
            turns: 0.35,
            lower: BANDA.0,
            upper: BANDA.1,
            falloff: BANDA.2,
            axis: BEND_AXIS,
        },
        Unary::Twist {
            turns: 0.60,
            lower: BANDA.0,
            upper: BANDA.1,
            falloff: BANDA.2,
            axis: TWIST_AXIS,
        },
        Unary::Taper {
            slope: 0.60,
            axis: TAPER_AXIS,
        },
    ]
}

fn doc_with(mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: HALF,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// O divisor que a árvore de facto cobra sobre uma bola — a MESMA porta que o produto usa.
fn divisor(mods: &[Unary], ball: Ball) -> f64 {
    let produto: f64 = ph2d_field_eval::stack_divisor_factors(mods, ball)
        .iter()
        .product();
    match ph2d_field_eval::stack_lipschitz_probe(mods, ball) {
        Some(novo) if novo < produto => novo,
        _ => produto,
    }
}

/// Uma sub-bola da caixa de `ball`, com o canto em `lo` e o tamanho `size`.
fn sub(lo: [f32; 3], size: [f32; 3]) -> Ball {
    let centro = std::array::from_fn(|e| lo[e] + size[e] * 0.5);
    let half: [f32; 3] = std::array::from_fn(|e| size[e] * 0.5);
    let raio = half[0].hypot(half[1]).hypot(half[2]);
    Ball::of(centro, raio, half)
}

/// ⭐⭐⭐ **A MEDIÇÃO: o divisor da peça contra o divisor de uma região de marcha.**
///
/// A grelha percorre a caixa da peça em blocos com a proporção de um ladrilho × fatia — a
/// especialização do traçado corta em `TILE = 24` px sobre um quadro de `320` (`~1/13` do lado) e
/// em `SLABS = 4` fatias de profundidade.
#[test]
#[ignore = "sonda: imprime a tabela que decide se o divisor por região é uma alavanca"]
fn measure_the_divisor_per_march_region() {
    let d = doc_with(mods());
    let reg = Registry::default();
    let local = bounds::local_balls(&d, &reg)[0].expect("a bola do nó");
    let (lo, hi) = local.aabb();
    let inteiro = divisor(&mods(), local);
    println!("\n  caixa do nó: lo {lo:?}  hi {hi:?}");
    println!("  divisor da PEÇA INTEIRA = {inteiro:.3}\n");

    for (nx, ny, nz, nome) in [
        (2, 2, 2, "octante (a recusa de 01/09)"),
        (4, 4, 2, "1/32 da peça"),
        (8, 8, 4, "~ladrilho × fatia"),
        (13, 13, 4, "TILE=24 num quadro de 320"),
    ] {
        let size: [f32; 3] = [
            (hi[0] - lo[0]) / nx as f32,
            (hi[1] - lo[1]) / ny as f32,
            (hi[2] - lo[2]) / nz as f32,
        ];
        let mut vals = Vec::new();
        let t0 = std::time::Instant::now();
        for i in 0..nx {
            for j in 0..ny {
                for k in 0..nz {
                    let canto = [
                        lo[0] + size[0] * i as f32,
                        lo[1] + size[1] * j as f32,
                        lo[2] + size[2] * k as f32,
                    ];
                    vals.push(divisor(&mods(), sub(canto, size)));
                }
            }
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        vals.sort_by(f64::total_cmp);
        let n = vals.len();
        let mediana = vals[n / 2];
        let media = vals.iter().sum::<f64>() / n as f64;
        println!(
            "  {nome:28} | {n:4} regiões | pior {:7.3} | mediana {mediana:7.3} | média {media:7.3} \
             | ganho na média {:5.2}× | {ms:6.1} ms",
            vals[n - 1],
            inteiro / media.max(1e-9)
        );
    }
    println!(
        "\n  ⚠️ O que a marcha paga é ~proporcional à MÉDIA sobre as regiões que os raios visitam,\n  \
         e não ao pior — o pior manda no ORÇAMENTO de passos, que é outra conta.\n"
    );
}

/// ⭐ A mesma pergunta para cada deformador **sozinho** — para saber qual deles é que tem o
/// gradiente concentrado num canto, que é o que a localização compra.
#[test]
#[ignore = "sonda"]
fn measure_which_deformer_is_local() {
    let todos = mods();
    let reg = Registry::default();
    for (nome, m) in [
        ("[Bend]", vec![todos[0]]),
        ("[Twist]", vec![todos[1]]),
        ("[Taper]", vec![todos[2]]),
        ("[Bend, Twist]", todos[..2].to_vec()),
        ("[Bend, Twist, Taper]", todos.clone()),
    ] {
        let d = doc_with(m.clone());
        let local = bounds::local_balls(&d, &reg)[0].expect("a bola");
        let (lo, hi) = local.aabb();
        let inteiro = divisor(&m, local);
        let size: [f32; 3] =
            std::array::from_fn(|e| (hi[e] - lo[e]) / if e == 2 { 4.0 } else { 8.0 });
        let mut vals = Vec::new();
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..4 {
                    let canto = [
                        lo[0] + size[0] * i as f32,
                        lo[1] + size[1] * j as f32,
                        lo[2] + size[2] * k as f32,
                    ];
                    vals.push(divisor(&m, sub(canto, size)));
                }
            }
        }
        vals.sort_by(f64::total_cmp);
        let media = vals.iter().sum::<f64>() / vals.len() as f64;
        println!(
            "  {nome:22} | inteiro {inteiro:7.3} | pior região {:7.3} | média {media:7.3} | ganho {:5.2}×",
            vals[vals.len() - 1],
            inteiro / media.max(1e-9)
        );
    }
}
