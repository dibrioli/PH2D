//! **As perguntas que uma INSTÂNCIA responde e o TIPO não** — o par de portas
//! que o `JointKind::Custom` obrigou a existir.
//!
//! Nos oito presets toda pergunta sobre o que um joint FAZ é respondida pelo
//! tipo: um Pin gira, um Slider desliza, uma solda trava. O `Custom` escolhe a
//! configuração, e com ela duas respostas deixaram de ser função do tipo — a
//! UNIDADE do motor (o eixo é autorado) e a existência de uma reação ANGULAR
//! (ela depende de o eixo de rotação estar restringido).
//!
//! ⚠️ **Módulo irmão do `joint.rs` pelo cap de 700 LOC**, e o corte é este
//! assunto: as duas moram juntas porque falham juntas — quem perguntar ao
//! `JointKind` em vez de aqui rotula um número na unidade errada, ou oferece um
//! limiar que nunca dispara.

use super::{JointKind, PhysicsJoint};

impl PhysicsJoint {
    /// This joint with every number forced back into a range the solver can
    /// use. **The door a loaded project file comes through.**
    ///
    /// The Inspector already sanitises what it writes, but a component is
    /// `serde` and travels in the project file, so the Inspector is not the
    /// only way values arrive — and this is the last place before rapier.
    /// `PhysicsSettings::clamped` exists for exactly this reason on the world
    /// side; without the twin, joints were the one loader-facing surface in
    /// the line that did not clamp.
    ///
    /// Measured on the unclamped version: `stiffness = NaN` drove the body's
    /// pose to `(NaN, NaN)` within 120 steps, and `readback` then wrote NaN
    /// straight into the entity's `Transform` — where it flows into the
    /// cross-OS determinism hash. `max_length = -1` behaved as an unrelated
    /// length, silently.
    ///
    /// ⚠️ **Inverted limits are a WELD, not a wide hinge.** `limit_min` and
    /// `limit_max` are authored independently, so `min > max` is one keystroke
    /// away — and rapier, handed `[min, max]` that way, froze the plank solid
    /// (measured: `rot 0.000` after 180 steps). A hinge the artist believes is
    /// limited to ±45° being a weld is the kind of wrong that has no symptom
    /// to search for, so the pair is ordered here.
    /// **ESTE joint pode partir sob TORQUE?** — a pergunta que o painel faz para
    /// oferecer a row e a ponte faz para entregar o limiar.
    ///
    /// [`JointKind::breaks_on_torque`] responde pelo TIPO; esta responde pela
    /// INSTÂNCIA, e a diferença é uma solda mole. rapier publica a reação de um
    /// eixo angular *limitado ou motorizado* e **nada** de um TRAVADO — e o
    /// `soft` é exatamente o que troca um pelo outro. Medido na mesma viga em
    /// balanço, com os mesmos defaults:
    ///
    /// | solda | força | torque |
    /// |---|---|---|
    /// | rígida | 1,9620 N | **0,0000 N·m** |
    /// | mole | 2,0044 N | **0,9619 N·m** |
    ///
    /// ⚠️ **É o caso do [`JointKind::Wheel`] outra vez** (o eixo dele é livre com
    /// o motor desligado e motorizado com ele ligado, 0,0000 contra 0,5125): quem
    /// manda é *o estado em que a row pode ser alcançada*, e negá-la deixaria a
    /// torção ser o único jeito de arrancar uma solda mole sem que exista o
    /// número que a segura. O preço, igual ao do Wheel, é que a caixa de Break
    /// de uma solda RÍGIDA mostra um limiar de torque que nunca dispara.
    #[must_use]
    /// ⚠️ **E o [`JointKind::Custom`] é o terceiro caso, pela mesma lei**: o eixo
    /// angular dele publica reação se estiver TRAVADO ou LIMITADO, e nada se
    /// estiver livre — a pergunta é ao EIXO que o artista configurou, não ao
    /// tipo. `JointKind::breaks_on_torque` devolve `false` para ele, que é o
    /// default conservador; quem sabe a resposta é esta porta.
    pub fn breaks_on_torque(&self) -> bool {
        self.kind.breaks_on_torque()
            || (self.kind.can_be_soft() && self.soft)
            || (self.kind == JointKind::Custom && self.custom.constrains_rotation())
    }

    /// **O motor deste joint é medido em METROS?** — a porta da INSTÂNCIA.
    ///
    /// [`JointKind::motor_in_metres`] responde pelo TIPO e está certa para os
    /// sete presets, onde o grau de liberdade livre é uma propriedade do tipo.
    /// Num [`JointKind::Custom`] ele é **escolhido**, então metro-ou-radiano é
    /// escolhido junto — e a porta do tipo passa a ser um default, não a
    /// resposta.
    ///
    /// ⚠️ **É a UI que tem de perguntar a esta**, e o preço de perguntar à outra
    /// não é cosmético: o alvo de um servo é convertido na fronteira do painel
    /// (graus ↔ radianos), então um Custom com o motor no eixo X seria rotulado
    /// em graus e teria o número dividido por 57,3 antes de chegar ao solver, que
    /// o lê em metros. O artista digitaria 90 e a peça andaria 1,57 m.
    #[must_use]
    pub fn motor_in_metres(&self) -> bool {
        if self.kind == JointKind::Custom {
            return self.custom.motor_axis.in_metres();
        }
        self.kind.motor_in_metres()
    }
}
