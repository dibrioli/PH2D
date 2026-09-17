//! ⭐⭐ **O PROGRAMA NÃO ESCREVE PALAVRA NENHUMA NESTE PAINEL** — o HR-15 na crate que não tem
//! tabela de strings.
//!
//! ⚠️ **Este é o único painel sem `ph2d-i18n`, e a ausência é decisão escrita** (`Cargo.toml`): o
//! título e o rótulo de cada row são o `Name` que o ARTISTA digitou no desenho, e o gerador emite-os
//! tal e qual para `src/generated/`. Passá-los por `tr()` daria a impressão de um painel traduzível
//! quando ele é *autorado*. ⇒ a lei aqui é a outra metade: **fora do ficheiro gerado não pode haver
//! texto com cara de língua** — um rótulo que o código deste painel escrevesse seria do programa, e
//! não teria para onde ir.
//!
//! ⛔ **O que se isenta, e porquê:**
//! - `generated/` — o desenho do artista, emitido pelo gerador. A garantia de que ali só há o que o
//!   gerador emite é o gate irmão `the_generated_panel_is_what_the_emitter_emits` (byte a byte).
//!
//! ✅ **A excepção do `Panel::TITLE` SAIU em 2026-09-17, e com ela a dívida:** aquela `const` passou
//! a ser um `TextKey` (`panel.authored.title`) e quem pinta a aba traduz — logo o literal saiu do
//! binário, não só do alcance deste censo. ⚠️ **Isto não contradiz a decisão do `Cargo.toml`:** o
//! que fica sem tabela são as ROWS, que são o `Name` do artista; o nome do PAINEL é do programa, e
//! o menu *Window* já o traduzia (`chrome.menu.authored_ui`) — a aba é que não.

use ph2d_label_census::gate::{self, Excecao};
use ph2d_label_census::language_literals;

const GERADO: &str = "generated/";

const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn outside_the_generated_design_no_word_is_written_in_the_source() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos: Vec<String> = gate::intrusos(&src, NOT_LANGUAGE)
        .into_iter()
        .filter(|l| !l.starts_with(GERADO))
        .collect();
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito pelo PROGRAMA no painel autorado (HR-15):\n  {}\n\n\
         Este painel não tem tabela de strings de propósito (ver o `Cargo.toml`): o texto dele é o \
         do desenho. Um rótulo do programa aqui é um rótulo sem tradução possível — ele pertence à \
         moldura (`ph2d-i18n/src/chrome*.rs`), não a este painel.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa** — a excepção do título ainda abriga o literal, e a do desenho ainda abriga
/// o desenho. ⚠️ O piso do desenho é também o controlo de vacuidade: uma régua cega devolve zero
/// literais em `generated/` e o gate acima leria verde sobre nada.
#[test]
fn the_exemptions_still_shelter_what_they_name() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções mortas:\n  {}",
        mortas.join("\n  ")
    );
    let desenho = language_literals(&src)
        .into_iter()
        .filter(|l| l.rel.starts_with(GERADO))
        .count();
    assert!(
        desenho >= 10,
        "o desenho emitido tem {desenho} textos — em 2026-09-16 eram 20; ou o painel de amostra \
         encolheu (baixe o piso com o número novo) ou a régua deixou de ler `generated/`"
    );
}
