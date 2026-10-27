//! O estado da família: os documentos por chave, a ferramenta, as cores e o gesto em curso.

use std::collections::BTreeMap;

use ph2d_pixel_core::{FrameId, History, LayerId, NewSprite, Op, OpError, Sprite, Stroke};
use ph2d_pixel_render::SpriteImage;

pub use ph2d_editor_core::pixel_panel::PixelOptions as Options;
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
    /// Os eixos da simetria em meias células, se o artista os arrastou (`None` = no meio).
    pub axis2: Option<(i32, i32)>,
    /// O último ponto do último traço livre (o `Shift`+clique do Aseprite traça a reta dali).
    pub(crate) last_point: Option<(i32, i32)>,
    /// A animação a tocar (o frame activo segue-a).
    pub(crate) play: Option<ph2d_pixel_core::play::Play>,
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
            axis2: None,
            last_point: None,
            play: None,
        }
    }

    /// O frame activo continua a existir depois de um passo (apagar, desfazer um New Frame): senão,
    /// o da posição em que estava (ou o último).
    pub(crate) fn sane_frame(&mut self, was: Option<usize>) {
        if self.sprite.frame_index(self.frame).is_none() {
            let n = self.sprite.frames().len();
            let i = was.unwrap_or(0).min(n.saturating_sub(1));
            if let Some(f) = self.sprite.frames().get(i) {
                self.frame = f.id;
            }
        }
        if let Some(p) = &self.play
            && p.index >= self.sprite.frames().len()
        {
            self.play = None;
        }
    }
}

/// O Solo em curso: `(chave, camada, a visibilidade de antes)`.
pub(crate) type Solo = (u32, LayerId, Vec<(LayerId, bool)>);

/// O gesto em curso e o objecto onde corre.
pub(crate) struct Gesture {
    pub(crate) entity: u64,
    pub(crate) key: u32,
    pub(crate) stroke: Stroke,
    pub(crate) kind: crate::gesture::Kind,
    /// O botão está premido (a curva e o polígono continuam com ele solto).
    pub(crate) held: bool,
    /// A célula do último evento (o `Espaço` desloca a forma por ela).
    pub(crate) cursor: (i32, i32),
}

/// **O estado da família Pixel Art.**
pub struct PixelApp {
    pub(crate) docs: BTreeMap<u32, Doc>,
    pub tool: Tool,
    /// As opções das ferramentas (pincel, balde, spray, simetria, ladrilho).
    pub options: Options,
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
    /// O papel vegetal (as opções do Trial por omissão) e se está ligado (F3).
    pub onion: ph2d_pixel_core::Onion,
    pub onion_on: bool,
    /// O `Tab` pediu mostrar/esconder a linha do tempo (aplica-se no painel, que vê o `hero`).
    pub(crate) alterna_timeline: bool,
    /// A linha do tempo da casa estava aberta antes do Draw (a banda de baixo volta-lhe ao sair).
    pub(crate) timeline_casa: Option<bool>,
    /// O instante do quadro anterior (o relógio do tocar).
    pub(crate) relogio: Option<std::time::Instant>,
    /// O enquadramento de entrada que ainda segue a área livre: `(objecto, área, vista posta)`.
    /// Cai quando o artista mexe na vista, começa um gesto ou sai do Draw.
    pub(crate) fit: Option<(
        u64,
        ph2d_render::camera_frame::Area,
        ph2d_render::camera_frame::View,
    )>,
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
    /// As amostras seleccionadas da paleta (a rampa do Shading; o alvo de Sort/Move).
    pub(crate) sel: std::collections::BTreeSet<usize>,
    /// A âncora do `Shift`+clique e a última amostra clicada (a do *Edit color*).
    pub(crate) ancora: Option<usize>,
    pub(crate) ultima: Option<usize>,
    pub(crate) sort_key: ph2d_editor_core::pixel_panel::PixelSortKey,
    /// O documento em Draw, lido no quadro (os pedidos da paleta aplicam-se a ele).
    pub(crate) chave: Option<u32>,
    /// A edição de cor em curso pelo seletor: `(chave, índice, passos do desfazer depois dela)` —
    /// arrastar no seletor reescreve O MESMO passo.
    pub(crate) edicao: Option<(u32, usize, usize)>,
    /// Os avisos a mostrar (a shell drena-os no quadro).
    pub(crate) avisos: Vec<(bool, String)>,
    /// O diálogo Color Mode a abrir com esta escolha quando houver documento em Draw (a cena de
    /// smoke 5, para a foto).
    pub(crate) abre_color_mode: Option<ph2d_editor_core::pixel_panel::ColorModeChoice>,
    /// A imagem copiada (Edit ▸ Copy), de qualquer sprite.
    pub(crate) clip: Option<crate::edicao::Prancheta>,
    /// O gesto de selecção em curso (marquee, laços).
    pub(crate) laco: Option<crate::selecao::Laco>,
    /// A moldura solta (Transform, Ctrl+arrasto com uma selecção, Paste): as células levantadas até
    /// ao Enter/Esc.
    pub(crate) solta: Option<crate::solta::Solta>,
    /// O relógio das formigas, em ms (o do host, como o tocar).
    pub(crate) formigas_ms: u64,
    /// O diálogo do Pixel a abrir no próximo [`crate::panel::frame`] (lá há o `hero`).
    pub(crate) abre_dialogo: Option<ph2d_editor_core::pixel_panel::PixelDialogKind>,
    /// Os botões de grupo da coluna: a cara de cada um e o flyout aberto.
    pub(crate) groups: ph2d_editor_core::pixel_panel::PixelGroups,
    /// O Solo em curso: `(chave, camada, a visibilidade de antes)`.
    pub(crate) solo: Option<Solo>,
    /// A pasta das paletas do artista (*Save as Preset* / *as Default*); `None` = só a de fábrica
    /// (os testes e os palcos nunca leem a HOME real).
    pub paletas: Option<std::path::PathBuf>,
    /// Os nomes das predefinições gravadas, lidos da pasta (`None` = ainda por ler).
    pub(crate) predefinicoes: Option<Vec<String>>,
}

impl Default for PixelApp {
    fn default() -> Self {
        Self {
            docs: BTreeMap::new(),
            tool: Tool::Pencil,
            options: Options::default(),
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
            onion: ph2d_pixel_core::Onion::default(),
            onion_on: false,
            alterna_timeline: false,
            timeline_casa: None,
            relogio: None,
            fit: None,
            smoke: None,
            pedidos: Vec::new(),
            painel_em_edicao: false,
            semeadas: [[0; 4]; 2],
            zoom: None,
            roda: crate::input::Roda::default(),
            sel: std::collections::BTreeSet::new(),
            ancora: None,
            ultima: None,
            sort_key: ph2d_editor_core::pixel_panel::PixelSortKey::Hue,
            chave: None,
            edicao: None,
            avisos: Vec::new(),
            abre_color_mode: None,
            clip: None,
            laco: None,
            solta: None,
            formigas_ms: 0,
            abre_dialogo: None,
            groups: ph2d_editor_core::pixel_panel::PixelGroups::default(),
            solo: None,
            paletas: None,
            predefinicoes: None,
        }
    }
}

impl PixelApp {
    /// O estado da shell: a cena de smoke de `PH2D_PIXEL_SMOKE` armada e as paletas na pasta do
    /// artista.
    #[must_use]
    pub fn with_armed_smoke() -> Self {
        Self {
            smoke: crate::smoke::armed_level(),
            paletas: crate::paleta::pasta_do_utilizador(),
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
        let sprite = Sprite::with_palette(spec, &self.paleta_de_omissao())?;
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
        let was = doc.sprite.frame_index(doc.frame);
        doc.history.commit(&mut doc.sprite, op)?;
        doc.stamp = stamp;
        doc.sane_frame(was);
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
        // A moldura solta e o laço em curso valem para o documento de agora: desfazer larga-os
        // sem os assentar (o Ctrl+Z do Aseprite com células levantadas).
        self.solta = self.solta.take().filter(|s| s.key != key);
        self.laco = self.laco.take().filter(|l| l.key != key);
        let stamp = self.stamp();
        let Some(doc) = self.docs.get_mut(&key) else {
            return false;
        };
        let was = doc.sprite.frame_index(doc.frame);
        let done = if redo {
            doc.history.redo(&mut doc.sprite)
        } else {
            doc.history.undo(&mut doc.sprite)
        };
        if done {
            doc.stamp = stamp;
            doc.sane_frame(was);
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

    /// Há uma moldura solta (células levantadas à espera do Enter/Esc).
    #[must_use]
    pub fn floating(&self) -> bool {
        self.solta.is_some()
    }
}
