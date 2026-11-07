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
    pub stamp: u64,
    /// Ainda não foi aberto em Draw: a primeira entrada enquadra-o (o *Fit Screen*).
    pub(crate) fresh: bool,
    /// Os eixos da simetria em meias células, se o artista os arrastou (`None` = no meio).
    pub axis2: Option<(i32, i32)>,
    /// O último ponto do último traço livre (o `Shift`+clique do Aseprite traça a reta dali).
    pub(crate) last_point: Option<(i32, i32)>,
    /// A animação a tocar (o frame activo segue-a).
    pub(crate) play: Option<ph2d_pixel_core::play::Play>,
    /// A velocidade do tocar deste documento (o Trial: por documento aberto, não se grava).
    pub(crate) speed: ph2d_editor_core::pixel_panel::PixelSpeed,
    /// O intervalo escolhido na linha do tempo ([`crate::intervalo`]).
    pub(crate) range: ph2d_editor_core::pixel_panel::PixelRange,
    /// O intervalo de antes de cada passo por desfazer e o de depois de cada um por refazer: o
    /// Ctrl+Z devolve-o com o passo (Trial 01/11, `mover_ctrl_undo_de_novo`).
    ranges: (
        Vec<ph2d_editor_core::pixel_panel::PixelRange>,
        Vec<ph2d_editor_core::pixel_panel::PixelRange>,
    ),
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
            speed: ph2d_editor_core::pixel_panel::PixelSpeed::One,
            range: ph2d_editor_core::pixel_panel::PixelRange::default(),
            ranges: (Vec::new(), Vec::new()),
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
    /// Os modificadores do último movimento (a caixa da forma que nasce viva ao largar).
    pub(crate) mods: crate::gesture::Mods,
    /// A forma VIVA (P6d): largado o arrasto, a forma flutua com as alças até assentar.
    pub(crate) viva: Option<crate::viva::Viva>,
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
    /// O premir em curso foi DESTA família: só o largar dele é dela, e só com ele o mover é consumido
    /// (o polígono ou a curva à espera não comem o clique do chrome nem o pan do botão do meio).
    pub(crate) pressed: bool,
    /// O cursor está sobre um eixo da simetria que o premir agarraria.
    pub(crate) sobre_eixo: bool,
    pub(crate) hover: Option<(u64, (i32, i32))>,
    pub(crate) next_stamp: u64,
    /// As miniaturas dos cartões do Inspector ([`crate::cartao`]).
    pub(crate) miniaturas: crate::cartao::Miniaturas,
    /// O objecto que nasceu e pede o modo Draw (uma vez).
    pub(crate) wants_edit: Option<u64>,
    /// O objecto em Draw no quadro anterior (para enquadrar à entrada).
    pub(crate) was_editing: Option<u64>,
    /// O papel vegetal (as opções do Trial por omissão) e se está ligado (F3).
    pub onion: ph2d_pixel_core::Onion,
    pub onion_on: bool,
    /// O `F4` pediu mostrar/esconder a linha do tempo (aplica-se no painel, que vê o `hero`).
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
    /// O Pixel Sprite que manda nos painéis e se está em Draw; o Bones a repor depois dele.
    pub(crate) sujeito_do_painel: Option<crate::panel::Sujeito>,
    pub(crate) ossos_abertos: bool,
    /// As cores que as amostras da coluna mostravam no fim do quadro anterior (frente, fundo): o
    /// que o seletor partilhado escrever por cima delas é escolha do artista.
    pub(crate) semeadas: [[u8; 4]; 2],
    /// O degrau de zoom do sprite em Draw (píxeis de ecrã por célula), lido no quadro — o rodapé.
    pub(crate) zoom: Option<u32>,
    /// A área livre do desenho e o retângulo do sprite em Draw no ecrã `(x, y, w, h)` px: os
    /// diálogos de filtro encostam-se na área e fogem do sprite.
    pub(crate) na_tela: Option<ph2d_editor_core::pixel_panel::PixelOnScreen>,
    /// A roda ainda por virar degrau (o trackpad manda o gesto aos bocados).
    pub(crate) roda: crate::input::Roda,
    /// As amostras seleccionadas da paleta (a rampa do Shading; o alvo de Sort/Move).
    pub(crate) sel: std::collections::BTreeSet<usize>,
    /// A âncora do `Shift`+clique e a última amostra clicada (a do *Edit color*).
    pub(crate) ancora: Option<usize>,
    pub(crate) ultima: Option<usize>,
    pub(crate) sort_key: ph2d_editor_core::pixel_panel::PixelSortKey,
    /// O *Remap Palette* pendente: `(chave, onde[índice antigo] = índice novo)` desde o último.
    pub(crate) remap: Option<(u32, Vec<usize>)>,
    /// As opções do tocar que valem para todos os documentos (a `speed` daqui não conta).
    pub(crate) tocar: ph2d_editor_core::pixel_panel::PixelPlayback,
    /// O tocar da janela Preview (o `Doc.frame` não anda) e a chave da imagem dela.
    pub(crate) preview: ph2d_panel_pixel_preview::PreviewPlay,
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
    /// *Paste as New Sprite*: o recorte à espera de que a shell nasça o objecto dele.
    pub(crate) nasce: Option<ph2d_pixel_core::layers::Clip>,
    /// Open e Import Sprite Sheet: o documento à espera de que a shell nasça o objecto dele.
    pub(crate) nasce_doc: Vec<(Sprite, String)>,
    /// A janela de Export e Import ([`crate::exportar`]).
    pub(crate) exporta: ph2d_panel_pixel_export::canal::LadoDaFamilia,
    /// A tela dos tilemaps ([`crate::tiles`]).
    pub(crate) tiles: crate::tiles::Estado,
    pub(crate) slices: crate::slices::Estado,
    /// O gesto de selecção em curso (marquee, laços).
    pub(crate) laco: Option<crate::selecao::Laco>,
    /// A moldura solta (Transform, Ctrl+arrasto com uma selecção, Paste): as células levantadas até
    /// ao Enter/Esc.
    pub(crate) solta: Option<crate::solta::Solta>,
    /// A forma viva que o `Ctrl+Z` largou, o tamanho do desfazer do documento nesse instante e os
    /// campos dela — o `Ctrl+Shift+Z` tra-la de volta enquanto o documento não andou.
    pub(crate) largada: Option<(Gesture, usize, crate::viva::Foto)>,
    /// O relógio das formigas, em ms (o do host, como o tocar).
    pub(crate) formigas_ms: u64,
    /// O diálogo do Pixel a abrir no próximo [`crate::panel::frame`] (lá há o `hero`).
    pub(crate) abre_dialogo: Option<ph2d_editor_core::pixel_panel::PixelDialogKind>,
    /// P8: a pré-visualização do diálogo de filtro aberto (nunca gravada).
    pub(crate) previa_filtro: Option<crate::filtros::Previa>,
    /// Os botões de grupo da coluna: a cara de cada um e o flyout aberto.
    pub(crate) groups: ph2d_editor_core::pixel_panel::PixelGroups,
    /// O Solo em curso: `(chave, camada, a visibilidade de antes)`.
    pub(crate) solo: Option<Solo>,
    /// A pasta das paletas do artista (*Save as Preset* / *as Default*); `None` = só a de fábrica
    /// (os testes e os palcos nunca leem a HOME real).
    pub paletas: Option<std::path::PathBuf>,
    /// Os nomes das predefinições gravadas, lidos da pasta (`None` = ainda por ler).
    pub(crate) predefinicoes: Option<Vec<String>>,
    /// O pincel copiado da selecção (Ctrl+B); `None` = o pincel de forma das opções.
    pub(crate) pincel: Option<std::sync::Arc<ph2d_pixel_core::pattern::PatternBrush>>,
    /// O pincel pinta com as cores da imagem; escolher uma cor de frente põe-no a `false`.
    pub(crate) pincel_imagem: bool,
    /// Ctrl+B sem selecção: o próximo arrasto define o pincel.
    pub(crate) recorte: bool,
    /// A mão do traço livre em curso (P7: dinâmica e corda); `None` = desligadas.
    pub(crate) mao: Option<crate::mao::Mao>,
    /// O Blur/Jumble em curso (P8).
    pub(crate) mancha: Option<crate::degrade::Mancha>,
    /// P8: a caixa do Text aberta (o texto ainda editável, nunca no documento).
    pub(crate) texto: Option<crate::texto::Caixa>,
    /// A pasta dos pincéis guardados (Alt+1…); `None` = só nesta sessão (os testes e os palcos
    /// nunca leem a HOME real).
    pub pinceis: Option<std::path::PathBuf>,
    /// Os espaços (lidos da pasta na 1.ª pergunta).
    pub(crate) espacos: Option<Vec<ph2d_pixel_io::brush_file::BrushSlot>>,
    /// O espaço do pincel na mão (o último criado ou recuperado): destrancado, segue-o.
    pub(crate) espaco_activo: Option<usize>,
    /// O pincel na mão como o espaço activo o viu da última vez (só se relê quando muda).
    pub(crate) espaco_visto: Option<crate::pinceis::Visto>,
}

impl Default for PixelApp {
    fn default() -> Self {
        Self {
            docs: BTreeMap::new(),
            tool: Tool::Pencil,
            options: crate::texto::opcoes_iniciais(),
            fg: [255, 255, 255, 255],
            bg: [0, 0, 0, 255],
            grid: false,
            gesture: None,
            space: false,
            panning: false,
            pressed: false,
            sobre_eixo: false,
            hover: None,
            next_stamp: 1,
            miniaturas: crate::cartao::Miniaturas::new(),
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
            sujeito_do_painel: None,
            ossos_abertos: false,
            semeadas: [[0; 4]; 2],
            zoom: None,
            na_tela: None,
            roda: crate::input::Roda::default(),
            sel: std::collections::BTreeSet::new(),
            ancora: None,
            ultima: None,
            sort_key: ph2d_editor_core::pixel_panel::PixelSortKey::Hue,
            remap: None,
            tocar: ph2d_editor_core::pixel_panel::PixelPlayback::default(),
            preview: Default::default(),
            chave: None,
            edicao: None,
            avisos: Vec::new(),
            abre_color_mode: None,
            clip: None,
            nasce: None,
            nasce_doc: Vec::new(),
            exporta: Default::default(),
            tiles: Default::default(),
            slices: Default::default(),
            laco: None,
            solta: None,
            largada: None,
            formigas_ms: 0,
            abre_dialogo: None,
            previa_filtro: None,
            groups: ph2d_editor_core::pixel_panel::PixelGroups::default(),
            solo: None,
            paletas: None,
            predefinicoes: None,
            pincel: None,
            pincel_imagem: true,
            recorte: false,
            mao: None,
            mancha: None,
            texto: None,
            pinceis: None,
            espacos: None,
            espaco_activo: None,
            espaco_visto: None,
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
            pinceis: crate::pinceis::pasta_do_utilizador(),
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

    /// A cor que a imagem do canvas tem na célula `(x, y)`, como o último quadro a compôs.
    #[must_use]
    pub fn canvas_rgba(&self, key: u32, x: u32, y: u32) -> Option<[u8; 4]> {
        let d = self.docs.get(&key)?;
        d.view.rgba_at(d.sprite.width(), x, y)
    }

    #[doc(hidden)]
    pub fn foca(&mut self, key: u32) -> Option<&mut Doc> {
        self.chave = Some(key);
        self.docs.get_mut(&key)
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
        doc.ranges.0.push(doc.range.clone());
        doc.ranges.1.clear();
        let fora = doc.ranges.0.len().saturating_sub(doc.history.undo_len());
        doc.ranges.0.drain(..fora);
        self.largada = None;
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
        self.remap = self.remap.take().filter(|r| r.0 != key);
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
            let (de, para) = if redo {
                (&mut doc.ranges.1, &mut doc.ranges.0)
            } else {
                (&mut doc.ranges.0, &mut doc.ranges.1)
            };
            if let Some(r) = de.pop() {
                para.push(std::mem::replace(&mut doc.range, r));
            }
            doc.stamp = stamp;
            doc.sane_frame(was);
            crate::camadas::sane_active(doc);
            crate::slices::poda(&mut self.slices, key, &doc.sprite);
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
        self.gesture.is_some() || self.tiles.carimbo.is_some() || self.slices.em_curso()
    }

    /// Há uma moldura solta (células levantadas à espera do Enter/Esc).
    #[must_use]
    pub fn floating(&self) -> bool {
        self.solta.is_some()
    }

    /// A máscara que as formigas da moldura solta do documento `key` contornam (o mesmo valor que
    /// o desenho da vista recebe); `None` = sem moldura nesse documento.
    #[must_use]
    pub fn floating_ants(&mut self, key: u32) -> Option<ph2d_pixel_core::selection::Mask> {
        self.solta.as_ref().filter(|s| s.key == key)?;
        self.previa(key).and_then(|(_, m)| m)
    }
}
