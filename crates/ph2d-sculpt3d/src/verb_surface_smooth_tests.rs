//! **O SURFACE SMOOTH (HC)** — o alisamento que devolve o que tirou.
//!
//! ⚠️ **O [`Verb::Smooth`] é o CONTROLE em quase todo gate daqui, e ele é
//! obrigatório:** o HC caminha para a MESMA média do anel, e o que o separa do
//! irmão é a correção subtraída depois. Um gate que medisse só o HC diria *"o
//! raio quase não mexeu"* — verdade também para um pincel que não faz nada. O
//! que prova a ferramenta é a **RAZÃO** entre as duas colunas nos dois verbos.
//!
//! ⚠️ **E a fixture é a esfera RUGOSA, não a lisa** — a irmã de outra wave
//! ([`ph2d_mesh::shapes::uv_sphere_shuffled`]) tem a forma exacta e a
//! distribuição torta, que é o que um *relax* conserta; aqui é o contrário, e um
//! verbo medido na fixture errada parece não fazer nada.
//!
//! **6 mutações, 6 sangram:**
//!
//! | mutação | sangra |
//! |---|---|
//! | apagar a correção (o HC vira Smooth) | a forma e o no-op |
//! | `hc_disp_average` devolve o próprio `b` (a média morre) | a forma |
//! | `fill_hc_disp` sem o `α` (`d = q` sempre) | só o gate do α |
//! | a correção sem o `w` | o no-op |
//! | tirar o clamp do piso do β | **só o gate de estabilidade** |
//! | `fill_hc_disp` correr para todo verbo | só o gate do buffer preguiçoso |

use super::*;
use ph2d_mesh::Mesh;

const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.45;

fn noisy() -> Mesh {
    ph2d_mesh::shapes::uv_sphere_noisy(48, 72, 1.0, 0.02)
}

/// Um traço PARADO no polo, com os dois knobs explícitos.
fn hold(verb: Verb, alpha: f32, beta: f32, dabs: usize) -> Mesh {
    let mut mesh = noisy();
    let brush = Brush {
        verb,
        radius: R,
        strength: 1.0,
        hc_shape: alpha,
        hc_vertex: beta,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for _ in 0..dabs {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::pulling(TIP, R, EYE, [0.0; 3]),
            Symmetry::default(),
        );
    }
    mesh
}

/// Os índices dentro da pegada — quem o traço de facto tocou.
fn footprint(mesh: &Mesh) -> Vec<usize> {
    let r2 = R * R;
    mesh.positions()
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let d = [p[0] - TIP[0], p[1] - TIP[1]];
            p[2] > 0.0 && d[0] * d[0] + d[1] * d[1] <= r2
        })
        .map(|(i, _)| i)
        .collect()
}

/// **A FORMA** — o raio médio da pegada. A esfera é unitária e o ruído é radial
/// e simétrico, então todo desvio de `1` é deformação.
fn mean_radius(mesh: &Mesh, idx: &[usize]) -> f64 {
    idx.iter()
        .map(|&i| {
            let p = mesh.positions()[i];
            f64::from(p[0].mul_add(p[0], p[1].mul_add(p[1], p[2] * p[2]))).sqrt()
        })
        .sum::<f64>()
        / idx.len() as f64
}

/// **A RUGOSIDADE** — o RMS de `|p − média do anel|` sobre a pegada.
fn roughness(mesh: &Mesh, idx: &[usize]) -> f64 {
    let adj = mesh.adjacency();
    let p = mesh.positions();
    let acc: f64 = idx
        .iter()
        .map(|&i| {
            let q = p[i];
            let avg = ph2d_mesh::ring_average(adj, i as u32, q, |nb| p[nb as usize]);
            let d = [avg[0] - q[0], avg[1] - q[1], avg[2] - q[2]];
            f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])))
        })
        .sum();
    (acc / idx.len() as f64).sqrt()
}

/// **O QUE A WAVE ENTREGA** — a mesma arrumação a uma fracção do custo de forma.
///
/// ⚠️ **O CONTROLE é a metade que torna isto uma medição e não uma afirmação:**
/// o Smooth TEM de encolher nesta fixture, senão *"o HC preserva a forma"* é
/// verdade sobre uma cena onde ninguém deformou nada.
#[test]
fn the_hc_preserves_the_shape_where_the_smooth_eats_it() {
    let base = noisy();
    let idx = footprint(&base);
    let r0 = mean_radius(&base, &idx);

    let smooth = mean_radius(&hold(Verb::Smooth, 0.0, 0.0, 32), &idx);
    let hc = mean_radius(
        &hold(Verb::SurfaceSmooth, HC_SHAPE_DEFAULT, HC_VERTEX_DEFAULT, 32),
        &idx,
    );
    let (ds, dh) = ((r0 - smooth).abs(), (r0 - hc).abs());

    // CONTROLE: a fixture contém o fenômeno.
    assert!(
        ds > 1e-3,
        "o Smooth tem de deformar esta fixture, senão o gate é vácuo: {ds:.6}"
    );
    assert!(
        dh * 10.0 < ds,
        "o HC tem de deformar uma ordem de grandeza menos: HC {dh:.6} contra Smooth {ds:.6}"
    );
}

/// **E ele ainda ALISA** — a outra metade, sem a qual *"preserva a forma"* é
/// satisfeito por um pincel morto.
#[test]
fn the_hc_still_smooths() {
    let base = noisy();
    let idx = footprint(&base);
    let n0 = roughness(&base, &idx);
    let n = roughness(
        &hold(Verb::SurfaceSmooth, HC_SHAPE_DEFAULT, HC_VERTEX_DEFAULT, 32),
        &idx,
    );
    assert!(
        n < n0 * 0.9,
        "o HC tem de baixar a rugosidade da pegada: {n:.6} contra {n0:.6}"
    );
}

/// **O CANTO MORTO, pinado:** `α = 0` com `β = 1` subtrai exactamente o que o
/// passo laplaciano somou, e a malha não se mexe.
///
/// ⚠️ Ele existe para ninguém descobrir este canto como *"o Surface Smooth
/// parou de funcionar"* — e para provar que a correção é a metade que ele diz
/// ser: uma correção sem o `w`, ou uma que não some com `β = 1`, deixa resíduo.
#[test]
fn the_hc_can_be_tuned_into_a_no_op() {
    let base = noisy();
    let after = hold(Verb::SurfaceSmooth, 0.0, 1.0, 4);
    let worst = base
        .positions()
        .iter()
        .zip(after.positions())
        .map(|(a, c)| {
            let d = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))).sqrt()
        })
        .fold(0.0f64, f64::max);
    assert!(
        worst < 1e-6,
        "alfa 0 / beta 1 tem de ser identidade: {worst:e}"
    );
}

/// **O α SEGURA A POSE DO PEN-DOWN** — e a régua é a DERIVA, não o raio.
///
/// ⚠️ A varredura mediu o encolhimento primeiro e o α movia-o dentro do ruído
/// (`−0,022 %` para `−0,006 %`): *o knob estava a ser julgado por uma régua que
/// não mede o que ele faz*.
///
/// ⚠️ **E são 32 dabs de propósito:** no PRIMEIRO `q == o`, então
/// `α·o + (1−α)·q == q` para todo α — um gate de um dab afirmaria *"o α não faz
/// nada"* sobre um knob correto.
#[test]
fn the_alpha_holds_the_pose_closer_to_the_pen_down() {
    let base = noisy();
    let idx = footprint(&base);
    let drift = |alpha: f32| {
        let after = hold(Verb::SurfaceSmooth, alpha, HC_VERTEX_DEFAULT, 32);
        idx.iter()
            .map(|&i| {
                let (a, c) = (base.positions()[i], after.positions()[i]);
                let d = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))).sqrt()
            })
            .sum::<f64>()
            / idx.len() as f64
    };
    let (loose, tight) = (drift(0.0), drift(1.0));
    assert!(
        tight < loose * 0.95,
        "segurar a pose tem de derivar menos: alfa 1 {tight:.6} contra alfa 0 {loose:.6}"
    );
}

/// **O PISO DO β MANTÉM O OPERADOR CONTRACTIVO** — e a mutação que o tira
/// rebenta a malha.
///
/// ⚠️ **A camada é o CLAMP no motor, não o mínimo do slider:** um documento
/// pode trazer qualquer número, e o que impede uma malha de explodir tem de
/// viver onde a lei corre. Medido sem o clamp, `β = 0,3` leva a rugosidade a
/// **43,8×** a da base em dezasseis dabs.
#[test]
fn the_beta_floor_keeps_the_operator_contracting() {
    let base = noisy();
    let idx = footprint(&base);
    let n0 = roughness(&base, &idx);
    // Um valor MUITO abaixo do piso — o motor tem de o corrigir.
    let n = roughness(&hold(Verb::SurfaceSmooth, 0.0, 0.05, 16), &idx);
    assert!(
        n < n0,
        "um beta abaixo do piso tem de ser clampado, nao amplificado: {n:.6} contra {n0:.6}"
    );
    // E o resultado é o MESMO que o piso entrega — o clamp não inventa uma lei
    // terceira, ele devolve a que existe.
    let floored = roughness(&hold(Verb::SurfaceSmooth, 0.0, HC_VERTEX_MIN, 16), &idx);
    assert!(
        (n - floored).abs() < 1e-9,
        "o clamp tem de dar exactamente o piso: {n:.9} contra {floored:.9}"
    );
}

/// **O BUFFER SÓ EXISTE PARA QUEM O USA** — os outros vinte e um verbos não
/// pagam 12 bytes por vértice tocado.
#[test]
fn the_hc_buffer_is_not_allocated_by_the_other_verbs() {
    let mut mesh = noisy();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let draw = Brush {
        verb: Verb::Draw,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    stroke.dab(
        &mut mesh,
        &draw,
        &Dab::pulling(TIP, R, EYE, [0.0; 3]),
        Symmetry::default(),
    );
    assert!(
        stroke.hc_b.is_empty(),
        "um traço de Draw não pode dimensionar o buffer do HC: {}",
        stroke.hc_b.len()
    );

    // E o CONTROLE: com o verbo que o lê, ele existe e cobre a pegada.
    let mut mesh = noisy();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let hc = Brush {
        verb: Verb::SurfaceSmooth,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    stroke.dab(
        &mut mesh,
        &hc,
        &Dab::pulling(TIP, R, EYE, [0.0; 3]),
        Symmetry::default(),
    );
    assert!(
        !stroke.hc_b.is_empty(),
        "o verbo que lê o buffer tem de o dimensionar"
    );
}
