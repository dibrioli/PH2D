//! ⛔⛔⛔ **AS TRÊS LEIS DE IDA-E-VOLTA DESTA CASA NÃO FECHAVAM, e a prova que as guardava
//! amostrava SEIS pontos.**
//!
//! Todo par *«que caixa serve este texto?»* / *«quanto desta caixa é do texto?»* deste app é da
//! forma `w = t + c` / `orçamento = w − c`. Em aritmética real isso fecha; em `f32` **não**: o
//! arredondamento de `t + c` faz `(t + c) − c` cair **abaixo** de `t`, e a elisão compara `<=`
//! ⇒ *um défice de um ULP corta a palavra inteira*.
//!
//! | par | falhas na varredura | pior défice |
//! |---|---:|---:|
//! | [`rect_for_label`] / [`label_budget`] | `2,65 %` do domínio | `3,05e-5 px` |
//! | [`Tag::width_for`] / [`Tag::label_budget`] | `~96 %` | `3,05e-5 px` |
//! | [`dropdown_chip_width_for`] / [`dropdown_label_budget`] | apanhado pelo PRODUTO | — |
//!
//! # ⚠️ O gate que existia passava nos seis valores que ele escolhia
//!
//! *Uma prova por amostras sobre uma lei que falha em `2,65 %` do domínio lê-se como prova.* Este
//! ficheiro varre o domínio em passos finos, com a mesma aritmética do produto, nos três pares.
//!
//! # ⭐ E o terceiro foi encontrado pelo ECRÃ, não por esta varredura
//!
//! Quando o chip de espécie do editor de variantes passou a pedir **exactamente** o que
//! `Dictionary` mede (`63,62`), ele recebeu esse número de volta e o rótulo saiu **`Dictiona…`**.
//! *A álgebra fecha e a aritmética de máquina não* — e a cura é a inversa perguntar à LEI
//! (`if orçamento(w) < t { w.next_up() }`) em vez de confiar na subtracção.

use ph2d_editor_core::paint::{label_budget, rect_for_label};
use ph2d_editor_core::widget::{Tag, dropdown_chip_width_for, dropdown_label_budget};
use ph2d_editor_core::zones::Rect;

/// ⚠️ **O passo é FINO de propósito.** Com seis amostras os três pares passam; com um passo de
/// `~1e-3` px sobre `0,5..600` cada par recebe `~460 000` pontos, e é aí que o defeito aparece.
const PASSO: f32 = 0.0013;
const INICIO: f32 = 0.5;
const FIM: f32 = 600.0;

/// As alturas em que uma pílula ou um chip vivem neste app — a linha, o ícone, os extremos.
const ALTURAS: &[f32] = &[14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 28.0, 32.0, 36.0];

fn varre(mut f: impl FnMut(f32) -> bool) -> (usize, usize) {
    let (mut n, mut mau) = (0usize, 0usize);
    let mut t = INICIO;
    while t < FIM {
        if !f(t) {
            mau += 1;
        }
        n += 1;
        t += PASSO;
    }
    (n, mau)
}

/// ⭐⭐⭐ **A CAIXA DE RÓTULO: o que a inversa pede, a lei aprova.**
#[test]
fn a_caixa_de_rotulo_fecha_no_dominio_inteiro() {
    let (n, mau) = varre(|t| label_budget(rect_for_label(t)) >= t);
    assert!(
        n > 400_000,
        "a varredura leu {n} pontos — com poucos, este gate vira a prova por amostras que ele \
         existe para substituir"
    );
    assert_eq!(
        mau, 0,
        "{mau} de {n} textos pediram uma caixa cujo orçamento não os aceita de volta"
    );
    // ⛔ CONTROLO: o respiro EXISTE. Sem esta metade, um `rect_for_label` que devolvesse o próprio
    //    texto passaria a varredura acima e o defeito que ela guarda voltava inteiro.
    assert!(
        rect_for_label(20.0) > 20.0,
        "a caixa tem de ser MAIOR que o texto — é isso que o respiro é"
    );
}

/// ⭐⭐⭐ **A PÍLULA: nas duas formas dela, em toda altura em que ela vive.**
#[test]
fn a_pilula_fecha_no_dominio_inteiro() {
    for &h in ALTURAS {
        for removivel in [false, true] {
            let (n, mau) = varre(|t| {
                let w = Tag::width_for(t, h, removivel);
                let pill = Tag::new(ph2d_a11y::NodeId(1), "x").removable(removivel);
                pill.label_budget(Rect::new(0.0, 0.0, w, h)) >= t
            });
            assert_eq!(
                mau, 0,
                "pílula removivel={removivel} a h={h}: {mau} de {n} textos pediram uma largura \
                 cujo orçamento não os aceita de volta"
            );
        }
    }
    // ⛔ CONTROLO: o invólucro EXISTE, e é MAIOR na removível (ela carrega o `×`).
    let lisa = Tag::width_for(40.0, 22.0, false);
    let com_x = Tag::width_for(40.0, 22.0, true);
    assert!(
        lisa > 40.0,
        "uma pílula lisa tem de ser maior que a palavra"
    );
    assert!(
        com_x > lisa,
        "a pílula com `×` ({com_x}) tem de ser mais larga que a lisa ({lisa}) — ela carrega mais \
         uma coisa"
    );
}

/// ⭐⭐⭐ **O CHIP DE ESCOLHA — o par que o `Dictiona…` denunciou.**
#[test]
fn o_chip_de_escolha_fecha_no_dominio_inteiro() {
    for &h in ALTURAS {
        let (n, mau) = varre(|t| {
            let w = dropdown_chip_width_for(t, h);
            dropdown_label_budget(Rect::new(0.0, 0.0, w, h)) >= t
        });
        assert_eq!(
            mau, 0,
            "chip a h={h}: {mau} de {n} rótulos pediram um chip cujo orçamento não os aceita de \
             volta"
        );
    }
    // ⛔ CONTROLO: o invólucro do chip EXISTE (dois recuos, o vão e o chevron).
    assert!(
        dropdown_chip_width_for(40.0, 22.0) > 40.0 + 20.0,
        "o chip tem de reservar o chevron e os recuos, senão esta varredura passa sobre nada"
    );
}
