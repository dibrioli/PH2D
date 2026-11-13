//! Os números do painel e o Drive: o valor em vigor, a edição do cartão que o escreve, as caixas e
//! a escolha de que rodas o motor gira. Irmão de [`crate::doc`] (tecto de 600 linhas por ficheiro).

use ph2d_editor_core::JointFieldEdit;
use ph2d_i18n::tr;
use ph2d_joint_templates::JointKind;

use crate::doc::{Drive, Edit, Field, Flag, JointEditorDoc, scope_options};

impl JointEditorDoc {
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
            .map(|&r| {
                (
                    Drive::Rule(r),
                    a.template.joints[usize::from(r)].name.to_owned(),
                )
            })
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
        let wheels = a
            .joints
            .iter()
            .filter(|j| j.joint.kind == JointKind::Wheel)
            .count();
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
                a.template
                    .joints
                    .iter()
                    .find_map(|r| r.tune.motor.map(|m| (m.value.to_degrees(), m.max_force)))
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
}
