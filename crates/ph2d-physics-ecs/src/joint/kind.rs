//! **As perguntas que um TIPO de joint responde** — o `JointKind` e a família de
//! portas que decide, por tipo, o que existe e em que unidade.
//!
//! Irmão de `joint.rs`, separado dele quando os dois juntos passaram do cap de
//! 700 LOC, e o corte é o que o arquivo pai já vinha desenhando: aqui *que
//! espécie de restrição é esta, e o que ela tem*; lá *o estado que ESTE joint
//! guarda*. Cada porta aqui existe porque duas perguntas parecidas se separaram
//! ao chegar um tipo novo (`has_length` do `!is_hinge` com o Weld, `has_limits`
//! do `is_hinge` com o Slider), e o histórico dessas separações mora nos
//! doc-comments — é ele que impede a próxima de ser colapsada de volta.

use serde::{Deserialize, Serialize};

/// Which constraint this joint applies. **Append-only** — postcard encodes the
/// discriminant positionally, so new kinds go at the END or every saved
/// project reads its joints as the wrong type.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum JointKind {
    /// The two bodies share a point and turn freely about it. The hinge, the
    /// pendulum's pivot, the ragdoll's elbow.
    #[default]
    Pin,
    /// A damped spring between the anchors — the distance is a target, not a
    /// law.
    Spring,
    /// The anchors may come as close as they like but never further apart than
    /// [`PhysicsJoint::max_length`].
    Rope,
    /// **Weld** — the two bodies are locked rigidly together at the anchor:
    /// no relative motion, no rotation. A pin with its rotation frozen. Useful
    /// for compound bodies and (later) breakable structures. It shares a point
    /// like a pin but has no motor, no limits, no length.
    Weld,
    /// **Slider** — the bodies may only slide along ONE axis and never rotate
    /// relative to each other. The elevator, the sliding door, the piston.
    ///
    /// The **axis is the joint entity's own `Transform::rotation`** — no new
    /// field, and no second place to keep it. That is the model Godot and
    /// Unreal use, and it is the one this component already implies: the
    /// entity's `Transform` is where a joint's *placement* lives (its
    /// translation is the anchor), so its rotation is where a placement's
    /// direction lives. It also means the axis is authorable on day one, in the
    /// Inspector's own Rotation field, with zero widgets added.
    ///
    /// Like a Pin it has **limits** — but they are a STROKE, in metres. See
    /// [`JointKind::limits_in_metres`].
    Slider,
    /// **Rod** — a rigid bar: the anchors are held at
    /// [`PhysicsJoint::max_length`] and both ends turn freely. The connecting
    /// rod, the tie bar, the strut of a four-bar linkage.
    ///
    /// The gap it fills is narrow enough to be easy to miss: a Weld holds the
    /// distance but *also* freezes the rotation, a Rope holds only the ceiling
    /// (it goes slack), and a Spring is deliberately bouncy.
    ///
    /// ⚠️ **The engine builds it as a stiff motor, not as a hard constraint,
    /// and the measurement is in `ph2d_physics::JointKind::Rod`** — rapier's
    /// coupled linear *limit* is unilateral (`// FIXME: handle min limit too.`
    /// in its own solver), so `[d, d]` cannot hold. That is why it reuses
    /// `max_length` and offers no stiffness of its own: **one authored number,
    /// the length**.
    Rod,
    /// **Wheel** — um cubo que ao mesmo tempo **gira** e **cavalga uma
    /// suspensão**. A roda do carro, o rodízio do carrinho, o rolete suspenso.
    ///
    /// É o primeiro tipo desta lista a deixar **DOIS** graus de liberdade
    /// livres, e é isso que ele é: o cubo desliza ao longo de um eixo (o curso
    /// da suspensão, com mola e amortecimento) e gira em torno de si para
    /// sempre (o giro, opcionalmente motorizado). Todo o resto do kit deixa um
    /// ou nenhum.
    ///
    /// Reusa os campos que já existem, e nenhum é emprestado por analogia: a
    /// mola da suspensão são [`PhysicsJoint::stiffness`]/[`PhysicsJoint::damping`]
    /// (o mesmo par de uma Spring, porque é a mesma coisa física), o curso são
    /// os `limit_min`/`limit_max` em METROS (o mesmo par de um Slider), e o
    /// motor é o do giro, em **graus** — que é o que faz este tipo separar duas
    /// perguntas que até aqui tinham uma resposta só (ver
    /// [`JointKind::motor_in_metres`]).
    Wheel,
}

/// **Em que campo do componente o comprimento de um tipo MORA.**
///
/// Existe porque a pergunta tinha **quatro** respostas independentes espalhadas
/// pelo repo — o `JointView` que o desenho lê, o gesto de criar-arrastando, a
/// escrita do anel de comprimento e a cena de smoke — e a terceira delas
/// APODRECEU no dia em que o [`JointKind::Rod`] chegou: ela enumerava
/// `Spring | Rope` com um `_ => return` cujo comentário afirmava *"Pin/Weld não
/// têm anel"*. O anel do rod era publicado, o grip pegava, o arrasto abria, e a
/// escrita voltava **em silêncio**.
///
/// ⚠️ Um `match` exaustivo, não uma tabela: o sétimo tipo **não compila** até
/// dizer onde o comprimento dele mora — que é a única forma de a resposta não
/// nascer errada de novo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LengthField {
    /// [`PhysicsJoint::rest_length`] — o alvo de uma mola.
    Rest,
    /// [`PhysicsJoint::max_length`] — o teto de uma corda, o comprimento de uma
    /// barra. Os dois compartilham o campo porque compartilham o **número**: é
    /// a distância que o vínculo governa, e o tipo é quem diz se ela é um teto
    /// ou uma igualdade.
    Max,
}

impl JointKind {
    /// **Todos os tipos, na ordem do discriminante.**
    ///
    /// Existe para um gate poder percorrer o conjunto INTEIRO em vez de uma
    /// lista escrita à mão — e a diferença não é teórica: o gate das portas de
    /// unidade (`the_unit_doors_disagree_about_a_rope_and_that_is_the_point`)
    /// listava cinco tipos, o [`JointKind::Rod`] chegou, **ninguém o
    /// acrescentou**, e ele passou uma wave inteira sem ter as respostas dele
    /// conferidas por nada.
    ///
    /// ⚠️ **Esta lista também é escrita à mão — o que a mantém honesta é o
    /// round-trip de TAG na shell** (`every_kind_chip_the_panel_offers_round_trips_
    /// to_a_distinct_kind`), que é exaustivo por `match` e falha a compilar com
    /// um tipo novo. Aqui há um gate exigindo que ela tenha um elemento por
    /// discriminante e nenhum repetido.
    pub const ALL: [JointKind; 7] = [
        JointKind::Pin,
        JointKind::Spring,
        JointKind::Rope,
        JointKind::Weld,
        JointKind::Slider,
        JointKind::Rod,
        JointKind::Wheel,
    ];

    /// Ver [`LengthField`]. `None` para quem não tem comprimento nenhum.
    ///
    /// **A porta única** — quem DESENHA, quem CRIA arrastando e quem ESCREVE o
    /// anel perguntam a ela; três respostas independentes é como duas delas
    /// passam a discordar sobre qual campo o artista acabou de editar.
    #[must_use]
    pub fn length_field(self) -> Option<LengthField> {
        match self {
            JointKind::Spring => Some(LengthField::Rest),
            JointKind::Rope | JointKind::Rod => Some(LengthField::Max),
            // ⚠️ Um Wheel **não** tem comprimento: a distância cubo↔chassi é
            // onde o artista o montou, e o que a governa é o par mola/curso.
            JointKind::Pin | JointKind::Weld | JointKind::Slider | JointKind::Wheel => None,
        }
    }

    /// Does this kind swing about a pivot? Only a [`JointKind::Pin`] does.
    ///
    /// ⚠️ **No longer the same question as *"does it have a motor?"*** — that is
    /// [`Self::has_motor`], and it grew a Slider and a Rope in W-J6. This one is
    /// still asked wherever the *angular* nature is the point.
    pub fn is_hinge(self) -> bool {
        matches!(self, JointKind::Pin)
    }

    /// **Is this kind's free degree of freedom a TRANSLATION?** A Slider slides
    /// along its axis; a Rope's is the distance between the anchors. A Pin's is
    /// an angle.
    ///
    /// ⚠️ **Já foi o fato único por trás de todo "metros ou radianos?" deste
    /// arquivo, e o [`JointKind::Wheel`] REFUTOU a premissa.** A frase supunha
    /// que um joint deixa livre **um** grau de liberdade, e uma roda deixa dois:
    /// o curso da suspensão (metros) e o giro (graus). Com isso
    /// [`Self::limits_in_metres`] e [`Self::motor_in_metres`] deixaram de poder
    /// derivar da mesma resposta — a roda diz **sim** para o primeiro e **não**
    /// para o segundo — e os dois viraram `match` exaustivo, cada um declarando
    /// a unidade DA PERGUNTA DELE.
    ///
    /// Ela sobrevive porque a pergunta ainda tem sentido próprio (*este tipo
    /// desliza?*) e um gate a pina por tipo; o que ela não pode mais é ser a
    /// definição de outra coisa. Um Wheel responde **true**: ele desliza — só
    /// não é *só* isso que ele faz.
    pub fn translates(self) -> bool {
        matches!(self, JointKind::Slider | JointKind::Rope | JointKind::Wheel)
    }

    /// Does this kind have a limit RANGE the artist tunes? A Pin (an angular
    /// range) and a Slider (a stroke) do.
    ///
    /// ⚠️ **Split out of `is_hinge` when the Slider arrived**, for the same
    /// reason `has_length` had to be split out of `!is_hinge` when the Weld did:
    /// *"is it a hinge?"* and *"does it have limits?"* had the same answer while
    /// the Pin was the only limited kind, and one of the two questions was
    /// always going to grow a second member. Collapsed, a Slider would either
    /// get a motor it has no model for or lose the limits that make it a rail.
    ///
    /// ⚠️ Um **Wheel** entrou aqui e o limite dele é o CURSO da suspensão — o
    /// batente de compressão e o de extensão. Ele é o primeiro cujo limite não
    /// governa o único grau de liberdade que ele tem, e sim **um dos dois**: o
    /// giro de uma roda não se limita (é isso que uma roda faz).
    pub fn has_limits(self) -> bool {
        matches!(self, JointKind::Pin | JointKind::Slider | JointKind::Wheel)
    }

    /// **Can this kind be DRIVEN?** A Pin's hinge, a Slider's rail and a Rope's
    /// distance (a winch) all have a free degree of freedom a motor can push
    /// along. A Spring and a Weld do not.
    ///
    /// **One door.** The Inspector asks it to decide whether to paint the Motor
    /// card, and the bridge asks it before handing a motor to the solver — two
    /// answers to *"does this joint have a motor?"* is how a knob comes to be
    /// painted for a kind that ignores it.
    ///
    /// ⚠️ **A Spring is excluded for a MECHANICAL reason, not a UI one:** rapier
    /// models a spring *as* a motor on the same axis, so a second motor there
    /// would overwrite the stiffness and damping the artist authored and turn the
    /// spring into a rate-driven rod. The wrapper states the same exclusion in
    /// `ph2d_physics::motor_axis`, which is what the solver actually reads.
    ///
    /// ⚠️ Um **Wheel** também, e nele o motor é o do GIRO — a tração. Ele
    /// carrega uma mola *e* um motor sem que colidam porque estão em **eixos
    /// diferentes** (`LinX` e `AngX`), que é precisamente o que a exclusão da
    /// Spring acima diz não ser possível no mesmo eixo.
    pub fn has_motor(self) -> bool {
        matches!(
            self,
            JointKind::Pin | JointKind::Slider | JointKind::Rope | JointKind::Wheel
        )
    }

    /// Are this kind's limits a distance rather than an angle?
    ///
    /// ⚠️ **`limit_min`/`limit_max` carry the unit of the KIND** — radians for a
    /// Pin, metres for a Slider — which is exactly how rapier models it (one
    /// `limits` field, belonging to whichever degree of freedom the joint left
    /// free). One door so the Inspector's label, its conversion, and the
    /// bridge's hand-off to the solver cannot disagree about what the number
    /// means; and the kind-change re-seeds the range, so `±0.785` rad never
    /// silently becomes `±0.785` m.
    ///
    /// ⚠️ **Era derivada de [`Self::translates`], e o [`JointKind::Wheel`] a
    /// separou** — ele limita uma translação (o curso) e motoriza uma rotação (o
    /// giro), então nenhuma das duas unidades é *a* unidade do tipo. Agora cada
    /// pergunta declara a sua num `match` exaustivo: o sétimo tipo **não
    /// compila** até dizer em que unidade o limite DELE está, que é a única
    /// forma de a resposta não nascer errada. (A Rope já mostrava metade disso:
    /// ela translada e não tem faixa de limite nenhuma.)
    pub fn limits_in_metres(self) -> bool {
        match self {
            JointKind::Slider | JointKind::Wheel => true,
            JointKind::Pin
            | JointKind::Spring
            | JointKind::Rope
            | JointKind::Weld
            | JointKind::Rod => false,
        }
    }

    /// Is this kind's MOTOR linear rather than angular — metres and metres per
    /// second rather than degrees and degrees per second?
    ///
    /// ⚠️ **Not the same question as [`Self::limits_in_metres`], and a Rope is
    /// the case that proves it:** a Rope has no limit range at all (so that one
    /// says `false`) and its motor is a winch reeling a distance (so this one
    /// says `true`). Sharing one door would have given the winch a target in
    /// degrees.
    /// **Can a break threshold on this kind ever fire on TORQUE?**
    ///
    /// Only where the angular axis is limited or motorised. rapier reports the
    /// reaction of a limited or motorised axis exactly (measured: 4.9050 and
    /// 4.9049 against `m·g·r` of 4.905) and reports **nothing** for a locked one
    /// — a Weld cantilever holds 4.905 N·m and reads `0.0000`. A Rope and a
    /// Spring leave the angular axis free, so their torque is genuinely zero.
    ///
    /// ⚠️ **Um [`JointKind::Wheel`] entrou por MEDIÇÃO, e o caso dele é o mais
    /// fino da lista:** o eixo angular de uma roda é livre *enquanto o motor
    /// está desligado* e motorizado quando está ligado — medido no mesmo carro,
    /// **0.0000 N·m sem motor e 0.5125 com** (a força fica em 29-35 N nos dois,
    /// que é o peso que a suspensão carrega). Quem manda é o estado em que a row
    /// pode ser alcançada: negar a row deixaria a tração ser o único jeito de
    /// arrancar um eixo sem que exista o número que a segura.
    ///
    /// Asked by the panel to decide whether to paint the row, and by nothing
    /// else: the wrapper compares whatever threshold it is handed, so a torque
    /// on a Weld is not *wrong*, it is **unreachable** — which is worse, because
    /// it looks like a control.
    #[must_use]
    pub fn breaks_on_torque(self) -> bool {
        matches!(self, JointKind::Pin | JointKind::Wheel)
    }

    ///
    /// ⚠️ **E o [`JointKind::Wheel`] é o caso que a separou de vez:** ele
    /// translada (a suspensão) e o motor dele é **angular** (a tração). Uma
    /// resposta derivada de `translates` daria a um carro uma tração em metros
    /// por segundo — o gêmeo exato do bug que a Rope evitou do outro lado.
    pub fn motor_in_metres(self) -> bool {
        match self {
            JointKind::Slider | JointKind::Rope => true,
            JointKind::Pin
            | JointKind::Spring
            | JointKind::Weld
            | JointKind::Rod
            | JointKind::Wheel => false,
        }
    }

    /// Does this kind have a length the artist tunes? Spring, Rope and Rod do —
    /// a Pin's length is zero (the anchors coincide) and a Weld's is too (it is
    /// rigid). **Not** `!is_hinge()`: a Weld is not a hinge but still has no
    /// length, so the two questions had to stop sharing an answer.
    ///
    /// ⚠️ A Rod answers `true` here and `false` to every other door, and that
    /// combination *is* what a rod is: one number, no limits, no motor. It also
    /// makes [`Self::shares_a_point`] answer `false` for it — correctly, since a
    /// rod's two ends are meant to start apart.
    pub fn has_length(self) -> bool {
        self.length_field().is_some()
    }

    /// Do the two bodies share one point (a Pin, a Weld or a Slider), rather than
    /// have two separate ends (a Spring or a Rope)? The anchor policy reads this:
    /// a shared-point joint anchors both bodies at the same world point, a
    /// two-ended one anchors body B at its own centre.
    ///
    /// A Slider shares a point **and that point is the zero of its stroke** —
    /// rapier measures the prismatic displacement from where the two anchors
    /// coincide, so authoring them apart would offset the whole range by the
    /// gap. It falls out of `!has_length()` correctly, but it is stated here
    /// rather than left to fall out by accident.
    ///
    /// ⚠️ **Um Wheel também, e pela mesma razão elevada a duas:** o cubo é o
    /// ponto, e é dele que a suspensão mede o curso **e** que a roda gira. É
    /// isso que faz a altura de marcha ser *onde o artista montou o carro* — a
    /// mola tem alvo zero, então ela quer o cubo exatamente ali, e o que se vê
    /// depois é o quanto o peso a afunda.
    pub fn shares_a_point(self) -> bool {
        !self.has_length()
    }
}
