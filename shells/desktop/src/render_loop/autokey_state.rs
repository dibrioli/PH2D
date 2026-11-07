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
    /// Bound poses the user displaced while PAUSED: the apply skips them until the playhead moves.
    pub displaced: BTreeSet<u64>,
    /// The playhead time `displaced` was collected at (the bridge clears the
    /// set when the time changes).
    pub displaced_t: f64,
    /// ⛔ **O instante que ESTE passe viu no quadro anterior.** O gatilho do auto-key é a MÃO, nunca o
    /// relógio: um quadro em que só o relógio mudou não é uma edição (report do dono, 17/09: 12 chaves
    /// em 30 quadros de arrasto). `None` no primeiro quadro.
    pub clock_t: Option<f64>,
    /// ⛔ **A mão está no cursor do tempo neste quadro** (o dreno das intents, `Scrub`/`SeekFrame`). Com o
    /// Snap o instante fica parado entre saltos e o `clock_t` não vê; ficam os dois guardas.
    pub scrub_now: bool,
    /// The refusal the animator has already been told about (a toast on the rising edge only).
    pub refusal: Option<ph2d_timeline::KeyRefusal>,
    /// Per `(entity, prop)` recorded span of the in-flight performing session —
    /// what to simplify (and over what tolerance) when the record drag ends.
    /// Empty outside a performing drag.
    pub(crate) record: BTreeMap<(u64, PropKind), RecSpan>,
    /// ⭐ Os campos de família que cada entidade da população publicou NESTE quadro (plano 35) — o
    /// `run` enche-os ao lado da pose; um gate que chame o `apply_samples` direto deixa-os vazios.
    pub(crate) campos: BTreeMap<u64, Vec<(ph2d_ecs::FieldKey, f32)>>,
    /// Os mesmos, do quadro anterior — a `baseline` dos campos (sem curva, um campo só nasce canal
    /// se MUDOU).
    pub(crate) base_campos: BTreeMap<u64, BTreeMap<ph2d_ecs::FieldKey, f32>>,
    /// Quem o ▶ deixou em prévia (`ph2d_app_sprite_anim::previa`): tocador fora do diff.
    pub(crate) previa: BTreeSet<u64>,
    /// O mundo acabou de ser restaurado: ver [`Self::esquece_o_quadro`].
    pub(crate) restaurado: bool,
}

impl AutokeyState {
    /// ⛔ **Um desfazer/refazer não é a mão, nem o quadro seguinte** (Ctrl+Z e Stop do jogo, GEF 23): sem base,
    /// o valor restaurado não se lê como «mudou», e o apply desse quadro ainda não alcança os bits novos.
    pub(crate) fn esquece_o_quadro(&mut self) {
        self.baseline.clear();
        self.base_campos.clear();
        self.restaurado = true;
    }
}
