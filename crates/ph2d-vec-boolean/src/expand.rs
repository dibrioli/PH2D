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
//! # A armadilha da ORIENTAÇÃO
//!
//! Contornos vindos da kurbo não têm orientação combinada com nada. Concatená-los num
//! `BezPath` só e pedir `NonZero` faz dois de sentidos opostos **se cancelarem** na
//! sobreposição — a forma ganha um buraco que ninguém pediu. Por isso todo conjunto aqui
//! passa por [`Region`], que só se constrói através do sweep (que ORIENTA), e só se compõe
//! por operação de conjunto (que testa cada operando por si).

use kurbo::{BezPath, Cap, Join, PathEl, Point, Shape, Stroke, StrokeOpts};
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{
    FillRule, LineCap, LineJoin, Paint, StrokePiece, StrokeSpec, VecPath, stroke_plan,
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

/// Um CONJUNTO de pontos: contornos já **orientados pelo sweep** e agrupados por
/// containment (o de fora primeiro, os de dentro depois).
///
/// Só se constrói por [`Region::of`] e só se compõe por [`Region::combine`] — as duas
/// passam pelo sweep. É essa restrição que torna impossível a cancelação por orientação
/// descrita no cabeçalho do módulo.
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

    /// `self` OP `other`, como conjuntos. Cada operando é testado por si, então orientações
    /// discordantes não se cancelam.
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
    let tol = tolerance(bez);
    let outline = kurbo::stroke(bez, pen, &StrokeOpts::default(), tol);
    Region::of(&weld_seams(&outline, tol), LsFillRule::NonZero)
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
    let Some(acc) = acc else {
        return Vec::new();
    };
    // A forma resultante é PREENCHIDA com a cor que o traço tinha, e não tem traço: ela É o
    // traço. Deixar o traço no resultado o desenharia uma segunda vez, agora em volta de si
    // mesmo, engordando o desenho no clique que não devia mudar nada.
    let style = VecPath {
        fill: Some(Paint::Solid(s.color)),
        stroke: None,
        ..VecPath::default()
    };
    acc.into_paths(&style)
}

/// **Offset Path** — a forma cresce (`d > 0`) ou encolhe (`d < 0`) por `d`, com quinas em
/// `join`. Devolve vazio se `d` for ~0, se o sweep falhar, ou se a forma **desaparecer** ao
/// encolher (o que é uma resposta correta: um traço fino encolhido de mais não sobra).
///
/// A geometria de partida é a **cozida** e o estilo é **preservado** (≠ Outline Stroke, que
/// produz uma forma nova): offsetar é mover a borda da mesma arte.
#[must_use]
pub fn offset_path(path: &VecPath, d: f64, join: LineJoin) -> Vec<VecPath> {
    if d.abs() < MIN_TOL || !d.is_finite() {
        return Vec::new();
    }
    // Regulariza ANTES de traçar: a banda tem de ser construída sobre a fronteira real do
    // conjunto (auto-interseções já resolvidas), senão ela descreve uma borda que a forma
    // não tem.
    let Some(region) = Region::of(&to_bez_with(path, Closing::Always), rule_of(path)) else {
        return Vec::new();
    };
    // A banda é traçada CONTORNO A CONTORNO e unida como conjunto. Traçar todos de uma vez
    // seria mais rápido e estaria errado: num compound os contornos correm em sentidos
    // opostos (é assim que um buraco é um buraco), e as bandas deles se cancelariam onde se
    // sobrepõem — exatamente na parede fina entre a borda e o furo, que é onde um offset
    // grande é interessante.
    let pen = Stroke::new(2.0 * d.abs())
        .with_join(join_of(join))
        .with_caps(Cap::Butt);
    let mut band: Option<Region> = None;
    for contour in region
        .bez()
        .elements()
        .split_inclusive(|e| matches!(e, kurbo::PathEl::ClosePath))
    {
        let one: BezPath = contour.iter().copied().collect();
        let Some(b) = penned(&one, &pen).filter(|r| !r.is_empty()) else {
            continue;
        };
        band = Some(match band {
            None => b,
            Some(a) => match a.combine(&b, BinaryOp::Union) {
                Some(u) => u,
                None => return Vec::new(),
            },
        });
    }
    let Some(band) = band else {
        return Vec::new();
    };
    let op = if d > 0.0 {
        BinaryOp::Union
    } else {
        BinaryOp::Difference
    };
    match region.combine(&band, op) {
        Some(r) => r.into_paths(path),
        None => Vec::new(),
    }
}

#[cfg(test)]
#[path = "expand_tests.rs"]
mod tests;
