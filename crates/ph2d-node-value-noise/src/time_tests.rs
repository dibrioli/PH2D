//! **O TEMPO e o RETICULADO** — o laço que fecha, o deslize do domínio e a
//! lacunarity que saiu de uma const.
//!
//! Os três P1 do grupo B deste nó (doc 89 folha 15). Segue FILHO por `#[path]`,
//! então `use super::*` alcança os privados.
//!
//! ⚠️ **Os três defaults reduzem ao mundo anterior**, e cada um por um mecanismo
//! DIFERENTE — é por isso que são três gates de identidade e não um: a lacunarity
//! reduz porque a folha já recebia o número, o laço porque `loop_times` devolve
//! `(t, t, 0)` e a segunda amostra nem é avaliada, e o pan porque somar `0.0` a
//! uma coordenada não a move. Um gate só provaria um deles.

use super::*;

/// A amostra do mundo ANTERIOR ao grupo B, escrita à mão. ⚠️ Ela **declara** os
/// quatro params novos nos defaults em vez de os herdar: uma fixture que chega ao
/// estado por omissão inverte de sentido no dia em que o default se move, e segue
/// verde a testar o oposto (a lição do grupo A).
fn before() -> Sample {
    Sample {
        frequency: 0.3,
        speed: 0.7,
        octaves: 4,
        roughness: 0.5,
        amplitude: 1.0,
        offset: 0.0,
        seed: 2.0,
        kernel: Kernel::Value,
        feature: CellFeature::Cells,
        jitter: 1.0,
        lacunarity: 2.0,
        loop_period: 0.0,
        pan_x: 0.0,
        pan_y: 0.0,
    }
}

/// **Os defaults novos são o mundo anterior, AO BIT.**
///
/// O oráculo é a expressão que SHIPAVA, escrita à mão aqui — chamar `Sample::at`
/// para computar o que se espera dela seria o gate sempre-verde. ⚠️ O `x` de
/// antes era `t·speed` e o `y` era `i·frequency + seed`, sem pan, sem laço, e a
/// lacunarity era a const `2.0` do `noise.rs`.
#[test]
fn the_three_defaults_are_the_world_before_the_group_to_the_bit() {
    let s = before();
    for i in 0..24u32 {
        for t in [0.0f32, 0.37, 1.5, 9.25] {
            let want = {
                let x = t * s.speed;
                let y = i as f32 * s.frequency + s.seed;
                fbm_2d(x, y, s.octaves, 2.0, s.roughness, |px, py| {
                    noise::base(s.kernel, s.feature, s.jitter, px, py)
                }) * s.amplitude
                    + s.offset
            };
            assert_eq!(
                s.at(i, t).to_bits(),
                want.to_bits(),
                "i {i} t {t}: o default moveu o campo"
            );
        }
    }
}

/// **A lacunarity é PROVADAMENTE INERTE numa oitava** — e é isso que a torna um
/// controlo que só existe onde há uma pilha.
///
/// O `px *= lacunarity` do laço acontece DEPOIS da única amostra, então com
/// `octaves = 1` nenhum valor dela pode mover um bit. ⚠️ Este é também o
/// **CONTROLE** do gate seguinte: sem ele, um kernel que ignorasse o param
/// passaria por *"a lacunarity não muda nada"* e ninguém saberia em qual das duas
/// razões.
#[test]
fn one_octave_cannot_see_the_lacunarity() {
    let s = Sample {
        octaves: 1,
        ..before()
    };
    let base = s.at(7, 1.25);
    for lac in [1.0f32, 1.5, 2.0, 3.0, 4.0] {
        let v = Sample {
            lacunarity: lac,
            ..s
        }
        .at(7, 1.25);
        assert_eq!(v.to_bits(), base.to_bits(), "lacunarity {lac} numa oitava");
    }
}

/// **Com uma pilha, a lacunarity MOVE o campo** — a metade que o controle acima
/// não pode provar.
#[test]
fn a_stack_of_octaves_hears_the_lacunarity() {
    let s = Sample {
        octaves: 5,
        ..before()
    };
    let two = (0..48).map(|i| s.at(i, 0.5)).collect::<Vec<_>>();
    let three = (0..48)
        .map(|i| {
            Sample {
                lacunarity: 3.0,
                ..s
            }
            .at(i, 0.5)
        })
        .collect::<Vec<_>>();
    let worst = two
        .iter()
        .zip(&three)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(worst > 0.05, "lacunarity 2 contra 3 mal difere: {worst}");
}

/// **O CAMPO FECHA O LAÇO** — a propriedade inteira do `loop_period`, e a razão
/// de ele ser o item de maior valor da família (*uma ferramenta de motion design
/// cujo ruído não fecha o laço não faz um GIF*).
///
/// ⚠️ O oráculo é a IGUALDADE entre as duas pontas, não um valor escolhido: em
/// `t = 0` e em `t = L` o campo tem de dar o MESMO número, para todo elemento —
/// e em `t = 2L` também, porque um laço que fecha uma vez e deriva na segunda
/// volta não é um laço.
#[test]
fn the_field_closes_the_loop_at_every_turn() {
    let l = 4.0f32;
    let s = Sample {
        loop_period: l,
        ..before()
    };
    for i in 0..24u32 {
        let a = s.at(i, 0.0);
        for turn in [1.0f32, 2.0, 5.0] {
            let b = s.at(i, l * turn);
            assert!(
                (a - b).abs() < 1e-5,
                "i {i} volta {turn}: {a} != {b} — a costura não fechou"
            );
        }
    }
}

/// **O laço não congela o campo** — o CONTROLE do gate acima.
///
/// Sem ele, um `loop_period` que devolvesse sempre `campo(0)` fecharia
/// perfeitamente e seria um ruído morto: as duas pontas iguais é metade da
/// propriedade, e a outra metade é que o meio da volta é DIFERENTE.
#[test]
fn the_loop_still_lets_the_field_move_inside_the_turn() {
    let l = 4.0f32;
    let s = Sample {
        loop_period: l,
        ..before()
    };
    let start: Vec<f32> = (0..24).map(|i| s.at(i, 0.0)).collect();
    let mid: Vec<f32> = (0..24).map(|i| s.at(i, l * 0.5)).collect();
    let worst = start
        .iter()
        .zip(&mid)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst > 0.05,
        "o meio da volta mal difere do começo: {worst}"
    );
}

/// **O laço fecha em QUALQUER velocidade** — as duas amostras atravessam o mesmo
/// `x_of`, então o `speed` escala as duas e a costura continua no mesmo número.
///
/// ⚠️ A fixture varre um `speed` que passa de 1: sem isso o campo mal se move
/// dentro de uma volta e o fecho seria verde por vácuo.
#[test]
fn the_seam_closes_whatever_the_speed() {
    let l = 3.0f32;
    for speed in [0.25f32, 1.0, 4.0, 12.0] {
        let s = Sample {
            loop_period: l,
            speed,
            ..before()
        };
        for i in [0u32, 5, 17] {
            let (a, b) = (s.at(i, 0.0), s.at(i, l));
            assert!((a - b).abs() < 1e-5, "speed {speed} i {i}: {a} != {b}");
        }
    }
}

/// **O `pan` mede RETICULADO, e a prova é que um pan de 1 É um seed de 1.**
///
/// ⚠️ É este gate que fixa a UNIDADE do knob, e ele é a razão de a escolha não
/// ser arbitrária: a alternativa (pan em unidades de MUNDO, `(px + pan)·frequency`)
/// faria o mesmo controlo medir duas grandezas diferentes conforme o `Sample`
/// fosse Index ou World. Aqui ele mede uma só nos dois — a régua do `seed`, que
/// é o vizinho com quem ele soma.
#[test]
fn a_pan_of_one_is_a_seed_of_one() {
    let s = before();
    for i in 0..24u32 {
        let by_pan = Sample { pan_y: 1.0, ..s }.at(i, 0.4);
        let by_seed = Sample {
            seed: s.seed + 1.0,
            ..s
        }
        .at(i, 0.4);
        assert_eq!(by_pan.to_bits(), by_seed.to_bits(), "i {i}");
    }
}

/// **O pan DESLIZA — é contínuo onde o seed é um degrau.**
///
/// O `seed` é inteiro de passo 1 (o widget é `Seed`), então ele re-sorteia; o pan
/// atravessa a célula. O oráculo é que meio passo de pan produz um campo
/// **entre** os dois inteiros e diferente dos dois — que é o que "deslizar"
/// significa e o que um re-sorteio nunca faz.
#[test]
fn the_pan_slides_where_the_seed_jumps() {
    let s = before();
    let at = |pan: f32| -> Vec<f32> {
        (0..24)
            .map(|i| Sample { pan_y: pan, ..s }.at(i, 0.4))
            .collect()
    };
    let (a, half, b) = (at(0.0), at(0.5), at(1.0));
    let far = |u: &[f32], v: &[f32]| {
        u.iter()
            .zip(v)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max)
    };
    // Meio passo já move o campo…
    assert!(
        far(&a, &half) > 0.01,
        "meio pan não moveu: {}",
        far(&a, &half)
    );
    // …e ainda não chegou ao passo inteiro (é um caminho, não um salto).
    assert!(far(&half, &b) > 0.01, "meio pan já era o passo inteiro");
    // E o pan em X anda no eixo do TEMPO, não no da fila — o outro eixo.
    let x_moved = far(
        &a,
        &(0..24)
            .map(|i| Sample { pan_x: 0.5, ..s }.at(i, 0.4))
            .collect::<Vec<_>>(),
    );
    assert!(x_moved > 0.01, "pan_x não moveu o campo: {x_moved}");
}

/// **O pan alcança o modo World** — o eixo espacial não é uma segunda lei.
#[test]
fn the_pan_reaches_the_world_axis_too() {
    let s = before();
    let p = (1.3f32, -0.7f32);
    let plain = s.at_world(p.0, p.1, 0.4);
    let panned = Sample { pan_y: 1.0, ..s }.at_world(p.0, p.1, 0.4);
    // A MESMA identidade do gate da fila: um pan de 1 é um seed de 1.
    let seeded = Sample {
        seed: s.seed + 1.0,
        ..s
    }
    .at_world(p.0, p.1, 0.4);
    assert_ne!(
        plain.to_bits(),
        panned.to_bits(),
        "o pan não alcançou World"
    );
    assert_eq!(panned.to_bits(), seeded.to_bits(), "outra régua em World");
}

/// **E o laço alcança o modo World também** — o mesmo `over_time`, o mesmo fecho.
#[test]
fn the_loop_reaches_the_world_axis_too() {
    let l = 4.0f32;
    let s = Sample {
        loop_period: l,
        ..before()
    };
    for p in [(0.0f32, 0.0f32), (2.5, -1.25), (-3.0, 4.0)] {
        let (a, b) = (s.at_world(p.0, p.1, 0.0), s.at_world(p.0, p.1, l));
        assert!((a - b).abs() < 1e-5, "world {p:?}: {a} != {b}");
    }
}

/// **A COSTURA NÃO É ESPECIAL** — e este gate existe porque uma MUTAÇÃO passou
/// sem ele, depois de nascer VERMELHO sobre código correto.
///
/// ⚠️ **O buraco que a mutação achou:** neutralizar a mistura (`over_time`
/// devolver sempre a primeira amostra) **sobreviveu** aos gates de fecho, e o
/// mecanismo é instrutivo — o wrap sozinho já fecha o VALOR: em `t = L` a folha
/// devolve `u = 0`, logo `τ = 0`, logo `campo(0)`. O que a mistura acrescenta é a
/// **DERIVADA**: sem ela chega-se à costura com inclinação `campo'(L)` e sai-se
/// com `campo'(0)`, dois números sem relação, e um salto de derivada num campo de
/// movimento lê como um TRANCO a cada volta.
///
/// ⚠️ **E o primeiro oráculo que escrevi para isso reprovava o produto:** ele
/// comparava a inclinação de chegada com a de saída por diferenças
/// UNILATERAIS, e as duas diferenças unilaterais de uma função lisa já diferem
/// por `δ·f''` — com quatro oitavas isso mediu `2,5e-1`. *Eu estava a medir a
/// curvatura do campo e a chamar-lhe costura.*
///
/// O oráculo que fica não tem número escolhido: mede o **salto de inclinação** na
/// costura e compara-o com o maior salto que o MESMO campo produz no interior da
/// volta. Uma costura C¹ é indistinguível de um ponto qualquer; uma costura C⁰ é
/// uma quina, e uma quina é sempre o maior salto da curva.
#[test]
fn the_seam_is_not_special() {
    let l = 4.0f32;
    let d = 0.02f32;
    let s = Sample {
        loop_period: l,
        speed: 1.0,
        ..before()
    };
    // O salto de inclinação em `t`: a diferença entre a inclinação que sai e a
    // que chega. Numa curva lisa é `O(δ·f'')`; numa quina é `O(Δf/δ)`.
    let kink = |i: u32, t: f32| -> f32 {
        let out = (s.at(i, t + d) - s.at(i, t)) / d;
        let inc = (s.at(i, t) - s.at(i, t - d)) / d;
        (out - inc).abs()
    };
    let (mut seam, mut interior) = (0.0f32, 0.0f32);
    for i in 0..24u32 {
        // A costura: `t = L` é o mesmo instante que `t = 0`, e o wrap manda
        // `L + δ` de volta para `δ`.
        seam = seam.max(kink(i, l));
        // O interior: trinta pontos espalhados pela volta, longe das pontas.
        for k in 1..31u32 {
            interior = interior.max(kink(i, l * k as f32 / 32.0));
        }
    }
    assert!(
        seam <= interior,
        "a costura é uma QUINA: salto {seam:e} contra {interior:e} no interior"
    );
    // ⚠️ CONTROLE: o campo tem de ter curvatura para a comparação significar
    // algo. Um campo quase reto daria `interior ≈ 0` e o gate acima seria vácuo
    // — mas também impossível de passar, o que é a forma segura de falhar.
    assert!(interior > 1e-2, "o campo é reto demais ({interior:e})");
}
