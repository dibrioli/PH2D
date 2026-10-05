//! ⭐⭐ doc 121 §9.18 (C) — **O CONTORNO DO EIXO NA CPU, pela lei da placa.** O `contorno.wgsl` emite as
//! arestas do traço de uma cópia no ecrã (`percorre`: o quadrilátero de cada troço ou pedaço de traço com a
//! FAIXA na bissectriz, a junta de quem chega a um vértice que a faixa não cobre, as pontas, o tracejado
//! andado pelo arco no ecrã). Aqui a MESMA lei, função a função e em `f32`, escreve POLÍGONOS fechados com
//! o sentido de cada peça da placa: preenchidos `nonzero`, as arestas que a placa não escreve (a da faixa,
//! partilhada com o vizinho; os raios interiores de um leque) cancelam-se aos pares, e a soma é a dela.
//!
//! Quem a usa: as MARCAS de uma cópia conforme (`geometry.rs`, no espaço local) e a rota Vello do Motion,
//! que traçava o tracejado pelo traçador da casa — cuja junta interior passa pelo pivô e MORDE um pedaço
//! curto depois de uma quina. ⚠️ Uma mudança na lei do `contorno.wgsl` muda aqui também: os gates do
//! tracejado comparam as duas (`ph2d-app-motion`, `motion_shape_placa_gpu_tracejado_tests.rs`).
//!
//! ⚠️ O espaço de saída é o ECRÃ: as constantes da placa são em pixels (a flecha `0,25` de um leque, a folga
//! `0,1` da faixa).

use std::ops::{Add, Mul, Neg, Sub};

use ph2d_vector::BezPath;

use crate::LEVELS;
use crate::eixo::{EixoItem, ITEM_BLOCO, ITEM_PONTA, ITEM_TROCO, SUB_FECHADO, SUB_INICIO};

/// O afim de uma cópia no espaço de saída e a caneta `√|det|` — a `Copia` do shader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AfimDaCopia {
    pub lin: [f32; 4],
    pub t: [f32; 2],
    pub caneta: f32,
}

impl AfimDaCopia {
    /// O espaço LOCAL (as marcas de uma cópia conforme).
    pub const IDENTIDADE: Self = Self {
        lin: [1.0, 0.0, 0.0, 1.0],
        t: [0.0, 0.0],
        caneta: 1.0,
    };
}

/// O que o `copia_de` do shader decide sobre uma cópia de afim linear `lin`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NivelDaCopia {
    /// O nível de detalhe mais GROSSO cujo erro no ecrã cabe em `0,25 px`.
    pub nivel: usize,
    /// Rotação e escala uniforme: a cópia desenha as MARCAS (o contorno no espaço local).
    pub conforme: bool,
}

/// O `copia_de` do `shape.wgsl`: o nível e a conformidade de uma cópia, com as tolerâncias `tol` do registo.
#[must_use]
pub fn nivel_da_copia(tol: &[f32; LEVELS], lin: [f32; 4]) -> NivelDaCopia {
    let [x, y, z, w] = lin;
    let e = 0.5 * (x + w);
    let f = 0.5 * (x - w);
    let g = 0.5 * (y + z);
    let h = 0.5 * (y - z);
    let escala = (e * e + h * h).sqrt() + (f * f + g * g).sqrt();
    let nivel = tol
        .iter()
        .position(|t| t * escala <= 0.25)
        .unwrap_or(LEVELS - 1);
    let l1 = x * x + y * y;
    let l2 = z * z + w * w;
    let esc2 = l1.max(l2).max(1.0e-30);
    let conforme = (l1 - l2).abs() <= esc2 * 1.0e-5 && (x * z + y * w).abs() <= esc2 * 1.0e-5;
    NivelDaCopia { nivel, conforme }
}

/// A caneta `√|det|` de `lin` (o shader: `sqrt(abs(lin.x * lin.w - lin.z * lin.y))`).
#[must_use]
pub fn caneta_de(lin: [f32; 4]) -> f32 {
    (lin[0] * lin[3] - lin[2] * lin[1]).abs().sqrt()
}

/// ⭐ **O contorno do traço de `eixo` sob `m`**, como polígonos fechados em `saida` (a regra é a
/// `nonzero`). `so_tracejado`: só os sub-caminhos tracejados (os contínuos ficam com quem os traçava).
pub fn contorno_do_eixo(
    eixo: &[EixoItem],
    m: &AfimDaCopia,
    so_tracejado: bool,
    saida: &mut BezPath,
) {
    let mut s = Saida {
        eixo,
        m: *m,
        bp: saida,
    };
    let ajuste = s.ajuste_do_tracejado();
    for (i, it) in eixo.iter().enumerate() {
        if it.tipo == ITEM_BLOCO {
            continue;
        }
        if !it.tracejado() {
            if !so_tracejado {
                s.peca(it);
            }
        } else if it.ponta & SUB_INICIO != 0 {
            s.tracejado(i, ajuste);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct V {
    x: f32,
    y: f32,
}

const fn v(x: f32, y: f32) -> V {
    V { x, y }
}

impl Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        v(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V {
    type Output = V;
    fn sub(self, o: V) -> V {
        v(self.x - o.x, self.y - o.y)
    }
}

impl Neg for V {
    type Output = V;
    fn neg(self) -> V {
        v(-self.x, -self.y)
    }
}

impl Mul<f32> for V {
    type Output = V;
    fn mul(self, k: f32) -> V {
        v(self.x * k, self.y * k)
    }
}

fn dot(a: V, b: V) -> f32 {
    a.x * b.x + a.y * b.y
}

fn length(a: V) -> f32 {
    dot(a, a).sqrt()
}

fn perp(u: V) -> V {
    v(-u.y, u.x)
}

/// O `sign` do WGSL (`0` em zero; o `f32::signum` dá `1`).
fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn pt(p: [f32; 2]) -> V {
    v(p[0], p[1])
}

/// O sinal que o `orienta(v, a, b, c)` daria.
fn positivo(a: V, b: V, c: V) -> bool {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0
}

/// Quantos traços um troço corta, no máximo (`TRACOS_POR_TROCO_MAX` do shader).
const TRACOS_POR_TROCO_MAX: f32 = 65536.0;
const FOLGA_DO_AJUSTE: f32 = 1.0e-4;
/// A folga da faixa num vértice LISO, em pixels (`FAIXA_FOLGA` do shader).
const FAIXA_FOLGA: f32 = 0.1;
/// A flecha de um leque, em pixels (a tolerância do Vello).
const FLECHA: f32 = 0.25;

#[derive(Clone, Copy)]
struct SubTracejado {
    n: u32,
    fechado: bool,
    tr: f32,
    per: f32,
    tot: f32,
    a_fim: f32,
    emenda: bool,
}

#[derive(Clone, Copy)]
struct Troco {
    s0: f32,
    fim: f32,
    len: f32,
    corda: f32,
    lprev: f32,
    lnext: f32,
    tem_ant: bool,
    tem_seg: bool,
    primeiro: bool,
    ultimo: bool,
    n0: f32,
    n1: f32,
}

#[derive(Clone, Copy, Default)]
struct Pedaco {
    valido: bool,
    x0: f32,
    x1: f32,
    liga0: bool,
    liga1: bool,
    recuo0: f32,
    recuo1: f32,
}

struct Saida<'a> {
    eixo: &'a [EixoItem],
    m: AfimDaCopia,
    bp: &'a mut BezPath,
}

impl Saida<'_> {
    fn aplica(&self, p: [f32; 2]) -> V {
        let l = self.m.lin;
        v(l[0] * p[0] + l[2] * p[1], l[1] * p[0] + l[3] * p[1]) + pt(self.m.t)
    }

    fn poligono(&mut self, pts: &[V]) {
        let mut it = pts.iter();
        if let Some(p0) = it.next() {
            self.bp.move_to((f64::from(p0.x), f64::from(p0.y)));
            for p in it {
                self.bp.line_to((f64::from(p.x), f64::from(p.y)));
            }
            self.bp.close_path();
        }
    }

    fn tri(&mut self, a: V, b: V, c: V) {
        if positivo(a, b, c) {
            self.poligono(&[a, b, c]);
        } else {
            self.poligono(&[a, c, b]);
        }
    }

    /// Um quadrilátero com o sentido do `positivo(a, b, c)` — o da placa.
    fn quad(&mut self, a: V, b: V, c: V, d: V) {
        if positivo(a, b, c) {
            self.poligono(&[a, b, c, d]);
        } else {
            self.poligono(&[a, d, c, b]);
        }
    }

    /// O `emite_leque`: cada triângulo `(centro, p, w)` com o seu sentido.
    fn leque(&mut self, centro: V, n0: V, n_fim: V, cos_alpha: f32, dir: f32, r: f32) {
        if r <= FLECHA {
            self.tri(centro, centro + n0, centro + n_fim);
            return;
        }
        let q = 1.0 - FLECHA / r;
        if cos_alpha >= 2.0 * q * q - 1.0 {
            self.tri(centro, centro + n0, centro + n_fim);
            return;
        }
        let alpha = cos_alpha.clamp(-1.0, 1.0).acos();
        let passo_max = 2.0 * q.acos();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "entre 1 e 64"
        )]
        let k = (alpha / passo_max.max(1.0e-4)).ceil().clamp(1.0, 64.0) as u32;
        #[expect(clippy::cast_precision_loss, reason = "k <= 64")]
        let ang = dir * alpha / k as f32;
        let (c, sn) = (ang.cos(), ang.sin());
        let mut w0 = n0;
        let mut p = centro + n0;
        for j in 0..k {
            let mut w = centro + n_fim;
            if j + 1 < k {
                w0 = v(c * w0.x - sn * w0.y, sn * w0.x + c * w0.y);
                w = centro + w0;
            }
            self.tri(centro, p, w);
            p = w;
        }
    }

    /// O `emite_junta`: a junta em `b` entre `b − u` e `b → c`, do lado de fora.
    fn junta(&mut self, u: V, b: V, c: V, r: f32, junta: u32, limite: f32) {
        let dbc = c - b;
        let lbc = length(dbc);
        if lbc <= 0.0 {
            return;
        }
        let w = v(dbc.x / lbc, dbc.y / lbc);
        let cr = u.x * w.y - u.y * w.x;
        let dt = dot(u, w).clamp(-1.0, 1.0);
        if cr.abs() < 1.0e-7 && dt > 0.0 {
            return;
        }
        let lado = if cr > 0.0 { -1.0 } else { 1.0 };
        let n0 = perp(u) * r * lado;
        let n1 = perp(w) * r * lado;
        if junta == 0 && 2.0 <= (1.0 + dt) * limite * limite {
            let s = n0 + n1;
            let m = b + v(s.x / (1.0 + dt), s.y / (1.0 + dt));
            self.quad(b, b + n0, m, b + n1);
            return;
        }
        if junta == 2 {
            let mut dir = sign(n0.x * n1.y - n0.y * n1.x);
            if dir == 0.0 {
                dir = sign(n0.x * u.y - n0.y * u.x);
            }
            self.leque(b, n0, n1, dt, dir, r);
            return;
        }
        self.tri(b, b + n0, b + n1);
    }

    /// O `bissectriz_ate`: o deslocamento da faixa em `p1`, ou `None` quando ela não serve.
    #[expect(
        clippy::too_many_arguments,
        reason = "a assinatura do `bissectriz_ate` do shader"
    )]
    fn bissectriz_ate(
        &self,
        p0: V,
        p1: V,
        p2: V,
        r: f32,
        quina: bool,
        junta: u32,
        limite: f32,
        recuo_max: f32,
    ) -> Option<V> {
        let d0 = p1 - p0;
        let d1 = p2 - p1;
        let l0 = length(d0);
        let l1 = length(d1);
        if l0 <= 0.0 || l1 <= 0.0 {
            return None;
        }
        let u0 = v(d0.x / l0, d0.y / l0);
        let u1 = v(d1.x / l1, d1.y / l1);
        let dt = dot(u0, u1);
        if quina {
            if junta != 0 || 2.0 > (1.0 + dt) * limite * limite {
                return None;
            }
        } else if dt <= 0.0 {
            return None;
        }
        let m = (perp(u0) + perp(u1)) * (r / (1.0 + dt));
        let fora = r + FAIXA_FOLGA;
        let recuo = r * ((1.0 - dt).max(0.0) / (1.0 + dt)).sqrt();
        if (!quina && dot(m, m) > fora * fora) || recuo > recuo_max {
            return None;
        }
        Some(m)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "a assinatura do `bissectriz` do shader"
    )]
    fn bissectriz(
        &self,
        p0: V,
        p1: V,
        p2: V,
        r: f32,
        quina: bool,
        junta: u32,
        limite: f32,
    ) -> Option<V> {
        let recuo = 0.5 * length(p1 - p0).min(length(p2 - p1));
        self.bissectriz_ate(p0, p1, p2, r, quina, junta, limite, recuo)
    }

    /// O quadrilátero de um troço ou pedaço, com a faixa em cada ponta que a tem (o vizinho escreve a
    /// aresta partilhada no outro sentido: no `nonzero` as duas cancelam-se, como na placa).
    fn quadrilatero(&mut self, q0: V, q1: V, m0: V, m1: V) {
        self.quad(q0 + m0, q1 + m1, q1 - m1, q0 - m0);
    }

    /// O `emite_peca`: um troço contínuo ou uma ponta.
    fn peca(&mut self, it: &EixoItem) {
        let r = it.meia_largura * self.m.caneta;
        let a = self.aplica(it.a);
        let b = self.aplica(it.b);
        let dab = b - a;
        let lab = length(dab);
        if lab <= 0.0 || r <= 0.0 {
            return;
        }
        let u = v(dab.x / lab, dab.y / lab);
        let nr = perp(u) * r;
        if it.tipo == ITEM_TROCO {
            let mut m0 = nr;
            let mut m1 = nr;
            if it.ponta & 1 != 0
                && let Some(e) = self.bissectriz(
                    self.aplica(it.d),
                    a,
                    b,
                    r,
                    it.ponta & 4 != 0,
                    it.junta,
                    it.limite_esquadria,
                )
            {
                m0 = e;
            }
            if it.ponta & 2 != 0 {
                let quina = it.ponta & 8 != 0;
                let cf = self.aplica(it.c);
                match self.bissectriz(a, b, cf, r, quina, it.junta, it.limite_esquadria) {
                    Some(e) => m1 = e,
                    None => self.junta(
                        u,
                        b,
                        cf,
                        r,
                        if quina { it.junta } else { 2 },
                        it.limite_esquadria,
                    ),
                }
            }
            self.quadrilatero(a, b, m0, m1);
            return;
        }
        if it.tipo == ITEM_PONTA {
            self.tampa(b, u, r, it.ponta);
        }
    }

    /// O `emite_tampa`: a ponta de estilo `tampa` em `c`, virada para `u`.
    fn tampa(&mut self, c: V, u: V, r: f32, tampa: u32) {
        let nr = perp(u) * r;
        if tampa == 1 {
            self.quad(c + nr, c + nr + u * r, c - nr + u * r, c - nr);
        } else if tampa == 2 {
            let dir = sign(nr.x * u.y - nr.y * u.x);
            self.leque(c, nr, -nr, -1.0, dir, r);
        }
    }

    fn comprimento(&self, a: [f32; 2], b: [f32; 2]) -> f32 {
        length(self.aplica(b) - self.aplica(a))
    }

    /// O `arco` do troço no ecrã: a corda mais `8h²/3c`.
    fn arco(&self, it: &EixoItem) -> f32 {
        let d = self.aplica(it.b) - self.aplica(it.a);
        let c = length(d);
        if c <= 0.0 {
            return 0.0;
        }
        let l = self.m.lin;
        let fl = it.flecha;
        let f = v(l[0] * fl[0] + l[2] * fl[1], l[1] * fl[0] + l[3] * fl[1]);
        let h = (d.x * f.y - d.y * f.x) / c;
        c + 8.0 * h * h / (3.0 * c)
    }

    fn proximo_troco(&self, i: usize) -> usize {
        let mut j = i;
        while j < self.eixo.len() && self.eixo[j].tipo != ITEM_TROCO {
            j += 1;
        }
        j
    }

    /// O `ajuste_do_tracejado`: o factor do período que fecha o sub-caminho tracejado mais LONGO.
    fn ajuste_do_tracejado(&self) -> f32 {
        let caneta = self.m.caneta;
        let (mut melhor, mut fechado, mut tr, mut per) = (0.0_f32, false, 0.0_f32, 0.0_f32);
        let (mut tot, mut restantes) = (0.0_f32, 0_u32);
        let (mut sub_fechado, mut sub_tr, mut sub_per) = (false, 0.0_f32, 0.0_f32);
        for it in self
            .eixo
            .iter()
            .filter(|it| it.tipo == ITEM_TROCO && it.tracejado())
        {
            if restantes == 0 {
                if it.ponta & SUB_INICIO == 0 || it._pad == 0 {
                    continue;
                }
                restantes = it._pad;
                tot = 0.0;
                sub_fechado = it.ponta & SUB_FECHADO != 0;
                sub_tr = it.traco * caneta;
                sub_per = (it.traco + it.vao) * caneta;
            }
            tot += self.arco(it);
            restantes -= 1;
            if restantes == 0 && tot > melhor {
                (melhor, fechado, tr, per) = (tot, sub_fechado, sub_tr, sub_per);
            }
        }
        if melhor <= 0.0 || per <= 0.0 {
            return 1.0;
        }
        let arredonda = |x: f32| (x + 0.5).floor();
        let denom = if fechado {
            arredonda(melhor / per).max(1.0) * per
        } else {
            arredonda((melhor - tr) / per).max(0.0) * per + tr
        };
        if denom <= 0.0 {
            return 1.0;
        }
        melhor / denom * if fechado { 1.0 + FOLGA_DO_AJUSTE } else { 1.0 }
    }

    fn troco(&self, it: &EixoItem, sub: &SubTracejado, k: u32, s0: f32) -> Troco {
        let len = self.arco(it);
        let fim = s0 + len;
        let n0 = (s0 / sub.per).floor();
        Troco {
            s0,
            fim,
            len,
            corda: self.comprimento(it.a, it.b),
            lprev: self.comprimento(it.d, it.a),
            lnext: self.comprimento(it.b, it.c),
            tem_ant: k > 0 || sub.fechado,
            tem_seg: k + 1 < sub.n || sub.fechado,
            primeiro: k == 0 && sub.fechado,
            ultimo: k + 1 == sub.n && sub.fechado,
            n0,
            n1: (fim / sub.per).floor().min(n0 + TRACOS_POR_TROCO_MAX),
        }
    }

    /// O `emite_tracejado`: o sub-caminho tracejado que começa no troço `i0`.
    fn tracejado(&mut self, i0: usize, ajuste: f32) {
        let it0 = self.eixo[i0];
        let caneta = self.m.caneta;
        let mut sub = SubTracejado {
            n: it0._pad,
            fechado: it0.ponta & SUB_FECHADO != 0,
            tr: it0.traco * caneta * ajuste,
            per: (it0.traco + it0.vao) * caneta * ajuste,
            tot: 0.0,
            a_fim: 0.0,
            emenda: false,
        };
        if sub.per <= 0.0 {
            return;
        }
        let mut i = i0;
        let mut s0 = 0.0_f32;
        for k in 0..sub.n {
            i = self.proximo_troco(i);
            let Some(it) = self.eixo.get(i).copied() else {
                return;
            };
            let tr = self.troco(&it, &sub, k, s0);
            let mut n = tr.n0;
            while n <= tr.n1 {
                if !(sub.fechado && k == 0 && n == 0.0) {
                    let p = pedaco(&tr, &sub, n);
                    if p.valido {
                        self.pedaco(&it, &tr, &p);
                    }
                }
                n += 1.0;
            }
            s0 = tr.fim;
            i += 1;
        }
        if sub.fechado {
            sub.tot = s0;
            sub.a_fim = (s0 / sub.per).floor() * sub.per;
            sub.emenda = sub.a_fim < s0 && s0 < sub.a_fim + sub.tr;
            let it = self.eixo[self.proximo_troco(i0)];
            let tr = self.troco(&it, &sub, 0, 0.0);
            let p = pedaco(&tr, &sub, 0.0);
            if p.valido {
                self.pedaco(&it, &tr, &p);
            }
        }
    }

    /// O `emite_pedaco`: um pedaço de traço — o quadrilátero, a faixa nos vértices ligados, a junta de
    /// quem chega a um vértice ligado que a faixa não cobre, e as pontas onde o traço começa ou acaba.
    fn pedaco(&mut self, it: &EixoItem, tr: &Troco, p: &Pedaco) {
        let r = it.meia_largura * self.m.caneta;
        if tr.corda <= 0.0 || tr.len <= 0.0 || r <= 0.0 {
            return;
        }
        let a = self.aplica(it.a);
        let b = self.aplica(it.b);
        let q0 = if p.x0 <= tr.s0 {
            a
        } else {
            mix(a, b, (p.x0 - tr.s0) / tr.len)
        };
        let q1 = if p.x1 >= tr.fim {
            b
        } else {
            mix(a, b, (p.x1 - tr.s0) / tr.len)
        };
        let ab = b - a;
        let u = v(ab.x / tr.corda, ab.y / tr.corda);
        let nr = perp(u) * r;
        let mut m0 = nr;
        let mut m1 = nr;
        if p.liga0 {
            if let Some(e) = self.bissectriz_ate(
                self.aplica(it.d),
                a,
                b,
                r,
                it.ponta & 4 != 0,
                it.junta,
                it.limite_esquadria,
                p.recuo0,
            ) {
                m0 = e;
            }
        } else {
            self.tampa(q0, -u, r, (it.ponta >> 6) & 3);
        }
        if p.liga1 {
            let quina = it.ponta & 8 != 0;
            let cf = self.aplica(it.c);
            match self.bissectriz_ate(a, b, cf, r, quina, it.junta, it.limite_esquadria, p.recuo1) {
                Some(e) => m1 = e,
                None => self.junta(
                    u,
                    b,
                    cf,
                    r,
                    if quina { it.junta } else { 2 },
                    it.limite_esquadria,
                ),
            }
        } else {
            self.tampa(q1, u, r, (it.ponta >> 8) & 3);
        }
        self.quadrilatero(q0, q1, m0, m1);
    }
}

/// O `mix` do WGSL.
fn mix(a: V, b: V, t: f32) -> V {
    a * (1.0 - t) + b * t
}

/// O `pedaco`: o traço `n` dentro de um troço.
fn pedaco(tr: &Troco, sub: &SubTracejado, n: f32) -> Pedaco {
    let a = n * sub.per;
    let b = a + sub.tr;
    if a >= tr.fim || (a < tr.s0 && b <= tr.s0) {
        return Pedaco::default();
    }
    let mut liga0 = a < tr.s0 && tr.tem_ant;
    let mut la = tr.s0 - a;
    if tr.primeiro && n == 0.0 {
        liga0 = sub.emenda;
        la = sub.tot - sub.a_fim;
    }
    let ld = if tr.ultimo { sub.tr } else { b - tr.fim };
    Pedaco {
        valido: true,
        x0: a.max(tr.s0),
        x1: b.min(tr.fim),
        liga0,
        liga1: b > tr.fim && tr.tem_seg,
        recuo0: 0.5 * tr.lprev.min(la).min(tr.corda.min(b - tr.s0)),
        recuo1: 0.5 * tr.corda.min(tr.fim - a).min(tr.lnext.min(ld)),
    }
}

#[cfg(test)]
#[path = "contorno_cpu_tests.rs"]
mod tests;
