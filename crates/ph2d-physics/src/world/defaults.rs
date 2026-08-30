//! [`BodyDefaults`] — the world-level values every new body is born with.
//!
//! ## Why these are a WORLD setting even though rapier stores them per-body
//!
//! rapier's [`IntegrationParameters`] has no global damping and no global sleep
//! thresholds — both live on the rigid body ([`RigidBodyActivation`], and the
//! builder's `linear_damping`/`angular_damping`). Measured, not assumed: the
//! parameter struct's public fields are `dt`, the contact/joint spring terms,
//! the solver iteration counts and the CCD knobs, and nothing else.
//!
//! Exposing them as world settings anyway is the idiom every 2D engine ships —
//! Godot's *Project Settings → Physics → 2D* carries `default_linear_damp`,
//! `default_angular_damp`, `sleep_threshold_linear`, `sleep_threshold_angular`
//! and `time_before_sleep`; Unity's *Physics 2D* carries the sleep tolerances.
//! An artist asking "how much drag does this world have?" is asking one
//! question, and answering it body-by-body would be a worse product.
//!
//! ⚠️ **So today there is exactly ONE door for each of these numbers**, and that
//! is the whole reason this type is safe. If a per-body damping override is ever
//! added, it MUST arrive with a combine mode (Godot's `damp_mode`
//! Combine/Replace) — a second field that silently wins over this one is the
//! "two doors to the same question" failure this repo keeps paying for.
//!
//! [`IntegrationParameters`]: rapier2d::dynamics::IntegrationParameters
//! [`RigidBodyActivation`]: rapier2d::dynamics::RigidBodyActivation

use rapier2d::dynamics::{RigidBodyActivation, RigidBodySet};

/// World-level defaults stamped onto every body [`super::PhysicsWorld::spawn_body`]
/// creates, and re-stamped onto every live body by
/// [`super::PhysicsWorld::set_body_defaults`].
///
/// Plain `Copy` data — no rapier types cross this boundary, so the ECS bridge
/// can build one without depending on rapier (the same rule [`super::BodyDesc`]
/// follows).
///
/// ⛔ **Até 2026-08-29 todo campo era o que a rapier usa, e a subida para a 0.35 partiu essa
/// identidade** — dois dos cinco mudaram lá. O valor que o produto usa é agora
/// [`BodyDefaults::ours`], e [`BodyDefaults::rapier`] fica como o **oráculo** contra o qual a
/// divergência é medida e datada. Ver o doc de [`BodyDefaults::ours`] para o mecanismo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyDefaults {
    /// Linear drag. `0.0` (rapier's own) = vacuum; higher slows translation.
    pub linear_damping: f32,
    /// Angular drag. `0.0` (rapier's own) = spins forever.
    pub angular_damping: f32,
    /// Linear speed below which a body may fall asleep (normalized — rapier
    /// multiplies it by `IntegrationParameters::length_unit`).
    /// **Nosso: `0,4`** · rapier 0.28: `0,4` · **rapier 0.35: `0,05`** (ver [`BodyDefaults::ours`]).
    pub sleep_linear_threshold: f32,
    /// Angular speed below which a body may fall asleep. **`0,5` nas duas** — inalterado.
    pub sleep_angular_threshold: f32,
    /// How long a body must stay under both thresholds before sleeping, in seconds.
    /// **Nosso: `2,0`** · rapier 0.28: `2,0` · **rapier 0.35: `0,5`** (ver [`BodyDefaults::ours`]).
    pub time_until_sleep: f32,
}

impl BodyDefaults {
    /// rapier's own defaults, read from rapier rather than transcribed — a
    /// literal here would silently drift the day the dependency bumps, and the
    /// whole point of this struct is that leaving it alone changes nothing.
    pub fn rapier() -> Self {
        Self {
            // `RigidBodyBuilder` starts both damping factors at zero (its
            // `Default` leaves the fields unset).
            linear_damping: 0.0,
            angular_damping: 0.0,
            sleep_linear_threshold: RigidBodyActivation::default_normalized_linear_threshold(),
            sleep_angular_threshold: RigidBodyActivation::default_angular_threshold(),
            time_until_sleep: RigidBodyActivation::default_time_until_sleep(),
        }
    }

    /// ⭐⭐ **Os valores que o PRODUTO usa — e por que eles deixaram de ser os da rapier.**
    ///
    /// A [`BodyDefaults::rapier`] acima existe por uma razão escrita: *«ler da rapier em vez de
    /// transcrever, porque um literal derivaria em silêncio no dia em que a dependência subisse»*.
    /// O desenho estava certo — e a subida para a `rapier2d` 0.35 mostrou que ele tem a deriva
    /// **do outro lado**:
    ///
    /// | | rapier 0.28 | rapier 0.35 |
    /// |---|---|---|
    /// | `sleep_linear_threshold` | `0,4` | **`0,05`** — o corpo tem de estar **8× mais parado** |
    /// | `time_until_sleep` | `2,0 s` | **`0,5 s`** — dorme **4× mais cedo** |
    ///
    /// ⛔ **E estes valores são PERSISTIDOS.** O `PhysicsSettings` do bridge é `serde` e viaja no
    /// `.ph2dproj`, com o `Default` a ler daqui. ⇒ depois da subida, uma cena **nova** nasceria com
    /// `0,05 / 0,5 s` e um projecto **gravado** continuaria em `0,4 / 2,0 s`: **duas cenas
    /// idênticas com tatos diferentes conforme a data em que foram salvas**, e nada na tela a
    /// explicá-lo.
    ///
    /// ⚠️ *O modo de falha que o desenho da `rapier()` evita — «um projecto que nunca abriu o
    /// painel a simular diferente de antes» — é exactamente o que acontece quando é a RAPIER que
    /// muda.* Ler de uma fonte que se move não é o oposto de transcrever: é a mesma deriva com
    /// outro dono.
    ///
    /// ⇒ A partir daqui os valores são **nossos**, escritos, e a `rapier()` fica como oráculo.
    /// Uma peça que hoje treme dois segundos antes de assentar continua a tremer dois segundos.
    #[must_use]
    pub fn ours() -> Self {
        Self {
            // Estes dois continuam a ser os da rapier, e continuam a ser zero nas duas versões:
            // um corpo sem arrasto autorado não é uma escolha nossa, é a ausência de uma.
            linear_damping: 0.0,
            angular_damping: 0.0,
            // ⚠️ Os três de adormecer são NOSSOS desde 2026-08-29 — ver o doc acima.
            sleep_linear_threshold: 0.4,
            sleep_angular_threshold: 0.5,
            time_until_sleep: 2.0,
        }
    }

    /// Stamp these values onto every body currently in `bodies`.
    ///
    /// Called when the settings CHANGE, because a value the artist just typed
    /// has to describe the world they are looking at — not only the bodies that
    /// happen to be spawned after it.
    ///
    /// ⚠️ **Sleeping bodies are woken**, and that is not politeness. A sleeping
    /// body is not integrated at all, so a threshold that now says it should be
    /// awake would never be consulted — the setting would appear to do nothing
    /// until something else happened to disturb the body. Waking a settled stack
    /// is cheap and self-correcting: the bodies are already at rest, so they
    /// re-settle on the next step.
    pub(crate) fn apply_to_all(&self, bodies: &mut RigidBodySet) {
        // Iteration order does not matter here: this writes the SAME values to
        // every body and reads nothing, so the result is order-independent
        // (unlike spawning, which assigns handles).
        for (_, body) in bodies.iter_mut() {
            self.apply_to(body);
            body.wake_up(true);
        }
    }

    /// Stamp these values onto one body.
    pub(crate) fn apply_to(&self, body: &mut rapier2d::dynamics::RigidBody) {
        body.set_linear_damping(self.linear_damping);
        body.set_angular_damping(self.angular_damping);
        let act = body.activation_mut();
        act.normalized_linear_threshold = self.sleep_linear_threshold;
        act.angular_threshold = self.sleep_angular_threshold;
        act.time_until_sleep = self.time_until_sleep;
    }
}

impl Default for BodyDefaults {
    fn default() -> Self {
        Self::rapier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapier2d::dynamics::RigidBodyBuilder;

    /// **`BodyDefaults::rapier()` é o que a rapier de facto entrega, e o oráculo é RAPIER — não nós.**
    ///
    /// This lives here, as a unit test, for one reason: an integration test
    /// can only reach `BodyDefaults::rapier()`, so every comparison it could
    /// make is this function against itself. That version of this gate was
    /// written first and stayed **green** while `linear_damping` was mutated
    /// to `0.05` — the same trap the audio line documented twice (a fast path
    /// measured against itself). The only oracle that can fail is a body
    /// rapier built and nobody configured.
    ///
    /// ⚠️ **O que ele defende MUDOU em 2026-08-29.** Ele afirmava *«os nossos defaults são os da
    /// rapier»*; hoje afirma que a **leitura** continua fiel — que a `rapier()` diz a verdade sobre
    /// a dependência. Quem o produto usa é a [`BodyDefaults::ours`], e a divergência entre as duas
    /// é medida pelo gate irmão abaixo.
    #[test]
    fn the_defaults_are_the_ones_rapier_hands_out_untouched() {
        let untouched = RigidBodyBuilder::dynamic().build();
        let ours = BodyDefaults::rapier();

        assert_eq!(
            ours.linear_damping,
            untouched.linear_damping(),
            "our default linear damping is not the one rapier gives an \
             unconfigured body"
        );
        assert_eq!(
            ours.angular_damping,
            untouched.angular_damping(),
            "our default angular damping is not the one rapier gives an \
             unconfigured body"
        );
        let act = untouched.activation();
        assert_eq!(
            ours.sleep_linear_threshold, act.normalized_linear_threshold,
            "our default sleep speed threshold is not rapier's"
        );
        assert_eq!(
            ours.sleep_angular_threshold, act.angular_threshold,
            "our default sleep spin threshold is not rapier's"
        );
        assert_eq!(
            ours.time_until_sleep, act.time_until_sleep,
            "our default sleep delay is not rapier's"
        );
    }

    /// ⭐⭐ **A DIVERGÊNCIA entre o que a rapier entrega e o que o produto usa — medida e datada.**
    ///
    /// A [`BodyDefaults::ours`] deixou de ser a `rapier()` na subida para a 0.35, porque dois
    /// valores de adormecer mudaram lá e eles são **persistidos** no `.ph2dproj` (o mecanismo está
    /// no doc dela). Este gate existe para que essa divergência seja **um facto verificado**, e não
    /// uma nota que envelhece:
    ///
    /// - o que **é nosso** tem de estar escrito, com o valor exacto;
    /// - o que **não divergiu** tem de continuar a não divergir — se a rapier um dia mudar o
    ///   amortecimento por omissão, este gate acorda e alguém decide, em vez de a mudança entrar
    ///   com uma subida de versão.
    ///
    /// ⚠️ *Uma escolha que não é comparada com a alternativa deixa de ser uma escolha ao fim de
    /// uma versão.*
    #[test]
    fn our_defaults_diverge_from_rapiers_only_where_we_chose_to() {
        let r = BodyDefaults::rapier();
        let o = BodyDefaults::ours();

        // Onde NÃO divergimos — e tem de continuar assim.
        assert_eq!(
            (o.linear_damping, o.angular_damping),
            (r.linear_damping, r.angular_damping),
            "o amortecimento por omissao deixou de ser o da rapier. Se foi ela que mudou, isto e' \
             uma DECISAO de produto (um corpo passa a travar sozinho, ou a nunca parar de rodar) e \
             tem de ser tomada, nao herdada. Se fomos nos, escreva a razao ao lado do valor."
        );

        // Onde divergimos DE PROPÓSITO, com os números escritos.
        assert_eq!(
            (o.sleep_linear_threshold, o.time_until_sleep),
            (0.4, 2.0),
            "os limiares de adormecer sao NOSSOS desde 2026-08-29 e valem 0,4 e 2,0 s. Eles estao \
             gravados em todo `.ph2dproj`, entao muda-los aqui faz uma cena nova comportar-se \
             diferente de um projecto salvo -- o defeito que a `ours()` existe para impedir."
        );
        assert_eq!(
            o.sleep_angular_threshold, r.sleep_angular_threshold,
            "o limiar ANGULAR nunca divergiu (0,5 nas duas versoes da rapier). Se ele divergir \
             agora, ou a rapier mudou ou alguem o fixou sem dizer porque^."
        );
    }
}
