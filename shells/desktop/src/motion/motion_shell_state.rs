//! **O estado da familia MOTION que vivia solto em `App`** — quatro campos numa struct.
//!
//! ⚠️ **Porque existe (W2/L1, 2026-09-11):** a `App` tem ~401 campos publicos e e' a razao
//! de a shell ser a ultima unidade de todo build grande. O censo desta familia mediu que
//! ela toca **6** campos de `App`, e que **4** deles sao lidos SO' por ela — logo nao sao
//! da `App`, sao da familia, e estavam ali por inercia (ADR-0075: o estado de uma feature
//! e' recurso dela, nao campo do hospedeiro).
//!
//! Os outros dois ficam, e a razao esta escrita: `gfx` (o renderer, de todos) e
//! `vec_entities`/`timeline`/`playhead` (de outras familias ou partilhados). Sao a lista
//! que a `ph2d-app-host` da L0 tem de cobrir na Fase B.
//!
//! ⚠️ **E' `App.motion_shell`, um campo so'** — nao um recurso do ECS. A escolha e' medida:
//! `motion_path_drag` e' lido pelo `input_dispatch` **antes** de haver mundo para consultar,
//! e `motion_leaf_images` tem de sobreviver ao quadro sem passar pela captura de undo. Um
//! recurso do ECS entraria no `WorldSnapshot`, e estes quatro sao **runtime-only** por
//! escrito: um arrasto vivo nao e' documento.
//!
//! ⚠️ **A `MotionState` (a de `AppGfx.motion`) e' OUTRA COISA e nao se funde com esta:** ela
//! vive dentro do `AppGfx`, que e' `Option` e so' existe depois de haver GPU. Estes quatro
//! sao lidos com `gfx` a `None`.

/// O estado de shell da familia motion — o que era quatro campos soltos na [`crate::App`].
#[derive(Default)]
pub(crate) struct MotionShellState {
    /// Latch do `PH2D_PATH_SMOKE` (a cena do motion path, uma vez).
    pub(crate) path_smoke_done: bool,

    /// A alça de tangente sob arrasto (ADR-0141, Fatia 3). Armada no press, limpa no
    /// release. Runtime-only: um arrasto vivo não é documento; o resultado (a geometria e as
    /// distâncias que as keys guardam) é, e vive no `TargetBinding.path` + na track.
    ///
    /// Carrega o alvo (e o índice, e qual alça) para que o move não precise re-perguntar
    /// qual binding é — a seleção pode mudar no meio de um arrasto (um atalho, um undo), e
    /// o gesto continua sendo sobre o que foi pego.
    pub(crate) path_drag: Option<crate::render_loop::motion_path_overlay::MotionPathGrab>,

    /// O último clique primário no canvas — `(instante, posição)` — para detectar o
    /// DUPLO-clique no caminho (ADR-0141), que insere um ponto na trajetória. O canvas não
    /// emite `DoubleClick` (é evento por-widget do chrome), então o par é rastreado aqui, o
    /// mesmo recurso do duplo-clique de texto (`vec_last_canvas_click`). Runtime-only.
    pub(crate) path_last_click: Option<(std::time::Instant, (f32, f32))>,

    /// ⭐ **A arte, em CPU, dos quads que o Motion desenha na cena vectorial** — a memória da
    /// terceira média (ver [`ph2d_app_motion::motion_leaf_images`]). Vive aqui porque toda leitura PARA a
    /// GPU, e ela tem de sobreviver ao quadro.
    pub(crate) leaf_images: ph2d_app_motion::motion_leaf_images::LeafImages,
}
