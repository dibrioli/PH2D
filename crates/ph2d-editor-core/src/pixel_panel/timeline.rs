//! O vocabulário da LINHA DO TEMPO do Pixel (P4): colunas, cels, tags e as acções do menu Frame.

/// Uma coluna da linha do tempo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelFrameCol {
    pub id: u32,
    pub duration_ms: u32,
}

/// Um cel da grelha camadas × frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelCel {
    pub layer: u32,
    pub frame: u32,
    /// O frame do dono da imagem (ele próprio se não for ligado): cels com o mesmo `owner` na mesma
    /// camada partilham a imagem (o Aseprite une-os numa pílula).
    pub owner: u32,
    /// A opacidade da imagem e o z deste cel (o Aseprite marca o z ≠ 0 com um «z»).
    pub opacity: u8,
    pub z: i16,
    /// A imagem não tem células (o cel existe só para ligados).
    pub empty: bool,
}

/// A direcção de uma tag (ids do `.ase`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelDirection {
    #[default]
    Forward,
    Reverse,
    PingPong,
    PingPongReverse,
}

impl PixelDirection {
    pub const ALL: [PixelDirection; 4] = [
        PixelDirection::Forward,
        PixelDirection::Reverse,
        PixelDirection::PingPong,
        PixelDirection::PingPongReverse,
    ];
}

/// Uma tag por cima das colunas `from..=to` (posições).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelTagInfo {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub direction: PixelDirection,
    /// 0 = para sempre.
    pub repeat: u16,
    pub color: [u8; 4],
}

/// O que um intervalo da linha do tempo escolhe (o *range* do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelRangeKind {
    #[default]
    None,
    /// Quadros inteiros (todas as camadas): clique / arrasto nos cabeçalhos.
    Frames,
    /// Cels: as camadas × os quadros (arrasto sobre os cels).
    Cels,
    /// Filas inteiras de camadas (clique no nome).
    Layers,
}

/// O intervalo escolhido na linha do tempo: ids estáveis, os quadros na ordem da linha do tempo.
/// Shift/Ctrl+clique ACRESCENTAM um quadro solto (nunca preenchem o vão); Esc limpa
/// (`docs/PixelArt/ferramentas/oraculo_timeline.md`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PixelRange {
    pub kind: PixelRangeKind,
    pub layers: Vec<u32>,
    pub frames: Vec<u32>,
}

impl PixelRange {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kind == PixelRangeKind::None
    }
}

/// As acções do menu Frame / dos botões da linha do tempo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFrameAction {
    /// Frame ▸ New Frame (copia os cels; liga nas camadas *continuous*).
    NewFrame,
    NewEmptyFrame,
    /// Frame ▸ Duplicate Linked Cels: um frame novo com cada cel ligado ao do activo.
    DuplicateLinked,
    Delete,
    /// Mover o frame activo uma coluna.
    MoveLeft,
    MoveRight,
    /// Frame ▸ Reverse Frames (a tag do activo, ou a linha inteira).
    Reverse,
    /// Navegação: Home / ← / → / End.
    First,
    Previous,
    Next,
    Last,
    /// Enter: tocar / parar.
    Play,
    /// F3: o papel vegetal; e a tinta vermelha/azul.
    Onion,
    OnionTint,
    /// Cel ▸ Links Cels com o frame anterior / Unlink.
    LinkPrevious,
    Unlink,
    /// Frame ▸ Tags ▸ New Tag sobre o frame activo.
    NewTag,
    /// Frame ▸ Jump to ▸ First / Last Frame in Tag: as pontas da tag MAIS INTERIOR que contém o
    /// activo (fora de tags, o primeiro / o último).
    FirstInTag,
    LastInTag,
    /// *Set Loop Section* (F2): a tag «Loop» passa a cobrir o intervalo (nasce na 1.ª vez).
    LoopSection,
    /// Shift+Enter (*Play Preview Animation*): toca / pára SÓ a janela Preview (abre-a); o
    /// editor fica no quadro dele.
    PlayPreview,
    /// F7 (*View ▸ Preview*): abre / fecha a janela Preview.
    Preview,
    /// P13 T: Cel ▸ Tween… — abre o diálogo dos quadros do meio entre os dois cels escolhidos.
    Tween,
}

/// *Playback Speed* (o menu do botão Play do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSpeed {
    Quarter,
    Half,
    #[default]
    One,
    OneAndHalf,
    Two,
    Three,
}

impl PixelSpeed {
    pub const ALL: [PixelSpeed; 6] = [
        PixelSpeed::Quarter,
        PixelSpeed::Half,
        PixelSpeed::One,
        PixelSpeed::OneAndHalf,
        PixelSpeed::Two,
        PixelSpeed::Three,
    ];
}

/// As opções do tocar: a velocidade é do documento; as outras valem para todos (o Trial).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelPlayback {
    pub speed: PixelSpeed,
    pub once: bool,
    /// *Play All Frames (Ignore Tags)*.
    pub all: bool,
    /// *Play Subtags & Repetitions* (ligado por omissão).
    pub subtags: bool,
    /// *Rewind on Stop*: parar volta ao quadro onde o tocar começou.
    pub rewind: bool,
}

impl Default for PixelPlayback {
    fn default() -> Self {
        Self {
            speed: PixelSpeed::One,
            once: false,
            all: false,
            subtags: true,
            rewind: false,
        }
    }
}
