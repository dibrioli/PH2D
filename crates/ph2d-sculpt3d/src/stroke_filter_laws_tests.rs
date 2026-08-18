//! **AS TRÊS LEIS QUE NÃO TÊM VERBO** — `Scale`, `Sphere` e `Random`.
//!
//! ⚠️ **Elas existem só como FILTRO, e é isso que as separa das outras
//! quatro:** o `Inflate`, o `Smooth`, o `Relax` e o `SurfaceSmooth` são verbos
//! de carimbo que o filtro reusa, então os gates deles podem perguntar ao
//! verbo. Estas três não têm carimbo nenhum — não há gesto que as arme por
//! outra via —, e por isso os gates aqui falam com o [`FilterKind`] direto.
//!
//! ⚠️ **O ORÁCULO de cada uma é escrito À MÃO a partir da referência**, nunca
//! chamando a função sob teste: o `t = base + base·f` do `calc_scale_filter`, o
//! `midpoint(unit(p), −p)·|f|` do `calc_sphere_translations` e o
//! `orig_normal · f · (hash − ½)` do `randomize_factors`. Um oráculo que
//! chamasse o kernel devolveria a nossa resposta com outro nome — o gate
//! sempre-verde que esta casa já documentou.

use super::*;
use crate::FilterKind;

/// A malha destes gates: uma esfera LISA, e aqui isso não é o caso degenerado
/// que a suíte irmã evita — nenhuma das três leis lê o anel, então uma malha
/// ruidosa só acrescentaria ruído à comparação.
fn mesh() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(16, 24, 1.0)
}

/// O pincel: as três leis **não leem nada dele** (nem verbo, nem referência,
/// nem os `hc_*`), e é isso que as torna endereçáveis só pelo [`FilterKind`].
/// Ele é passado porque a porta o exige para as leis de anel.
fn any_brush() -> Brush {
    Brush {
        verb: Verb::Draw,
        radius: 10.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        front_faces_only: false,
        auto_smooth: 0.0,
        ..Brush::default()
    }
}

/// **A malha dos gates de ESCALA — um ELIPSOIDE, e a esfera não serve.**
///
/// ⚠️ **Numa esfera unitária centrada na origem a posição É a normal**, então
/// `base + base·f` e `base + normal·f` são BYTE-IDÊNTICOS — o
/// [`FilterKind::Scale`] e o [`FilterKind::Inflate`] ficam indistinguíveis, e
/// todo gate de escala passa a ser verde por acidente da fixture. Com raios que
/// variam de 1 a 3, a homotetia move o ponto distante três vezes mais e a
/// inflação move os dois igual.
fn ellipsoid() -> Mesh {
    let mut m = mesh();
    for p in m.positions_mut() {
        p[0] *= 3.0;
    }
    m
}

fn run_on(mut m: Mesh, kind: FilterKind, amount: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let pre = m.positions().to_vec();
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    s.filter(&mut m, &any_brush(), kind, amount);
    (pre, m.positions().to_vec())
}

fn run(kind: FilterKind, amount: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    run_on(mesh(), kind, amount)
}

fn worst(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

// ---------------------------------------------------------------- SCALE

/// **A ESCALA É SOBRE A ORIGEM DO OBJECTO**, e o oráculo é o
/// `calc_scale_filter` escrito à mão.
///
/// ⚠️ **Este é o gate que impede a lei de virar a do `MaskTransform`**, que
/// escala sobre o **centroide ponderado do que está livre** — a sonda
/// `measure_scale_filter` mede a diferença entre as duas em **48% do
/// movimento** com a peça na origem e **90%** com ela deslocada, e é por isso
/// que a composição não exprimia este filtro.
#[test]
fn the_scale_filter_grows_about_the_object_origin() {
    for &f in &[0.05_f32, 0.3, -0.2] {
        let (pre, post) = run_on(ellipsoid(), FilterKind::Scale, f);
        let want: Vec<[f32; 3]> = pre
            .iter()
            .map(|p| [p[0] + p[0] * f, p[1] + p[1] * f, p[2] + p[2] * f])
            .collect();
        assert!(
            worst(&want, &post) < 1e-6,
            "f={f}: a escala nao e' `base + base*f` (desvio {})",
            worst(&want, &post)
        );
    }
}

/// **O CONTROLE da lei acima:** um ponto NA origem não se move por escala
/// nenhuma, e um ponto longe se move em proporção à distância.
///
/// ⚠️ **Sem esta metade, *"escala sobre a origem"* seria satisfeito por
/// qualquer campo radial** — inclusive por um que escalasse sobre o centroide,
/// que numa esfera centrada É a origem. A discriminação é o ponto distante
/// mover-se **proporcionalmente**, que é o que uma homotetia faz e um
/// deslocamento uniforme não.
#[test]
fn the_scale_filter_moves_a_point_in_proportion_to_its_distance() {
    let (pre, post) = run_on(ellipsoid(), FilterKind::Scale, 0.25);
    assert!(
        {
            let rs: Vec<f32> = pre
                .iter()
                .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
                .collect();
            rs.iter().fold(0.0f32, |a, &b| a.max(b))
                > 2.0 * rs.iter().fold(f32::MAX, |a, &b| a.min(b))
        },
        "a fixture perdeu a premissa: os raios nao variam o bastante"
    );
    let mut near = (f32::MAX, 0.0f32);
    let mut far = (0.0f32, 0.0f32);
    for (p, q) in pre.iter().zip(&post) {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        let d = worst(&[*p], &[*q]);
        if r < near.0 {
            near = (r, d);
        }
        if r > far.0 {
            far = (r, d);
        }
    }
    // A lei é a homotetia, então `d` tem de ser `0,25·r` em TODO vértice —
    // e é o CONTRASTE entre o perto e o longe que separa isto de um campo
    // radial de magnitude constante.
    assert!(
        (near.1 - 0.25 * near.0).abs() < 1e-5 && (far.1 - 0.25 * far.0).abs() < 1e-5,
        "o deslocamento nao acompanha a distancia: perto {near:?} longe {far:?}"
    );
}

// --------------------------------------------------------------- SPHERE

/// **A ESFERIZAÇÃO PUXA PARA A ESFERA UNITÁRIA**, pelo oráculo do
/// `calc_sphere_translations` escrito à mão.
#[test]
fn the_sphere_filter_pulls_halfway_to_the_unit_sphere() {
    // Uma esfera de raio 1 já ESTÁ no alvo — a fixture tem de conter o
    // fenômeno, então ela é inflada primeiro.
    let mut m = mesh();
    for p in m.positions_mut() {
        for c in p.iter_mut() {
            *c *= 2.0;
        }
    }
    let pre = m.positions().to_vec();
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    s.filter(&mut m, &any_brush(), FilterKind::Sphere, 0.4);
    let post = m.positions().to_vec();

    let want: Vec<[f32; 3]> = pre
        .iter()
        .map(|p| {
            let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            let a = 0.4f32.abs() * 0.5;
            [
                p[0] + (p[0] / len - p[0]) * a,
                p[1] + (p[1] / len - p[1]) * a,
                p[2] + (p[2] / len - p[2]) * a,
            ]
        })
        .collect();
    assert!(
        worst(&want, &post) < 1e-5,
        "a esferizacao nao e' `midpoint(unit(p), -p)*|f|` (desvio {})",
        worst(&want, &post)
    );
    // E o CONTROLE de direcção: uma esfera de raio 2 tem de ENCOLHER.
    let r_pre = (pre[0][0] * pre[0][0] + pre[0][1] * pre[0][1] + pre[0][2] * pre[0][2]).sqrt();
    let r_post =
        (post[0][0] * post[0][0] + post[0][1] * post[0][1] + post[0][2] * post[0][2]).sqrt();
    assert!(
        r_post < r_pre,
        "um vertice a raio {r_pre} nao veio para dentro (foi para {r_post})"
    );
}

/// **O `|f|` É DA REFERÊNCIA:** arrastar para os dois lados esferiza IGUAL.
///
/// ⚠️ **É o gate que impede a lei de ganhar uma metade que a referência não
/// tem.** Escrever `f` cru daria *afastar-se da esfera* ao arrastar para trás —
/// uma lei nossa a vestir o nome dela —, e nenhum outro gate o veria, porque
/// todos os demais arrastam num sentido só.
#[test]
fn the_sphere_filter_ignores_the_sign_of_the_drag() {
    let mut a = mesh();
    let mut b = mesh();
    for m in [&mut a, &mut b] {
        for p in m.positions_mut() {
            for c in p.iter_mut() {
                *c *= 1.7;
            }
        }
    }
    let start = a.positions().to_vec();

    let mut sa = SculptStroke::default();
    sa.filter_begin(&a);
    sa.filter(&mut a, &any_brush(), FilterKind::Sphere, 0.5);

    let mut sb = SculptStroke::default();
    sb.filter_begin(&b);
    sb.filter(&mut b, &any_brush(), FilterKind::Sphere, -0.5);

    let (pa, pb) = (a.positions().to_vec(), b.positions().to_vec());
    assert!(
        worst(&pa, &pb) < 1e-6,
        "os dois sentidos do arrasto divergem ({})",
        worst(&pa, &pb)
    );
    // CONTROLE: e os dois de facto MOVERAM (senão o acordo seria por vacuo).
    assert!(
        worst(&start, &pa) > 0.1,
        "nenhum dos sentidos moveu nada — o gate acima e' verde por vacuo"
    );
}

/// **UM VÉRTICE NA ORIGEM NÃO TEM DIRECÇÃO PARA UMA ESFERA**, e a única
/// resposta finita é ficar onde está.
///
/// ⚠️ **A guarda é NOSSA e é load-bearing:** sem ela `normalize` de um vector
/// nulo entra no alvo e o `NaN` alastra à malha inteira pela recomputação de
/// normais — um vértice degenerado apagaria o objecto.
#[test]
fn the_sphere_filter_leaves_a_vertex_at_the_origin_finite() {
    let mut m = mesh();
    m.positions_mut()[0] = [0.0, 0.0, 0.0];
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    s.filter(&mut m, &any_brush(), FilterKind::Sphere, 0.6);
    assert!(
        m.positions().iter().flatten().all(|c| c.is_finite()),
        "a origem produziu um valor nao-finito, e ele alastrou"
    );
    assert_eq!(
        m.positions()[0],
        [0.0, 0.0, 0.0],
        "o vertice da origem devia ficar onde esta'"
    );
}

// --------------------------------------------------------------- RANDOM

/// **O SORTEIO É POR VÉRTICE, DETERMINISTA E LIMITADO** — as três primeiras
/// das quatro propriedades que fazem de um ruído um ruído.
///
/// ⚠️ **Não há paridade bit-a-bit a afirmar**, e a divergência está declarada
/// no kernel: a referência mistura com o `BLI_hash_int_2d`, cuja definição não
/// está no clone. O que se pina é o COMPORTAMENTO — e o limite `|f|/2` é
/// exactamente a faixa `[−½, ½)` do `randomize_factors`.
#[test]
fn the_random_filter_is_deterministic_and_bounded() {
    let (pre, a) = run(FilterKind::Random, 0.4);
    let (_, b) = run(FilterKind::Random, 0.4);
    assert_eq!(a, b, "duas corridas iguais deram poses diferentes");

    let mut moved = 0usize;
    for (p, q) in pre.iter().zip(&a) {
        let d = worst(&[*p], &[*q]);
        assert!(
            d <= 0.4 / 2.0 + 1e-5,
            "um vertice andou {d}, acima do teto |f|/2 = 0.2"
        );
        if d > 1e-6 {
            moved += 1;
        }
    }
    // CONTROLE: sem isto, um kernel que nao movesse NADA passaria nos dois.
    assert!(
        moved > pre.len() / 2,
        "so' {moved} de {} vertices se moveram — o sorteio esta' mudo",
        pre.len()
    );
}

/// **VIZINHOS RECEBEM VALORES DIFERENTES** — a quarta propriedade, e a que
/// separa um *jitter* de um deslocamento uniforme.
///
/// ⚠️ **É o gate que a hipótese fácil não satisfaz:** um kernel que devolvesse
/// o MESMO sorteio a todos (uma inflação disfarçada) passa nos limites, no
/// determinismo e na estabilidade, e cai só aqui.
#[test]
fn the_random_filter_decorrelates_neighbours() {
    let (pre, post) = run(FilterKind::Random, 0.5);
    // O deslocamento COM SINAL ao longo do raio (a esfera é unitária, então o
    // raio é a normal): se todos concordassem, o desvio seria zero.
    let signed: Vec<f32> = pre
        .iter()
        .zip(&post)
        .map(|(p, q)| {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            ((q[0] - p[0]) * p[0] + (q[1] - p[1]) * p[1] + (q[2] - p[2]) * p[2]) / r
        })
        .collect();
    let n = signed.len() as f32;
    let mean = signed.iter().sum::<f32>() / n;
    let var = signed.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    assert!(
        var.sqrt() > 0.02,
        "o desvio-padrao do sorteio e' {} — o campo e' quase uniforme",
        var.sqrt()
    );
    // E os dois primeiros vizinhos da malha discordam entre si.
    assert!(
        (signed[0] - signed[1]).abs() > 1e-4,
        "dois vertices vizinhos receberam o mesmo sorteio"
    );
}

/// **A SEMENTE RE-SORTEIA SEM MOVER NADA**, e o default `0` é o que torna
/// todos os gates acima reproduzíveis.
///
/// ⚠️ **A referência sorteia a semente por invocação** (`rand()`), o que faria
/// de cada gate deste verbo uma medição de sorte. O determinismo é a decisão
/// desta casa; variar é gesto do shell.
#[test]
fn the_random_seed_re_rolls_the_same_pose() {
    let mut m = mesh();
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    s.filter(&mut m, &any_brush(), FilterKind::Random, 0.4);
    let with_zero = m.positions().to_vec();

    let mut m2 = mesh();
    let mut s2 = SculptStroke::default();
    s2.set_filter_seed(7);
    s2.filter_begin(&m2);
    s2.filter(&mut m2, &any_brush(), FilterKind::Random, 0.4);

    assert!(
        worst(&with_zero, m2.positions()) > 1e-3,
        "a semente nao mudou o sorteio"
    );
}

// ------------------------------------------------------- as tres juntas

/// **FORÇA ZERO DEVOLVE A POSE EXACTA, nas três** — a promessa do filtro
/// inteiro (*o arrasto de volta é exacto*) aplicada às leis novas.
///
/// ⚠️ **E ela cobre a rota, não só a aritmética:** `f = 0` passa pelo
/// `restore_frozen_pose`, pela lei e pelo aplicador com `accum = 1`, então um
/// gate verde aqui prova que os três degraus compõem para a identidade.
#[test]
fn every_new_law_is_the_identity_at_zero() {
    for kind in [FilterKind::Scale, FilterKind::Sphere, FilterKind::Random] {
        let (pre, post) = run(kind, 0.0);
        assert_eq!(pre, post, "{kind:?} nao devolveu a pose ao bit em f = 0");
    }
}

/// **UM ARRASTO É UM PASSO, NÃO UMA COMPOSIÇÃO** — a lei do
/// `restore_frozen_pose` medida sobre as três leis novas.
///
/// ⚠️ **É a propriedade que faz o desenho não depender de quantos eventos o
/// rato mandou** — a que esta casa já pagou quatro vezes no relevo do Painter.
/// Aplicar a mesma força duas vezes tem de dar a MESMA pose que aplicá-la uma.
#[test]
fn every_new_law_is_one_step_from_the_frozen_pose() {
    for kind in [FilterKind::Scale, FilterKind::Sphere, FilterKind::Random] {
        let mut once = mesh();
        let mut a = SculptStroke::default();
        a.filter_begin(&once);
        a.filter(&mut once, &any_brush(), kind, 0.3);

        let mut twice = mesh();
        let mut b = SculptStroke::default();
        b.filter_begin(&twice);
        b.filter(&mut twice, &any_brush(), kind, 0.3);
        b.filter(&mut twice, &any_brush(), kind, 0.3);

        assert!(
            worst(once.positions(), twice.positions()) < 1e-6,
            "{kind:?} COMPOS as duas chamadas (desvio {})",
            worst(once.positions(), twice.positions())
        );
    }
}
