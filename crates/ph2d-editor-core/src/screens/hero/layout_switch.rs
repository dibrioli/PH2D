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
use crate::ToolId;
use crate::action_bus::EditorAction;
use crate::screens::task_layout::{CanvasOwner, TaskLayout};
use crate::tool::ToolRegistry;
use crate::tool_activation::activation_gate;

/// Arruma a tela para `layout` — **o clique numa aba**. Ver o cabeçalho do módulo.
///
/// ⭐⭐ **O canvas muda de dono (D3), e não há caso de «não mexe»** — ver `CanvasOwner`. Um layout
/// que não largasse a ferramenta traria os painéis dela atrás, porque quem os abre é a ponte da
/// ferramenta e não esta função; foi o report de 2026-08-31. O pedido vai pelo barramento porque o
/// hero não alcança o registo de ferramentas; ⛔ o `Model3d` não pede nada — quem larga a
/// ferramenta é a lei do `field3d_mode` no shell, acordada pelo painel que a lista de abertos acabou
/// de abrir.
pub fn apply(hero: &mut HeroScreen, layout: TaskLayout) {
    if let CanvasOwner::Tool(tool_id) = arrange(hero, layout) {
        hero.bus.push(EditorAction::ActivateTool { tool_id });
    }
}

/// ⭐⭐⭐ **O ARRANQUE** — a mesma arrumação, com o dono do canvas pegado **agora**, antes do primeiro
/// quadro.
///
/// ⛔⛔ **Pelo barramento o pedido chegava DEPOIS da cena de smoke** (medido 2026-09-16, com a foto):
/// o barramento só é drenado a meio do 1.º quadro, e o prólogo desse quadro — onde uma cena escolhe
/// a ferramenta dela — corre antes. Com o layout `Nodes` gravado, a cena dos ossos pegava o vetor e
/// o dreno trocava-o pela ferramenta de nós: **nenhum osso desenhado**. E com o layout `Vector`
/// gravado era pior e mais mudo: o pedido do layout chegava com o vetor JÁ activo, e o pill
/// **alterna** — a ferramenta era largada. Vale para toda cena que escolhe uma ferramenta.
///
/// ⚠️ **A mesma lei do dreno** ([`crate::tool_activation::activation_gate`]), sem a alternância: no
/// arranque ninguém clicou, e pedir a ferramenta que já está activa não é um gesto de a largar.
pub fn install_at_startup(hero: &mut HeroScreen, tools: &mut ToolRegistry, layout: TaskLayout) {
    if let CanvasOwner::Tool(tool_id) = arrange(hero, layout)
        && activation_gate(tool_id, hero.image_edit.mode_on).gate_on
    {
        tools.set_active(&ToolId::new(tool_id));
    }
}

/// Tudo o que trocar de layout faz **menos** pegar a ferramenta; devolve quem deve ser o dono do
/// canvas, para cada chamador o pegar pela porta que tem.
fn arrange(hero: &mut HeroScreen, layout: TaskLayout) -> CanvasOwner {
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

    // ⭐⭐ **E a memória de um fecho de COLUNA morre com a tarefa que a produziu.**
    //
    // ⚠️ Ela guarda *«estes painéis estavam nesta coluna quando o dedo a fechou»*, e a linha acima
    // acabou de reescrever a visibilidade dos 26 pela lista desta tarefa. Sobreviver seria a
    // reabertura da coluna trazer de volta o Motion Params dentro do modo de desenho — um conjunto
    // que já não descreve nada. Ver [`super::dock_columns`], que cai no `fallback` quando não há
    // memória, e o `fallback` lê exactamente a `LayoutSpec::open` que acabou de ser aplicada.
    hero.dock_closed = [None, None];

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

    spec.canvas
}

// ⚠️ **Os gates deste módulo NÃO vivem aqui.** Ele mede-se pelo que acontece aos painéis, e nesta
// crate o `test_support::ensure_panel_registry` é um `{}` — uma varredura sobre zero painéis
// passaria sobre nada. Eles vivem em
// `ph2d-panel-registry-init/tests/it/switching_layout_rearranges_the_screen.rs`.
