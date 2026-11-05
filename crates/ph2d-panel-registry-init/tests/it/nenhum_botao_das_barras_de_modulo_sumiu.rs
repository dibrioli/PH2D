//! O inventário das barras dos módulos contra o `main` (report do dono, 04/11): a coluna e a fila do
//! Vector em cada ferramenta, e a coluna do Pixel com cada grupo aberto. A fixtura foi gravada no `main`
//! por este mesmo teste (`PH2D_GRAVA_INVENTARIO=<ficheiro>`). Um botão que saia reprova; um novo não
//! precisa de editar nada. A fila da imagem e a do pintor moram em `ph2d-tool-registry-init`.

use ph2d_editor_core::interaction::ToolBars;
use ph2d_editor_core::pixel_panel::{PixelGroups, PixelPanelInfo, PixelTool, PixelToolGroup};
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_panel_pixel_tools::column;
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};
use std::collections::BTreeSet;

const FIXTURA: &str = include_str!("../fixtures/inventario_barras_de_modulo.tsv");

fn linhas(fam: &str, bars: &ToolBars, out: &mut Vec<String>) {
    let mut com_nome: Vec<(u64, String)> = bars
        .row_lead
        .iter()
        .chain(&bars.row_tail)
        .filter_map(|e: &ToolRailEntry| {
            e.node_id()
                .map(|i| (i.0, e.label().unwrap_or_default().to_owned()))
        })
        .collect();
    let grupos = bars
        .column
        .iter()
        .chain(&bars.actions)
        .chain(&bars.actions_below);
    for b in grupos.flatten() {
        com_nome.push((b.id.0, b.label.clone()));
    }
    for id in bars.painted_ids() {
        if !com_nome.iter().any(|(i, _)| *i == id.0) {
            com_nome.push((id.0, String::new()));
        }
    }
    for (id, nome) in com_nome {
        out.push(format!("{fam}\t{id:#018x}\t{nome}"));
    }
}

fn inventario() -> Vec<String> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = Vec::new();
    let hero = HeroScreen::new(NodeId(1));
    let sel = Rect::new(600.0, 400.0, 120.0, 90.0);
    for &mode in DrawMode::ALL {
        ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
            mode,
            ..VectorStyleSnapshot::default()
        }));
        ph2d_panel_vector::set_current_selection_count(2);
        ph2d_panel_vector::set_current_vertex_count(3);
        let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, Some(sel), Vec::new());
        linhas(&format!("vetor.{mode:?}"), &bars, &mut out);
    }
    let grupos = PixelToolGroup::ALL.into_iter().map(Some).chain([None]);
    for aberto in grupos {
        let mut groups = PixelGroups::default();
        groups.open = aberto;
        let info = PixelPanelInfo {
            editing: true,
            tool: PixelTool::Pencil,
            palette: vec![[1, 2, 3, 255]; 32],
            groups,
            ..Default::default()
        };
        linhas(&format!("pixel.{aberto:?}"), &column::bars(&info), &mut out);
    }
    out.sort();
    out.dedup();
    out
}

/// ⭐⭐⭐ Tudo o que o `main` lista está aqui. Mutação: tirar uma ferramenta da coluna ⇒ vermelho.
#[test]
fn todo_botao_do_main_esta_nas_barras_de_modulo() {
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
    assert!(esperadas.len() > 100, "controlo: a fixtura está vazia");
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
