//! ⭐⭐ **O TRIÂNGULO DE TRÊS VÉRTICES QUAISQUER** (W131) — a primeira forma da paleta cujo
//! contorno o artista escreve **ponto a ponto** sem desenhar.
//!
//! # ⭐ Por que ele é uma primitiva e não «desenha-se»
//!
//! Medido (`probe_is_a_polygon_already_reachable`, contra uma esfera): um triângulo pela porta do
//! **desenho** custa `4,66×`, e o **prisma** de 3 lados custa `1,62×` — a extrusão é
//! **`2,6×`–`3,1×`** mais cara em toda a faixa de `3` a `32` lados. ⚠️ *O levantamento dizia
//! `1,27×`, e essa leitura era de outra comparação.*
//!
//! ⛔ **E o prisma só faz REGULARES.** O que falta é o escaleno — a rampa, a empena, a ponta de
//! seta —, e ele são **seis números**, não uma lista de tamanho variável: cabe no `Primitive` sem
//! tocar na forma do blob.
//!
//! # A construção: três semiplanos, e a ORIENTAÇÃO é coagida
//!
//! Um triângulo é a **intersecção** de três semiplanos, e é isso que o faz servir o
//! [`crate::ops::plate_joint_n`] — o aro sai tão liso quanto o filete sozinho o faria.
//!
//! ⚠️ **O `half_plane` é negativo à ESQUERDA de `a → b`**, logo os três só apontam para dentro se a
//! volta for **anti-horária**. Uma ordem horária é entrada legítima (o artista arrasta os vértices e
//! a volta inverte-se sozinha ao cruzar), então ela é **corrigida aqui**, no construtor — *coagir na
//! porta obrigaria o artista a saber de que lado ele está a rodar*.

use fidget::context::Tree;

use crate::ops::{half_plane, plate_joint_n, slab_and_walls};
use crate::ops_joint::Edge;

/// O dobro da área com sinal — positivo se `a → b → c` dá a volta ao contrário dos ponteiros.
#[must_use]
pub fn signed_area2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// ⭐ **O raio da circunferência INSCRITA** — o tecto exacto do filete e do chanfro desta forma.
///
/// ⚠️ **Não é uma escolha, é a geometria:** um recuo maior que o inraio come o triângulo inteiro,
/// e o incírculo é o maior disco que cabe lá dentro (`área / semiperímetro`).
#[must_use]
pub fn inradius(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let lado = |p: [f64; 2], q: [f64; 2]| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
    let s = (lado(a, b) + lado(b, c) + lado(c, a)) * 0.5;
    if s <= f64::MIN_POSITIVE {
        return 0.0;
    }
    signed_area2(a, b, c).abs() * 0.5 / s
}

/// O cosseno do ângulo INTERNO em `p`, entre os lados que vão para `q` e `r`.
///
/// ⚠️ **É exactamente o que o [`Edge::at`] quer** — o prisma já o diz: num hexágono ele dá `−½`
/// (quina obtusa de `120°`) e num triângulo equilátero `+½` (aguda de `60°`).
#[must_use]
pub fn cos_corner_of(p: [f64; 2], q: [f64; 2], r: [f64; 2]) -> f64 {
    cos_corner(p, q, r)
}

fn cos_corner(p: [f64; 2], q: [f64; 2], r: [f64; 2]) -> f64 {
    let (u, v) = ([q[0] - p[0], q[1] - p[1]], [r[0] - p[0], r[1] - p[1]]);
    let (lu, lv) = (u[0].hypot(u[1]), v[0].hypot(v[1]));
    if lu <= f64::MIN_POSITIVE || lv <= f64::MIN_POSITIVE {
        return 0.0;
    }
    ((u[0] * v[0] + u[1] * v[1]) / (lu * lv)).clamp(-1.0, 1.0)
}

/// ⭐⭐ **TRIÂNGULO de três vértices quaisquer, puxado em Z.**
#[must_use]
pub fn sd_triangle(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    // ⚠️ **A volta é corrigida aqui** — ver o cabeçalho.
    let (a, b, c) = if signed_area2(a, b, c) < 0.0 {
        (a, c, b)
    } else {
        (a, b, c)
    };
    let pecas = [half_plane(a, b), half_plane(b, c), half_plane(c, a)];
    // ⛔⛔⛔ **O CONTORNO DOBRA-SE AOS PARES, e não pelo n-ário** — a lei que esta casa já tinha
    // escrita: *o n-ário com `chamfer == 0` INFLA*, porque os planos de corte passam pela aresta e
    // contam no `√(activas)`. Medido nesta forma antes da cura: com o filete a **zero** e um
    // triângulo de `10,7°` de quina, a constante de Lipschitz **medida pela definição** era `1,94`
    // — a marcha atravessaria a superfície. *Duas rotas, sempre.*
    //
    // ⭐ E cada junta leva o **cosseno da própria quina** ([`Edge::at`]), que é o padrão do
    // [`crate::ops::sd_prism`]. ⚠️ **A segunda junta forma DUAS quinas ao mesmo tempo** (o `max` de
    // uma composta com o terceiro semiplano), e ali entra a **mais aguda** das duas: majorar a
    // sharpness estreita o filete, e estreitar é o lado seguro.
    // ⛔⛔⛔ **E as juntas levam `Edge::square`, NÃO o cosseno da quina — MEDIDO e recusado.**
    //
    // A 1.ª redacção deu a cada junta o cosseno da própria quina, que é o padrão do
    // [`crate::ops::sd_prism`]. Num prisma **todas as quinas são iguais**; num triângulo qualquer
    // não, e a conta parte na quina **OBTUSA**:
    //
    // | a 1.ª junta leva | L medido (barra `1,02`) |
    // |---|---:|
    // | `cos = −0,966` (a quina de `165°` do caso do censo) | **`5,35`** |
    // | `cos = 0` (`Edge::square`) | **`0,99`** |
    //
    // ⚠️ **E é um DEGRAU, não uma degradação:** com o filete a `0` o campo lê `0,707`, e com
    // **qualquer** filete acima de zero lê `5,35` — o tamanho do filete não entra. ⚠️ E não é a
    // quina AGUDA: um isósceles de `3°` com o filete a `90 %` do inraio lê `0,707`.
    //
    // ⇒ *o operador do ângulo serve duas faces que se ENCONTRAM, e a `165°` elas são quase o mesmo
    // plano* — a família das duas peças quase coincidentes, pelo lado que ele não cobre.
    // `Edge::square` é o caminho de toda chapa desta casa (o paralelogramo, o atraso, o display), e
    // o preço declarado é o de sempre: numa quina que não é recta o raio do filete não é exactamente
    // o pedido.
    //
    // ⛔⛔ **E são DUAS ROTAS, que é a lei desta casa.** Com **chanfro** as peças têm de entrar
    // INTEIRAS no [`plate_joint_n`]: a dobra aos pares entrega ao 2.º passo um perfil **composto**,
    // e um perfil composto leva a costura dele para o aro — medido, a aresta ia de `8,1°` (só com
    // filete) para `41,0°`, `5,05×` contra a barra de `2,60×`.
    if chamfer > 0.0 {
        let arestas = [
            (pecas[0].clone(), pecas[1].clone()),
            (pecas[1].clone(), pecas[2].clone()),
            (pecas[2].clone(), pecas[0].clone()),
        ];
        return plate_joint_n(&pecas, &arestas, half_height, e);
    }
    let j = crate::ops_joint::intersection_joint(&pecas[0], &pecas[1], e);
    let contorno = crate::ops_joint::intersection_joint(&j, &pecas[2], e);
    slab_and_walls(&contorno, half_height, e)
}

/// ⭐ A mesma forma com os **DOIS** cossenos à vista — é o instrumento que produziu a tabela do
/// [`sd_triangle`], e por isso ele fica: *uma recusa medida sem o instrumento que a mediu é um
/// número que ninguém pode reconferir*.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn sd_triangle_with(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    half_height: f64,
    round: f64,
    chamfer: f64,
    cos_primeira: f64,
    cos_segunda: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (a, b, c) = if signed_area2(a, b, c) < 0.0 {
        (a, c, b)
    } else {
        (a, b, c)
    };
    let pecas = [half_plane(a, b), half_plane(b, c), half_plane(c, a)];
    let j = crate::ops_joint::intersection_joint(
        &pecas[0],
        &pecas[1],
        Edge::at(round, chamfer, cos_primeira),
    );
    let contorno =
        crate::ops_joint::intersection_joint(&j, &pecas[2], Edge::at(round, chamfer, cos_segunda));
    slab_and_walls(&contorno, half_height, e)
}
