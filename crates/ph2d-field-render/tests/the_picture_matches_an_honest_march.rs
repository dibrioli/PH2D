//! ⭐⭐⭐ **A IMAGEM QUE O PRODUTO DESENHA CONCORDA COM UMA MARCHA HONESTA** — o gate que faltava,
//! e o report do Enio de 2026-08-30 pagou-o (*«piorou os artefatos ao rotacionar»*).
//!
//! # ⛔⛔ Por que os gates que já existiam ficaram TODOS verdes com o produto partido
//!
//! Duas rotas baixam um [`FieldDoc`] a uma árvore: o [`ph2d_field_eval::compile_with`] (as sondas e
//! os gates) e o `hybrid::Builder` (**o produto**). O divisor da aresta tinha sido escrito só na
//! primeira. Medido no mesmo raio, na mesma caixa: o traçado via um campo **`8×`** maior que o dos
//! gates, marchava o passo cheio sobre ele, e atravessava a superfície — enquanto **catorze** gates
//! do censo mediam a rota que a produção não usa e diziam `passo × ‖∇f‖ ≤ 0,80`.
//!
//! ⇒ *Um gate que avalia por outra porta que não a do produto mede um programa que ninguém corre.*
//!
//! # A régua: um ORÁCULO, e não uma segunda opinião
//!
//! A marcha de referência aqui é deliberadamente **burra e lenta** — passo minúsculo, `f64`, sem
//! JIT, sem fita, sem ladrilho, sem fatia, sem anti-serrilhado. Ela não partilha código nenhum com
//! a marcha do produto a não ser o campo e a câmera. Se as duas imagens concordam, tudo o que está
//! entre elas está certo; se divergem, o gate não diz **onde**, e é essa a virtude — ele apanha a
//! família inteira, não o defeito de hoje.
//!
//! ⚠️ **A régua é a NORMAL, e não a máscara.** O defeito que o Enio fotografou não abre buraco: ele
//! pinta facetas escuras no meio da peça, porque o raio aterra fundo dentro e o gradiente ali é
//! outro. Uma régua de silhueta não o vê — a que já existia
//! (`a_shape_with_both_recesses_draws_whole_and_strands_no_ray`) ficou verde a jornada inteira.
//!
//! ⚠️ **A silhueta é EXCLUÍDA de propósito** (`erode`): ali meio pixel de diferença vira a normal ao
//! contrário nas duas marchas, e a peça está correcta. *Uma barra que tem de tolerar o contorno não
//! consegue ser apertada no interior, que é onde o defeito vive.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::{Field, hybrid::Registry};
use ph2d_field_render::{Orbit, Screen, trace_with_threads};

/// Lado da imagem. ⚠️ **Ele paga a marcha `f64` do oráculo**, que é `O(lado² × passos)` no
/// interpretador — medido `1,4 s` em release a `72`, e é por isso que ele não é `420`.
const SIDE: u32 = 72;

/// ⚠️ **A MESMA tolerância de acerto do produto** (`Sharpness::for_frame`, que a este enquadramento
/// dá o tecto `HIT_EPS`). ⛔ Um oráculo mais apertado que o produto mede a TOLERÂNCIA e não a
/// marcha: num vinco, `10×` de diferença de profundidade vira a normal — foram `10` pixels a `28°`
/// numa caixa correcta, e nenhum deles era defeito.
const HIT: f64 = 2.0e-4;

/// ⭐⭐⭐ **O passo do ORÁCULO é minúsculo DE PROPÓSITO** — e isso é o que o torna um oráculo.
///
/// ⛔⛔ A 1.ª versão andava o valor do campo inteiro (`t += v`), que é **a lei do produto**. Com a
/// prova de mutação isso apareceu na hora: com o divisor removido, o oráculo herdava o mesmo campo
/// desonesto, atravessava a superfície pela mesma razão, e as duas imagens **concordavam no
/// errado** — o gate só morria pela cláusula de controle (*«a peça não está a ser desenhada»*).
///
/// ⇒ com `0,1` a marcha de referência continua correcta sobre um campo que exagere a distância até
/// **`10×`**; o pior que este módulo produz é `2×` (o `√4` de uma caixa com os dois recuos, quatro
/// superfícies activas na mesma quina). *Um oráculo que partilha a lei do que ele julga não é um
/// oráculo — é um espelho.*
const ORACLE_STEP: f64 = 0.1;

fn doc_of(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça")
}

/// A marcha do ORÁCULO: `f64`, passo do campo, e a normal por diferença central. `None` = fundo.
fn honest(f: &Field, o: [f64; 3], d: [f64; 3]) -> Option<[f64; 3]> {
    let mut t = 0.0f64;
    for _ in 0..4000 {
        let q = [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t];
        let v = f.at(q[0], q[1], q[2]);
        if v < HIT {
            let e = 1.0e-4;
            let g = [
                f.at(q[0] + e, q[1], q[2]) - f.at(q[0] - e, q[1], q[2]),
                f.at(q[0], q[1] + e, q[2]) - f.at(q[0], q[1] - e, q[2]),
                f.at(q[0], q[1], q[2] + e) - f.at(q[0], q[1], q[2] - e),
            ];
            let n = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
            if n <= 0.0 {
                return None;
            }
            return Some([g[0] / n, g[1] / n, g[2] / n]);
        }
        t += v * ORACLE_STEP;
        if t > 8.0 {
            return None;
        }
    }
    None
}

/// Quantos pixels do INTERIOR o produto pinta com uma normal a mais de `limite` graus do oráculo.
fn disagreeing_pixels(p: &Primitive, yaws: &[f32], limite_graus: f64) -> (usize, usize, f64) {
    let doc = doc_of(p.clone());
    let f = Field::new(&doc);
    let reg = Registry::new();
    let screen = Screen::new(SIDE, SIDE, 0.85);
    let (mut mal, mut medidos, mut pior) = (0usize, 0usize, 0.0f64);
    for &a in yaws {
        let (sy, cy) = (a * 0.5).sin_cos();
        let cam = Orbit {
            half_extent: 0.85,
            rotation: [0.0, sy, 0.0, cy],
            ..Orbit::default()
        };
        // ⚠️ **Sem anti-serrilhado**: a 2.ª passagem re-marcha a silhueta com outra amostragem, e o
        // que este gate mede é o INTERIOR. Ligá-lo acrescentaria ruído exactamente onde a régua já
        // não olha.
        let g = trace_with_threads(&doc, &reg, &cam, SIDE, SIDE, true);
        let dentro = |x: u32, y: u32| g.hit[(y * SIDE + x) as usize];
        let (bx, by, bz) = cam.basis();
        for y in 1..SIDE - 1 {
            for x in 1..SIDE - 1 {
                // ⭐ Só o INTERIOR — ver a nota do módulo.
                if !(dentro(x, y)
                    && dentro(x - 1, y)
                    && dentro(x + 1, y)
                    && dentro(x, y - 1)
                    && dentro(x, y + 1))
                {
                    continue;
                }
                let (sx, sy2) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
                let (o, d) = cam.ray_at_plane(sx, sy2);
                let of = [f64::from(o[0]), f64::from(o[1]), f64::from(o[2])];
                let df = [f64::from(d[0]), f64::from(d[1]), f64::from(d[2])];
                let Some(certo) = honest(&f, of, df) else {
                    continue;
                };
                // O produto entrega a normal em espaço de VISTA; o oráculo em mundo.
                let dot = |b: [f32; 3]| {
                    certo[0] * f64::from(b[0])
                        + certo[1] * f64::from(b[1])
                        + certo[2] * f64::from(b[2])
                };
                let alvo = [dot(bx), dot(by), dot(bz)];
                let n = g.normal[(y * SIDE + x) as usize];
                let c = (alvo[0] * f64::from(n[0])
                    + alvo[1] * f64::from(n[1])
                    + alvo[2] * f64::from(n[2]))
                .clamp(-1.0, 1.0);
                let graus = c.acos().to_degrees();
                medidos += 1;
                pior = pior.max(graus);
                if graus > limite_graus {
                    mal += 1;
                }
            }
        }
    }
    (mal, medidos, pior)
}

/// As vistas: **a girar**, porque um campo desonesto só morde onde o raio raspa a superfície.
fn yaws() -> Vec<f32> {
    (0..4).map(|i| 0.37 + 0.42 * i as f32).collect()
}

/// ⭐⭐⭐ **A CAIXA com os dois recuos** — a forma exacta do report.
///
/// ⛔⛔ **Prova de mutação (2026-08-30):** devolver o `primitive()` do
/// `ph2d_field_eval::primitive_tree` à fórmula crua (isto é, tirar o divisor da porta única) leva
/// este gate de **`0`** pixels em desacordo para **`1 186` de `2 308`** — `51,4 %` do interior —,
/// com o pior desvio em **`77,0°`**. O irmão do prisma vai a `1 810` de `6 217` e `83,3°`. *Era
/// isso que o Enio fotografou.*
#[test]
fn the_traced_box_agrees_with_an_honest_march() {
    let p = Primitive::Box {
        half: [0.42, 0.30, 0.26],
        round: 0.12,
        chamfer: 0.12,
    };
    let (mal, medidos, pior) = disagreeing_pixels(&p, &yaws(), 12.0);
    assert!(
        medidos > 2_000,
        "⛔ o CONTROLE falhou: só {medidos} pixels de interior — a peça não está a ser desenhada"
    );
    assert_eq!(
        mal, 0,
        "{mal} de {medidos} pixels do INTERIOR têm a normal a mais de 12° do oráculo (pior \
         {pior:.1}°) — o traçado está a ler um campo diferente do que os gates medem, ou a marcha \
         está a atravessar a superfície"
    );
}

/// O irmão numa forma de parede **não-ortogonal**, que arredonda por outra receita.
#[test]
fn the_traced_prism_agrees_with_an_honest_march() {
    let p = Primitive::Prism {
        sides: 6,
        bottom: 0.5,
        top: 0.5,
        half_height: 0.55,
        round: 0.10,
        chamfer: 0.10,
    };
    let (mal, medidos, pior) = disagreeing_pixels(&p, &yaws(), 12.0);
    assert!(
        medidos > 2_000,
        "⛔ o CONTROLE falhou: só {medidos} pixels de interior"
    );
    assert_eq!(
        mal, 0,
        "{mal} de {medidos} pixels do INTERIOR divergem do oráculo (pior {pior:.1}°)"
    );
}
