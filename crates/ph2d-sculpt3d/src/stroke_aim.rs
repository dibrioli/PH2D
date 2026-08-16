//! **A ARITMÉTICA COM QUE UM ALVO É ESCRITO** — filho do [`super`], cortado por
//! ASSUNTO.
//!
//! O pai responde *para onde cada verbo aponta*, e para isso ele fala de
//! [`crate::Verb`], de [`crate::RefMode`] e do `pre` congelado. Aqui não há nada
//! disso: são sete funções de geometria pura que **não sabem que existe um
//! verbo** — somar ao longo de uma direção, girar em torno de um eixo, medir a
//! distância a um plano, caminhar para ele.
//!
//! ⚠️ **O corte é o que a visibilidade permite, e ele não é arbitrário:** o
//! [`super::stroke_axis`] e o `lateral_pull` FICAM no pai porque carregam LEI (o
//! primeiro é citado de fora, o segundo pergunta ao modo) — descê-los aqui
//! obrigaria o `pub(super)` deles a virar `pub(crate)`, e a visibilidade
//! passaria a ser função do TAMANHO do arquivo, que é exatamente o que o pai
//! diz recusar.

use super::PlaneFit;

pub(super) fn add(p: [f32; 3], dir: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k]
}

pub(super) fn add_vec(p: [f32; 3], v: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + v[0] * k, p[1] + v[1] * k, p[2] + v[2] * k]
}

/// `p` girado de `angle` radianos em torno da reta que passa por `pivot` na
/// direção `axis` — **Rodrigues**, e o eixo tem de chegar UNITÁRIO
/// ([`Dab::turning`] é quem garante).
///
/// ⚠️ **Dois transcendentais por vértice por dab**, e é o único ponto deste
/// módulo que os paga (o falloff é polinomial e a raiz é instrução de hardware).
/// Um `sin_cos` é UMA chamada para os dois — pedir `sin` e `cos` separados
/// dobraria a redução de argumento. O preço está medido na sonda
/// `measure_brush_kernel`; ele não é aproximável por tabela, porque quantizar o
/// ângulo faria a torção sair em degraus e interpolar entre nós de uma tabela
/// devolveria uma rotação de norma menor que 1 — o encolhimento, de novo, por
/// outra porta.
pub(super) fn rotate_about(p: [f32; 3], pivot: [f32; 3], axis: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    let v = [p[0] - pivot[0], p[1] - pivot[1], p[2] - pivot[2]];
    let dot = axis[0] * v[0] + axis[1] * v[1] + axis[2] * v[2];
    let cross = [
        axis[1] * v[2] - axis[2] * v[1],
        axis[2] * v[0] - axis[0] * v[2],
        axis[0] * v[1] - axis[1] * v[0],
    ];
    let k = dot * (1.0 - c);
    [
        pivot[0] + v[0] * c + cross[0] * s + axis[0] * k,
        pivot[1] + v[1] * c + cross[1] * s + axis[1] * k,
        pivot[2] + v[2] * c + cross[2] * s + axis[2] * k,
    ]
}

/// O passo em direção ao plano: `p − n · (d · w)`.
///
/// ⚠️ **Porta única dos QUATRO verbos de plano** (`Flatten`, `Fill`, `Scrape`,
/// `Clay`), e é o que os mantém a mesma lei com gates diferentes. Escrita quatro
/// vezes, o dia em que um deles ganhasse um caso especial seria o dia em que os
/// outros três parariam de concordar com ele sem ninguém notar.
pub(super) fn to_plane(p: [f32; 3], normal: [f32; 3], d: f32, w: f32) -> [f32; 3] {
    add(p, normal, -d * w)
}

pub(super) fn signed_distance(p: [f32; 3], plane: &PlaneFit) -> f32 {
    (p[0] - plane.point[0]) * plane.normal[0]
        + (p[1] - plane.point[1]) * plane.normal[1]
        + (p[2] - plane.point[2]) * plane.normal[2]
}

/// A parte de `d` que sobra depois de remover o que corre ao longo de `axis`.
///
/// ⚠️ **`axis` chega UNITÁRIO** — os dois chamadores já o têm normalizado (a
/// normal de área e o [`stroke_axis`]), e re-normalizar por vértice pagaria uma
/// raiz para responder o que já se sabe. É a mesma lei do
/// [`crate::kelvinlet::pinch`] quando ela existia no produto.
pub(super) fn remove_along(d: [f32; 3], axis: [f32; 3]) -> [f32; 3] {
    let along = d[0] * axis[0] + d[1] * axis[1] + d[2] * axis[2];
    [
        d[0] - axis[0] * along,
        d[1] - axis[1] * along,
        d[2] - axis[2] * along,
    ]
}

/// ⚠️ **Um degrau mais largo que os vizinhos, e é o mínimo:** o irmão
/// [`crate::stroke_plane`] monta a dobradiça da lâmina em V e precisa deste
/// produto vetorial. `pub(in crate::stroke)` o abre ao módulo do traço inteiro e
/// **a nada mais** — a alternativa era uma segunda cópia de três linhas lá, que
/// é como duas respostas à mesma pergunta nascem.
pub(in crate::stroke) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
