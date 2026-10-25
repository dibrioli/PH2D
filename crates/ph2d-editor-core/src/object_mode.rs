//! ⭐⭐ **O MODO DO OBJECTO ACTIVO** — spec/06 §3.2 (F2), como o Blender: o modo é do objecto e o
//! TIPO declara quais tem (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`). Escolher
//! OUTRO objecto num modo de criação selecciona-o e volta a Object (dono 24/10, [`switches`];
//! `docs/GameEngineFuncional/01_W1_modos_selecao_inspector.md`).
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
/// O tipo é parte do pedido [`ModeRequest::Open`] e do que se publica: viaja com a API.
pub use ph2d_component_desc::ObjectKind;
use ph2d_i18n::TextKey;
use std::collections::BTreeMap;

/// **Um modo de edição.** `Object` é universal; os outros, só o tipo que os declara. ⭐ O mesmo nome
/// diz o mesmo GESTO em todo tipo (dono 24/10): `Draw` cria o conteúdo do objecto, `Edit` mexe no
/// que existe — o que cada um abre é da família, por par `(tipo, modo)`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectMode {
    /// Mover, rodar, escalar — o gizmo de transformação.
    Object,
    /// Criar o conteúdo: pintar a imagem (o Painter), traçar no Flip (o Draw Mode do Grease Pencil),
    /// desenhar formas no vetor.
    Draw,
    /// Mexer no que já está desenhado: as ferramentas de imagem, seleccionar e esculpir o traço do
    /// Flip, a pose de repouso do esqueleto, os nós do Motion.
    Edit,
    /// Posar o esqueleto: rodar e mover ossos, IK, corrigir o peso (Skeleton ▸ Pose — o Pose Mode da
    /// Armature do Blender; o Edit do esqueleto é a pose de repouso). A14.
    Pose,
}

impl ObjectMode {
    /// Todos, em ordem — a fonte da iteração (⛔ nunca escreva a lista uma segunda vez).
    pub const ALL: [ObjectMode; 4] = [
        ObjectMode::Object,
        ObjectMode::Draw,
        ObjectMode::Edit,
        ObjectMode::Pose,
    ];

    /// O nome que o artista lê.
    #[must_use]
    pub const fn label_key(self) -> TextKey {
        TextKey::new(match self {
            ObjectMode::Object => "object_mode.object",
            ObjectMode::Draw => "object_mode.draw",
            ObjectMode::Edit => "object_mode.edit",
            ObjectMode::Pose => "object_mode.pose",
        })
    }

    /// O id da linha deste modo no seletor.
    #[must_use]
    pub const fn row_id(self) -> NodeId {
        match self {
            ObjectMode::Object => crate::ids::OBJECT_MODE_OBJECT,
            ObjectMode::Draw => crate::ids::OBJECT_MODE_DRAW,
            ObjectMode::Edit => crate::ids::OBJECT_MODE_EDIT,
            ObjectMode::Pose => crate::ids::OBJECT_MODE_POSE,
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
    /// Uma aba de cima que pede o modo de UM tipo ao abrir (§3.3): entra se o activo for desse tipo
    /// e o tiver, senão fica em Object com a ferramenta de omissão — ⛔ nunca cria um objecto (§6.5).
    /// O tipo vai no pedido porque o `Draw` é de três tipos: a aba Flip não abre o Painter.
    Open(ObjectKind, ObjectMode),
    /// ⭐ Uma sub-linha de CONTEÚDO na Hierarquia (`hier_rows`): selecciona `entity`, entra
    /// em `mode` e entrega `key` (a camada…) à família — o «clicar abre o modo certo» do doc 01 §4.3.
    OpenContent {
        entity: u64,
        mode: ObjectMode,
        key: u64,
    },
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
    /// O activo, o tipo e os modos dele, publicados pela composição em todo quadro. `None` = nada
    /// seleccionado.
    published: Option<(u64, ObjectKind, Vec<ObjectMode>)>,
    /// ⭐ Uma porta de selecção escolheu OUTRO objecto ([`switches`]): o quadro volta a Object
    /// deixando a selecção como o artista a fez.
    release: bool,
    /// O último modo de criação de cada objecto — o destino do `Tab`.
    last: BTreeMap<u64, ObjectMode>,
    /// ⭐ **As PARTES da entidade trancada**, publicadas em todo quadro pela família do modo:
    /// `None` = o modo edita o objecto inteiro (Paint, Draw); `Some` = o modo edita o que está
    /// DENTRO dele, e a selecção pode ser qualquer parte (o Edit do vetor: as formas do objecto).
    parts: Option<Vec<u64>>,
    /// As partes transformam-se pelo gizmo de objecto ([`crate::screens::hero::mode_drive::ModeFamily::parts_take_the_object_gizmo`]).
    part_gizmo: bool,
    /// O modo é do TIPO ([`crate::screens::hero::mode_drive::ModeFamily::holds_the_whole_kind`]).
    whole_kind: bool,
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
        self.published
            .as_ref()
            .map_or(&[], |(_, _, m)| m.as_slice())
    }

    /// O activo publicado neste quadro (a entidade trancada num modo; em Object, a escolhida).
    #[must_use]
    pub fn published_entity(&self) -> Option<u64> {
        self.published.as_ref().map(|(e, _, _)| *e)
    }

    /// ⭐ **A composição publica o activo, o tipo dele e os modos que o tipo declara** — `Object` à
    /// frente, os da família a seguir, sem repetidos.
    pub fn publish(&mut self, active: Option<(u64, ObjectKind)>, family_modes: &[ObjectMode]) {
        self.published = active.map(|(e, kind)| {
            let mut modes = vec![ObjectMode::Object];
            for m in family_modes {
                if !modes.contains(m) {
                    modes.push(*m);
                }
            }
            (e, kind, modes)
        });
    }

    /// Uma porta de selecção escolheu outro objecto: o próximo quadro volta a Object.
    pub fn ask_release(&mut self) {
        self.release = self.active.is_some();
    }

    /// O pedido de [`Self::ask_release`], consumido.
    pub fn take_release(&mut self) -> bool {
        std::mem::take(&mut self.release)
    }

    /// Entra em `mode` sobre `entity` e lembra-o para o `Tab`.
    pub fn enter(&mut self, entity: u64, mode: ObjectMode) {
        debug_assert_ne!(mode, ObjectMode::Object, "Object não é um modo de criação");
        self.active = Some(ActiveMode { entity, mode });
        self.last.insert(entity, mode);
    }

    /// Volta a Object; devolve o modo que estava.
    pub fn leave(&mut self) -> Option<ActiveMode> {
        self.parts = None;
        self.whole_kind = false;
        self.release = false;
        self.active.take()
    }

    /// ⭐ A família do modo em curso publica as partes da entidade dele (ver o campo).
    pub fn publish_parts(&mut self, parts: Option<Vec<u64>>) {
        self.parts = parts;
    }

    /// Publica se as partes do modo em curso usam o gizmo de objecto.
    pub fn publish_part_gizmo(&mut self, on: bool) {
        self.part_gizmo = on;
    }

    /// As partes do modo em curso usam o gizmo de objecto.
    #[must_use]
    pub fn part_gizmo(&self) -> bool {
        self.part_gizmo
    }

    /// Publica se o modo em curso é do tipo inteiro.
    pub fn publish_whole_kind(&mut self, on: bool) {
        self.whole_kind = on;
    }

    /// ⭐ O modo em curso é do TIPO inteiro: escolher um objecto de outro tipo sai dele (em vez de
    /// o cadeado recusar).
    #[must_use]
    pub fn whole_kind(&self) -> bool {
        self.whole_kind
    }

    /// As partes publicadas da entidade trancada.
    #[must_use]
    pub fn parts(&self) -> Option<&[u64]> {
        self.parts.as_deref()
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
                    .and_then(|(e, _, _)| self.last.get(e).copied())
                    .filter(|m| has(*m));
                match remembered.or_else(|| available.iter().copied().find(|m| has(*m))) {
                    Some(m) => Step::Enter(m),
                    None => Step::Refuse,
                }
            }
            ModeRequest::Open(kind, m) => {
                let of_kind = self.published.as_ref().is_some_and(|(_, k, _)| *k == kind);
                if of_kind && self.current() == m && self.active.is_some() {
                    Step::Stay
                } else if of_kind && has(m) {
                    Step::Enter(m)
                } else {
                    Step::LeaveToDefault
                }
            }
            // O quadro troca-o por `Enter` depois de seleccionar a entidade (`mode_drive::drive`).
            ModeRequest::OpenContent { mode, .. } => self.resolve(ModeRequest::Enter(mode)),
        }
    }

    /// ⭐⭐ **O modo em curso ainda se segura?** — a rede de segurança de cada quadro.
    ///
    /// Ele só vale enquanto o módulo tem a entidade em mãos e a selecção é EXACTAMENTE ela — ou,
    /// num modo que edita as [partes](Self::publish_parts), só partes dela — ou NENHUMA: o Edit do
    /// Blender não sai ao desseleccionar (escolha do dono, 05/10: *«permita desselecionar mesmo sem
    /// sair do modo edit»*). A selecção muda por dezenas de portas que não são gestos (criar,
    /// duplicar, desfazer, largar um ficheiro) — em vez de as ensinar uma a uma, quem selecciona
    /// OUTRA coisa volta a Object; a entidade APAGADA cai pelo `module_holds` (o quadro do modo não
    /// acha família para uma entidade sem tipo).
    #[must_use]
    pub fn still_holds(&self, selection: Option<u64>, extras: &[u64], module_holds: bool) -> bool {
        self.active.is_none_or(|a| {
            module_holds
                && (selection.is_none() && extras.is_empty()
                    || match &self.parts {
                        None => selection == Some(a.entity) && extras.is_empty(),
                        Some(parts) => selection
                            .iter()
                            .chain(extras)
                            .all(|b| *b == a.entity || parts.contains(b)),
                    })
        })
    }

    /// ⭐ **O seletor** — o 1.º pulldown da fila; `None` sem objecto activo. Escreve o
    /// `ButtonState` de cada linha (a acesa é a do modo em curso).
    #[must_use]
    pub fn menu(&self, store: &mut WidgetStore) -> Option<AreaMenu> {
        let modes = self.published.as_ref().map(|(_, _, m)| m.clone())?;
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

/// ⭐⭐ **Trocar de objecto sai do modo** (dono 24/10 — revoga o cadeado de 03/10): a lei, pura. `true`
/// = a escolha de uma porta de selecção é OUTRO objecto e o modo em curso volta a Object.
///
/// - em Object ⇒ nada;
/// - o alvo é o trancado, ou uma parte DE DENTRO dele (os ossos do esqueleto) ⇒ fica;
/// - num modo do TIPO ([`ModeState::whole_kind`]) as partes são os outros objectos do tipo ⇒ sai
///   (*«seja do mesmo tipo ou de outro tipo»*);
/// - limpar (alvo `None`, `range == false`) ⇒ fica (05/10: desseleccionar não sai do Edit);
/// - um intervalo da Hierarquia (`range`) apanha linhas que a porta não nomeia ⇒ sai.
#[must_use]
pub fn switches(
    locked: Option<u64>,
    parts: Option<&[u64]>,
    whole_kind: bool,
    target: Option<u64>,
    range: bool,
) -> bool {
    let Some(locked) = locked else {
        return false;
    };
    let inside = |t: u64| t == locked || (!whole_kind && parts.is_some_and(|p| p.contains(&t)));
    range || target.is_some_and(|t| !inside(t))
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
pub const TOOLS_OPENED_BY_A_MODE: &[(&str, ObjectMode)] = &[("painter", ObjectMode::Draw)];

#[cfg(test)]
#[path = "object_mode_tests.rs"]
mod tests;
