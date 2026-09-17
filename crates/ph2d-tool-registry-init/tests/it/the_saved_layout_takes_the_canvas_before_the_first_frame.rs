//! ⭐⭐⭐ **O LAYOUT GRAVADO PEGA A FERRAMENTA DELE ANTES DO PRIMEIRO QUADRO** — e não deixa pedido
//! nenhum no barramento que troque, a meio desse quadro, a ferramenta que uma cena escolheu.
//!
//! # O defeito (medido 2026-09-16, com a foto da tela virtual)
//!
//! O arranque instalava o layout gravado (`~/.ph2d/layout.txt`) pelo mesmo verbo do clique numa
//! aba, e esse verbo **pede** a ferramenta pelo barramento. O barramento só é drenado a meio do
//! 1.º quadro, depois do prólogo onde as cenas de smoke escolhem a delas:
//!
//! - com `Nodes` gravado, a cena dos ossos pegava o vetor e o dreno trocava-o pela ferramenta de
//!   nós — **nenhum osso desenhado**, e o grafo a partir a área de desenho;
//! - com `Vector` gravado, o pedido chegava com o vetor JÁ activo, e o pill **alterna**: a
//!   ferramenta era largada, sem aviso.
//!
//! ⚠️ **O dono corre as cenas na máquina dele, com o layout que deixou gravado** — e a foto com um
//! `HOME` limpo mostrava os ossos. *Uma cena fotografada só na arrumação de fábrica foi medida num
//! programa que o dono não corre.*
//!
//! ⚠️ **Mora aqui** porque só esta crate tem as duas metades do arranque: os manifestos (que dizem
//! o cluster de cada ferramenta) e as ferramentas de verdade.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{HeroScreen, layout_switch};
use ph2d_editor_core::screens::task_layout::{CanvasOwner, TaskLayout};
use ph2d_editor_core::tool_activation::activation_gate;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_registry::Registry;

/// O registo de manifestos do boot. ⚠️ `install_registry` é um `OnceLock` do processo: se outro
/// módulo deste binário já o instalou, o conteúdo é o mesmo (o mesmo `register_all`).
fn instala_os_manifestos() {
    let mut reg = Registry::default();
    ph2d_tool_registry_init::register_all(&mut reg);
    reg.build().expect("o registry do boot tem de construir");
    let _ = ph2d_editor_core::install_registry(reg);
}

/// As ferramentas como o boot as deixa: registadas e com a de omissão activa.
fn ferramentas() -> ToolRegistry {
    let mut t = ToolRegistry::new();
    ph2d_tool_registry_init::register_all_tools(&mut t);
    t.activate_default();
    t
}

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(ph2d_a11y::NodeId(1))
}

fn activa(t: &ToolRegistry) -> Option<ToolId> {
    t.active().map(ph2d_editor_core::Tool::id)
}

fn pedidos_de_ferramenta(h: &HeroScreen) -> Vec<&'static str> {
    h.bus
        .iter()
        .filter_map(|a| match a {
            EditorAction::ActivateTool { tool_id } => Some(*tool_id),
            _ => None,
        })
        .collect()
}

/// ⚠️ **O dreno do quadro, com a lei dele** — o `fase_image_tool_activation` da shell, que não é
/// alcançável daqui. A lei é a MESMA porta ([`activation_gate`]); só a alternância é repetida.
fn drena_como_o_quadro(h: &mut HeroScreen, t: &mut ToolRegistry) {
    let pedidos: Vec<EditorAction> = h.bus.drain().collect();
    for a in pedidos {
        if let EditorAction::ActivateTool { tool_id } = a {
            let lei = activation_gate(tool_id, h.image_edit.mode_on);
            let ja_activa = activa(t) == Some(ToolId::new(tool_id));
            if lei.gate_on && ja_activa && lei.toggles_off {
                t.activate_default();
            } else if lei.gate_on {
                t.set_active(&ToolId::new(tool_id));
            }
        }
    }
}

/// ⭐ **Cada layout pega a ferramenta dele NO ARRANQUE, e não deixa pedido para trás.**
#[test]
fn every_saved_layout_takes_its_tool_at_startup_and_leaves_no_request_behind() {
    instala_os_manifestos();
    let mut pegaram = Vec::new();
    for l in TaskLayout::ALL {
        let (mut h, mut t) = (hero(), ferramentas());
        layout_switch::install_at_startup(&mut h, &mut t, l);
        assert_eq!(
            pedidos_de_ferramenta(&h),
            Vec::<&str>::new(),
            "{l:?}: o arranque deixou um pedido no barramento — ele chega a meio do 1.º quadro, \
             depois da cena, e troca a ferramenta dela"
        );
        if let CanvasOwner::Tool(id) = l.spec().canvas
            && activation_gate(id, h.image_edit.mode_on).gate_on
        {
            assert_eq!(
                activa(&t),
                Some(ToolId::new(id)),
                "{l:?}: o layout gravado não pegou a ferramenta dele no arranque"
            );
            pegaram.push(id);
        }
    }
    // ⛔ Controlo de população: uma lei que respondesse «não» a tudo passava as duas metades acima.
    for id in ["vector", "flip", "motion"] {
        assert!(
            pegaram.contains(&id),
            "controlo: o layout de `{id}` devia pegar a ferramenta no arranque (pegaram {pegaram:?})"
        );
    }
}

/// ⭐⭐ **A cena que escolhe uma ferramenta no 1.º quadro FICA com ela, qualquer que seja o layout
/// gravado** — o cenário do defeito, com o dreno do quadro a correr depois da cena.
#[test]
fn a_scene_that_picks_a_tool_keeps_it_under_every_saved_layout() {
    instala_os_manifestos();
    let vetor = ToolId::new("vector");
    for l in TaskLayout::ALL {
        let (mut h, mut t) = (hero(), ferramentas());
        layout_switch::install_at_startup(&mut h, &mut t, l);
        // O prólogo do 1.º quadro: a cena escolhe.
        assert!(t.set_active(&vetor), "o vetor tem de estar registado");
        // A meio do quadro: o dreno.
        drena_como_o_quadro(&mut h, &mut t);
        assert_eq!(
            activa(&t),
            Some(vetor.clone()),
            "{l:?}: a ferramenta da cena foi trocada pelo que o arranque deixou pendente"
        );
    }
}

/// ⛔ **O clique numa aba continua a pedir pelo barramento** — é a metade que o arranque NÃO usa,
/// e ela tem de continuar viva (o hero não alcança o registo de ferramentas).
#[test]
fn clicking_a_layout_tab_still_requests_its_tool() {
    for l in TaskLayout::ALL {
        let mut h = hero();
        layout_switch::apply(&mut h, l);
        let esperado: Vec<&str> = match l.spec().canvas {
            CanvasOwner::Tool(id) => vec![id],
            CanvasOwner::Model3d => vec![],
        };
        assert_eq!(pedidos_de_ferramenta(&h), esperado, "{l:?}");
    }
}
