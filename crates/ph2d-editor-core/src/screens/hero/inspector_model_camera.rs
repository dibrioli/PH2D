//! **O modelo da secção CAMERA** (TOP-20 #7, W3) — snapshot e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — mesmo padrão dos outros nove.
//!
//! # ⚠️ QUATRO coisas viajam DERIVADAS, e nenhuma delas se podia derivar aqui
//!
//! O `ph2d-editor-core` é chrome e **não depende do `ph2d-ecs`** (ADR-0029). Mas há perguntas cuja
//! resposta o painel precisa e que só quem tem o mundo — ou a janela — sabe responder:
//!
//! - **quantas câmeras a cena tem** (`camera_count`) e **esta é a que MANDA?**
//!   (`is_active_camera`) — sem isso o artista afina uma câmera que ninguém usa, e o sintoma é
//!   *«mexo nos números e não acontece nada»*;
//! - **o alvo existe?** (`target_found`) — é uma resolução por NOME contra a cena inteira, e é a
//!   diferença entre *«segue ninguém»* e *«segue alguém que está parado»*, que dão exactamente a
//!   mesma câmera imóvel;
//! - ⭐⭐ **a cerca cabe na janela?** (`limits_smaller_than_view`) — isto é **geometria da JANELA**,
//!   não do componente: depende da proporção do ecrã e da altura da câmera. Quando ela não cabe, a
//!   lei fixa a câmera no centro da caixa (medido no oráculo), e *nenhum artista adivinha isso a
//!   olhar para quatro números*.
//!
//! ⇒ as quatro chegam **no snapshot**. *O painel não adivinha o mundo; ele recebe-o.*

/// A CÂMERA, como o Inspector a lê.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorGameCamera {
    pub height_world: f32,
    pub offset: [f32; 2],
    pub priority: i32,
    /// ⭐⭐⭐ **O DOLLY** (plano 24, W5) — ver [`CameraFieldEdit::Dolly`].
    pub dolly: f32,
    pub active: bool,
    pub cull_mask: u32,
}

/// ⭐⭐ **A FAIXA do dolly, numa porta só** (auditoria 26, §2.6): ela vivia como dois literais —
/// a pista do painel (`populate_camera`) e o clamp do applier (`camera_inspector`) —, e a mutação
/// que subia um deles para `0,99` SOBREVIVIA à bancada inteira. Os dois lêem daqui.
///
/// - **`0,9` em cima é o DOMÍNIO DA LEI** — em `δ = 1` o plano do mundo tem tamanho aparente zero
///   (e a `escala_do_dolly` recusa, auditoria 26 §3).
/// - **`−1` em baixo é a SATURAÇÃO, medida:** o céu lê `1,79×` a `−1`, `2,42×` a `−2` e `2,94×` a
///   `−3` — cada passo compra menos.
pub const DOLLY_MIN: f32 = -1.0; // LITERAL-PX-OK: saturação medida do dolly
/// Ver [`DOLLY_MIN`].
pub const DOLLY_MAX: f32 = 0.9; // LITERAL-PX-OK: domínio da lei do dolly

/// Quem ela segue, quando segue.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorCameraFollow {
    pub target: String,
    pub damping: [f32; 2],
    pub dead_zone: [f32; 2],
    pub lookahead: [f32; 2],
    pub offset: [f32; 2],
    /// ⭐ **O nome resolve para alguém na cena?** Derivado do MUNDO — ver o doc do módulo.
    pub target_found: bool,
}

/// A cerca da fase, quando existe.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorCameraLimits {
    pub min: [f32; 2],
    pub max: [f32; 2],
    /// ⭐⭐ **A caixa é mais estreita que a janela visível** — nalgum eixo.
    ///
    /// ⚠️ **Derivado da JANELA**, e por isso impossível de derivar no painel a partir dos quatro
    /// números: depende da proporção do ecrã e da altura da câmera. Quando isto é verdade a câmera
    /// **fixa-se no centro da caixa** e deixa de seguir seja quem for — comportamento medido no
    /// oráculo, e o único aviso que separa *«a cerca está apertada»* de *«a câmera avariou»*.
    pub smaller_than_view: bool,
}

/// Snapshot da secção CAMERA da entidade selecionada.
///
/// ⚠️ **Ela existe se o objecto tiver a CÂMERA** — o `follow` e os `limits` são corpos opcionais
/// dentro dela. É o ADR-0166: *o Inspector mostra o que o objecto TEM*.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorCameraInfo {
    pub entity_bits: u64,
    pub camera: InspectorGameCamera,
    pub follow: Option<InspectorCameraFollow>,
    pub limits: Option<InspectorCameraLimits>,
    /// Quantas câmeras a cena tem. ⚠️ `1` é o caso comum e não se anuncia.
    pub camera_count: usize,
    /// ⭐ **Esta é a que MANDA?** Com várias, ganha a de maior prioridade (desempate por
    /// identidade) — e o painel tem de o dizer, senão o artista afina uma que ninguém usa.
    pub is_active_camera: bool,
    /// A vista está a ser conduzida pela câmera da cena AGORA? ⚠️ Estado de VISTA, não documento.
    pub preview_on: bool,
    pub selected_count: usize,
}

/// Uma edição de um campo da secção CAMERA.
#[derive(Clone, Debug, PartialEq)]
pub enum CameraFieldEdit {
    Height(f32),
    Offset([f32; 2]),
    Priority(i32),
    /// ⭐⭐⭐ **O DOLLY** (plano 24, W5) — o único campo desta secção que não muda o que a câmera
    /// enquadra: ele muda a ESCALA com que cada plano de paralaxe é desenhado.
    ///
    /// ⚠️ **Ele é adimensional de propósito** (uma fracção da distância focal): a lei depende só de
    /// `k` e de `d/z₀`, logo exprimi-lo em metros obrigaria a inventar um `z₀` que esta câmera não
    /// tem. Ver [`ph2d_ecs::ScrollFactor::escala_do_dolly`].
    Dolly(f32),
    Active(bool),
    /// Um bit da máscara de camadas — `(bit, ligado)`.
    CullBit(u8, bool),
    /// ⭐ **Olhar pela câmera da cena.** ⚠️ Não escreve nada no documento — é um gesto de editor,
    /// como o `Preview` do áudio e o transporte da §11.
    Preview(bool),
    Target(String),
    Damping([f32; 2]),
    DeadZone([f32; 2]),
    Lookahead([f32; 2]),
    FollowOffset([f32; 2]),
    LimitMin([f32; 2]),
    LimitMax([f32; 2]),
}
