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
            || {
                ph2d_joint_templates::by_id(self.template)
                    .unwrap_or(&ph2d_joint_templates::ALL[0])
            },
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
            let g = a.template.joints.get(usize::from(o.rules[0])).map(|r| r.group);
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

    /// O valor de um número agora, na unidade do cartão (o da 1.ª junta do alcance).
    #[must_use]
    pub fn value(&self, f: Field) -> Option<f32> {
        if f == Field::Wheelbase {
            return self.wheelbase();
        }
        let info = self.targets().first()?.info.clone()?;
        Some(match f {
            Field::LimitMin => info.limit_min_ui,
            Field::LimitMax => info.limit_max_ui,
            Field::MotorSpeed => info.motor_speed_ui,
            Field::MotorForce => info.motor_max_force,
            Field::Stiffness => info.stiffness,
            Field::Damping => info.damping,
            Field::RestLength => info.rest_length,
            Field::MaxLength => info.max_length,
            Field::BreakForce => info.break_force,
            Field::BreakTorque => info.break_torque,
            Field::Wheelbase => return None,
        })
    }

    /// O entre-eixos medido na cena: a distância entre as peças das duas primeiras regras de roda.
    #[must_use]
    pub fn wheelbase(&self) -> Option<f32> {
        let a = self.assembly()?;
        let wheels: Vec<u16> = a
            .template
            .joints
            .iter()
            .filter(|r| r.kind == JointKind::Wheel)
            .map(|r| r.b)
            .collect();
        let [rear, front] = wheels[..] else {
            return None;
        };
        let (r, f) = (a.parts_of(rear).next()?, a.parts_of(front).next()?);
        Some(libm::hypotf(f.at[0] - r.at[0], f.at[1] - r.at[1]))
    }

    /// A edição do cartão que escreve `v` em `f`.
    #[must_use]
    pub const fn edit_of(f: Field, v: f32) -> Option<JointFieldEdit> {
        Some(match f {
            Field::LimitMin => JointFieldEdit::LimitMin(v),
            Field::LimitMax => JointFieldEdit::LimitMax(v),
            Field::MotorSpeed => JointFieldEdit::MotorSpeed(v),
            Field::MotorForce => JointFieldEdit::MotorMaxForce(v),
            Field::Stiffness => JointFieldEdit::Stiffness(v),
            Field::Damping => JointFieldEdit::Damping(v),
            Field::RestLength => JointFieldEdit::RestLength(v),
            Field::MaxLength => JointFieldEdit::MaxLength(v),
            Field::BreakForce => JointFieldEdit::BreakForce(v),
            Field::BreakTorque => JointFieldEdit::BreakTorque(v),
            Field::Wheelbase => return None,
        })
    }

    /// Escreve `v` no número `f` das juntas do alcance (um passo de desfazer).
    pub fn commit(&mut self, f: Field, v: f32) {
        if !v.is_finite() {
            return;
        }
        if f == Field::Wheelbase {
            if let Some(a) = self.assembly() {
                self.pending.push(Edit::Wheelbase {
                    assembly: a.sid,
                    metres: v,
                });
            }
            return;
        }
        let joints: Vec<u64> = self.targets().iter().map(|j| j.bits).collect();
        if let Some(edit) = Self::edit_of(f, v)
            && !joints.is_empty()
        {
            self.pending.push(Edit::Joints { joints, edit });
        }
    }

    /// Uma caixa do painel: inverte o estado da 1.ª junta do alcance, em todas.
    pub fn toggle(&mut self, f: Flag) {
        let t = self.targets();
        let Some(info) = t.first().and_then(|j| j.info.clone()) else {
            return;
        };
        let joints: Vec<u64> = t.iter().map(|j| j.bits).collect();
        let edit = match f {
            Flag::Limits => JointFieldEdit::LimitsEnabled(!info.limits_enabled),
            Flag::Motor => JointFieldEdit::MotorEnabled(!info.motor_enabled),
            Flag::Breakable => JointFieldEdit::BreakEnabled(!info.break_enabled),
        };
        self.pending.push(Edit::Joints { joints, edit });
    }

    /// As opções do Drive: cada regra de roda, «Both» e «None».
    #[must_use]
    pub fn drive_options(&self) -> Vec<(Drive, String)> {
        let Some(a) = self.assembly() else {
            return Vec::new();
        };
        let wheels: Vec<u16> = (0..a.template.joints.len())
            .filter(|&i| a.template.joints[i].kind == JointKind::Wheel)
            .filter_map(|i| u16::try_from(i).ok())
            .collect();
        if wheels.len() < 2 {
            return Vec::new();
        }
        let mut out: Vec<(Drive, String)> = wheels
            .iter()
            .map(|&r| (Drive::Rule(r), a.template.joints[usize::from(r)].name.to_owned()))
            .collect();
        out.push((Drive::All, tr("joint_editor.drive.all").to_owned()));
        out.push((Drive::None, tr("joint_editor.drive.none").to_owned()));
        out
    }

    /// O Drive em vigor, lido das juntas.
    #[must_use]
    pub fn drive(&self) -> Option<Drive> {
        let a = self.assembly()?;
        let on: Vec<u16> = a
            .joints
            .iter()
            .filter(|j| j.joint.kind == JointKind::Wheel && j.joint.motor_enabled)
            .map(|j| j.rule)
            .collect();
        let wheels = a.joints.iter().filter(|j| j.joint.kind == JointKind::Wheel).count();
        Some(match on.len() {
            0 => Drive::None,
            n if n == wheels => Drive::All,
            _ => Drive::Rule(on[0]),
        })
    }

    /// Escolhe que rodas o motor gira. Uma roda que passa a ser puxada herda a velocidade e a força da
    /// que já puxava (ou as de fábrica do gabarito).
    pub fn set_drive(&mut self, d: Drive) {
        let Some(a) = self.assembly() else {
            return;
        };
        let from = a
            .joints
            .iter()
            .find(|j| j.joint.motor_enabled)
            .and_then(|j| j.info.as_ref())
            .map(|i| (i.motor_speed_ui, i.motor_max_force))
            .or_else(|| {
                a.template.joints.iter().find_map(|r| {
                    r.tune
                        .motor
                        .map(|m| (m.value.to_degrees(), m.max_force))
                })
            });
        let mut out = Vec::new();
        for j in a.joints.iter().filter(|j| j.joint.kind == JointKind::Wheel) {
            let on = match d {
                Drive::All => true,
                Drive::None => false,
                Drive::Rule(r) => j.rule == r,
            };
            if on == j.joint.motor_enabled {
                continue;
            }
            out.push(Edit::Joints {
                joints: vec![j.bits],
                edit: JointFieldEdit::MotorEnabled(on),
            });
            if on && let Some((speed, force)) = from {
                out.push(Edit::Joints {
                    joints: vec![j.bits],
                    edit: JointFieldEdit::MotorSpeed(speed),
                });
                out.push(Edit::Joints {
                    joints: vec![j.bits],
                    edit: JointFieldEdit::MotorMaxForce(force),
                });
            }
        }
        self.pending.extend(out);
    }

    /// Escolhe a opção de alcance `i`: o grupo liga «Same as»; uma regra só desliga-o.
    pub fn set_scope(&mut self, i: usize) {
        let Some(a) = self.assembly() else {
            return;
        };
        let opts = scope_options(a.template);
        let Some(o) = opts.get(i) else {
            return;
        };
        let rule = o.rules[0];
        let group = a.template.joints.get(usize::from(rule)).map(|r| r.group);
        let joint = a.joints.iter().find(|j| j.rule == rule).map(|j| j.bits);
        let (sid, want) = (a.sid, o.group.is_some());
        if let (Some(g), Some(joint)) = (group, joint)
            && a.is_linked(g) != want
            && opts.iter().any(|o| o.group == Some(g))
        {
            self.pending.push(Edit::Linked { joint, on: want });
        }
        self.scope.insert(sid, i);
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
        self.hits.iter().rev().find(|(_, x, _)| x == h).map(|(r, _, _)| *r)
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
