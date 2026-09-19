//! Os portões da [`super`] — a lei do pré-multiplicado em ecrã.

use super::para_ecra;

/// Nenhuma luz acrescentada — o caso de toda a peça e de toda a silhueta.
const SEM_LUZ: [f32; 3] = [0.0; 3];

/// ⭐⭐⭐ **A PEÇA OPACA E O FUNDO LIMPO NÃO MUDAM UM BIT** — os dois pontos em que a lei nova e a de
/// ontem coincidem, e a razão de esta wave não reabrir toda paridade já paga.
///
/// ⚠️ Com `a = 255` a divisão e a multiplicação cancelam-se **exactamente** (`v/1 · 1`); com
/// `a = 0` a cobertura é zero e nada é dividido. *Uma cura cujo alcance é exactamente a população
/// do defeito é o que torna as barras dos outros gates ainda válidas.*
#[test]
fn nos_dois_extremos_a_lei_nova_e_a_de_ontem() {
    for i in 0..=255u16 {
        let v = f32::from(i) / 255.0;
        let linear = ph2d_color::srgb::srgb_to_linear_unit(v);
        let ontem = ph2d_color::srgb::linear_to_srgb_byte(linear);
        assert_eq!(
            para_ecra([linear; 3], 255, SEM_LUZ)[0],
            ontem,
            "com cobertura CHEIA a lei nova tem de ser a de ontem, e divergiu em {i}"
        );
        assert_eq!(
            para_ecra(SEM_LUZ, 0, [linear; 3])[0],
            ontem,
            "a LUZ com cobertura zero tem de passar intacta, e divergiu em {i}"
        );
    }
}

/// ⭐⭐⭐ **A LEI, no número que o compositor foi medido a devolver.**
///
/// Uma peça branca (`C = 1,0` em linear) a cobrir meio pixel tem de gravar `128` e **não** `188` —
/// ver a tabela do cabeçalho da [`super`], e o gate que a mediu no `VelloPass` real.
#[test]
fn meia_cobertura_de_branco_grava_metade_do_branco() {
    let a = 128u8;
    let premul = f32::from(a) / 255.0; // `C·a` com `C = 1,0`
    assert_eq!(
        para_ecra([premul; 3], a, SEM_LUZ),
        [128, 128, 128],
        "meia cobertura de branco tem de gravar metade do BRANCO (`sRGB(C)·a`), e não \
         `sRGB(C·a)` — que é o rebordo claro que o dono fotografou"
    );
    // ⭐ O CONTROLO: a lei de ontem, calculada aqui, é o número que ele via.
    assert_eq!(
        ph2d_color::srgb::linear_to_srgb_byte(premul),
        188,
        "a lei de ontem deixou de dar 188 — então este gate já não está a contrastar com o defeito"
    );
}

/// ⭐⭐⭐ **A LUZ ADITIVA NÃO É DIVIDIDA PELO ALFA** — o defeito que o gate do chão apanhou.
///
/// ⚠️ A mesma luz tem de gravar o **mesmo byte** com qualquer cobertura por baixo: ela soma no
/// compositor (`img + fundo·(1−a)`, medido) e a cobertura da SOMBRA não é a dela. *Uma luz que
/// encolhe quando a sombra escurece é a sombra a comer a luz que a peça devolve.*
#[test]
fn a_luz_nao_encolhe_com_a_cobertura_por_baixo() {
    let luz = [0.25f32; 3];
    let so_a_luz = para_ecra(SEM_LUZ, 0, luz)[0];
    for alfa in [0u8, 1, 40, 128, 200, 255] {
        // A cobertura é PRETA (o fundo do modelador), logo ela contribui zero em todo alfa.
        let com_sombra = para_ecra(SEM_LUZ, alfa, luz)[0];
        assert_eq!(
            com_sombra, so_a_luz,
            "com alfa {alfa} a luz gravou {com_sombra} contra {so_a_luz} sem sombra — ela está a \
             ser dividida por uma cobertura que não é dela"
        );
    }
}

/// ⭐ **NENHUMA COBERTURA PARCIAL SAI MAIS CLARA DO QUE A LEI DE ONTEM** — a concavidade da curva,
/// afirmada sobre a porta.
///
/// ⚠️ É o controlo de SENTIDO: se um dia alguém inverter a conta, este gate acusa antes de a foto o
/// fazer. *A medição no produto leu `0` canais escuros demais em `9 852`; aqui a mesma afirmação é
/// exaustiva.*
#[test]
fn a_cura_so_escurece_e_nunca_clareia() {
    for alfa in 1..255u8 {
        let a = f32::from(alfa) / 255.0;
        for i in 0..=255u16 {
            let c = f32::from(i) / 255.0;
            let linear = ph2d_color::srgb::srgb_to_linear_unit(c);
            let novo = para_ecra([linear * a; 3], alfa, SEM_LUZ)[0];
            let ontem = ph2d_color::srgb::linear_to_srgb_byte(linear * a);
            assert!(
                novo <= ontem,
                "com alfa {alfa} e cor {i} a lei nova devolveu {novo} contra {ontem} de ontem — a \
                 curva sRGB é côncava e isto não pode acontecer"
            );
        }
    }
}

/// ⛔ **E UM PIXEL IMPOSSÍVEL SATURA em vez de estourar.** Num pré-multiplicado `rgb ≤ alfa`
/// sempre; um chamador novo que se engane tem de receber um byte, e não um `NaN`.
#[test]
fn um_pixel_impossivel_satura_em_vez_de_explodir() {
    let saida = para_ecra([1.0; 3], 1, SEM_LUZ);
    assert_eq!(
        saida,
        [1, 1, 1],
        "o byte saturado tem de ser o próprio alfa — `C = 1,0` é o mais claro que existe"
    );
    assert_eq!(
        para_ecra([f32::INFINITY; 3], 128, [f32::INFINITY; 3]),
        [255, 255, 255],
        "um valor não-finito tem de saturar no branco, e não escrever lixo"
    );
}
