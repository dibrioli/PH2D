//! **As juntas do gabarito por fazer** (dono 15/11): o painel de criação mostra os mesmos cartões da
//! montagem feita, com os números de fábrica, e o mesmo «Applies to» («Plank» = todas as juntas da fila).
//! Cada edição guarda-se por regra, por ordem; o «Make the parts» escreve-as pela porta do cartão Joint
//! (`joint_with_edit`), a mesma que o rascunho usa para se mostrar — o número visto é o número feito.

use ph2d_editor_core::JointFieldEdit;
use ph2d_editor_core::screens::hero::InspectorJointInfo;
use ph2d_joint_assembly::joint_info::{joint_with_edit, limit_out, tag_of};
use ph2d_physics_ecs::{MotorMode, PhysicsJoint};

use crate::doc::{Edit, JointEditorDoc, scope_options};
use crate::model::JointRow;

impl JointEditorDoc {
    /// A junta da regra `rule` como o «Make the parts» a faria: a de fábrica (o comprimento = a folga
    /// do desenho, a que as peças de fábrica terão) com as edições do rascunho por cima.
    #[must_use]
    pub fn draft_joint(&self, rule: u16) -> Option<PhysicsJoint> {
        let t = self.template();
        let r = t.rule(rule)?;
        let counts = self.draft_counts();
        let gap = crate::paint_stage::pairs(t, rule, &counts)
            .first()
            .and_then(|&n| crate::stage::rule_ends(t, &counts, rule, n))
            .map_or(0.0, |(a, b)| libm::hypotf(b[0] - a[0], b[1] - a[1]));
        let mut j = r.joint(gap);
        for &(_, e) in self.draft_edits.iter().filter(|(k, _)| *k == rule) {
            j = joint_with_edit(j, e).unwrap_or(j);
        }
        Some(j)
    }

    /// As juntas que a aba mostra: as da montagem aberta, ou uma por regra do gabarito por fazer
    /// (`bits == 0`: ainda não existem no mundo).
    #[must_use]
    pub fn rows(&self) -> Vec<JointRow> {
        if let Some(a) = self.assembly() {
            return a.joints.clone();
        }
        (0..self.template().joints.len())
            .filter_map(|i| u16::try_from(i).ok())
            .filter_map(|rule| {
                let joint = self.draft_joint(rule)?;
                Some(JointRow {
                    bits: 0,
                    rule,
                    joint,
                    info: Some(info_of(&joint)),
                })
            })
            .collect()
    }

    /// As juntas a que um número se aplica agora.
    #[must_use]
    pub fn targets(&self) -> Vec<JointRow> {
        let opts = scope_options(self.template());
        let i = match self.assembly() {
            Some(a) => self.scope_index(a, &opts),
            None => self.scope.get(&0).copied().unwrap_or(0),
        };
        let Some(o) = opts.get(i) else {
            return Vec::new();
        };
        let mut rows = self.rows();
        rows.retain(|j| o.rules.contains(&j.rule));
        rows
    }

    /// Escreve `edit` nas juntas `rows`: no mundo (um passo de desfazer) ou, por fazer, no rascunho.
    pub(crate) fn write(&mut self, rows: &[JointRow], edit: JointFieldEdit) {
        if self.assembly().is_some() {
            let joints: Vec<u64> = rows.iter().map(|j| j.bits).collect();
            if !joints.is_empty() {
                self.pending.push(Edit::Joints { joints, edit });
            }
        } else {
            self.draft_edits.extend(rows.iter().map(|j| (j.rule, edit)));
        }
    }
}

/// Os números de uma junta na unidade do cartão, pelas portas do `build_joint_info` (sem o mundo: nomes,
/// identidade e conta-gotas ficam vazios — a junta por fazer não tem nenhum deles).
fn info_of(j: &PhysicsJoint) -> InspectorJointInfo {
    InspectorJointInfo {
        kind_tag: tag_of(j.kind),
        one_chain: j.one_chain,
        soft: j.soft,
        limits_enabled: j.limits_enabled,
        limit_min_ui: limit_out(j.kind, j.limit_min),
        limit_max_ui: limit_out(j.kind, j.limit_max),
        motor_enabled: j.motor_enabled,
        motor_mode_tag: u8::from(j.motor_mode == MotorMode::Position),
        motor_speed_ui: j.motor_no_cartao(j.motor_speed),
        motor_target_ui: j.motor_no_cartao(j.motor_target),
        motor_max_force: j.motor_max_force,
        rest_length: j.rest_length,
        stiffness: j.stiffness,
        damping: j.damping,
        max_length: j.max_length,
        break_enabled: j.break_enabled,
        break_force: j.break_force,
        break_torque: j.break_torque,
        breaks_on_torque: j.breaks_on_torque(),
        active: j.active,
        collide_connected: j.collide_connected,
        ..InspectorJointInfo::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_joint_assembly::joint_info::kind_of;

    /// ⭐ O rascunho mostra a junta pela MESMA porta que o mundo: a de fábrica, e a edição por cima só na
    /// regra escolhida. Mutação que sangra: aplicar a edição a todas as regras.
    #[test]
    fn o_rascunho_mostra_a_de_fabrica_e_a_edicao_so_na_regra() {
        let mut d = JointEditorDoc::default();
        let t = ph2d_joint_templates::by_name(ph2d_joint_templates::JointKind::Pin, "Bridge")
            .expect("ponte");
        d.choose_template(t.id.0);
        assert_eq!(
            kind_of(d.rows()[1].info.as_ref().expect("i").kind_tag),
            t.kind
        );
        assert!(!d.rows()[1].joint.break_enabled, "de fábrica");
        let plank: Vec<JointRow> = d.rows().into_iter().filter(|j| j.rule == 1).collect();
        d.write(&plank, JointFieldEdit::BreakEnabled(true));
        let rows = d.rows();
        assert!(rows[1].joint.break_enabled);
        assert!(!rows[0].joint.break_enabled && !rows[2].joint.break_enabled);
        d.new_draft();
        assert!(!d.rows()[1].joint.break_enabled, "«New» começa de fábrica");
    }
}
