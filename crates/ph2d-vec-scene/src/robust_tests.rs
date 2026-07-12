//! **O gate de ROBUSTEZ do `cook`** — o usuário arrasta sliders, então o teste arrasta também.
//!
//! `cook` é chamado a cada frame enquanto o usuário mexe num campo do painel. Ele tem de
//! ser uma **função total**: qualquer combinação de valores, qualquer caixa de gesto —
//! inclusive uma degenerada, de largura zero, que nasce no instante em que o mouse desce —
//! produz geometria finita, ou pelo menos não derruba o editor.
//!
//! Não é hipótese. O primeiro smoke do catálogo novo morreu num
//! `clamp(min = 0.5, max = 0.49999999999999994)`, mexendo nos parâmetros do balão de fala:
//! o piso e o teto eram **o mesmo número em matemática real** (`q + hb` e `1 − q − hb`, com
//! `hb = (1 − 2q)/2`), e o ponto flutuante pôs o teto 1 ulp abaixo do piso. Uma janela que
//! colapsa não é um clamp — é uma divisão por zero disfarçada. O `f64::clamp` entra em
//! pânico nesse caso, por projeto.
//!
//! Este gate não sabe onde estão os bugs: ele **procura**. Para cada forma do catálogo,
//! varre milhares de vetores de parâmetros — os extremos exatos das faixas, os valores fora
//! delas (o painel clampa, mas um save de outra versão pode não ter clampado), zero,
//! negativos, e as combinações — e exige que nada exploda e que toda coordenada saia finita.
//! É o único jeito honesto de cobrir a INTERAÇÃO entre dois parâmetros: o pânico do smoke
//! precisava de raio grande **e** base larga ao mesmo tempo, e nenhuma varredura de um
//! campo por vez o encontraria.

use crate::{ALL_SHAPES, MAX_SHAPE_FIELDS, ShapeKind, VecPath, cook};

/// Os valores "interessantes" de um campo. Cobrem: bem abaixo da faixa, as bordas exatas
/// onde os clamps mordem, os pontos de colapso (`0.5`, onde duas metades se encontram), e
/// bem acima. Contagens (lados, pontas, voltas) e ângulos (graus) vivem na mesma lista —
/// `cook` tem de aguentar um `sides = -3` vindo de um documento corrompido sem cair.
const PROBES: &[f64] = &[
    -1.0e6,
    -360.0,
    -3.0,
    -1.0,
    -0.5,
    -1.0e-12,
    0.0,
    1.0e-12,
    0.02,
    0.05,
    0.1,
    0.2,
    0.25,
    0.3,
    0.4,
    0.45,
    0.449_999_999_999_999_9,
    0.5,
    0.500_000_000_000_000_1,
    0.55,
    0.6,
    0.7,
    0.8,
    0.9,
    0.95,
    0.99,
    1.0,
    1.5,
    2.0,
    3.0,
    5.0,
    8.0,
    12.0,
    24.0,
    30.0,
    45.0,
    60.0,
    90.0,
    180.0,
    359.0,
    360.0,
    361.0,
    500.0,
    1.0e6,
];

/// As caixas de gesto que o editor produz de verdade — incluindo as degeneradas. A caixa
/// de largura zero NÃO é teórica: ela existe no primeiro frame de todo arrasto, entre o
/// `pointer_down` e o primeiro `pointer_move`.
const BOXES: &[([f64; 2], [f64; 2])] = &[
    ([-2.0, -1.0], [2.0, 1.0]),     // normal, deitada
    ([-1.0, -2.0], [1.0, 2.0]),     // normal, em pé
    ([-1.0, -1.0], [1.0, 1.0]),     // quadrada
    ([0.0, 0.0], [0.0, 0.0]),       // o primeiro frame do arrasto: caixa NULA
    ([1.0, 1.0], [1.0, 3.0]),       // largura zero
    ([1.0, 1.0], [3.0, 1.0]),       // altura zero
    ([2.0, 1.0], [-2.0, -1.0]),     // invertida (arrasto para trás)
    ([-1e4, -1e4], [1e4, 1e4]),     // gigante (o zoom-out extremo)
    ([-1e-6, -1e-6], [1e-6, 1e-6]), // minúscula (zoom-in extremo)
];

/// Um gerador determinístico (splitmix64) — o gate não pode ser flaky. A mesma semente dá
/// a mesma varredura em toda máquina e em todo CI.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Um valor da lista de sondas.
    fn probe(&mut self) -> f64 {
        PROBES[(self.next() % PROBES.len() as u64) as usize]
    }
}

/// Toda coordenada — âncoras E handles, contorno principal E sub-contornos — é finita.
///
/// Um `NaN` não derruba o editor na hora: ele **contamina** silenciosamente. Vira uma bbox
/// `NaN`, o gizmo some, o hit-test para de responder, e o usuário reporta "a forma sumiu"
/// três telas adiante da causa.
fn all_finite(p: &VecPath) -> Result<(), String> {
    let check = |v: &crate::VecVertex, whose: &str| -> Result<(), String> {
        for (name, pt) in [
            ("anchor", v.anchor),
            ("in_handle", v.in_handle),
            ("out_handle", v.out_handle),
        ] {
            if !pt[0].is_finite() || !pt[1].is_finite() {
                return Err(format!("{whose}: {name} nao-finito {pt:?}"));
            }
        }
        Ok(())
    };
    for v in &p.verts {
        check(v, "contorno")?;
    }
    for (i, c) in p.subpaths.iter().enumerate() {
        for v in &c.verts {
            check(v, &format!("sub-contorno {i}"))?;
        }
    }
    Ok(())
}

/// **O gate.** Para cada forma, milhares de vetores de parâmetros × cada caixa de gesto.
/// Se `cook` entra em pânico, o teste morre com o nome da forma e os valores exatos — que é
/// tudo de que se precisa para reproduzir.
#[test]
fn cook_survives_every_slider_position_and_every_gesture_box() {
    /// Vetores de parâmetros por forma. Cobre a interação entre campos, que é onde o
    /// pânico do smoke morava (raio grande **e** base larga; nenhum campo sozinho o faz).
    const DRAWS: usize = 3_000;

    for &kind in ALL_SHAPES {
        let mut rng = Rng(kind.as_u16() as u64 * 0x1234_5678 + 1);

        // Primeiro os defaults — se a forma nem nasce, nada mais importa.
        for &(a, b) in BOXES {
            let p = cook(kind, a, b, &kind.defaults());
            if let Err(e) = all_finite(&p) {
                panic!("{kind:?} nos DEFAULTS, caixa {a:?}..{b:?}: {e}");
            }
        }

        for _ in 0..DRAWS {
            let mut values = kind.defaults();
            for v in values.iter_mut().take(MAX_SHAPE_FIELDS) {
                *v = rng.probe();
            }
            for &(a, b) in BOXES {
                let p = cook(kind, a, b, &values);
                if let Err(e) = all_finite(&p) {
                    panic!("{kind:?} com valores {values:?}, caixa {a:?}..{b:?}: {e}");
                }
            }
        }
    }
}

/// A varredura de UM campo de cada vez, com os outros nos defaults — é o que o usuário
/// realmente faz (arrastar um slider), e o relatório de falha aponta o campo culpado.
#[test]
fn dragging_any_single_slider_end_to_end_never_breaks_a_shape() {
    let (a, b) = ([-2.0, -1.0], [2.0, 1.0]);
    for &kind in ALL_SHAPES {
        for i in 0..MAX_SHAPE_FIELDS {
            for &probe in PROBES {
                let mut values = kind.defaults();
                values[i] = probe;
                let p = cook(kind, a, b, &values);
                if let Err(e) = all_finite(&p) {
                    panic!("{kind:?}: campo {i} em {probe} quebrou a forma: {e}");
                }
            }
        }
    }
}

/// Uma forma nunca sai VAZIA de um cook (a não ser as que são abertas e degeneram numa
/// caixa nula). Uma forma vazia é invisível: o usuário desenha, nada aparece, e não há erro
/// nenhum em lugar nenhum.
#[test]
fn no_shape_cooks_to_nothing_in_a_real_box() {
    let (a, b) = ([-2.0, -1.0], [2.0, 1.0]);
    for &kind in ALL_SHAPES {
        let p = cook(kind, a, b, &kind.defaults());
        assert!(
            !p.verts.is_empty(),
            "{kind:?} cozinhou para um contorno VAZIO — seria invisivel na tela"
        );
    }
}

/// O discriminante de toda forma sobrevive ao round-trip pelo `u16` do documento. (Aqui só
/// por completude: é o par do gate de append-only.)
#[test]
fn every_kind_round_trips_through_its_document_discriminant() {
    for &kind in ALL_SHAPES {
        assert_eq!(ShapeKind::from_u16(kind.as_u16()), Some(kind));
    }
}
