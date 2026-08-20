//! ⚠️ **A fila de Image Tools é PINTADA a partir do registry — e era DESPACHADA por listas
//! escritas à mão.** Foi assim que o pill `[SHEET]` nasceu morto (Enio, 2026-08-19: *«botão sheet
//! não funciona»*): o `ph2d-tool-sheet-packer` entrou pela porta do drop-crate (ADR-0075), o
//! manifesto foi lido pelo painter, o pill apareceu na barra — e o clique não fazia **nada**. Sem
//! erro, sem toast, sem log. Duas listas centrais não sabiam do tool novo:
//!
//! 1. **`topbar::populate()`** — a lista de `store.register(id, InteractiveState::Button)`. Sem
//!    registro, `is_focusable() == false`: o Down nunca arma o `active`, o Up nunca emite `Click`.
//!    *O pill está morto debaixo do rato antes de qualquer despacho existir.*
//! 2. **`chrome::image_actions::oneshot_tool_for`** — um `if`/`else` contra quatro consts. O id
//!    novo caía no `else` final e a função devolvia `None`.
//!
//! Cada uma sozinha basta para o botão não funcionar, e **as duas falham em silêncio**. É a mesma
//! doença que este repositório já pagou três vezes — os quatro pills de vetor (`0661862`), o
//! Undo/Redo da barra esquerda, e o *Use as Brush Shape* que shipou pintado e morto. O gate
//! `architecture_topbar_registration_parity` é a memória institucional da primeira, mas ele varre
//! `ids::TOPBAR_*`: a fila de Image Tools **não tem const `TOPBAR_`**, porque é derivada do
//! registry — o painter cresceu e o gate ficou.
//!
//! Este gate fecha o buraco pelo lado do COMPORTAMENTO, e por isso não pode ser enganado por
//! nenhuma das duas listas: ele instala o registry **inteiro** (o mesmo `register_all` do boot),
//! popula a barra como o shell popula, e **clica em cada pill do cluster** perguntando duas coisas
//! que só o produto responde — *o clique foi consumido?* e *chegou ao barramento a ação com o id
//! DESTE manifesto?*
//!
//! ⚠️ **A fonte é o registry, nunca uma lista aqui dentro.** Um tool novo entra neste gate no dia
//! em que se regista, sem ninguém se lembrar — que é exatamente o que faltou. *Um gate que precisa
//! de ser atualizado para cobrir o caso novo não teria apanhado este.*

use ph2d_editor::action_bus::EditorAction;
use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::screens::hero::HeroScreen;
use ph2d_tool_registry::{Registry, hash_node_id};

/// Instala o registry do boot — `register_all`, o mesmo que
/// `shells/<plat>/src/init.rs` chama — e devolve-o.
///
/// ⚠️ **`install_registry` é `OnceLock`**: uma instalação por processo. Cada ficheiro de teste é
/// um binário próprio, então este `install` é o único deste binário.
fn install_boot_registry() -> &'static Registry {
    let mut reg = Registry::default();
    ph2d_tool_registry_init::register_all(&mut reg);
    reg.build().expect("o registry do boot tem de construir");
    ph2d_editor::install_registry(reg);
    ph2d_editor::installed_registry().expect("acabou de ser instalado")
}

/// Cada pill da fila `image_tools` **despacha o seu próprio manifesto**.
///
/// A asserção é dupla de propósito, porque as duas metades falham em sítios diferentes:
///
/// - `consumed == true` prova que a barra **reconhece** o id (o clique não caiu no chão);
/// - a ação no barramento prova que ele chega ao **tool certo** — comparada contra `m.id`, não
///   contra uma tabela, para que um id trocado entre dois tools também reprove.
///
/// Um tool `OneShot` levanta `OneShotImageOp` (uma por sprite selecionada — daí a seleção
/// sintética); um `Stateful` levanta `ActivateTool`. Qualquer das duas serve: o que este gate
/// recusa é **nenhuma**.
#[test]
fn every_image_tool_pill_dispatches_its_own_manifest() {
    ph2d_editor::test_support::ensure_panel_registry();
    let reg = install_boot_registry();

    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    // Uma seleção sintética: sem ela um `OneShot` consome o clique e não empurra nada (a
    // difusão é por sprite selecionada), e o gate leria "morto" um pill que está vivo.
    hero.gizmo.selection = Some(0xDEAD_BEEF);

    let mut dead: Vec<String> = Vec::new();
    for m in reg.cluster("image_tools") {
        let id = hash_node_id(m.id);
        let consumed = hero.apply_event(WidgetEvent::Click(id));
        let drained: Vec<EditorAction> = hero.bus.drain().collect();
        let routed = drained.iter().any(|a| match a {
            EditorAction::OneShotImageOp { tool_id, .. } => *tool_id == m.id,
            EditorAction::ActivateTool { tool_id } => *tool_id == m.id,
            _ => false,
        });
        if !consumed || !routed {
            dead.push(format!(
                "  - {} (pill {:#018x}): consumido={consumed}, no barramento={drained:?}",
                m.id, id.0
            ));
        }
    }

    assert!(
        dead.is_empty(),
        "pills da fila `image_tools` que o painter DESENHA e o despacho não conhece \
         — clicar neles não faz nada, e nada no ecrã o diz:\n{}\n\n\
         Cure na fonte, não com mais um `else if`: o despacho de\n\
         `screens/hero/chrome/image_actions.rs` lê o registry (filtra o cluster por\n\
         `ToolHandler::OneShot` / `::Stateful` e casa `hash_node_id(m.id)`), e o\n\
         registro de `screens/hero/topbar/mod.rs::populate()` percorre\n\
         `image_action_pills()` — a MESMA porta que o painter usa.",
        dead.join("\n")
    );
}

/// O pill tem de estar **registado no store**, senão morre debaixo do rato antes de haver
/// despacho nenhum.
///
/// ⚠️ Este é o irmão do `architecture_topbar_registration_parity` para a fila derivada do
/// registry — aquele varre `ids::TOPBAR_*` no texto do ficheiro, e um pill de Image Tools **não
/// tem const `TOPBAR_`**. Este pergunta ao store, que é onde a resposta vive.
#[test]
fn every_image_tool_pill_is_registered_and_therefore_focusable() {
    ph2d_editor::test_support::ensure_panel_registry();
    let reg = install_boot_registry();

    let hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    let unregistered: Vec<&str> = reg
        .cluster("image_tools")
        .iter()
        .filter(|m| !hero.store.contains(hash_node_id(m.id)))
        .map(|m| m.id)
        .collect();

    assert!(
        unregistered.is_empty(),
        "pills de `image_tools` pintados mas SEM `InteractiveState` em `topbar::populate()` \
         → `is_focusable() == false` → o Up nunca emite `Click`: {unregistered:?}.\n\
         O loop de registo tem de percorrer `image_action_pills()` (a mesma porta do painter), \
         não uma lista escrita à mão."
    );
}
