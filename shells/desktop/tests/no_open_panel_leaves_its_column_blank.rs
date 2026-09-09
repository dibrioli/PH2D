//! ⭐⭐⭐ **UM PAINEL ABERTO DEVE AO ARTISTA UMA SUPERFÍCIE** — e calar-se custa-lhe a COLUNA.
//!
//! # O defeito, medido em 2026-09-09
//!
//! Com o `sculpt3d` e o `inspector` a partilharem a coluna da direita e o módulo 3D desarmado:
//!
//! | quem está à frente | rect do sculpt | rect do inspector | glifos do quadro |
//! |---|---|---|---|
//! | `inspector` | — | publicado | 269 |
//! | `sculpt3d`  | — | **—** | **218** (só o cromo de base) |
//!
//! Clicar na aba *Sculpt 3D* fazia **a coluna inteira desaparecer**, com a fileira de abas dentro
//! dela — logo não sobrava aba nenhuma para clicar de volta. A cadeia é a do modelo de encaixes:
//! quem está à frente esconde os outros, quem não pinta não publica rect, e o
//! `DockSides::from_published` lê uma coluna sem rects publicados como **livre**.
//!
//! ⇒ a lei: *um painel FECHADO cala-se; um painel ABERTO publica o rect e, se não tem conteúdo,
//! diz porquê* (`panel_chrome::paint_panel_empty`).
//!
//! # ⛔⛔ Por que este gate mora no SHELL e não na `ph2d-panel-registry-init`
//!
//! Aquela crate liga **22** painéis por omissão; o shell liga **26** (o `flip`, o `flip_frames`,
//! o `painter_layers` e o `wet_tuning` só existem aqui). Um censo corrido lá mede uma população
//! que o produto não habita — é a lei do `CLAUDE.md` §2 (*«`-p <crate>` sozinho usa as features
//! POBRES»*) aplicada a um censo. ⇒ **o controlo de população não é um número escrito à mão: é a
//! contagem de `crates/ph2d-panel-*` no disco**, e um painel novo que ninguém ligue reprova aqui.

use ph2d_editor::screens::hero::{HeroScreen, paint_hero_screen};
use ph2d_editor::screens::slot::Slot;
use ph2d_editor::zones::Rect;
use ph2d_text::TextSystem;
use std::collections::BTreeMap;
use std::path::PathBuf;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

struct P {
    id: &'static str,
    node: ph2d_editor::NodeId,
    slot: Slot,
    icon: &'static str,
    /// Quantos comandos de desenho o glifo tem — `0` seria um ícone MUDO.
    strokes: usize,
    floats: bool,
}

fn panels() -> Vec<P> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut v = Vec::new();
    ph2d_editor::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            let m = &p.manifest;
            v.push(P {
                id: m.id,
                node: m.panel_node_id,
                slot: m.default_slot,
                icon: m.icon.slug(),
                strokes: m.icon.cmds().len(),
                floats: m.can_float,
            });
        }
    });
    v
}

/// Quantas crates `ph2d-panel-*` existem no disco — a população REAL, derivada.
///
/// ⚠️ **A `ph2d-panel-registry-init` sai por ser o registo, não um painel.** É a única excepção, e
/// ela é estrutural: um nome novo com esse prefixo que fosse infra teria de vir aqui, e isso é
/// exactamente a conversa que se quer ter.
fn panel_crates_on_disk() -> Vec<String> {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .expect("a pasta das crates existe");
    let mut v: Vec<String> = std::fs::read_dir(crates)
        .expect("li a pasta das crates")
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            (e.path().is_dir() && n.starts_with("ph2d-panel-") && n != "ph2d-panel-registry-init")
                .then_some(n)
        })
        .collect();
    v.sort();
    v
}

/// Pinta quatro quadros com **só** este painel aberto e devolve se ele publicou rect.
///
/// ⚠️ **Quatro quadros porque o `DockSides::from_published` lê o quadro ANTERIOR** — num quadro só
/// nenhuma coluna está reservada e a medição apanharia o estado transitório.
///
/// `width` é a largura da coluna; `None` deixa a de fábrica.
fn publishes_alone_at(p: &P, width: Option<f32>) -> bool {
    let mut h = HeroScreen::new(ph2d_editor::NodeId(1));
    ph2d_editor::panel::with_registry_ref(|reg| {
        for q in reg.panels() {
            h.panel_visibility
                .insert(q.manifest.id, q.manifest.id == p.id);
        }
    });
    if let (Some(w), Some(side)) = (width, p.slot.dock_side()) {
        h.store.set_dock_width(side, w);
    }
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..4 {
        paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    }
    h.store.panel_rect(p.node).is_some()
}

fn publishes_alone(p: &P) -> bool {
    publishes_alone_at(p, None)
}

/// ⛔ **O controlo da POPULAÇÃO, e ele é derivado do disco.**
#[test]
fn the_binary_carries_every_panel_crate_that_exists() {
    let on_disk = panel_crates_on_disk();
    let registered: Vec<String> = panels()
        .iter()
        .map(|p| format!("ph2d-panel-{}", p.id.replace('_', "-")))
        .collect();
    assert!(
        on_disk.len() >= 20,
        "só {} crates de painel no disco — a varredura perdeu o alvo",
        on_disk.len()
    );
    let missing: Vec<&String> = on_disk.iter().filter(|c| !registered.contains(c)).collect();
    assert!(
        missing.is_empty(),
        "estas crates de painel existem e NÃO chegam ao binário do shell: {missing:?}\ncura: a \
         feature no `shells/desktop/Cargo.toml` (a linha `panel-x = [...]` E o `default`). Um \
         painel fora do binário mantém a linha do menu *Window* pintada, clicável e MORTA — foi o \
         report «Widget Lab não abriu» de 2026-09-01."
    );
}

/// ⭐⭐⭐ **Nenhum painel de COLUNA fica mudo estando aberto.**
#[test]
fn no_open_panel_leaves_its_column_blank() {
    let all = panels();
    let mut mute: Vec<&str> = Vec::new();
    let mut measured = 0usize;
    for p in &all {
        // ⚠️ Os que FLUTUAM têm rect próprio e não seguram coluna nenhuma; os que vivem no CENTRO
        //    partem a área de desenho em vez de a ocupar — nenhum dos dois produz este fenómeno.
        if p.floats || p.slot.dock_side().is_none() {
            continue;
        }
        measured += 1;
        if !publishes_alone(p) {
            mute.push(p.id);
        }
    }
    assert!(
        measured >= 12,
        "só {measured} painéis de coluna medidos — a varredura perdeu a população"
    );
    assert!(
        mute.is_empty(),
        "estes painéis ficam MUDOS de coluna aberta: {mute:?}\ncura: publique o `ctx.slot` e pinte \
         a face vazia (`panel_chrome::paint_panel_empty`) dizendo porquê não há nada. Calar-se \
         fecha a coluna e leva consigo a fileira de abas e o painel vizinho."
    );
}

/// ⭐⭐⭐ **CADA PAINEL TEM O GLIFO DELE, E NENHUM PARTILHA O DE OUTRO.**
///
/// Uma aba encolhe até um quadrado de `ROW_H_PX` e o nome sai; o que resta a dizer *de que painel
/// se trata* é o glifo. **Dois painéis com o mesmo desenho são piores que o `…` que o ícone veio
/// curar** — aquele pelo menos não finge dizer alguma coisa.
///
/// ⚠️ A população é TODO painel registado, e não só os que hoje ganham aba: o `CAN_FLOAT` é uma
/// linha que qualquer wave vira, e uma lei cuja população depende de uma declaração mutável cai
/// quando essa declaração muda.
#[test]
fn no_two_panels_share_a_glyph() {
    let all = panels();
    assert_eq!(
        all.len(),
        panel_crates_on_disk().len(),
        "o censo de glifos não está a ver a população inteira — ver \
         `the_binary_carries_every_panel_crate_that_exists`"
    );
    let mut by_slug: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for p in &all {
        by_slug.entry(p.icon).or_default().push(p.id);
    }
    let shared: Vec<String> = by_slug
        .iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(slug, ids)| format!("{slug} \u{00b7} {ids:?}"))
        .collect();
    assert!(
        shared.is_empty(),
        "estes painéis partilham um glifo, e espremidos ficam indistinguíveis:\n  {}\ncura: outro \
         `Panel::ICON` (o catálogo é `docs/design/icons/*.svg`) ou um SVG novo.",
        shared.join("\n  ")
    );
}

/// ⚠️ **E o glifo tem de EXISTIR**: o `IconId` indexa tabelas geradas do SVG por discriminante, e
/// um variante fora da ordem alfabética devolveria os comandos de outro desenho — ou nenhum.
#[test]
fn every_declared_glyph_draws_something() {
    let mute: Vec<&str> = panels()
        .iter()
        .filter(|p| p.strokes == 0)
        .map(|p| p.id)
        .collect();
    assert!(
        mute.is_empty(),
        "estes glifos de aba não desenham nada: {mute:?}"
    );
}

/// ⭐⭐⭐ **ESTREITAR A COLUNA NÃO CALA NINGUÉM** — a outra metade do report do dono.
///
/// > *«inspector buga de vez em quando: apaga ao ser estreitado sem que o painel seja
/// > recolhido»* — Enio, 2026-09-08.
///
/// A causa provável daquele report era o **takeover** (oito bridges escondiam o Inspector para
/// lhe tomar o encaixe), retirado na w41; esta lei fecha a *outra* rota pela qual o mesmo ecrã
/// aparece — um painel que desista de pintar por a coluna estar apertada some **e leva a coluna**,
/// porque uma coluna sem rects publicados lê-se livre.
///
/// ⚠️ **A varredura vai até ao DEGRAU DO FECHO** (`DOCK_W_COLLAPSE`), que é o piso do gesto: abaixo
/// dele a coluna fecha de propósito, e ali o silêncio é a resposta certa. As larguras nomeiam os
/// dois números da lei — o mínimo do painel e o degrau — e as vizinhas deles.
#[test]
fn narrowing_a_column_never_mutes_the_panel_in_it() {
    let floor = ph2d_editor::interaction::WidgetStore::DOCK_W_COLLAPSE;
    let min = ph2d_editor::interaction::WidgetStore::DOCK_W_MIN;
    let widths = [340.0, 280.0, min + 1.0, min, min - 1.0, floor + 1.0, floor];
    let mut mute: Vec<String> = Vec::new();
    let mut measured = 0usize;
    for p in panels() {
        if p.floats || p.slot.dock_side().is_none() {
            continue;
        }
        for w in widths {
            measured += 1;
            if !publishes_alone_at(&p, Some(w)) {
                mute.push(format!("{} a {w} px", p.id));
            }
        }
    }
    assert!(
        measured >= 80,
        "só {measured} células medidas — a varredura perdeu a população ou as larguras"
    );
    assert!(
        mute.is_empty(),
        "estes painéis calam-se ao estreitar a coluna, e um painel calado FECHA a coluna:\n  {}",
        mute.join("\n  ")
    );
}
