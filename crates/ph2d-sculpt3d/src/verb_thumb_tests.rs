//! **OS GATES DO POLEGAR** — [`Verb::ClayThumb`], o `clay_thumb.cc`.
//!
//! ⚠️ **As barras saem da sonda `tests/measure_clay_thumb.rs`**, e não de um
//! número escolhido: ela mede o corte **nos vértices**, por ajuste de plano por
//! mínimos quadrados, com o [`Verb::Flatten`] de CONTROLE. Um gate que
//! comparasse a inclinação contra `dabs × CLAY_THUMB_TILT_STEP_DEG` estaria a
//! citar a constante que ele diz vigiar.

use super::*;

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)
}

const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.35;
const STEP: f32 = 0.06;

/// Um traço que anda ao longo de `+x` e TERMINA no polo — o que se mede é o
/// corte sob o último dab, que é o que carrega a inclinação acumulada.
fn walk(verb: Verb, dabs: usize, sym: Symmetry) -> Mesh {
    let mut mesh = sphere();
    let b = Brush {
        verb,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..dabs {
        let back = (dabs - 1 - k) as f32 * STEP * R;
        let d = Dab::pulling([TIP[0] - back, TIP[1], TIP[2]], R, EYE, [0.0; 3]);
        s.dab(&mut mesh, &b, &d, sym);
    }
    mesh
}

/// A normal do plano ajustado por **mínimos quadrados** aos vértices frontais
/// dentro de meio raio do polo.
///
/// ⚠️ **O `p[2] > 0` é load-bearing** — um filtro só em `xy` é um cilindro, e um
/// cilindro através de uma esfera apanha as DUAS calotas; com os dois polos
/// dentro o menor autovetor cai no plano `xy` e o ajuste devolve `±90°` para
/// tudo, inclusive para o Flatten de controle. A sonda pagou este erro primeiro.
fn fitted_normal(mesh: &Mesh) -> [f64; 3] {
    let r2 = (0.5 * R) * (0.5 * R);
    let pts: Vec<[f64; 3]> = mesh
        .positions()
        .iter()
        .filter(|p| {
            let d = [p[0] - TIP[0], p[1] - TIP[1]];
            p[2] > 0.0 && d[0] * d[0] + d[1] * d[1] <= r2
        })
        .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    assert!(pts.len() >= 8, "poucos pontos no ajuste: {}", pts.len());
    let n = pts.len() as f64;
    let mut c = [0.0; 3];
    for p in &pts {
        for i in 0..3 {
            c[i] += p[i] / n;
        }
    }
    let mut m = [[0.0f64; 3]; 3];
    for p in &pts {
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += d[i] * d[j];
            }
        }
    }
    let tr = m[0][0] + m[1][1] + m[2][2];
    let mut v = [0.0, 0.0, 1.0];
    for _ in 0..200 {
        let mut w = [0.0f64; 3];
        for i in 0..3 {
            w[i] = tr * v[i] - (m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2]);
        }
        let len = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if len < 1e-18 {
            break;
        }
        v = [w[0] / len, w[1] / len, w[2] / len];
    }
    if v[2] < 0.0 { [-v[0], -v[1], -v[2]] } else { v }
}

/// O ângulo do corte contra o eixo `+z`, em graus, COM SINAL.
fn tilt_deg(mesh: &Mesh) -> f64 {
    let n = fitted_normal(mesh);
    n[0].atan2(n[2]).to_degrees()
}

fn max_shift(rest: &[[f32; 3]], mesh: &Mesh) -> f32 {
    rest.iter()
        .zip(mesh.positions())
        .map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// **A INCLINAÇÃO CRESCE COM O TRAÇO**, e é isso que separa este verbo do
/// Flatten.
///
/// Medido (sonda, esfera 96×144, `R = 0,35`, passo `0,06 R`):
///
/// | dabs | inclinação do corte |
/// |---|---|
/// | 2 | −0,65° |
/// | 5 | −2,83° |
/// | 10 | −5,75° |
/// | 20 | −15,18° |
/// | 40 | **−44,42°** |
///
/// ⚠️ **O CONTROLE é o Flatten no MESMO traço: −3,64°.** Sem ele o gate mediria
/// a curvatura da esfera e chamaria isso de inclinação — um traço de 40 dabs
/// anda meio radiano sobre uma esfera unitária, e o corte que ele deixa não é
/// plano nem no Flatten.
#[test]
fn the_thumb_tilt_grows_along_the_stroke_and_the_flatten_stays_put() {
    let short = tilt_deg(&walk(Verb::ClayThumb, 5, Symmetry::default()));
    let long = tilt_deg(&walk(Verb::ClayThumb, 40, Symmetry::default()));
    let control = tilt_deg(&walk(Verb::Flatten, 40, Symmetry::default()));
    assert!(
        long < short - 20.0,
        "a inclinação tem de CRESCER com o traço: 5 dabs {short:.2}°, 40 dabs {long:.2}°"
    );
    assert!(
        control.abs() < 10.0,
        "o CONTROLE (Flatten, mesmo traço) tem de sair quase plano: {control:.2}°"
    );
    assert!(
        long.abs() > control.abs() * 4.0,
        "o polegar tem de inclinar muito mais que o Flatten: {long:.2}° contra {control:.2}°"
    );
}

/// **O TETO É ALCANÇÁVEL, E A FERRAMENTA SATURA.**
///
/// Duas metades numa asserção só, porque separá-las deixaria cada uma
/// verdadeira por um motivo errado:
///
/// - o corte a 120 e a 200 dabs é **idêntico** (`−81,75°`, desloc `0,6917`) — a
///   projeção num plano é auto-limitada, e depois do teto não há plano novo para
///   onde caminhar;
/// - e ele **não é** o corte de 40 dabs, senão a saturação seria só *"a
///   ferramenta parou de funcionar"*.
#[test]
fn the_thumb_saturates_at_the_ceiling_instead_of_tilting_for_ever() {
    let rest = sphere().positions().to_vec();
    let a = walk(Verb::ClayThumb, 120, Symmetry::default());
    let b = walk(Verb::ClayThumb, 200, Symmetry::default());
    let mid = walk(Verb::ClayThumb, 40, Symmetry::default());
    let (ta, tb, tm) = (tilt_deg(&a), tilt_deg(&b), tilt_deg(&mid));
    assert!(
        (ta - tb).abs() < 0.5,
        "depois do teto o corte não muda mais: 120 dabs {ta:.2}°, 200 dabs {tb:.2}°"
    );
    assert!(
        (ta - tm).abs() > 20.0,
        "e ele TEM de diferir do de 40 dabs, senão a saturação é a ferramenta morta: \
         40 {tm:.2}°, 120 {ta:.2}°"
    );
    assert!(
        (max_shift(&rest, &a) - max_shift(&rest, &b)).abs() < 1e-4,
        "a malha saturada é a MESMA malha"
    );
}

/// **SEM DIREÇÃO NÃO HÁ DEPÓSITO** — os dois `return` do `clay_thumb.cc`
/// (*"delay the first daub"* e `is_zero(grab_delta)`) por uma pergunta só.
///
/// ⚠️ **O CONTROLE é o Flatten no mesmo dab isolado:** ele deposita, o que é o
/// que torna a recusa do polegar uma LEI e não um dab que caiu fora da malha.
#[test]
fn a_thumb_without_a_path_lays_nothing_and_a_flatten_does() {
    let rest = sphere().positions().to_vec();
    let one = |verb: Verb| {
        let mut mesh = sphere();
        let b = Brush {
            verb,
            radius: R,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &Dab::pulling(TIP, R, EYE, [0.0; 3]),
            Symmetry::default(),
        );
        max_shift(&rest, &mesh)
    };
    let thumb = one(Verb::ClayThumb);
    let control = one(Verb::Flatten);
    assert!(
        thumb < 1e-6,
        "um dab sem caminho não pode depositar: desloc {thumb:.6}"
    );
    assert!(
        control > 1e-3,
        "o CONTROLE tem de depositar, senão a recusa acima é vácuo: {control:.6}"
    );
}

/// **A INCLINAÇÃO É DO TRAÇO, NUNCA DO ESPELHO.**
///
/// A referência avança o `front_angle` só em `stroke_is_main_symmetry_pass`;
/// aqui o avanço mora na fronteira da CHAMADA a [`SculptStroke::dab`], que é a
/// mesma passada. Sem isto, armar a simetria faria a inclinação correr ao dobro
/// — e o artista veria a ferramenta mudar de lei ao ligar um espelho.
///
/// ⚠️ **O oráculo é o corte no lado QUE A MÃO TOCOU**, e o gate compara o mesmo
/// traço com e sem espelho.
///
/// ⚠️ **O espelho é o de `z`, e a escolha é da FIXTURE:** o traço anda em `+x`
/// terminando no polo `+z`, que está *sobre* os planos `x = 0` e `y = 0` — com
/// `MIRROR_X` ou `y` a cópia cairia em cima da região medida e o gate estaria a
/// somar as duas. Em `z` ela vai para o polo oposto, que o filtro `p[2] > 0` do
/// ajuste não vê.
#[test]
fn the_tilt_counts_dabs_not_mirror_copies() {
    let mirror_z = Symmetry {
        z: true,
        ..Symmetry::default()
    };
    let plain = tilt_deg(&walk(Verb::ClayThumb, 40, Symmetry::default()));
    let mirrored = tilt_deg(&walk(Verb::ClayThumb, 40, mirror_z));
    assert!(
        (plain - mirrored).abs() < 1.0,
        "o espelho não pode acelerar a inclinação: sem {plain:.2}°, com {mirrored:.2}°"
    );
}

/// **UM TRAÇO NOVO COMEÇA DO ZERO** — `clay_thumb.cc:166`.
///
/// Sem o reset do [`SculptStroke::begin`] o segundo traço herdaria a inclinação
/// do primeiro, e a mesma ferramenta cavaria mais fundo por ter sido usada
/// antes.
#[test]
fn a_new_stroke_starts_the_tilt_over() {
    let mut mesh = sphere();
    let b = Brush {
        verb: Verb::ClayThumb,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    // Um traço LONGO que termina longe do polo — ele carrega a inclinação até
    // perto do teto e não toca a região que o segundo traço vai medir.
    s.begin(&mesh);
    for k in 0..80 {
        let d = Dab::pulling([-0.9 + k as f32 * 0.004, 0.0, 0.436], R, EYE, [0.0; 3]);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    // E agora o traço CURTO no polo, contra uma malha virgem naquela região.
    let mut fresh = mesh.clone();
    s.begin(&fresh);
    for k in 0..5 {
        let back = (4 - k) as f32 * STEP * R;
        let d = Dab::pulling([TIP[0] - back, TIP[1], TIP[2]], R, EYE, [0.0; 3]);
        s.dab(&mut fresh, &b, &d, Symmetry::default());
    }
    let after = tilt_deg(&fresh);
    let alone = tilt_deg(&walk(Verb::ClayThumb, 5, Symmetry::default()));
    assert!(
        (after - alone).abs() < 2.0,
        "o traço curto tem de sair igual sozinho ou depois de um longo: \
         {after:.2}° contra {alone:.2}°"
    );
}

/// **O PLANO PASSA PELO CENTRO DO DAB, e a consequência tem SINAL.**
///
/// Medido no mesmo traço de 40 dabs: o volume assinado contra a esfera de
/// repouso é **−10,68 no Flatten e +11,30 no polegar** — um REMOVE, o outro
/// ACRESCENTA. A causa é só a origem do plano: o Flatten o ancora no centro de
/// ÁREA (que fica abaixo da superfície numa calota curva, logo ele corta), o
/// polegar no centro do DAB (que está *sobre* a superfície, logo ele enche).
///
/// ⚠️ **É o gate que morre se alguém "unificar" as duas origens** achando que a
/// diferença é cosmética.
#[test]
fn the_thumb_fills_where_the_flatten_cuts() {
    let rest = sphere().positions().to_vec();
    let signed = |mesh: &Mesh| -> f64 {
        rest.iter()
            .zip(mesh.positions())
            .map(|(a, b)| {
                let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                f64::from(d[0] * a[0] + d[1] * a[1] + d[2] * a[2])
            })
            .sum()
    };
    let thumb = signed(&walk(Verb::ClayThumb, 40, Symmetry::default()));
    let flat = signed(&walk(Verb::Flatten, 40, Symmetry::default()));
    assert!(
        flat < -1.0,
        "o CONTROLE (Flatten) tem de REMOVER: {flat:.2}"
    );
    assert!(
        thumb > 1.0,
        "o polegar tem de ACRESCENTAR: {thumb:.2} (se ficou negativo, a origem do \
         plano voltou ao centro de área)"
    );
}
