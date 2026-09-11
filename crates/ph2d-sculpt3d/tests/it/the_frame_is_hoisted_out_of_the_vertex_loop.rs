//! **O FRAME DO PADRÃO SAI UMA VEZ POR DAB, E ISTO É O QUE PROVA.**
//!
//! O doc do [`ph2d_sculpt3d::Brush::alpha_frame`] afirma que a ASSINATURA impede
//! o caminho quente de pagar o preço errado — `alpha_weight` **recebe** o frame
//! em vez de o derivar. Essa afirmação não é verificável por nenhum gate de
//! identidade: derivar o frame por vértice devolve exatamente os mesmos bits, e
//! as 108 asserções da suíte ficam VERDES sobre um dab cinco vezes mais caro
//! (medido).
//!
//! ⚠️ **O que torna a regressão catastrófica é o rotor:** `rotate_by_degrees`
//! ACUMULA uma rotação de um grau, ou seja é **O(graus)** — até 359 iterações.
//! Uma vez por dab isso é ruído; por vértice, num dab de dezenas de milhares
//! deles, é o padrão inteiro pago de novo em cada um.
//!
//! # O ORÁCULO, e a primeira versão dele que era VERDE POR CONSTRUÇÃO
//!
//! A tentativa óbvia foi comparar um dab de padrão **direcional** contra um
//! **isotrópico**, e ela é uma *razão entre dois doentes* — a doença que esta
//! casa já pagou no campo de smear do Painter. O argumento `&brush.alpha_frame()`
//! é avaliado **no sítio da chamada**, antes de o padrão decidir se o usa, então
//! a mutação encarece os dois braços por igual: medido, `0,155 → 0,602` no
//! isotrópico e `0,168 → 0,637` no direcional. A razão não se move, e o gate
//! passa.
//!
//! O que DISCRIMINA é o próprio rotor: **o custo de um dab não pode depender do
//! ÂNGULO**. Uma vez por dab, `az = 0` e `az = 359` custam o mesmo; por vértice,
//! o segundo paga 359 iterações vezes o número de vértices.
//!
//! | | `az=0, elev=0` | `az=359, elev=90` | razão |
//! |---|---|---|---|
//! | **o produto** | 0,214 ms | 0,180 ms | **0,84×** |
//! | frame por vértice | 0,209 ms | **1,133 ms** | **5,41×** |
//!
//! ⚠️ E é uma RAZÃO e não um kill de relógio de propósito: um bar de wall-clock
//! mede o PERFIL do build (a `line/FLIP` pagou exatamente isso — o mesmo código
//! reprovando em debug e passando em release). Aqui os dois lados são o MESMO
//! trabalho no MESMO perfil, e só o ângulo muda.

use ph2d_sculpt3d::{Alpha, Brush, Dab, SculptStroke, Symmetry, Verb};
use std::time::Instant;

/// A razão que separa *o frame sai uma vez* de *o frame sai por vértice*.
///
/// ⚠️ **Folgada, e a folga é medida:** o produto mede `0,84×` e a mutação `5,41×`
/// — o bar em `2,0` fica a mais de duas vezes de distância dos dois lados, então
/// ele não pode falhar por ruído de escalonador nem passar com a regressão.
const MAX_RATIO: f32 = 2.0;

/// O custo de um dab com o eixo em `(az, elev)`.
fn dab_cost(az: u16, elev: u16) -> f64 {
    // ⚠️ **Malha DENSA e raio GRANDE, e as duas são a fixture:** o custo por
    // vértice só é visível contra um número grande de vértices por dab. Num
    // pincel pequeno a mutação também dispararia, mas contra um total tão baixo
    // que a razão viraria ruído — *a fixture tem de conter o fenômeno*.
    let mut mesh = ph2d_mesh::shapes::uv_sphere(160, 240, 1.0);
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.5,
        // ⚠️ **Um padrão DIRECIONAL**, senão o frame não é lido por ninguém e o
        // gate mediria a própria isenção.
        alpha: Some(Alpha::Strata),
        alpha_az_deg: az,
        alpha_elev_deg: elev,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]);
    // O primeiro dab paga a captura; medimos o regime.
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let t = Instant::now();
    for _ in 0..40 {
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    t.elapsed().as_secs_f64() / 40.0
}

#[test]
fn the_frame_is_hoisted_out_of_the_vertex_loop() {
    // ⚠️ **O MÍNIMO de três corridas**, e é o redutor certo aqui: as duas metades
    // fazem o MESMO trabalho por amostra (nenhuma é estruturalmente diferente da
    // outra), e uma máquina carregada só sabe deixar mais lento.
    let cheap = (0..3).map(|_| dab_cost(0, 0)).fold(f64::MAX, f64::min);
    let turned = (0..3).map(|_| dab_cost(359, 90)).fold(f64::MAX, f64::min);
    let ratio = (turned / cheap) as f32;
    assert!(
        ratio < MAX_RATIO,
        "girar o eixo deixou o dab {ratio:.2}× mais caro ({:.3} contra {:.3} ms) — \
         o frame voltou para dentro do laço de vértices, e o rotor é O(graus)",
        turned * 1000.0,
        cheap * 1000.0
    );
}
