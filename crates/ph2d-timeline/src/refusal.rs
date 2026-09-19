//! Why a key was not written where the animator asked for it (ADR-0115 R9).
//!
//! Under a clip stack, "key it here" can genuinely have **no answer**. The
//! document's job is to refuse — writing a key that quietly moves the object is
//! the one outcome that is never acceptable — but a refusal the animator cannot
//! see is indistinguishable from a bug: they drag, the object snaps back, and
//! nothing says why.
//!
//! So the refusal is a **value**, not an absence. It travels from the evaluator
//! (which is the only place that knows the reason) to the shell (which is the
//! only place that can speak), and the shell says it out loud.

/// The three ways "key it here" can fail to name a place in the clip being edited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyRefusal {
    /// The clip being edited is **not playing** at this instant: no strip of it
    /// covers the playhead, so "here" is nowhere in it.
    NotPlaying,
    /// It is playing **more than once**: "here" names two places in the clip, and
    /// picking one silently would drop the key somewhere the animator never looked.
    PlaysTwice,
    /// A lane above **owns the channel**: the blend is insensitive to what this
    /// clip stores (an `Override` lane at full weight), so no value in it produces
    /// the pose that was asked for. The affine solve is degenerate — `A ~ 0`.
    Overridden,
    /// The channel is driven by a **formula** the key cannot invert (ADR-0152 W5): a
    /// value-independent one (`wiggle`, `time*10` — the solve is degenerate in the
    /// stored `value`), or a non-linear one (`value*value` — the affine check fails).
    /// Distinct from `Overridden`: the fix is the FORMULA, not the lane stack — clean
    /// or rewrite it, never "delete a lane". `value + g(time)` (the affine idiom) keys
    /// and pre-compensates instead of refusing.
    ExpressionDriven,
    /// The pose is a **motion-path ANCHOR**, and a trajectory belongs to its CLIP — but
    /// the panel is showing a STACK (Arrange, or a container's interior), where the pose
    /// on screen is a blend of strips and the active clip is not what the animator picked.
    ///
    /// The other refusals are about a value the clip cannot *express*; this one is about a
    /// clip the animator is not *looking at*. Writing the anchor anyway would edit the
    /// geometry of a clip the tab does not even name, and rewrite the distance that every
    /// key in it holds (`rewrite_path_key_values`) — a large, invisible edit from a gesture
    /// that looks local. It is the authoring half of the overlay's `keys_tab` rule
    /// (`motion_path_overlay::active_path`), and the SAME boolean decides both.
    PathNeedsKeysTab,
}

/// Why a container could not be placed inside another (ADR-0133 §4).
///
/// **Refusing out loud is the whole point, and the research is why.** Every product that
/// ships nesting checks for cycles at the moment the link is authored — none checks at
/// runtime — but the two that refuse *silently* (After Effects greys the cursor; Animate just
/// will not drop) are also the two whose exact error text could not be found anywhere,
/// because there is none. Godot and Unity name it, and that is the side to be on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NestRefusal {
    /// A container placed **inside itself** — the trivial cycle.
    SelfNest,
    /// The link would close a longer loop: the container being placed already reaches the
    /// host, so playing it would require playing it.
    WouldCycle,
    /// The host or the source does not exist.
    NoSuchContainer,
}

impl NestRefusal {
    /// ⭐ **A CHAVE da linha que o animador lê**, resolvida por quem a mostra (a shell, num toast).
    ///
    /// ⛔ Este motor **não** depende da `ph2d-i18n`: uma folha de lei não precisa do catálogo, e um
    /// `message()` que devolvesse o inglês por `tr_em(Ingles, …)` é indistinguível do caminho certo
    /// num processo em inglês.
    #[must_use]
    pub fn message_key(self) -> &'static str {
        match self {
            Self::SelfNest => "timeline.nest.self",
            Self::WouldCycle => "timeline.nest.cycle",
            Self::NoSuchContainer => "timeline.nest.missing",
        }
    }
}

impl KeyRefusal {
    /// ⭐ **A CHAVE da linha que o animador lê**, resolvida por quem a mostra (a shell, num toast).
    ///
    /// ⚠️ **A lei da frase fica onde estava**, e ela é o motivo de cada uma existir: *cada uma diz o
    /// que aconteceu E o que a pilha está a fazer, porque «can't key here» sem razão é só um pouco
    /// melhor que silêncio*. O texto mudou de sítio — para a tabela —, não de intenção.
    #[must_use]
    pub fn message_key(self) -> &'static str {
        match self {
            Self::NotPlaying => "timeline.key.not_playing",
            Self::PlaysTwice => "timeline.key.plays_twice",
            Self::Overridden => "timeline.key.overridden",
            Self::ExpressionDriven => "timeline.key.expression_driven",
            Self::PathNeedsKeysTab => "timeline.key.path_needs_keys_tab",
        }
    }
}
