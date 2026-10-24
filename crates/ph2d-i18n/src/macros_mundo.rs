//! **As etapas da família «mundo»** (W19, `docs/Macros/17_W19_plano.md` §5): o nome de cada etapa no
//! painel MACROS e na linha «Macro steps», e porque uma etapa do mundo não se apaga nem troca de lugar.

/// A tradução de uma chave `panel.macros.step.world.*` / `panel.macros.error.world_*`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.macros.step.world.added" => "{names} added",
        "panel.macros.step.world.changed" => "{names} changed",
        "panel.macros.step.world.removed" => "{names} removed",
        "panel.macros.step.world.parent" => "moved to another parent",
        "panel.macros.step.world.on" => "{what} \u{b7} {object}",
        "panel.macros.step.world.appears" => "{object} appears",
        "panel.macros.step.world.deleted" => "{object} is deleted",
        "panel.macros.step.world.more" => "{what} (+{n} more)",
        "panel.macros.step.world.unnamed" => "object",
        "panel.macros.step.world.simulate" => "Simulate \u{b7} {seconds} s",
        "panel.macros.error.world_step_is_fixed" => {
            "This step changes objects the next steps build on \u{2014} it cannot be deleted or replaced."
        }
        "panel.macros.error.world_step_keeps_its_order" => {
            "This step cannot pass another one: the steps after it build on what it does."
        }
        _ => return None,
    })
}
