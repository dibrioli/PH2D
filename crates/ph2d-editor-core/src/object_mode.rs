//! ⭐⭐ **O MODO DO OBJECTO ACTIVO** — spec/06 §3.2 (F2), como o Blender: o modo é do objecto, o
//! TIPO declara quais tem, e num modo de criação a selecção fica trancada nele
//! (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`).
//!
//! - **O vocabulário fecha-se com o que funciona hoje.** Um modo entra em [`ObjectMode`] no dia em
//!   que o módulo dele abre sobre a entidade (D6: modo que não faz nada é controlo morto).
//! - **O estado guarda a ENTIDADE, nunca o tipo** ([`ActiveMode`]). O tipo pergunta-se ao marcador
//!   (`ph2d_app_components::component_attach::kind_of`), e quem compõe as famílias publica aqui os
//!   modos do activo em todo quadro ([`ModeState::publish`]). O quadro que os corre é o
//!   `screens::hero::mode_drive` (este módulo não conhece o Hero: a fundação é um DAG).
//! - **As leis são puras** ([`ModeState::resolve`], [`ModeState::still_holds`], [`decide`]); quem
//!   abre e larga o módulo é a família dona, com os recursos dela.

use crate::interaction::{AreaMenu, InteractiveState, WidgetStore};
use crate::widget::{ButtonState, ToolRailEntry};
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use std::collections::BTreeMap;

/// **Um modo de edição.** `Object` é universal; os outros, só o tipo que os declara.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectMode {
    /// Mover, rodar, escalar — o gizmo de transformação.
    Object,
    /// Pintar com o Painter: a imagem (Image ▸ Paint) ou a peça esculpida (Sculpt ▸ Paint, D6).
    Paint,
    /// Esculpir a peça com o barro na tela (Sculpt ▸ Sculpt, D6).
    Sculpt,
}

impl ObjectMode {
    /// Todos, em ordem — a fonte da iteração (⛔ nunca escreva a lista uma segunda vez).
    pub const ALL: [ObjectMode; 3] = [ObjectMode::Object, ObjectMode::Paint, ObjectMode::Sculpt];

    /// O nome que o artista lê.
    #[must_use]
    pub const fn label_key(self) -> TextKey {
        TextKey::new(match self {
            ObjectMode::Object => "object_mode.object",
            ObjectMode::Paint => "object_mode.paint",
            ObjectMode::Sculpt => "object_mode.sculpt",
        })
    }

    /// O id da linha deste modo no seletor.
    #[must_use]
    pub const fn row_id(self) -> NodeId {
        match self {
            ObjectMode::Object => crate::ids::OBJECT_MODE_OBJECT,
            ObjectMode::Paint => crate::ids::OBJECT_MODE_PAINT,
            ObjectMode::Sculpt => crate::ids::OBJECT_MODE_SCULPT,
        }
    }

    /// O inverso de [`Self::row_id`].
    #[must_use]
    pub fn of_row(id: NodeId) -> Option<ObjectMode> {
        Self::ALL.into_iter().find(|m| m.row_id() == id)
    }
}

/// **O modo em curso** — o `ModoActivo` do spec. ⛔ Sem o tipo: ver o cabeçalho.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ActiveMode {
    /// Os bits da entidade em edição.
    pub entity: u64,
    /// Nunca [`ObjectMode::Object`]: Object é a ausência de modo de criação.
    pub mode: ObjectMode,
}

/// **Um pedido de modo**, pelo barramento.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModeRequest {
    /// Uma linha do seletor: entrar neste modo (`Object` = sair).
    Enter(ObjectMode),
    /// `Tab`: Object ↔ o último modo de criação deste objecto.
    Toggle,
    /// Uma aba de cima que pede um modo ao abrir (§3.3): entra se o activo o tiver, senão fica em
    /// Object com a ferramenta de omissão — ⛔ nunca cria um objecto (§6.5).
    Open(ObjectMode),
}

/// **O que fazer** com um pedido — a resposta de [`ModeState::resolve`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Nada muda (já está lá).
    Stay,
    /// Entrar neste modo sobre o activo (largando primeiro o modo em curso, se houver).
    Enter(ObjectMode),
    /// Voltar a Object.
    Leave,
    /// Voltar a Object E devolver o canvas à ferramenta de omissão (o [`ModeRequest::Open`] sem
    /// activo compatível).
    LeaveToDefault,
    /// O activo não tem o modo pedido (ou não há activo): responde com a razão.
    Refuse,
}

/// **O modo, ao lado da selecção que ele tranca** (vive em `GizmoStateGroup::mode`).
#[derive(Clone, Debug, Default)]
pub struct ModeState {
    active: Option<ActiveMode>,
    /// O activo e os modos do tipo dele, publicados pela composição em todo quadro. `None` = nada
    /// seleccionado.
    published: Option<(u64, Vec<ObjectMode>)>,
    /// O último modo de criação de cada objecto — o destino do `Tab`.
    last: BTreeMap<u64, ObjectMode>,
}

impl ModeState {
    /// O modo em curso, se há um de criação.
    #[must_use]
    pub fn active(&self) -> Option<ActiveMode> {
        self.active
    }

    /// O modo que o seletor mostra.
    #[must_use]
    pub fn current(&self) -> ObjectMode {
        self.active.map_or(ObjectMode::Object, |a| a.mode)
    }

    /// ⭐ **A entidade trancada** — a que um modo de criação tem em mãos. `None` em Object.
    #[must_use]
    pub fn locked_entity(&self) -> Option<u64> {
        self.active.map(|a| a.entity)
    }

    /// Os modos do activo publicados neste quadro (vazio = nada seleccionado).
    #[must_use]
    pub fn available(&self) -> &[ObjectMode] {
        self.published.as_ref().map_or(&[], |(_, m)| m.as_slice())
    }

    /// ⭐ **A composição publica o activo e os modos do tipo dele** — `Object` à frente, os da
    /// família a seguir, sem repetidos.
    pub fn publish(&mut self, active: Option<u64>, family_modes: &[ObjectMode]) {
        self.published = active.map(|e| {
            let mut modes = vec![ObjectMode::Object];
            for m in family_modes {
                if !modes.contains(m) {
                    modes.push(*m);
                }
            }
            (e, modes)
        });
    }

    /// Entra em `mode` sobre `entity` e lembra-o para o `Tab`.
    pub fn enter(&mut self, entity: u64, mode: ObjectMode) {
        debug_assert_ne!(mode, ObjectMode::Object, "Object não é um modo de criação");
        self.active = Some(ActiveMode { entity, mode });
        self.last.insert(entity, mode);
    }

    /// Volta a Object; devolve o modo que estava.
    pub fn leave(&mut self) -> Option<ActiveMode> {
        self.active.take()
    }

    /// ⭐⭐ **A LEI do pedido.** Pura: o mesmo estado e o mesmo pedido dão sempre o mesmo passo.
    #[must_use]
    pub fn resolve(&self, req: ModeRequest) -> Step {
        let available = self.available();
        let has = |m: ObjectMode| m != ObjectMode::Object && available.contains(&m);
        match req {
            ModeRequest::Enter(ObjectMode::Object) => {
                if self.active.is_some() {
                    Step::Leave
                } else {
                    Step::Stay
                }
            }
            ModeRequest::Enter(m) if self.current() == m => Step::Stay,
            ModeRequest::Enter(m) if has(m) => Step::Enter(m),
            ModeRequest::Enter(_) => Step::Refuse,
            ModeRequest::Toggle if self.active.is_some() => Step::Leave,
            ModeRequest::Toggle => {
                let remembered = self
                    .published
                    .as_ref()
                    .and_then(|(e, _)| self.last.get(e).copied())
                    .filter(|m| has(*m));
                match remembered.or_else(|| available.iter().copied().find(|m| has(*m))) {
                    Some(m) => Step::Enter(m),
                    None => Step::Refuse,
                }
            }
            ModeRequest::Open(m) if self.current() == m && self.active.is_some() => Step::Stay,
            ModeRequest::Open(m) if has(m) => Step::Enter(m),
            ModeRequest::Open(_) => Step::LeaveToDefault,
        }
    }

    /// ⭐⭐ **O modo em curso ainda se segura?** — a rede de segurança de cada quadro.
    ///
    /// Ele só vale enquanto a selecção é EXACTAMENTE a entidade dele e o módulo a tem em mãos. A
    /// selecção muda por dezenas de portas que não são gestos (criar, duplicar, apagar, desfazer,
    /// largar um ficheiro) — em vez de as ensinar uma a uma, quem perde a entidade volta a Object.
    #[must_use]
    pub fn still_holds(&self, selection: Option<u64>, extras: usize, module_holds: bool) -> bool {
        self.active
            .is_none_or(|a| selection == Some(a.entity) && extras == 0 && module_holds)
    }

    /// ⭐ **O seletor** — o 1.º pulldown da fila; `None` sem objecto activo. Escreve o
    /// `ButtonState` de cada linha (a acesa é a do modo em curso).
    #[must_use]
    pub fn menu(&self, store: &mut WidgetStore) -> Option<AreaMenu> {
        let modes = self.published.as_ref().map(|(_, m)| m.clone())?;
        let current = self.current();
        let rows = modes
            .iter()
            .map(|m| {
                let label = m.label_key().tr();
                if let Some(InteractiveState::Button { state }) = store.get_mut(m.row_id()) {
                    *state = if *m == current {
                        ButtonState::Pressed
                    } else {
                        ButtonState::Normal
                    };
                }
                ToolRailEntry::compound(m.row_id(), label, label, "")
            })
            .collect();
        Some(AreaMenu {
            label: ph2d_i18n::tr("object_mode.menu").to_string(),
            face: current.label_key().tr().to_string(),
            faces: modes
                .iter()
                .map(|m| m.label_key().tr().to_string())
                .collect(),
            rows,
        })
    }
}

/// O que fazer com uma tentativa de mudar a selecção.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Deixa passar.
    Allow,
    /// Recusa ([`REFUSAL`]).
    Refuse,
}

/// ⭐⭐ **O CADEADO** (escolha 4 do dono: *não trocar*) — a lei, pura. `locked` é a
/// [`ModeState::locked_entity`].
///
/// - em Object ⇒ nada muda;
/// - o alvo é a MESMA entidade ⇒ passa (re-seleccionar é um no-op, e recusá-lo ensinaria o artista
///   a ignorar avisos);
/// - **acrescentar** ⇒ recusa, mesmo sobre a própria: o que o modo não sabe representar é o ESTADO
///   de duas seleccionadas;
/// - limpar (alvo `None`) ⇒ passa — o `Esc` e o clique no vazio não podem parecer partidos;
/// - outra entidade ⇒ recusa.
#[must_use]
pub fn decide(locked: Option<u64>, target: Option<u64>, additive: bool) -> Decision {
    let Some(locked) = locked else {
        return Decision::Allow;
    };
    if additive {
        return Decision::Refuse;
    }
    match target {
        None => Decision::Allow,
        Some(t) if t == locked => Decision::Allow,
        Some(_) => Decision::Refuse,
    }
}

/// A frase da recusa do cadeado — nomeia a SAÍDA (`{mode}` é o modo em curso).
pub const REFUSAL: TextKey = TextKey::new("object_mode.locked");

/// A recusa já redigida, para o modo em curso.
#[must_use]
pub fn refusal(state: &ModeState) -> String {
    ph2d_i18n::tr_with(
        REFUSAL.key(),
        &[("mode", &state.current().label_key().tr())],
    )
}

/// ⭐ **O objecto ACTIVO** — o último que o artista acrescentou à selecção (o do Blender, e o que o
/// Painter guarda ao colapsar uma selecção múltipla); sem extras, o primário.
#[must_use]
pub fn active_of(selection: Option<u64>, extras: &[u64]) -> Option<u64> {
    extras.last().copied().or(selection)
}

/// ⛔ **As ferramentas que um MODO abre, e que por isso NÃO são botão da barra IMG** — dois caminhos
/// para o mesmo módulo divergem (spec/06 §5). A barra salta-as; os gates de alcance perguntam aqui
/// por que porta elas chegam.
pub const TOOLS_OPENED_BY_A_MODE: &[(&str, ObjectMode)] = &[("painter", ObjectMode::Paint)];

#[cfg(test)]
#[path = "object_mode_tests.rs"]
mod tests;
