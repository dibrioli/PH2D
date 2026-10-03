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
//! ⭐ **O tracejado mede-se no ECRÃ** (doc 121 §9.9): sob escala não uniforme o comprimento de arco no
//! mundo não é proporcional ao local, e a lei da casa traceja a geometria JÁ transformada com o padrão
//! `× √|det|`. Os troços levam o padrão e as pontas; o shader percorre cada sub-caminho somando o
//! comprimento no ecrã e corta os traços onde o `kurbo::dash` os cortaria. Só um padrão que o eixo
//! não exprime ([`tracejado_do_eixo`]) fica com a [`crate::geometry::FLAG_SO_CONFORME`].

use bytemuck::{Pod, Zeroable};
use ph2d_vector::{
    BezPath, Cap, Join, ParamCurve, ParamCurveNearest, PathEl, Point, Stroke, flatten,
};

/// Um troço do eixo: de `a` a `b`, com o vizinho de trás em `d` e o da frente em `c` (lidos só
/// quando o bit [`FAIXA_INICIO`]/[`FAIXA_FIM`] de `ponta` está aceso).
///
/// ⭐ **Não há peça de junta** (doc 121 §9.4): o vértice entre dois troços é do troço que CHEGA a
/// ele. Os dois acabam na mesma bissectriz quando ela serve — a aresta partilhada cancela-se na
/// soma —, e quando não serve o que chega põe a junta (a autorada numa quina, a redonda num ponto
/// liso). Até 01/10 cada vértice era uma peça própria, com caixa, três pontos e quatro arestas.
pub const ITEM_TROCO: u32 = 0;
/// No `ponta` de um TROÇO: o vértice `a` é INTERIOR (tem o vizinho `d`) — o troço começa na
/// bissectriz que o shader calcular, ou na normal se ela não servir.
pub const FAIXA_INICIO: u32 = 1;
/// No `ponta` de um TROÇO: o vértice `b` é INTERIOR (tem o vizinho `c`) — o troço acaba na
/// bissectriz, ou na normal MAIS a junta.
pub const FAIXA_FIM: u32 = 2;
/// No `ponta` de um TROÇO: o vértice `a` é uma QUINA (as tangentes discordam). Ali a bissectriz só
/// serve se a junta autorada for a ESQUADRIA dentro do limite — e é então exactamente ela.
pub const QUINA_INICIO: u32 = 4;
/// No `ponta` de um TROÇO: o vértice `b` é uma QUINA.
pub const QUINA_FIM: u32 = 8;
/// (O tipo `1` foi a JUNTA, retirada a 01/10 — os números dos outros ficam onde a placa os lê.)
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

/// doc 121 §9.9 — no `ponta` de um troço TRACEJADO: o primeiro troço do sub-caminho (o `_pad` dele
/// diz quantos troços o sub-caminho tem).
pub const SUB_INICIO: u32 = 16;
/// No `ponta` do primeiro troço tracejado: o sub-caminho é FECHADO.
pub const SUB_FECHADO: u32 = 32;
/// No `ponta` de um troço tracejado, a ponta do INÍCIO de cada traço (`PONTA_*`) a partir deste bit,
/// e a do FIM dois bits acima — o kurbo põe a `start_cap` e a `end_cap` em cada traço.
pub const TAMPA_INICIO_BIT: u32 = 6;
pub const TAMPA_FIM_BIT: u32 = 8;
/// No `ponta` de um cabeçalho de BLOCO: há um troço tracejado lá dentro — o desenho pixel a pixel
/// não o salta pela caixa (o comprimento de arco tem de passar por todos os troços).
pub const BLOCO_TRACEJADO: u32 = 1;

/// Um item do eixo, como a placa o lê (`shape.wgsl`, `Eixo`): `72` bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct EixoItem {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub c: [f32; 2],
    /// O vizinho de TRÁS de um troço (o `c` é o da frente). Zero nos outros tipos.
    pub d: [f32; 2],
    /// Metade da largura AUTORADA (unidades locais) — o shader multiplica-a por `√|det|`.
    pub meia_largura: f32,
    pub limite_esquadria: f32,
    pub tipo: u32,
    pub junta: u32,
    pub ponta: u32,
    pub _pad: u32,
    /// doc 121 §9.9 — o tracejado do troço em unidades LOCAIS (`traço`, `vão`); `(0, 0)` ⇒ contínuo.
    /// O shader multiplica-o pela caneta `√|det|`, como a casa (`stroke_uniform::pen_for`).
    pub traco: f32,
    pub vao: f32,
    /// A FLECHA da corda `a → b` (local): o ponto da curva no meio dela menos o meio da corda; zero
    /// num troço recto. O tracejado mede o ARCO no ecrã (`c + 8h²/3c`): só com as cordas, os cantos
    /// arredondados ficavam curtos e o padrão escorregava ao longo do contorno.
    pub flecha: [f32; 2],
}

/// O tracejado que o eixo sabe percorrer: `[traço, vão]` com fase `0` — o que o `kurbo_stroke` da
/// casa produz. `None` para um traço contínuo e para um padrão que o eixo não exprime (outra
/// contagem, fase, período nulo): esse fica com a [`crate::geometry::FLAG_SO_CONFORME`].
#[must_use]
pub fn tracejado_do_eixo(style: &Stroke) -> Option<[f32; 2]> {
    let [t, v] = style.dash_pattern.as_slice() else {
        return None;
    };
    let ok = |x: f64| x.is_finite() && x >= 0.0;
    (style.dash_offset == 0.0 && ok(*t) && ok(*v) && t + v > 0.0).then(|| {
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        let p = [*t as f32, *v as f32];
        p
    })
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
    /// A flecha da corda que CHEGA a cada ponto (zero no primeiro).
    flechas: Vec<Point>,
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
                    flechas: vec![Point::ZERO],
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
                let seg = bp.segments().next();
                flatten(bp.iter(), tol, |e| {
                    if let PathEl::LineTo(p) = e
                        && let Some(&u) = s.pontos.last()
                        && u != p
                    {
                        let meio = u.midpoint(p);
                        let flecha = seg.map_or(Point::ZERO, |sg| {
                            let c = sg.eval(sg.nearest(meio, 1e-9).t);
                            Point::new(c.x - meio.x, c.y - meio.y)
                        });
                        s.pontos.push(p);
                        s.quinas.push(false);
                        s.flechas.push(flecha);
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
            for q in [it.a, it.b] {
                for k in 0..2 {
                    lo[k] = lo[k].min(q[k]);
                    hi[k] = hi[k].max(q[k]);
                }
            }
            // O mesmo alcance que o shader dá à peça (`alcance_da_peca`).
            let fator = match it.tipo {
                ITEM_TROCO if alcanca_a_esquadria(it) => it.limite_esquadria.max(1.0),
                ITEM_PONTA if it.ponta == PONTA_QUADRADA => 1.5,
                _ => 1.0,
            };
            alcance = alcance.max(it.meia_largura * fator);
        }
        let tracejado = bloco.iter().any(|it| it.traco + it.vao > 0.0);
        out.push(EixoItem {
            a: lo,
            b: hi,
            meia_largura: alcance,
            tipo: ITEM_BLOCO,
            ponta: if tracejado { BLOCO_TRACEJADO } else { 0 },
            #[expect(clippy::cast_possible_truncation, reason = "um bloco tem 16 peças")]
            _pad: bloco.len() as u32,
            ..EixoItem::default()
        });
        out.extend_from_slice(bloco);
    }
    out
}

/// O troço pode acabar numa esquadria autorada (uma das pontas é quina e a junta é a esquadria) —
/// então ele vai até `limite` meias larguras para fora do eixo, e não só `1`. O MESMO teste que o
/// shader faz em `alcance_da_peca`.
#[must_use]
pub fn alcanca_a_esquadria(it: &EixoItem) -> bool {
    it.tipo == ITEM_TROCO
        && it.junta == JUNTA_ESQUADRIA
        && it.ponta & (QUINA_INICIO | QUINA_FIM) != 0
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
    let tracejado = tracejado_do_eixo(style);
    let [traco, vao] = tracejado.unwrap_or_default();
    let base = EixoItem {
        meia_largura: meia,
        limite_esquadria: limite,
        junta: junta_de(style.join),
        traco,
        vao,
        ..EixoItem::default()
    };
    // ⭐ doc 121 §9.9 — com tracejado cada TRAÇO tem pontas (o shader põe-nas onde o traço começa e
    // acaba): os troços levam o estilo delas, e não há itens de ponta.
    let tampas = if tracejado.is_some() {
        (ponta_de(style.start_cap) << TAMPA_INICIO_BIT) | (ponta_de(style.end_cap) << TAMPA_FIM_BIT)
    } else {
        0
    };
    let mut ext: f32 = 1.0;
    if tracejado.is_some() && (style.start_cap == Cap::Square || style.end_cap == Cap::Square) {
        ext = ext.max(std::f32::consts::SQRT_2);
    }
    for s in sub_caminhos(path, tol) {
        // ⚠️ Os pontos vão à placa em `f32`, e dois pontos distintos em `f64` podem coincidir lá:
        // um troço de comprimento ZERO partiria a faixa (o shader não tem direcção para ele, e os
        // vizinhos dele perderiam a bissectriz partilhada). ⇒ a deduplicação é sobre o que a placa
        // LÊ, e uma quina absorvida passa ao ponto que fica.
        let mut p: Vec<[f32; 2]> = Vec::with_capacity(s.pontos.len());
        let mut q: Vec<bool> = Vec::with_capacity(s.pontos.len());
        let mut fl: Vec<[f32; 2]> = Vec::with_capacity(s.pontos.len());
        for ((pt, &eh), fc) in s.pontos.iter().zip(&s.quinas).zip(&s.flechas) {
            let pf = f(*pt);
            if p.last() == Some(&pf) {
                if let Some(u) = q.last_mut() {
                    *u |= eh;
                }
            } else {
                p.push(pf);
                q.push(eh);
                fl.push(f(*fc));
            }
        }
        // Um fechado cujo último ponto É o primeiro: tira-se o repetido, e a junta do fecho fica
        // no ponto 0 (com a flecha da corda que lá chega).
        if s.fechado && p.len() > 2 && p.first() == p.last() {
            p.pop();
            q.pop();
            if let Some(ultima) = fl.pop() {
                fl[0] = ultima;
            }
        }
        let n = p.len();
        if n < 2 {
            continue;
        }
        // A quina de cada VÉRTICE: num fechado a do ponto `0` é a do fecho; num aberto os extremos
        // não são vértices (lá moram as pontas).
        if s.fechado {
            q[0] = match (s.t_fim, s.t_inicio) {
                (Some(a), Some(b)) => quina(a, b),
                _ => true,
            };
        }
        let interior = |i: usize| s.fechado || (i > 0 && i + 1 < n);
        // ⭐ **A FAIXA** (doc 121 §9.4): a junta deixa de ser uma peça — os dois troços que se
        // encontram num vértice acabam na MESMA bissectriz, e a aresta partilhada cancela-se na
        // soma. O shader decide, no ecrã, se a bissectriz serve: num ponto liso quando a esquadria
        // fica a `0,1 px` do arco, numa quina quando a junta autorada É a esquadria e cabe no
        // limite. Quando não serve, o troço que CHEGA ao vértice põe a junta.
        let troços = if s.fechado { n } else { n - 1 };
        for i in 0..troços {
            let j = (i + 1) % n;
            let mut flags = tampas;
            let mut pad = 0;
            if tracejado.is_some() && i == 0 {
                flags |= SUB_INICIO | if s.fechado { SUB_FECHADO } else { 0 };
                pad = u32::try_from(troços)
                    .expect("um sub-caminho com mais de 4 mil milhoes de troços");
            }
            if interior(i) {
                flags |= FAIXA_INICIO | if q[i] { QUINA_INICIO } else { 0 };
            }
            if interior(j) {
                flags |= FAIXA_FIM | if q[j] { QUINA_FIM } else { 0 };
                if q[j] && base.junta == JUNTA_ESQUADRIA {
                    ext = ext.max(limite);
                }
            }
            out.push(EixoItem {
                a: p[i],
                b: p[j],
                c: p[(i + 2) % n],
                d: p[(i + n - 1) % n],
                tipo: ITEM_TROCO,
                ponta: flags,
                _pad: pad,
                flecha: if tracejado.is_some() { fl[j] } else { [0.0; 2] },
                ..base
            });
        }
        if !s.fechado && tracejado.is_none() {
            for (de, em, cap) in [
                (p[1], p[0], style.start_cap),
                (p[n - 2], p[n - 1], style.end_cap),
            ] {
                if cap == Cap::Square {
                    ext = ext.max(std::f32::consts::SQRT_2);
                }
                out.push(EixoItem {
                    a: de,
                    b: em,
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
