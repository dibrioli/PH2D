//! ⭐⭐⭐ **NENHUMA PALAVRA DA PONTE DO PAINTER É ESCRITA NO FONTE** — o HR-15 na crate de família da
//! pintura.
//!
//! Medido em 2026-09-16: **18** textos — os avisos de carregar uma textura/forma de pincel, as
//! quedas da pré-visualização de GPU para o caminho de CPU, e a recusa de trocar de sprite com o
//! Painter aberto. Migrados para `ph2d-i18n/src/app_painter.rs`.
//!
//! ⚠️ **Quatro deles estavam em PORTUGUÊS num app inglês** (*«upload da preview pra GPU falhou»*) e
//! foram reescritos em inglês na tabela — a chave deles não guarda a redacção antiga.
//!
//! ⚠️ **As cenas de smoke do Painter vivem aqui desde a W2 Fase D** (a nota dizia que viviam na
//! shell), e até 2026-09-23 nenhuma escrevia um literal fora de um `println!` — por isso o gate media
//! ZERO. O `composite_smoke` pôs o primeiro (o NOME do objecto da tela, que entra no `Name` e é
//! identidade durável — a tabela `app_painter.rs` exclui-os por escrito), e o zero virou um PISO.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_painter.rs";

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[(
    "paint_perf.rs",
    "o RETRATO de performance impresso no terminal (`GPU`/`CPU`, `TELA`/`rect` numa linha de \
     `println!`) — diagnóstico de quem caça um engasgo, nunca ecrã",
)];

/// Nenhum literal-identificador sobra nesta crate — a lista existe para o dia em que sobrar.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, gate::CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da ponte do Painter (HR-15):\n  {}\n\n\
         A cura é uma chave `app.painter.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`). ⚠️ Um `const` não pode chamar `tr`: ele guarda \
         uma `ph2d_i18n::TextKey` e quem pinta escreve `.tr()` — ver `painter_lock::REFUSAL`.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa** — e o ZERO de cenas medido, não suposto.
#[test]
fn every_named_exemption_still_shelters_what_it_names() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas: Vec<String> = gate::excecoes_mortas(&src, NOT_LANGUAGE)
        .into_iter()
        .chain(gate::isentos_mortos(&src, FORA))
        .collect();
    assert!(
        mortas.is_empty(),
        "isenções mortas:\n  {}",
        mortas.join("\n  ")
    );
    let em_cena = gate::literais_de_cena(&src, gate::CENAS);
    assert!(
        em_cena >= 1,
        "as cenas de smoke abrigam {em_cena} textos — em 2026-09-23 era 1 (o nome da tela do \
         `composite_smoke`). Ou ele saiu desta crate, ou a régua por NOME deixou de casar e este \
         gate mede o nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.painter.", &[TABLE]);
    assert!(
        c.declaradas >= 15 && c.usadas >= 15,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas — o `tr` pinta o identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}
