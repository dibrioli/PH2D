//! ⭐ **A SAÍDA do Quadro** (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 E1–E7): o que se exporta (o
//! quadro, a selecção, os frames — [`Page`]), a gravação do desenho de uma página ([`paint_page`]: o
//! MESMO `ph2d_board_render::paint` do ecrã, no modo de export) e os dois escritores vectoriais que
//! a leem ([`svg::write`], [`pdf::write`]). O PNG é a mesma cena pela placa (a família do quadro tem o
//! dispositivo).

mod glyphs;
pub mod pdf;
pub mod png;
pub mod svg;

use std::collections::BTreeSet;

use ph2d_board_model::{Board, BoardOp, Camera, ElementId, End};
pub use ph2d_board_render::ExportTarget;
use ph2d_board_render::{Detail, ExportLook, Images, RenderCache};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color, VectorScene};

/// E2 — a margem à volta da área do quadro ou da selecção, em unidades do mundo: o `exportPadding`
/// de nascença do Excalidraw (oráculo, §1.3).
pub const PAD: f64 = 10.0;

/// E3 — os tamanhos do PNG, em MEGAPÍXEIS (área, não factor), medidos nos diálogos oficiais do Miro
/// (§1.2: Small 2449×1632, Medium 4898×3265, Large 8573×5715).
pub const PNG_MEGAPIXELS: [f64; 3] = [4.0, 16.0, 49.0];

/// O tema dos tokens de INTERFACE que o desenho ainda lê no export (a borda de um frame): um claro.
/// A tinta do documento lê-se contra o papel (`ThemeInk::over`), não contra ele.
pub const EXPORT_THEME: Theme = Theme::Blueprint;

/// Uma página (PDF) ou imagem (PNG/SVG): a área do mundo `[x0, y0, x1, y1]` e, se é um frame, qual —
/// então só ele e o que vive dentro dele se desenham (o Miro: *«only widgets inside of frames»*).
#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    pub rect: [f64; 4],
    pub frame: Option<ElementId>,
}

impl Page {
    #[must_use]
    pub fn size(&self) -> [f64; 2] {
        [self.rect[2] - self.rect[0], self.rect[3] - self.rect[1]]
    }
}

fn union(a: Option<[f64; 4]>, b: [f64; 4]) -> Option<[f64; 4]> {
    Some(a.map_or(b, |a| {
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    }))
}

fn padded(r: [f64; 4]) -> [f64; 4] {
    [r[0] - PAD, r[1] - PAD, r[2] + PAD, r[3] + PAD]
}

/// A caixa de `ids` (ou de tudo, com `None`): as caixas dos elementos e as rotas das setas.
fn extent(
    board: &Board,
    routes: &RouteCache,
    ids: Option<&BTreeSet<ElementId>>,
) -> Option<[f64; 4]> {
    let mut r = None;
    for el in board.doc.live() {
        if ids.is_some_and(|s| !s.contains(&el.id)) {
            continue;
        }
        if el.connector().is_some() {
            if let Some(rt) = routes.get(el.id) {
                r = union(r, rt.bbox);
            }
        } else {
            r = union(r, el.aabb());
        }
    }
    r
}

/// E2 — o quadro inteiro (a caixa dos vivos + [`PAD`]). `None` = vazio.
#[must_use]
pub fn board_page(board: &Board, routes: &RouteCache) -> Option<Page> {
    extent(board, routes, None).map(|r| Page {
        rect: padded(r),
        frame: None,
    })
}

/// E2 — a selecção (e o que vive dentro dos frames dela) + [`PAD`]. `None` = nada vivo nela.
#[must_use]
pub fn selection_page(board: &Board, routes: &RouteCache, selection: &[ElementId]) -> Option<Page> {
    let mut ids: BTreeSet<ElementId> = selection.iter().copied().collect();
    for &s in selection {
        ids.extend(board.doc.descendants(s));
    }
    extent(board, routes, Some(&ids)).map(|r| Page {
        rect: padded(r),
        frame: None,
    })
}

/// E4 — as páginas do PDF: um frame por página. Sem selecção, os diapositivos (a ordem do painel,
/// sem os escondidos), senão o quadro numa página; com selecção, os frames de TOPO dela, senão a
/// caixa dela numa página (a auditoria W8 viu a selecção de uma nota exportar o deck inteiro).
#[must_use]
pub fn pdf_pages(board: &Board, routes: &RouteCache, selection: &[ElementId]) -> Vec<Page> {
    if selection.is_empty() {
        let slides = ph2d_board_edit::slides(board);
        if slides.is_empty() {
            return board_page(board, routes).into_iter().collect();
        }
        return slides
            .into_iter()
            .filter_map(|id| frame_page(board, id))
            .collect();
    }
    let chosen: Vec<ElementId> = board
        .doc
        .frames_in_panel_order()
        .into_iter()
        .filter(|f| f.frame.is_none() && selection.contains(&f.id))
        .map(|f| f.id)
        .collect();
    if chosen.is_empty() {
        return selection_page(board, routes, selection)
            .into_iter()
            .collect();
    }
    chosen
        .into_iter()
        .filter_map(|id| frame_page(board, id))
        .collect()
}

/// A página de UM frame: a caixa dele (um frame nunca roda), só ele e o que vive dentro.
#[must_use]
pub fn frame_page(board: &Board, frame: ElementId) -> Option<Page> {
    let f = board.doc.get(frame).filter(|e| e.as_frame().is_some())?;
    Some(Page {
        rect: [f.x, f.y, f.x + f.w, f.y + f.h],
        frame: Some(frame),
    })
}

/// E3 — os píxeis por unidade do mundo de um PNG de `megapixels` sobre `page` (o rácio é o da área).
#[must_use]
pub fn png_scale(page: &Page, megapixels: f64) -> f64 {
    let [w, h] = page.size();
    (megapixels * 1e6 / (w * h).max(f64::EPSILON)).sqrt()
}

/// O quadro só com o frame `frame` e o que vive dentro dele, e as setas cujas pontas presas ficam
/// todas lá dentro (e, soltas, cujo meio cai dentro dele). Uma CÓPIA com lápides: os índices do
/// documento continuam certos pelas mesmas operações de sempre.
fn only_frame(board: &Board, routes: &RouteCache, frame: ElementId) -> Board {
    let mut keep = board.doc.descendants(frame);
    keep.insert(frame);
    let inside = board.doc.get(frame).map(|f| f.aabb());
    let mut out = board.clone();
    let gone: Vec<ElementId> = board
        .doc
        .live_with_hidden()
        .filter(|el| {
            if keep.contains(&el.id) {
                return false;
            }
            let Some(c) = el.connector() else {
                return true;
            };
            let mut bound = c.ends().into_iter().filter_map(|e| match e {
                End::Bound { target, .. } => Some(target),
                End::Free(_) => None,
            });
            let all_in = bound.all(|t| keep.contains(&t));
            let mid_in = routes.get(el.id).zip(inside).is_some_and(|(r, b)| {
                (b[0]..=b[2]).contains(&r.mid[0]) && (b[1]..=b[3]).contains(&r.mid[1])
            });
            !(all_in && mid_in)
        })
        .map(|el| el.id)
        .collect();
    for id in gone {
        BoardOp::Delete(id).apply(&mut out.doc);
    }
    out
}

/// ⭐ W17 — o que as páginas de UM export partilham: as notas de outros com o texto escondido de quem
/// exporta (ADR-0182: saem como no ecrã, com o marcador, nunca com um texto que ele já não lê) e as
/// imagens descodificadas — a mesma fotografia em N frames descodifica-se UMA vez, não uma por página
/// (nem por ladrilho do PNG).
#[derive(Default)]
pub struct ExportCache {
    unread: BTreeSet<ElementId>,
    images: Images,
}

impl ExportCache {
    /// Um export com estas notas escondidas.
    #[must_use]
    pub fn new(unread: BTreeSet<ElementId>) -> Self {
        Self {
            unread,
            images: Images::default(),
        }
    }

    /// As imagens do export (um teste troca-lhes o descodificador).
    pub fn images(&mut self) -> &mut Images {
        &mut self.images
    }
}

/// ⭐ Desenha `page` de `board` a `scale` píxeis por unidade, no modo de export (E6/E7: `paper` =
/// `None` transparente), numa cena que GRAVA (para o SVG e o PDF) — e que a placa desenha para o PNG.
/// `target` = o destino ([`ExportTarget`]: o detalhe e o que uma imagem leva); `shared` = o que as
/// páginas do mesmo export partilham ([`ExportCache`]). Devolve a cena e o tamanho dela em píxeis.
pub fn paint_page(
    board: &Board,
    page: &Page,
    scale: f64,
    paper: Option<Color>,
    target: ExportTarget,
    shared: &mut ExportCache,
    ts: &mut TextSystem,
) -> (VectorScene, [f64; 2]) {
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    // Uma cópia com a câmara da página (o `paint` lê a vista do quadro); 100 mil elementos: ~13 ms.
    let mut view = match page.frame {
        Some(f) => {
            let sub = only_frame(board, &routes, f);
            routes.sync(&sub.doc);
            sub
        }
        None => board.clone(),
    };
    let [w, h] = page.size();
    let size = [w * scale, h * scale];
    view.camera = Camera {
        center_x: (page.rect[0] + page.rect[2]) / 2.0,
        center_y: (page.rect[1] + page.rect[3]) / 2.0,
        zoom: scale,
    };
    let mut cache = RenderCache::default();
    if target.vector() {
        cache.detail = Detail {
            frame_title_only_px: 0.0,
            note_blocks: false,
            sketch_final_below_px: 0.0,
        };
    }
    cache.hide_votes = true;
    cache.unread.clone_from(&shared.unread);
    cache.export = Some(ExportLook { paper, target });
    std::mem::swap(&mut cache.images, &mut shared.images);
    let mut scene = VectorScene::new();
    scene.record();
    let area = [0.0, 0.0, size[0], size[1]];
    ph2d_board_render::paint(
        &view,
        area,
        &mut scene,
        EXPORT_THEME,
        ts,
        &mut cache,
        &routes,
        None,
    );
    std::mem::swap(&mut cache.images, &mut shared.images);
    (scene, size)
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod image_tests;
#[cfg(test)]
#[path = "image_w15_tests.rs"]
mod image_w15_tests;
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
