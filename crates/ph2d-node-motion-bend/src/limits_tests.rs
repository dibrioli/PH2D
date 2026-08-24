//! Os gates da [`super::LIMITS`] e do [`super::MODE`] — **qual fatia do eixo dobra, e o que
//! acontece com o que fica de fora** (doc 89, folha 04).
//!
//! ⚠️ **A fixtura que a lei antiga usava não pode falsificar a nova.** O layout INTEIRO é o
//! caso em que os três modos coincidem por construção (não há «fora»), então ele serve de
//! CONTROLE e de prova de identidade — nunca de prova de que a fatia funciona. Toda afirmação
//! sobre a fatia é medida com um limite que de facto corta.

use super::*;

/// Uma fileira densa ao longo de X, centrada na origem: 21 pontos de `−2` a `+2`.
///
/// ⚠️ **Densa de propósito** — a rigidez do troço de fora mede-se entre VIZINHOS, e dois pontos
/// só não distinguem *rígido* de *esticado*.
fn row() -> Vec<[f32; 2]> {
    (0..21).map(|i| [-2.0 + i as f32 * 0.2, 0.0]).collect()
}

/// O passo entre vizinhos da [`row`], em unidades de mundo.
const STEP: f32 = 0.2;

/// **A LEI QUE SHIPOU**, escrita à mão — o oráculo da identidade do default.
///
/// ⚠️ Ela vive aqui, e não é derivada da [`super::bend`], de propósito: um oráculo que chame a
/// função sob teste concorda com ela por construção, inclusive quando as duas estão erradas.
fn shipped(base: &[[f32; 2]], pivot: [f32; 2], angle_deg: f32, falloff: &[f32]) -> Vec<[f32; 2]> {
    let x_extent = base
        .iter()
        .map(|p| (p[0] - pivot[0]).abs())
        .fold(0.0_f32, f32::max);
    base.iter()
        .enumerate()
        .map(|(i, p)| {
            let (dx, dy) = (p[0] - pivot[0], p[1] - pivot[1]);
            let theta_max = angle_deg * PI / 180.0;
            let bent = if x_extent < MIN_ANGLE_RAD || theta_max.abs() < MIN_ANGLE_RAD {
                [dx, dy]
            } else {
                let k = theta_max / x_extent;
                let r = 1.0 / k;
                let (c, s) = cos_sin_cycles((k * dx) / TAU);
                [(r - dy) * s, r * (1.0 - c) + dy * c]
            };
            let f = falloff.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
            [
                p[0] + (pivot[0] + bent[0] - p[0]) * f,
                p[1] + (pivot[1] + bent[1] - p[1]) * f,
            ]
        })
        .collect()
}

fn run(mode: i32, lo: f32, hi: f32, angle: f32) -> Vec<[f32; 2]> {
    let r = row();
    let f = vec![1.0; r.len()];
    bend(&r, [0.0, 0.0], angle, 0.0, mode, lo, hi, &[], &f)
}

/// ⭐ **O DEFAULT É A LEI DE SEMPRE, AO BIT** — e não «a menos de um epsilon».
///
/// Com `−1, +1` temos `mid = 0,0` e `half = x_extent` **exatamente** (multiplicar e dividir por
/// 2 é exato em IEEE-754), então `k = θ/half` É `θ/x_extent`, `held − mid` É `dx`, e `run` é
/// `0,0` ⇒ o ramo tomado é a MESMA expressão. Um gate de tolerância deixaria passar uma
/// re-associação da conta, que é como uma cena guardada muda de figura numa wave que não a
/// tocou.
#[test]
fn the_default_slice_is_the_law_that_shipped_bit_for_bit() {
    let r = row();
    let f = vec![1.0; r.len()];
    for angle in [90.0_f32, -37.5, 180.0, 0.0, 270.0] {
        let now = bend(
            &r,
            [0.0, 0.0],
            angle,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[],
            &f,
        );
        let then = shipped(&r, [0.0, 0.0], angle, &f);
        for (i, (a, b)) in now.iter().zip(&then).enumerate() {
            assert_eq!(
                (a[0].to_bits(), a[1].to_bits()),
                (b[0].to_bits(), b[1].to_bits()),
                "angulo {angle}, elemento {i}: {a:?} contra {b:?}"
            );
        }
    }
}

/// ⚠️ **E os TRÊS modos coincidem no default**, porque não há nada fora da fatia — o controle
/// que impede o gate acima de provar só o modo que ele nomeia.
#[test]
fn with_the_whole_layout_inside_every_mode_draws_the_same_figure() {
    let base = run(MODE_UNLIMITED, -1.0, 1.0, 90.0);
    for mode in [MODE_LIMITED, MODE_WITHIN_BOX] {
        let other = run(mode, -1.0, 1.0, 90.0);
        for (i, (a, b)) in base.iter().zip(&other).enumerate() {
            assert_eq!(
                (a[0].to_bits(), a[1].to_bits()),
                (b[0].to_bits(), b[1].to_bits()),
                "modo {mode}, elemento {i}: {a:?} contra {b:?}"
            );
        }
    }
}

/// ⭐⭐ **A FATIA RE-ESCALA A CURVATURA, e este gate separa os DOIS desenhos possíveis.**
///
/// Com a fatia em metade do extent e `Limited`, o ângulo INTEIRO acontece dentro dela ⇒ o
/// troço rígido sai pela tangente de `angle`. A `90°` isso é **exatamente para cima**.
///
/// ⚠️ Se os limites apenas ESCONDESSEM parte da dobra (a leitura contrária, em que `k` não
/// muda), a volta no limite seria `45°` e a cauda sairia na diagonal. A barra abaixo mede o
/// ângulo da cauda, e as duas respostas estão a 45° uma da outra — longe de qualquer
/// tolerância.
#[test]
fn the_slice_rescales_the_curvature_so_the_tail_leaves_at_the_full_angle() {
    let out = run(MODE_LIMITED, -0.5, 0.5, 90.0);
    // Os dois últimos elementos estão os dois FORA da fatia (`dx = 1,8` e `2,0` contra `b = 1`).
    let (p, q) = (out[out.len() - 2], out[out.len() - 1]);
    let d = [q[0] - p[0], q[1] - p[1]];
    // A tangente de 90° é `(0, 1)`: a cauda sobe a prumo.
    assert!(
        d[0].abs() < 1e-3 && d[1] > 0.0,
        "a cauda tinha de sair a prumo (tangente de 90 graus), e saiu {d:?}"
    );
    // CONTROLE: a leitura CONTRÁRIA (limites que só escondem) poria a cauda a 45°, onde
    // `d[0]` valeria `~0,707 · passo`. A barra acima recusa-a por duas ordens de grandeza.
    let a45 = STEP * std::f32::consts::FRAC_1_SQRT_2;
    assert!(
        d[0].abs() < a45 * 0.05,
        "e nao a 45 graus (dx seria ~{a45:.4}, medido {:.4})",
        d[0].abs()
    );
}

/// ⭐ **`Limited`: o que fica de fora é RÍGIDO** — ele anda, mas não estica nem entorta.
///
/// A régua é a distância entre VIZINHOS de fora: um troço rígido preserva-a exatamente (era o
/// passo da fileira), um troço que continuasse a dobrar encurtaria a corda, e um troço deixado
/// para trás daria zero deslocamento.
#[test]
fn the_limited_tail_rides_along_rigidly() {
    let out = run(MODE_LIMITED, -1.0, 0.0, 90.0);
    let r = row();
    // Os índices estritamente à direita de `b = 0` (a fileira tem o `0,0` no índice 10).
    let outside: Vec<usize> = (0..r.len()).filter(|&i| r[i][0] > 1e-6).collect();
    assert!(outside.len() >= 5, "a fixtura tem de conter o fenomeno");
    for w in outside.windows(2) {
        let (p, q) = (out[w[0]], out[w[1]]);
        let d = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
        assert!(
            (d - STEP).abs() < 1e-4,
            "vizinhos {} e {}: a distancia era {STEP} e ficou {d:.5}",
            w[0],
            w[1]
        );
    }
    // E ele NÃO é a identidade: a cauda foi levada pela ponta dobrada.
    let last = out[r.len() - 1];
    assert!(
        (last[0] - 2.0).abs() > 0.3 || last[1].abs() > 0.3,
        "a cauda tinha de ACOMPANHAR a ponta, e ficou em {last:?}"
    );
    // E não há salto na junta: o primeiro de fora fica a um passo do último de dentro.
    let joint = ((out[outside[0]][0] - out[outside[0] - 1][0]).powi(2)
        + (out[outside[0]][1] - out[outside[0] - 1][1]).powi(2))
    .sqrt();
    assert!(
        (joint - STEP).abs() < 1e-4,
        "a junta abriu um vao de {joint:.5}"
    );
}

/// ⭐ **`Within Box`: o que fica de fora NÃO se mexe** — e é aí que ele diverge do `Limited`.
#[test]
fn the_within_box_tail_stays_exactly_where_it_was() {
    let out = run(MODE_WITHIN_BOX, -1.0, 0.0, 90.0);
    let r = row();
    for (i, p) in r.iter().enumerate() {
        if p[0] > 1e-6 {
            assert!(
                (out[i][0] - p[0]).abs() < 1e-6 && (out[i][1] - p[1]).abs() < 1e-6,
                "elemento {i} estava em {p:?} e foi parar a {:?}",
                out[i]
            );
        }
    }
    // CONTROLE: o que está DENTRO dobrou — senão o gate acima ficaria verde sobre um nó morto.
    assert!(
        out[0][1].abs() > 0.2,
        "o que esta' dentro tinha de dobrar: {:?}",
        out[0]
    );
}

/// **Os três modos desenham três figuras** — a falsificação de um enum que o `eval` não lê.
#[test]
fn the_three_modes_are_three_pictures() {
    let (u, l, w) = (
        run(MODE_UNLIMITED, -1.0, 0.0, 90.0),
        run(MODE_LIMITED, -1.0, 0.0, 90.0),
        run(MODE_WITHIN_BOX, -1.0, 0.0, 90.0),
    );
    let far = |a: &[[f32; 2]], b: &[[f32; 2]]| -> f32 {
        a.iter()
            .zip(b)
            .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
            .fold(0.0_f32, f32::max)
    };
    for (name, d) in [
        ("unlimited/limited", far(&u, &l)),
        ("limited/within", far(&l, &w)),
        ("unlimited/within", far(&u, &w)),
    ] {
        assert!(d > 0.1, "{name} coincidiram (maior desvio {d:.4})");
    }
}

/// ⚠️ **Os dois limites são um INTERVALO, não um percurso** — trocá-los nomeia a mesma fatia.
/// Sem a ordenação de [`super::slice_of`] o `f32::clamp` da CPU entraria em **pânico**.
#[test]
fn swapping_the_two_limits_names_the_same_slice() {
    let a = run(MODE_LIMITED, -1.0, 0.25, 90.0);
    let b = run(MODE_LIMITED, 0.25, -1.0, 90.0);
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            (p[0].to_bits(), p[1].to_bits()),
            (q[0].to_bits(), q[1].to_bits()),
            "elemento {i}: {p:?} contra {q:?}"
        );
    }
}

/// **Uma fatia VAZIA é a identidade**, não uma divisão por zero: `half` cai abaixo do mesmo
/// piso que já guardava o extent, e o nó devolve o layout intacto.
#[test]
fn an_empty_slice_is_the_identity() {
    let out = run(MODE_LIMITED, 0.3, 0.3, 90.0);
    for (i, (p, q)) in row().iter().zip(&out).enumerate() {
        assert!(
            (p[0] - q[0]).abs() < 1e-6 && (p[1] - q[1]).abs() < 1e-6,
            "elemento {i}: {p:?} virou {q:?}"
        );
        assert!(
            q[0].is_finite() && q[1].is_finite(),
            "elemento {i} nao e' finito"
        );
    }
}

/// ⭐ **A FATIA VIVE NO QUADRO LOCAL DA DOBRA** — a mesma costura da [`super::DIRECTION`], e é
/// isto que torna o `Within Box` deste nó diferente de uma `field.box` composta a montante.
///
/// Dobrar uma fileira já rodada por θ, com `direction = θ` e uma fatia que CORTA, dá a mesma
/// figura que dobrar a original com a mesma fatia e rodar o resultado. Se os limites fossem
/// medidos no eixo X do MUNDO, a fatia cortaria noutro sítio e a igualdade cairia.
#[test]
fn the_slice_is_measured_in_the_bends_own_frame() {
    let deg = 40.0_f32;
    let (c, s) = cos_sin_cycles(deg / 360.0);
    let rot = |p: [f32; 2]| [p[0] * c - p[1] * s, p[0] * s + p[1] * c];
    let r = row();
    let turned: Vec<[f32; 2]> = r.iter().map(|p| rot(*p)).collect();
    let f = vec![1.0; r.len()];
    let a = bend(
        &turned,
        [0.0, 0.0],
        90.0,
        deg,
        MODE_LIMITED,
        -1.0,
        0.3,
        &[],
        &f,
    );
    let b: Vec<[f32; 2]> = bend(&r, [0.0, 0.0], 90.0, 0.0, MODE_LIMITED, -1.0, 0.3, &[], &f)
        .into_iter()
        .map(rot)
        .collect();
    // A barra é a mesma 0,5% dos gates da direção: os dois caminhos rodam em ORDENS diferentes
    // sobre a senoide parabólica do HR-5 (~0,09% fora da trig verdadeira).
    let span = 4.0_f32;
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        let d = (p[0] - q[0]).abs().max((p[1] - q[1]).abs());
        assert!(
            d < span * 0.005,
            "elemento {i}: {p:?} contra {q:?} (desvio {d:.5})"
        );
    }
    // CONTROLE: com a fatia a cortar, a figura NÃO é a fileira rodada — senão o gate seria vácuo.
    let moved = a
        .iter()
        .zip(&turned)
        .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
        .fold(0.0_f32, f32::max);
    assert!(moved > 0.3, "CONTROLE: a dobra nao fez nada ({moved:.4})");
}

/// **O device declara os params novos** — sem isto o kernel lê `params.mode` e a compilação do
/// WGSL falha, ou pior, lê um slot alheio. (O gate de fonte; a paridade numérica vive no
/// arnês de GPU, que precisa de adapter.)
#[test]
fn the_kernel_declares_the_slice_it_reads() {
    for p in [MODE, LIMITS.0, LIMITS.1] {
        assert!(
            GPU_KERNEL.params.contains(&p),
            "o kernel le' `params.{p}` e nao o declara"
        );
    }
    assert!(
        GPU_KERNEL.wgsl.contains("bd_half") && GPU_KERNEL.wgsl.contains("bd_run"),
        "o kernel tem de correr a MESMA lei da CPU"
    );
}

/// **Cada modo tem um nome, e a lista é a que o painel mostra.**
#[test]
fn every_mode_has_the_references_word_for_it() {
    assert_eq!(MODE_LABELS.len(), 3, "tres modos, tres rotulos");
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == MODE)
        .expect("o modo tem hint");
    assert_eq!(
        hint.max, 2.0,
        "o curso do enum tem de alcancar o ultimo modo"
    );
}
