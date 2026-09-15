//! **Os latches das cenas de smoke da família das INSTÂNCIAS** — irmão de [`super`] por tecto de
//! LOC (HR-18).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o pai declara o que a `App` É; isto é o estado de UM
//! assunto dentro dela.
//!
//! ⚠️ **Eles ficam na SHELL e não na crate da família**, e a razão está escrita no
//! `components_scenes`: um latch dentro dela seria a família a ter opinião sobre **quando** o
//! quadro a chama.

/// ⭐⭐ **Os latches das cenas de smoke da família das INSTÂNCIAS** — um por roteador.
///
/// ⚠️ **Uma struct e não cinco campos soltos** (2026-09-14): a catraca `the_app_only_sheds_fields`
/// diz, no próprio texto da falha, que *«um campo novo tem DONO: ponha-o no estado da família do
/// assunto dele»* — e o dono destes é o assunto, não a `App`. ⛔ Subir o número é a última saída.
///
/// ⚠️ **O `game_camera` é lido FORA do prólogo** (`render_loop::fase_game_camera` decide mover o
/// herói da cena), e é por isso que ele não podia ser um `thread_local` da crate — ao contrário do
/// que os outros quatro poderiam ser, se a lei do latch não fosse a que é.
#[derive(Default)]
pub(crate) struct ComponentsSmokeLatches {
    /// ⭐⭐⭐ O smoke do `Timer` (TOP-20 #2). `PH2D_TIMER_SMOKE=1`.
    pub(crate) timer: bool,
    /// ⭐ A cena do `SignalActions` (TOP-20 #5).
    pub(crate) signal_action: bool,
    /// ⭐ A cena do SOM DE CENA (TOP-20 #4).
    pub(crate) audio_2d: bool,
    /// ⭐ A cena da CÂMERA DE JOGO (TOP-20 #7).
    pub(crate) game_camera: bool,
    /// ⭐ As duas cenas das TAGS (TOP-20 #9). `PH2D_TAGS_SMOKE=1|2`.
    pub(crate) tags: bool,
    /// ⭐ A FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12) — `PH2D_FACTORY_SMOKE`.
    pub(crate) factory: bool,
    /// ⭐ O MOVER DE VISTA DE CIMA (TOP-20 #13) — `PH2D_TOPDOWN_SMOKE=1|2`.
    pub(crate) topdown: bool,
    /// ⭐ O PROJÉCTIL (TOP-20 #14) — `PH2D_PROJECTILE_SMOKE=1|2`.
    pub(crate) projectile: bool,
    /// O ragdoll instanciado 3× (ADR-0164 F4). `PH2D_INSTANCE_SMOKE=1..7`.
    pub(crate) instance: bool,
}
