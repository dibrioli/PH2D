//! **Os cursores de edição do PH2D**, no idioma dos editores vectoriais (Illustrator/Figma): a seta directa, a
//! seta com o selo do ponto, a pena com «+» e «−», a seta de fundir e a de alternar. Cada figura é desenhada em
//! RGBA a partir dos tokens (corpo `Text1`, contorno `Bg0`, selo `Accent`/`Danger`), por campo de distância com
//! anti-serrilhado analítico, ao factor de escala da janela (o winit trata o buffer como píxeis físicos).
//!
//! Quem edita escolhe a [`Figura`]; o [`Jogo`] guarda os `CustomCursor` já criados e devolve o `Cursor` do winit.
//! Arrastar usa a mão fechada do SISTEMA ([`Figura::Agarrando`]), a única que não se desenha aqui.

use ph2d_tokens::{ColorToken, Theme};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CursorIcon, CustomCursor};

/// O lado da grelha de desenho, em píxeis lógicos (o tamanho de cursor do sistema).
pub const LADO: f32 = 24.0; // LITERAL-PX-OK: grelha do cursor, px lógicos
/// O bico de toda figura (seta e pena): o ponto quente.
pub const BICO: [f32; 2] = [2.0, 2.0]; // LITERAL-PX-OK: hotspot na grelha
const HALO: f32 = 1.5; // LITERAL-PX-OK: contorno escuro à volta de cada traço
const SELO: [f32; 2] = [17.0, 18.0]; // LITERAL-PX-OK: centro do selo (canto de baixo à direita)

/// O que o cursor promete.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Figura {
    /// Seleção directa: o canvas em modo de pontos, nada sob o ponteiro.
    Seta,
    /// Sobre um ponto: a seta com o quadradinho do ponto (arrastar move-o).
    SetaPonto,
    /// A arrastar um ponto: a mão fechada do sistema.
    Agarrando,
    /// Sobre uma aresta: a pena com «+» (clicar acrescenta).
    PenaMais,
    /// `Alt` sobre um ponto: a pena com «−» (clicar tira).
    PenaMenos,
    /// A arrastar sobre outro ponto: soltar funde (o selo de dois anéis, em perigo).
    Funde,
    /// `Shift` sobre um ponto: a seta com «±» (alterna na seleção).
    Alterna,
}

impl Figura {
    pub const TODAS: [Figura; 7] = [
        Figura::Seta,
        Figura::SetaPonto,
        Figura::Agarrando,
        Figura::PenaMais,
        Figura::PenaMenos,
        Figura::Funde,
        Figura::Alterna,
    ];

    /// O ícone do sistema que mais se aproxima (a caneta Wayland e as plataformas sem cursor próprio).
    #[must_use]
    pub fn do_sistema(self) -> CursorIcon {
        match self {
            Figura::Seta | Figura::SetaPonto | Figura::Alterna => CursorIcon::Default,
            Figura::Agarrando => CursorIcon::Grabbing,
            Figura::PenaMais | Figura::PenaMenos | Figura::Funde => CursorIcon::Crosshair,
        }
    }
}

/// As cores de uma figura, já resolvidas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cores {
    pub corpo: [u8; 4],
    pub contorno: [u8; 4],
    pub realce: [u8; 4],
    pub perigo: [u8; 4],
}

impl Cores {
    #[must_use]
    pub fn do_tema(theme: Theme) -> Self {
        let c = |t: ColorToken| {
            let k = t.resolve(theme);
            [k.r, k.g, k.b, 255]
        };
        Self {
            corpo: c(ColorToken::Text1),
            contorno: c(ColorToken::Bg0),
            realce: c(ColorToken::Accent),
            perigo: c(ColorToken::Danger),
        }
    }
}

/// Uma figura em RGBA (alfa directo), com o ponto quente em píxeis físicos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imagem {
    pub lado: u16,
    pub rgba: Vec<u8>,
    pub quente: (u16, u16),
}

impl Imagem {
    /// O alfa do píxel `(x, y)`.
    #[must_use]
    pub fn alfa(&self, x: u16, y: u16) -> u8 {
        self.rgba[(usize::from(y) * usize::from(self.lado) + usize::from(x)) * 4 + 3]
    }
}

#[derive(Clone, Debug)]
enum Forma {
    Poligono(Vec<[f32; 2]>),
    Disco([f32; 2], f32),
    Anel([f32; 2], f32, f32),
    Traco([f32; 2], [f32; 2], f32),
    Caixa([f32; 2], f32),
}

fn seg(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = (dx * dx + dy * dy).max(f32::EPSILON);
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0);
    (a[0] + t * dx - p[0]).hypot(a[1] + t * dy - p[1])
}

impl Forma {
    /// A distância com sinal (negativa dentro), em unidades da grelha.
    fn d(&self, p: [f32; 2]) -> f32 {
        match self {
            Forma::Poligono(v) => {
                let mut d = f32::MAX;
                let mut dentro = false;
                for i in 0..v.len() {
                    let (a, b) = (v[i], v[(i + 1) % v.len()]);
                    d = d.min(seg(p, a, b));
                    if (a[1] > p[1]) != (b[1] > p[1])
                        && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0])
                    {
                        dentro = !dentro;
                    }
                }
                if dentro { -d } else { d }
            }
            Forma::Disco(c, r) => (p[0] - c[0]).hypot(p[1] - c[1]) - r,
            Forma::Anel(c, r, w) => ((p[0] - c[0]).hypot(p[1] - c[1]) - r).abs() - w / 2.0,
            Forma::Traco(a, b, w) => seg(p, *a, *b) - w / 2.0,
            Forma::Caixa(c, h) => (p[0] - c[0]).abs().max((p[1] - c[1]).abs()) - h,
        }
    }
}

/// A seta (bico em [`BICO`]).
fn seta() -> Forma {
    let [x, y] = BICO;
    // LITERAL-PX-OK: o desenho da seta na grelha de 24
    let v = [
        [0.0, 0.0],
        [0.0, 15.5],
        [3.9, 12.0],
        [6.4, 17.6],
        [8.9, 16.5],
        [6.5, 11.0],
        [11.2, 11.0],
    ];
    Forma::Poligono(v.iter().map(|p| [x + p[0], y + p[1]]).collect())
}

/// A pena: o aparo com o bico em [`BICO`], deitado a 45°; o golpe e o furo no contorno.
fn pena() -> (Vec<Forma>, Vec<Forma>) {
    let r = |p: [f32; 2]| {
        let k = std::f32::consts::FRAC_1_SQRT_2;
        // +y local → (1, 1)/√2 no ecrã: o aparo desce para a direita a partir do bico.
        [BICO[0] + k * (p[0] + p[1]), BICO[1] + k * (p[1] - p[0])]
    };
    // LITERAL-PX-OK: o desenho do aparo na grelha de 24
    let corpo = [
        [0.0, 0.0],
        [4.6, 7.6],
        [3.6, 12.2],
        [-3.6, 12.2],
        [-4.6, 7.6],
    ];
    let corpo = Forma::Poligono(corpo.iter().map(|p| r(*p)).collect());
    let golpe = Forma::Traco(r([0.0, 1.5]), r([0.0, 6.4]), 1.1);
    let furo = Forma::Disco(r([0.0, 7.4]), 1.35);
    let cabo = Forma::Traco(r([-3.4, 14.0]), r([3.4, 14.0]), 2.0);
    (vec![corpo, cabo], vec![golpe, furo])
}

/// Uma camada: as formas e a cor do seu corpo; todas levam o halo do contorno por baixo.
struct Camada {
    formas: Vec<Forma>,
    cor: [u8; 4],
}

fn camadas(f: Figura, c: &Cores) -> Vec<Camada> {
    let [sx, sy] = SELO;
    let corpo = |formas| Camada {
        formas,
        cor: c.corpo,
    };
    let mais = |cor| Camada {
        formas: vec![
            Forma::Traco([sx - 3.2, sy], [sx + 3.2, sy], 2.0),
            Forma::Traco([sx, sy - 3.2], [sx, sy + 3.2], 2.0),
        ],
        cor,
    };
    let menos = |cor| Camada {
        formas: vec![Forma::Traco([sx - 3.2, sy], [sx + 3.2, sy], 2.0)],
        cor,
    };
    let pena_de = |selo: Camada| {
        let (corpo_da_pena, marcas) = pena();
        vec![
            corpo(corpo_da_pena),
            Camada {
                formas: marcas,
                cor: c.contorno,
            },
            selo,
        ]
    };
    match f {
        Figura::Seta | Figura::Agarrando => vec![corpo(vec![seta()])],
        Figura::SetaPonto => vec![
            corpo(vec![seta()]),
            Camada {
                formas: vec![Forma::Caixa([sx, sy], 2.9)],
                cor: c.corpo,
            },
            Camada {
                formas: vec![Forma::Caixa([sx, sy], 1.4)],
                cor: c.contorno,
            },
        ],
        Figura::PenaMais => pena_de(mais(c.realce)),
        Figura::PenaMenos => pena_de(menos(c.perigo)),
        Figura::Funde => vec![
            corpo(vec![seta()]),
            Camada {
                formas: vec![
                    Forma::Anel([sx - 1.6, sy], 2.6, 1.6),
                    Forma::Anel([sx + 1.6, sy], 2.6, 1.6),
                ],
                cor: c.perigo,
            },
        ],
        Figura::Alterna => vec![
            corpo(vec![seta()]),
            Camada {
                formas: vec![
                    Forma::Traco([sx - 3.0, sy - 1.6], [sx + 3.0, sy - 1.6], 1.6),
                    Forma::Traco([sx, sy - 4.6], [sx, sy + 1.4], 1.6),
                    Forma::Traco([sx - 3.0, sy + 3.4], [sx + 3.0, sy + 3.4], 1.6),
                ],
                cor: c.realce,
            },
        ],
    }
}

fn sobre(dst: &mut [f32; 4], cor: [u8; 4], a: f32) {
    let a = a * f32::from(cor[3]) / 255.0;
    let out = a + dst[3] * (1.0 - a);
    if out <= 0.0 {
        return;
    }
    for k in 0..3 {
        let s = f32::from(cor[k]) / 255.0;
        dst[k] = (s * a + dst[k] * dst[3] * (1.0 - a)) / out;
    }
    dst[3] = out;
}

/// Desenha a figura ao factor de escala `escala` (≥ 1 lógico = 1 físico).
#[must_use]
pub fn desenha(f: Figura, escala: f64, c: &Cores) -> Imagem {
    let s = escala.clamp(1.0, 4.0) as f32;
    let lado = (LADO * s).ceil() as u16;
    let cam = camadas(f, c);
    let mut rgba = Vec::with_capacity(usize::from(lado) * usize::from(lado) * 4);
    for y in 0..lado {
        for x in 0..lado {
            let p = [(f32::from(x) + 0.5) / s, (f32::from(y) + 0.5) / s];
            let mut px = [0.0_f32; 4];
            // O halo de TODAS as camadas primeiro: o contorno é um só, à volta do conjunto.
            let halo = cam
                .iter()
                .flat_map(|k| &k.formas)
                .map(|fm| fm.d(p) - HALO)
                .fold(f32::MAX, f32::min);
            sobre(&mut px, c.contorno, (0.5 - halo * s).clamp(0.0, 1.0));
            for k in &cam {
                let d = k.formas.iter().map(|fm| fm.d(p)).fold(f32::MAX, f32::min);
                sobre(&mut px, k.cor, (0.5 - d * s).clamp(0.0, 1.0));
            }
            rgba.extend(px.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
        }
    }
    let q = |v: f32| (v * s).round() as u16;
    Imagem {
        lado,
        rgba,
        quente: (q(BICO[0]), q(BICO[1])),
    }
}

/// Os cursores já entregues ao sistema, por figura; refeitos quando a escala ou o tema mudam.
#[derive(Default)]
pub struct Jogo {
    feitos: Vec<(Figura, CustomCursor)>,
    chave: Option<(u64, Theme)>,
}

impl Jogo {
    /// Cria (ou refaz) as figuras para esta escala e tema. Barato quando nada mudou.
    pub fn prepara(&mut self, el: &ActiveEventLoop, escala: f64, theme: Theme) {
        let chave = Some((escala.to_bits(), theme));
        if self.chave == chave {
            return;
        }
        let cores = Cores::do_tema(theme);
        self.feitos = Figura::TODAS
            .into_iter()
            .filter(|f| *f != Figura::Agarrando)
            .filter_map(|f| {
                let im = desenha(f, escala, &cores);
                let src =
                    CustomCursor::from_rgba(im.rgba, im.lado, im.lado, im.quente.0, im.quente.1);
                src.ok().map(|s| (f, el.create_custom_cursor(s)))
            })
            .collect();
        self.chave = chave;
    }

    /// O cursor do winit para a figura: o desenhado, ou o do sistema se ainda não há.
    #[must_use]
    pub fn cursor(&self, f: Figura) -> Cursor {
        self.feitos
            .iter()
            .find(|(g, _)| *g == f)
            .map_or_else(|| f.do_sistema().into(), |(_, c)| c.clone().into())
    }
}

#[cfg(test)]
mod tests;
