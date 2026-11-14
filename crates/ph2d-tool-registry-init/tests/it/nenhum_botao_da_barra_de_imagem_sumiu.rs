//! O inventário da fila do topo contra o `main` (report do dono, 04/11). A fixtura foi gravada no
//! `main` por este mesmo teste (`PH2D_GRAVA_INVENTARIO=<ficheiro>`): cada linha é uma família e um id.
//! Um botão que saia da fila reprova; um botão novo não precisa de editar nada. Tirar uma linha da
//! fixtura é a decisão consciente de retirar o botão.

use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen, painter_column, tool_bar};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use std::collections::BTreeSet;

const FIXTURA: &str = include_str!("../fixtures/inventario_barra_de_imagem.tsv");

fn linhas(fam: &str, bars: impl IntoIterator<Item = (u64, String)>, out: &mut Vec<String>) {
    for (id, nome) in bars {
        out.push(format!("{fam}\t{id:#018x}\t{nome}"));
    }
}

fn da_fila(rail: &ph2d_editor_core::widget::ToolRail) -> Vec<(u64, String)> {
    rail.entries
        .iter()
        .filter_map(|e| {
            e.node_id()
                .map(|i| (i.0, e.label().unwrap_or_default().to_owned()))
        })
        .collect()
}

fn inventario() -> Vec<String> {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut reg = ph2d_tool_registry::Registry::default();
    ph2d_tool_registry_init::register_all(&mut reg);
    reg.build().expect("o registry do boot tem de construir");
    ph2d_editor_core::install_registry(reg);
    let mut ts = TextSystem::without_system_fonts();
    let store = HeroScreen::new(ph2d_a11y::NodeId(1)).store;
    let mut out = Vec::new();
    // A fila de repouso é a do Object de toda família, a do Flip e a do esqueleto.
    let repouso = tool_bar::bar_rail(&store, &mut ts, false, false);
    linhas("repouso", da_fila(&repouso), &mut out);
    let edit = tool_bar::bar_rail(&store, &mut ts, false, true);
    linhas("imagem.edit", da_fila(&edit), &mut out);
    let pintor = painter_column::painter_tool_bars(&store);
    let ids = pintor.painted_ids();
    let nomes = pintor.row_lead.iter().chain(&pintor.row_tail);
    let mut com_nome: Vec<(u64, String)> = nomes
        .filter_map(|e| {
            e.node_id()
                .map(|i| (i.0, e.label().unwrap_or_default().to_owned()))
        })
        .collect();
    for b in pintor.column.iter().flatten() {
        com_nome.push((b.id.0, b.label.clone()));
    }
    for id in ids {
        if !com_nome.iter().any(|(i, _)| *i == id.0) {
            com_nome.push((id.0, String::new()));
        }
    }
    linhas("imagem.draw", com_nome, &mut out);
    out.sort();
    out.dedup();
    out
}

/// ⭐⭐⭐ Tudo o que o `main` lista está aqui. Mutação: tirar uma ferramenta da fila ⇒ vermelho.
#[test]
fn todo_botao_do_main_esta_na_fila() {
    let agora = inventario();
    if let Ok(caminho) = std::env::var("PH2D_GRAVA_INVENTARIO") {
        std::fs::write(caminho, agora.join("\n") + "\n").expect("gravar o inventário");
        return;
    }
    let ids: BTreeSet<(&str, &str)> = agora
        .iter()
        .filter_map(|l| {
            let mut c = l.split('\t');
            Some((c.next()?, c.next()?))
        })
        .collect();
    let esperadas: Vec<&str> = FIXTURA
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();
    assert!(esperadas.len() > 30, "controlo: a fixtura está vazia");
    let sumiram: Vec<&str> = esperadas
        .iter()
        .copied()
        .filter(|l| {
            let mut c = l.split('\t');
            let chave = (c.next().unwrap_or(""), c.next().unwrap_or(""));
            !ids.contains(&chave)
        })
        .collect();
    assert!(
        sumiram.is_empty(),
        "botões do main que sumiram: {sumiram:#?}"
    );
}

/// Toda ferramenta em Edit cabe na fila à largura de fábrica, e em toda janela menor está na fila ou
/// atrás do `⋯`, nunca em lado nenhum.
#[test]
fn nada_some_em_janela_menor() {
    let mut ts = TextSystem::without_system_fonts();
    let _ = inventario();
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, 1930.0, 1040.0);
    paint_hero_screen(&mut hero, vp, &mut scene, &mut ts);
    let fabrica = hero.last_layout.expect("houve um quadro").tool_bar.w;
    let (_, over) = tool_bar::bar_split(&hero.store, &mut ts, false, true, fabrica);
    assert!(
        over.is_empty(),
        "a {fabrica} px a fila de fábrica deixa botões atrás do `⋯`"
    );
    let todos: Vec<u64> = tool_bar::bar_rail(&hero.store, &mut ts, false, true)
        .entries
        .iter()
        .filter_map(|e| e.node_id().map(|i| i.0))
        .collect();
    for area in [1300.0, 1000.0, 746.0, 640.0, 480.0, 320.0_f32] {
        let (fila, over) = tool_bar::bar_split(&hero.store, &mut ts, false, true, area);
        let vistos: BTreeSet<u64> = fila
            .entries
            .iter()
            .chain(&over)
            .filter_map(|e| e.node_id().map(|i| i.0))
            .collect();
        for id in &todos {
            assert!(vistos.contains(id), "a {area} px um botão sumiu: {id:#x}");
        }
        // A régua da barra: a da fila fina quando ela é fina (BarraDeMenus onda 6), a do trilho largo senão.
        let linhas = tool_bar::bar_lines(&fila, area, hero.store.rail_button_size());
        assert!(
            linhas <= 1,
            "a {area} px a fila parte: {linhas} linhas, o resto cortado"
        );
    }
}
