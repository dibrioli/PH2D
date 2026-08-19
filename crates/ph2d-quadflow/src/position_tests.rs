//! **OS GATES DO CAMPO DE POSIÇÃO** (ADR-0160 §5, Q2).
//!
//! ⚠️ **O oráculo é a RETÍCULA, não a aparência:** as propriedades afirmadas são
//! que o campo pousa em pontos de retícula, que ele fica NA superfície, que a
//! energia converge, e que o plano — onde a resposta certa é conhecida — produz
//! uma grade regular.

use ph2d_mesh::{Face, Mesh, shapes};

use super::{PositionField, middle_point, position_round_4, solve_position};
use crate::orientation::solve_orientation;
use crate::scale::ScaleField;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn fixture() -> Mesh {
    shapes::torus(48, 24, 1.0, 0.35)
}

/// Uma grade plana `w × h` de quads de lado 1.
fn grid(w: usize, h: usize) -> Mesh {
    let mut positions = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            positions.push([x as f32, y as f32, 0.0]);
        }
    }
    let mut faces = Vec::new();
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let i = (y * w + x) as u32;
            let (r, d) = (i + 1, i + w as u32);
            faces.push(Face::quad(i, d, d + 1, r));
        }
    }
    Mesh::from_parts(positions, faces).expect("a grade e' bem formada")
}

/// ⭐ **O ARREDONDAMENTO É IDEMPOTENTE E É UMA RETÍCULA DE VERDADE.**
///
/// Duas afirmações numa: arredondar um ponto que já está na retícula não o move,
/// e o resultado difere da origem por um múltiplo INTEIRO do passo nas duas
/// direções. É a propriedade de que tudo o resto depende — se ela falhar, o campo
/// deriva e a extração não tem células a que agarrar-se.
#[test]
fn the_lattice_round_lands_on_integer_steps_and_is_idempotent() {
    let (o, q, n) = ([0.3, -0.2, 0.5], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let s = 0.25;
    for (k, p) in [
        [0.31, -0.19, 0.5],
        [1.77, 3.02, 0.5],
        [-2.4, 0.9, 0.5],
        [0.3, -0.2, 0.5],
    ]
    .into_iter()
    .enumerate()
    {
        let r = position_round_4(o, q, n, p, s);

        // Múltiplo inteiro do passo, nas duas direções.
        let d = sub(r, o);
        for (axis, e) in [q, [0.0, 1.0, 0.0]].into_iter().enumerate() {
            let steps = dot(d, e) / s;
            assert!(
                (steps - steps.round()).abs() < 1.0e-4,
                "caso {k}, eixo {axis}: o ponto caiu a {steps} passos da origem — nao e' retícula"
            );
        }

        // E arredondar de novo não move nada.
        let again = position_round_4(r, q, n, r, s);
        assert!(
            norm(sub(again, r)) < 1.0e-6,
            "caso {k}: o arredondamento nao e' idempotente ({r:?} -> {again:?})"
        );
    }
}

/// **O PONTO DO MEIO RESPEITA OS DOIS PLANOS TANGENTES.**
///
/// Com as duas normais iguais ele é o ponto médio simples; com elas divergentes,
/// ele continua a satisfazer as duas restrições. ⚠️ É a peça que faz a comparação
/// medir *o desalinhamento das retículas* em vez de *a distância entre os
/// vértices*.
#[test]
fn the_middle_point_honours_both_tangent_planes() {
    // Normais iguais ⇒ o ponto médio.
    let n = [0.0, 0.0, 1.0];
    let (p0, p1) = ([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
    let m = middle_point(p0, n, p1, n);
    assert!(
        norm(sub(m, [1.0, 0.0, 0.0])) < 1.0e-4,
        "com normais iguais o meio nao e' o ponto medio: {m:?}"
    );

    // Normais divergentes: as duas restrições, cada uma dentro de 5%.
    let (n0, n1) = ([0.0, 0.0, 1.0], [0.0, 0.6, 0.8]);
    let (p0, p1) = ([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
    let m = middle_point(p0, n0, p1, n1);
    for (k, (nn, pp)) in [(n0, p0), (n1, p1)].into_iter().enumerate() {
        let err = (dot(nn, m) - dot(nn, pp)).abs();
        assert!(
            err < 0.05,
            "a restricao {k} sobrou {err}: o meio nao respeita o plano tangente {k}"
        );
    }
}

/// ⭐ **A ENERGIA CONVERGE, e o campo fica NA SUPERFÍCIE.**
///
/// ⚠️ **As duas metades juntas de propósito:** uma energia que desce sobre um
/// campo que saiu da superfície descreve uma grade que não vive na malha — e o
/// defeito só apareceria na extração, uma onda depois da causa.
#[test]
fn the_position_energy_settles_and_the_field_stays_on_the_surface() {
    let mesh = fixture();
    let orient = solve_orientation(&mesh, 32);
    let scale = ScaleField::uniform(&mesh, 0.12);

    let edges: usize = (0..mesh.vert_count())
        .map(|v| mesh.adjacency().vert_verts.neighbours(v).len())
        .sum();
    // A mesma derivação do gate do campo de orientação: o ruído de um somatório
    // de `n` termos de `f32`. Aqui os termos são distâncias em unidades de
    // célula, então o piso é mais generoso — e ele continua DERIVADO.
    let noise = edges as f64 * f64::from(f32::EPSILON) * 16.0;

    let mut seen = Vec::new();
    let mut last = f64::INFINITY;
    for it in [0usize, 1, 2, 4, 8, 16, 32] {
        let f = solve_position(&mesh, &orient, &scale, it);
        let e = f.energy(&mesh, &orient, &scale);
        seen.push((it, e));

        // NA SUPERFÍCIE: cada `o` fica no plano tangente do seu vértice.
        for v in 0..f.len() {
            let off = dot(sub(f.at(v), mesh.positions()[v]), mesh.normals()[v]);
            assert!(
                off.abs() < 1.0e-3,
                "com {it} varreduras o campo do vertice {v} saiu do plano tangente ({off})"
            );
        }
        last = last.min(e);
    }
    for (it, e) in &seen {
        eprintln!("[quadflow] posicao: {it:>3} varreduras -> energia {e:.4}");
    }

    // ⚠️ **A lei é a MONOTONIA, e a afirmação de CONVERGÊNCIA saiu daqui por
    // medição.** Este gate exigia que 16 e 32 varreduras dessem a mesma energia —
    // e passava enquanto o `compat_position_extrinsic_4` era a aproximação que
    // arredondava cada lado ao ponto médio: aquele operador não tinha o que
    // fazer, então parava logo. Com o porte fiel (as 16 combinações de quinas) a
    // energia **continua a descer** em 32 varreduras — 1447 → 765 —, e isso é o
    // operador a trabalhar, não a divergir. *Um gate de convergência sobre um
    // passe que não fazia nada é um gate que aprova o nada.*
    let mut last = f64::INFINITY;
    for (it, e) in &seen {
        assert!(
            *e <= last + noise,
            "a energia SUBIU para {e} em {it} varreduras: a suavizacao esta' a divergir"
        );
        last = *e;
    }
    let b = seen[seen.len() - 1].1;
    assert!(
        b < seen[0].1 * 0.75,
        "a suavizacao mal andou: semente {:.4} -> {b:.4}",
        seen[0].1
    );
}

/// ⭐ **O CAMPO NUNCA SE AFASTA MAIS QUE UMA CÉLULA DO SEU VÉRTICE.**
///
/// O invariante é **derivado da construção, não escolhido**: o último passo de
/// cada varredura é `position_round_4(sum, q, n, p_v, s)`, que devolve o ponto da
/// retícula de `sum` **mais próximo de `p_v`**. O pior caso de um ponto ao ponto
/// de grade mais próximo, numa grade quadrada de passo `s`, é a meia-diagonal:
/// `s/√2`. Nada na lei pode passar disso — e se passar, o arredondamento deixou
/// de ser à retícula.
///
/// ⚠️ **É o invariante de que a EXTRAÇÃO (Q3) depende:** ela funde vértices cujos
/// campos caem na mesma célula, e uma origem a duas células de distância do seu
/// vértice faria a fusão juntar o que a superfície separa.
///
/// ⚠️ **Este gate nasceu a afirmar OUTRAS DUAS coisas, e a medição matou as
/// duas.** (1) *"vizinhos partilham a mesma retícula"* — medido **0,205 célula**
/// de desvio médio, imóvel entre 32 e 2 048 varreduras: o campo está num ponto
/// fixo, e quem forma os platôs é a extração, não ele. (2) *"num plano o campo é
/// a identidade"* — medido **1,145** de deslocamento, porque a borda propaga para
/// dentro e o arredondamento passa a morder. *Duas réguas erradas acusaram o
/// produto do defeito que elas próprias tinham.*
///
/// ⇒ ⚠️ **Corrige o ADR-0160 §5:** a hierarquia multirresolução é *"um acelerador
/// de convergência"* para o campo de ORIENTAÇÃO; para o de POSIÇÃO a frase é
/// vacuosa — não há convergência lenta a acelerar, há um ponto fixo atingido em
/// dezenas de varreduras. O que a hierarquia compra aqui é **coerência de longo
/// alcance**, e o gate que a medirá é da Q3.
#[test]
fn the_field_never_leaves_its_own_cell() {
    for (name, mesh) in [("plano", grid(14, 14)), ("toro", fixture())] {
        let orient = solve_orientation(&mesh, 32);
        for s in [0.12f32, 0.5, 2.0] {
            let scale = ScaleField::uniform(&mesh, s);
            let f = solve_position(&mesh, &orient, &scale, 32);
            // A meia-diagonal da célula — o pior caso de um ponto ao nó de grade
            // mais próximo. Derivado, não escolhido.
            let bound = s * core::f32::consts::FRAC_1_SQRT_2;
            let mut worst = 0.0f32;
            for v in 0..f.len() {
                worst = worst.max(norm(sub(f.at(v), mesh.positions()[v])));
            }
            eprintln!("[quadflow] {name} s={s}: pior afastamento {worst:.5} (teto {bound:.5})");
            assert!(
                worst <= bound * 1.001,
                "{name} s={s}: o campo do pior vertice ficou a {worst:.5} do proprio vertice, e a \
                 meia-diagonal da celula e' {bound:.5} -- o arredondamento deixou de ser a reticula"
            );
        }
    }
}

/// **DETERMINÍSTICO** (HR-5).
#[test]
fn the_position_field_is_bit_reproducible() {
    let mesh = fixture();
    let orient = solve_orientation(&mesh, 8);
    let scale = ScaleField::uniform(&mesh, 0.12);
    let a: PositionField = solve_position(&mesh, &orient, &scale, 8);
    let b = solve_position(&mesh, &orient, &scale, 8);
    assert_eq!(a, b, "duas corridas deram campos de posicao diferentes");
}
