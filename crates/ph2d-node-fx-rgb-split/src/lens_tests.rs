//! **A LENTE** — os gates do eixo autorável e do raio interno (doc 89, folha 11).
//!
//! Assunto próprio, arquivo próprio: o `lib.rs` responde *que fantasmas este nó emite* e isto
//! responde *onde está o eixo, e onde ele começa a doer*.

use super::*;

/// Uma fileira simétrica em torno da origem — o centroide dela É a origem, então o eixo default
/// e o mundo coincidem e um deslocamento autorado lê-se directamente.
fn row(n: usize) -> Stream {
    #[expect(clippy::cast_precision_loss, reason = "uma fixture pequena")]
    let p: Vec<[f32; 2]> = (0..n)
        .map(|i| [i as f32 - (n as f32 - 1.0) * 0.5, 0.0])
        .collect();
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("size", Column::Vec2(vec![[0.4, 0.4]; n]))
}

fn ps(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    }
}

/// Quanto o fantasma R de cada elemento se afastou da peça dele.
fn spread(s: &Stream, n: usize) -> Vec<f32> {
    let p = ps(s);
    (0..n)
        .map(|i| {
            let (g, e) = (p[i], p[2 * n + i]);
            (g[0] - e[0]).hypot(g[1] - e[1])
        })
        .collect()
}

/// **O DEFAULT É O NÓ DE SEMPRE, AO BIT** — nos dois params novos e nos dois modos.
///
/// ⚠️ A metade do `start` é a que precisa de cuidado: uma divisão por `r` daria *quase* o mesmo
/// número, e falharia exactamente nas peças em cima do eixo — onde o piso do denominador morde.
/// É por isso que o desligado é um braço LITERAL e não um `× 1.0`.
#[test]
fn the_appended_params_are_the_node_that_always_shipped() {
    let src = row(5);
    for (mode, x, y, strength) in [(0.0, 0.1, 0.05, 0.0), (1.0, 0.0, 0.0, 0.06)] {
        let before = ps(&split(&src, mode, x, y, strength, 1.0, Lens::CENTRED));
        let after = ps(&split(
            &src,
            mode,
            x,
            y,
            strength,
            1.0,
            Lens {
                axis: [0.0, 0.0],
                start: 0.0,
            },
        ));
        assert_eq!(before, after, "modo {mode}: o default nao move um bit");
    }
    // E um `start` LIXO conta como desligado, pela mesma porta que o `opacity` já usa.
    for junk in [f32::NAN, f32::INFINITY, -1.0, -0.0] {
        let out = split(
            &src,
            1.0,
            0.0,
            0.0,
            0.06,
            1.0,
            Lens {
                axis: [0.0, 0.0],
                start: junk,
            },
        );
        assert_eq!(
            ps(&out),
            ps(&split(&src, 1.0, 0.0, 0.0, 0.06, 1.0, Lens::CENTRED)),
            "um start lixo ({junk}) conta como desligado"
        );
    }
}

/// **NO MODO `Split` A LENTE INTEIRA É INERTE** — a outra metade do [`super::PARAM_GATES`].
///
/// ⚠️ **As duas metades ou nenhuma** (a lei do doc 90): o portão do painel esconde os três ali,
/// e um portão que escondesse um param que AGE seria pior que nenhum — o artista mexeria numa
/// coisa que não vê. Este gate afirma o lado do KERNEL; o do painel é o `params_visible`.
#[test]
fn the_lens_is_inert_in_the_split_mode() {
    let src = row(5);
    let plain = ps(&split(&src, 0.0, 0.1, 0.05, 0.0, 1.0, Lens::CENTRED));
    let lensed = ps(&split(
        &src,
        0.0,
        0.1,
        0.05,
        0.0,
        1.0,
        Lens {
            axis: [3.0, -2.0],
            start: 1.5,
        },
    ));
    assert_eq!(
        plain, lensed,
        "em `Split` o deslocamento e' uniforme e nao ha' eixo nenhum: a lente nao pode mexer"
    );
}

/// **O EIXO MOVE-SE, E O QUE ELE MOVE É ONDE A FRANJA É ZERO.**
///
/// ⚠️ O oráculo é **qual elemento fica limpo**, não *"a figura mudou"*: uma lente cujo eixo se
/// desloca tem um ponto de fuga, e é ele que se tem de mexer. Medir só a excursão passaria numa
/// implementação que apenas escalasse tudo.
#[test]
fn moving_the_axis_moves_the_element_that_stays_clean() {
    let n = 5;
    let src = row(n); // posições −2 −1 0 1 2, centroide na origem
    let centred = spread(&split(&src, 1.0, 0.0, 0.0, 0.06, 1.0, Lens::CENTRED), n);
    let cleanest = |v: &[f32]| {
        v.iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).expect("finito"))
            .expect("nao vazio")
            .0
    };
    assert_eq!(
        cleanest(&centred),
        2,
        "fixture: o miolo e' o elemento do meio"
    );
    let shifted = spread(
        &split(
            &src,
            1.0,
            0.0,
            0.0,
            0.06,
            1.0,
            Lens {
                axis: [2.0, 0.0],
                start: 0.0,
            },
        ),
        n,
    );
    assert_eq!(
        cleanest(&shifted),
        4,
        "com o eixo em +2 o elemento LIMPO passa a ser o da ponta: {shifted:?}"
    );
}

/// **DENTRO DO RAIO INTERNO O DESLOCAMENTO É ZERO EXACTO** — e é isso que a cadeia por
/// `falloff` não conseguia.
///
/// ⚠️ **A metade que mede a CERCA é a segunda**: a rota antiga (um `field.*` a escrever
/// `falloff`) deixava as cópias **separadas e transparentes**. Este gate prova que aqui elas
/// não se afastaram, o que é uma afirmação sobre a POSIÇÃO e não sobre o alfa — e é a única
/// forma de as duas rotas não se confundirem num smoke.
#[test]
fn inside_the_start_radius_the_ghosts_have_not_moved_at_all() {
    let n = 5;
    let src = row(n);
    let start = 1.5;
    let out = split(
        &src,
        1.0,
        0.0,
        0.0,
        0.06,
        1.0,
        Lens {
            axis: [0.0, 0.0],
            start,
        },
    );
    let s = spread(&out, n);
    // Elementos a −1, 0, +1 do eixo estão DENTRO de 1,5.
    for i in [1, 2, 3] {
        assert_eq!(s[i], 0.0, "elemento {i} esta' dentro do raio limpo: {s:?}");
    }
    // E os das pontas (a 2,0) ficaram de fora e afastaram-se.
    for i in [0, 4] {
        assert!(s[i] > 0.0, "elemento {i} esta' fora e tem franja: {s:?}");
    }
    // ⚠️ **E o ALFA não foi tocado**: a cerca é sobre o deslocamento, e um gate que medisse o
    // alfa passaria com a rota antiga.
    match out.get("tint") {
        Some(Column::Vec4(t)) => {
            assert!(
                t[1][3] > 0.0 && t[2][3] > 0.0,
                "os fantasmas do miolo continuam OPACOS -- eles apenas nao se separaram"
            );
        }
        _ => panic!("tint"),
    }
}

/// **A RAMPA CONTINUA LINEAR, COM A ORIGEM EMPURRADA PARA FORA.**
///
/// ⚠️ Sem isto, uma implementação que apenas *cortasse* o miolo (`if r < start { 0 }` sem
/// reescalar) passaria no gate acima e daria um DEGRAU na borda do raio — a franja apareceria
/// já larga, que é o artefacto que a referência não tem.
#[test]
fn beyond_the_radius_the_fringe_grows_from_zero_not_from_a_step() {
    let start = 1.0;
    let strength = 0.5;
    // Um elemento exactamente na borda, e outro uma unidade adiante.
    let at = |x: f32| {
        let s = Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
        spread(
            &split(
                &s,
                1.0,
                0.0,
                0.0,
                strength,
                1.0,
                Lens {
                    // Com UM elemento o centroide é ele próprio; o eixo autorado é que manda.
                    axis: [-x, 0.0],
                    start,
                },
            ),
            1,
        )[0]
    };
    assert_eq!(at(start), 0.0, "na borda exacta a franja ainda e' zero");
    let one_out = at(start + 1.0);
    assert!(
        (one_out - strength).abs() < 1e-5,
        "uma unidade para la' da borda vale `strength` ({one_out} vs {strength})"
    );
    let two_out = at(start + 2.0);
    assert!(
        (two_out - 2.0 * strength).abs() < 1e-5,
        "e duas valem o dobro -- a rampa e' LINEAR desde a borda ({two_out})"
    );
}
