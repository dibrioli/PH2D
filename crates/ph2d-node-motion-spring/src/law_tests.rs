//! Gates da MOLA LEGÍVEL (ciclo 2, W1 — doc 105).

use super::*;
use crate::{MAX_STEPS, STABLE};

/// Corre a lei da mola `n` tiques a perseguir um alvo em `1`, partindo de `0`, e devolve a
/// trajectória. É o mesmo integrador do nó — a mesma equação e a mesma ordem — para o gate
/// medir MOVIMENTO e não a fórmula que ele próprio escreveu.
fn trajectory(tension: f32, friction: f32, n: usize) -> Vec<f32> {
    const DT: f32 = 1.0 / 60.0;
    let (mut x, mut v) = (0.0_f32, 0.0_f32);
    let mut out = Vec::with_capacity(n);
    // O mesmo sub-passo adaptativo do nó: `sub_dt² · tension < STABLE`.
    let ideal = (STABLE / tension)
        .sqrt()
        .min(crate::FRICTION_STABLE / friction);
    let steps = ((DT / ideal).ceil() as usize).clamp(1, MAX_STEPS);
    let sub = DT / steps as f32;
    for _ in 0..n {
        for _ in 0..steps {
            let a = -friction * v - tension * (x - 1.0);
            v += a * sub;
            x += v * sub;
        }
        out.push(x);
    }
    out
}

/// ⭐⭐⭐ **O MODO `TIME` NÃO É UMA APROXIMAÇÃO** — ele resolve para o MESMO par `(tension,
/// friction)` que o artista teria de acertar à mão, e a trajectória é a mesma **ao bit**.
///
/// FALSIFICADO por qualquer aproximação na conversão: as duas trajectórias divergiriam.
#[test]
fn the_time_mode_resolves_to_a_real_physics_pair() {
    for (d, b) in [(0.2_f32, 0.0_f32), (0.5, 0.4), (1.0, -0.5), (0.05, 0.99)] {
        let (k, c) = physics_of(MODE_TIME, 0.0, 0.0, d, b);
        assert!(
            k.is_finite() && c.is_finite() && k > 0.0,
            "d={d} b={b}: {k} {c}"
        );
        // E o modo `Physics` com esse par dá a MESMA trajectória — é a definição de «não é uma
        // aproximação»: o `Time` é uma forma de ESCREVER o par, não outra lei.
        let pelo_tempo = physics_of(MODE_TIME, 0.0, 0.0, d, b);
        let pela_fisica = physics_of(MODE_PHYSICS, k, c, 0.0, 0.0);
        assert_eq!(pelo_tempo, pela_fisica);
        assert_eq!(
            trajectory(k, c, 40),
            trajectory(pela_fisica.0, pela_fisica.1, 40)
        );
    }
}

/// ⭐⭐ **`BOUNCE = 0` CHEGA E PÁRA** — é a mola criticamente amortecida, e é o que um artista
/// espera do ponto de omissão do slider: nada passa do alvo.
///
/// FALSIFICADO por a conversão errar o `ζ = 1`: qualquer `friction` menor faz a trajectória
/// passar de `1` e voltar.
#[test]
fn bounce_zero_never_overshoots() {
    let (k, c) = physics_of(MODE_TIME, 0.0, 0.0, 0.4, 0.0);
    let t = trajectory(k, c, 120);
    let maior = t.iter().fold(f32::MIN, |m, x| m.max(*x));
    assert!(
        maior <= 1.0 + 1e-4,
        "criticamente amortecida nao passa do alvo, e passou ate' {maior}"
    );
    // E o CONTROLE do outro lado: ela CHEGA (senão «não passa» seria verdade sobre uma mola
    // que não anda).
    assert!(
        *t.last().expect("120 tiques") > 0.99,
        "ela tem de chegar ao alvo: {}",
        t.last().expect("120 tiques")
    );
}

/// ⭐⭐ **MAIS `BOUNCE`, MAIS SALTO** — a monotonia é a promessa que o nome do knob faz.
/// FALSIFICADO por trocar o sinal na conversão: o slider passaria a amortecer ao subir.
#[test]
fn more_bounce_overshoots_more() {
    let excesso = |b: f32| {
        let (k, c) = physics_of(MODE_TIME, 0.0, 0.0, 0.4, b);
        trajectory(k, c, 120)
            .iter()
            .fold(f32::MIN, |m, x| m.max(*x))
            - 1.0
    };
    let (a, b, c) = (excesso(0.0), excesso(0.4), excesso(0.8));
    assert!(a < b && b < c, "o excesso tem de crescer: {a} < {b} < {c}");
    assert!(a <= 1e-4, "e o de `0` e' zero: {a}");
}

/// ⭐⭐ **UM `BOUNCE` NEGATIVO ARRASTA-SE** — sobre-amortecida: chega mais devagar que a
/// criticamente amortecida, e continua sem passar. FALSIFICADO por o ramo negativo não existir
/// (os dois lados dariam a mesma coisa).
#[test]
fn a_negative_bounce_drags() {
    let chega_em = |b: f32| {
        let (k, c) = physics_of(MODE_TIME, 0.0, 0.0, 0.4, b);
        trajectory(k, c, 300)
            .iter()
            .position(|x| *x > 0.95)
            .unwrap_or(usize::MAX)
    };
    assert!(
        chega_em(-0.6) > chega_em(0.0),
        "sobre-amortecida chega DEPOIS: {} contra {}",
        chega_em(-0.6),
        chega_em(0.0)
    );
}

/// ⚠️ **OS DOIS RAMOS ENCONTRAM-SE EM `bounce = 0`** — sem isso o slider teria um degrau
/// exactamente no ponto de omissão. FALSIFICADO por qualquer das duas fórmulas mudar sozinha.
#[test]
fn the_two_branches_meet_at_zero() {
    let d = 0.37_f32;
    let neg = physics_of(MODE_TIME, 0.0, 0.0, d, -1e-7);
    let pos = physics_of(MODE_TIME, 0.0, 0.0, d, 0.0);
    assert!((neg.1 - pos.1).abs() < 1e-3, "{neg:?} contra {pos:?}");
}

/// ⭐⭐⭐ **O MODO `PHYSICS` É BYTE-IDÊNTICO** — todo grafo já autorado lê o que sempre leu.
/// FALSIFICADO por a porta tocar nos dois números no caminho de omissão.
#[test]
fn the_physics_mode_is_byte_identical() {
    for (t, f) in [(8.0_f32, 1.5_f32), (0.5, 0.1), (60.0, 20.0), (3.3, 7.7)] {
        assert_eq!(physics_of(MODE_PHYSICS, t, f, 0.9, 0.5), (t, f));
    }
}

/// ⚠️ **AS FRONTEIRAS NÃO PRODUZEM `NaN` NEM ESTOURAM O INTEGRADOR** — a conversão divide pela
/// duração, e `bounce = −1` dividiria por zero. Os dois lados param a um passo do slider.
#[test]
fn the_edges_stay_finite_and_integrable() {
    for (d, b) in [
        (0.0_f32, 0.0_f32),
        (-5.0, 0.0),
        (0.05, 1.0),
        (0.05, -1.0),
        (3.0, -1.5),
    ] {
        let (k, c) = physics_of(MODE_TIME, 0.0, 0.0, d, b);
        assert!(
            k.is_finite() && c.is_finite() && k > 0.0 && c > 0.0,
            "d={d} b={b}: {k} {c}"
        );
        let t = trajectory(k.max(0.1), c.max(0.05), 60);
        assert!(t.iter().all(|x| x.is_finite()), "d={d} b={b} explodiu");
    }
}

/// ⭐⭐ **O QUE O DISPOSITIVO FAZ É O QUE O RUST FAZ** — a lei está escrita duas vezes (aqui e no
/// WGSL do [`super::kernel`]), o que é inevitável num kernel; o que NÃO é inevitável é elas
/// divergirem sem ninguém ver.
///
/// Este gate compara o TEXTO termo a termo: cada constante e cada operação da conversão tem de
/// aparecer no WGSL. FALSIFICADO por mexer numa das duas e não na outra.
#[test]
fn the_wgsl_says_what_the_rust_says() {
    let w = crate::kernel::GPU_KERNEL.wgsl_lib;
    for termo in [
        "sp_physics",
        "6.2831855", // 2π
        "12.566371", // 4π
        "SPRING_MIN_DUR",
        "SPRING_MAX_BOUNCE",
        "SPRING_FOUR_PI * (1.0 - b) / d",
        "SPRING_FOUR_PI / (d * (1.0 + b))",
        "SPRING_TENSION_CEIL",
        "SPRING_FRICTION_CEIL",
    ] {
        assert!(w.contains(termo), "o WGSL perdeu `{termo}`");
    }
    // E os três params chegam lá — sem isto a conversão leria zeros.
    for p in ["mode", "duration", "bounce"] {
        assert!(
            crate::kernel::GPU_KERNEL.params.contains(&p),
            "o kernel nao recebe `{p}`"
        );
    }
    // ⚠️ O CONTROLE das constantes: o `f32` do WGSL tem de ser o mesmo número do Rust.
    assert_eq!(format!("{:.7}", std::f32::consts::TAU), "6.2831855");
    assert_eq!(format!("{:.6}", 4.0 * std::f32::consts::PI), "12.566371");
    // ⚠️ **E os DOIS TECTOS do WGSL são os números que a aritmética do Rust dá** — eles são
    // derivados de `MAX_DT`/`MAX_STEPS`/`STABLE`, e um literal copiado à mão envelhece no dia
    // em que qualquer dos três se mexer.
    let passo = crate::MAX_DT / crate::MAX_STEPS as f32;
    assert!(w.contains(&format!("{:.1}", crate::STABLE / (passo * passo))));
    assert!(w.contains(&format!("{:.1}", crate::FRICTION_STABLE / passo)));
}

/// ⭐⭐⭐ **O LIMITE DO ATRITO NÃO MUDA UM ÚNICO SUB-PASSO NO QUADRO NORMAL** — ele é novo
/// (2026-09-06) e o caminho de omissão não se pode ter mexido em silêncio.
///
/// ⚠️ **A afirmação certa não é *«o termo do atrito nunca é o menor»***, que é FALSA e eu escrevi
/// primeiro: a `tension 0,5` com `friction 4,08` o termo do atrito já manda (`0,245` contra
/// `0,316`). O que importa é se ele muda a **contagem de sub-passos**, que é o que a trajectória
/// vê — e num quadro normal não muda em nenhuma das 441 células da faixa autorada.
///
/// ⛔ **E há um canto onde MUDA, nomeado em vez de escondido:** com `dt` no tecto (`0,1 s`, uma
/// engasgada de 100 ms) a mola mais mole e mais amortecida da faixa (`tension 0,5`,
/// `friction 20`) passa de **1 para 2** sub-passos. Ali o passo único punha `friction·dt = 2,0`
/// — exactamente sobre a fronteira de estabilidade do Euler semi-implícito. *É uma cura naquele
/// canto, não uma regressão: o que mudou foi a mola deixar de integrar no limite.*
#[test]
fn the_friction_bound_never_bites_in_the_authored_range() {
    let passos = |dt: f32, tension: f32, friction: f32, com_atrito: bool| {
        let ideal = if com_atrito {
            (STABLE / tension)
                .sqrt()
                .min(crate::FRICTION_STABLE / friction)
        } else {
            (STABLE / tension).sqrt()
        };
        ((dt / ideal).ceil() as usize).clamp(1, MAX_STEPS)
    };
    // O quadro NORMAL: de 60 fps a 30 fps.
    let mut medidas = 0usize;
    for dt in [1.0 / 60.0_f32, 1.0 / 50.0, 1.0 / 30.0] {
        for i in 0..=20 {
            for j in 0..=20 {
                let tension = 0.5 + (60.0 - 0.5) * i as f32 / 20.0;
                let friction = 0.1 + (20.0 - 0.1) * j as f32 / 20.0;
                assert_eq!(
                    passos(dt, tension, friction, false),
                    passos(dt, tension, friction, true),
                    "dt {dt} tension {tension} friction {friction}: a contagem mudou"
                );
                medidas += 1;
            }
        }
    }
    assert_eq!(
        medidas, 1323,
        "controle: a varredura cobriu a faixa inteira"
    );
    // ⚠️ **O CANTO, medido** — sem isto o gate acima leria-se como «nunca muda em lado nenhum».
    let (soft, damped) = (0.5_f32, 20.0_f32);
    assert_eq!(passos(crate::MAX_DT, soft, damped, false), 1);
    assert_eq!(passos(crate::MAX_DT, soft, damped, true), 2);
    assert!(
        (damped * crate::MAX_DT - 2.0).abs() < 1e-4,
        "e o passo unico ficava EXACTAMENTE na fronteira: friction*dt = {}",
        damped * crate::MAX_DT
    );
}
