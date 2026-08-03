//! [`PropKind`] — the **general** enumeration of animatable properties the
//! timeline document can bind, plus its opaque-target mapping.
//!
//! `ph2d-anim` keeps [`AnimTarget`] meaningless (HR-8). `PropKind` is the
//! document-level *authority* on what a target means, and each per-system
//! resolver (sprite first, vector/painter/node later) interprets the subset it
//! knows. The sprite resolver lives in [`crate::sprite`] as [`SpriteProp`];
//! `PropKind::TranslationX ..= ScaleY` share their [`AnimTarget`] ids with it so
//! a track authored either way names the same target.

use ph2d_anim::AnimTarget;
use serde::{Deserialize, Serialize};

use crate::sprite::SpriteProp;

/// A property a [`crate::TargetBinding`] can drive, across every animatable
/// system. The `u64` discriminant is the opaque [`AnimTarget`] id (HR-8), so it
/// is a **frozen wire value** — only append new variants, never renumber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u64)]
pub enum PropKind {
    /// Sprite local translation X (meters).
    TranslationX = 0,
    /// Sprite local translation Y (meters).
    TranslationY = 1,
    /// Sprite rotation (radians, CCW from +X).
    Rotation = 2,
    /// Sprite local scale X.
    ScaleX = 3,
    /// Sprite local scale Y.
    ScaleY = 4,
    /// Sprite opacity — the alpha channel of `Sprite.tint` (`[0, 1]`).
    Opacity = 5,
    /// **Time remap** (W5, AE model): the timeline's own meta-property — a
    /// keyed curve mapping playhead time → the SOURCE time this entity's other
    /// tracks sample at (seconds → seconds). Slope < 1 is slow motion, > 1
    /// speeds up, flat freezes, negative slope reverses. Never writes a scene
    /// property (`as_sprite_transform` is `None`; the apply consumes it as the
    /// entity's clock) and never auto-keys (it is not in [`PropKind::ALL`],
    /// the pose list). Appended — the discriminant is a frozen wire value.
    TimeRemap = 6,
    /// **Morph** — the `t` of a `ph2d_ecs::VecMorph`: where along the path between its two source
    /// shapes the morphed shape sits (`0` = A, `1` = B). The Vector line's animatable channel, and
    /// the reason the Morph object exists at all: without a keyed `t` it is a slider, not
    /// animation.
    ///
    /// Outside [`PropKind::ALL`], like [`PropKind::TimeRemap`]: `ALL` is the sprite POSE, and `t`
    /// is not part of any sprite's pose. The artist creates the track from the "+ Track" list.
    ///
    /// ⚠️ **Mas fora do `ALL` NÃO quer dizer fora do auto-key** — a redação anterior desta cerca
    /// deixava isso ambíguo e o smoke do Enio (2026-07-28) cobrou: *"nem autokey funcionou para
    /// morph"*. `ALL` é a pose; quem o auto-key varre é [`PropKind::AUTOKEYED`], e Morph está lá.
    /// O próprio [`PropKind::Position`] já provava que o array não é a fronteira (ele também fica
    /// fora do `ALL` e é autokeyado, por um ramo de geometria 2D). Appended — the discriminant is
    /// a frozen wire value.
    Morph = 7,
    /// **Position** — the object's place on the canvas as ONE channel, following an
    /// authored trajectory ([ADR-0141]): the After Effects model, where *Separate
    /// Dimensions precludes having Spatial Keyframes* and the two are therefore
    /// **modes**, not a mode plus a flag. Binding this kind IS choosing the path
    /// mode; [`PropKind::TranslationX`]/`Y` remain the separate-axes mode, and the
    /// conversion between them is an explicit, named operation.
    ///
    /// **The value of its track is DISTANCE ALONG THE PATH**, in world units — a
    /// plain scalar, which is what lets the graph editor, weighted tangents, the
    /// speed graph and roving keep working on it untouched. The trajectory itself
    /// lives on the binding ([`crate::TargetBinding::path`]), and key `i` of the
    /// track pairs with anchor `i` of that path.
    ///
    /// Outside [`PropKind::ALL`], like [`PropKind::TimeRemap`] and
    /// [`PropKind::Morph`]: `ALL` is the *separate-axes* sprite pose the auto-key
    /// samples ([`crate::PoseSample`] is exactly that array's shape), and a Position
    /// track is the alternative to two of its entries, never a seventh member of it.
    /// Appended — the discriminant is a frozen wire value.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    Position = 8,
    /// **O ALVO de um servo** — `PhysicsJoint::motor_target`: o ângulo (dobradiça)
    /// ou a posição (trilho) para onde o motor puxa. O canal com que se ANIMA uma
    /// máquina em vez de a assistir.
    ///
    /// ⚠️ **A unidade depende do TIPO do joint** (`JointKind::motor_in_metres`): é
    /// radiano num Pin e metro num Slider. Por isso [`PropKind::fit_channel`] o
    /// trata como escalar linear e **não** como ângulo, ao contrário de
    /// [`PropKind::Rotation`] — desenrolar um sawtooth de ±2π num número que às
    /// vezes é comprimento seria consertar o que não está quebrado e quebrar o que
    /// está certo. E um alvo de servo não CHEGA como sawtooth de qualquer forma:
    /// ele é digitado ou keyado, nunca derivado de um `atan2` de gizmo, que é o
    /// mecanismo inteiro pelo qual a rotação precisa do unwrap.
    ///
    /// Fora de [`PropKind::ALL`] e de [`PropKind::AUTOKEYED`] pelo mesmo motivo do
    /// `TimeRemap`: `ALL` é a pose de um sprite, e o auto-key diffa a pose que o
    /// artista ARRASTA. Um parâmetro de joint é digitado no Inspector, e a porta
    /// que o keya é o `K` sobre a track (a mesma de todo canal que não é pose).
    /// Appended — the discriminant is a frozen wire value.
    JointMotorTarget = 9,
    /// **A TAXA de um motor** — `PhysicsJoint::motor_speed`: a velocidade que a
    /// dobradiça motorizada, o trilho ou o guincho perseguem. A esteira que
    /// acelera, o guindaste que recolhe mais rápido perto do fim.
    ///
    /// Irmã de [`PropKind::JointMotorTarget`] e **não** a mesma coisa: o alvo diz
    /// ONDE parar, a taxa diz QUÃO RÁPIDO ir, e o modo do motor
    /// (`MotorMode::Position`/`Velocity`) decide qual dos dois o solver lê. Manter
    /// os dois canais é o que deixa o artista animar o número que o motor DELE de
    /// fato usa, em vez de descobrir que keyou o outro.
    /// Appended — the discriminant is a frozen wire value.
    JointMotorSpeed = 10,
    /// **O comprimento que uma mola QUER** — `PhysicsJoint::rest_length`: o
    /// músculo que contrai, o pistão que estende, a suspensão que baixa.
    ///
    /// É o canal que anima uma mola sem tocar na rigidez dela — animar
    /// `stiffness` seria animar o CARÁTER do mecanismo, e é o *"tweak to correct
    /// its behavior"* que a pesquisa do plano 02 pegou o Unity documentando como
    /// knob-fudge. Appended — the discriminant is a frozen wire value.
    JointRestLength = 11,
    /// **O comprimento que um vínculo GOVERNA** — `PhysicsJoint::max_length`: o
    /// teto de uma corda, o tamanho de uma barra. A corda que é recolhida, o
    /// mastro que telescopa.
    ///
    /// Os dois tipos partilham o campo porque partilham o NÚMERO (`LengthField`),
    /// e é o tipo que diz se ele é um teto ou uma igualdade — o que também vale
    /// aqui: keyar este canal encurta uma corda ou encolhe uma barra, e a
    /// diferença é do joint, não da track.
    /// Appended — the discriminant is a frozen wire value.
    JointMaxLength = 12,
}

impl PropKind {
    /// The six SCENE properties of a sprite's POSE, in authoring order.
    /// [`PropKind::TimeRemap`] is deliberately absent: it is the timeline's
    /// own clock, not a scene value (the "+ Track" list adds it separately).
    ///
    /// ⚠️ **Isto é a POSE, não a lista do auto-key** — para essa, veja
    /// [`PropKind::AUTOKEYED`]. As duas coincidiram até 2026-07-28, e é por isso que
    /// os doc-comments antigos (aqui e no `Morph`) diziam que `PoseSample` tem a forma
    /// *deste* array: hoje ele tem a forma do outro.
    pub const ALL: [PropKind; 6] = [
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::Rotation,
        PropKind::ScaleX,
        PropKind::ScaleY,
        PropKind::Opacity,
    ];

    /// **Os canais ESCALARES que o auto-key diffa** — a pose mais o `t` do Morph, e a
    /// forma exata de [`crate::PoseSample`].
    ///
    /// A pose de um sprite (`ALL`) responde *"onde este objeto está"*; esta lista responde
    /// *"que números o artista pode mexer e esperar que o auto-key grave"*. Elas eram a
    /// mesma até o Morph chegar, e tratá-las como uma só era o que impedia o auto-key de
    /// gravar o canal cuja razão de existir é ser animado (`Morph`: *"sem um `t` keyado é
    /// um slider, não animação"*).
    ///
    /// ⚠️ [`PropKind::Position`] **não** entra aqui, e não é esquecimento: capturá-la é
    /// ACRESCENTAR UMA ÂNCORA à trajetória, o que reescreve a distância de toda key
    /// posterior (ADR-0141 §2) — geometria 2D, não um escalar. Ela tem o ramo próprio do
    /// auto-key (`AutokeyPlan::path_key`), que é o precedente de que estar fora do `ALL`
    /// nunca significou estar fora do auto-key. [`PropKind::TimeRemap`] também não: é o
    /// relógio, e não tem valor de cena para amostrar.
    pub const AUTOKEYED: [PropKind; 7] = [
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::Rotation,
        PropKind::ScaleX,
        PropKind::ScaleY,
        PropKind::Opacity,
        PropKind::Morph,
    ];

    /// The opaque [`AnimTarget`] a track uses to drive this property.
    #[must_use]
    pub const fn target(self) -> AnimTarget {
        AnimTarget::new(self as u64)
    }

    /// Recover a kind from its opaque target id, if it names a known property.
    #[must_use]
    pub const fn from_target(target: AnimTarget) -> Option<PropKind> {
        match target.get() {
            0 => Some(PropKind::TranslationX),
            1 => Some(PropKind::TranslationY),
            2 => Some(PropKind::Rotation),
            3 => Some(PropKind::ScaleX),
            4 => Some(PropKind::ScaleY),
            5 => Some(PropKind::Opacity),
            6 => Some(PropKind::TimeRemap),
            7 => Some(PropKind::Morph),
            8 => Some(PropKind::Position),
            9 => Some(PropKind::JointMotorTarget),
            10 => Some(PropKind::JointMotorSpeed),
            11 => Some(PropKind::JointRestLength),
            12 => Some(PropKind::JointMaxLength),
            _ => None,
        }
    }

    /// The i18n key suffix for this property's label (`panel.timeline.prop.*`).
    /// Presentation strings are resolved by the panel (HR-15); this is the
    /// stable key, never a display string.
    #[must_use]
    pub const fn i18n_suffix(self) -> &'static str {
        match self {
            PropKind::TranslationX => "translation_x",
            PropKind::TranslationY => "translation_y",
            PropKind::Rotation => "rotation",
            PropKind::ScaleX => "scale_x",
            PropKind::ScaleY => "scale_y",
            PropKind::Opacity => "opacity",
            PropKind::TimeRemap => "time",
            PropKind::Morph => "morph",
            PropKind::Position => "position",
            PropKind::JointMotorTarget => "motor_target",
            PropKind::JointMotorSpeed => "motor_speed",
            PropKind::JointRestLength => "rest_length",
            PropKind::JointMaxLength => "max_length",
        }
    }

    /// Resolve the tail of a **prop-link** identifier (`Name.<tail>`) to a kind —
    /// the ADR-0144 expression syntax. Artist-friendly aliases (`x`/`rot`/`sx`),
    /// case-insensitive. `TimeRemap` is deliberately absent (it is the timeline's
    /// meta-clock, not a scene value to read); a bare `time` is the playhead, so
    /// it is resolved by the pass's `Bindings`, not here.
    /// ⚠️ **[`PropKind::i18n_suffix`] is one of the accepted spellings**, and it was not:
    /// a link typed `Ball.translation_x` — the name this very enum gives the property,
    /// and the one the panel's own label is keyed by — parsed cleanly and resolved to
    /// **0.0**, teleporting the follower to the origin with nothing said. The two tables
    /// are now checked against each other by
    /// `every_prop_answers_to_the_name_the_panel_shows_it_under`, so a kind added with a
    /// label but no spelling cannot ship.
    #[must_use]
    pub fn from_expr_name(name: &str) -> Option<PropKind> {
        match name.to_ascii_lowercase().as_str() {
            "x" | "translationx" | "translation_x" | "tx" => Some(PropKind::TranslationX),
            "y" | "translationy" | "translation_y" | "ty" => Some(PropKind::TranslationY),
            "rotation" | "rot" | "r" => Some(PropKind::Rotation),
            "scalex" | "scale_x" | "sx" => Some(PropKind::ScaleX),
            "scaley" | "scale_y" | "sy" => Some(PropKind::ScaleY),
            "opacity" | "alpha" | "a" => Some(PropKind::Opacity),
            "position" | "pos" | "p" => Some(PropKind::Position),
            "morph" | "m" => Some(PropKind::Morph),
            "motor_target" | "motortarget" => Some(PropKind::JointMotorTarget),
            "motor_speed" | "motorspeed" => Some(PropKind::JointMotorSpeed),
            "rest_length" | "restlength" => Some(PropKind::JointRestLength),
            "max_length" | "maxlength" => Some(PropKind::JointMaxLength),
            _ => None,
        }
    }

    /// The sprite-transform resolver's view of this kind, if it is one of the
    /// five `Transform` properties. `Opacity` returns `None` — it resolves to
    /// `Sprite.tint[3]`, not a `Transform` field (see [`crate::apply`]).
    #[must_use]
    pub const fn as_sprite_transform(self) -> Option<SpriteProp> {
        match self {
            PropKind::TranslationX => Some(SpriteProp::TranslationX),
            PropKind::TranslationY => Some(SpriteProp::TranslationY),
            PropKind::Rotation => Some(SpriteProp::Rotation),
            PropKind::ScaleX => Some(SpriteProp::ScaleX),
            PropKind::ScaleY => Some(SpriteProp::ScaleY),
            // Position drives TWO Transform fields through a trajectory, so it is
            // not one `SpriteProp` — `crate::apply_path` is its resolver.
            PropKind::Opacity
            | PropKind::TimeRemap
            | PropKind::Morph
            | PropKind::Position
            // Um parâmetro de joint não é uma pose de sprite: ele mora no
            // `PhysicsJoint` da entidade-JOINT, e o resolver dele é o
            // `crate::apply_prop`.
            | PropKind::JointMotorTarget
            | PropKind::JointMotorSpeed
            | PropKind::JointRestLength
            | PropKind::JointMaxLength => None,
        }
    }

    /// What a record-cleanup fit must know about this channel beyond its numbers
    /// ([`ph2d_anim::FitChannel`]) — the property's SEMANTICS, which live here
    /// with the property and not in the fit (which stays a pure numeric routine).
    ///
    /// [`PropKind::Rotation`] is **angular**: the rotate gizmo writes it through
    /// `atan2`, so a recorded spin arrives as a ±2π sawtooth and must be unwrapped
    /// or a two-turn spin reconstructs as a net rotation of zero.
    /// [`PropKind::Opacity`] is **bounded** to `[0, 1]` — the alpha of
    /// `Sprite.tint`; a least-squares cubic through a fade that settles on 1.0
    /// otherwise overshoots past it, which the graph editor draws.
    ///
    /// [`PropKind::Morph`] is **bounded** to `[0, 1]` for the same reason as `Opacity`, and it
    /// is not a coincidence: both are a fraction with two hard ends. A least-squares cubic through
    /// a morph that settles on B overshoots past `t = 1`, and the motor clamps there — so the
    /// graph editor would draw a curve that leaves the shape standing still, which reads as a bug
    /// in the easing rather than in the fit.
    ///
    /// The rest are unbounded scalars. [`PropKind::TimeRemap`] never records (it
    /// is not in [`PropKind::ALL`], the auto-key pose list), so its value here is
    /// only the safe default.
    #[must_use]
    pub const fn fit_channel(self) -> ph2d_anim::FitChannel {
        match self {
            PropKind::Rotation => ph2d_anim::FitChannel::ANGLE,
            PropKind::Opacity | PropKind::Morph => ph2d_anim::FitChannel::bounded(0.0, 1.0),
            PropKind::TranslationX
            | PropKind::TranslationY
            | PropKind::ScaleX
            | PropKind::ScaleY
            | PropKind::TimeRemap
            // Distance along a path. Bounded in principle by the path's length — but
            // that bound MOVES when an anchor does, and a fit that clamped to a
            // stale one would pin a recorded pose to the wrong end.
            | PropKind::Position
            // ⚠️ **`LINEAR` e não `ANGLE` no alvo do servo, de propósito** — a
            // mesma unidade é radiano numa dobradiça e metro num trilho
            // (`JointKind::motor_in_metres`), e o unwrap existe para um
            // sawtooth que só um `atan2` de gizmo produz. Os comprimentos e a
            // taxa são escalares sem fronteira nenhuma.
            | PropKind::JointMotorTarget
            | PropKind::JointMotorSpeed
            | PropKind::JointRestLength
            | PropKind::JointMaxLength => ph2d_anim::FitChannel::LINEAR,
        }
    }

    /// How an **additive** clip lane combines with what is under it (ADR-0115).
    ///
    /// This is the distinction that Blender got wrong first and had to invent
    /// `COMBINE` to fix ([T47035]): "additive" cannot mean "add the number".
    /// Adding two scale clips of 1.0 gives **2.0** — double size, where the
    /// honest answer is *no change*. A channel whose neutral value is 1 and whose
    /// meaning is proportional composes by **ratio**, not by sum.
    ///
    /// [T47035]: https://developer.blender.org/T47035
    #[must_use]
    pub const fn algebra(self) -> Algebra {
        match self {
            // Position and angle: neutral 0, additive means "displace by".
            PropKind::TranslationX | PropKind::TranslationY | PropKind::Rotation => Algebra::Sum,
            // Scale and alpha: neutral 1, additive means "scale by".
            PropKind::ScaleX | PropKind::ScaleY | PropKind::Opacity => Algebra::Ratio,
            // Never stacked: it IS the clock (ADR-0115 R6). The value is a safe
            // default, not a semantic claim.
            PropKind::TimeRemap => Algebra::Sum,
            // The morph `t` is a POSITION along a path, not a proportion: neutral 0, and an
            // additive lane means "advance further along it". By ratio, two lanes at 0.3 and 0.5
            // would give 0.15 — less progress than either, which is not a thing anyone meant.
            PropKind::Morph => Algebra::Sum,
            // Distance travelled: neutral 0, and an additive lane means "go further
            // along it" — the Morph argument exactly, for the same kind of quantity.
            //
            // Blending DISTANCES is also what keeps a crossfade ON the trajectory:
            // halfway between "3 m along" and "7 m along" is "5 m along", still on
            // the curve, where blending the two POINTS would cut the corner off it.
            PropKind::Position => Algebra::Sum,
            // Alvo, taxa e comprimento: neutro 0, e uma lane aditiva quer dizer
            // *"mais um tanto"* — o argumento do Morph, para a mesma espécie de
            // grandeza. Por RAZÃO, dois comprimentos de 0,5 m dariam 0,25 m, que
            // é menos que qualquer um dos dois e não é coisa que alguém quis.
            PropKind::JointMotorTarget
            | PropKind::JointMotorSpeed
            | PropKind::JointRestLength
            | PropKind::JointMaxLength => Algebra::Sum,
        }
    }
}

/// How a channel composes with another value of itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algebra {
    /// Neutral 0. Additive contribution is a **difference** (`v - base`), applied
    /// by addition.
    Sum,
    /// Neutral 1. Additive contribution is a **ratio** (`v / base`), applied by
    /// multiplication. Scale, and alpha.
    Ratio,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_roundtrips_for_every_kind() {
        for k in PropKind::ALL {
            assert_eq!(PropKind::from_target(k.target()), Some(k));
        }
        assert_eq!(PropKind::from_target(AnimTarget::new(999)), None);
    }

    #[test]
    fn sprite_transform_ids_match_sprite_prop() {
        // The four+one transform kinds share their opaque id with SpriteProp,
        // so a track authored via either names the same target.
        for k in PropKind::ALL {
            if let Some(sp) = k.as_sprite_transform() {
                assert_eq!(sp.target(), k.target());
            }
        }
        assert!(PropKind::Opacity.as_sprite_transform().is_none());
    }
}
