//! **O que se COPIA de um joint, e o que NUNCA se copia** — a porta única do
//! copy/paste da §12 (o *Copy properties to…* do Unreal PhAT).
//!
//! # A linha do corte já estava desenhada na tela
//!
//! O painel da §12 se divide em duas metades, e o doc do `joint_pair_rows` diz
//! exatamente onde: *"aqui **entre QUAIS DOIS** isto está, e como eles se
//! tratam; lá **o que a restrição FAZ**"*. Uma propriedade é a segunda metade.
//! Colar a primeira não seria copiar propriedades — seria duplicar o joint em
//! cima de outro, e o resultado teria dois objetos apontando para os mesmos dois
//! corpos.
//!
//! # ⚠️ O TIPO viaja, e é o que torna a colagem SEGURA
//!
//! Metade destes campos não tem unidade própria: ela é função do tipo.
//! `limit_min`/`limit_max` são **radianos** num Pin e **metros** num Slider
//! ([`JointKind::limits_in_metres`](super::JointKind::limits_in_metres));
//! `motor_speed`/`motor_target` são rad/s num eixo e m/s num trilho ou num
//! guincho ([`motor_in_metres`](super::JointKind::motor_in_metres)).
//!
//! Um paste que trouxesse os NÚMEROS sem o TIPO seria uma reinterpretação de
//! unidade em silêncio — o ±0,785 rad de um Pin virando ±0,785 **metro** de
//! curso —, que é o mesmo defeito que o braço `Kind` de `joint_with_edit` existe
//! para evitar quando o artista troca o tipo à mão. Aqui a cura é oposta e mais
//! simples: os números e a unidade deles chegam JUNTOS, então **não há re-seed a
//! fazer**. É por isso que o paste não pode ser escrito como *"um `Kind(tag)`
//! seguido de quinze edições de campo"*: o `Kind` re-semearia limites, motor e
//! mola para os defaults do tipo novo, e as quinze edições seguintes estariam
//! desfazendo esse re-seed campo a campo.
//!
//! # E a ÂNCORA é re-semeada quando o tipo muda
//!
//! [`PhysicsJoint::anchored`](super::PhysicsJoint::anchored) é do ALVO — a
//! colocação é dele —, com uma exceção: a *política* de âncora é função do tipo
//! (um Pin/Weld ancora as duas pontas no pivô compartilhado, uma Spring/Rope
//! ancora o lado B no centro do próprio corpo). Colar um tipo diferente é, por
//! isso, uma reposição da ponta B, e o sentinela cai para `false` para pedir ao
//! reconcile UMA re-derivação — o 5º sítio de autoria, ao lado do dot de canvas,
//! do commit de Position, do re-pick e do braço `Kind`.
//!
//! # Um campo novo QUEBRA A COMPILAÇÃO aqui
//!
//! O corpo desestrutura a fonte **exaustivamente**. Isso não é estilo: é a
//! resposta estrutural ao *"enumeração apodrece"*. Uma lista escrita à mão do
//! que viaja envelhece em silêncio nos dois sentidos — um campo de afinação novo
//! que não viaja faz o paste ficar incompleto sem ninguém notar, e um campo de
//! IDENTIDADE novo que viaja faz o joint apontar para o corpo errado. Com o
//! destructuring, o campo dezoito **não compila** até alguém dizer de que lado
//! ele está.

use super::PhysicsJoint;

impl PhysicsJoint {
    /// **Este joint, com as propriedades daquele.**
    ///
    /// A identidade (os dois corpos), a colocação (as âncoras body-local) e o
    /// interruptor `active` permanecem os DESTE joint; tudo o que descreve o que
    /// a restrição faz vem de `source`. Docs do módulo para o porquê de cada
    /// lado da linha.
    ///
    /// Pura e sem clamp: quem chama passa por `clamped()` como toda outra porta
    /// de autoria, para que uma única resposta decida o que é um número válido.
    #[must_use]
    pub fn with_properties_of(&self, source: &Self) -> Self {
        // ⚠️ EXAUSTIVO de propósito — docs do módulo. Um campo novo em
        // `PhysicsJoint` faz esta linha falhar a compilação até ser classificado.
        let Self {
            // ── IDENTIDADE — *entre quais dois isto está*. Nunca viaja: colar
            //    isto seria duplicar o joint, não copiar as propriedades dele.
            body_a: _,
            body_b: _,
            // ── COLOCAÇÃO — *onde nesses corpos*. Nunca viaja: um offset medido
            //    no corpo da fonte não significa nada no corpo do alvo, que pode
            //    ter outro tamanho e outra forma.
            local_a: _,
            local_b: _,
            anchored: _,
            // ── O INTERRUPTOR DO EXPERIMENTO. Não viaja, e a razão é o USO:
            //    `active` é o *"experimente o rig sem este aqui"*, uma
            //    investigação sobre UM joint, enquanto a razão de existir do
            //    paste é agir sobre MUITOS. Uma colagem que re-ligasse o joint
            //    que você acabou de desligar destruiria o experimento que estava
            //    correndo enquanto você copiava.
            active: _,
            // ── O QUE A RESTRIÇÃO FAZ — a metade que viaja, inteira.
            kind,
            limits_enabled,
            limit_min,
            limit_max,
            motor_enabled,
            motor_speed,
            motor_max_force,
            rest_length,
            stiffness,
            damping,
            max_length,
            motor_mode,
            motor_target,
            break_enabled,
            break_force,
            break_torque,
            collide_connected,
        } = *source;

        Self {
            body_a: self.body_a,
            body_b: self.body_b,
            local_a: self.local_a,
            local_b: self.local_b,
            // A política de âncora é função do TIPO: mudou o tipo, a ponta B foi
            // reposicionada e o reconcile precisa re-derivar (docs do módulo).
            anchored: self.anchored && kind == self.kind,
            active: self.active,
            kind,
            limits_enabled,
            limit_min,
            limit_max,
            motor_enabled,
            motor_speed,
            motor_max_force,
            rest_length,
            stiffness,
            damping,
            max_length,
            motor_mode,
            motor_target,
            break_enabled,
            break_force,
            break_torque,
            collide_connected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{JointKind, MotorMode, PhysicsJoint};

    /// Uma fonte com TODO campo longe do default, para que qualquer campo que
    /// deixe de viajar apareça como uma diferença e não como uma coincidência.
    fn tuned_source() -> PhysicsJoint {
        PhysicsJoint {
            body_a: 111,
            body_b: 222,
            kind: JointKind::Slider,
            limits_enabled: true,
            limit_min: -0.25,
            limit_max: 0.75,
            motor_enabled: true,
            motor_speed: 3.5,
            motor_max_force: 42.0,
            rest_length: 2.5,
            stiffness: 400.0,
            damping: 12.0,
            max_length: 3.25,
            local_a: [0.5, -0.5],
            local_b: [1.5, 2.5],
            anchored: true,
            motor_mode: MotorMode::Position,
            motor_target: 1.75,
            break_enabled: true,
            break_force: 900.0,
            break_torque: 55.0,
            active: false,
            collide_connected: true,
        }
    }

    /// O alvo: outros corpos, outra âncora, outro tipo, e TODA afinação no
    /// default — o oposto da fonte em cada campo que viaja.
    fn plain_target() -> PhysicsJoint {
        PhysicsJoint {
            body_a: 777,
            body_b: 888,
            local_a: [-3.0, -4.0],
            local_b: [-5.0, -6.0],
            anchored: true,
            ..PhysicsJoint::default()
        }
    }

    /// **A metade que viaja chega inteira** — e o oráculo não é uma lista de
    /// campos escrita à mão (que envelheceria junto com a função sob teste): é a
    /// FONTE com a identidade e a colocação do alvo enxertadas de volta. Se
    /// qualquer campo de afinação deixar de viajar, os dois structs diferem.
    #[test]
    fn every_property_travels_and_nothing_else_does() {
        let src = tuned_source();
        let dst = plain_target();
        let out = dst.with_properties_of(&src);

        let expected = PhysicsJoint {
            body_a: dst.body_a,
            body_b: dst.body_b,
            local_a: dst.local_a,
            local_b: dst.local_b,
            // O tipo mudou (Pin -> Slider), logo a âncora pede re-derivação.
            anchored: false,
            active: dst.active,
            ..src
        };
        assert_eq!(out, expected);
    }

    /// **A identidade e a colocação são do ALVO** — dito como asserção direta,
    /// porque é a metade cujo modo de falha é um joint apontando para o corpo
    /// errado (silencioso, e pior que o paste incompleto).
    #[test]
    fn the_pair_and_the_anchors_stay_the_targets_own() {
        let out = plain_target().with_properties_of(&tuned_source());
        assert_eq!(out.body_a, 777);
        assert_eq!(out.body_b, 888);
        assert_eq!(out.local_a, [-3.0, -4.0]);
        assert_eq!(out.local_b, [-5.0, -6.0]);
    }

    /// **`active` é do alvo** — o experimento que estava correndo sobrevive à
    /// colagem (docs do módulo). A fonte está inativa e o alvo ativo: se ele
    /// viajasse, o alvo apagaria.
    #[test]
    fn pasting_does_not_switch_the_target_off() {
        let src = tuned_source(); // active: false
        assert!(!src.active);
        let dst = PhysicsJoint {
            active: true,
            ..plain_target()
        };
        assert!(dst.with_properties_of(&src).active);
        // E a recíproca: um alvo que o artista desligou continua desligado.
        let dst_off = PhysicsJoint {
            active: false,
            ..plain_target()
        };
        assert!(
            !dst_off
                .with_properties_of(&PhysicsJoint {
                    active: true,
                    ..src
                })
                .active
        );
    }

    /// **O tipo VIAJA, e por isso os números têm unidade.** O gate mede a
    /// consequência em vez do campo: as duas metades chegam juntas, então o
    /// curso de 0,75 do Slider chega sob um tipo que o lê em metros.
    #[test]
    fn the_kind_travels_with_the_numbers_that_wear_its_unit() {
        let out = plain_target().with_properties_of(&tuned_source());
        assert_eq!(out.kind, JointKind::Slider);
        assert!(out.kind.limits_in_metres());
        assert!((out.limit_max - 0.75).abs() < 1e-6);
        assert!((out.motor_speed - 3.5).abs() < 1e-6);
        assert_eq!(out.motor_mode, MotorMode::Position);
    }

    /// **Tipo IGUAL não mexe no sentinela de âncora** — a metade que impede o
    /// paste de pedir uma re-derivação que ninguém precisa, que jogaria fora as
    /// âncoras que o artista posicionou à mão.
    #[test]
    fn pasting_onto_the_same_kind_keeps_the_anchors_seeded() {
        let src = PhysicsJoint {
            kind: JointKind::Pin,
            limit_max: 1.25,
            ..tuned_source()
        };
        let dst = plain_target(); // Pin, anchored: true
        assert_eq!(dst.kind, JointKind::Pin);
        let out = dst.with_properties_of(&src);
        assert!(
            out.anchored,
            "same kind = same anchor policy: nothing to re-derive"
        );
        assert!((out.limit_max - 1.25).abs() < 1e-6);
    }

    /// **Colar em si mesmo é a identidade.** Não é trivialidade: é o controle
    /// que prova que a função não perde nada pelo caminho — se algum campo de
    /// afinação caísse, ele apareceria aqui como uma diferença contra o próprio
    /// struct de onde veio.
    #[test]
    fn pasting_a_joint_onto_itself_changes_nothing() {
        let j = tuned_source();
        assert_eq!(j.with_properties_of(&j), j);
    }
}
