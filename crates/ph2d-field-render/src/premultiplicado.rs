//! ⭐⭐⭐ **O ALFA DESTA IMAGEM É PRÉ-MULTIPLICADO EM ECRÃ, NUNCA EM LINEAR** — a lei que o
//! compositor deste app lê, medida nele.
//!
//! # ⛔⛔⛔ O defeito que esta porta cura (report do dono, 2026-09-19, com foto)
//!
//! *«toda forma apresenta uma falsa outline branca de 1 pixel»*. Os dois motores faziam a média das
//! quatro sub-amostras da silhueta **em LINEAR** e só **depois** codificavam ⇒ gravavam
//! `sRGB(C·a)`. O consumidor ([`ph2d_vector::StableImage::from_rgba_premultiplied`] → `VelloPass`)
//! espera `sRGB(C)·a`.
//!
//! ⚠️ **A curva sRGB é CÔNCAVA e passa pela origem**, logo `sRGB(C·a) ≥ sRGB(C)·a` para todo
//! `a ∈ [0,1]`, com igualdade **só** em `a = 0` e `a = 1` ⇒ *todo pixel de cobertura parcial saía
//! claro demais, e nenhum saía escuro demais* — que é a forma exacta de um rebordo claro à volta de
//! toda silhueta, clara ou escura. Medido na cena `=36` a `1920×1080`: `+21,5` bytes em média,
//! **`+73` no pior**, e **`0`** canais escuros demais em `9 852`.
//!
//! # ⭐⭐ O CONSUMIDOR foi medido, e não deduzido
//!
//! [`o_pixel_de_meia_cobertura_aterra_entre_os_vizinhos`](../../ph2d-render/tests/it/o_compositor_e_meia_cobertura.rs)
//! desenha as duas convenções sobre um fundo `110` com a peça a `255`, pelo `VelloPass` real:
//!
//! | o que a imagem leva no pixel do meio | o que o compositor devolve |
//! |---|---|
//! | `188` (`sRGB(C·a)` — a lei de ontem) | **`243`** — *fora* do intervalo `[110, 255]` |
//! | `128` (`sRGB(C)·a`) | **`183`** — entre os dois vizinhos |
//!
//! ⇒ o compositor faz `img + fundo·(1−a)` **em bytes**, e `183 = 128 + 110·0,5` fecha a conta.
//!
//! ⚠️⚠️ **E este repositório tinha uma nota MEDIDA a dizer o contrário** — a
//! `ph2d_render::premul::premultiply_rgba8_in_linear` documenta que pré-multiplicar **em linear**
//! é o que cura um *«light halo at the silhouette edge»*. Ela descreve o pipeline de SPRITES, que
//! tem **dois** consumidores (o shader com decode de hardware e o Vello) e por isso outra resposta.
//! *Uma lei portada traz a premissa do alvo sobre a disposição DELE* — por isso este caminho foi
//! medido no consumidor DELE, e não lido daquela nota.
//!
//! # ⚠️ Porque a porta fala em ALFA-BYTE e não em `f32`
//!
//! O consumidor compõe com o alfa que está **no byte**. Pré-multiplicar por um `f32` que depois é
//! arredondado para outro valor deixa produtor e consumidor a usar dois números — pequeno, e
//! exactamente o tipo de meia-verdade que faz uma ida-e-volta deixar de fechar.
//!
//! # ⛔⛔⛔ A LUZ ADITIVA ENTRA POR UM ARGUMENTO PRÓPRIO, e não no mesmo `vec4`
//!
//! A luz que a peça devolve ao chão **soma sem tapar** — ela não é `C·a` e dividi-la por `a` é
//! inventar uma cobertura que ela não tem. Enfiada no mesmo `vec4` que a cobertura (a forma que a
//! `ground_shade::mais_luz` tinha), o empacotamento dividia-a pelo alfa da SOMBRA, e o gate
//! `a_luz_devolvida_atravessa_o_material_do_chao` apanhou-o na primeira corrida: `0,004777` contra
//! `0,011194`.
//!
//! ⚠️ ⇒ **duas grandezas, dois argumentos:** `sRGB(C)·a + sRGB(L)`. O compositor soma os bytes
//! (`img + fundo·(1−a)`, medido), logo a luz gravada como `sRGB(L)` chega ao ecrã **inteira**,
//! qualquer que seja o alfa.
//!
//! # ⭐ O ALCANCE DA CURA é exactamente a população do defeito
//!
//! Com `a = 255` a divisão e a multiplicação cancelam-se; com `a = 0` a cobertura é zero e só a luz
//! passa; com `L = 0` e `a` cheio nada muda. ⇒ **a peça opaca, o fundo limpo e o chão com luz
//! devolvida não mudam um bit**, e o que muda é a banda de cobertura PARCIAL — que é onde o dono
//! fotografou o rebordo.

/// De um pixel para os bytes que o compositor espera — **a cobertura e a luz entram SEPARADAS**.
///
/// * `cobertura` é a cor que TAPA, pré-multiplicada em linear (`C·a`).
/// * `alfa` é o byte que vai ser gravado no canal `A` — ver a nota do módulo.
/// * `luz` é a que SOMA sem tapar, em linear.
#[must_use]
pub(crate) fn para_ecra(cobertura: [f32; 3], alfa: u8, luz: [f32; 3]) -> [u8; 3] {
    let a = f32::from(alfa) / 255.0;
    let inv = if alfa == 0 { 0.0 } else { 1.0 / a };
    [0, 1, 2].map(|k| {
        let tapa = ph2d_color::srgb::linear_to_srgb_unit(cobertura[k] * inv) * a;
        let soma = ph2d_color::srgb::linear_to_srgb_unit(luz[k]);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            (tapa + soma).mul_add(255.0, 0.5).clamp(0.0, 255.0) as u8
        }
    })
}

#[cfg(test)]
#[path = "premultiplicado_tests.rs"]
mod premultiplicado_tests;
