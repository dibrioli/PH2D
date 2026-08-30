//! ⭐⭐⭐ **AS DUAS TECLAS QUE A HIERARQUIA NÃO TINHA** — `Delete` e `Ctrl/Cmd+D`.
//!
//! # O report
//!
//! Enio, 2026-08-30: *«temos um bug: delete não funciona na hierarquia. Avalie também duplicate»*.
//!
//! # ⛔ A causa: elas nunca existiram, e TRÊS doc-comments do shell diziam que sim
//!
//! `keyboard.rs` e `keyboard_painter.rs` afirmam, em três sítios, que a cadeia específica *«corta
//! ANTES do hero, cujo caminho genérico de Delete apaga a ENTIDADE selecionada»*. **Esse caminho
//! genérico não existe:** o `KEY_DELETE` do dispatcher vira `GraphKey::Delete`, e o único consumidor
//! dele em toda a árvore é o painel do grafo de motion. Apagar um objeto da cena só era possível
//! pelo menu de contexto de uma linha da Hierarquia — e duplicar, só por lá também.
//!
//! *Um comentário que descreve um caminho ausente é pior que nenhum: ele faz cada leitor seguinte
//! assumir que a metade que falta já está feita.*
//!
//! # ⭐⭐ Elas são um segundo PRODUTOR do mesmo verbo, nunca uma segunda lei
//!
//! As teclas **não** despacham entidades: elas resolvem `selecção → linha` pela ponte e empurram
//! `EditorAction::HierDelete` / `HierDuplicate` — exactamente o que o item do menu empurra. Toda a
//! lei (o que fazer com a multi-selecção, a limpeza do gizmo, a cópia profunda, o passo da cascata,
//! o undo) fica onde já estava, com um chamador a mais. *Uma segunda porta para o mesmo verbo é
//! como duas respostas começam a divergir.*
//!
//! # ⚠️ Por que o gesto é PONTEIRO-SOBRE-A-HIERARQUIA, e não global
//!
//! `Delete` tem donos a mais neste app — o traço do Flip, o nó de uma curva, a figura em mãos do
//! Painter, a key da timeline, o nó do grafo. Uma rota global roubaria a tecla de todos eles. A
//! regra de área é a que o `cursor_over_timeline` já usa para a mesma pergunta, é a do Blender, e é
//! literalmente o que o report descreve: *«na hierarquia»*.

use super::App;
use ph2d_editor::action_bus;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl App {
    /// O ponteiro está sobre o corpo do painel da Hierarquia?
    pub(crate) fn cursor_over_hierarchy(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.panel_rect(ph2d_editor::ids::HIER_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }

    /// A linha da Hierarquia que corresponde ao objeto **primário** da selecção.
    ///
    /// ⚠️ `None` quando não há selecção **ou** quando a ponte ainda não viu aquela entidade — e as
    /// duas respondem a mesma coisa a quem chama: *não há a quem aplicar o verbo*.
    fn selected_hierarchy_row(&self) -> Option<ph2d_editor::NodeId> {
        let gfx = self.gfx.as_ref()?;
        let bits = gfx.hero_screen.as_ref()?.gizmo.selection?;
        gfx.hero_live.as_ref()?.bridge.node_for(bits)
    }

    /// Empurra uma acção de linha para o barramento do editor — a MESMA que o menu empurra.
    fn push_hier_action(
        &mut self,
        make: fn(ph2d_editor::NodeId) -> action_bus::EditorAction,
    ) -> bool {
        let Some(row) = self.selected_hierarchy_row() else {
            return false;
        };
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        hero.bus.push(make(row));
        true
    }

    /// **A cadeia da Hierarquia** — `Delete` apaga a selecção, `Ctrl/Cmd+D` duplica-a.
    ///
    /// Devolve `true` (consumindo a tecla) só quando de facto empurrou o verbo.
    ///
    /// ⚠️ **Aqui só se JUNTAM OS FATOS**; quem decide é a [`verb_for`], que é pura e por isso
    /// alcançável de um teste. *Um `impl App` pede janela, superfície e device — uma lei escrita
    /// dentro dele só se pode gatear por texto, e um gate textual não distingue a chamada viva da
    /// chamada atrás de um `if false`* (medido: a mutação sobreviveu ao gate da costura).
    pub(crate) fn hierarchy_key_chain(
        &mut self,
        state: ElementState,
        repeat: bool,
        physical_key: PhysicalKey,
    ) -> bool {
        let facts = KeyFacts {
            pressed: state == ElementState::Pressed,
            repeat,
            over_panel: self.cursor_over_hierarchy(),
            text_focused: self.text_entry_focused(),
            cmd: self.modifiers.super_key() || self.modifiers.control_key(),
            key: match physical_key {
                PhysicalKey::Code(c) => Some(c),
                PhysicalKey::Unidentified(_) => None,
            },
        };
        match verb_for(facts) {
            None => false,
            Some(HierKeyVerb::Delete) => {
                self.push_hier_action(|row| action_bus::EditorAction::HierDelete { row })
            }
            Some(HierKeyVerb::Duplicate) => {
                self.push_hier_action(|row| action_bus::EditorAction::HierDuplicate { row })
            }
        }
    }
}

/// O verbo que uma tecla pede sobre a linha selecionada.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HierKeyVerb {
    Delete,
    Duplicate,
}

/// Tudo o que a decisão precisa de saber — e nada do `App`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct KeyFacts {
    pub pressed: bool,
    pub repeat: bool,
    pub over_panel: bool,
    pub text_focused: bool,
    pub cmd: bool,
    pub key: Option<KeyCode>,
}

/// ⭐⭐ **A LEI, pura** — que tecla, sob que condições, pede que verbo.
///
/// ⚠️ **Um campo de texto focado fica com as teclas**: o rename de uma linha vive dentro deste mesmo
/// painel, e sem essa guarda apagar uma letra do nome apagaria o objeto.
///
/// ⚠️ **`Delete` exige que NENHUM modificador esteja em jogo** — `Ctrl+Delete` é «apagar a palavra»
/// em todo campo de texto do mundo, e reivindicá-lo aqui seria roubar um gesto que não é nosso.
pub(crate) fn verb_for(f: KeyFacts) -> Option<HierKeyVerb> {
    if !f.pressed || f.repeat || !f.over_panel || f.text_focused {
        return None;
    }
    match f.key? {
        KeyCode::Delete | KeyCode::Backspace if !f.cmd => Some(HierKeyVerb::Delete),
        KeyCode::KeyD if f.cmd => Some(HierKeyVerb::Duplicate),
        _ => None,
    }
}

#[cfg(test)]
#[path = "keyboard_hierarchy_tests.rs"]
mod tests;
