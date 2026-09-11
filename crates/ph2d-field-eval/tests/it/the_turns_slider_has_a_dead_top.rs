//! ⛔⛔⛔ **A METADE DE CIMA DO `Turns` DA DOBRA NÃO FAZ NADA** — e o número escrito **mente**.
//!
//! Perguntas do Enio, 2026-08-31: *«Porque chamar de Turn se não dá voltas? Porque o máximo é 0.5?
//! Porque não 1?»*. As duas têm a mesma resposta, e ela é medida.
//!
//! # O que o slider promete e o que ele entrega
//!
//! O `MAX_BEND_TURNS` é `0,5` **voltas por unidade de comprimento**, e nessa unidade o nome está
//! certo: a `0,266` voltas/unidade sobre uma banda de `1,0`, a peça vira `96°` — que é o arco da
//! foto. ⛔ **O que está errado é o número.** Na chapa dele (`0,400 × 0,997 × 0,063`):
//!
//! | `turns` pedido | `κ` pedido | `κ` entregue | voltas entregues |
//! |---:|---:|---:|---:|
//! | `0,25` | `1,5708` | `1,5708` | `0,2500` |
//! | `0,375` | `2,3562` | **`1,6727`** | **`0,2662`** |
//! | `0,50` | `3,1416` | **`1,6727`** | **`0,2662`** |
//! | `1,00` | `6,2832` | **`1,6727`** | **`0,2662`** |
//! | `2,00` | `12,5664` | **`1,6727`** | **`0,2662`** |
//!
//! ⇒ **`47 %` do curso do slider é inerte**, e no topo dele o número mente por `1,9×`. *Subir o
//! `MAX_BEND_TURNS` para `1` — a outra pergunta — só tornaria a zona morta maior.*
//!
//! # ⭐ Quem manda não é o `MAX_*`: é a PAREDE, e o `W` dela é a grandeza errada
//!
//! A parede (`stack_bend::bend_curvature`) satura em `κ·W = 0,9`, com `W = bend_reach(bola)` — o
//! **raio da bola de bordo**. Numa chapa alta e fina esse raio é dominado pela **altura**
//! (`0,4985`), e a grandeza que a parede quer é a meia-extensão na direcção em que a dobra
//! **deflecte** — aqui a espessura, `0,0315`. ⛔ **`17×` de diferença**: com a grandeza certa a
//! parede deixaria `κ ≤ 28,6` em vez de `1,67`, e o slider inteiro estaria vivo.
//!
//! ⛔⛔ **E não é uma linha de conserto:** `bounds::Ball` é uma **esfera** e não tem eixos, e a
//! marcha é presa à AABB dela — logo o `W` que o avaliador de facto percorre **é** o raio. A cura é
//! um bordo **alinhado aos eixos**, e ela paga duas dívidas de uma vez: esta e a faixa da banda
//! (`Span::Along`, que hoje sobra `15×` na mesma chapa). É wave própria.
//!
//! ⚠️ **A catraca abaixo só ENCOLHE** — quando aquela wave entrar, este ficheiro reprova e sai.

use ph2d_field::{Axis, FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::Field;

/// A chapa da foto do Enio, com a dobra dele.
fn chapa(turns: f32) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.200, 0.4985, 0.0315],
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    n.mods = vec![Unary::Bend {
        turns,
        lower: -0.5,
        upper: 0.5,
        falloff: 0.063,
        axis: Axis::Y,
    }];
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// Quão longe a peça deflecte — a régua de *«este ponto do slider ainda faz alguma coisa?»*.
///
/// ⚠️ **É a ponta em `+Z`**, que é para onde a dobra em `Y` deflecte (medido: com `axis = Y` o `X`
/// não se mexe e o `Z` cresce de `0,027` para `0,187`). *Uma régua no eixo errado leria zero em
/// toda a faixa e chamaria ao slider inteiro morto.*
fn ponta(turns: f32) -> f64 {
    let f = Field::new(&chapa(turns));
    let mut pior = f64::NEG_INFINITY;
    for i in 0..=120 {
        let z = -0.2 + 0.005 * f64::from(i);
        for j in 0..=40 {
            let y = -0.6 + 0.03 * f64::from(j);
            for k in 0..=16 {
                let x = -0.2 + 0.025 * f64::from(k);
                if f.at(x, y, z) <= 0.0 {
                    pior = pior.max(z);
                }
            }
        }
    }
    pior
}

/// ⛔ **A fracção MORTA do curso, medida em 2026-08-31.** Ela só encolhe — ver o doc do módulo.
const MORTO_TOLERADO: f64 = 0.50;

/// ⭐⭐⭐ **QUANTO DO SLIDER ESTÁ MORTO** — a catraca, com as duas metades.
///
/// ⛔⛔ **Prova de mutação:** tirar o `clamp` da `stack_bend::bend_curvature` (a parede) leva a
/// fracção morta de `0,47` a `0,00` — e mata o gate da imagem
/// (`ph2d_field_render::the_bend_draws_what_an_honest_march_draws`, `478` de `1 610` pixels). *A
/// parede paga-se; o que não se paga é o `W` que ela usa.*
#[test]
fn the_top_of_the_bend_turns_slider_is_measured_dead() {
    let passos = 40;
    let teto = f64::from(ph2d_field::mods::MAX_BEND_TURNS);
    #[allow(clippy::cast_possible_truncation)]
    let em = |i: i32| ponta((teto * f64::from(i) / f64::from(passos)) as f32);
    let cheio = em(passos);
    // O primeiro ponto do curso a partir do qual nada mais se move.
    let mut vivo_ate = passos;
    while vivo_ate > 0 && (em(vivo_ate - 1) - cheio).abs() < 1.0e-4 {
        vivo_ate -= 1;
    }
    let morto = f64::from(passos - vivo_ate) / f64::from(passos);
    assert!(
        morto <= MORTO_TOLERADO,
        "{:.0} % do curso do `Turns` não muda a peça (tolerado {:.0} %) — piorou",
        morto * 100.0,
        MORTO_TOLERADO * 100.0
    );
    // ⛔ **A METADE QUE FAZ A CATRACA DESCER**: quando o bordo passar a ser alinhado aos eixos, a
    // zona morta desaparece e esta linha manda apagar o ficheiro. *Uma catraca sem censo de
    // obsolescência não desce: ela vira licença.*
    assert!(
        morto > 0.05,
        "o `Turns` já não tem zona morta ({:.0} %) — APAGUE este ficheiro e a nota do \
         `MAX_BEND_TURNS` que o cita",
        morto * 100.0
    );
    // ⛔ **O CONTROLE**: se a régua lesse o mesmo em toda a faixa, as duas linhas acima passariam
    // por a peça não se estar a mexer de todo.
    assert!(
        (cheio - em(0)).abs() > 0.05,
        "a ponta mede {cheio} no topo e {} na base — a régua não está a ver a dobra",
        em(0)
    );
}
