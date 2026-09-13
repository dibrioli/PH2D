//! **`ph2d-label-fold` — quando dois nomes escritos por um artista são O MESMO.**
//!
//! Decisão do dono (2026-09-13, `docs/Components/08_plano_tags.md` §2.5 D2): *«maiúscula não
//! importa, letra acentuada não importa»*. `Inimigo`, `inimigo` e `inímigo` são uma tag só.
//!
//! # ⛔ Uma porta, e só uma
//!
//! [`fold`] é a ÚNICA resposta desta casa a *«estes dois rótulos são o mesmo?»*. Um `to_lowercase`
//! escrito à mão noutro sítio é uma segunda regra, e as duas discordam em silêncio no primeiro `ß`.
//!
//! # ⚠️ Divergência DECLARADA do Blender e do Godot
//!
//! Os dois distinguem as três grafias — medido nesta máquina (§1.1 do plano): o Blender cria três
//! coleções `Inimigo`, `inimigo`, `inímigo`, e o Godot responde `is_in_group("Enemy") == false`
//! para um nó de `enemy`. É a decisão do dono que manda aqui, e ela é a do *Gameplay Tags* do Unreal
//! (comparação sem distinção de maiúsculas) levada até aos acentos.
//!
//! # ⚠️ A chave NÃO é o que se mostra
//!
//! Esta função devolve uma **chave de comparação**, nunca um nome a pintar: quem escreveu
//! `Inimigo Voador` continua a ler `Inimigo Voador`. É a regra de um sistema de ficheiros
//! *case-preserving*.

use icu_casemap::CaseMapperBorrowed;
use icu_normalizer::DecomposingNormalizerBorrowed;
use icu_properties::CodePointMapData;
use icu_properties::props::GeneralCategory;

/// ⭐ **A chave de comparação de um rótulo** — os quatro passos, e a ordem é a do *caseless
/// matching* canónico do Unicode (§3.13 do padrão) com a remoção dos acentos no fim:
///
/// 1. **NFD** — `é` pré-composto e `e` + U+0301 passam a ser a mesma sequência;
/// 2. ***case folding* COMPLETO** (`icu_casemap`) — ⚠️ não `to_lowercase`: só o *folding* leva o
///    `ß` a `ss`, e sem ele `Straße` e `STRASSE` seriam duas tags;
/// 3. **NFD outra vez** — o *folding* pode produzir sequências que voltam a decompor;
/// 4. **sem marcas não-espaçadoras** (`GeneralCategory::NonspacingMark`) — é aqui que o acento sai.
///    ⚠️ Pela CATEGORIA e não por uma tabela de letras: uma lista escrita à mão esquece o primeiro
///    alfabeto que ninguém testou.
///
/// E, por fim, os espaços colapsam (`Inimigo  Voador` = `inimigo voador`) — um espaço a mais não
/// pode ser uma tag nova.
#[must_use]
pub fn fold(label: &str) -> String {
    let nfd = DecomposingNormalizerBorrowed::new_nfd();
    let categoria = CodePointMapData::<GeneralCategory>::new();
    let decomposto = nfd.normalize(label);
    let dobrado = CaseMapperBorrowed::new().fold_string(&decomposto);
    let redecomposto = nfd.normalize(&dobrado);
    let sem_marcas: String = redecomposto
        .chars()
        .filter(|&c| categoria.get(c) != GeneralCategory::NonspacingMark)
        .collect();
    sem_marcas.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
#[path = "fold_tests.rs"]
mod tests;
