//! As opções de cada janela: ficam de uma abertura para a outra, como no Aseprite.

use ph2d_pixel_io::abrir::Grelha;
use ph2d_pixel_io::render::{Camadas, Direcao, Pedido, Quadros};
use ph2d_pixel_io::saida::Formato;
use ph2d_pixel_io::sheet::{Limite, OpcoesFolha, OpcoesJson};

use ph2d_pixel_core::TilesetId;
use ph2d_pixel_io::tiles_saida::{COLUNAS_INICIAIS, ExportTiles};

use crate::canal::{Contexto, Tela};

/// O Export As.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OpcoesAs {
    pub formato: Formato,
    pub camadas: Camadas,
    pub quadros: Quadros,
    pub direcao: Direcao,
    /// Em %.
    pub escala: u32,
    /// 0 = para sempre.
    pub lacos: u16,
}

impl Default for OpcoesAs {
    fn default() -> Self {
        Self {
            formato: Formato::Png,
            camadas: Camadas::Visiveis,
            quadros: Quadros::Todos,
            direcao: Direcao::Forward,
            escala: PCT,
            lacos: 0,
        }
    }
}

const PCT: u32 = 100;

impl OpcoesAs {
    pub(crate) fn pedido(&self) -> Pedido {
        Pedido {
            camadas: self.camadas,
            quadros: self.quadros,
            direcao: self.direcao,
            escala: f64::from(self.escala) / f64::from(PCT),
            area: Default::default(),
        }
    }
}

/// O Export Sprite Sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpcoesFo {
    pub folha: OpcoesFolha,
    /// O número do limite, guardado ao trocar de tipo de limite.
    pub limite_n: u32,
    pub json_ligado: bool,
    pub json: OpcoesJson,
}

/// O limite de partida ao ligar um: 8 colunas, como o diálogo do Aseprite (foto 31).
const LIMITE_INICIAL: u32 = 8;

impl Default for OpcoesFo {
    fn default() -> Self {
        Self {
            folha: OpcoesFolha::default(),
            limite_n: LIMITE_INICIAL,
            json_ligado: true,
            json: OpcoesJson {
                tags: true,
                ..OpcoesJson::default()
            },
        }
    }
}

impl OpcoesFo {
    /// Põe o número em `limite`, mantendo o tipo escolhido.
    pub(crate) fn acerta_limite(&mut self, n: u32) {
        self.limite_n = n;
        let l = &mut self.folha.limite;
        *l = match *l {
            Limite::Nenhum => Limite::Nenhum,
            Limite::Colunas(_) => Limite::Colunas(n),
            Limite::Linhas(_) => Limite::Linhas(n),
            Limite::Largura(_) => Limite::Largura(n),
            Limite::Altura(_) => Limite::Altura(n),
        };
    }
}

/// O Export Tileset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpcoesTi {
    pub conjunto: Option<u32>,
    pub colunas: u32,
}

impl Default for OpcoesTi {
    fn default() -> Self {
        Self {
            conjunto: None,
            colunas: COLUNAS_INICIAIS,
        }
    }
}

impl OpcoesTi {
    /// O tileset escolhido, ou o último do sprite.
    pub(crate) fn escolhido(&self, x: &Contexto) -> Option<u32> {
        let existe = |id: &u32| x.tilesets.iter().any(|t| t.0 == *id);
        self.conjunto
            .filter(existe)
            .or_else(|| x.tilesets.last().map(|t| t.0))
    }

    pub(crate) fn pedido(&self, x: &Contexto) -> Option<ExportTiles> {
        Some(ExportTiles {
            conjunto: TilesetId(self.escolhido(x)?),
            colunas: self.colunas,
        })
    }
}

/// O tamanho de partida da grelha do Import, sem sugestão: 16 × 16.
pub(crate) const GRELHA_INICIAL: u32 = 16;

pub(crate) fn grelha_inicial() -> Grelha {
    Grelha {
        x: 0,
        y: 0,
        w: GRELHA_INICIAL,
        h: GRELHA_INICIAL,
        gap_x: 0,
        gap_y: 0,
        por_colunas: false,
    }
}

#[derive(Clone)]
pub struct PixelExportPanelState {
    pub(crate) tela: Tela,
    pub(crate) as_: OpcoesAs,
    pub(crate) fo: OpcoesFo,
    pub(crate) tiles: OpcoesTi,
    pub(crate) grelha: Grelha,
    /// A imagem do Import de que a grelha partiu (uma imagem nova traz a sugestão dela).
    pub(crate) grelha_de: Option<u64>,
    /// A resposta na janela do File ▸ Import.
    pub(crate) como_pixel: bool,
    /// A imagem da GPU e o `seq` da publicação de que veio.
    pub(crate) imagem: Option<(u64, ph2d_vector::StableImage)>,
    pub(crate) xadrez: std::sync::Arc<std::sync::Mutex<ph2d_pixel_render::Checker>>,
}

impl Default for PixelExportPanelState {
    fn default() -> Self {
        Self {
            tela: Tela::default(),
            as_: OpcoesAs::default(),
            fo: OpcoesFo::default(),
            tiles: OpcoesTi::default(),
            grelha: grelha_inicial(),
            grelha_de: None,
            como_pixel: false,
            imagem: None,
            xadrez: std::sync::Arc::default(),
        }
    }
}

impl PixelExportPanelState {
    /// A janela em que está (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn tela(&self) -> Tela {
        self.tela
    }
}
