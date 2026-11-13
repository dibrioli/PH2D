//! **O estado da aba** e as decisões que não pintam: que montagem está aberta, que gabarito vem a seguir,
//! o que uma queda faria, a que juntas um número se aplica. Tudo o que muda o mundo sai como [`Edit`] e é
//! aplicado pela família no quadro ([`crate::apply`]).

use std::collections::BTreeMap;

use ph2d_editor_core::JointFieldEdit;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::{JointKind, Template};

use crate::model::{AssemblyRow, JointRow, Snapshot};
use crate::stage::StageMap;

/// O que a aba mostra no centro.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Focus {
    /// Uma montagem da cena (o `StableId` dela).
    Assembly(u64),
    /// Uma montagem POR FAZER do gabarito escolhido: nasce na primeira peça largada.
    Draft,
}

/// A lista suspensa aberta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Menu {
    Kind,
    Template,
}

/// Um número do painel de baixo, na unidade do cartão do Inspector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Field {
    LimitMin,
    LimitMax,
    MotorSpeed,
    MotorForce,
    Stiffness,
    Damping,
    RestLength,
    MaxLength,
    BreakForce,
    BreakTorque,
    /// A distância entre os eixos das duas rodas, metros (move a da frente).
    Wheelbase,
}

/// Uma caixa de marcar do painel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Flag {
    Limits,
    Motor,
    Breakable,
}

/// Que rodas o motor gira.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Drive {
    Rule(u16),
    All,
    None,
}

/// Uma alça no desenho.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Handle {
    LimitMin,
    LimitMax,
    Wheelbase,
}

impl Handle {
    #[must_use]
    pub const fn field(self) -> Field {
        match self {
            Self::LimitMin => Field::LimitMin,
            Self::LimitMax => Field::LimitMax,
            Self::Wheelbase => Field::Wheelbase,
        }
    }
}

/// O que um alvo pintado é.
#[derive(Clone, Debug, PartialEq)]
pub enum Hit {
    Row(u64),
    New,
    Remove,
    OpenMenu(Menu),
    Kind(JointKind),
    Template(u16),
    /// A forma `n` do papel `role`: um clique seleciona a peça na cena.
    Shape(u16, u16),
    /// «Pin to the scene»: o papel fica por conta do cenário.
    World(u16),
    Scope(usize),
    Field(Field),
    Flag(Flag),
    Drive(Drive),
    Handle(Handle),
    /// «Open in Inspector»: seleciona a junta.
    Inspect,
}

/// O que a aba pede ao mundo.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// Põe os objetos `bodies` (pelos nomes) no papel; `assembly == 0` = uma montagem nova do gabarito.
    Place {
        assembly: u64,
        template: u16,
        role: u16,
        bodies: Vec<String>,
    },
    /// O papel fica por conta do cenário.
    PlaceWorld {
        assembly: u64,
        template: u16,
        role: u16,
    },
    /// Apaga a montagem e as juntas dela (as peças ficam).
    Remove { assembly: u64 },
    /// A mesma edição do cartão Joint, em cada junta.
    Joints {
        joints: Vec<u64>,
        edit: JointFieldEdit,
    },
    /// «Same as …» do grupo desta junta.
    Linked { joint: u64, on: bool },
    /// Seleciona um objeto na cena (o Inspector mostra-o).
    Select { bits: u64 },
    /// Muda o entre-eixos para `metres`.
    Wheelbase { assembly: u64, metres: f32 },
}

/// Uma opção do «a que juntas isto se aplica»: «Both wheels», «Rear wheel», «Front wheel».
#[derive(Clone, Debug, PartialEq)]
pub struct ScopeOpt {
    pub label: String,
    pub rules: Vec<u16>,
    /// `Some(g)` = a opção é o grupo inteiro (liga «Same as»).
    pub group: Option<u8>,
}

/// Um arrasto de fora sobre a aba.
#[derive(Clone, Debug, PartialEq)]
pub struct Dropping {
    pub role: Option<u16>,
    pub names: Vec<String>,
    pub ok: bool,
    pub line: String,
}

/// Uma alça presa.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Drag {
    pub handle: Handle,
    pub value: f32,
}

pub struct JointEditorDoc {
    pub(crate) snap: Snapshot,
    pub(crate) focus: Focus,
    pub(crate) kind: JointKind,
    pub(crate) template: u16,
    /// A opção de um papel escolhida por montagem (quando o grupo não está ligado).
    pub(crate) scope: BTreeMap<u64, usize>,
    pub(crate) menu: Option<Menu>,
    pub(crate) hits: Vec<(Rect, Hit, String)>,
    pub(crate) hover: Option<Hit>,
    pub(crate) pressed: Option<Hit>,
    pub(crate) dropping: Option<Dropping>,
    pub(crate) typing: Option<(Field, String)>,
    pub(crate) drag: Option<Drag>,
    pub(crate) pending: Vec<Edit>,
    pub(crate) map: Option<StageMap>,
    pub(crate) counts: Vec<u16>,
    /// A frase do pé: o que a última queda fez.
    pub(crate) note: Option<String>,
    pub(crate) area: Rect,
    /// Onde cada alça pintada mede o cursor.
    pub(crate) geo: Vec<(Handle, crate::paint_stage::Geo)>,
}

impl Default for JointEditorDoc {
    fn default() -> Self {
        let first = ph2d_joint_templates::ALL[0];
        Self {
            snap: Snapshot::default(),
            focus: Focus::Draft,
            kind: first.kind,
            template: first.id.0,
            scope: BTreeMap::new(),
            menu: None,
            hits: Vec::new(),
            hover: None,
            pressed: None,
            dropping: None,
            typing: None,
            drag: None,
            pending: Vec::new(),
            map: None,
            counts: Vec::new(),
            note: None,
            area: Rect::default(),
            geo: Vec::new(),
        }
    }
}

/// Os tipos que têm gabarito, pela ordem do catálogo.
#[must_use]
pub fn kinds() -> Vec<JointKind> {
    let mut out: Vec<JointKind> = Vec::new();
    for t in ph2d_joint_templates::ALL {
        if !out.contains(&t.kind) {
            out.push(t.kind);
        }
    }
    out
}

/// A chave do nome do tipo (a do seletor Kind do cartão Joint).
#[must_use]
pub fn kind_key(k: JointKind) -> &'static str {
    match k {
        JointKind::Pin => "panel.inspector.joint.pin",
        JointKind::Spring => "panel.inspector.joint.spring",
        JointKind::Rope => "panel.inspector.joint.rope",
        JointKind::Weld => "panel.inspector.joint.weld",
        JointKind::Slider => "panel.inspector.joint.slider",
        JointKind::Rod => "panel.inspector.joint.rod",
        JointKind::Wheel => "panel.inspector.joint.wheel",
        JointKind::Pulley => "panel.inspector.joint.pulley",
        JointKind::Custom => "panel.inspector.joint.custom",
    }
}

/// As opções de alcance de uma montagem: por grupo com mais de uma regra, o grupo e depois cada uma.
#[must_use]
pub fn scope_options(t: &Template) -> Vec<ScopeOpt> {
    let mut groups: Vec<u8> = t.joints.iter().map(|r| r.group).collect();
    groups.dedup();
    groups.sort_unstable();
    groups.dedup();
    let mut out = Vec::new();
    for g in groups {
        let rules: Vec<u16> = (0..t.joints.len())
            .filter(|&i| t.joints[i].group == g)
            .filter_map(|i| u16::try_from(i).ok())
            .collect();
        if rules.len() > 1 {
            let all = t.joints[usize::from(rules[0])].kind == JointKind::Wheel && rules.len() == 2;
            let label = if all {
                tr("joint_editor.scope.both_wheels").to_owned()
            } else {
                tr_with("joint_editor.scope.all", &[("n", &rules.len())])
            };
            out.push(ScopeOpt {
                label,
                rules: rules.clone(),
                group: Some(g),
            });
        }
        for r in rules {
            out.push(ScopeOpt {
                label: t.joints[usize::from(r)].name.to_owned(),
                rules: vec![r],
                group: None,
            });
        }
    }
    out
}

impl JointEditorDoc {
    /// A montagem aberta, se a aba mostra uma.
    #[must_use]
    pub fn assembly(&self) -> Option<&AssemblyRow> {
        match self.focus {
            Focus::Assembly(sid) => self.snap.find(sid),
            Focus::Draft => None,
        }
    }

    /// O gabarito do centro: o da montagem aberta, ou o escolhido nos seletores.
    #[must_use]
    pub fn template(&self) -> &'static Template {
        self.assembly().map_or_else(
            || ph2d_joint_templates::by_id(self.template).unwrap_or(&ph2d_joint_templates::ALL[0]),
            |a| a.template,
        )
    }

    /// Abre a montagem `sid` (ou a montagem dona da junta `sid`); `0` ou desconhecida = o gabarito a fazer.
    pub fn open_on(&mut self, sid: u64) {
        let found = self
            .snap
            .find(sid)
            .or_else(|| self.snap.owner_of_joint(sid))
            .map(|a| (a.sid, a.template.kind, a.template.id.0));
        match found {
            Some((sid, kind, t)) => {
                self.focus = Focus::Assembly(sid);
                self.kind = kind;
                self.template = t;
            }
            None if sid != 0 => self.focus = Focus::Draft,
            None => {}
        }
        self.menu = None;
        self.typing = None;
    }

    /// O índice da opção de alcance em vigor: o grupo, se está ligado; senão a escolhida.
    #[must_use]
    pub fn scope_index(&self, a: &AssemblyRow, opts: &[ScopeOpt]) -> usize {
        let chosen = self.scope.get(&a.sid).copied().unwrap_or(0);
        if let Some(o) = opts.get(chosen)
            && o.group.is_none()
        {
            let g = a
                .template
                .joints
                .get(usize::from(o.rules[0]))
                .map(|r| r.group);
            if g.is_some_and(|g| !a.is_linked(g)) {
                return chosen;
            }
        }
        // O grupo ligado: a opção dele; sem grupo (uma regra só), a escolhida ou a primeira.
        opts.iter()
            .position(|o| o.group.is_some_and(|g| a.is_linked(g)))
            .unwrap_or(chosen.min(opts.len().saturating_sub(1)))
    }

    /// As juntas a que um número se aplica agora.
    #[must_use]
    pub fn targets(&self) -> Vec<&JointRow> {
        let Some(a) = self.assembly() else {
            return Vec::new();
        };
        let opts = scope_options(a.template);
        let Some(o) = opts.get(self.scope_index(a, &opts)) else {
            return Vec::new();
        };
        a.joints
            .iter()
            .filter(|j| o.rules.contains(&j.rule))
            .collect()
    }

    /// **O que largar `names` sobre o papel `role` faria** — dito ANTES de soltar.
    #[must_use]
    pub fn verdict(&self, role: u16, names: &[String]) -> (bool, String) {
        let t = self.template();
        let Some(spec) = t.role(role) else {
            return (false, String::new());
        };
        let Some(first) = names.first() else {
            return (false, String::new());
        };
        let asm = self.assembly();
        let whose = asm.map_or_else(
            || tr_with("joint_editor.drop.new_assembly", &[("template", &t.name)]),
            |a| a.name.clone(),
        );
        if !self.snap.bodies.iter().any(|b| b == first) {
            return (
                false,
                tr_with("joint_editor.drop.no_body", &[("name", first)]),
            );
        }
        if let Some(a) = asm {
            if let Some(p) = a.parts.iter().find(|p| &p.name == first) {
                let role = t.role(p.role).map_or("", |r| r.name);
                return (
                    false,
                    tr_with(
                        "joint_editor.drop.already",
                        &[("name", first), ("role", &role)],
                    ),
                );
            }
            if spec.many.is_none()
                && let Some(p) = a.parts_of(role).next()
            {
                let who = if p.sid == 0 {
                    tr("joint_editor.scene").to_owned()
                } else {
                    p.name.clone()
                };
                return (
                    false,
                    tr_with(
                        "joint_editor.drop.taken",
                        &[("role", &spec.name), ("name", &who)],
                    ),
                );
            }
        }
        let n = if spec.many.is_some() { names.len() } else { 1 };
        let line = if n > 1 {
            tr_with(
                "joint_editor.drop.many",
                &[("n", &n), ("role", &spec.name), ("assembly", &whose)],
            )
        } else {
            tr_with(
                "joint_editor.drop.one",
                &[("name", first), ("role", &spec.name), ("assembly", &whose)],
            )
        };
        (true, line)
    }

    /// Larga `names` no papel `role`: o pedido sai para o quadro.
    pub fn drop_into(&mut self, role: u16, names: &[String]) -> bool {
        let (ok, _) = self.verdict(role, names);
        if !ok {
            return false;
        }
        let many = self.template().role(role).is_some_and(|r| r.many.is_some());
        let bodies = if many {
            names.to_vec()
        } else {
            names.iter().take(1).cloned().collect()
        };
        self.pending.push(Edit::Place {
            assembly: self.assembly().map_or(0, |a| a.sid),
            template: self.template().id.0,
            role,
            bodies,
        });
        true
    }

    /// Começa uma montagem nova do gabarito escolhido.
    pub fn new_draft(&mut self) {
        self.focus = Focus::Draft;
        self.menu = None;
        self.typing = None;
        self.note = None;
    }

    pub fn choose_kind(&mut self, k: JointKind) {
        self.kind = k;
        if let Some(t) = ph2d_joint_templates::of_kind(k).next() {
            self.template = t.id.0;
        }
        self.new_draft();
    }

    pub fn choose_template(&mut self, id: u16) {
        if let Some(t) = ph2d_joint_templates::by_id(id) {
            self.kind = t.kind;
            self.template = id;
        }
        self.new_draft();
    }

    /// Recebe o retrato do quadro; uma montagem aberta que deixou de existir volta ao gabarito a fazer.
    pub fn set_snapshot(&mut self, snap: Snapshot) {
        self.snap = snap;
        if let Focus::Assembly(sid) = self.focus
            && self.snap.find(sid).is_none()
        {
            self.focus = Focus::Draft;
        }
    }

    /// Drena os pedidos do quadro.
    pub fn take_pending(&mut self) -> Vec<Edit> {
        std::mem::take(&mut self.pending)
    }

    /// A montagem nova nasceu: a aba passa a mostrá-la.
    pub fn made(&mut self, sid: u64, line: String) {
        self.focus = Focus::Assembly(sid);
        self.note = Some(line);
    }

    /// Há um gesto a meio (uma alça presa, um número a ser escrito): o desfazer espera por ele.
    #[must_use]
    pub const fn gesture_in_progress(&self) -> bool {
        self.drag.is_some() || self.typing.is_some()
    }

    /// Onde o alvo `h` foi pintado neste quadro.
    #[must_use]
    pub fn hits_for(&self, h: &Hit) -> Option<Rect> {
        self.hits
            .iter()
            .rev()
            .find(|(_, x, _)| x == h)
            .map(|(r, _, _)| *r)
    }

    /// A frase que o arrasto em curso mostra (o que soltar faria).
    #[must_use]
    pub fn dropping_line(&self) -> Option<String> {
        self.dropping.as_ref().map(|d| d.line.clone())
    }

    /// O alvo pintado debaixo do cursor (o último pintado ganha: as listas abertas vêm por cima).
    #[must_use]
    pub fn hit_at(&self, x: f32, y: f32) -> Option<&Hit> {
        self.hits
            .iter()
            .rev()
            .find(|(r, _, _)| r.contains(x, y))
            .map(|(_, h, _)| h)
    }
}

#[cfg(test)]
#[path = "doc_tests.rs"]
mod tests;
