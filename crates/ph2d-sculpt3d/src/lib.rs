#![forbid(unsafe_code)]
//! `ph2d-sculpt3d` — **os verbos de escultura, e a lei que os governa**.
//!
//! A W1 provou que o custo de um dab é da PEGADA e não da malha. A W2 é o
//! barro: doze verbos, máscara, simetria — e, antes de tudo isso, a **lei do
//! traço** de [`stroke`], que é o que separa este módulo de um port ingênuo.
//!
//! # A entrada, em três linhas
//!
//! ```no_run
//! # use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};
//! # let mut mesh = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
//! # let hit_point = [0.0, 0.0, 1.0];
//! let brush = Brush { verb: Verb::Draw, radius: 0.3, ..Brush::default() };
//! let mut stroke = SculptStroke::default();
//! stroke.begin(&mesh);                                        // congela o `pre`
//! let eye = [0.0, 0.0, -1.0];  // o `dir` do raio que produziu o acerto
//! stroke.dab(&mut mesh, &brush, &Dab::at(hit_point, 0.3, eye), Symmetry::MIRROR_X);
//! ```
//!
//! # Sobre a "porta única" que o `docs/3D/03.5` promete
//!
//! Aquele documento desenha `sculpt_kernel_device(vertices) -> Device`, e ela
//! **continua não construída, de propósito**. Uma porta com uma resposta só e um
//! variant inalcançável é um controle que não faz nada. Ela nasce **quando a
//! medição pedir**: se o K1 disparar num regime que o artista use, o caminho de
//! GPU e a porta chegam juntos, com o de CPU virando o oráculo de paridade dele.
//!
//! # O que a W1 mediu, e o que mudou
//!
//! A sonda `tests/measure_brush_kernel.rs` media um `apply_dab` que **não existe
//! mais** — o traço o subsumiu, e manter os dois seria a segunda porta para
//! *"aplicar um dab"*. Ela agora dirige o produto (`begin` + `dab`), que faz
//! estritamente MAIS trabalho por dab (captura + envelope + alvo), então os
//! números da W1 foram **re-medidos** em vez de herdados.

mod alpha;

/// **ONDE UM GESTO PEGA O BARRO** — ver [`ancora`]. A porta que o app e a
/// bancada de paridade chamam, para os dois medirem a mesma âncora.
pub mod ancora;
/// **O ORÇAMENTO DE ALISAMENTO** — quantas passadas do laplaciano um
/// `auto_smooth` compra, e com que força cada uma. Ver [`auto_smooth`].
pub mod auto_smooth;
/// **A SILHUETA DE UM DAB** — ver [`footprint`].
mod footprint;
pub use footprint::{Blade, Footprint, Strip, Tectos, rounded_box};
/// A lei de quanto pente um traço leva. Ver [`stroke_rake::pente_do_traco`].
pub use stroke::stroke_rake::pente_do_traco;

/// ⭐⭐⭐ **A metade do pente que mora no PASSE DE TOPOLOGIA** — ver
/// [`ph2d_rake::campo_do_pente`].
///
/// ⚠️ **Ela é re-exportada daqui e não importada directamente pela crate da
/// app**, pela mesma razão que o [`pente_do_traco`] vive neste ficheiro: esta
/// crate é a **porta** da família para a lei, e uma segunda entrada na
/// crate-folha seria um segundo sítio a manter alinhado com ela. *O deslocamento
/// e o alinhamento são duas metades da MESMA lei, e quem as chama tem de as ver
/// pela mesma porta.*
pub use ph2d_rake::{Porta, campo_do_pente, preferencia_do_pente};

/// Os controlos próprios do pincel de CONTORNO — ver [`boundary_controlos`].
pub mod boundary_controlos;
mod brush;
/// **OS CINCO TIPOS DO FILTRO DE TECIDO** (espec §7) — ver [`cloth_filter_kind`].
mod cloth_filter_kind;
mod cloth_filter_props;
mod cloth_force_falloff;
mod cloth_mode;
mod coat;
/// ⭐⭐⭐ **PORQUE É QUE A CURVA NÃO CHEGA AO BARRO** — ver [`curva_inerte`].
mod curva_inerte;
#[path = "dab_alcance.rs"]
mod dab_alcance;
/// **Quantos raios de pincel a superfície anda antes de se chamar «não
/// alcança»** — o número é DERIVADO de um planalto medido; ver
/// [`dab_alcance::ALCANCE_TECTO`].
pub use dab_alcance::{ALCANCE_TECTO, NORMAL_LIMIAR, RAZAO_MAXIMA};
/// **A CURVA DO PINCEL** — o falloff, irmão do [`brush`]. Ver [`falloff`].
mod falloff;
/// **A LEI QUE UM ARRASTO DE FILTRO APLICA** — a uniao das duas familias.
mod filter_law;
mod grip;
/// ⭐⭐⭐ **AS DUAS COLUNAS DO PENTE** — a régua do alinhamento e a do pior
/// triângulo. ⚠️ **`pub` de propósito:** ela tem dois consumidores em espaços
/// diferentes — a bancada da lei (sobre uma chapa) e o gate da cena de smoke
/// (sobre uma bola, noutra crate). Ver [`medida_do_pente`].
pub mod medida_do_pente;

/// ⭐⭐⭐⭐ **A RÉGUA QUE O OLHO USA — o comprimento da FILEIRA.** Irmã da
/// [`medida_do_pente`], e o corte é a GRANDEZA: ali conta-se **cada aresta uma
/// a uma**, aqui mede-se **quantas seguidas continuam a mesma linha**. Ver
/// [`medida_da_fileira`].
pub mod medida_da_fileira;
/// ⭐⭐ **PARA ONDE O ESFREGÃO EMPURRA** — ver [`smear_mode`].
mod smear_mode;

/// ⭐⭐⭐ **A FORMA QUE O BOX TRIM CORTA** — ver [`trim_forma`].
mod trim_forma;

/// ⭐⭐ **PARA ONDE O RAIO DA PROJECÇÃO APONTA** — ver [`project_mode`].
mod project_mode;

/// O passo e a atenuação do traço arrastado do [`Verb::Plane`] (espec §14.4).
pub mod atenuacao_do_traco;
/// ⭐⭐⭐ **O PASSO MEDIDO SOBRE A SUPERFÍCIE** — a cura do vinco pontilhado
/// junto à silhueta. Ver o módulo.
#[path = "passo_no_mundo.rs"]
mod passo_no_mundo;
/// ⭐⭐ **DUAS LEIS PARA A MESMA TECLA** — o que o `Ctrl` faz ao pincel de plano;
/// ver [`plano_inversao`].
mod plano_inversao;
/// A memória do plano do [`Verb::Plane`] — os dois estabilizadores (espec §6).
pub mod plano_memoria;
pub use passo_no_mundo::{
    CaminhoNoMundo, CarimboDoCaminho, MAX_DABS_POR_PASSO, PASSO_INICIAL_DO_CANDIDATO_PX,
    PISO_DO_CANDIDATO_PX, levado_pela_deformacao, passo_no_mundo,
};

pub use atenuacao_do_traco::{
    ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT, ESPACAMENTO_DO_AFIADO_PCT, ESPACAMENTO_DO_PLANO_PCT,
    atenuacao_por_espacamento, espacamento_do_traco, espacamento_do_verbo, passo_de_um_espacamento,
    passo_do_traco,
};

/// ⭐⭐⭐ **A DISTÂNCIA ATÉ À OUTRA PEÇA** — ver [`projectar`].
mod projectar;
pub use boundary_controlos::BoundaryControlos;
/// ⭐ **Os dois selectores do contorno, re-exportados.** O painel escolhe entre
/// eles e não precisa de conhecer a crate da lei — a mesma forma com que ele já
/// lê o `ClothMode` e o `PoseModo`.
pub use ph2d_boundary::{Modo as BoundaryModo, QuedaNoContorno as BoundaryQueda};

/// Os controlos próprios do pincel de POSE — ver [`pose_controlos`].
pub mod pose_controlos;
pub use pose_controlos::PoseControlos;
/// ⭐⭐ **O INDICADOR do pincel de POSE** — o osso que se vê antes de premir, com
/// a cache e o orçamento que o separam do alvo. Ver [`pose_previa`].
pub mod boundary_previa;
pub mod pose_previa;
pub use boundary_previa::TrechoDaBorda;
/// ⭐⭐ **Os CINCO gestos do pincel de pose, re-exportados.** O painel escolhe
/// entre eles directamente desde 2026-09-15, por ordem do dono — ⛔ o
/// [`PoseModo`] acima continua a ser o que a LEI lê, e a ponte entre os dois é a
/// [`ph2d_pose::Deformacao::modo_e_inversao`], com gate de ida-e-volta.
/// ⭐⭐ **Quanto do arrasto a escala da pose lê, re-exportado.** O painel escolhe
/// entre os dois e não precisa de conhecer a crate da lei — a mesma forma com
/// que ele já lê o `PoseModo` e o `ClothMode`.
pub use ph2d_pose::Arrasto as PoseArrasto;
pub use ph2d_pose::Deformacao as PoseDeformacao;
/// ⭐ **O modo do pincel de pose, re-exportado.** O painel escolhe entre os três
/// e não precisa de conhecer a crate da lei — a mesma forma com que ele já lê o
/// `ClothMode`.
pub use ph2d_pose::Modo as PoseModo;
pub use pose_previa::Osso as PoseOsso;
/// **O CAMPO ELÁSTICO** — os Kelvinlets regularizados (de Goes & James 2017),
/// que são o `l-mode` da família que agarra. Ver [`kelvinlet`].
pub mod kelvinlet;
pub mod mask_ops;
mod peso_do_ponto;
/// ⭐ **O PREENCHIMENTO** (`Fill`) — ver o cabeçalho dele.
pub mod preenche;
#[cfg(test)]
#[path = "preenche_tests.rs"]
mod preenche_tests;
mod preview;
/// **OS KERNELS DA REFERÊNCIA** — o porte 1:1 do SculptGL, `f64` na aritmética
/// e `f32` no armazenamento, gateado bit a bit contra o JS EXECUTANDO
/// (`tests/sculptgl_parity.rs`). Ver [`ref_kernels`].
pub mod ref_kernels;
mod ref_mode;
/// **OS TRÊS MODOS DE REFERÊNCIA** — de qual fonte (SculptGL · Blender ·
/// literatura) um verbo herda o que ele é. Ver [`ref_mode`] e o plano
/// `docs/3D/21_plano_modos_e_ferramentas.md`.
/// A tabela DECLARATIVA dos modos — ver o cabeçalho dela.
mod ref_profiles;
mod spacing;
mod stroke;
/// ⭐ **A TELA DO PAINTER POUSADA NA PEÇA** — ver o cabeçalho dele.
pub mod tela_na_malha;
#[cfg(test)]
#[path = "tela_na_malha_tests.rs"]
mod tela_na_malha_tests;
/// ⭐⭐ **O RETRATO DA PEÇA** — a imagem com que a tela do Painter começa nos
/// modos que lêem a cor debaixo do pincel; ver o cabeçalho dele.
pub mod tela_semente;
#[cfg(test)]
#[path = "tela_semente_tests.rs"]
mod tela_semente_tests;
pub mod tinta_fina;
#[cfg(test)]
#[path = "tinta_fina_tests.rs"]
mod tinta_fina_tests;
mod transform;

/// ⭐⭐⭐ **A porta que os censos usam para perguntar «este dab mudou alguma
/// coisa?»** — ver [`canal_de_teste`]. Ela nasceu quando o segundo verbo de
/// canal chegou e TRÊS harnesses acusaram produto correcto, cada um com a sua
/// cópia de `if paints_mask() { máscara } else { posições }`.
///
/// ⚠️ **E ela ATRAVESSA a fronteira da crate desde 2026-09-19, pela feature
/// `test-support` e do TAMANHO do que atravessa** (HOWTO §2.5): o censo dos
/// knobs da `ph2d-app-sculpt3d` precisa de semear a cor da peça — sem isso os
/// dois verbos que leem o ANEL medem-se inertes sobre uma peça branca — e um
/// `#[cfg(test)]` é invisível do outro lado. ⛔ Só a [`canal_de_teste::semeia_cor`]
/// é pública; o resto do módulo continua `cfg(test)`, senão uma build com a
/// feature ligada e sem testes entrega `dead_code` no que ninguém lê.
#[cfg(any(test, feature = "test-support"))]
pub mod canal_de_teste;

pub use alpha::{
    Alpha, AlphaFrame, AlphaImage, AlphaStencil, DEFAULT_ALPHA_SCALE, MAX_ALPHA_SCALE,
    MAX_AXIS_ELEV_DEG, MIN_ALPHA_SCALE, recommended_scale, sampled_edge,
};
pub use brush::{
    BLENDER_REACH_FRACTION, Brush, CLAY_PLANE_FRACTION, CLAY_THUMB_TILT_MAX_DEG,
    CLAY_THUMB_TILT_STEP_DEG, CREASE_FRACTION, DEFAULT_MULTIPLANE_ANGLE_DEG, FilterKind,
    LAYER_HEIGHT_HARD_MAX, LAYER_HEIGHT_UI_MAX, MAX_MASK_HARDNESS, MULTIPLANE_ANGLE_MAX_DEG,
    MULTIPLANE_ANGLE_SMOOTH, MULTIPLANE_TIP_STRETCH, PINCH_GAIN, Pass, REACH_FRACTION,
    RingOperator, STRIP_PLANE_FRACTION, Symmetry, TAUBIN_LAMBDA, TAUBIN_MU, TAUBIN_PASS_BAND,
    TECTOS_CENTRO_MAX, Verb,
};
pub use cloth_filter_kind::{ClothFilterKind, ClothFilterOrientation};
pub use cloth_filter_props::ClothFilterProps;
pub use cloth_force_falloff::ClothForceFalloff;
pub use cloth_mode::{ClothArea, ClothMode};
pub use coat::{COAT_HEAD, coat_step};
pub use curva_inerte::CurvaInerte;
pub use falloff::{Falloff, fora_da_pegada};
pub use filter_law::FilterLaw;
pub use grip::{Amount, Grip, GripLaw};
pub use kelvinlet::KELVINLET_REACH;
pub use plano_inversao::PlanoInversao;
pub use preview::{NO_PREVIEW, preview_into, preview_verts};
pub use project_mode::ProjectMode;
pub use ref_mode::{Field, FrontFace, KernelLaw, LateralPull, PlaneReach, RefMode};
pub use ref_profiles::VerbProfile;
pub use smear_mode::SmearMode;
pub use trim_forma::TrimForma;

/// **SÓ PARA A BANCADA** — a lei do raio, sem o traço à volta. Ver
/// [`projectar::distancia`].
#[must_use]
pub fn distancia_de_projeccao_para_teste(
    ponto: [f32; 3],
    direccao: [f32; 3],
    activo: ph2d_mesh::Pose,
    alvos: &[(ph2d_mesh::Mesh, ph2d_mesh::Pose)],
    bidir: bool,
    folga: f32,
) -> Option<f32> {
    projectar::distancia(ponto, direccao, activo, alvos, bidir, folga)
}
pub use spacing::{MIN_SPACING_FRACTION, Walk, min_spacing, walk};
pub use stroke::ClothFilterStep;
pub use stroke::cloth_repica;
pub use stroke::{
    Dab, FILTER_DRAG_PER_PX, HC_SHAPE_DEFAULT, HC_VERTEX_DEFAULT, HC_VERTEX_MIN, SculptStroke,
    sharpen_total_for_measurement,
};
pub use transform::{Gesture, MIN_SCALE_FACTOR, MaskTransform, TransformKind, free_pivot};
