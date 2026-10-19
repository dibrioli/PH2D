//! **A seta do quadro** (W2): a RELAÇÃO entre duas pontas e os pontos por onde o artista a fez
//! passar — a geometria não se guarda, é derivada (`ph2d-board-route`) e refaz-se quando a seta ou
//! uma das suas formas mexe. ⛔ Nunca por causa de OUTRA forma (ordem do dono, 06/10: «setas não se
//! reajustam sozinhas», o idioma do Miro).
//!
//! ⚠️ postcard é posicional: variante NOVA só no fim de cada enum.

use serde::{Deserialize, Serialize};

use crate::{ElementId, Style};

/// Uma ponta da seta.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum End {
    /// Solta, num ponto do mundo.
    Free([f64; 2]),
    /// Presa a uma forma.
    Bound { target: ElementId, anchor: Anchor },
}

/// Como a ponta presa acha o sítio onde encosta (a semântica do Miro, pesquisa 01 §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    /// Ao CENTRO: a linha sai pela face que olha para a outra ponta, re-escolhida quando as formas
    /// mexem, e nunca atravessa a forma.
    Center,
    /// Num PONTO FIXO `[u, v]` da caixa da forma antes de rodar (`0..1`, o `fixedPoint` do
    /// Excalidraw): roda e escala com ela, e a linha sai sempre dali.
    Fixed([f64; 2]),
    /// ⭐ **Em ÓRBITA do foco `[u, v]`** (W9, o `bindMode "orbit"` do Excalidraw, lei XB1 do
    /// [estudo 07](../../../docs/MiroClone/07_fidelidade_excalidraw.md) §2): a ponta fica na recta
    /// entre os DOIS focos, onde ela entra na faixa de [`ORBIT_GAP`] `+ traço da forma / 2` à volta
    /// do contorno — PÁRA antes da forma. Foco fora da forma e longe dela = a ponta é o foco.
    Orbit([f64; 2]),
}

/// ⭐ O vão entre a ponta de uma seta em [`Anchor::Orbit`] e a forma, ANTES de somar meio traço da
/// forma (XB1, as duas versões do oráculo): `5 + strokeWidth/2` ⇒ 5,5 · 6 · 7 para o traço 1 · 2 · 4.
pub const ORBIT_GAP: f64 = 5.0;

/// Como a rota se desenha.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// Segmentos rectos pelos pontos do artista.
    Straight,
    /// Só segmentos horizontais e verticais (contorna só as suas duas formas).
    Elbow,
    /// A curva suave do Miro, que sai e entra perpendicular às formas e passa pelos pontos do
    /// artista — a de nascença (ordem do dono, 06/10).
    #[default]
    Curved,
    /// ⭐ **A curva do EXCALIDRAW pelos pontos** (W9, lei XA1): Catmull-Rom uniforme com pegas de
    /// 1/6 — com só as duas pontas é RECTA. É a seta `roundness` que vem de um `.excalidraw` (a curva
    /// DELE pelos pontos DELE); a do quadro continua [`Route::Curved`] (ordem do dono, 10/10).
    Spline,
}

/// O desenho numa ponta (o catálogo de pontas do vectorial, `ph2d_vec_scene::Marker`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Head {
    None,
    /// Duas riscas em «V» — a seta de nascença do Excalidraw (`endArrowhead: "arrow"`).
    Arrow,
    Triangle,
    Diamond,
    DiamondOpen,
    Circle,
    CircleOpen,
    Bar,
    /// O triângulo VAZADO (`triangle_outline`, W9).
    TriangleOpen,
    /// As pontas de CARDINALIDADE dos diagramas entidade-relação (o pé-de-galinha do Excalidraw,
    /// `cardinality_*`, W9): um · muitos · um ou muitos · exactamente um · zero ou um · zero ou muitos.
    One,
    Many,
    OneOrMany,
    ExactlyOne,
    ZeroOrOne,
    ZeroOrMany,
}

impl Head {
    /// Todas, na ordem do enum. (A barra oferece as suas, `HEADS`; as outras entram pelo ficheiro.)
    pub const ALL: [Head; 15] = [
        Head::None,
        Head::Arrow,
        Head::Triangle,
        Head::Diamond,
        Head::DiamondOpen,
        Head::Circle,
        Head::CircleOpen,
        Head::Bar,
        Head::TriangleOpen,
        Head::One,
        Head::Many,
        Head::OneOrMany,
        Head::ExactlyOne,
        Head::ZeroOrOne,
        Head::ZeroOrMany,
    ];
}

/// W23 B5(2) — um texto da seta (o *caption* do Miro): fica num ponto da LINHA e anda com ela.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Caption {
    pub text: String,
    /// Onde, em fracção do comprimento da linha desenhada (`0` = início, `1` = fim; o `position` do
    /// Miro e o `labelPosition` do tldraw e do Excalidraw).
    pub at: f64,
}

impl Caption {
    /// Onde nasce o texto de uma seta: a meio.
    pub const MIDDLE: f64 = 0.5;

    #[must_use]
    pub fn new(text: impl Into<String>, at: f64) -> Self {
        Self {
            text: text.into(),
            at: at.clamp(0.0, 1.0),
        }
    }
}

/// Como os textos de uma seta se deitam (o `textOrientation` do Miro, um por seta).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextTurn {
    /// Ao longo da linha, rodados pela direcção dela no ponto do texto (nunca de pernas para o ar) —
    /// o de nascença do Miro.
    #[default]
    Along,
    /// Sempre na horizontal — o do Excalidraw e o dos quadros gravados antes do formato 15.
    Level,
}

/// Uma seta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connector {
    pub start: End,
    pub end: End,
    pub route: Route,
    /// `[início, fim]`.
    pub heads: [Head; 2],
    /// O traço (`stroke`, espessura, tracejado, opacidade) e a letra do rótulo; `fill` não se usa.
    pub style: Style,
    /// Os textos ao longo da linha, pela ordem em que nasceram. Ao fim de uma edição nenhum tem o texto
    /// vazio. (`default`: o JSON de antes do formato 15, no `customData` de um `.excalidraw`, não os
    /// tem — `ph2d-board-excalidraw` traz-lhe o rótulo.)
    #[serde(default)]
    pub captions: Vec<Caption>,
    /// Como os textos se deitam.
    #[serde(default)]
    pub turn: TextTurn,
    /// Os pontos (mundo) por onde o artista fez a seta passar, do início para o fim — os «pontos de
    /// ajuste» do Miro. Vazio = a rota de nascença entre as pontas.
    pub waypoints: Vec<[f64; 2]>,
    /// W23 B5(1) — onde fica o segmento do MEIO de cada trecho do cotovelo (um por trecho entre
    /// estações), em fracção do vão entre os recuos das pontas; `None` = o de nascença. Só o cotovelo
    /// a lê; num trecho que não sai em Z fica guardada sem uso (a lei do tldraw, doc 13). Alinhada
    /// com `waypoints`: mexe-se só por [`Connector::insert_waypoint`], [`Connector::remove_waypoint`]
    /// e [`Connector::set_mid`].
    #[serde(default)]
    pub mids: Vec<Option<f64>>,
}

impl Connector {
    /// As pontas de nascença — as do Excalidraw (oráculo: `startArrowhead: null`,
    /// `endArrowhead: "arrow"`): nada no início, «V» no fim.
    pub const DEFAULT_HEADS: [Head; 2] = [Head::None, Head::Arrow];

    /// Uma seta nova com as pontas de nascença.
    #[must_use]
    pub fn new(start: End, end: End, route: Route, style: Style) -> Self {
        Self {
            start,
            end,
            route,
            heads: Self::DEFAULT_HEADS,
            style,
            captions: Vec::new(),
            turn: TextTurn::Along,
            waypoints: Vec::new(),
            mids: Vec::new(),
        }
    }

    /// Os textos de um rótulo dos formatos antigos (um só, a meio).
    #[must_use]
    pub fn captions_of(label: String) -> Vec<Caption> {
        if label.is_empty() {
            Vec::new()
        } else {
            vec![Caption::new(label, Caption::MIDDLE)]
        }
    }

    /// Tira os textos que ficaram vazios (apagar as letras todas apaga o texto, o Miro).
    pub fn drop_empty_captions(&mut self) {
        self.captions.retain(|c| !c.text.is_empty());
    }

    /// A fracção do segmento do meio do trecho `leg`, se o artista o arrastou.
    #[must_use]
    pub fn mid(&self, leg: usize) -> Option<f64> {
        self.mids.get(leg).copied().flatten()
    }

    /// Fixa (ou larga, com `None`) o segmento do meio do trecho `leg`.
    pub fn set_mid(&mut self, leg: usize, frac: Option<f64>) {
        if frac.is_some() && self.mids.len() <= leg {
            self.mids.resize(leg + 1, None);
        }
        if let Some(m) = self.mids.get_mut(leg) {
            *m = frac;
        }
        self.trim_mids();
    }

    /// Um ponto de ajuste novo em `i`: o trecho `i` parte-se em dois, cada um com o meio de nascença.
    pub fn insert_waypoint(&mut self, i: usize, at: [f64; 2]) {
        self.waypoints.insert(i, at);
        if i < self.mids.len() {
            self.mids[i] = None;
            self.mids.insert(i + 1, None);
        }
        self.trim_mids();
    }

    /// Tira o ponto de ajuste `i`: os dois trechos dele fundem-se num, com o meio de nascença.
    pub fn remove_waypoint(&mut self, i: usize) {
        self.waypoints.remove(i);
        if i < self.mids.len() {
            self.mids[i] = None;
            if i + 1 < self.mids.len() {
                self.mids.remove(i + 1);
            }
        }
        self.trim_mids();
    }

    fn trim_mids(&mut self) {
        let legs = self.waypoints.len() + 1;
        self.mids.truncate(legs);
        while self.mids.last() == Some(&None) {
            self.mids.pop();
        }
    }

    /// As duas pontas `[início, fim]`.
    #[must_use]
    pub fn ends(&self) -> [End; 2] {
        [self.start, self.end]
    }

    pub fn end_mut(&mut self, which: usize) -> &mut End {
        if which == 0 {
            &mut self.start
        } else {
            &mut self.end
        }
    }

    /// As formas a que está presa.
    pub fn targets(&self) -> impl Iterator<Item = ElementId> + '_ {
        self.ends().into_iter().filter_map(|e| match e {
            End::Bound { target, .. } => Some(target),
            End::Free(_) => None,
        })
    }
}

#[cfg(test)]
#[path = "connector_tests.rs"]
mod tests;
