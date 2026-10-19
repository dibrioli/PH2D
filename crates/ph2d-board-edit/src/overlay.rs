//! **O que se desenha por cima do quadro** — a selecção, as pegas, as guias, o texto em edição, as
//! setas seleccionadas, os pontos de criação rápida e os rastos (no MUNDO; quem desenha converte).

use std::time::Instant;

use ph2d_board_model::{BoardDoc, Element, ElementId};
use ph2d_text::TextSystem;

use crate::{Editor, Frame, Guide, Metrics, Tool, gesture};

/// O que se desenha por cima do quadro, no MUNDO (quem desenha converte para o ecrã).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overlay {
    /// O contorno fino de cada elemento seleccionado.
    pub boxes: Vec<Frame>,
    /// A moldura com as pegas (`None` sem selecção, ou a escrever).
    pub frame: Option<Frame>,
    /// O rectângulo da selecção por arrasto `[x0, y0, x1, y1]`.
    pub marquee: Option<[f64; 4]>,
    /// As guias do encaixe: direitas, inclinadas (os eixos de um rodado), os vãos iguais e as duas do
    /// ângulo a rodar.
    pub guides: Vec<Guide>,
    /// O texto em edição: o elemento, os rectângulos da selecção e o do cursor, no espaço do
    /// texto (origem no início do bloco, [`ph2d_board_geom::text_origin`]).
    pub text: Option<TextOverlay>,
    /// As setas seleccionadas: a linha (realce) e as duas pontas (pegas).
    pub wires: Vec<Wire>,
    /// A forma onde a ponta arrastada se vai prender (realce), e o ponto fixo, se é um.
    pub target: Option<Target>,
    /// Os pontos azuis de criação rápida: o meio do lado (mundo) e a direcção para fora (unitária)
    /// — quem desenha afasta-os `Metrics::dot` px do lado, no ecrã.
    pub dots: Vec<([f64; 2], [f64; 2])>,
    /// A pega de ARRUMAR EM GRELHA (uma selecção de duas ou mais notas): o canto superior direito
    /// da moldura (mundo) — quem desenha afasta-a [`grid_handle_offset`] px dele, na diagonal.
    pub grid: Option<[f64; 2]>,
    /// O rasto do laser: cada ponto (mundo) com a vida que lhe resta (`1` novo → `0` a sumir).
    pub laser: Vec<([f64; 2], f64)>,
    /// O centro do anel da borracha (mundo) — o raio é `Metrics::eraser` px de ecrã.
    pub eraser: Option<[f64; 2]>,
    /// Os «+» do nó de mapa seleccionado (W6): o meio do lado de fora e a direcção, como os `dots`.
    pub mind_plus: Vec<([f64; 2], [f64; 2])>,
    /// W23 — o «−» de recolher do nó de mapa seleccionado ou pairado: o meio da borda de onde saem os
    /// ramos (mundo) e a direcção para fora — quem desenha afasta-o [`crate::mind_minus_offset`] px.
    pub mind_minus: Option<([f64; 2], [f64; 2])>,
    /// A caixa `[x, y, w, h]` do nó onde o arrasto largaria o nó de mapa como filho (o realce).
    pub mind_drop: Option<[f64; 4]>,
    /// W16 — o recorte aberto: a imagem e os quatro cantos da janela no mundo (as pegas em «L»).
    pub crop: Option<(ElementId, [[f64; 2]; 4])>,
}

/// Quanto a pega de arrumar em grelha se afasta (px de ecrã, nos dois eixos) do canto superior
/// direito da moldura: fora da pega de redimensionar e ao lado da de rodar.
#[must_use]
pub fn grid_handle_offset(m: &Metrics) -> f64 {
    m.handle + m.rotate_offset / 2.0
}

/// Uma seta seleccionada, como o realce a desenha.
#[derive(Clone, Debug, PartialEq)]
pub struct Wire {
    pub path: ph2d_board_route::VecPath,
    pub ends: [[f64; 2]; 2],
    /// Os pontos de ajuste (círculos ocos: arrastam-se; duplo-clique apaga).
    pub points: Vec<[f64; 2]>,
    /// O meio de cada trecho (bolinhas cheias: arrastadas, criam um ponto ali) — menos os trechos em
    /// Z de um cotovelo, que mostram o segmento do meio.
    pub mids: Vec<[f64; 2]>,
    /// W23 B5(1) — o segmento do meio de cada trecho em Z: o meio dele e se é vertical (a pega
    /// arrasta-o na perpendicular).
    pub segments: Vec<([f64; 2], bool)>,
}

/// A forma alvo de uma ligação em curso: realça-se o contorno dela, e a seta ao vivo mostra onde a
/// ponta pára — nenhum ponto no foco (o idioma do Excalidraw, medido na W10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub element: ElementId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextOverlay {
    pub element: ElementId,
    /// Onde começa o bloco de texto, em coordenadas locais da caixa do elemento.
    pub origin: [f64; 2],
    pub selection: Vec<[f64; 4]>,
    pub caret: Option<[f64; 4]>,
}

impl Editor {
    /// O que desenhar por cima do quadro agora.
    pub fn overlay(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem) -> Overlay {
        self.settle_crop(doc);
        if std::mem::take(&mut self.resync) {
            self.sync(doc, ts);
        }
        self.fit_pending(doc, ts);
        self.routes.sync(doc);
        let mid = self.label_mid(doc);
        let text = self.editing.as_mut().and_then(|e| e.overlay(doc, ts, mid));
        let editing = text.is_some();
        let sel = self.selected(doc);
        let shapes: Vec<&&Element> = sel.iter().filter(|el| el.connector().is_none()).collect();
        let mut o = Overlay {
            boxes: if sel.len() > 1 {
                shapes.iter().map(|el| Frame::of(el)).collect()
            } else {
                Vec::new()
            },
            wires: sel
                .iter()
                .filter_map(|el| self.routes.get(el.id))
                .map(|r| Wire {
                    path: r.path.clone(),
                    ends: r.ends(),
                    points: r.waypoints().to_vec(),
                    mids: (0..r.leg_mids.len())
                        .filter(|i| r.elbows.get(*i).copied().flatten().is_none())
                        .map(|i| r.leg_mids[i])
                        .collect(),
                    segments: r
                        .elbows
                        .iter()
                        .flatten()
                        .map(|e| (e.at, e.vertical))
                        .collect(),
                })
                .collect(),
            target: self.gesture.as_ref().and_then(|g| g.target(doc)),
            dots: if editing || self.gesture.is_some() {
                Vec::new()
            } else {
                self.dots(doc)
            },
            frame: if editing { None } else { self.frame(doc) },
            grid: if editing || self.gesture.is_some() {
                None
            } else {
                self.selected_notes(doc)
                    .and_then(|_| self.frame(doc))
                    .map(|f| [f.center[0] + f.w / 2.0, f.center[1] - f.h / 2.0])
            },
            marquee: self.gesture.as_ref().and_then(gesture::Gesture::marquee),
            guides: self
                .gesture
                .as_ref()
                .map(gesture::Gesture::guides)
                .unwrap_or_default(),
            text,
            laser: self.laser_trail(Instant::now()),
            eraser: matches!(self.tool, Tool::Eraser { .. })
                .then_some(self.cursor)
                .flatten(),
            mind_plus: if self.gesture.is_some() {
                Vec::new()
            } else {
                self.mind_plus(doc)
                    .into_iter()
                    .map(|(a, d, _)| (a, d))
                    .collect()
            },
            mind_minus: if self.gesture.is_some() {
                None
            } else {
                self.mind_minus(doc).map(|(_, at, d)| (at, d))
            },
            mind_drop: self
                .gesture
                .as_ref()
                .and(self.mind_drop)
                .and_then(|id| doc.get(id))
                .map(|e| [e.x, e.y, e.w, e.h]),
            crop: None,
        };
        // W16 — no recorte só se vêem as pegas dele (a moldura, os pontos e os «+» saem).
        if let Some(c) = self.crop_corners(doc) {
            o.frame = None;
            o.boxes.clear();
            o.dots.clear();
            o.grid = None;
            o.mind_plus.clear();
            o.mind_minus = None;
            o.crop = Some(c);
        }
        o
    }

    /// Onde vive o rótulo da seta em edição (o meio da rota), se se edita uma.
    fn label_mid(&self, doc: &BoardDoc) -> Option<[f64; 2]> {
        let id = self.editing.as_ref()?.id;
        doc.get(id)?.connector()?;
        self.routes.get(id).map(|r| r.mid)
    }
}
