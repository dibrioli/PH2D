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
        match f {
            Field::Wheelbase => return self.wheelbase(),
            Field::Count => return self.row_role().map(|i| f32::from(self.draft_counts()[i])),
            _ => {}
        }
        let info = self.targets().first()?.info.clone()?;
        Some(match f {
            Field::LimitMin => info.limit_min_ui,
            Field::LimitMax => info.limit_max_ui,
            Field::MotorSpeed => info.motor_speed_ui,
            Field::MotorTarget => info.motor_target_ui,
            Field::MotorForce => info.motor_max_force,
            Field::Stiffness => info.stiffness,
            Field::Damping => info.damping,
            Field::RestLength => info.rest_length,
            Field::MaxLength => info.max_length,
            Field::BreakForce => info.break_force,
            Field::BreakTorque => info.break_torque,
            Field::Wheelbase | Field::Count => return None,
        })
    }

    /// O papel de várias peças do gabarito escolhido (as tábuas, os elos), se há um.
    #[must_use]
    pub fn row_role(&self) -> Option<usize> {
        self.template().roles.iter().position(|r| r.many.is_some())
    }

    /// Quantas peças cada papel leva no «Make the parts»: o número do campo no papel de várias.
    #[must_use]
    pub fn draft_counts(&self) -> Vec<u16> {
        let t = self.template();
        let counts: Vec<u16> = t
            .roles
            .iter()
            .map(|r| if r.many.is_some() { self.count } else { 0 })
            .collect();
        (0..t.roles.len())
            .map(|i| ph2d_joint_assembly::joint_factory::count_of(&t.roles[i], &counts, i))
            .collect()
    }

    /// O valor vivo de `f` enquanto se arrasta (pela alça ou pela letra): o que o painel e o desenho mostram.
    #[must_use]
    pub fn live(&self, f: Field) -> Option<f32> {
        self.drag.filter(|d| d.field == f).map(|d| d.value)
    }

    /// O arrasto da letra de `f`: o valor de partida, o passo por pixel e o intervalo. Os do campo da
    /// junta no Inspector (`populate_physics`: passo e faixa); em metros, o passo fino de 1 cm.
    #[must_use]
    pub fn scrub_law(&self, f: Field) -> Option<(f32, f64, (f64, f64))> {
        let (step, bounds) = self.step_of(f);
        Some((self.value(f)?, step, bounds))
    }

    /// O passo (por pixel do scrub, e o grão da alça do desenho) e o intervalo de `f`.
    #[must_use]
    pub fn step_of(&self, f: Field) -> (f64, (f64, f64)) {
        const WIDE: f64 = 3600.0; // LITERAL-PX-OK: a faixa do Inspector (graus/s ou m/s)
        const BIG: f64 = 10_000.0; // LITERAL-PX-OK: N, N·m (o tecto do Inspector)
        const CM: f64 = 0.01; // LITERAL-PX-OK: metros
        let (lm, mm) = (self.kind.limits_in_metres(), self.kind.motor_in_metres());
        let fine = |metres: bool| if metres { CM } else { 1.0 };
        match f {
            Field::LimitMin | Field::LimitMax => (fine(lm), (-WIDE, WIDE)),
            Field::MotorSpeed | Field::MotorTarget => (fine(mm), (-WIDE, WIDE)),
            Field::MotorForce => (0.1, (0.0, BIG)),
            Field::Stiffness => (1.0, (0.0, BIG * 10.0)),
            Field::Damping => (0.1, (0.0, BIG / 10.0)),
            Field::RestLength | Field::Wheelbase => (CM, (0.0, BIG / 10.0)),
            Field::MaxLength => (CM, (CM / 10.0, BIG / 10.0)),
            Field::BreakForce | Field::BreakTorque => (1.0, (0.0, BIG)),
            Field::Count => (1.0, (1.0, f64::from(u16::MAX))),
        }
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
            Field::MotorTarget => JointFieldEdit::MotorTarget(v),
            Field::MotorForce => JointFieldEdit::MotorMaxForce(v),
            Field::Stiffness => JointFieldEdit::Stiffness(v),
            Field::Damping => JointFieldEdit::Damping(v),
            Field::RestLength => JointFieldEdit::RestLength(v),
            Field::MaxLength => JointFieldEdit::MaxLength(v),
            Field::BreakForce => JointFieldEdit::BreakForce(v),
            Field::BreakTorque => JointFieldEdit::BreakTorque(v),
            Field::Wheelbase | Field::Count => return None,
        })
    }

    /// Escreve `v` no número `f` das juntas do alcance (um passo de desfazer).
    pub fn commit(&mut self, f: Field, v: f32) {
        if !v.is_finite() {
            return;
        }
        if f == Field::Count {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            // preso a 1..=u16::MAX
            let n = v.round().clamp(1.0, f32::from(u16::MAX)) as u16; // CLAMP-OK: o tipo do contador
            self.count = n;
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
        let rows = self.targets();
        if let Some(edit) = Self::edit_of(f, v) {
            self.write(&rows, edit);
        }
    }

    /// Uma caixa do painel: inverte o estado da 1.ª junta do alcance, em todas.
    pub fn toggle(&mut self, f: Flag) {
        let t = self.targets();
        let Some(info) = t.first().and_then(|j| j.info.clone()) else {
            return;
        };
        let edit = match f {
            Flag::Limits => JointFieldEdit::LimitsEnabled(!info.limits_enabled),
            Flag::Motor => JointFieldEdit::MotorEnabled(!info.motor_enabled),
            Flag::Breakable => JointFieldEdit::BreakEnabled(!info.break_enabled),
        };
        self.write(&t, edit);
    }

    /// As opções do Drive: cada regra de roda, «Both» e «None».
    #[must_use]
    pub fn drive_options(&self) -> Vec<(Drive, String)> {
        let t = self.template();
        let wheels: Vec<u16> = (0..t.joints.len())
            .filter(|&i| t.joints[i].kind == JointKind::Wheel)
            .filter_map(|i| u16::try_from(i).ok())
            .collect();
        if wheels.len() < 2 {
            return Vec::new();
        }
        let mut out: Vec<(Drive, String)> = wheels
            .iter()
            .map(|&r| (Drive::Rule(r), t.joints[usize::from(r)].name.to_owned()))
            .collect();
        out.push((Drive::All, tr("joint_editor.drive.all").to_owned()));
        out.push((Drive::None, tr("joint_editor.drive.none").to_owned()));
        out
    }

    /// O Drive em vigor, lido das juntas.
    #[must_use]
    pub fn drive(&self) -> Option<Drive> {
        let rows = self.rows();
        let on: Vec<u16> = rows
            .iter()
            .filter(|j| j.joint.kind == JointKind::Wheel && j.joint.motor_enabled)
            .map(|j| j.rule)
            .collect();
        let wheels = rows
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
        let rows = self.rows();
        let from = rows
            .iter()
            .find(|j| j.joint.motor_enabled)
            .and_then(|j| j.info.as_ref())
            .map(|i| (i.motor_speed_ui, i.motor_max_force))
            .or_else(|| {
                self.template()
                    .joints
                    .iter()
                    .find_map(|r| r.tune.motor.map(|m| (m.value.to_degrees(), m.max_force)))
            });
        for j in rows.iter().filter(|j| j.joint.kind == JointKind::Wheel) {
            let on = match d {
                Drive::All => true,
                Drive::None => false,
                Drive::Rule(r) => j.rule == r,
            };
            if on == j.joint.motor_enabled {
                continue;
            }
            let one = std::slice::from_ref(j);
            self.write(one, JointFieldEdit::MotorEnabled(on));
            if on && let Some((speed, force)) = from {
                self.write(one, JointFieldEdit::MotorSpeed(speed));
                self.write(one, JointFieldEdit::MotorMaxForce(force));
            }
        }
    }

    /// Escolhe a opção de alcance `i`: o grupo liga «Same as»; uma regra só desliga-o.
    pub fn set_scope(&mut self, i: usize) {
        let Some(a) = self.assembly() else {
            self.scope.insert(0, i);
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
