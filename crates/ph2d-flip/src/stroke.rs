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
    /// **Selecionado** (W6 — o Edit Mode). É o atributo `.selection` do GP no domínio
    /// **Curve** (`02_referencia §11`).
    ///
    /// **Por que um CAMPO, e não uma lista de índices no shell:** a identidade de um
    /// traço é a posição dele na `Vec` — e o balde **insere no meio da lista**
    /// (`flip_fill::fill_click` → `strokes.insert(at, …)`), a borracha reescreve a lista
    /// inteira, o undo a restaura. Uma seleção guardada como índices apodrece em silêncio
    /// contra qualquer uma dessas ops: você seleciona um traço, preenche outro, e o painel
    /// recolore o traço errado. **O atributo VIAJA com o traço** e é imune às três.
    ///
    /// (O domínio Point — meio-traço selecionado — vive em [`Self::point_sel`], com a
    /// conversão de domínio explícita do §11.)
    pub selected: bool,
    /// **Seleção no domínio Point** (W8 — `02_referencia §11`): vetor paralelo aos pontos.
    ///
    /// **VAZIO = sem dado de ponto** — a seleção vive no domínio Curve ([`Self::selected`])
    /// e todo ponto herda o estado do traço (o "ausente = broadcast" do GP). Não-vazio ⇒
    /// mesmo comprimento dos arrays SoA **e `selected == any(point_sel)`** — o domínio
    /// Curve é mantido como a PROJEÇÃO `any()` do Point (a regra de promoção do §11,
    /// permanente), para que os consumidores por-traço (painel, delete de traço, máscara
    /// grossa do Sculpt, realce) nunca divirjam do que os pontos dizem.
    ///
    /// Privado de propósito: toda escrita passa pelos choke points ([`Self::set_point_selected`],
    /// [`Self::broadcast_selection_to_points`], [`Self::promote_points_to_stroke`]), que
    /// mantêm as DUAS invariantes acima. Ler é [`Self::point_selected`].
    point_sel: Vec<bool>,
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
            selected: false,
            point_sel: Vec::new(),
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
    ///
    /// **O `selected` NÃO viaja** — um traço novo nasce não-selecionado. Os dois usos
    /// querem coisas opostas: o **tween** produz um traço DERIVADO (um in-between; herdar
    /// o realce pintaria os quadros gerados de selecionado), enquanto a **borracha**
    /// produz os PEDAÇOS do mesmo traço e deve herdar — ela o faz explicitamente, no sítio
    /// onde essa semântica mora (`flip_erase`).
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
            selected: false, // um traço novo nasce não-selecionado (ver acima)
            point_sel: Vec::new(),
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
        // Domínio Point materializado: o ponto novo nasce não-selecionado (o vetor
        // acompanha os arrays SoA; vazio = domínio Curve, nada a manter).
        if !self.point_sel.is_empty() {
            self.point_sel.push(false);
        }
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
        if !self.point_sel.is_empty() {
            self.point_sel.insert(i, false);
        }
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
        if !self.point_sel.is_empty() {
            self.point_sel.remove(i);
            self.selected = self.point_sel.iter().any(|&b| b);
        }
        Some(p)
    }

    /// Verdadeiro se os quatro arrays SoA têm o mesmo comprimento (invariante) — e, com o
    /// domínio Point materializado, se o vetor de seleção acompanha E o Curve é o `any()`
    /// dele. Usado em `debug_assert` e nos testes; nunca deve ser falso em uso normal.
    #[must_use]
    pub fn soa_is_consistent(&self) -> bool {
        let n = self.pos.len();
        let soa = self.width.len() == n && self.opacity.len() == n && self.color.len() == n;
        let sel = self.point_sel.is_empty()
            || (self.point_sel.len() == n && self.selected == self.point_sel.iter().any(|&b| b));
        soa && sel
    }

    // ── Seleção no domínio POINT (W8 — `02_referencia §11`) ──

    /// Há dado de seleção no domínio Point? (Vazio = a seleção vive no Curve.)
    #[must_use]
    pub fn has_point_selection(&self) -> bool {
        !self.point_sel.is_empty()
    }

    /// O ponto `i` está selecionado? Sem dado de ponto, herda o estado do TRAÇO
    /// (o "ausente = broadcast" do GP). Fora do range = `false`.
    #[must_use]
    pub fn point_selected(&self, i: usize) -> bool {
        if self.point_sel.is_empty() {
            return self.selected && i < self.len();
        }
        self.point_sel.get(i).copied().unwrap_or(false)
    }

    /// Escreve a seleção do ponto `i`, **materializando** o domínio Point se preciso
    /// (broadcast do estado do traço — a conversão explícita do §11) e re-derivando o
    /// Curve como `any()`. Devolve `true` se algo mudou. Fora do range = no-op.
    pub fn set_point_selected(&mut self, i: usize, on: bool) -> bool {
        if i >= self.len() {
            return false;
        }
        if self.point_sel.is_empty() {
            self.point_sel = vec![self.selected; self.len()];
        }
        let changed = std::mem::replace(&mut self.point_sel[i], on) != on;
        self.selected = self.point_sel.iter().any(|&b| b);
        changed
    }

    /// **Curve → Point** (troca de domínio): materializa o vetor com o estado do traço
    /// em todo ponto. Idempotente sobre um domínio já materializado? NÃO — sobrescreve
    /// (é a conversão do §11: o dado é do domínio NOVO). No-op num traço vazio.
    pub fn broadcast_selection_to_points(&mut self) {
        if self.is_empty() {
            return;
        }
        self.point_sel = vec![self.selected; self.len()];
    }

    /// **Point → Curve** (troca de domínio): promove `any(point_sel)` ao traço e
    /// DESmaterializa o vetor (half-selected só existe em Point, §11).
    pub fn promote_points_to_stroke(&mut self) {
        if !self.point_sel.is_empty() {
            self.selected = self.point_sel.iter().any(|&b| b);
            self.point_sel.clear();
        }
    }

    /// Os índices dos pontos selecionados (domínio Point materializado; com o vetor
    /// ausente, o broadcast: todos se o traço está selecionado, nenhum senão).
    #[must_use]
    pub fn selected_point_indices(&self) -> Vec<usize> {
        (0..self.len())
            .filter(|&i| self.point_selected(i))
            .collect()
    }

    /// Todos os pontos estão efetivamente selecionados? (Domínio Curve: o estado do
    /// traço. Point: `all()`.) `false` num traço vazio.
    #[must_use]
    pub fn all_points_selected(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        if self.point_sel.is_empty() {
            return self.selected;
        }
        self.point_sel.iter().all(|&b| b)
    }

    /// Translada os pontos SELECIONADOS por `delta`. Os **buracos** só andam quando o
    /// traço INTEIRO está selecionado — eles não têm seleção própria, e mover o contorno
    /// parcialmente é deformação (o anel fica; é o render que re-tria o fill). Devolve
    /// `true` se algo se moveu.
    pub fn translate_selected_points(&mut self, delta: Vec2) -> bool {
        if delta.x == 0.0 && delta.y == 0.0 {
            return false;
        }
        let all = self.all_points_selected();
        let mut moved = false;
        for i in 0..self.len() {
            if self.point_selected(i) {
                self.pos[i] += delta;
                moved = true;
            }
        }
        if moved && all {
            for h in &mut self.holes {
                for p in h.iter_mut() {
                    *p += delta;
                }
            }
        }
        moved
    }

    /// Remove os pontos selecionados (dissolve — o traço continua ligado pelos que
    /// ficam). Devolve quantos saíram. O Curve é re-derivado (vetor sem `true` ⇒
    /// `selected = false`).
    pub fn remove_selected_points(&mut self) -> usize {
        if self.point_sel.is_empty() {
            return 0; // domínio Curve: quem apaga traço inteiro é `delete_selected`
        }
        let keep: Vec<bool> = self.point_sel.iter().map(|&b| !b).collect();
        let removed = keep.iter().filter(|&&k| !k).count();
        if removed == 0 {
            return 0;
        }
        let mut it = keep.iter().copied();
        self.pos.retain(|_| it.next().unwrap_or(true));
        let mut it = keep.iter().copied();
        self.width.retain(|_| it.next().unwrap_or(true));
        let mut it = keep.iter().copied();
        self.opacity.retain(|_| it.next().unwrap_or(true));
        let mut it = keep.iter().copied();
        self.color.retain(|_| it.next().unwrap_or(true));
        self.point_sel.retain(|&b| !b);
        self.selected = self.point_sel.iter().any(|&b| b);
        removed
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

    /// 🔴 **O domínio Curve é a projeção `any()` do Point** — selecionar UM ponto acende
    /// o traço (os consumidores por-traço continuam certos); desmarcar todos o apaga.
    /// Mutação que sangra: `set_point_selected` não re-derivar o `selected`.
    #[test]
    fn the_curve_domain_is_the_any_projection_of_the_points() {
        let mut s = FlipStroke::new();
        for i in 0..4 {
            s.push_default(Vec2::new(i as f32, 0.0));
        }
        assert!(!s.selected);
        assert!(s.set_point_selected(2, true));
        assert!(s.selected, "um ponto aceso tem de ligar o traco (any)");
        assert!(s.soa_is_consistent());
        assert!(s.set_point_selected(2, false));
        assert!(!s.selected, "nenhum ponto aceso = traco apagado");
    }

    /// **A conversão de domínio é a do §11**: Curve→Point é broadcast (traço selecionado
    /// ⇒ todos os pontos); Point→Curve é promoção `any()` + desmaterializa (half-selected
    /// só existe em Point).
    #[test]
    fn domain_conversion_is_broadcast_down_and_any_up() {
        let mut s = FlipStroke::new();
        for i in 0..3 {
            s.push_default(Vec2::new(i as f32, 0.0));
        }
        s.selected = true;
        s.broadcast_selection_to_points();
        assert!((0..3).all(|i| s.point_selected(i)), "broadcast: todos");
        assert!(s.set_point_selected(0, false));
        assert!(s.set_point_selected(1, false));
        s.promote_points_to_stroke();
        assert!(s.selected, "um ponto vivo promove o traco");
        assert!(!s.has_point_selection(), "Point desmaterializado na volta");
    }

    /// **Sem dado de ponto, o ponto herda o TRAÇO** (ausente = broadcast) — é o que faz
    /// a máscara do Sculpt e o overlay lerem `point_selected` sem se importar com o
    /// domínio corrente.
    #[test]
    fn absent_point_data_inherits_the_stroke_state() {
        let mut s = FlipStroke::new();
        s.push_default(Vec2::ZERO);
        assert!(!s.point_selected(0));
        s.selected = true;
        assert!(s.point_selected(0));
        assert!(
            !s.point_selected(99),
            "fora do range nunca esta selecionado"
        );
    }

    /// 🔴 **Dissolver pontos mantém o invariante SoA** (todos os arrays encolhem juntos)
    /// e re-deriva o Curve. Mutação que sangra: esquecer um dos `retain`.
    #[test]
    fn dissolving_selected_points_keeps_the_soa_invariant() {
        let mut s = FlipStroke::new();
        for i in 0..5 {
            s.push_point(Point {
                pos: Vec2::new(i as f32, 0.0),
                width: i as f32,
                opacity: 1.0,
                color: Rgba::WHITE,
            });
        }
        s.set_point_selected(1, true);
        s.set_point_selected(3, true);
        assert_eq!(s.remove_selected_points(), 2);
        assert!(s.soa_is_consistent());
        assert_eq!(s.len(), 3);
        // Os que ficam são os certos (0, 2, 4) — largura carrega a identidade.
        assert_eq!(s.widths(), &[0.0, 2.0, 4.0]);
        assert!(!s.selected, "nada mais selecionado depois do dissolve");
    }

    /// **Os buracos só andam com o traço INTEIRO selecionado** — eles não têm seleção
    /// própria; num move parcial (deformação) o anel fica, e o fill re-tria no render.
    #[test]
    fn holes_follow_only_a_fully_selected_stroke() {
        let mut s = FlipStroke::new();
        for i in 0..3 {
            s.push_default(Vec2::new(i as f32, 0.0));
        }
        s.holes.push(vec![Vec2::new(5.0, 5.0)]);
        // Parcial: o buraco fica.
        s.set_point_selected(0, true);
        assert!(s.translate_selected_points(Vec2::new(1.0, 0.0)));
        assert_eq!(s.holes[0][0], Vec2::new(5.0, 5.0));
        // Inteiro: o buraco anda.
        for i in 0..3 {
            s.set_point_selected(i, true);
        }
        assert!(s.translate_selected_points(Vec2::new(1.0, 0.0)));
        assert_eq!(s.holes[0][0], Vec2::new(6.0, 5.0));
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
