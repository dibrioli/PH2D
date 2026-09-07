//! Os gates do vidro jateado que **não precisam de uma GPU**.
//!
//! ⚠️ O passe é textura→textura: provar o que ele PINTA exige um adapter, e os gates de GPU deste
//! repo são `#[ignore]` (o CI nunca os corre). O que se prende aqui é o que uma leitura rápida do
//! diff entende ao contrário e custa uma janela do Enio a diagnosticar.

use super::FrostPass;

/// ⛔ **Uma janela minimizada não pode produzir uma textura de lado ZERO** — isso é erro de
/// validação do wgpu, e o app morre ao minimizar.
///
/// **Mutação que deve sangrar:** tirar o `.max(1)` de qualquer um dos dois eixos.
#[test]
fn the_work_size_is_never_zero() {
    for size in [(0, 0), (1, 1), (1, 900), (1600, 1)] {
        let (w, h) = FrostPass::work_size(size);
        assert!(w >= 1 && h >= 1, "{size:?} deu {w}x{h}");
    }
}

/// ⭐ **A resolução de trabalho é METADE, e isso é metade do borrão.**
///
/// O kernel de 5 taps tem alcance fixo em texels; correr na tela cheia daria um borrão de metade
/// do raio — visivelmente mais duro — e quatro vezes mais caro. *Um número que decide o LOOK não
/// pode viver só num literal no meio de uma expressão.*
#[test]
fn the_work_size_halves_the_screen() {
    assert_eq!(FrostPass::work_size((1600, 900)), (800, 450));
    assert_eq!(FrostPass::work_size((1366, 1024)), (683, 512));
}

// ⛔ **Não há gate sobre a FORÇA do véu, e a ausência é a decisão.** Um `assert!` de uma constante
// contra dois literais é uma asserção **vácua** — ela não pode reprovar nada que a compilação já
// não saiba, e o clippy diz-o pelo nome (`assertions_on_constants`). A faixa em que aquele número
// é honesto é um juízo de LOOK, e o sítio dele é o doc da constante, onde as duas falhas que ela
// evita estão nomeadas. *Um gate que só sabe reler o literal dá a impressão de o defender.*
