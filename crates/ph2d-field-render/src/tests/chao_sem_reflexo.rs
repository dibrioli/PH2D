//! ⭐⭐⭐ **A SOMBRA DO CHÃO NÃO MUDA QUANDO A CÂMERA RODA** — a promessa que a
//! [`crate::ground::catcher_surface`] escreve sobre si mesma, e que o código violava.
//!
//! O doc dela diz, por extenso: *«Sem especular de propósito: um reflexo depende da direcção de
//! vista, e a sombra de um chão não pode mudar quando a câmera roda.»* Ela desliga o realce
//! (`specular_weight: 0`) e ficava por aí.
//!
//! ⛔⛔⛔ **E isso não chegava.** O `specular_weight` do OpenPBR não multiplica o lobo: ele modula o
//! ÍNDICE, e a zero o índice fica `1` — um interface que não existe. O albedo direccional do lobo
//! indirecto tinha o `F90` **cravado em `1`**, logo aquela superfície reflectia o céu no rasante, e
//! o [`crate::ground_shade::catcher`] lê-a **com o `v` do pixel** ⇒ *a razão da sombra dependia de
//! onde a câmera estava*. A cura vive em [`ph2d_material::bsdf::grazing_dielectric`], e este gate é
//! a promessa do chão a ser **afirmada** em vez de escrita.
//!
//! ⚠️ **Ele mede a SUPERFÍCIE e não o pixel, de propósito:** a razão do `catcher` é
//! `luz_que_chega / luz_livre`, e a única entrada dela que vê o `v` é aquele `indirect`. *Um gate
//! que pintasse dois quadros de ângulos diferentes mediria também o reenquadramento, e não saberia
//! dizer qual das duas coisas mudou.*
//!
//! # ⛔⛔ E a PRIMEIRA redacção desta régua media o NADA — o teste da FORNALHA
//!
//! Ela usava um céu **uniforme** (`radiance = irradiance = 1`) sobre a difusa **branca** do chão, e
//! o CONTROLO — a mesma superfície com o realce LIGADO — leu `1,000000` em todos os oito olhares.
//! ⭐ *Isso não é cegueira da régua, é a lei a estar certa*: sob um céu uniforme uma superfície
//! branca devolve exactamente `1` em todo ângulo, porque o que o realce reflecte é exactamente o
//! que ele tira ao difuso por baixo (`fg + albedo·(1 − fg)`, com `albedo = 1`). **Um teste de
//! fornalha é cego a QUALQUER redistribuição entre os lobos** — que é precisamente a grandeza que
//! esta régua tem de ver.
//!
//! ⇒ o céu desta régua é só o **ESPELHO** (`radiance = 1`, `irradiance = 0`): o que sai é o albedo
//! direccional do realce e mais nada.

use crate::ground::LUMA;

/// Um céu que só existe para o lobo ESPELHADO — ver o cabeçalho.
struct SoEspelho;

impl ph2d_material::Environment for SoEspelho {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [1.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [0.0; 3]
    }
}

/// ⭐⭐⭐ **A RÉGUA DO CHÃO NÃO REFLECTE O CÉU EM ÂNGULO NENHUM** — e o CONTROLO é a mesma difusa
/// branca com o realce ligado, que reflecte e varia com o olhar.
#[test]
fn a_regua_do_chao_nao_reflecte_o_ceu() {
    let branco = crate::ground::catcher_surface();
    let com_realce = ph2d_material::OpenPbr {
        base_color: [1.0; 3],
        ..ph2d_material::OpenPbr::default()
    }
    .prepare();
    // A normal do chão em VISTA é `+y` para uma câmera de nível; o que varia aqui é o OLHAR.
    let n = [0.0, 1.0, 0.0];
    // Oito direcções sobre o hemisfério, da quase-vertical à quase-rasante — é no rasante que o
    // `F90` manda, e é ali que a sombra de um chão é olhada.
    let olhares: Vec<[f32; 3]> = (0..8)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let t = 0.1 + 0.8 * k as f32 / 7.0;
            let (s, c) = (t * std::f32::consts::FRAC_PI_2).sin_cos();
            [c, s, 0.0]
        })
        .collect();
    let faixa = |s: &ph2d_material::Surface| -> (f32, f32) {
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for v in &olhares {
            let r = s.indirect(n, *v, &SoEspelho);
            let y = LUMA[0] * r[0] + LUMA[1] * r[1] + LUMA[2] * r[2];
            lo = lo.min(y);
            hi = hi.max(y);
        }
        (lo, hi)
    };
    let (lo, hi) = faixa(&branco);
    assert!(
        hi <= 1.0e-6,
        "a régua do chão reflecte o céu ({lo:.6}..{hi:.6}) — ela é uma DIFUSA com o realce \
         desligado, e o `catcher` lê-a com o `v` do pixel: a sombra do chão muda quando a câmera \
         roda, e o doc da `catcher_surface` promete que não"
    );
    // ⭐ O CONTROLO: com o realce LIGADO a mesma régua TEM de reflectir e de variar com o olhar,
    // senão a asserção acima passa por vácuo sobre uma fixtura que não contém o fenómeno.
    let (clo, chi) = faixa(&com_realce);
    assert!(
        clo > 1.0e-4 && chi - clo > 1.0e-2,
        "o CONTROLO ficou mudo ({clo:.6}..{chi:.6}): uma superfície COM realce reflecte o céu e \
         depende do olhar"
    );
}
