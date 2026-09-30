//! ⭐⭐⭐ **O EIXO DO TRAÇO** (doc 121 W4) — a linha que o traço segue, aplanada, com as juntas e as
//! pontas que o estilo pede, para o shader construir o traço de uma cópia **NÃO conforme** no ecrã.
//!
//! ## Por que existe
//!
//! Sob um afim conforme o traço expandido no espaço LOCAL (o `expand_stroke` do kurbo, em
//! [`crate::geometry`]) é exactamente o que o Vello desenha. Sob escala NÃO uniforme ele não é: a lei
//! da casa (bug #27, `ph2d_vec_render::stroke_uniform`) traça a geometria **já transformada** com uma
//! caneta REDONDA de largura `w·√|det|` — *«quando engrossa, engrossa por igual nos dois eixos»*. O
//! traço dessa caneta depende da cópia, logo não se pode preparar uma vez: prepara-se o EIXO, e o
//! shader faz, por cópia e no ecrã, a mesma conta que o kurbo faria sobre a geometria transformada —
//! um quadrilátero por troço, a junta que o estilo pede em cada QUINA, a ponta que ele pede em cada
//! extremo aberto, tudo somado pela regra não-nula (a mesma com que o Vello preenche o contorno que o
//! kurbo lhe entrega).
//!
//! ⚠️ **Quina é só onde o caminho a tem.** Entre as cordas de uma MESMA curva aplanada a junta é
//! REDONDA — a união de discos ao longo da curva é o contorno verdadeiro dela, e é o que o `kurbo`
//! entrega ao deslocar uma curva lisa. O estilo (esquadria · chanfro · redonda) vale só entre dois
//! ELEMENTOS do caminho cujas tangentes discordam.
//!
//! ⛔ **O tracejado NÃO entra:** sob escala não uniforme o comprimento de arco no MUNDO não é
//! proporcional ao local (depende da direcção), e a lei da casa mede o tracejado no mundo. Uma
//! geometria com traço tracejado é marcada ([`crate::geometry::FLAG_SO_CONFORME`]) e quem chama manda
//! as cópias não conformes dela ao Vello.

use bytemuck::{Pod, Zeroable};
use ph2d_vector::{BezPath, Cap, Join, PathEl, Point, Stroke, flatten};

/// Um troço do eixo: de `a` a `b`.
pub const ITEM_TROCO: u32 = 0;
/// Uma junta em `b`, entre o troço `a → b` e o troço `b → c`.
pub const ITEM_JUNTA: u32 = 1;
/// Uma ponta em `b`, no fim do troço `a → b` (a direcção `a → b` aponta para FORA).
pub const ITEM_PONTA: u32 = 2;
/// ⭐ O CABEÇALHO de um bloco de peças consecutivas: `a`/`b` são a caixa LOCAL dos pontos delas,
/// `meia_largura` o maior alcance para fora do eixo (`meia × fator`: a esquadria numa quina em esquadria,
/// `1,5` numa ponta quadrada, `1` no resto — ainda sem a caneta),
/// e `_pad` quantas peças se seguem. O shader salta o bloco inteiro quando a caixa, alargada pela
/// caneta da cópia, não toca no pixel — cada peça é fechada, logo uma que não toca soma zero.
pub const ITEM_BLOCO: u32 = 3;

/// Quantas peças cabem num bloco. ⚠️ MEDIDO (sonda `sonda_relogio_das_estrelas_grandes`, `72`
/// estrelas arredondadas de `115 × 38 px`, RTX, `--release`): sem blocos cada pixel lia as `240`
/// peças da estrela e o traço custava `1,34 ms` contra `0,37` do Vello. Com os blocos e as caixas
/// no ECRÃ: `4` → `0,48` · **`8` → `0,40`** · `16` → `0,43` · `32` → `0,53–0,60 ms`. Abaixo de `8`
/// os cabeçalhos pesam mais do que as peças que poupam; acima, a caixa do bloco engorda.
pub const PECAS_POR_BLOCO: usize = 8;

/// A junta, como o shader a lê: `0` esquadria · `1` chanfro · `2` redonda.
pub const JUNTA_ESQUADRIA: u32 = 0;
pub const JUNTA_CHANFRO: u32 = 1;
pub const JUNTA_REDONDA: u32 = 2;
/// A ponta: `0` rente · `1` quadrada · `2` redonda.
pub const PONTA_RENTE: u32 = 0;
pub const PONTA_QUADRADA: u32 = 1;
pub const PONTA_REDONDA: u32 = 2;

/// Um item do eixo, como a placa o lê (`shape.wgsl`, `Eixo`): `48` bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct EixoItem {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub c: [f32; 2],
    /// Metade da largura AUTORADA (unidades locais) — o shader multiplica-a por `√|det|`.
    pub meia_largura: f32,
    pub limite_esquadria: f32,
    pub tipo: u32,
    pub junta: u32,
    pub ponta: u32,
    pub _pad: u32,
}

fn junta_de(j: Join) -> u32 {
    match j {
        Join::Miter => JUNTA_ESQUADRIA,
        Join::Bevel => JUNTA_CHANFRO,
        Join::Round => JUNTA_REDONDA,
    }
}

fn ponta_de(c: Cap) -> u32 {
    match c {
        Cap::Butt => PONTA_RENTE,
        Cap::Square => PONTA_QUADRADA,
        Cap::Round => PONTA_REDONDA,
    }
}

/// A tangente à SAÍDA de `de` num elemento, e a da CHEGADA — dos pontos de controlo (a primeira
/// que não é nula), que é o que decide se a fronteira entre dois elementos é uma quina.
fn tangentes(de: Point, el: PathEl) -> Option<(Point, Point)> {
    let dif = |a: Point, b: Point| Point::new(b.x - a.x, b.y - a.y);
    let nula = |p: Point| p.x == 0.0 && p.y == 0.0;
    let primeira = |v: &[Point]| v.iter().copied().find(|p| !nula(*p));
    match el {
        PathEl::LineTo(p) => {
            let t = dif(de, p);
            (!nula(t)).then_some((t, t))
        }
        PathEl::QuadTo(c, p) => Some((
            primeira(&[dif(de, c), dif(de, p)])?,
            primeira(&[dif(c, p), dif(de, p)])?,
        )),
        PathEl::CurveTo(c1, c2, p) => Some((
            primeira(&[dif(de, c1), dif(de, c2), dif(de, p)])?,
            primeira(&[dif(c2, p), dif(c1, p), dif(de, p)])?,
        )),
        PathEl::MoveTo(_) | PathEl::ClosePath => None,
    }
}

/// Duas tangentes discordam? — o limiar é o de uma quina que se veria: abaixo de `~1e-3` rad a junta
/// do estilo e a redonda coincidem muito abaixo da tolerância.
fn quina(t0: Point, t1: Point) -> bool {
    let cruz = t0.x * t1.y - t0.y * t1.x;
    let ponto = t0.x * t1.x + t0.y * t1.y;
    let n = (t0.x * t0.x + t0.y * t0.y) * (t1.x * t1.x + t1.y * t1.y);
    ponto < 0.0 || cruz * cruz > 1.0e-6 * n
}

/// Um sub-caminho aplanado: os pontos, e em cada ponto se ele é uma QUINA (a fronteira entre dois
/// elementos cujas tangentes discordam).
struct Sub {
    pontos: Vec<Point>,
    quinas: Vec<bool>,
    fechado: bool,
    /// A tangente à saída do 1.º elemento e à chegada do último — para a quina do fecho.
    t_inicio: Option<Point>,
    t_fim: Option<Point>,
}

fn sub_caminhos(path: &BezPath, tol: f64) -> Vec<Sub> {
    let mut subs: Vec<Sub> = Vec::new();
    let mut atual: Option<Sub> = None;
    let mut cur = Point::ZERO;
    let mut inicio = Point::ZERO;
    let mut t_ant: Option<Point> = None;
    let fecha = |s: Option<Sub>, subs: &mut Vec<Sub>| {
        if let Some(s) = s
            && s.pontos.len() >= 2
        {
            subs.push(s);
        }
    };
    let mut elementos: Vec<PathEl> = path.elements().to_vec();
    // O fecho é um troço como os outros — a `LineTo` implícita de volta ao início.
    let mut i = 0;
    while i < elementos.len() {
        if let PathEl::ClosePath = elementos[i] {
            elementos.insert(i, PathEl::LineTo(Point::new(f64::NAN, f64::NAN)));
            i += 1;
        }
        i += 1;
    }
    for el in elementos {
        match el {
            PathEl::MoveTo(p) => {
                fecha(atual.take(), &mut subs);
                atual = Some(Sub {
                    pontos: vec![p],
                    quinas: vec![false],
                    fechado: false,
                    t_inicio: None,
                    t_fim: None,
                });
                cur = p;
                inicio = p;
                t_ant = None;
            }
            PathEl::ClosePath => {
                if let Some(s) = atual.as_mut() {
                    s.fechado = true;
                }
                fecha(atual.take(), &mut subs);
                cur = inicio;
                t_ant = None;
            }
            el => {
                // O marcador do fecho vira o caminho de volta ao início.
                let el = match el {
                    PathEl::LineTo(p) if p.x.is_nan() => PathEl::LineTo(inicio),
                    e => e,
                };
                let Some(s) = atual.as_mut() else { continue };
                let Some((t0, t1)) = tangentes(cur, el) else {
                    continue;
                };
                if let Some(ta) = t_ant {
                    if let Some(q) = s.quinas.last_mut() {
                        *q = quina(ta, t0);
                    }
                } else {
                    s.t_inicio = Some(t0);
                }
                t_ant = Some(t1);
                s.t_fim = Some(t1);
                let mut bp = BezPath::new();
                bp.move_to(cur);
                bp.push(el);
                flatten(bp.iter(), tol, |e| {
                    if let PathEl::LineTo(p) = e
                        && s.pontos.last().is_none_or(|u| *u != p)
                    {
                        s.pontos.push(p);
                        s.quinas.push(false);
                    }
                });
                cur = match el {
                    PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => p,
                    _ => cur,
                };
            }
        }
    }
    fecha(atual, &mut subs);
    subs
}

/// As peças de um nível, arrumadas em blocos de [`PECAS_POR_BLOCO`], cada um com o cabeçalho
/// [`ITEM_BLOCO`] à frente. A ordem das peças fica — a regra não-nula não depende dela.
#[must_use]
pub fn em_blocos(pecas: &[EixoItem]) -> Vec<EixoItem> {
    let mut out = Vec::with_capacity(pecas.len() + pecas.len() / PECAS_POR_BLOCO + 1);
    for bloco in pecas.chunks(PECAS_POR_BLOCO) {
        let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
        let mut alcance: f32 = 0.0;
        for it in bloco {
            let pontos: &[[f32; 2]] = if it.tipo == ITEM_JUNTA {
                &[it.a, it.b, it.c]
            } else {
                &[it.a, it.b]
            };
            for q in pontos {
                for k in 0..2 {
                    lo[k] = lo[k].min(q[k]);
                    hi[k] = hi[k].max(q[k]);
                }
            }
            // O mesmo alcance que o shader dá à peça (`alcance_da_peca`).
            let fator = match it.tipo {
                ITEM_JUNTA if it.junta == JUNTA_ESQUADRIA => it.limite_esquadria.max(1.0),
                ITEM_PONTA if it.ponta == PONTA_QUADRADA => 1.5,
                _ => 1.0,
            };
            alcance = alcance.max(it.meia_largura * fator);
        }
        out.push(EixoItem {
            a: lo,
            b: hi,
            meia_largura: alcance,
            tipo: ITEM_BLOCO,
            #[expect(clippy::cast_possible_truncation, reason = "um bloco tem 16 peças")]
            _pad: bloco.len() as u32,
            ..EixoItem::default()
        });
        out.extend_from_slice(bloco);
    }
    out
}

fn f(p: Point) -> [f32; 2] {
    #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
    [p.x as f32, p.y as f32]
}

/// Acrescenta a `out` os itens do eixo de `path` sob o estilo `style`, aplanado a `tol`. Devolve a
/// extensão para FORA do eixo, em múltiplos da meia largura, que as juntas e pontas pedem (a
/// esquadria vai até ao limite; a ponta quadrada até `√2`).
pub fn eixo(path: &BezPath, style: &Stroke, tol: f64, out: &mut Vec<EixoItem>) -> f32 {
    #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
    let (meia, limite) = ((style.width * 0.5) as f32, style.miter_limit as f32);
    let base = EixoItem {
        meia_largura: meia,
        limite_esquadria: limite,
        junta: junta_de(style.join),
        ..EixoItem::default()
    };
    let mut ext: f32 = 1.0;
    for s in sub_caminhos(path, tol) {
        let mut p = s.pontos;
        let mut q = s.quinas;
        // Um fechado cujo último ponto É o primeiro: tira-se o repetido, e a junta do fecho fica
        // no ponto 0.
        if s.fechado && p.len() > 2 && p.first() == p.last() {
            p.pop();
            q.pop();
        }
        let n = p.len();
        if n < 2 {
            continue;
        }
        let troços = if s.fechado { n } else { n - 1 };
        for i in 0..troços {
            out.push(EixoItem {
                a: f(p[i]),
                b: f(p[(i + 1) % n]),
                tipo: ITEM_TROCO,
                ..base
            });
        }
        let junta_em = |i: usize, eh_quina: bool| EixoItem {
            a: f(p[(i + n - 1) % n]),
            b: f(p[i]),
            c: f(p[(i + 1) % n]),
            tipo: ITEM_JUNTA,
            junta: if eh_quina { base.junta } else { JUNTA_REDONDA },
            ..base
        };
        // As juntas INTERIORES: `1..n−1` num aberto; num fechado também a do último ponto (a do
        // ponto `0` é a do fecho, abaixo).
        let fim = if s.fechado { n } else { n - 1 };
        for (i, &eh_quina) in q.iter().enumerate().take(fim).skip(1) {
            out.push(junta_em(i, eh_quina));
            if eh_quina && base.junta == JUNTA_ESQUADRIA {
                ext = ext.max(limite);
            }
        }
        if s.fechado {
            let eh_quina = match (s.t_fim, s.t_inicio) {
                (Some(a), Some(b)) => quina(a, b),
                _ => true,
            };
            out.push(junta_em(0, eh_quina));
            if eh_quina && base.junta == JUNTA_ESQUADRIA {
                ext = ext.max(limite);
            }
        } else {
            for (de, em, cap) in [
                (p[1], p[0], style.start_cap),
                (p[n - 2], p[n - 1], style.end_cap),
            ] {
                if cap == Cap::Square {
                    ext = ext.max(std::f32::consts::SQRT_2);
                }
                out.push(EixoItem {
                    a: f(de),
                    b: f(em),
                    tipo: ITEM_PONTA,
                    ponta: ponta_de(cap),
                    ..base
                });
            }
        }
    }
    ext
}

#[cfg(test)]
#[path = "eixo_tests.rs"]
mod tests;
