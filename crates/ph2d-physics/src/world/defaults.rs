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

/// ⭐ **O valor que [`BodyDefaults::sleep_angular_threshold`] carrega quando adormecer está
/// DESLIGADO** — e não é um número escolhido: é o que a própria `rapier2d` escreve em
/// `RigidBodyActivation::cannot_sleep()` (`rigid_body_components.rs`, `-1.0`).
///
/// # O mecanismo, medido na `rapier2d` 0.35.3
///
/// `RigidBodyActivation::update_energy` decide, para um corpo `Dynamic`:
///
/// ```text
/// let angular_ok = if max_extent > 0.0 {            // tem collider  ⇒ toda entidade real
///     self.angular_threshold >= 0.0 && sq_angvel < FRAC_PI_2 * FRAC_PI_2
/// } else {                                          // sem collider
///     sq_angvel < self.angular_threshold * self.angular_threshold.abs()
/// };
/// can_sleep = angular_ok && drift * 0.5 < linear_threshold * dt;
/// ```
///
/// ⇒ com o limiar **negativo**, `angular_ok` é `false` em **todos** os ramos, `can_sleep` é
/// `false`, e o temporizador `time_since_can_sleep` é zerado a cada passo: o corpo **nunca dorme**.
/// Com o limiar **não-negativo** o corpo pode dormir, e a barra angular é um `π/2` **fixo** dentro
/// da rapier — a nossa magnitude não é lida.
///
/// ⛔ **Não escolha outro negativo por gosto.** Qualquer `< 0` produz o mesmo comportamento hoje;
/// este é o que a referência usa, e é o que faz um `.ph2dproj` nosso ler igual a um corpo que a
/// rapier própria configurou.
pub const SLEEP_SPIN_DISABLED: f32 = -1.0;

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
    /// **Nosso: `0,05` (MEDIDO)** · rapier 0.28: `0,4` · rapier 0.35: `0,05` — ver
    /// [`BodyDefaults::ours`], secção «o limiar de sono»: fixar o `0,4` da 0.28 fazia corpos
    /// adormecerem **tortos**, a meio do assentamento.
    pub sleep_linear_threshold: f32,
    /// Angular speed below which a body may fall asleep. **`0,5` nas três versões da rapier.**
    ///
    /// ⛔⛔ **O NÚMERO nunca mudou; o CONSUMO dele mudou, e hoje este knob está MORTO para toda
    /// entidade real.** Na `rapier2d` 0.35 (`rigid_body_components.rs::update_energy`), um corpo
    /// **com collider** (`max_extent > 0` — isto é, todos os que existem numa cena) passa por:
    ///
    /// ```text
    /// self.angular_threshold >= 0.0 && sq_angvel < FRAC_PI_2 * FRAC_PI_2
    /// ```
    ///
    /// ⇒ do nosso valor lê-se **apenas o sinal**; a barra a sério é um `π/2` **fixo** dentro da
    /// rapier. O doc do campo deles di-lo: *«this raw threshold only applies to collider-less
    /// bodies»*. Na 0.28 era `sq_angvel < limiar²`, e o knob mandava.
    ///
    /// ⭐⭐ **RESOLVIDO em 2026-08-30 — a saída escolhida foi «o controlo exprime o que o motor
    /// lê».** O campo FICA (ele é persistido; apagá-lo partiria todo `.ph2dproj` gravado) e o
    /// **slider** do painel de física morreu: o que o artista vê agora é um **interruptor**, porque
    /// o que a rapier lê deste `f32` é **um bit**. Ver [`SLEEP_SPIN_DISABLED`] para o que cada lado
    /// do sinal significa, e `PhysicsSettings::sleep_enabled` (`ph2d-physics-ecs`) para a porta.
    ///
    /// ⚠️ **A magnitude não é «quase morta», é morta com uma excepção nomeada:** ela ainda decide
    /// para um corpo **sem collider** (`max_extent == 0`), que é o ramo `else` do trecho acima.
    /// Nenhuma entidade da cena está nesse ramo — um corpo sem collider não colide com nada —, e é
    /// por isso que o interruptor não perde nada que alguém possa ver.
    ///
    /// *Um controlo que não move nada é o defeito que esta casa já pagou noutros painéis;
    /// a autoridade de um número expira com a versão da biblioteca que o consome.*
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
    ///
    /// # ⛔⛔ O limiar de sono: fixar o `0,4` foi ERRADO, e o smoke provou-o
    ///
    /// A regra acima — *«preservar o número preserva o tato»* — tem um buraco, e ele apareceu no
    /// smoke do Enio em 2026-08-29 (foto: uma caixa parada **inclinada**, sem assentar no chão):
    ///
    /// ⭐⭐⭐ **Preservar uma constante de AFINAÇÃO através de uma reescrita de MOTOR não é
    /// conservador — pode ser ELA a mudança.** O `0,4` era o limiar da 0.28 e afinava o solver da
    /// 0.28. O solver da 0.35 é outro: um corpo assenta por um caminho diferente, e passa a cair
    /// abaixo de `0,4 m/s` **a meio do assentamento**. Ele adormece ali, torto, e não acorda mais.
    /// *A `rapier` baixou o dela para `0,05` — 8× — e nós tínhamos fixado exactamente o valor que
    /// ela abandonou.*
    ///
    /// **A medição** (cena de smoke 4 — 12 corpos, 3 tamanhos, 20 s a 60 Hz; ângulo em graus, e
    /// um corpo assente num chão plano deve ficar em `~0`):
    ///
    /// | limiar / atraso | pior ângulo | corpos congelados | deriva 10 s→20 s |
    /// |---|---|---|---|
    /// | ⛔ `0,4` / `2,0 s` — **o que fixámos** | **`2,320°`** | 12/12 | `0` |
    /// | ⭐ `0,05` / `2,0 s` — **o que shipa** | **`0,04455°`** | 12/12 | `0` |
    /// | `0,05` / `0,5 s` (a rapier inteira) | `0,04454°` | 12/12 | `0` |
    /// | ⛔ `0,4` / `0,5 s` | **`7,621°`** | 12/12 | `0` |
    /// | **controle: SEM sono** | **`0,04455°`** | **0/12** | `1,15e-7` |
    ///
    /// ⭐⭐ **O CONTROLE é o que torna isto uma cura e não uma troca.** Com `0,05` os corpos
    /// **adormecem mesmo** — 12 de 12 congelados, deriva **exactamente zero** entre o segundo 10 e
    /// o 20 — e a pose em que adormecem é a de **nunca dormir** a menos de `1e-5` de grau. Sem o
    /// controle, *«igual a não dormir»* podia ser a tautologia de nada ter adormecido.
    ///
    /// ⚠️ **O ATRASO não é a alavanca, e encurtá-lo sozinho PIORA 3×** (`7,62°`). ⇒ o
    /// `time_until_sleep` fica nos nossos `2,0 s`: medido, ele não muda a pose com o limiar certo,
    /// e é o lado que erra por ficar acordado. *Muda-se o que a medição exige, e nada mais.*
    ///
    /// ⚠️ **E a lição sobre a lição:** o parágrafo acima continua válido — estes valores são
    /// **persistidos** e têm de ser **escritos**, não lidos de uma fonte que se move. O que estava
    /// errado não era escrever o número; era escolher, para o escrever, o número de um motor que
    /// já não existe. *Um valor fixado herda a autoridade da versão de onde veio, e essa
    /// autoridade caduca com ela.*
    #[must_use]
    pub fn ours() -> Self {
        Self {
            // Estes dois continuam a ser os da rapier, e continuam a ser zero nas duas versões:
            // um corpo sem arrasto autorado não é uma escolha nossa, é a ausência de uma.
            linear_damping: 0.0,
            angular_damping: 0.0,
            // ⚠️ Os três de adormecer são NOSSOS desde 2026-08-29 — ver o doc acima.
            // ⛔⛔ **O `0,05` é MEDIDO, e substituiu um `0,4` que era o da rapier 0.28** — ver
            // [`BodyDefaults::ours`] §«o limiar de sono».
            sleep_linear_threshold: 0.05,
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
        //
        // ⚠️ **O limiar linear já NÃO diverge, e isso é o registo de um defeito.** Ele valeu `0,4`
        // (o da rapier 0.28) entre 2026-08-29 e o mesmo dia, até um smoke do Enio mostrar uma
        // pilha adormecida **torta**: fixar uma constante de afinação através de uma reescrita de
        // solver não é conservador. Medido, `0,05` dá a pose de quem nunca dorme a menos de `1e-5`
        // de grau — ver [`BodyDefaults::ours`], secção «o limiar de sono», e o gate de produto
        // `a_settled_stack_falls_asleep_lying_flat_not_halfway_there`.
        assert_eq!(
            (o.sleep_linear_threshold, o.time_until_sleep),
            (0.05, 2.0),
            "o limiar de adormecer e' MEDIDO (0,05) e o atraso e' NOSSO (2,0 s). O limiar coincide \
             hoje com o da rapier 0.35 por MEDICAO, nao por heranca: a tabela esta' no doc de \
             `ours()`. O atraso fica em 2,0 porque a medicao mostrou que ele nao muda a pose com o \
             limiar certo -- e encurta-lo sozinho PIORA 3x."
        );
        // ⛔ E a metade que o par acima não cobre: se um dia a rapier voltar a subir o dela, o
        // nosso `0,05` tem de ficar — ele é uma medição nossa, não um espelho.
        assert!(
            o.sleep_linear_threshold <= r.sleep_linear_threshold,
            "a rapier subiu o limiar dela para {} e o nosso e' {}. O nosso e' MEDIDO: um corpo tem \
             de adormecer na mesma pose de quem nao dorme. Se quiser adoptar o dela, meça primeiro \
             com o gate `a_settled_stack_falls_asleep_lying_flat_not_halfway_there`.",
            r.sleep_linear_threshold,
            o.sleep_linear_threshold
        );
        assert_eq!(
            o.sleep_angular_threshold, r.sleep_angular_threshold,
            "o limiar ANGULAR nunca divergiu (0,5 nas duas versoes da rapier). Se ele divergir \
             agora, ou a rapier mudou ou alguem o fixou sem dizer porque^."
        );
    }
}
