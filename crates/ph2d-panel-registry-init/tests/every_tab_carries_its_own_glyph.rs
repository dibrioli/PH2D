//! ⭐⭐⭐ **CADA PAINEL TEM O GLIFO DELE, E NENHUM PARTILHA O DE OUTRO** — a metade de POPULAÇÃO do
//! ícone da aba.
//!
//! > *«as abas … colocam os `...` no nome»* — Enio, 2026-09-08.
//!
//! Uma aba encolhe até um quadrado de [`ph2d_tokens::ROW_H_PX`] e o nome sai
//! (`hero::slot_tabs_face`). O que resta a dizer *de que painel se trata* é o glifo — e **dois
//! painéis com o mesmo desenho são piores que o `…` que isto veio curar**: aquele pelo menos não
//! finge dizer alguma coisa.
//!
//! # ⚠️ A população é TODO painel registado, e não só os que hoje ganham aba
//!
//! O [`Panel::CAN_FLOAT`] exclui um painel da fileira — e é **uma linha** que qualquer wave pode
//! virar. Uma lei que só olhasse os encaixáveis passaria a admitir uma colisão no dia em que
//! alguém tornasse um flutuante encaixável, **sem tocar em nenhum dos dois glifos**. *Uma lei cuja
//! população depende de uma declaração mutável cai quando essa declaração muda.*
//!
//! ⚠️ **E o artista move qualquer painel para qualquer encaixe** (decisão D4), logo *quaisquer
//! dois* podem acabar lado a lado — não há par isento por posição.

use std::collections::BTreeMap;

/// ⚠️ **O controlo do INSTRUMENTO.** Sem registo, a varredura corre sobre zero painéis e um censo
/// de unicidade sobre um conjunto vazio é **verde por vacuidade** — a forma exacta com que este
/// ficheiro passaria a não medir nada.
const AT_LEAST: usize = 20;

fn glyphs() -> Vec<(&'static str, &'static str)> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            v.push((p.manifest.id, p.manifest.icon.slug()));
        }
    });
    v
}

/// ⭐⭐⭐ **Dois painéis nunca desenham a mesma aba.**
#[test]
fn no_two_panels_share_a_glyph() {
    let all = glyphs();
    assert!(
        all.len() >= AT_LEAST,
        "só {} painéis registados — a varredura perdeu o registry e o censo de unicidade seria \
         verde sobre nada",
        all.len()
    );

    let mut by_slug: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (id, slug) in &all {
        by_slug.entry(slug).or_default().push(id);
    }
    let shared: Vec<String> = by_slug
        .iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(slug, ids)| format!("{slug} → {ids:?}"))
        .collect();
    assert!(
        shared.is_empty(),
        "estes painéis partilham um glifo, e espremidos ficam indistinguíveis:\n  {}\ncura: \
         escolha outro `Panel::ICON` (o catálogo é `docs/design/icons/*.svg`) ou acrescente um \
         SVG novo — um glifo por painel é a identidade que sobra quando o nome não cabe.",
        shared.join("\n  ")
    );
}

/// ⚠️ **E o glifo tem de EXISTIR**: o `IconId` indexa tabelas geradas do SVG por discriminante, e
/// um variante sem ficheiro devolveria comandos de outro desenho — ou nenhum.
#[test]
fn every_declared_glyph_draws_something() {
    let mute: Vec<&str> = glyphs()
        .into_iter()
        .filter_map(|(id, slug)| {
            let cmds = ph2d_editor_core::panel::with_registry_ref(|reg| {
                reg.panels()
                    .iter()
                    .find(|p| p.manifest.id == id)
                    .map(|p| p.manifest.icon.cmds().len())
                    .unwrap_or(0)
            });
            (cmds == 0).then_some(slug)
        })
        .collect();
    assert!(
        mute.is_empty(),
        "estes glifos de aba não desenham nada: {mute:?}"
    );
}
