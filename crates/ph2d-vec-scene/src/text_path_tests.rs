//! Gates do [`GlyphFrame`] — o motor do texto em caminho.
//!
//! Os oráculos são **geométricos**: um círculo de raio conhecido, uma reta de comprimento
//! conhecido. Nenhum recomputa `frame_at` para saber o que esperar — isso seria um espelho da
//! função sob teste, e espelho é sempre verde.

use super::GlyphFrame;
use crate::arc_path::ArcPath;
use crate::{VecVertex, VertexKind};

const R: f64 = 60.0;

/// Um círculo em quatro cúbicas, centrado na origem.
fn circle() -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let tang = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - tang[i][0], p[i][1] - tang[i][1]],
            out_handle: [p[i][0] + tang[i][0], p[i][1] + tang[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

fn circle_path() -> ArcPath {
    ArcPath::from_contour(&circle(), true).expect("círculo")
}

fn line_path(len: f64) -> ArcPath {
    ArcPath::from_contour(
        &[VecVertex::corner([0.0, 0.0]), VecVertex::corner([len, 0.0])],
        false,
    )
    .expect("reta")
}

/// Alguns arcos espalhados pelo caminho, incluindo as duas pontas.
fn probes(total: f64) -> [f64; 5] {
    [0.0, total * 0.17, total * 0.5, total * 0.83, total]
}

/// **O referencial é uma ROTAÇÃO.** Eixos unitários e ortogonais em todo ponto — é isto que faz
/// o glifo chegar ao mundo sem cisalhar nem esticar, e é isto que dispensa refit (um afim comuta
/// com Bézier). Se algum dia entrar uma orientação NÃO-rígida, ela não passa por este gate: terá
/// o seu.
#[test]
fn the_rainbow_frame_is_a_rigid_rotation_everywhere() {
    let ap = circle_path();
    for s in probes(ap.total()) {
        let f = GlyphFrame::on_path(&ap, s, 0.0, false).expect("frame");
        let nx = f.x_axis[0].hypot(f.x_axis[1]);
        let ny = f.y_axis[0].hypot(f.y_axis[1]);
        let dot = f.x_axis[0].mul_add(f.y_axis[0], f.x_axis[1] * f.y_axis[1]);
        assert!(
            (nx - 1.0).abs() < 1e-9,
            "eixo x nao unitario em s={s}: {nx}"
        );
        assert!(
            (ny - 1.0).abs() < 1e-9,
            "eixo y nao unitario em s={s}: {ny}"
        );
        assert!(dot.abs() < 1e-9, "eixos nao ortogonais em s={s}: {dot}");
    }
}

/// **Num CÍRCULO o texto fica no círculo.** A origem de todo referencial está exatamente a `R` do
/// centro — o oráculo mais direto que existe para "o texto segue a curva", e não conhece a
/// fórmula.
#[test]
fn every_glyph_on_a_circle_sits_at_the_radius() {
    let ap = circle_path();
    for s in probes(ap.total()) {
        let f = GlyphFrame::on_path(&ap, s, 0.0, false).expect("frame");
        let d = f.origin[0].hypot(f.origin[1]);
        assert!((d - R).abs() < 0.05, "s={s}: raio {d}, esperado {R}");
    }
}

/// **O `dy` afasta da baseline, e o lado é o ESQUERDO do sentido de marcha.**
///
/// ⚠️ Este gate nasceu a afirmar *"para FORA da curva"* e ficou vermelho: o círculo do fixture é
/// **anti-horário**, e a esquerda de quem o percorre aponta para o CENTRO. O código estava certo
/// e a minha frase é que estava errada — o árbitro é o gate da reta, onde `dy = 7` põe o glifo em
/// `y = +7`, ou seja **acima** da baseline, que é o que texto precisa. "Fora" é winding-dependente
/// e não serve de especificação; "à esquerda da marcha" serve.
#[test]
fn the_baseline_offset_lifts_the_glyph_to_the_left_of_travel() {
    let ap = circle_path();
    let s = ap.total() * 0.3;
    let base = GlyphFrame::on_path(&ap, s, 0.0, false).expect("frame");
    let lifted = GlyphFrame::on_path(&ap, s, 10.0, false).expect("frame");
    let d0 = base.origin[0].hypot(base.origin[1]);
    let d1 = lifted.origin[0].hypot(lifted.origin[1]);
    assert!(
        (d1 - d0 + 10.0).abs() < 0.05,
        "num circulo ANTI-HORARIO a esquerda e para DENTRO: moveu {} (de {d0} para {d1})",
        d1 - d0
    );
}

/// **Virar põe o texto do OUTRO LADO** — e sem um segundo sinal a lembrar, porque a normal
/// inverte **junto** com a tangente. Um `dy` que punha o glifo à esquerda da marcha passa a
/// pô-lo à direita, sozinho.
///
/// Este gate é o que impede a "cura" de ser um `flip` que só anda ao contrário e continua do
/// mesmo lado.
#[test]
fn flipping_puts_the_text_on_the_other_side_of_the_curve() {
    let ap = circle_path();
    let s = ap.total() * 0.3;
    let plain = GlyphFrame::on_path(&ap, s, 10.0, false).expect("frame");
    let flipped = GlyphFrame::on_path(&ap, s, 10.0, true).expect("frame");
    let d_plain = plain.origin[0].hypot(plain.origin[1]);
    let d_flip = flipped.origin[0].hypot(flipped.origin[1]);
    // Anti-horário: sem flip a esquerda da marcha e' para DENTRO; virar troca o lado.
    assert!(
        d_plain < R - 5.0,
        "sem flip tem de ir para DENTRO: {d_plain}"
    );
    assert!(d_flip > R + 5.0, "com flip tem de ir para FORA: {d_flip}");
}

/// **Virar também inverte o sentido de leitura**: o mesmo arco `s` cai na outra ponta do caminho.
/// Sem isto o texto virado ficaria do outro lado mas a correr para trás.
#[test]
fn flipping_reads_from_the_other_end() {
    let ap = line_path(100.0);
    let straight = GlyphFrame::on_path(&ap, 20.0, 0.0, false).expect("frame");
    let flipped = GlyphFrame::on_path(&ap, 20.0, 0.0, true).expect("frame");
    assert!((straight.origin[0] - 20.0).abs() < 1e-6, "{straight:?}");
    assert!((flipped.origin[0] - 80.0).abs() < 1e-6, "{flipped:?}");
    assert!(
        (flipped.x_axis[0] + 1.0).abs() < 1e-9,
        "o eixo x tem de apontar para tras: {:?}",
        flipped.x_axis
    );
}

/// **Numa RETA horizontal o referencial é a identidade transladada** — texto em caminho sobre uma
/// linha reta tem de ser indistinguível de texto normal, senão a feature muda o desenho de quem
/// não a pediu.
#[test]
fn on_a_straight_line_the_frame_is_plain_text_layout() {
    let ap = line_path(100.0);
    let f = GlyphFrame::on_path(&ap, 42.0, 7.0, false).expect("frame");
    assert!((f.origin[0] - 42.0).abs() < 1e-6, "x {:?}", f.origin);
    assert!((f.origin[1] - 7.0).abs() < 1e-6, "y {:?}", f.origin);
    assert!((f.x_axis[0] - 1.0).abs() < 1e-9 && f.x_axis[1].abs() < 1e-9);
    assert!(f.y_axis[0].abs() < 1e-9 && (f.y_axis[1] - 1.0).abs() < 1e-9);
    // E o `apply` de um ponto qualquer é a mesma translação — o glifo não se deforma.
    // ⚠️ Tolerância, não igualdade: a posição sai de uma INVERSÃO de comprimento de arco
    // (bisseção de 40 iterações), e o resíduo medido aqui é ~2,5e-11. Um `assert_eq!` de f64
    // sobre um resultado de bisseção é um gate que falha por motivo errado.
    let q = f.apply([3.0, 2.0]);
    assert!(
        (q[0] - 45.0).abs() < 1e-6 && (q[1] - 9.0).abs() < 1e-9,
        "apply deu {q:?}"
    );
}

/// `apply([0, 0])` **é** a origem — o contrato do espaço de texto em uma linha.
#[test]
fn the_text_space_origin_maps_to_the_frame_origin() {
    let ap = circle_path();
    let f = GlyphFrame::on_path(&ap, ap.total() * 0.4, 3.0, false).expect("frame");
    let p = f.apply([0.0, 0.0]);
    assert!((p[0] - f.origin[0]).abs() < 1e-12 && (p[1] - f.origin[1]).abs() < 1e-12);
}

/// **O glifo chega ao mundo com o TAMANHO que tinha.** Um afim rígido preserva distâncias: dois
/// pontos do espaço de texto a `d` um do outro continuam a `d` no mundo, em qualquer ponto da
/// curva. É a diferença entre *texto em caminho* e *envelope* — e é por isso que este motor não
/// precisa de refit.
#[test]
fn the_glyph_keeps_its_size_anywhere_on_the_curve() {
    let ap = circle_path();
    let (a, b): ([f64; 2], [f64; 2]) = ([-2.0, -1.5], [2.0, 4.5]);
    let expected = (b[0] - a[0]).hypot(b[1] - a[1]);
    for s in probes(ap.total()) {
        let f = GlyphFrame::on_path(&ap, s, 0.0, false).expect("frame");
        let (pa, pb) = (f.apply(a), f.apply(b));
        let got = (pb[0] - pa[0]).hypot(pb[1] - pa[1]);
        assert!(
            (got - expected).abs() < 1e-9,
            "s={s}: {got} contra {expected}"
        );
    }
}

/// **Deslocar a origem ao longo da linha É somar ao `local[0]`** — a identidade que autoriza a
/// âncora ao meio a ser paga uma vez por glifo em vez de uma vez por ponto de contorno.
///
/// O oráculo é a EQUIVALÊNCIA das duas rotas, não um número: se ela vale, o `shifted_along`
/// não pode estar a fazer outra coisa (rodar, escalar, deslocar pela normal) sem quebrá-la.
#[test]
fn shifting_the_origin_along_the_line_equals_offsetting_every_local_x() {
    let ap = circle_path();
    let f = GlyphFrame::on_path(&ap, ap.total() * 0.23, 2.0, false).expect("frame");
    let h = 0.4;
    let shifted = f.shifted_along(-h);
    for local in [[0.0, 0.0], [1.5, -0.75], [-3.0, 2.25]] {
        let a = shifted.apply(local);
        let b = f.apply([local[0] - h, local[1]]);
        assert!(
            (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
            "{local:?}: {a:?} contra {b:?}"
        );
    }
    // E os EIXOS não se mexem — é uma translação, não uma re-orientação.
    assert_eq!(shifted.x_axis, f.x_axis);
    assert_eq!(shifted.y_axis, f.y_axis);
}

/// **Numa cúspide não há referencial** — e a resposta é `None`, não um ângulo inventado. Quem
/// chama decide o que fazer; esta função não escolhe por ele.
#[test]
fn a_cusp_has_no_frame() {
    // Duas âncoras coincidentes com alças coincidentes: um segmento sem direção nenhuma.
    let degenerate = vec![VecVertex::corner([5.0, 5.0]), VecVertex::corner([5.0, 5.0])];
    let ap = ArcPath::from_contour(&degenerate, false).expect("contorno");
    assert!(GlyphFrame::on_path(&ap, 0.0, 0.0, false).is_none());
}
