//! **O ESTADO do auto-key** — a memória que o passe carrega de um quadro para o seguinte.
//!
//! ⚠️ Irmão do [`super::autokey_pass`] pelo tecto de LOC (HR-18, 600), e o corte é por
//! RESPONSABILIDADE: lá mora **a decisão** (o que conta como uma edição, o que se cunha, quem
//! recusa); aqui **o que ela precisa de lembrar**. Os dois campos que os reports de 2026-09-17
//! trouxeram (`clock_t` e `scrub_now`) são memória pura, e foram eles que puseram o ficheiro em
//! `609` de `600`.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_timeline::{PoseSample, PropKind};

use ph2d_timeline::record_fit::RecSpan;

/// The shell-owned state of the auto-key / pose machinery (one per `App`).
#[derive(Default)]
pub(crate) struct AutokeyState {
    /// Last frame's pose per selected entity — the reference for unbound
    /// first-touch auto-create.
    pub baseline: BTreeMap<u64, PoseSample>,
    /// A gizmo-drag undo bracket is open.
    pub drag_active: bool,
    /// Entities whose bound pose the user displaced while PAUSED and disarmed.
    /// The apply pass skips them so the pose holds for a manual K. Cleared by
    /// the bridge when the playhead moves; an entity heals out here when its
    /// pose returns to its curve.
    pub displaced: BTreeSet<u64>,
    /// The playhead time `displaced` was collected at (the bridge clears the
    /// set when the time changes).
    pub displaced_t: f64,
    /// ⛔⛔⛔ **O instante que ESTE passe viu no quadro anterior** — a metade que faltava ao
    /// guarda do `playing`.
    ///
    /// Report do dono (2026-09-17): *«ao arrastar o tempo na timeline cria keyframes em todos os
    /// quadros»*. Reproduzido: **12 chaves em 30 quadros** de arrasto, com a mão parada e a pose
    /// constante.
    ///
    /// ⚠️⚠️ **A razão do `playing` vale IGUAL para um arrasto, e o comentário dela já o dizia sem
    /// dar por isso:** *«a pose muda em cada quadro tocado porque é a ANIMAÇÃO que a conduz, não o
    /// utilizador»*. Arrastar o cursor conduz a pose exactamente da mesma maneira — só que
    /// `is_playing()` responde `false`, logo o guarda não armava e `capturing = armed`.
    ///
    /// ⇒ *o gatilho do auto-key é a MÃO, nunca o relógio* — e um quadro em que a única coisa que
    /// mudou foi o relógio não é uma edição, toque ou não toque o transporte.
    ///
    /// `None` no primeiro quadro: sem um instante anterior não há movimento que se afirme.
    pub clock_t: Option<f64>,
    /// ⛔⛔⛔ **A MÃO ESTÁ NO CURSOR DO TEMPO NESTE QUADRO** — escrito pelo dreno das intents
    /// ([`super::fase_timeline_drain`]) a partir de um `Scrub`/`SeekFrame`.
    ///
    /// ⚠️⚠️ **Ele existe porque o [`Self::clock_t`] sozinho NÃO chegou** (2.º report do dono no
    /// mesmo dia: *«criando keys de todo modo ao arrastar o tempo da timeline»*): com o **Snap**
    /// ligado o instante salta de quadro em quadro e fica **parado** entre saltos, logo os quadros
    /// de ecrã do meio liam *«o relógio não andou»* e voltavam a capturar.
    ///
    /// ⇒ *um gesto inferido de uma CONSEQUÊNCIA perde-se onde a consequência satura* — e este é o
    /// gesto em si, como o `is_playing()` é para o transporte. Os dois guardas ficam: o `Scrub` é a
    /// mão na régua, o `clock_t` apanha o que move o relógio sem ser ela (um passo de quadro por
    /// botão, um `rewind`, uma volta de loop).
    pub scrub_now: bool,
    /// The refusal the animator has already been told about. A drag against an
    /// overriding lane refuses on EVERY frame — sixty identical toasts a second is
    /// not information, it is noise. The toast fires on the rising edge, again if
    /// the REASON changes, and re-arms once the refusals stop.
    pub refusal: Option<ph2d_timeline::KeyRefusal>,
    /// Per `(entity, prop)` recorded span of the in-flight performing session —
    /// what to simplify (and over what tolerance) when the record drag ends.
    /// Empty outside a performing drag.
    pub(crate) record: BTreeMap<(u64, PropKind), RecSpan>,
}
