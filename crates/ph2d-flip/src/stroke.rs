//! [`FlipStroke`] — uma polilinha desenhada, **SoA** (structure-of-arrays) por
//! atributo, pronta pro upload GPU do W1.
//!
//! Por-ponto: posição, largura, opacidade, cor. Por-curva: fechada, caps,
//! hardness, material, fill. Os quatro arrays por-ponto têm **o mesmo
//! comprimento** — o invariante que a API `push_point`/`insert_point` mantém.
//! Defaults vêm da tabela de atributos do GP (`02_referencia §1`).

use crate::color::Rgba;
use crate::ids::MaterialId;
use ph2d_core::Vec2;
use serde::{Deserialize, Serialize};

/// Largura default de ponto, em unidades de mundo. É o `radius` default do GP
/// (`BKE_grease_pencil.hh:196` = `0.01`); a espessura em pixels é `raio·zoom` no
/// render (2D-ortográfico, W1).
pub const DEFAULT_WIDTH: f32 = 0.01;
/// Opacidade default de ponto (= "strength" do GP).
pub const DEFAULT_OPACITY: f32 = 1.0;
/// Hardness default de curva. O GP guarda `softness` (default `0.0`); a hardness
/// efetiva é `1 - softness` = `1.0` (borda dura).
pub const DEFAULT_HARDNESS: f32 = 1.0;

/// Ponta do traço. `Round` arredonda (default do GP), `Flat` corta reto.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Cap {
    #[default]
    Round,
    Flat,
}

/// Preenchimento por-curva. Ausente (`None` no [`FlipStroke::fill`]) = traço sem
/// fill (o `fill_id == 0` do GP). O agrupamento de fills compostos (várias curvas
/// = um fill com buracos) é do W1/W4 — aqui a cor e opacidade bastam.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    /// Cor do preenchimento.
    pub color: Rgba,
    /// Opacidade do preenchimento (`fill_opacity`, default `1.0`).
    pub opacity: f32,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            color: Rgba::WHITE,
            opacity: 1.0,
        }
    }
}

/// Uma amostra de ponto (view de conveniência pra construir/ler um ponto SoA sem
/// indexar os quatro arrays na mão).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub pos: Vec2,
    pub width: f32,
    pub opacity: f32,
    pub color: Rgba,
}

impl Point {
    /// Ponto na posição `pos` com os defaults de largura/opacidade/cor (§1).
    #[must_use]
    pub fn at(pos: Vec2) -> Self {
        Self {
            pos,
            width: DEFAULT_WIDTH,
            opacity: DEFAULT_OPACITY,
            color: Rgba::WHITE,
        }
    }
}

/// Um traço: polilinha SoA + atributos por-curva.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlipStroke {
    // ── POR-PONTO (SoA): os quatro têm SEMPRE o mesmo comprimento ──
    pos: Vec<Vec2>,
    width: Vec<f32>,
    opacity: Vec<f32>,
    color: Vec<Rgba>,
    // ── POR-CURVA ──
    /// Traço fechado (cíclico).
    pub closed: bool,
    /// Pontas (início, fim).
    pub cap: (Cap, Cap),
    /// Dureza da borda `[0,1]` — `1` = dura, `0` = airbrush (`1 - softness`).
    pub hardness: f32,
    /// Material (paleta). `0` = default.
    pub material: MaterialId,
    /// Preenchimento, se houver.
    pub fill: Option<Fill>,
    /// **Os BURACOS do preenchimento** (W4): anéis fechados, no mesmo espaço dos
    /// pontos, que são SUBTRAÍDOS da área preenchida. Vazio no caso comum.
    ///
    /// A letra "O" é o caso trivial que exige isto — e é por isso que o resultado do
    /// balde é UM traço (o contorno externo) que carrega os seus buracos, e não N
    /// traços agrupados por um `fill_id`: um fill é **uma** unidade de seleção, de
    /// undo, de delete e de animação (a promessa do Grease Pencil, `02 §6`), e um
    /// grupo em que apagar um anel destrói a forma não é isso.
    pub holes: Vec<Vec<Vec2>>,
    /// **Só o preenchimento aparece** — o contorno deste traço não é rasterizado
    /// (`hide_stroke` do GP). É o que o balde produz: o preenchimento entra POR BAIXO
    /// do line-art que já existe, sem desenhar um segundo contorno em cima dele.
    pub hide_stroke: bool,
}

impl Default for FlipStroke {
    fn default() -> Self {
        Self {
            pos: Vec::new(),
            width: Vec::new(),
            opacity: Vec::new(),
            color: Vec::new(),
            closed: false,
            cap: (Cap::Round, Cap::Round),
            hardness: DEFAULT_HARDNESS,
            material: MaterialId::default(),
            fill: None,
            holes: Vec::new(),
            hide_stroke: false,
        }
    }
}

impl FlipStroke {
    /// Traço vazio com atributos default.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Número de pontos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pos.len()
    }

    /// Sem pontos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }

    // ── acesso SoA (imutável) ──
    #[must_use]
    pub fn positions(&self) -> &[Vec2] {
        &self.pos
    }
    #[must_use]
    pub fn widths(&self) -> &[f32] {
        &self.width
    }
    #[must_use]
    pub fn opacities(&self) -> &[f32] {
        &self.opacity
    }
    #[must_use]
    pub fn colors(&self) -> &[Rgba] {
        &self.color
    }

    // ── acesso SoA (mutável — o comprimento NÃO deve mudar por aqui; use
    //    push/insert/remove para isso, senão o invariante quebra) ──
    pub fn positions_mut(&mut self) -> &mut [Vec2] {
        &mut self.pos
    }
    pub fn widths_mut(&mut self) -> &mut [f32] {
        &mut self.width
    }
    pub fn opacities_mut(&mut self) -> &mut [f32] {
        &mut self.opacity
    }
    pub fn colors_mut(&mut self) -> &mut [Rgba] {
        &mut self.color
    }

    /// Um traço VAZIO com os mesmos atributos de curva (fechado, caps, hardness,
    /// material, fill) — o molde para reconstruir a polilinha ponto a ponto (o
    /// tween, o offset, o refit). Os arrays SoA saem vazios, invariante intacto.
    #[must_use]
    pub fn clone_attrs(&self) -> Self {
        Self {
            pos: Vec::new(),
            width: Vec::new(),
            opacity: Vec::new(),
            color: Vec::new(),
            closed: self.closed,
            cap: self.cap,
            hardness: self.hardness,
            material: self.material,
            fill: self.fill,
            holes: self.holes.clone(),
            hide_stroke: self.hide_stroke,
        }
    }

    /// Lê o ponto `i` (view). `None` fora do range.
    #[must_use]
    pub fn point(&self, i: usize) -> Option<Point> {
        Some(Point {
            pos: *self.pos.get(i)?,
            width: self.width[i],
            opacity: self.opacity[i],
            color: self.color[i],
        })
    }

    /// Acrescenta um ponto ao fim (mantém o invariante SoA).
    pub fn push_point(&mut self, p: Point) {
        self.pos.push(p.pos);
        self.width.push(p.width);
        self.opacity.push(p.opacity);
        self.color.push(p.color);
    }

    /// Acrescenta um ponto na posição `pos` com os atributos default.
    pub fn push_default(&mut self, pos: Vec2) {
        self.push_point(Point::at(pos));
    }

    /// Insere um ponto no índice `i` (clampado a `len`). Mantém o invariante.
    pub fn insert_point(&mut self, i: usize, p: Point) {
        let i = i.min(self.len());
        self.pos.insert(i, p.pos);
        self.width.insert(i, p.width);
        self.opacity.insert(i, p.opacity);
        self.color.insert(i, p.color);
    }

    /// Remove o ponto `i`; devolve-o se existia.
    pub fn remove_point(&mut self, i: usize) -> Option<Point> {
        if i >= self.len() {
            return None;
        }
        let p = Point {
            pos: self.pos.remove(i),
            width: self.width.remove(i),
            opacity: self.opacity.remove(i),
            color: self.color.remove(i),
        };
        Some(p)
    }

    /// Verdadeiro se os quatro arrays SoA têm o mesmo comprimento (invariante).
    /// Usado em `debug_assert` e nos testes; nunca deve ser falso em uso normal.
    #[must_use]
    pub fn soa_is_consistent(&self) -> bool {
        let n = self.pos.len();
        self.width.len() == n && self.opacity.len() == n && self.color.len() == n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_insert_keep_soa_consistent() {
        let mut s = FlipStroke::new();
        assert!(s.is_empty());
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_point(Point {
            pos: Vec2::new(1.0, 0.0),
            width: 2.0,
            opacity: 0.5,
            color: Rgba::BLACK,
        });
        assert_eq!(s.len(), 2);
        assert!(s.soa_is_consistent());

        s.insert_point(1, Point::at(Vec2::new(0.5, 0.0)));
        assert_eq!(s.len(), 3);
        assert!(s.soa_is_consistent());
        assert_eq!(s.point(1).unwrap().pos, Vec2::new(0.5, 0.0));

        // Insert além do fim clampa pra len (append).
        s.insert_point(999, Point::at(Vec2::new(9.0, 9.0)));
        assert_eq!(s.point(3).unwrap().pos, Vec2::new(9.0, 9.0));

        let removed = s.remove_point(0).unwrap();
        assert_eq!(removed.pos, Vec2::new(0.0, 0.0));
        assert_eq!(s.len(), 3);
        assert!(s.soa_is_consistent());
    }

    #[test]
    fn defaults_match_reference_table() {
        let s = FlipStroke::new();
        assert_eq!(s.hardness, DEFAULT_HARDNESS);
        assert_eq!(s.cap, (Cap::Round, Cap::Round));
        assert!(!s.closed);
        assert_eq!(s.material, MaterialId(0));
        assert!(s.fill.is_none());

        let p = Point::at(Vec2::new(1.0, 2.0));
        assert_eq!(p.width, DEFAULT_WIDTH);
        assert_eq!(p.opacity, DEFAULT_OPACITY);
        assert_eq!(p.color, Rgba::WHITE);
    }

    #[test]
    fn round_trips_through_postcard() {
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 1.0));
        s.closed = true;
        s.fill = Some(Fill {
            color: Rgba::BLACK,
            opacity: 0.8,
        });
        let bytes = postcard::to_allocvec(&s).unwrap();
        let back: FlipStroke = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, s);
    }
}
