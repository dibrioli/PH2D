//! A Bancada no quadro da família (plano 13 Onda B): segue a cena para a aba activa, e o que os
//! cliques pediram vira a edição do Inspector (pelo bus) ou a selecção.

use crate::docs::CodeDocs;
use ph2d_code_bancada::{Open, act};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::bancada::Intent;
use ph2d_script::ScriptHost;

/// Um quadro: as edições a empurrar no bus e o objecto a seleccionar (`active` = a aba activa).
pub(crate) fn frame(
    docs: &mut CodeDocs,
    world: &mut ph2d_ecs::World,
    host: Option<&ScriptHost>,
    active: Option<u64>,
    now: u128,
) -> (Vec<EditorAction>, Option<u64>) {
    docs.bench.tick(now);
    let open = active
        .and_then(|id| docs.docs.iter().find(|d| d.script.0 == id))
        .map(|d| Open {
            script: d.script,
            syntax: &d.syntax,
            text: d.buffer.text(),
        });
    let script = open.as_ref().map(|o| o.script);
    let want = (docs.bench.map_on, docs.bench.peeking());
    docs.bancada.follow(world, host, open, want);
    let decls = script.and_then(|s| ph2d_script::decls_shown(world, host, s).map(<[_]>::to_vec));
    let (mut edits, mut pick) = (Vec::new(), None);
    for i in docs.bench.take_intents() {
        match &i {
            Intent::Jump(line) => {
                if let Some(id) = active {
                    docs.go_to(id, *line, 0);
                }
            }
            Intent::Select(h) => pick = docs.bancada.who.holders.get(*h).map(|h| h.bits),
            Intent::Edit(..) => edits.extend(act(&docs.bancada.who, decls.as_deref(), &i)),
        }
    }
    (edits, pick)
}

impl CodeDocs {
    /// O objecto do «quem · quando» do Console sob o rato: o índice na Bancada e a caixa do nome.
    pub(crate) fn console_holder(&self, id: u64, x: f32, y: f32) -> Option<(usize, Rect)> {
        let d = self.docs.iter().find(|d| d.script.0 == id)?;
        let (row, r) = d.console_view.tag_at(x, y)?;
        let bits = self.consoles.get(&d.script)?.who_of(row)?;
        let h = self
            .bancada
            .who
            .holders
            .iter()
            .position(|h| h.bits == bits)?;
        Some((h, r))
    }
}
