//! Gates do [`super::bake_xform`] — **um afim entra na geometria e o frame desaparece**.
//!
//! ⚠️ Este módulo nasceu em 2026-09-05 com a PILHA DE APARÊNCIA (v20), e a razão é o próprio
//! histórico do ficheiro que ele gateia: ele carrega **três** cicatrizes da mesma classe de
//! defeito — a largura do traço que ficou de fora da lista de comprimentos, a estampa do contorno
//! que não seguia a forma, e a estampa do traço que seguia a lei errada. As três eram *código que
//! diz `fill` e devia dizer **tinta***. Uma pilha de N tintas é a quarta oportunidade de repetir a
//! conta, e estes gates são o que a fecha.

use crate::{
    FillRule, GradientStop, Paint, PaintEntry, Rgba8, StrokeSpec, VecPath, VecVertex, Xform,
    bake_xform,
};

fn triangulo() -> Vec<VecVertex> {
    [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]]
        .map(VecVertex::corner)
        .to_vec()
}

fn rampa() -> Paint {
    Paint::Linear {
        stops: vec![
            GradientStop::new(0.0, Rgba8::new(255, 0, 0, 255)),
            GradientStop::new(1.0, Rgba8::new(0, 0, 255, 255)),
        ],
        start: [0.0, 0.0],
        end: [2.0, 0.0],
    }
}

/// Uma forma com o chão da pilha **e** duas camadas por cima: um contorno largo e um preenchimento
/// com gradiente. É a fixtura que contém o fenómeno — uma forma só com a base leria igual com a
/// lei certa e com a errada.
fn com_pilha() -> VecPath {
    VecPath {
        verts: triangulo(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0)),
        fill_rule: FillRule::NonZero,
        paints: vec![
            PaintEntry::stroke(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 3.0)),
            PaintEntry::fill(rampa()),
        ],
        ..VecPath::default()
    }
}

/// ⭐⭐⭐ **A LARGURA DE CADA CONTORNO DA PILHA ESCALA** — não só a do chão.
///
/// ⚠️ Mutação que tem de sangrar: aplicar a caneta só a `path.stroke`. O sintoma seria uma forma
/// com dois contornos a **mudar de proporção ao ser escalada** — o de baixo engorda, o de cima
/// não —, que é o mesmo report que o repo já pagou em 30/08 com a estampa.
#[test]
fn every_stroke_in_the_stack_scales_with_the_shape() {
    let mut p = com_pilha();
    bake_xform(&mut p, &Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
    assert!(
        (p.stroke.as_ref().expect("o chao").width - 2.0).abs() < 1e-9,
        "o contorno de base: {}",
        p.stroke.as_ref().expect("o chao").width
    );
    let crate::PaintKind::Stroke(s) = &p.paints[0].kind else {
        panic!("a 1.a camada e' um contorno");
    };
    assert!(
        (s.width - 6.0).abs() < 1e-9,
        "o contorno da PILHA tem de escalar igual: {}",
        s.width
    );
}

/// ⭐⭐⭐ **O GRADIENTE DE CADA CAMADA VIAJA** — a geometria dele é world-space, como a do chão.
///
/// ⚠️ Mutação que tem de sangrar: transformar só `path.fill`. O sintoma seria uma forma a mover-se
/// e a rampa da camada de cima **ficar onde estava**, que é literalmente o defeito de 30/08 num
/// nível acima.
#[test]
fn every_gradient_in_the_stack_travels_with_the_shape() {
    let mut p = com_pilha();
    bake_xform(&mut p, &Xform([1.0, 0.0, 0.0, 1.0, 10.0, -4.0]));
    let crate::PaintKind::Fill(Paint::Linear { start, end, .. }) = &p.paints[1].kind else {
        panic!("a 2.a camada e' um gradiente linear");
    };
    assert!(
        (start[0] - 10.0).abs() < 1e-9 && (start[1] + 4.0).abs() < 1e-9,
        "o inicio da rampa tem de seguir a forma: {start:?}"
    );
    assert!(
        (end[0] - 12.0).abs() < 1e-9 && (end[1] + 4.0).abs() < 1e-9,
        "e o fim tambem: {end:?}"
    );
}

/// **O CONTROLO: uma forma sem pilha assa exactamente como assava.** Sem ele, os dois gates acima
/// ficariam verdes sobre uma lei que mudou o caminho comum.
#[test]
fn a_shape_without_a_stack_bakes_exactly_as_before() {
    let mut base = VecPath {
        verts: triangulo(),
        closed: true,
        fill: Some(rampa()),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.5)),
        ..VecPath::default()
    };
    let x = Xform([3.0, 0.0, 0.0, 3.0, 1.0, 2.0]);
    bake_xform(&mut base, &x);
    assert!((base.stroke.as_ref().expect("traco").width - 4.5).abs() < 1e-9);
    let Some(Paint::Linear { start, .. }) = &base.fill else {
        panic!("o chao e' um gradiente");
    };
    assert!((start[0] - 1.0).abs() < 1e-9 && (start[1] - 2.0).abs() < 1e-9);
    assert!(base.paints.is_empty(), "e a pilha continua vazia");
}

/// **A PILHA é a única coisa que a porta [`VecPath::widest_stroke`] responde diferente do
/// `stroke.width`** — e é essa a pergunta que um hit-test e uma moldura de exportação fazem.
///
/// ⚠️ Sem ela, uma forma com um contorno extra de `3` sobre um de `1` mede-se como se tivesse `1`,
/// e a caixa que a envolve **corta o desenho**.
#[test]
fn the_widest_stroke_reads_the_whole_stack() {
    let p = com_pilha();
    assert!(
        (p.widest_stroke() - 3.0).abs() < 1e-9,
        "{}",
        p.widest_stroke()
    );
    let so_base = VecPath {
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0)),
        ..VecPath::default()
    };
    assert!((so_base.widest_stroke() - 1.0).abs() < 1e-9);
    assert!(
        (VecPath::default().widest_stroke()).abs() < 1e-9,
        "sem traco nenhum a largura e' zero, e nao um infinito negativo"
    );
}
