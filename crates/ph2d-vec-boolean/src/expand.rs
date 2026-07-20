//! **Expand** — o traço vira FORMA, e a forma CRESCE. Módulo irmão da booleana.
//!
//! Dois comandos de edição, um motor:
//!
//! - [`outline_stroke`] — o *Outline Stroke* do Illustrator. O traço deixa de ser estilo e
//!   passa a ser geometria: a partir daí ele aceita gradiente, entra numa booleana, ganha
//!   quina viva. É também o pré-requisito de largura variável.
//! - [`offset_path`] — o *Offset Path*. A forma engorda (`d > 0`) ou emagrece (`d < 0`).
//!
//! # Por que isto mora na crate da booleana, e não numa própria
//!
//! O handoff da linha registrou o achado que decide a arquitetura: **offset correto exige
//! remover auto-interseções**, e quem sabe fazer isso é o motor booleano. A
//! `ph2d-vec-scene` é sem-dependências de propósito e todo efeito da pilha é avaliado
//! DENTRO dela, então nenhum efeito alcança a booleana — Offset tem de ser um **comando**,
//! como as booleanas que já existem. E o comando mora aqui porque aqui já se fala as duas
//! línguas (`VecPath` e kurbo): uma crate nova re-derivaria o `to_bez`, e duas portas para
//! *"como um VecPath vira BezPath"* divergem.
//!
//! # Zero geometria nova
//!
//! O offset não tem kernel próprio. O conjunto de pontos a distância ≤ `d` da fronteira de
//! `S` é **exatamente** o traço da fronteira com largura `2d` — chame-o de `B`. Então
//! `S ⊕ disco(d) = S ∪ B` e `S ⊖ disco(d) = S \ B`, e as duas perguntas caem na booleana
//! que já existe. É o mesmo espírito do Shape Builder: o realce e o resultado saem do MESMO
//! motor, então não podem divergir.
//!
//! A junção escolhe o estilo de quina — `Round` é o offset métrico verdadeiro (o disco de
//! verdade); `Miter` deixa a quina afiada e `Bevel` a corta reta, que são os outros dois
//! itens do diálogo do Illustrator.
//!
//! # Geometria de fora entra pelo sweep
//!
//! O contorno que a kurbo devolve é um traçado, não um conjunto: ele **se auto-intersecta**
//! sempre que o caminho se cruza ou que a curvatura aperta mais que a largura da caneta, e
//! não vem agrupado por containment (quem é buraco de quem). Por isso todo conjunto aqui é
//! uma [`Region`], que só se constrói passando pelo sweep — e daí em diante `NonZero` e
//! `EvenOdd` concordam, porque os contornos saem orientados por ele.

use kurbo::{BezPath, Cap, Join, PathEl, Point, Shape, Stroke, StrokeOpts, Vec2};
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{
    FillRule, LineCap, LineJoin, OffsetSide, Paint, StrokePiece, StrokeSpec, VecPath, WidthProfile,
    stroke_plan,
};

use crate::{Closing, binary_grouped, compound_from, flatten_groups, to_bez_with};

/// Tolerância de achatamento do stroke-to-fill, **relativa** ao tamanho da forma.
///
/// Relativa e não absoluta porque o documento não tem unidade fixa: o mesmo comando roda
/// sobre um ícone de 2 unidades e sobre um layout de 2000, e um número absoluto seria fino
/// demais num e grosseiro no outro. 1e-4 da diagonal ⇒ o erro de achatamento fica ~4
/// ordens de grandeza abaixo do que a forma mede, muito além do que o zoom alcança.
const REL_TOL: f64 = 1e-4;

/// Piso da tolerância — uma forma degenerada (todos os pontos coincidentes) tem diagonal
/// zero, e tolerância zero faria a kurbo subdividir para sempre.
const MIN_TOL: f64 = 1e-9;

/// Abaixo disto, [`offset_path`] é IDENTIDADE (devolve vazio — "nada a fazer"). Público
/// porque o preview VIVO da shell precisa da MESMA cerca: para ele, `|d|` abaixo daqui
/// significa "mostre a forma como está", nunca "a forma sumiu" — duas cópias do número
/// divergiriam e o preview apagaria a forma no instante do grab (o slider recentra em 0).
pub const MIN_OFFSET: f64 = MIN_TOL;

/// Um CONJUNTO de pontos: contornos já **orientados pelo sweep** e agrupados por
/// containment (o de fora primeiro, os de dentro depois).
///
/// Só se constrói por [`Region::of`] e só se compõe por [`Region::combine`] — as duas
/// passam pelo sweep, e é isso que garante que um traçado da kurbo vire um CONJUNTO antes
/// de qualquer um perguntar o que está dentro dele.
struct Region {
    groups: Vec<Vec<BezPath>>,
}

impl Region {
    /// O conjunto que `bez` delimita sob `rule`, regularizado.
    ///
    /// A união de um conjunto com ELE MESMO é ele mesmo — e é essa identidade que serve de
    /// porta de entrada: o sweep resolve as auto-interseções (que o traço de um caminho que
    /// se cruza sempre tem) e orienta o que sai.
    fn of(bez: &BezPath, rule: LsFillRule) -> Option<Self> {
        let groups = binary_grouped(bez, bez, rule, BinaryOp::Union)?;
        Some(Self { groups })
    }

    /// `self` OP `other`, como conjuntos.
    fn combine(&self, other: &Self, op: BinaryOp) -> Option<Self> {
        let groups = binary_grouped(
            &flatten_groups(&self.groups),
            &flatten_groups(&other.groups),
            // Depois do sweep os contornos estão orientados: as duas regras concordam.
            LsFillRule::NonZero,
            op,
        )?;
        Some(Self { groups })
    }

    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    fn bez(&self) -> BezPath {
        flatten_groups(&self.groups)
    }

    /// De volta ao documento: um `VecPath` por grupo (compound quando há buraco), com o
    /// estilo de `style`.
    fn into_paths(self, style: &VecPath) -> Vec<VecPath> {
        self.groups
            .iter()
            .filter_map(|g| compound_from(g, style))
            .collect()
    }
}

/// A regra de preenchimento do path, na língua do sweep.
fn rule_of(path: &VecPath) -> LsFillRule {
    match path.fill_rule {
        FillRule::EvenOdd => LsFillRule::EvenOdd,
        FillRule::NonZero => LsFillRule::NonZero,
    }
}

/// Tolerância para esta geometria — ver [`REL_TOL`].
fn tolerance(bez: &BezPath) -> f64 {
    let b = bez.bounding_box();
    (b.width().hypot(b.height()) * REL_TOL).max(MIN_TOL)
}

/// A caneta da LINHA: largura, ponta, junção e tracejado do estilo.
fn line_pen(s: &StrokeSpec) -> Stroke {
    let pen = Stroke::new(s.width)
        .with_caps(match s.cap {
            LineCap::Butt => Cap::Butt,
            LineCap::Round => Cap::Round,
            LineCap::Square => Cap::Square,
        })
        .with_join(join_of(s.join));
    // Os comprimentos vêm do `StrokeSpec` — a MESMA porta que o renderer usa, senão o
    // tracejado assado sairia noutra cadência que o desenhado.
    match s.dash_lengths() {
        Some(d) => pen.with_dashes(0.0, d),
        None => pen,
    }
}

fn join_of(j: LineJoin) -> Join {
    match j {
        LineJoin::Miter => Join::Miter,
        LineJoin::Round => Join::Round,
        LineJoin::Bevel => Join::Bevel,
    }
}

/// O traço de `bez` com `pen`, como conjunto.
fn penned(bez: &BezPath, pen: &Stroke) -> Option<Region> {
    Region::of(&penned_outline(bez, pen), LsFillRule::NonZero)
}

/// O CONTORNO cru do traço, já com a emenda soldada mas **sem passar pelo sweep** — para quem
/// vai acumular muitos e regularizar uma vez só ([`power_stroke`]).
fn penned_outline(bez: &BezPath, pen: &Stroke) -> BezPath {
    let tol = tolerance(bez);
    weld_seams(&kurbo::stroke(bez, pen, &StrokeOpts::default(), tol), tol)
}

/// **Solda a emenda de cada contorno**: o último ponto vira EXATAMENTE o primeiro quando já
/// está a menos de `tol` dele.
///
/// Sem isto o `linesweeper` **recusa o caminho inteiro** com `NonClosedPath`: ele compara as
/// coordenadas do fim com as do começo, e um contorno construído por aproximação de ARCO não
/// volta ao ponto exato. Medido, no cap redondo de uma linha horizontal: o começo é
/// `(0, -1)` e o fim `(-1.22e-16, -1)` — que é o `sin(π)` da última cúbica do arco. O
/// resultado era um Outline Stroke que devolvia **nada** para todo traço com ponta ou junção
/// redonda, calado.
///
/// `tol` é a MESMA tolerância que a kurbo recebeu para construir o contorno: ela promete
/// precisão nessa ordem, então um vão abaixo dela está dentro da barra de erro dela própria
/// e soldá-lo não inventa geometria. Um épsilon escolhido à parte seria um palpite.
fn weld_seams(bez: &BezPath, tol: f64) -> BezPath {
    let mut out = BezPath::new();
    let mut sub: Vec<PathEl> = Vec::new();
    let flush = |out: &mut BezPath, sub: &mut Vec<PathEl>| {
        if let [PathEl::MoveTo(start), ..] = sub[..] {
            // O último elemento que POUSA em algum lugar é o que fecha a emenda.
            if let Some(last) = sub.iter_mut().rev().find(|e| end_of(e).is_some())
                && let Some(p) = end_of(last)
                && p != start
                && (p - start).hypot() <= tol
            {
                retarget(last, start);
            }
        }
        out.extend(sub.drain(..));
    };
    for el in bez.elements() {
        match *el {
            PathEl::MoveTo(_) => {
                if !sub.is_empty() {
                    flush(&mut out, &mut sub);
                }
                sub.push(*el);
            }
            PathEl::ClosePath => {
                flush(&mut out, &mut sub);
                out.push(PathEl::ClosePath);
            }
            _ => sub.push(*el),
        }
    }
    if !sub.is_empty() {
        flush(&mut out, &mut sub);
    }
    out
}

/// Onde este elemento POUSA (`None` para `ClosePath`, que não carrega ponto).
fn end_of(el: &PathEl) -> Option<Point> {
    match *el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => {
            Some(p)
        }
        PathEl::ClosePath => None,
    }
}

/// Move o ponto de chegada de `el` para `to`, preservando os controles.
fn retarget(el: &mut PathEl, to: Point) {
    match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => {
            *p = to;
        }
        PathEl::ClosePath => {}
    }
}

/// **Outline Stroke** — o traço de `path` vira forma(s) preenchida(s) com a cor dele.
///
/// Devolve vazio quando não há o que converter (sem traço, ou geometria degenerada).
///
/// **O que é convertido é o que está na TELA**: as peças vêm de
/// [`ph2d_vec_scene::stroke_plan`], a mesma receita que o renderer pinta — incluindo o
/// tracejado (cada traço vira uma forma) e as PONTAS (uma seta convertida continua tendo a
/// cabeça; perdê-la em silêncio seria apagar desenho). E a geometria de partida é a
/// **cozida**, então quina viva e a pilha de efeitos entram assadas, como o olho as vê.
///
/// ⚠️ **O preenchimento NÃO vem junto.** Este comando converte o TRAÇO; se o path também
/// tinha fill, essa região continua sendo o path original, que o chamador preserva. Assar
/// as duas coisas numa forma só as fundiria num blob de uma cor — o traço e o miolo têm
/// cores diferentes justamente porque são coisas diferentes.
#[must_use]
pub fn outline_stroke(path: &VecPath) -> Vec<VecPath> {
    let Some(s) = path.stroke else {
        return Vec::new();
    };
    let mut acc: Option<Region> = None;
    for piece in stroke_plan(path, &s) {
        let region = match piece {
            StrokePiece::Line { path: p } => {
                penned(&to_bez_with(&p, Closing::AsDrawn), &line_pen(&s))
            }
            // A caneta do símbolo é crua e sólida — ver `StrokePiece::Symbol`.
            StrokePiece::Symbol { path: p } => {
                penned(&to_bez_with(&p, Closing::AsDrawn), &Stroke::new(s.width))
            }
            // Uma ponta cheia já É uma região.
            StrokePiece::Fill { path: p } => {
                Region::of(&to_bez_with(&p, Closing::Always), LsFillRule::NonZero)
            }
        };
        let Some(region) = region.filter(|r| !r.is_empty()) else {
            continue;
        };
        acc = Some(match acc {
            None => region,
            Some(a) => match a.combine(&region, BinaryOp::Union) {
                Some(u) => u,
                None => return Vec::new(),
            },
        });
    }
    match acc {
        Some(acc) => acc.into_paths(&ink_style(&s)),
        None => Vec::new(),
    }
}

/// O estilo de uma forma que **É** o traço: preenchida com a cor dele, e sem traço.
///
/// Deixar o traço no resultado o desenharia uma segunda vez, agora em volta de si mesmo,
/// engordando o desenho no clique que não devia mudar nada. Porta única porque os dois
/// comandos que assam tinta ([`outline_stroke`] e [`power_stroke`]) têm de responder igual.
fn ink_style(s: &StrokeSpec) -> VecPath {
    VecPath {
        fill: Some(Paint::Solid(s.color)),
        stroke: None,
        ..VecPath::default()
    }
}

/// **Power Stroke** — o traço com largura VARIÁVEL, assado em forma preenchida.
///
/// # A largura variável é uma FITA de dois trilhos, não uma pilha de discos
///
/// A primeira versão fatiava o arco e unia um disco por fatia. Convergia, mas cada disco tem
/// **ponta redonda**, e a união de discos de larguras vizinhas deixa um FESTÃO na borda — o
/// "traço rugoso" que o smoke reprovou (`2026-07-20`). A borda de um traço de largura
/// variável não é uma sucessão de tampas: é uma curva contínua a `w(s)/2` da linha de centro.
///
/// Então o motor é o clássico do Inkscape/Illustrator: achata a linha de centro num polígono
/// fino e, em cada ponto, desloca por `±w(s)/2` na NORMAL — dois **trilhos**, ligados por
/// tampas nas pontas, formando UM contorno. A borda passa a seguir `w(s)` de forma lisa; não
/// há festão porque não há tampa entre amostras. As cúspides e auto-cruzamentos (onde a
/// curvatura aperta mais que a largura) somem no sweep, como sempre — o [`Region::of`]
/// regulariza a fita UMA vez, e é isso que dispensa o offsetter analítico que a pesquisa do
/// módulo diz que ninguém tem.
///
/// É também mais barato: um polígono e um sweep, no lugar de 64 traçados-e-sweeps.
///
/// Devolve vazio sem traço, com perfil UNIFORME (aí o comando é o [`outline_stroke`], e ter
/// dois botões para a mesma saída seria pior que ter um), ou se o sweep falhar.
#[must_use]
pub fn power_stroke(path: &VecPath, profile: &WidthProfile) -> Vec<VecPath> {
    let Some(s) = path.stroke.filter(|_| !profile.is_uniform()) else {
        return Vec::new();
    };
    let cooked = path.cooked();
    let mut ink = BezPath::new();
    for c in 0..cooked.contour_count() {
        let Some((verts, closed)) = cooked.contour(c) else {
            continue;
        };
        let piece = VecPath {
            verts: verts.to_vec(),
            closed,
            ..VecPath::default()
        };
        ribbon_into(
            &mut ink,
            &to_bez_with(&piece, Closing::AsDrawn),
            &s,
            profile,
            closed,
        );
    }
    match Region::of(&ink, LsFillRule::NonZero).filter(|r| !r.is_empty()) {
        Some(acc) => drop_slivers(acc.into_paths(&ink_style(&s))),
        None => Vec::new(),
    }
}

/// Remove as **lascas** de área ~nula que o sweep deixa nos PINÇOS da fita (onde a curvatura
/// aperta mais que a largura, o trilho de dentro inverte e o motor emite um contorno
/// degenerado de área zero — medido: a senoide adversária saía com a peça real + duas lascas
/// pontuais). É a mesma lasca que o Shape Builder já paga para reconhecer: sem área não há
/// preenchimento, então ela pinta como uma LINHA solta.
///
/// O piso é RELATIVO ao total (livre de escala) e não uma densidade: um traço fino e ondulado
/// tem densidade baixa mas área REAL, e um piso de densidade o mataria junto com a lasca.
fn drop_slivers(paths: Vec<VecPath>) -> Vec<VecPath> {
    let total: f64 = paths.iter().map(crate::area).sum();
    if total <= MIN_TOL {
        return Vec::new();
    }
    paths
        .into_iter()
        .filter(|p| crate::area(p) > total * 1e-4)
        .collect()
}

/// Amostras mínimas ao longo do arco. A geometria já foi capturada pelo achatamento; isto é a
/// densidade em que o PERFIL de largura é lido — e ele varia mesmo onde a linha é RETA (o
/// achatamento não subdivide uma reta, então uma linha reta voltava com dois pontos só e o
/// meio grosso do perfil nunca era amostrado). 128 é folgado para o `smoothstep`, que é liso.
const RIBBON_SAMPLES: usize = 128;

/// Achata `bez` (um contorno) num polígono fino, o **densifica por arco** para amostrar o
/// perfil de largura, e devolve os pontos + o `t` de ARCO normalizado de cada um (`0` no
/// começo, `1` no fim). `None` se degenerar.
///
/// Por arco e não por parâmetro de Bézier: é a unidade em que o [`WidthProfile`] mora, a mesma
/// do Zig Zag e do Blend — duas formas que se veem iguais têm de se comportar igual.
fn flatten_arc(bez: &BezPath, closed: bool) -> Option<(Vec<Point>, Vec<f64>)> {
    let tol = tolerance(bez);
    let mut raw: Vec<Point> = Vec::new();
    kurbo::flatten(bez.elements().iter().copied(), tol, |el| match el {
        PathEl::MoveTo(p) if raw.is_empty() => raw.push(p),
        PathEl::LineTo(p) => raw.push(p),
        _ => {}
    });
    // Amostras coincidentes (o achatamento repete um ponto numa tangente afiada) quebram a
    // normal por diferença — funde. No fechado, o último ponto volta ao primeiro.
    raw.dedup_by(|a, b| (*a - *b).hypot() <= MIN_TOL);
    if closed && raw.len() >= 2 && (raw[0] - raw[raw.len() - 1]).hypot() <= MIN_TOL {
        raw.pop();
    }
    if raw.len() < 2 {
        return None;
    }
    // Comprimento total do arco (o fechado inclui o segmento de volta ao início).
    let n = raw.len();
    let seg_count = if closed { n } else { n - 1 };
    let total: f64 = (0..seg_count)
        .map(|i| (raw[(i + 1) % n] - raw[i]).hypot())
        .sum();
    if total <= MIN_TOL {
        return None;
    }
    // Densifica: nenhum vão maior que `step`. Preserva os pontos do achatamento (a geometria) e
    // insere amostras nos trechos longos (a variação de largura).
    #[allow(clippy::cast_precision_loss)]
    let step = total / RIBBON_SAMPLES as f64;
    let mut pts = Vec::with_capacity(RIBBON_SAMPLES + n);
    let mut arc = Vec::with_capacity(RIBBON_SAMPLES + n);
    let mut acc = 0.0;
    for i in 0..seg_count {
        let (a, b) = (raw[i], raw[(i + 1) % n]);
        let seg = (b - a).hypot();
        pts.push(a);
        arc.push(acc / total);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let subs = (seg / step).ceil() as usize;
        for k in 1..subs {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f64 / subs as f64;
            pts.push(a.lerp(b, f));
            arc.push((acc + seg * f) / total);
        }
        acc += seg;
    }
    if !closed {
        pts.push(raw[n - 1]);
        arc.push(1.0);
    }
    Some((pts, arc))
}

/// A normal unitária (perpendicular à tangente) no ponto `i` de `pts`. Cíclica quando
/// `closed`, forward/backward nas pontas quando aberto.
fn normal_at(pts: &[Point], i: usize, closed: bool) -> Vec2 {
    let n = pts.len();
    let tangent = if closed {
        pts[(i + 1) % n] - pts[(i + n - 1) % n]
    } else if i == 0 {
        pts[1] - pts[0]
    } else if i == n - 1 {
        pts[n - 1] - pts[n - 2]
    } else {
        pts[i + 1] - pts[i - 1]
    };
    let len = tangent.hypot();
    if len <= MIN_TOL {
        return Vec2::new(0.0, 0.0);
    }
    let t = tangent / len;
    Vec2::new(-t.y, t.x)
}

/// **A fita** de `bez`: acumula em `ink` o contorno cru (pré-sweep) dos dois trilhos a
/// `±w(s)/2` da linha de centro, **um quadrilátero por segmento**.
///
/// Por quadrilátero, e não um polígono só: onde a curvatura aperta mais que a largura os
/// trilhos se CRUZAM, e um polígono único auto-intersectante confunde o winding do `NonZero`
/// (medido: a senoide adversária saía em 3 peças). Os quads vizinhos partilham a aresta
/// `[esquerda_{i+1}, direita_{i+1}]` — estão COLADOS por ela, então a união é conexa mesmo
/// quando um quad se torce. A borda externa continua sendo os trilhos (lisa, sem festão), e um
/// contorno FECHADO vira um anel de graça (os quads ladrilham a fita; o miolo fica sem
/// cobertura = o furo).
fn ribbon_into(
    ink: &mut BezPath,
    bez: &BezPath,
    s: &StrokeSpec,
    profile: &WidthProfile,
    closed: bool,
) {
    let Some((pts, arc)) = flatten_arc(bez, closed) else {
        return;
    };
    let half = |i: usize| 0.5 * s.width * profile.at(arc[i]).max(0.0);
    let rail = |i: usize, sign: f64| pts[i] + normal_at(&pts, i, closed) * (sign * half(i));
    let n = pts.len();
    let seg_count = if closed { n } else { n - 1 };
    for i in 0..seg_count {
        let j = (i + 1) % n;
        push_loop(
            ink,
            [rail(i, 1.0), rail(j, 1.0), rail(j, -1.0), rail(i, -1.0)].into_iter(),
        );
    }
    if !closed {
        cap_loop(
            ink,
            pts[n - 1],
            half(n - 1),
            normal_at(&pts, n - 1, false),
            s.cap,
            true,
        );
        cap_loop(
            ink,
            pts[0],
            half(0),
            normal_at(&pts, 0, false),
            s.cap,
            false,
        );
    }
}

/// Emite `pts` como um subpath fechado (poligonal) em `ink`.
fn push_loop(ink: &mut BezPath, mut pts: impl Iterator<Item = Point>) {
    if let Some(first) = pts.next() {
        ink.move_to(first);
        for p in pts {
            ink.line_to(p);
        }
        ink.close_path();
    }
}

/// A **tampa** da ponta, como um contorno fechado próprio a unir com o último quad: liga o
/// trilho de um lado ao do outro em torno do centro `c` com meia-largura `h`. `forward` = a
/// tampa do FIM (bojo para +tangente). `normal` é a normal do centro naquele extremo.
fn cap_loop(ink: &mut BezPath, c: Point, h: f64, normal: Vec2, cap: LineCap, forward: bool) {
    if h <= MIN_TOL {
        return; // ponta afilada num bico — nada a arredondar
    }
    // A direção do bojo é ±tangente = ∓ a perpendicular da normal (a normal aponta para o
    // trilho esquerdo; girá-la −90° dá a tangente para a frente).
    let dir = Vec2::new(normal.y, -normal.x) * if forward { 1.0 } else { -1.0 };
    let mut poly: Vec<Point> = Vec::with_capacity(16);
    // O diâmetro (a aresta que o quad já toca) + o miolo do arco/quadrado à frente.
    poly.push(c + normal * h);
    match cap {
        // Butt: o quad já tem a aresta reta na ponta — não há tampa a somar.
        LineCap::Butt => return,
        // Square: dois cantos projetados `h` à frente.
        LineCap::Square => {
            poly.push(c + normal * h + dir * h);
            poly.push(c - normal * h + dir * h);
        }
        // Round: semicírculo de raio `h`, amostrado. A borda macia da caligrafia.
        LineCap::Round => {
            const SEGS: usize = 12;
            for k in 1..SEGS {
                #[allow(clippy::cast_precision_loss)]
                let theta = std::f64::consts::PI * (k as f64) / (SEGS as f64);
                poly.push(c + (normal * theta.cos() + dir * theta.sin()) * h);
            }
        }
    }
    poly.push(c - normal * h);
    push_loop(ink, poly.into_iter());
}

/// **Offset Path** — move a borda da forma por `d`, com quinas em `join`, e `side` diz QUAIS
/// contornos participam.
///
/// # É POR CONTORNO, e é isso que faz a quina aparecer no furo
///
/// Cada contorno (o de fora, e cada furo de um compound) é offsetado por conta própria: `d>0`
/// o empurra para FORA, ao longo da sua normal externa (para longe do miolo que ele fecha);
/// `d<0` para dentro. Um crescer arredonda as quinas CONVEXAS do contorno — e como o furo tem
/// as suas, `d>0` num furo (Inner/Both) faz o furo crescer para dentro da tinta e **suas
/// quinas ganham Round/Bevel**, que era o que o smoke pedia (`2026-07-20`, "round e bevel só
/// no externo"). A versão anterior offsetava a região INTEIRA de uma vez (a Minkowski do
/// conjunto): correta, mas `d>0` só expande a fronteira externa, então o furo — que é
/// côncavo visto da tinta — nunca arredondava.
///
/// A geometria de partida é a **cozida** e o estilo é **preservado** (≠ Outline Stroke, que
/// produz uma forma nova): offsetar é mover a borda da mesma arte. Devolve vazio se `d` for
/// ~0, se o sweep falhar, ou se a forma **desaparecer** ao encolher (resposta correta: um
/// traço fino encolhido demais não sobra).
#[must_use]
pub fn offset_path(path: &VecPath, d: f64, join: LineJoin, side: OffsetSide) -> Vec<VecPath> {
    if d.abs() < MIN_OFFSET || !d.is_finite() {
        return Vec::new();
    }
    // Regulariza ANTES de offsetar: os contornos têm de sair orientados e agrupados (quem é
    // furo de quem), senão não dá para offsetar cada um por conta própria.
    let Some(region) = Region::of(&to_bez_with(path, Closing::Always), rule_of(path)) else {
        return Vec::new();
    };
    let mut acc: Option<Region> = None;
    for group in &region.groups {
        // O grupo é `[fora, furo…]` — o linesweeper já os agrupa por containment.
        let Some((outer, holes)) = group.split_first() else {
            continue;
        };
        let Some(outer_r) =
            loop_region(outer, side.hits_outer(), d, join).filter(|r| !r.is_empty())
        else {
            continue; // o contorno de fora sumiu ao encolher — o grupo inteiro sai
        };
        // ⚠️ **UM contorno passando por cima do outro TROCA de papel — não some.** Junta TODOS
        // os contornos (o de fora + cada furo, offsetados ou não) e deixa o SWEEP decidir a
        // topologia por **EvenOdd**: um ponto está dentro sse um número ÍMPAR de contornos o
        // cerca. É o que faz o furo, ao crescer ALÉM da borda, virar o novo contorno de fora
        // (o que era buraco vira sólido — os "contornos trocados" que o Enio pediu), em vez de
        // fragmentar e sumir. A subtração sequencial `fora − furos` quebrava exatamente aqui: a
        // diferença ia a vazio e a forma "pulava"/sumia. EvenOdd é livre de orientação, então
        // os contornos de offsets independentes compõem sem se cancelar.
        let mut all = outer_r.bez();
        for hole in holes {
            if let Some(hr) =
                loop_region(hole, side.hits_inner(), d, join).filter(|r| !r.is_empty())
            {
                all.extend(hr.bez().iter());
            }
        }
        if let Some(group_r) = Region::of(&all, LsFillRule::EvenOdd).filter(|r| !r.is_empty()) {
            acc = Some(match acc {
                None => group_r,
                Some(a) => a.combine(&group_r, BinaryOp::Union).unwrap_or(a),
            });
        }
    }
    match acc {
        Some(acc) => drop_slivers(acc.into_paths(path)),
        None => Vec::new(),
    }
}

/// Achata `bez` em RETAS na tolerância da forma — o preço que faz o Offset AO VIVO caber
/// num frame.
///
/// O sweep sobre CÚBICAS é caro: na rosquinha do smoke com quina **Round** (a banda vira
/// arcos), `offset_path` custava **19–43 ms** contra ~1 ms do Miter/Bevel (medido, debug) —
/// o arrasto caía a ~12 fps ("severa queda de FPS", Enio 2026-07-20). Sobre retas o mesmo
/// sweep custa ~1 ms. É a MESMA moeda do power stroke (o ribbon achata e varre uma vez): o
/// erro fica na [`tolerance`] relativa, ~4 ordens abaixo do que a forma mede — e o output
/// do sweep já fragmentava os arcos em centenas de verts de qualquer jeito.
fn flat_lines(bez: &BezPath) -> BezPath {
    let tol = tolerance(bez);
    let mut out = BezPath::new();
    kurbo::flatten(bez.iter(), tol, |el| out.push(el));
    out
}

/// A região que UM contorno fechado delimita, opcionalmente offsetada por `d` (com quinas em
/// `join`). Sem offset (`offset == false`), é só a região do contorno como está.
///
/// O offset de um laço é a MESMA redução da booleana de antes: o traço da fronteira com
/// largura `2|d|` é a banda, e a região `∪ banda` (crescer) ou `∖ banda` (encolher). A quina
/// escolhe o estilo — `Round` é o offset métrico verdadeiro (o disco de verdade).
///
/// ⚠️ Tudo que entra num sweep aqui passa por [`flat_lines`] — este caminho roda POR FRAME
/// no arrasto do slider, e cúbica no sweep não cabe num frame (ver o doc do `flat_lines`).
fn loop_region(loop_bez: &BezPath, offset: bool, d: f64, join: LineJoin) -> Option<Region> {
    let base = Region::of(&flat_lines(loop_bez), LsFillRule::NonZero)?;
    if !offset {
        return Some(base);
    }
    let pen = Stroke::new(2.0 * d.abs())
        .with_join(join_of(join))
        .with_caps(Cap::Butt);
    let band = Region::of(
        &flat_lines(&penned_outline(&base.bez(), &pen)),
        LsFillRule::NonZero,
    )?;
    if band.is_empty() {
        return Some(base);
    }
    let op = if d > 0.0 {
        BinaryOp::Union
    } else {
        BinaryOp::Difference
    };
    base.combine(&band, op)
}

#[cfg(test)]
#[path = "expand_tests.rs"]
mod tests;
