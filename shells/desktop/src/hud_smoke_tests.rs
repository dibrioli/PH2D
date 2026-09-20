//! Os gates da ESCADA da cena do HUD — *ela tem fim?*
//!
//! ⛔⛔ **Report do dono, 2026-09-20: «infinitos logs. melhor tirar.»** A conferência do dedo corria
//! no quadro em que a subida do Inspector chegava a `0`, e [`crate::App::levanta_o_inspector`]
//! devolve `0` para sempre a partir daí ⇒ ela corria em **todos** os quadros seguintes, a `60 Hz`.
//!
//! *Um contador que SATURA não é um estado terminal* — e o que esta lei afirma é o fim da escada.

use super::{FEITO, proximo_estado};

/// ⭐⭐⭐ **A escada TERMINA**: [`FEITO`] é ponto fixo, e é dele que sai a garantia de a cauda da
/// cena correr uma vez só.
///
/// **Mutação que deve sangrar:** um braço que devolva `2` no estado terminal (que é o estado de
/// antes desta cura: a escada voltava sempre ao passo da subida).
#[test]
fn a_escada_da_cena_do_hud_tem_fim() {
    // ⚠️ O CONTROLO vem primeiro: a escada de facto ANDA. Sem ele, um `proximo_estado` que
    // devolvesse `FEITO` sempre passaria o `assert` de baixo a medir o nada.
    assert_eq!(proximo_estado(0, 0), 1, "a cena nao sai do 1.o quadro");
    assert_eq!(proximo_estado(1, 0), 2, "a cena nao veste os componentes");
    assert_eq!(
        proximo_estado(2, 1),
        2,
        "a subida do Inspector foi saltada — com quadros por correr o estado tem de FICAR"
    );
    assert_eq!(
        proximo_estado(2, 0),
        FEITO,
        "a cena nunca chega ao fim, e a conferencia do dedo nunca corre"
    );

    // ⛔ O ponto fixo: qualquer que seja a subida, quem chegou ao fim fica lá.
    for subida in [0_u8, 1, 3, 255] {
        assert_eq!(
            proximo_estado(FEITO, subida),
            FEITO,
            "o estado terminal voltou atras com subida={subida} — a cauda da cena volta a correr, \
             e com ela uma linha de diagnostico por QUADRO"
        );
    }
}

/// ⭐⭐ **A cauda corre na TRANSIÇÃO, nunca no estado** — a metade que faz a de cima valer alguma
/// coisa no produto.
///
/// ⚠️ Textual, e pela razão dos irmãos: o `hud_smoke` é um método de `App` que exige janela e GPU.
/// O que se afirma é a FORMA da condição, que é onde o defeito morava.
///
/// **Mutação que deve sangrar:** tirar o `estado != FEITO` da guarda.
#[test]
fn a_conferencia_do_dedo_corre_na_transicao_e_nao_no_estado() {
    let src = include_str!("hud_smoke.rs");
    let i = src
        .find("self.hud_smoke_confere_o_dedo();")
        .expect("a cena deixou de conferir o dedo");
    let guarda = src[..i]
        .rfind("if seguinte == FEITO")
        .expect("a conferencia do dedo deixou de ser guardada pela transicao");
    assert!(
        src[guarda..i].contains("estado != FEITO"),
        "a guarda olha o ESTADO e nao a TRANSICAO — a conferencia volta a correr a cada quadro"
    );
}
