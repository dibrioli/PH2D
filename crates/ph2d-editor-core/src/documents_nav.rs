//! O estado de SESSÃO da navegação de um quadro (MiroClone W5) — do lado dos DOCUMENTOS, para o
//! ecrã depender dele e nunca o contrário (o gate `the_foundation_modules_form_a_dag`). Quem o lê e
//! pinta é `screens::hero::board_nav`.

use crate::zones::Rect;
use ph2d_board_model::ElementId;
use ph2d_vector::VectorScene;

/// O estado de SESSÃO da navegação de um quadro.
#[derive(Default)]
pub(crate) struct Nav {
    pub(crate) frames_open: bool,
    pub(crate) minimap_open: bool,
    pub(crate) search_open: bool,
    /// O campo de pôr uma etiqueta nas notas seleccionadas está aberto.
    pub(crate) tagging: bool,
    /// A etiqueta por que a busca filtra.
    pub(crate) tag_filter: Option<String>,
    /// O que a busca encontrou, por ordem de leitura, e o que está à vista.
    pub(crate) results: Vec<ElementId>,
    pub(crate) current: usize,
    /// O frame cuja linha do painel é, agora, o campo de renomear.
    pub(crate) renaming: Option<ElementId>,
    /// Arrastar no minimapa move a vista.
    pub(crate) minimap_drag: bool,
    pub(crate) minimap_cache: Option<Cache>,
    /// Os elementos com link, refeitos quando o documento muda (revisão).
    pub(crate) links: Option<(u64, Vec<ElementId>)>,
    /// Enquadrar tudo na próxima pintura (quando a área já tem o tamanho dela).
    pub(crate) fit_pending: bool,
    /// O que enquadrar em vez do quadro inteiro (uma cena que reserva a margem dos painéis).
    pub(crate) fit_box: Option<[f64; 4]>,
}

/// O mundo `extent` encaixado (com a proporção dele) no rectângulo `r` do ecrã.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Map {
    pub(crate) extent: [f64; 4],
    pub(crate) scale: f64,
    pub(crate) origin: [f64; 2],
}

impl Map {
    pub(crate) fn new(extent: [f64; 4], r: Rect) -> Self {
        let (ew, eh) = (
            (extent[2] - extent[0]).max(1.0),
            (extent[3] - extent[1]).max(1.0),
        );
        let scale = (f64::from(r.w) / ew).min(f64::from(r.h) / eh);
        let origin = [
            f64::from(r.x) + (f64::from(r.w) - ew * scale) / 2.0,
            f64::from(r.y) + (f64::from(r.h) - eh * scale) / 2.0,
        ];
        Self {
            extent,
            scale,
            origin,
        }
    }

    pub(crate) fn to_screen(self, p: [f64; 2]) -> [f64; 2] {
        [
            self.origin[0] + (p[0] - self.extent[0]) * self.scale,
            self.origin[1] + (p[1] - self.extent[1]) * self.scale,
        ]
    }

    pub(crate) fn to_world(self, p: [f64; 2]) -> [f64; 2] {
        [
            self.extent[0] + (p[0] - self.origin[0]) / self.scale,
            self.extent[1] + (p[1] - self.origin[1]) / self.scale,
        ]
    }
}

/// As células já rasterizadas, válidas para uma revisão do documento e um rectângulo.
pub(crate) struct Cache {
    pub(crate) key: (u64, [u32; 4]),
    pub(crate) map: Map,
    pub(crate) cells: VectorScene,
}
