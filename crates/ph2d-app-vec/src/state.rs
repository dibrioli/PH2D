//! ⭐⭐ **O estado da família `vec` que já pode viver fora da shell** — W2/L4, A2 (ADR-0075).
//!
//! # A régua que escolheu estes campos, e os 56 que ficaram
//!
//! `App` tem ~401 campos públicos e **75** deles chamam-se `vec_*`. Quantos podem sair não é uma
//! escolha — mede-se por duas perguntas, nesta ordem:
//!
//! 1. **quem os lê?** `app_state.rs` (a declaração) e `main.rs` (o construtor) não contam. Restam
//!    **30** campos cujos únicos consumidores de fora da família são os dois roteadores da shell
//!    (`render_loop/mod.rs`, `input_dispatch.rs`) — ou ninguém. Os outros 45 estão espalhados, e
//!    três deles são o substrato do módulo: `vec_entities` (189 ficheiros), `vec_scene` (131),
//!    `vec_pen` (36). ⛔ Esses não são «estado da família»: são o documento e a árvore, que a shell
//!    inteira conduz.
//! 2. **o TIPO deles vem de uma crate?** Dos 30, **19** sim (11 de `ph2d_vec_*`, 8 primitivos) e
//!    **11 não** — o tipo mora na shell (`crate::blend_live::BlendSpines`,
//!    `crate::vec_bucket::BucketCache`, `crate::morph_live::MorphPlans`, …). Um campo cujo tipo
//!    está na shell **não pode** viver aqui, e agrupá-lo num `VecState` da shell seria trocar 11
//!    campos por 1 sem mover uma linha de lugar.
//!
//! ⇒ estes 19. Os 11 bloqueados estão nomeados no handoff de integração: cada um espera que o
//! TIPO dele saia, não que alguém o agrupe.
//!
//! # ⚠️ Porque não é um recurso do ECS
//!
//! O ADR-0075 manda desacoplar por ECS, e a primeira leitura disto é «ponha num componente». ⛔
//! Errado aqui, e o mecanismo está escrito no [`undo.rs`](../../../shells/desktop/src/undo.rs): o
//! `ProjectState` que o undo fotografa é `{WorldSnapshot + VecScene + FlipDoc + …}`, e o
//! `WorldSnapshot` **cobre toda entidade com componente registado**. Um `ShapeTool` ou um `Pencil`
//! como componente entraria na fotografia ⇒ **cada nudge do lápis viraria um passo de undo**.
//!
//! Estado de FERRAMENTA não é estado de documento. O precedente da casa é o mesmo: `prop_state`,
//! `walk_state`, `z_walk_state` e `input` já são sub-structs num campo de `App`.
//!
//! # ⚠️ `Default` é derivado, e isso foi VERIFICADO campo a campo
//!
//! Os 19 inicializadores do `main.rs` eram `false` / `0` / `None` / `Vec::new()` /
//! `Pencil::default()` / **`ShapeTool::new()`** — e esse último é literalmente `Self::default()`
//! ([`shape.rs`](../../ph2d-vec-edit/src/shape.rs)). *Um `derive(Default)` sobre um campo cujo
//! `new()` faz mais que o `Default` muda o arranque do app em silêncio* — aqui não faz, e é por
//! isso que se conferiu antes de derivar.

/// Os campos da família `vec` que saíram de [`App`](../../../shells/desktop/src/app_state.rs).
///
/// O nome de cada um perdeu o prefixo `vec_`, que agora é o do campo em `App` (`vec_state`).
#[derive(Default)]
pub struct VecState {
    /// A cena do **fade vetorial** já montou? (`PH2D_VEC_FADE_SMOKE`, uma vez por sessão.)
    /// A cena da APARÊNCIA do objecto (`PH2D_VEC_APPEARANCE_SMOKE`) já montou.
    pub appearance_smoke_done: bool,

    /// O **overlay ordenado** de TODOS os blends (passos + fontes reempilhadas, em z), cozido em
    /// MUNDO a cada frame pelo `blend_live::recook` e desenhado por
    /// `ph2d_vec_render::draw_blend_overlay`. Não está na cena (não é pickável) — é o que torna o
    /// blend UM objeto, e não N. Runtime-only.
    pub blend_overlay: Vec<ph2d_vec_scene::VecPath>,

    /// **Pick Shapes** (ADR-0128 C2b): as formas fechadas que o artista clicou **na ordem**, no
    /// modo `DrawMode::PickBlend`. O botão Blend as liga nessa sequência (em vez da ordem de z), e a
    /// prévia do spine as costura no canvas. Esvaziado ao criar o blend ou ao sair do modo.
    /// Runtime-only.
    pub blend_picks: Vec<ph2d_vec_scene::VecPathId>,

    /// Em que TEMPO está a cena dos ossos (`PH2D_VEC_BONE_SMOKE`): `0` monta, `1` deixa o `sync`
    /// dar entidade às formas, `2` prende, `3` acabou. ⚠️ Ela precisa de dois quadros porque
    /// prender exige a ENTIDADE da forma, e quem a cria corre depois do prólogo.
    pub bone_smoke_step: u8,

    /// ⭐ **A IMAGEM da cena de osso** — os bits da sprite e a raiz do esqueleto dela.
    ///
    /// ⚠️ **Slot próprio, e não uma 4.ª entrada no `_pend`:** aquele guarda `VecPathId`, e uma
    /// imagem não é um caminho. Enfiá-la lá pediria um id inventado, e o passo que espera as
    /// entidades (`vec_entities::contains_key`) procuraria por ele para sempre.
    pub bone_smoke_img: Option<(u64, Option<ph2d_ecs::Entity>)>,
    pub bone_smoke_pend: Option<[(ph2d_vec_scene::VecPathId, Option<ph2d_ecs::Entity>); 3]>,

    /// In-app path clipboard for Vector Ctrl+C/X/V — a clone of the copied path
    /// (geometry + style, id-less). `None` until the first copy/cut.
    pub clipboard: Option<ph2d_vec_scene::VecClip>,

    /// A forma cujo CONTOUR o painel está espelhando (os três sliders + os dois trios). Mesmo
    /// papel do [`Self::vec_offset_mirrored`] e pela mesma razão: o `paint` lê o STORE primeiro
    /// (senão o número saltaria durante o arrasto), então sem uma borda que reescreva o store na
    /// troca de seleção, escolher outra forma mostraria os valores da anterior. Runtime-only.
    pub contour_mirrored: Option<ph2d_vec_scene::VecPathId>,

    /// A **LINHA DE CORTE recém-começada** (o press da caneta em modo `Cut`), esperando a
    /// entidade dela nascer no `vec_entities::sync` para receber o `VecCutPath`. Espelho exato do
    /// `vec_connect_pending` e do `vec_blend_pending`, e pela mesma razão: sem esta fila de um
    /// item, a lâmina ficaria na cena como um caminho comum — desenhada como arte, exportada
    /// como arte, e fora do alcance do botão que existe para a descartar.
    pub cut_pending: Option<ph2d_vec_scene::VecPathId>,

    pub fade_smoke_done: bool,

    pub grad_drag: Option<ph2d_vec_render::GradHandle>,

    /// The selected gradient handle (drives the overlay highlight + the Remove-
    /// point / Influence / Jitter targets, via [`GradHandle::point`]). `None` = none.
    pub grad_selected: Option<ph2d_vec_render::GradHandle>,

    /// **O LÁPIS** — a mão livre (W1 do plano 25). Irmão do `App::vec_pen` (que FICOU na shell: 36
    /// ficheiros o conduzem) e do [`Self::shape`], que veio com ele: a shell
    /// converte tela→mundo e ele acumula as amostras, decima e ajusta a spline AO VIVO. O path
    /// vivo mora na cena desde o press (o padrão da `ShapeTool`), então preview, undo de um passo
    /// e seleção no release saem do caminho normal.
    pub pencil: ph2d_vec_edit::Pencil,

    /// Snap + grid settings of the Vector tool (edited by the panel's Snap section).
    /// "Set Center" armado: a próxima pressão no canvas põe a ORIGEM da forma
    /// selecionada ali (ADR-0112). Desarma no press.
    pub pivot_edit: bool,

    /// A forma cujo **perfil de largura** os quatro knobs `W *` estão espelhando (ADR-0148).
    /// Mesmo papel do [`Self::vec_offset_mirrored`] e pela mesma razão: a borda é a SELEÇÃO, e
    /// sem ela escolher uma forma com perfil vivo mostraria os knobs globais do painel.
    pub profile_mirrored: Option<ph2d_vec_scene::VecPathId>,

    /// ADR-0108 Fase 1: drag-to-size shape drawing (Rectangle / Ellipse /
    /// Polygon). Sibling of `vec_pen`; the shell routes canvas input to one or
    /// the other by `vec_draw_mode` (mirrored from the tool by `vector_bridge`).
    pub shape: ph2d_vec_edit::ShapeTool,

    /// A cena da PILHA DE APARÊNCIA (`PH2D_VEC_STACK_SMOKE`) já montou.
    pub stack_smoke_done: bool,

    /// **O eixo de SESSÃO da simetria**, em MUNDO — a linha que aparece no instante em que o botão
    /// liga, com a cena vazia e nada seleccionado.
    ///
    /// ⚠️ Mora aqui e não na ferramenta porque nasce do centro do **ecrã** (*"a tela é a referência
    /// para a posição inicial da linha"*), e só a shell tem câmera. `None` = por semear: a
    /// semeadura acontece na aresta desligado→ligado, e é isso que faz a linha ficar no lugar ao
    /// longo de quantos desenhos o artista quiser.
    pub symmetry_origin: Option<[f64; 2]>,

    /// A largura de refluxo corrente do texto (`None` = Auto). Default da PRÓXIMA sessão e
    /// espelho da que está viva — o mesmo papel dos irmãos `vec_text_*`.
    pub text_wrap: Option<f64>,

    /// A alça do TEXTO EM CAMINHO está sob arrasto? (W5) Armada no press de Node, limpa no
    /// release. Runtime-only: um arrasto vivo não é documento — o resultado (`start_offset`) é,
    /// e vive no `VecTextPath`. Um booleano e não um alvo porque há UMA alça (o `start_offset` do
    /// texto selecionado); qual texto vem da seleção, como no painel. Ver [`crate::text_ride`].
    pub textpath_handle_drag: bool,

    /// A geometria do realce, em MUNDO — derivada do [`Self::vec_trim_hit`] no mesmo quadro.
    pub trim_piece: Vec<ph2d_vec_scene::VecPath>,
}

/// A pose que ESTE passe escreveu no frame anterior, por rótulo. Runtime-only (não vai para o
/// save nem para o undo): o pior que um cache perdido causa é um frame de absorção idempotente
/// — o estado restaurado é auto-consistente (`centro = âncora + offset`), então re-absorver
/// devolve o MESMO offset.
///
/// ⚠️ Desceu da shell (`label_live.rs`) em 2026-09-12 (`line/render-loop`, A9 da auditoria de
/// arquitectura): hoje a instância dela é o campo `vec_label_poses` da `App`, e o passe dos
/// rótulos, que continua na shell, recebe-a por referência.
pub type LabelPoses = std::collections::BTreeMap<ph2d_vec_scene::VecPathId, ph2d_ecs::Transform>;
