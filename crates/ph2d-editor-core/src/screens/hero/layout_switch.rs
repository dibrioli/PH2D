//! ⭐⭐⭐ **TROCAR DE LAYOUT** — o verbo da decisão **D7**, e o que ele toca.
//!
//! Escolher uma aba arruma a tela para a tarefa: os painéis daquele layout abrem, **todos os
//! outros fecham**, as excepções de encaixe voltam ao que os painéis declaram, e a ferramenta
//! opcional é pegada.
//!
//! # ⛔ Por que a lista de abertos é ABSOLUTA e não um diff
//!
//! Um layout que só *acrescentasse* painéis acumularia o que a tarefa anterior deixou: escolher
//! *Nodes* depois de *Draw* daria o grafo **mais** as camadas do pintor, e a segunda tarefa
//! herdaria o desarrumo da primeira. *Um layout é o estado da tela, não um passo sobre ele.*
//!
//! ⚠️ **E é por isso que trocar de layout também limpa as excepções de encaixe**: elas pertencem à
//! arrumação de quem as fez. Quem as quer de volta volta à aba onde as fez — a persistência guarda
//! uma arrumação **por layout** (`layout_persist`).
//!
//! # ⚠️ O que ele NÃO toca, de propósito
//!
//! A largura das colunas. Ela é a **medida da mão** de quem usa o ecrã, não da tarefa: um artista
//! que alarga a coluna porque o monitor é estreito não quer que ela encolha ao mudar de tarefa.
//! ⇒ ela viaja com o layout no ficheiro (para quem a quiser diferente por tarefa) mas **não é
//! reposta** pela troca.

use super::HeroScreen;
use crate::action_bus::EditorAction;
use crate::screens::task_layout::{CanvasOwner, TaskLayout};

/// Arruma a tela para `layout`. Ver o cabeçalho do módulo.
pub fn apply(hero: &mut HeroScreen, layout: TaskLayout) {
    let spec = layout.spec();
    hero.store.set_active_layout(layout);

    // ⚠️ **Fecha tudo e abre a lista** — nesta ordem, e num passo só sobre o registry: um painel que
    // esteja nas duas metades acabaria fechado se a ordem fosse a outra.
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            let open = spec.open.contains(&p.manifest.id);
            hero.panel_visibility.insert(p.manifest.id, open);
        }
    });

    // As excepções de encaixe da tarefa anterior não são desta.
    hero.store.reset_panel_slots();
    crate::panel::with_registry_opt(|reg| {
        for (id, slot) in spec.slots {
            let Some(p) = reg.panels().iter().find(|p| p.manifest.id == *id) else {
                continue;
            };
            // ⛔ A mesma cerca da leitura do ficheiro: o `ALLOWED_SLOTS` é do produto, e nem a
            // tabela dos layouts o pode contrariar.
            if p.manifest.allowed_slots.contains(*slot) {
                hero.store.set_panel_slot(p.manifest.panel_node_id, *slot);
            }
        }
    });

    // ⭐⭐ **O canvas muda de dono (D3), e não há caso de «não mexe»** — ver `CanvasOwner`. Um
    // layout que não largasse a ferramenta traria os painéis dela atrás, porque quem os abre é a
    // ponte da ferramenta e não esta função; foi o report de 2026-08-31.
    match spec.canvas {
        CanvasOwner::Tool(tool_id) => hero.bus.push(EditorAction::ActivateTool { tool_id }),
        // ⛔ Nada a pedir: quem larga a ferramenta é a lei do `field3d_mode` no shell, acordada
        // pelo painel que a lista de abertos acabou de abrir. Ver `CanvasOwner::Model3d`.
        CanvasOwner::Model3d => {}
    }
}

// ⚠️ **Os gates deste módulo NÃO vivem aqui.** Ele mede-se pelo que acontece aos painéis, e nesta
// crate o `test_support::ensure_panel_registry` é um `{}` — uma varredura sobre zero painéis
// passaria sobre nada. Eles vivem em
// `ph2d-panel-registry-init/tests/switching_layout_rearranges_the_screen.rs`.
