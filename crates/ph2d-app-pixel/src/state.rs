//! O estado da família: os documentos por chave, a ferramenta, as cores e o gesto em curso.

use std::collections::BTreeMap;

use ph2d_pixel_core::{FrameId, History, LayerId, NewSprite, Op, OpError, Sprite, Stroke};
use ph2d_pixel_render::SpriteImage;

/// A ferramenta na mão (Aseprite: `B`, `E`, `I`, `H`). O dono é o vocabulário do chrome (o painel
/// PIXEL mostra-a sem ver a família); aqui é o mesmo tipo.
pub use ph2d_editor_core::pixel_panel::PixelTool as Tool;

/// Um documento de pixels e o que é dele: o desfazer, a camada/frame em edição e a imagem.
pub struct Doc {
    pub sprite: Sprite,
    pub history: History,
    pub layer: LayerId,
    pub frame: FrameId,
    pub(crate) view: SpriteImage,
    /// Muda a cada mudança visível (documento ou traço em curso) — a chave da imagem.
    pub(crate) stamp: u64,
    /// Ainda não foi aberto em Draw: a primeira entrada enquadra-o (o *Fit Screen*).
    pub(crate) fresh: bool,
}

impl Doc {
    pub(crate) fn new(sprite: Sprite, stamp: u64) -> Self {
        let layer = sprite.layers().last().map_or(LayerId(0), |l| l.id);
        let frame = sprite.frames().first().map_or(FrameId(0), |f| f.id);
        Self {
            sprite,
            history: History::default(),
            layer,
            frame,
            view: SpriteImage::default(),
            stamp,
            fresh: true,
        }
    }
}

/// O traço em curso e o objecto onde corre.
pub(crate) struct Gesture {
    pub(crate) entity: u64,
    pub(crate) key: u32,
    pub(crate) stroke: Stroke,
}

/// **O estado da família Pixel Art.**
pub struct PixelApp {
    pub(crate) docs: BTreeMap<u32, Doc>,
    pub tool: Tool,
    /// Cor de frente e de fundo (RGBA de alfa reto). O Aseprite 1.3.18 abre com branco/preto
    /// (`docs/PixelArt/aseprite/01_editor.png`).
    pub fg: [u8; 4],
    pub bg: [u8; 4],
    /// A grelha de pixels (View ▸ Show ▸ Pixel Grid; desligada por omissão, como no Aseprite).
    pub grid: bool,
    pub(crate) gesture: Option<Gesture>,
    /// `Espaço` segurado em Draw (a mão temporária do Aseprite).
    pub(crate) space: bool,
    /// Um arrasto da mão em curso (a shell move a câmera; ao largar, as células realinham).
    pub(crate) panning: bool,
    pub(crate) hover: Option<(u64, (i32, i32))>,
    pub(crate) next_stamp: u64,
    /// O objecto que nasceu e pede o modo Draw (uma vez).
    pub(crate) wants_edit: Option<u64>,
    /// O objecto em Draw no quadro anterior (para enquadrar à entrada).
    pub(crate) was_editing: Option<u64>,
    /// A cena de smoke armada por `PH2D_PIXEL_SMOKE`, encenada no primeiro quadro.
    pub(crate) smoke: Option<u32>,
    /// Os pedidos do painel PIXEL deste quadro, aplicados no [`crate::panel::frame`].
    pub(crate) pedidos: Vec<ph2d_editor_core::pixel_panel::PixelRequest>,
    /// Havia sprite em Draw no quadro anterior (o painel abre/fecha no FLANCO).
    pub(crate) painel_em_edicao: bool,
    /// As cores que as amostras da coluna mostravam no fim do quadro anterior (frente, fundo): o
    /// que o seletor partilhado escrever por cima delas é escolha do artista.
    pub(crate) semeadas: [[u8; 4]; 2],
    /// O degrau de zoom do sprite em Draw (píxeis de ecrã por célula), lido no quadro — o rodapé.
    pub(crate) zoom: Option<u32>,
    /// A roda ainda por virar degrau (o trackpad manda o gesto aos bocados).
    pub(crate) roda: crate::input::Roda,
}

impl Default for PixelApp {
    fn default() -> Self {
        Self {
            docs: BTreeMap::new(),
            tool: Tool::Pencil,
            fg: [255, 255, 255, 255],
            bg: [0, 0, 0, 255],
            grid: false,
            gesture: None,
            space: false,
            panning: false,
            hover: None,
            next_stamp: 1,
            wants_edit: None,
            was_editing: None,
            smoke: None,
            pedidos: Vec::new(),
            painel_em_edicao: false,
            semeadas: [[0; 4]; 2],
            zoom: None,
            roda: crate::input::Roda::default(),
        }
    }
}

impl PixelApp {
    /// O estado com a cena de smoke de `PH2D_PIXEL_SMOKE` armada.
    #[must_use]
    pub fn with_armed_smoke() -> Self {
        Self {
            smoke: crate::smoke::armed_level(),
            ..Self::default()
        }
    }

    pub(crate) fn stamp(&mut self) -> u64 {
        let s = self.next_stamp;
        self.next_stamp += 1;
        s
    }

    /// Cria um documento e devolve a chave nova (a maior + 1, nunca reciclada nesta sessão).
    pub fn create(&mut self, spec: NewSprite) -> Result<u32, ph2d_pixel_core::SpriteError> {
        let sprite = Sprite::new(spec)?;
        let key = self.docs.keys().next_back().map_or(1, |k| k + 1);
        let stamp = self.stamp();
        self.docs.insert(key, Doc::new(sprite, stamp));
        Ok(key)
    }

    #[must_use]
    pub fn doc(&self, key: u32) -> Option<&Doc> {
        self.docs.get(&key)
    }

    /// Os documentos por chave.
    pub fn docs(&self) -> impl Iterator<Item = (u32, &Doc)> {
        self.docs.iter().map(|(k, d)| (*k, d))
    }

    /// Aplica `op` ao documento `key` como UM passo do desfazer dele.
    pub fn commit(&mut self, key: u32, op: Op) -> Result<(), OpError> {
        let stamp = self.stamp();
        let Some(doc) = self.docs.get_mut(&key) else {
            return Ok(());
        };
        doc.history.commit(&mut doc.sprite, op)?;
        doc.stamp = stamp;
        Ok(())
    }

    /// Desfaz no documento `key`. `false` = nada a desfazer.
    pub fn undo(&mut self, key: u32) -> bool {
        self.step(key, false)
    }

    /// Refaz no documento `key`. `false` = nada a refazer.
    pub fn redo(&mut self, key: u32) -> bool {
        self.step(key, true)
    }

    fn step(&mut self, key: u32, redo: bool) -> bool {
        let stamp = self.stamp();
        let Some(doc) = self.docs.get_mut(&key) else {
            return false;
        };
        let done = if redo {
            doc.history.redo(&mut doc.sprite)
        } else {
            doc.history.undo(&mut doc.sprite)
        };
        if done {
            doc.stamp = stamp;
        }
        done
    }

    /// O degrau de zoom do sprite em Draw, em píxeis de ecrã por célula.
    #[must_use]
    pub fn zoom(&self) -> Option<u32> {
        self.zoom
    }

    /// Há um traço em curso.
    #[must_use]
    pub fn stroking(&self) -> bool {
        self.gesture.is_some()
    }
}
