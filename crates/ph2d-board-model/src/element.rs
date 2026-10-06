//! Os elementos de um quadro e o documento que os guarda.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Connector, End, FracKey};

/// Identidade de um elemento dentro do seu quadro. Nunca reusada (as lápides guardam-na).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElementId(pub u64);

/// Cor do DOCUMENTO — dado do artista, não da UI (a paleta por omissão vem dos tokens).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    /// Luminância relativa (WCAG 2), `0` = preto, `1` = branco.
    #[must_use]
    pub fn luminance(self) -> f64 {
        let lin = |c: u8| {
            let c = f64::from(c) / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let [r, g, b, _] = self.0;
        0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
    }

    /// A tinta de texto que se lê sobre esta cor: a escura de nascença ou a clara, a de MAIOR
    /// contraste WCAG.
    #[must_use]
    pub fn readable_ink(self) -> Rgba {
        let (dark, light) = (Rgba(crate::DEFAULT_INK), Rgba(crate::DEFAULT_PAPER));
        let l = self.luminance();
        let contrast = |o: Rgba| {
            let (a, b) = (l.max(o.luminance()), l.min(o.luminance()));
            (a + 0.05) / (b + 0.05)
        };
        if contrast(dark) >= contrast(light) {
            dark
        } else {
            light
        }
    }
}

/// O que um elemento é. ⚠️ postcard é posicional: variante NOVA só no fim, e campo novo numa
/// variante existente sobe o [`crate::FORMAT_VERSION`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ElementKind {
    /// Uma forma com estilo e (talvez) texto dentro.
    Shape(Shape),
    /// Uma seta entre duas pontas (W2). A caixa `x, y, w, h` não se usa: a geometria é a rota.
    /// Numa caixa: com ela inline o `BoardOp::Put` passava os 200 bytes do `large_enum_variant`.
    Connector(Box<Connector>),
}

/// Uma forma do quadro: o contorno, o estilo e o texto que vive DENTRO dela (centrado, com quebra).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub kind: ShapeType,
    pub style: Style,
    /// Texto do artista (pode ter `\n`). Vazio = sem texto.
    pub text: String,
}

/// O contorno de uma forma — o catálogo de quadro (básicas + fluxograma ISO 5807 + as de nota).
/// ⚠️ Append-only (o discriminante vai para o ficheiro). A geometria é do `ph2d-board-geom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Diamond,
    Triangle,
    /// Terminador (início/fim).
    Pill,
    /// Dados (entrada/saída).
    Parallelogram,
    /// Operação manual.
    Trapezoid,
    /// Preparação.
    Hexagon,
    /// Base de dados.
    Cylinder,
    Document,
    /// Processo predefinido (sub-rotina).
    PredefinedProcess,
    /// Ligação fora da página.
    OffPage,
    Delay,
    Display,
    /// Balão de fala rectangular.
    SpeechRect,
    Cloud,
    Star,
    ArrowRight,
}

impl ShapeType {
    /// Todas, na ordem do enum — a que a UI oferece.
    pub const ALL: &'static [ShapeType] = &[
        ShapeType::Rectangle,
        ShapeType::Ellipse,
        ShapeType::Diamond,
        ShapeType::Triangle,
        ShapeType::Pill,
        ShapeType::Parallelogram,
        ShapeType::Trapezoid,
        ShapeType::Hexagon,
        ShapeType::Cylinder,
        ShapeType::Document,
        ShapeType::PredefinedProcess,
        ShapeType::OffPage,
        ShapeType::Delay,
        ShapeType::Display,
        ShapeType::SpeechRect,
        ShapeType::Cloud,
        ShapeType::Star,
        ShapeType::ArrowRight,
    ];
}

/// Traço contínuo, tracejado ou pontilhado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dash {
    Solid,
    Dashed,
    Dotted,
}

/// O aspecto de uma forma. Tamanhos em unidades do MUNDO do quadro (escalam com o zoom).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Style {
    /// `None` = sem preenchimento (transparente).
    pub fill: Option<Rgba>,
    /// `None` = sem contorno.
    pub stroke: Option<Rgba>,
    pub stroke_width: f64,
    pub dash: Dash,
    /// Cantos arredondados (o raio é regra da geometria, não um número livre).
    pub round: bool,
    /// Opacidade do OBJECTO inteiro, `0..=100` (compõe a forma uma vez e depois desvanece).
    pub opacity: u8,
    pub text_color: Rgba,
    pub font_size: f64,
}

/// Um elemento. Os quatro últimos campos são a gramática da colaboração (Etapa 2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub id: ElementId,
    pub kind: ElementKind,
    /// Canto superior esquerdo e tamanho, em unidades do mundo do quadro — da caixa ANTES de rodar.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// Rotação em radianos à volta do centro (positivo = horário no ecrã, y para baixo).
    pub angle: f64,
    /// Ordem em z.
    pub z: FracKey,
    /// Sobe a cada mudança deste elemento.
    pub version: u64,
    /// Desempate entre duas mudanças com a mesma `version` (Etapa 2).
    pub nonce: u32,
    /// Lápide: apagado continua no mapa, para não ressuscitar numa fusão.
    pub deleted: bool,
}

impl Style {
    /// Um estilo com as espessuras e tamanhos de nascença (`DEFAULT_*`), sólido, opaco, sem cantos
    /// redondos — as CORES são de quem chama (o editor tira-as dos tokens).
    #[must_use]
    pub fn new(fill: Option<Rgba>, stroke: Option<Rgba>, text_color: Rgba) -> Self {
        Self {
            fill,
            stroke,
            stroke_width: crate::DEFAULT_STROKE_WIDTH,
            dash: Dash::Solid,
            round: false,
            opacity: 100,
            text_color,
            font_size: crate::DEFAULT_FONT_SIZE,
        }
    }
}

impl Element {
    /// Uma forma nova, sem rotação, na caixa `[x, y, w, h]`.
    #[must_use]
    pub fn new_shape(id: ElementId, z: FracKey, shape: Shape, [x, y, w, h]: [f64; 4]) -> Self {
        Self {
            id,
            kind: ElementKind::Shape(shape),
            x,
            y,
            w,
            h,
            angle: 0.0,
            z,
            version: 0,
            nonce: 0,
            deleted: false,
        }
    }

    /// Uma seta nova (a caixa fica a zero: a geometria é a rota derivada).
    #[must_use]
    pub fn new_connector(id: ElementId, z: FracKey, c: Connector) -> Self {
        Self {
            id,
            kind: ElementKind::Connector(Box::new(c)),
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            angle: 0.0,
            z,
            version: 0,
            nonce: 0,
            deleted: false,
        }
    }

    #[must_use]
    pub fn center(&self) -> [f64; 2] {
        [self.x + self.w / 2.0, self.y + self.h / 2.0]
    }

    /// Ponto do mundo → coordenadas da caixa sem rotação (o mesmo referencial de `x, y, w, h`).
    #[must_use]
    pub fn unrotate(&self, p: [f64; 2]) -> [f64; 2] {
        rotate_about(p, self.center(), -self.angle)
    }

    /// O inverso de [`Self::unrotate`].
    #[must_use]
    pub fn rotate(&self, p: [f64; 2]) -> [f64; 2] {
        rotate_about(p, self.center(), self.angle)
    }

    /// Os quatro cantos no mundo (já rodados), a partir do superior esquerdo, no sentido horário.
    #[must_use]
    pub fn corners(&self) -> [[f64; 2]; 4] {
        let (x0, y0, x1, y1) = (self.x, self.y, self.x + self.w, self.y + self.h);
        [[x0, y0], [x1, y0], [x1, y1], [x0, y1]].map(|p| self.rotate(p))
    }

    /// A caixa alinhada aos eixos que contém o elemento rodado: `[x0, y0, x1, y1]`.
    #[must_use]
    pub fn aabb(&self) -> [f64; 4] {
        let c = self.corners();
        let fold =
            |f: fn(f64, f64) -> f64, i: usize, init: f64| c.iter().map(|p| p[i]).fold(init, f);
        [
            fold(f64::min, 0, f64::INFINITY),
            fold(f64::min, 1, f64::INFINITY),
            fold(f64::max, 0, f64::NEG_INFINITY),
            fold(f64::max, 1, f64::NEG_INFINITY),
        ]
    }

    /// A forma, se é uma.
    #[must_use]
    pub fn shape(&self) -> Option<&Shape> {
        match &self.kind {
            ElementKind::Shape(s) => Some(s),
            ElementKind::Connector(_) => None,
        }
    }

    pub fn shape_mut(&mut self) -> Option<&mut Shape> {
        match &mut self.kind {
            ElementKind::Shape(s) => Some(s),
            ElementKind::Connector(_) => None,
        }
    }

    /// A seta, se é uma.
    #[must_use]
    pub fn connector(&self) -> Option<&Connector> {
        match &self.kind {
            ElementKind::Connector(c) => Some(c),
            ElementKind::Shape(_) => None,
        }
    }

    pub fn connector_mut(&mut self) -> Option<&mut Connector> {
        match &mut self.kind {
            ElementKind::Connector(c) => Some(c),
            ElementKind::Shape(_) => None,
        }
    }

    /// O estilo, seja de uma forma ou de uma seta.
    #[must_use]
    pub fn style(&self) -> &Style {
        match &self.kind {
            ElementKind::Shape(s) => &s.style,
            ElementKind::Connector(c) => &c.style,
        }
    }

    pub fn style_mut(&mut self) -> &mut Style {
        match &mut self.kind {
            ElementKind::Shape(s) => &mut s.style,
            ElementKind::Connector(c) => &mut c.style,
        }
    }

    /// Desloca o elemento por `d` (mundo). Numa seta, só as pontas SOLTAS — as presas seguem a forma.
    pub fn translate(&mut self, d: [f64; 2]) {
        match &mut self.kind {
            ElementKind::Shape(_) => {
                self.x += d[0];
                self.y += d[1];
            }
            ElementKind::Connector(c) => {
                for e in [&mut c.start, &mut c.end] {
                    if let End::Free(p) = e {
                        *p = [p[0] + d[0], p[1] + d[1]];
                    }
                }
            }
        }
    }
}

/// Roda `p` à volta de `c` por `angle` radianos.
#[must_use]
pub fn rotate_about(p: [f64; 2], c: [f64; 2], angle: f64) -> [f64; 2] {
    if angle == 0.0 {
        return p;
    }
    let (s, k) = angle.sin_cos();
    let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
    [c[0] + dx * k - dy * s, c[1] + dx * s + dy * k]
}

/// O conteúdo de um quadro. `BTreeMap` por lei (determinismo).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BoardDoc {
    pub(crate) elements: BTreeMap<ElementId, Element>,
    pub(crate) next_id: u64,
    /// A maior chave de z que já entrou (lápides incluídas) — o `z_on_top` em O(1). Só sobe: uma
    /// chave acima de um elemento apagado continua acima de todos os vivos.
    pub(crate) top_z: Option<FracKey>,
    /// A revisão da SESSÃO (não vai para o ficheiro): muda a cada operação aplicada.
    #[serde(skip)]
    pub(crate) rev: Rev,
}

/// ⭐ **Revisão de um documento nesta sessão** — o «mudou alguma coisa?» em O(1) dos caches
/// derivados (as rotas das setas). Tirada de um contador GLOBAL do processo: dois documentos
/// distintos nunca partilham uma (um cache que passa de um quadro a outro não confunde os dois),
/// e um documento lido de bytes nasce com uma nova. Não é conteúdo: duas revisões são sempre
/// «iguais» para o `PartialEq` do documento.
#[derive(Clone, Copy, Debug)]
pub struct Rev(u64);

static NEXT_REV: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Rev {
    pub(crate) fn fresh() -> Self {
        Self(NEXT_REV.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl Default for Rev {
    fn default() -> Self {
        Self::fresh()
    }
}

impl PartialEq for Rev {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl BoardDoc {
    /// A revisão desta sessão: muda a cada operação aplicada (e só então).
    #[must_use]
    pub fn rev(&self) -> u64 {
        self.rev.0
    }

    /// Um id ainda não usado neste quadro.
    pub fn mint_id(&mut self) -> ElementId {
        self.next_id += 1;
        ElementId(self.next_id)
    }

    /// O elemento vivo (não apagado) com este id.
    #[must_use]
    pub fn get(&self, id: ElementId) -> Option<&Element> {
        self.elements.get(&id).filter(|e| !e.deleted)
    }

    /// Os elementos vivos, por id (sem ordenar por z).
    pub fn live(&self) -> impl Iterator<Item = &Element> {
        self.elements.values().filter(|e| !e.deleted)
    }

    /// Os elementos vivos, de trás para a frente (ordem de desenho).
    #[must_use]
    pub fn live_in_z_order(&self) -> Vec<&Element> {
        let mut v: Vec<&Element> = self.elements.values().filter(|e| !e.deleted).collect();
        v.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.cmp(&b.id)));
        v
    }

    /// Quantos elementos vivos.
    #[must_use]
    pub fn live_len(&self) -> usize {
        self.elements.values().filter(|e| !e.deleted).count()
    }

    /// A chave de z para pôr um elemento novo à frente de todos.
    #[must_use]
    pub fn z_on_top(&self) -> FracKey {
        FracKey::between(self.top_z.as_ref(), None)
    }
}
