//! ⭐ **O LUGAR DE UM OBJECTO MUDA DE DONO** (A17) — as tracks da posição (`TranslationX`,
//! `TranslationY`, `Position`) passam de uma entidade para outra posta no lugar dela, com o mesmo
//! pai: o esqueleto que a porta das raízes soltas põe por cima de um osso animado
//! (`ph2d-app-skeleton::loose`). O alvo (`AnimTarget`) não muda, então as keys, a trajectória, o
//! `rest` e a semente da expressão vão com a binding sem cópia nenhuma.
//!
//! ⚠️ É uma mudança de IDENTIDADE, não um passo: reescreve o documento E cada fotografia do
//! histórico (um Ctrl+Z da timeline que devolvesse a binding ao osso poria o osso em `L + key`
//! debaixo do esqueleto). E deixa um registo de sessão ([`TargetBinding::moved`]) para o undo
//! GLOBAL, que repõe o mundo e não o documento: ver [`rehome_moved_places`].

use ph2d_anim::AnimTarget;
use ph2d_ecs::stable_name_id;

use crate::{PropKind, TimelineDoc, TimelineState, WireId};

/// As propriedades que dizem ONDE o objecto está.
pub const PLACE_PROPS: [PropKind; 3] = [
    PropKind::TranslationX,
    PropKind::TranslationY,
    PropKind::Position,
];

/// De onde e para onde uma track da posição foi movida — registo de sessão, nunca gravado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceMove {
    /// A identidade do objecto de onde a track veio (o osso).
    pub home: WireId,
    /// A identidade do novo dono (o esqueleto).
    pub owner: WireId,
}

/// Porque mover o lugar de um objecto para outro não seria exacto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceMoveRefusal {
    /// O objecto tem relógio próprio (Time Remap): as keys da posição correm nele, e o novo dono
    /// correria no da cena.
    TimeRemap,
    /// A trajectória gira o objecto (auto-orient): o ângulo iria para o novo dono e somar-se-ia à
    /// rotação do objecto.
    AutoOrient,
    /// Uma fórmula lê a posição deste objecto pelo nome — passaria a ler o `0` local.
    Formula,
}

impl TimelineDoc {
    /// `entity` tem alguma track da posição?
    #[must_use]
    pub fn has_place_track(&self, entity: u64) -> bool {
        PLACE_PROPS
            .into_iter()
            .any(|p| self.binding_for(entity, p).is_some())
    }

    /// Porque as tracks da posição de `entity` (chamada `name`) não podem mudar de dono ao bit;
    /// `None` = podem.
    #[must_use]
    pub fn place_move_refusal(&self, entity: u64, name: Option<&str>) -> Option<PlaceMoveRefusal> {
        if self.binding_for(entity, PropKind::TimeRemap).is_some() {
            return Some(PlaceMoveRefusal::TimeRemap);
        }
        // A bandeira, e não o `AutoOrient::Active`: uma recusada pela track de Rotation do
        // objecto ACORDARIA no novo dono, que não a tem.
        if self
            .binding_for(entity, PropKind::Position)
            .is_some_and(|b| b.auto_orient)
        {
            return Some(PlaceMoveRefusal::AutoOrient);
        }
        let name = name?;
        let names = std::collections::BTreeMap::from([(stable_name_id(name), entity)]);
        let le_o_lugar = |src: &str| {
            let Ok(ir) = ph2d_expr_parse::parse(src) else {
                return false;
            };
            let mut out = Vec::new();
            crate::frame_solve::collect_links(&ir, &names, &mut out);
            out.iter()
                .any(|&(e, p)| e == entity && PLACE_PROPS.contains(&p))
        };
        let formulas = self
            .bindings()
            .iter()
            .filter_map(|b| b.expr.as_deref())
            .chain(
                self.clips()
                    .iter()
                    .flat_map(|c| c.expr.values().map(String::as_str)),
            );
        for src in formulas {
            if le_o_lugar(src) {
                return Some(PlaceMoveRefusal::Formula);
            }
        }
        None
    }
}

impl TimelineState {
    /// ⭐ **Move as tracks da posição de `from` para `to`**, no documento e em todo o histórico,
    /// e regista `mv` em cada uma. Devolve quantas mudaram. Quem chama já perguntou
    /// [`TimelineDoc::place_move_refusal`].
    pub fn move_place(&mut self, from: u64, to: u64, mv: PlaceMove) -> usize {
        let alvos: Vec<AnimTarget> = self
            .doc
            .bindings()
            .iter()
            .filter(|b| b.entity == from && PLACE_PROPS.contains(&b.prop))
            .map(|b| b.target)
            .collect();
        for &t in &alvos {
            relocate(self, t, to, mv.owner, Some(mv));
        }
        alvos.len()
    }
}

/// Põe a binding de `target` em `entity`/`wire` no documento e em cada fotografia do histórico.
fn relocate(
    state: &mut TimelineState,
    target: AnimTarget,
    entity: u64,
    wire: WireId,
    mv: Option<PlaceMove>,
) {
    let docs = std::iter::once(&mut state.doc).chain(state.history.docs_mut());
    for doc in docs {
        if let Some(b) = doc.bindings_mut().iter_mut().find(|b| b.target == target) {
            b.entity = entity;
            b.wire_id = wire;
            b.missing = false;
            if mv.is_some() {
                b.moved = mv;
            }
        }
    }
}

/// ⭐ **O undo GLOBAL atravessa a mudança de dono, e a track vai com ele** — chamada pelo `upkeep`
/// entre a cura e a purga. Duas leis, ambas lidas do MUNDO que o undo repôs:
///
/// - a track está no dono e o dono não existe (o undo voltou a antes da adopção) mas a casa
///   existe ⇒ volta para casa (sem isto a purga apagava-a, e o Ctrl+Z da timeline não a trazia);
/// - a track está em casa, o dono existe e a casa é filha dele (o redo) ⇒ volta para o dono.
///
/// `entity_of` resolve uma identidade; `parent_of` dá o pai de uns bits. Devolve quantas mudaram.
/// Sem nenhuma track movida não aloca.
pub fn rehome_moved_places(
    state: &mut TimelineState,
    entity_of: impl Fn(WireId) -> Option<u64>,
    parent_of: impl Fn(u64) -> Option<u64>,
) -> usize {
    let mut mudam: Vec<(AnimTarget, u64, WireId)> = Vec::new();
    for b in state.doc.bindings() {
        let Some(mv) = b.moved else {
            continue;
        };
        if b.missing && b.wire_id == mv.owner {
            if entity_of(mv.owner).is_none()
                && let Some(casa) = entity_of(mv.home)
            {
                mudam.push((b.target, casa, mv.home));
            }
        } else if !b.missing
            && b.wire_id == mv.home
            && let Some(dono) = entity_of(mv.owner)
            && parent_of(b.entity) == Some(dono)
        {
            mudam.push((b.target, dono, mv.owner));
        }
    }
    for &(t, e, w) in &mudam {
        relocate(state, t, e, w, None);
    }
    mudam.len()
}
