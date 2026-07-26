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

/// **A PONTA do pincel ao longo do traço** — o *tip* pontilhado do Ciallo/GP (03 §8).
/// `Continuous` é a linha cheia de sempre; `Dots`/`Squares` recortam a cobertura em contas
/// espaçadas por ARC-LENGTH (não por densidade de input — dois traços da mesma forma pontilham
/// igual, tenha um 10 pontos e o outro 1000). O tamanho da conta é a espessura do traço; o
/// espaçamento entre contas é [`FlipStroke::dot_spacing`], **relativo à espessura** (para o
/// padrão aparecer em qualquer grossura).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StrokeTip {
    /// A linha cheia — o comportamento de sempre (byte-idêntico ao traço sem *tip*).
    #[default]
    Continuous,
    /// Contas REDONDAS (discos) ao longo do arco.
    Dots,
    /// Contas QUADRADAS ao longo do arco.
    Squares,
}

/// Espaçamento default entre contas quando o *tip* é pontilhado, como **MÚLTIPLO do diâmetro
/// do traço** (relativo à espessura, não mundo absoluto — senão um traço grosso funde as
/// contas). `2.0` = centros a 2 diâmetros ⇒ vão de 1 diâmetro (contas claramente separadas);
/// `1.0` = contas encostadas; `0` = ignorado (o `Continuous` nem lê). O artista ajusta.
pub const DEFAULT_DOT_SPACING: f32 = 2.0;

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
    /// **A ponta ao longo do traço** ([`StrokeTip`]): linha cheia (default) ou contas
    /// pontilhadas/quadradas. É atributo POR-CURVA (o pincel decide), como `hardness`.
    pub tip: StrokeTip,
    /// **O espaçamento centro-a-centro das contas** como MÚLTIPLO do diâmetro do traço,
    /// quando [`Self::tip`] é pontilhado. Ignorado no `Continuous`. Relativo à ESPESSURA (não
    /// mundo absoluto): a pitch escala com a grossura, então traço grosso e fino mostram o
    /// padrão igual (o report do Enio 2026-07-25 — em mundo absoluto o grosso fundia as
    /// contas). `2.0` = vão de 1 diâmetro; `1.0` = encostadas.
    pub dot_spacing: f32,
    /// **Auto-sobreposição com ACÚMULO** (03 §8) — quando o traço cruza a si mesmo (um laço,
    /// um X, um rabisco que volta), a tinta **ESCURECE** onde as passagens se sobrepõem, em vez
    /// de ficar a união chapada. É o `GP_STROKE_OVERLAP` do Grease Pencil (opção de material lá),
    /// para marcador/nanquim expressivo. Atributo POR-CURVA (o pincel decide), como [`Self::tip`].
    ///
    /// **O que muda no render:** a profundidade passa de por-TRAÇO para por-SEGMENTO, então as
    /// faces sobrepostas de partes diferentes do traço **BLENDam** (premult over) em vez de serem
    /// descartadas pelo depth GREATER estrito. `false` (o default) é **byte-idêntico** ao traço de
    /// sempre (o bit `FLAG_SELF_OVERLAP` fica 0, o ramo do shader nem roda). Só é visível com
    /// opacidade < 1 (a tinta opaca já satura). Ver `docs/Flip/03_traco_rasterizacao.md` §8 e
    /// `flip.wgsl`.
    pub self_overlap: bool,
    /// **Pincel AIRBRUSH analítico** (Ciallo, 03 §8) — o falloff deixa de ser o `pow`+smoothstep
    /// e vira a **transmitância física** da tinta por um dab esférico (Beer-Lambert):
    /// `A(dn) = 1 − exp(−k·√(1−dn²))`, onde `dn` é a distância normalizada à linha-de-centro e
    /// `k = mix(1, 8, hardness)` é a densidade (o slider Hardness vira a densidade da névoa). É um
    /// **domo largo** de núcleo chato e borda SEMPRE macia — o oposto do pico estreito do `pow`. Casa
    /// com [`Self::self_overlap`]: a acumulação `over` de airbrush é a multiplicação de transmitâncias,
    /// que é o build-up físico correto. Atributo POR-CURVA (o pincel decide), como [`Self::tip`].
    ///
    /// `false` (o default) é **byte-idêntico** (o bit `FLAG_AIRBRUSH` fica 0, o ramo do shader nem
    /// roda). Ver `docs/Flip/03_traco_rasterizacao.md` §8 e `flip.wgsl` (`hardness_mask`).
    pub airbrush: bool,
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
    /// Privado de propósito: toda escrita passa pelos choke points ([`Self::set_point_selected`]
    /// e [`Self::promote_points_to_stroke`]), que mantêm as DUAS invariantes acima. Ler é
    /// [`Self::point_selected`].
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
            tip: StrokeTip::Continuous,
            dot_spacing: DEFAULT_DOT_SPACING,
            self_overlap: false,
            airbrush: false,
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

    /// **Os SEGMENTOS da polilinha** — `(i, a, b)`, com `i` = índice do ponto de PARTIDA
    /// (de onde o consumidor tira espessura/atributo). Menos de 2 pontos = vazio.
    ///
    /// **Um traço `closed` inclui a COSTURA** (último → primeiro); um traço aberto NUNCA
    /// a ganha (desenhar/apontar a linha que liga as pontas mostraria algo que o usuário
    /// não fez — BUGS #17).
    ///
    /// **Por que isto é uma porta só:** "quais são os segmentos deste traço?" tinha
    /// QUATRO respostas independentes — o render (`ph2d_flip_render::pack::stroke_segments`)
    /// e o halo fechavam; o pick (`flip_select::hits`), o marquee
    /// (`flip_edit_gesture::stroke_touches_rect`) e o hover de arte (`flip_gizmo_view`)
    /// iteravam `positions().windows(2)` e **perdiam a costura**. O sintoma (Enio,
    /// smoke do §4.A): *"uma linha do triângulo e uma linha do quadrado não são sensíveis
    /// à seleção"* — dava para VER e REALÇAR uma aresta que não dava para APONTAR. Esta é
    /// a resposta única; a convenção espelha a do render (`for a in 0..last` + a costura).
    pub fn segments(&self) -> impl Iterator<Item = (usize, Vec2, Vec2)> + '_ {
        let n = self.pos.len();
        let seam = self.closed && n >= 2;
        (0..n.saturating_sub(1))
            .chain(seam.then(|| n - 1))
            .map(move |i| (i, self.pos[i], self.pos[(i + 1) % n]))
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
            tip: self.tip,
            dot_spacing: self.dot_spacing,
            self_overlap: self.self_overlap,
            airbrush: self.airbrush,
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
#[path = "stroke_tests.rs"]
mod tests;
