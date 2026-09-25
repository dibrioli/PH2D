//! SONDA — **quais são os comandos que o painel Vector pinta, e onde.**
//!
//! Ordem do dono (2026-09-24): *«Arrumar o painel Vector»* — o censo `quantas_entradas_tem_cada_painel`
//! mede-o em `45` comandos contra `18` valores, e a triagem da `D2` (cada comando vai para o sítio do
//! âmbito dele) pede a LISTA, não o número. O nome de cada id sai das PRÓPRIAS declarações do
//! `ph2d-panel-vector/src/ids/` (literais `hash_node_id("…")` e formatos `…{}…` expandidos), porque o
//! id é um hash e não diz nada sozinho.

use std::collections::BTreeMap;

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_ui_testkit::MockPanelHost;

/// A largura do dono (a coluna docada fotografada em 2026-09-19).
const VISTA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 300.0,
    h: 4000.0,
};

fn junta(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            junta(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// `hash → nome`, colhido de todo literal com forma de id nas crates do vector e do núcleo.
fn nomes() -> BTreeMap<u64, String> {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut fontes = Vec::new();
    for c in [
        "ph2d-panel-vector/src",
        "ph2d-tool-vector/src",
        "ph2d-editor-core/src",
    ] {
        junta(&raiz.join(c), &mut fontes);
    }
    let mut out = BTreeMap::new();
    for f in fontes {
        let src = std::fs::read_to_string(f).expect("legível");
        for cap in src.split('"').skip(1).step_by(2) {
            let parece_id = cap.contains('.')
                && cap.len() < 80
                && cap
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._{}".contains(c));
            if !parece_id {
                continue;
            }
            if cap.contains('{') {
                let k = cap.find('{').expect("abre");
                let f = cap[k..].find('}').expect("fecha") + k;
                for i in 0..96 {
                    let n = format!("{}{i}{}", &cap[..k], &cap[f + 1..]);
                    out.insert(ph2d_tool_registry::hash_node_id_runtime(&n).0, n);
                }
            } else {
                out.insert(
                    ph2d_tool_registry::hash_node_id_runtime(cap).0,
                    cap.to_string(),
                );
            }
        }
    }
    out
}

#[cfg(feature = "panel-vector")]
#[test]
#[ignore]
fn diag_os_comandos_do_vector() {
    let nomes = nomes();
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "vector")
            .expect("o painel vector tem de estar no registo");
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        super::quantas_entradas_tem_cada_painel::abre_tudo(host.store_mut());
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VISTA);
        });
        let em_grupo: std::collections::BTreeSet<u64> =
            grupos.iter().flatten().map(|id| id.0).collect();
        let pintados = host.registos_da_ultima_pintura();
        let store = host.store();
        println!("\n  espécie   y      x      w      id");
        for (id, r) in &pintados {
            let esp = match store.get(*id) {
                Some(InteractiveState::Button { .. }) if em_grupo.contains(&id.0) => "grupo",
                Some(InteractiveState::Button { .. }) => "COMANDO",
                Some(InteractiveState::Plain) | None => continue,
                Some(_) => "valor",
            };
            let n = nomes
                .get(&id.0)
                .cloned()
                .unwrap_or_else(|| format!("(desconhecido {:x})", id.0));
            println!("  {:8} {:6.0} {:6.0} {:6.0}  {}", esp, r.y, r.x, r.w, n);
        }
    });
}
