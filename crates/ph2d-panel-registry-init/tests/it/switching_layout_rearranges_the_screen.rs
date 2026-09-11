//! ⭐⭐⭐ **TROCAR DE LAYOUT ARRUMA A TELA** — a decisão **D7**, medida sobre os painéis reais.
//!
//! > *«Layout | o **utilizador** | barra de cima (abas) | que **áreas** existem e que editor está em
//! > cada»* — `00_DECISOES_DO_ENIO.md`, D3
//!
//! # ⛔ A lista de abertos é ABSOLUTA, e é isso que este gate defende
//!
//! Um layout que só *acrescentasse* painéis acumularia o que a tarefa anterior deixou: escolher
//! *Nodes* depois de *Draw* daria o grafo **mais** as camadas do pintor. *Um layout é o estado da
//! tela, não um passo sobre ele.*

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{HeroScreen, layout_switch, layout_tabs};
use ph2d_editor_core::screens::task_layout::TaskLayout;

fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    HeroScreen::new(ph2d_editor_core::NodeId(1))
}

fn registered(id: &str) -> bool {
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().any(|p| p.manifest.id == id)
    })
}

/// ⭐⭐ **Cada layout abre EXACTAMENTE o que declara** — e fecha tudo o resto.
#[test]
fn every_layout_opens_exactly_what_it_declares() {
    let mut h = hero();
    let mut checked = 0usize;
    for l in TaskLayout::ALL {
        layout_switch::apply(&mut h, l);
        let spec = l.spec();
        ph2d_editor_core::panel::with_registry_ref(|reg| {
            for p in reg.panels() {
                let should = spec.open.contains(&p.manifest.id);
                assert_eq!(
                    h.is_panel_visible(p.manifest.id),
                    should,
                    "{l:?}: `{}` devia estar {} e está {}",
                    p.manifest.id,
                    if should { "aberto" } else { "fechado" },
                    if h.is_panel_visible(p.manifest.id) {
                        "aberto"
                    } else {
                        "fechado"
                    }
                );
            }
        });
        checked += 1;
    }
    assert_eq!(checked, TaskLayout::ALL.len());
}

/// ⛔⛔ **Uma tarefa não herda o desarrumo da anterior.**
///
/// ⚠️ **A régua mudou de painel em 2026-08-31, e a razão é o report do Enio:** ela media o
/// `vector`, que **pertence à ponte da ferramenta** e não ao layout — o que o tornava um sujeito
/// errado para uma lei sobre a lista de abertos (nesta suíte não há pontes, então ele media o
/// puro). O `timeline` é do layout, e por isso é o que esta lei pode afirmar.
#[test]
fn a_layout_never_inherits_what_the_previous_one_left_open() {
    let mut h = hero();
    layout_switch::apply(&mut h, TaskLayout::Animation);
    assert!(
        h.is_panel_visible("timeline"),
        "controlo: o layout *Animate* não abre a linha do tempo e o gate mediria o vazio"
    );
    layout_switch::apply(&mut h, TaskLayout::Modeling3d);
    assert!(
        !h.is_panel_visible("timeline"),
        "a linha do tempo sobreviveu à troca para o *Model* — um layout virou um passo sobre o \
         anterior em vez do estado da tela"
    );
    assert!(h.is_panel_visible("model3d"));
}

/// ⚠️ **A tabela nomeia painéis que EXISTEM.** Um id com erro de escrita não falha nada — ele
/// simplesmente nunca abre, e a aba fica com um painel a menos, em silêncio.
///
/// ⛔⛔ **O oráculo é a PASTA da crate, e não o registry, e a diferença é uma medição:** esta build
/// de teste corre com as features de omissão da `ph2d-panel-registry-init`, e a do **app** liga mais
/// três (`painter_layers`, `flip`, `flip_frames`). Perguntar ao registry acusaria três ids
/// **correctos** de não existir. *Uma ausência por feature e um erro de escrita leem-se iguais num
/// registry; só a árvore os separa.*
#[test]
fn every_panel_a_layout_names_is_a_crate_that_exists() {
    let mut missing = Vec::new();
    let mut named = 0usize;
    let exists = |id: &str| {
        std::path::Path::new(&format!("../ph2d-panel-{}", id.replace('_', "-"))).is_dir()
    };
    for l in TaskLayout::ALL {
        for id in l.spec().open {
            named += 1;
            if !exists(id) {
                missing.push(format!("{l:?} abre `{id}`, que não é uma crate de painel"));
            }
        }
        for (id, _) in l.spec().slots {
            if !exists(id) {
                missing.push(format!("{l:?} encaixa `{id}`, que não existe"));
            }
        }
    }
    // ⚠️ O piso desceu de 12 para 8 em 2026-08-31, e **não é afrouxar**: a tabela deixou de
    // nomear os painéis que pertencem às pontes das ferramentas (ver o gate
    // `a_layout_never_commands_a_panel_a_bridge_owns` no shell). O que ele ainda mede é que ela
    // não se esvaziou — são 9 nomes hoje, um por cada coisa que o layout de facto comanda.
    assert!(
        named >= 8,
        "só {named} painéis nomeados em toda a tabela — ela esvaziou-se"
    );
    assert!(
        exists("inspector") && !exists("um_painel_de_2030"),
        "controlo: o oráculo da pasta deixou de distinguir o que existe do que não existe"
    );
    assert!(missing.is_empty(), "{}", missing.join("\n  "));
}

/// ⭐ **E o que está REGISTADO nesta build abre de facto.**
///
/// ⚠️ O irmão de cima mede a tabela; este mede o produto **naquilo que esta build tem**. Os dois são
/// precisos: um id certo numa build sem a feature não é um defeito, e um id errado é.
#[test]
fn every_named_panel_that_this_build_registers_actually_opens() {
    let mut h = hero();
    let mut opened = 0usize;
    for l in TaskLayout::ALL {
        layout_switch::apply(&mut h, l);
        for id in l.spec().open {
            if registered(id) {
                assert!(
                    h.is_panel_visible(id),
                    "{l:?} nomeia `{id}` e ele não abriu"
                );
                opened += 1;
            }
        }
    }
    // ⚠️ Mesmo motivo do piso irmão: a tabela encolheu de propósito em 2026-08-31.
    assert!(
        opened >= 7,
        "só {opened} aberturas medidas — as features desta build encolheram e o gate mede pouco"
    );
}

/// ⭐⭐ **E a ABA chega lá** — o verbo, não só a lei.
#[test]
fn clicking_a_layout_tab_rearranges_the_screen() {
    let mut h = hero();
    layout_switch::apply(&mut h, TaskLayout::Drawing2d);

    let consumed = h.apply_event(WidgetEvent::Click(layout_tabs::tab_node_id(
        TaskLayout::Animation,
    )));
    assert!(
        consumed,
        "o clique na aba *Animate* não foi consumido — a aba é muda"
    );
    assert_eq!(h.store.active_layout(), TaskLayout::Animation);
    assert!(
        h.is_panel_visible("timeline"),
        "a aba foi clicada e a linha do tempo não abriu"
    );
    // ⚠️ E o que a tarefa anterior tinha aberto fechou. (O `painter_layers` não serve de régua
    // aqui desde 31/08: ele é da ponte do pintor, não da lista de abertos.)
    assert!(!h.is_panel_visible("model3d"));
}

/// ⭐⭐⭐ **TODO layout entrega o canvas, e NENHUM o herda** — a régua do report do Enio de
/// 2026-08-31: *«se abro Nodes e depois Model, o grafo de Nodes persiste»*.
///
/// ⛔⛔ **O gate que aqui estava afirmava o defeito.** Ele chamava-se
/// `…_and_one_that_does_not_leaves_the_hand_alone` e media a decisão (*«o Animate não declara
/// ferramenta, logo não pede nenhuma»*) em vez da consequência (*«e por isso a tela dele fica com
/// os painéis da tarefa anterior»*) — e ficou **verde durante o report inteiro**. *Um gate escrito
/// a partir da intenção do código pina o que o código faz, não o que ele deve.*
#[test]
fn every_layout_hands_the_canvas_over_and_none_inherits_it() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::screens::task_layout::CanvasOwner;
    let mut h = hero();

    let mut asked_by = Vec::new();
    for l in TaskLayout::ALL {
        let _ = h.bus.drain().count();
        layout_switch::apply(&mut h, l);
        let asked: Vec<&'static str> = h
            .bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::ActivateTool { tool_id } => Some(tool_id),
                _ => None,
            })
            .collect();
        match l.spec().canvas {
            CanvasOwner::Tool(id) => assert_eq!(
                asked,
                vec![id],
                "{l:?} declara a ferramenta `{id}` e não a pediu — a tela arruma-se e o canvas \
                 fica no modo anterior, com os painéis dele atrás"
            ),
            // ⛔ O modelador não é uma `Tool`: quem larga a que está em mãos é a lei do
            // `field3d_mode`, no shell, acordada pelo painel que a lista de abertos abriu.
            CanvasOwner::Model3d => assert!(
                asked.is_empty(),
                "{l:?} entrega o canvas ao modelador e mesmo assim pediu uma ferramenta \
                 ({asked:?}) — a ponte leria isso como *«outro tomou o canvas»* e fecharia o \
                 painel que a abriu"
            ),
        }
        asked_by.push(asked);
    }
    assert_eq!(asked_by.len(), TaskLayout::ALL.len());
    assert!(
        asked_by.iter().filter(|a| !a.is_empty()).count() >= 5,
        "só {} layouts pediram ferramenta — voltou a haver herança",
        asked_by.iter().filter(|a| !a.is_empty()).count()
    );
}

/// ⛔⛔ **Trocar de layout devolve os ENCAIXES ao que os painéis declaram.**
///
/// ⚠️ Este gate nasceu de uma **mutação sobrevivente**: apagar o `reset_panel_slots` deixava a suíte
/// verde, porque nada movia um painel antes de trocar de tarefa. *As excepções de encaixe pertencem
/// à arrumação de quem as fez* — vê-las noutra tarefa é herdar o desarrumo pela porta do lado.
#[test]
fn switching_layout_returns_every_panel_to_the_slot_it_declares() {
    use ph2d_editor_core::screens::slot::Slot;
    let mut h = hero();
    layout_switch::apply(&mut h, TaskLayout::Vector);

    let node = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.id == "inspector")
            .map(|p| p.manifest.panel_node_id)
            .expect("o inspector existe")
    });
    h.store.set_panel_slot(node, Slot::LeftTop);
    assert_eq!(
        h.store.panel_slot(node),
        Some(Slot::LeftTop),
        "controlo: a excepção não ficou posta e o gate mediria o vazio"
    );

    layout_switch::apply(&mut h, TaskLayout::Animation);
    assert_eq!(
        h.store.panel_slot(node),
        None,
        "a excepção de encaixe do *Vector* sobreviveu à troca para o *Animate*"
    );
}
