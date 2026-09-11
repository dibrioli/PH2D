//! ⭐⭐ **OS CENSOS DO CARTÃO** — as sondas `#[ignore]` que IMPRIMEM o estado do catálogo, irmãs
//! por RESPONSABILIDADE (HR-18) dos gates em [`super::card_params_tests`].
//!
//! ⚠️ **Um censo não é um gate**, e é por isso que eles vivem noutro ficheiro: um gate reprova
//! quando o mundo se afasta de uma lei; um censo devolve o NÚMERO com que se escolhe a lei
//! seguinte (*quantos params por cartão · o que o painel ainda oferece a mais · quantas rows são
//! de cada espécie · o que o grupo do ciclo aberto precisa*). Misturá-los faz uma corrida normal
//! carregar o custo dos censos e um censo parecer dívida.

use super::card_params_tests::open_every_section;
use super::*;
use crate::motion::motion_state::MotionState;

/// **O CENSO: quantos params cada tipo de nó põe no cartão** — a sonda que responde *«o cartão
/// está vazio porque ninguém o encheu, ou porque ninguém o pintou?»* sem arqueologia de pixels.
///
/// `cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture what_each_card_carries`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_each_card_carries() {
    let m = MotionState::new();
    let mut linhas: Vec<(usize, String)> = Vec::new();
    let mut sem_hints = 0usize;
    for man in m.registry.manifests() {
        let mut aux = MotionState::new();
        let id = aux.doc.graph.add_node(man.name.to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&aux.doc.graph, &aux.registry);
        stamp_card_params(&aux, ph2d_editor::ProjectSettings::default(), &mut snap);
        let n = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or(0, |v| v.params.len());
        if aux.registry.param_ui(man.id).is_none() {
            sem_hints += 1;
        }
        linhas.push((n, man.name.to_string()));
    }
    linhas.sort_by_key(|a| std::cmp::Reverse(a.0));
    let total: usize = linhas.iter().map(|(n, _)| n).sum();
    let vazios = linhas.iter().filter(|(n, _)| *n == 0).count();
    eprintln!(
        "  {} tipos · {total} rows no total · {vazios} cartoes VAZIOS · {sem_hints} sem hints",
        linhas.len()
    );
    eprintln!("  --- os 12 mais cheios ---");
    for (n, name) in linhas.iter().take(12) {
        eprintln!("  {n:>3} │ {name}");
    }
    // ⭐ E o que a DOBRA por omissão faria à altura — a pergunta que decide se o cartão de 30
    // rows do L-System é um problema de desenho ou de folding.
    eprintln!("  --- se o cartao dobrasse as seccoes como o painel dobra ---");
    eprintln!("  {:>4} │ {:>4} │ {:>4} │ nó", "rows", "abre", "secs");
    for alvo in [
        "source.lsystem",
        "motion.bezier_warp",
        "motion.emitter",
        "motion.noise",
        "motion.grid",
    ] {
        let mut aux = MotionState::new();
        let id = aux.doc.graph.add_node(alvo.to_string());
        let tid = aux.doc.graph.node(id).expect("no'").type_id();
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&aux.doc.graph, &aux.registry);
        stamp_card_params(&aux, ph2d_editor::ProjectSettings::default(), &mut snap);
        let ps = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map(|v| v.params.clone())
            .unwrap_or_default();
        let dobradas = aux.registry.param_groups_folded(tid);
        let mut secs: Vec<&str> = Vec::new();
        let mut abertas = 0usize;
        for c in &ps {
            let g = aux.registry.param_group(tid, c.hint.param);
            match g {
                Some(g) => {
                    if !secs.contains(&g) {
                        secs.push(g);
                    }
                    if !dobradas.contains(&g) {
                        abertas += 1;
                    }
                }
                // Sem grupo = sempre visível (o topo do painel).
                None => abertas += 1,
            }
        }
        eprintln!(
            "  {:>4} │ {abertas:>4} │ {:>4} │ {alvo}",
            ps.len(),
            secs.len()
        );
    }
    eprintln!("  --- os do smoke ---");
    for alvo in [
        "source.lsystem",
        "motion.move",
        "motion.output",
        "motion.grid",
    ] {
        let n = linhas
            .iter()
            .find(|(_, s)| s == alvo)
            .map_or(usize::MAX, |(n, _)| *n);
        eprintln!("  {n:>3} │ {alvo}");
    }
}

/// ⭐⭐⭐ **O CARTÃO OFERECE TUDO O QUE O PAINEL OFERECE** — o critério de aceitação para o
/// painel lateral SAIR (doc 103: decisão do Enio, 2026-09-05).
///
/// ⚠️ Não basta a porta de VISIBILIDADE ser a mesma (`shown_params`): o painel monta as rows
/// por outro caminho (`build_params_snapshot`), que dobra canais de cor em amostras, acrescenta
/// rows do canal de TEXTO (curva, gradiente, ficheiro) e pode consumir params num controlo só.
/// Duas contagens diferentes sobre a mesma lei é como um knob fica **inalcançável** no dia em
/// que o painel for embora.
///
/// A régua é de INCLUSÃO, não de igualdade: o cartão pode oferecer mais (e oferece — ele não
/// dobra nada ainda), nunca menos.
///
/// `cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture what_the_panel_offers_and_the_card_does_not`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_the_panel_offers_and_the_card_does_not() {
    let mut faltam: Vec<(String, usize, usize, Vec<String>)> = Vec::new();
    let mut tot_painel = 0usize;
    let mut tot_cartao = 0usize;
    let base = MotionState::new();
    let tipos: Vec<String> = base
        .registry
        .manifests()
        .map(|m| m.name.to_string())
        .collect();
    drop(base);
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let painel = build_params_snapshot(&m, ph2d_editor::ProjectSettings::default());
        let no_painel: Vec<String> = painel
            .as_ref()
            .map(|s| {
                s.rows
                    .iter()
                    .flat_map(|r| {
                        r.params()
                            .iter()
                            .map(|p| (*p).to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor::ProjectSettings::default(), &mut snap);
        let no_cartao: Vec<String> = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map(|v| v.params.iter().map(|c| c.hint.param.to_string()).collect())
            .unwrap_or_default();
        tot_painel += no_painel.len();
        tot_cartao += no_cartao.len();
        let em_falta: Vec<String> = no_painel
            .iter()
            .filter(|p| !no_cartao.contains(p))
            .cloned()
            .collect();
        if !em_falta.is_empty() {
            faltam.push((nome.clone(), no_painel.len(), no_cartao.len(), em_falta));
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    eprintln!(
        "  {} tipos · painel {tot_painel} params · cartao {tot_cartao} · {} tipos com FALTA",
        tipos.len(),
        faltam.len()
    );
    faltam.sort_by_key(|(_, _, _, f)| std::cmp::Reverse(f.len()));
    for (n, p, c, f) in faltam.iter().take(14) {
        eprintln!(
            "  {n:<26} painel {p:>3} · cartao {c:>3} · faltam {:>2}: {f:?}",
            f.len()
        );
    }
}

/// **O CENSO DAS ESPÉCIES DE CONTROLO** — onde o esforço dos editores ricos tem de ir.
///
/// `cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture what_species_of_control_the_catalogue_has`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_species_of_control_the_catalogue_has() {
    use std::collections::BTreeMap;
    let mut por_especie: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    let base = MotionState::new();
    let tipos: Vec<String> = base
        .registry
        .manifests()
        .map(|m| m.name.to_string())
        .collect();
    drop(base);
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        open_every_section(&mut m, id);
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor::ProjectSettings::default(), &mut snap);
        let Some(v) = snap.nodes.iter().find(|v| v.id == id.0) else {
            continue;
        };
        let mut vistos: Vec<&'static str> = Vec::new();
        for c in &v.params {
            let e = match c.hint.widget {
                ph2d_node_registry::ParamWidget::Slider => "Slider",
                ph2d_node_registry::ParamWidget::IntSlider => "IntSlider",
                ph2d_node_registry::ParamWidget::Angle => "Angle",
                ph2d_node_registry::ParamWidget::Toggle => "Toggle",
                ph2d_node_registry::ParamWidget::Seed => "Seed",
                ph2d_node_registry::ParamWidget::Color { .. } => "Color",
                ph2d_node_registry::ParamWidget::Enum { .. } => "Enum",
                ph2d_node_registry::ParamWidget::Channels { .. } => "Channels",
                ph2d_node_registry::ParamWidget::Source => "Source",
                ph2d_node_registry::ParamWidget::PickSelection => "PickSelection",
                ph2d_node_registry::ParamWidget::Text => "Text",
                ph2d_node_registry::ParamWidget::Curve => "Curve",
                ph2d_node_registry::ParamWidget::Gradient => "Gradient",
                ph2d_node_registry::ParamWidget::Palette => "Palette",
                ph2d_node_registry::ParamWidget::File { .. } => "File",
            };
            let slot = por_especie.entry(e).or_default();
            slot.0 += 1;
            if !vistos.contains(&e) {
                vistos.push(e);
                slot.1 += 1;
            }
        }
    }
    let total: usize = por_especie.values().map(|(n, _)| n).sum();
    eprintln!("  {total} rows de cartao, por especie de controlo:");
    let mut linhas: Vec<_> = por_especie.iter().collect();
    linhas.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
    for (e, (n, nos)) in linhas {
        eprintln!(
            "  {n:>4} ({:>4.1}%) │ em {nos:>3} nos │ {e}",
            *n as f64 * 100.0 / total as f64
        );
    }
}

/// **O GRUPO DO CICLO ABERTO, controlo a controlo** — a pergunta que decide se os editores
/// ricos bloqueiam o ciclo 1 (doc 104) ou pertencem a um ciclo mais à frente.
///
/// `cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture what_the_open_cycle_group_needs`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_the_open_cycle_group_needs() {
    const GRUPO: [&str; 10] = [
        "motion.grid",
        "motion.scatter",
        "motion.distribute_radial",
        "motion.fibonacci",
        "motion.lattice",
        "motion.voronoi",
        "motion.distribute_poisson",
        "motion.distribute_curve",
        "motion.path",
        "motion.clone",
    ];
    let ricos = [
        "Color", "Text", "Curve", "Gradient", "Palette", "File", "Source", "Channels",
    ];
    let mut total_ricos = 0usize;
    eprintln!(
        "  {:>4} │ {:>4} │ {:>5} │ nó · espécies",
        "rows", "secs", "ricos"
    );
    for nome in GRUPO {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        open_every_section(&mut m, id);
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor::ProjectSettings::default(), &mut snap);
        let Some(v) = snap.nodes.iter().find(|v| v.id == id.0) else {
            continue;
        };
        let mut especies: Vec<&'static str> = Vec::new();
        let mut n_ricos = 0usize;
        for c in &v.params {
            let e = match c.hint.widget {
                ph2d_node_registry::ParamWidget::Slider => "Slider",
                ph2d_node_registry::ParamWidget::IntSlider => "IntSlider",
                ph2d_node_registry::ParamWidget::Angle => "Angle",
                ph2d_node_registry::ParamWidget::Toggle => "Toggle",
                ph2d_node_registry::ParamWidget::Seed => "Seed",
                ph2d_node_registry::ParamWidget::Color { .. } => "Color",
                ph2d_node_registry::ParamWidget::Enum { .. } => "Enum",
                ph2d_node_registry::ParamWidget::Channels { .. } => "Channels",
                ph2d_node_registry::ParamWidget::Source => "Source",
                ph2d_node_registry::ParamWidget::PickSelection => "PickSelection",
                ph2d_node_registry::ParamWidget::Text => "Text",
                ph2d_node_registry::ParamWidget::Curve => "Curve",
                ph2d_node_registry::ParamWidget::Gradient => "Gradient",
                ph2d_node_registry::ParamWidget::Palette => "Palette",
                ph2d_node_registry::ParamWidget::File { .. } => "File",
            };
            if ricos.contains(&e) {
                n_ricos += 1;
            }
            if !especies.contains(&e) {
                especies.push(e);
            }
        }
        total_ricos += n_ricos;
        eprintln!(
            "  {:>4} │ {:>4} │ {n_ricos:>5} │ {nome} · {especies:?}",
            v.params.len(),
            v.sections.len()
        );
    }
    eprintln!("  -- controlos RICOS no grupo do ciclo 1: {total_ricos}");
}
