//! ⭐⭐⭐ **A SONDA DO ORÇAMENTO DO BALÃO** — o que o artista LÊ, e não o que foi escrito.
//!
//! ⛔⛔ Report do dono, 22/09: *«as mensagens estão cortadas com … não consigo ler tudo»*. Ela
//! mede pela régua do PRODUTO ([`crate::toast::text_budget_px`] + [`crate::text_elide::fit`]) e
//! imprime a frase elidida ao lado da inteira. Corra-a com `-- --nocapture`.
//!
//! ⚠️ **Ela fica VERSIONADA e não se apaga depois da cura:** as duas leituras (antes e depois)
//! valem uma pela outra, e foi assim que esta casa mediu a tinta fina.
//!
//! ⚠️⚠️ **O que ela mediu, e que muda a ATRIBUIÇÃO:** a frase de ANTES da cláusula da tinta fina
//! já media `60` caracteres contra um orçamento de ~`48` — *o aviso do que um formato não carrega
//! nunca foi legível*. Pré-existente, e não uma dívida que aquela wave criou.
//!
//! ⛔ **As frases são LITERAIS aqui de propósito:** esta crate não alcança a `ph2d-mesh` (medido),
//! e quem mede o texto do produto formato a formato é o gate
//! `o_aviso_cabe_no_balao`, na família da escultura, que alcança as duas.

use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;

/// O que a exportação escrevia ANTES da cura — as duas cortadas, e é esse o achado.
const ANTES: &[&str] = &[
    "Exported 1 piece(s), 412 KB -- teste.obj (not carried: mask, fine paint (mesh resolution only))",
    "Exported 1 piece(s), 118 KB -- teste.obj (not carried: mask)",
];

/// ⭐⭐ **As quatro medições que DECIDIRAM a partição**, e nenhuma é decoração.
///
/// A segunda é a que matou a frase única: ela cabe com `teste.obj` e **estoura com um nome de
/// ficheiro real** — e a elisão corta o FIM, que é exactamente onde o aviso está. A terceira e a
/// quarta são o pior caso do aviso (o STL perde tudo) com e sem a EXPLICAÇÃO: *um nome perde a
/// explicação antes de perder letras*, e é o parêntesis que faz a diferença entre `68` e `45`.
const DECIDIRAM: &[&str] = &[
    "teste.obj, 412 KB -- not saved: mask, fine paint",
    "retrato-da-personagem-v3.obj, 412 KB -- lost: mask, fine paint",
    "Lost: mask, colour, pieces merged, fine paint (mesh resolution only)",
    "Lost: mask, colour, pieces merged, fine paint",
];

#[test]
fn diag_o_que_cabe_no_balao() {
    let mut text = TextSystem::without_system_fonts();
    let orcamento = crate::toast::text_budget_px();
    let corpo = TypeToken::Base.px();

    eprintln!("\n── ORÇAMENTO DO BALÃO ──────────────────────────────────────");
    eprintln!(
        "coluna {:.0} px · corpo {:.1} px · orçamento do TEXTO {:.1} px",
        crate::progress::toast_column_w(),
        corpo,
        orcamento
    );
    for (rotulo, lista) in [("ANTES da cura", ANTES), ("o que decidiu", DECIDIRAM)] {
        eprintln!("\n  {rotulo}:");
        for frase in lista {
            let vista = crate::text_elide::fit(&mut text, frase, corpo, orcamento);
            eprintln!(
                "  {:>3} ch  {}  {vista}",
                frase.chars().count(),
                if vista == *frase { "✓" } else { "✗" }
            );
        }
    }
    eprintln!("────────────────────────────────────────────────────────────\n");
}
