//! **O PREVIEW PREVÊ** — e é isso, e só isso, que estes gates afirmam.
//!
//! ⚠️ O oráculo central **não** compara o preview com a expressão que o produz:
//! isso seria um espelho, sempre verde. Ele compara o preview com o que o
//! TRAÇO de fato deposita — a única coisa que o preview promete.

use super::*;
use crate::{Alpha, Dab, SculptStroke, Symmetry, Verb};
use ph2d_mesh::{Mesh, shapes};

/// A esfera das suítes de alpha: aresta `0,0245`, que a lei das dez arestas
/// resolve na escala abaixo. Uma malha grossa mediria o aliasing.
fn sphere() -> Mesh {
    shapes::uv_sphere(128, 192, 1.0)
}

const ALPHA_SCALE: f32 = 0.20;

fn textured(verb: Verb) -> Brush {
    Brush {
        verb,
        strength: 0.8,
        radius: 0.35,
        alpha: Some(Alpha::Pores),
        alpha_scale: ALPHA_SCALE,
        ..Brush::default()
    }
}

/// **ONDE O PREVIEW É FORTE, O BARRO ANDA MAIS** — a única coisa que ele
/// promete, medida contra o traço em vez de contra a fórmula que o gera.
///
/// ⚠️ **O oráculo é a CORRELAÇÃO por vértice**, e não um par de pontos: o padrão
/// é ruído, então dois pontos escolhidos a dedo dizem o que a semente quiser. E
/// ele é medido **dentro da pegada**, porque fora dela o barro não anda por
/// motivo nenhum e os zeros afogariam o sinal.
///
/// ⚠️ **O falloff é `Constant` por NECESSIDADE, e a primeira versão deste gate
/// mediu 0,481 sem ele.** O que um dab deposita é `alpha × falloff`, e num
/// falloff graduado o segundo fator varia por toda a pegada — ele é um
/// CONFUNDIDOR que dilui a correlação até um número que não fala sobre o
/// preview. Com o disco duro o depósito é `alpha` vezes uma constante, e o que
/// sobra na medição é exatamente a afirmação em teste.
#[test]
fn where_the_preview_is_strong_the_clay_moves_more() {
    let mut mesh = sphere();
    let brush = Brush {
        falloff: crate::Falloff::Constant,
        ..textured(Verb::Draw)
    };

    let mut prev = Vec::new();
    preview_into(&mesh, &brush, &mut prev);
    assert_eq!(prev.len(), mesh.vert_count());

    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &brush,
        &Dab::at([0.0, 0.0, 1.0], brush.radius, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );

    // Só os que se mexeram: a pegada.
    let mut pairs: Vec<(f32, f32)> = Vec::new();
    for (v, (a, b)) in before.iter().zip(mesh.positions()).enumerate() {
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let moved = d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt();
        if moved > 0.0 {
            pairs.push((prev[v], moved));
        }
    }
    assert!(
        pairs.len() > 100,
        "a pegada trouxe só {} vértices — a fixture não contém o fenômeno",
        pairs.len()
    );

    let r = correlation(&pairs);
    assert!(
        r > 0.9,
        "o preview e o depósito correlacionam {r:.3} — o que se vê não é o que o pincel faz"
    );
}

/// **A PORTA DA PEGADA CONCORDA COM A DA MALHA INTEIRA, AO BIT.**
///
/// ⚠️ É o gate que impede a segunda lei. As duas portas existem por CUSTO (uma
/// mede o documento, a outra a pegada), e no dia em que uma delas ganhasse um
/// termo que a outra não tem, o preview passaria a mostrar coisas diferentes
/// conforme o artista tivesse acabado de esculpir ou não — sem erro e sem
/// warning.
#[test]
fn the_footprint_door_agrees_with_the_whole_mesh_door_bit_for_bit() {
    let mesh = sphere();
    let brush = textured(Verb::Draw);

    let mut whole = Vec::new();
    preview_into(&mesh, &brush, &mut whole);

    let all: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let mut patched = vec![f32::NAN; mesh.vert_count()];
    preview_verts(&mesh, &brush, &all, &mut patched);

    assert_eq!(whole, patched, "as duas portas divergiram");
}

/// **BARRO PROTEGIDO NÃO MOSTRA PADRÃO** — porque não vai receber nenhum.
///
/// ⚠️ E o **controle** é o verbo de máscara, que o dab NÃO freia: um preview
/// gateado por uma regra própria em vez da do dab mentiria exatamente sobre o
/// verbo que existe para editar o freio.
#[test]
fn protection_hides_the_pattern_except_for_the_verb_that_edits_it() {
    let mut mesh = sphere();
    let n = mesh.vert_count();
    mesh.masks_mut().fill(1.0);

    let mut prev = Vec::new();
    preview_into(&mesh, &textured(Verb::Draw), &mut prev);
    assert!(
        prev.iter().all(|&w| w == 0.0),
        "o preview pintou padrão sobre barro totalmente protegido"
    );

    preview_into(&mesh, &textured(Verb::Mask), &mut prev);
    let live = prev.iter().filter(|&&w| w > 0.0).count();
    assert!(
        live * 4 > n,
        "o verbo de máscara viu {live} de {n} vértices — ele não é freado pela máscara, \
         e um preview que o freia descreve um depósito que não acontece"
    );
}

/// **SEM PADRÃO ARMADO O PREVIEW É VAZIO** — e vazio, não um vetor de zeros: o
/// vetor custaria 4 B por vértice para dizer a mesma coisa que o comprimento já
/// diz.
#[test]
fn without_a_pattern_the_preview_is_empty() {
    let mesh = sphere();
    let mut prev = vec![0.5; 3];
    preview_into(&mesh, &Brush::default(), &mut prev);
    assert!(prev.is_empty(), "um pincel liso desenhou um preview");
}

/// **UM PREVIEW DE OUTRO COMPRIMENTO É DE OUTRA TOPOLOGIA, e a porta da pegada
/// o RECUSA.**
///
/// Sem a recusa, um remesh no meio de um traço faria a janela escrever valores
/// perfeitamente válidos nos vértices errados — o modo de falha que o
/// `upload_region_at` já recusa um canal ao lado, e pela mesma razão.
#[test]
fn a_preview_that_does_not_measure_the_mesh_is_refused() {
    let mesh = sphere();
    let brush = textured(Verb::Draw);
    let mut stale = vec![7.0_f32; mesh.vert_count() / 2];
    preview_verts(&mesh, &brush, &[0, 1, 2], &mut stale);
    assert!(
        stale.iter().all(|&w| w == 7.0),
        "a porta escreveu num preview de outra topologia"
    );
}

/// Pearson sobre os pares, com a variância zero tratada como *sem correlação* —
/// um padrão constante não prediz nada, e chamar isso de `1.0` seria o gate
/// concordando consigo mesmo.
fn correlation(pairs: &[(f32, f32)]) -> f32 {
    let n = pairs.len() as f32;
    let (mx, my) = pairs
        .iter()
        .fold((0.0, 0.0), |(a, b), (x, y)| (a + x, b + y));
    let (mx, my) = (mx / n, my / n);
    let (mut sxy, mut sxx, mut syy) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (x, y) in pairs {
        let (dx, dy) = (x - mx, y - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return 0.0;
    }
    sxy / (sxx * syy).sqrt()
}
