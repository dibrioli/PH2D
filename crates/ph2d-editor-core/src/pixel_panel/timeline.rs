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
}
