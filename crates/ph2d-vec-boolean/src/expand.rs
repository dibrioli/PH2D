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

use kurbo::{BezPath, Cap, Join, PathEl, Point, Shape, Stroke, StrokeOpts};
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{
    FillRule, LineCap, LineJoin, OffsetSide, Paint, StrokePiece, StrokeSpec, VecPath, stroke_plan,
};

use crate::{Closing, binary_grouped, compound_from, flatten_groups, to_bez_with};

/// **A fita do Power Stroke** — módulo irmão (teto de LOC): o que MOLDA o traço de largura
/// variável. O sweep e a limpeza de lascas ficam aqui, compartilhados com o Offset.
#[path = "expand_ribbon.rs"]
mod ribbon;
pub use ribbon::power_stroke;

/// **O offset DIRETO** — módulo irmão (teto de LOC): o caminho barato do Contour VIVO
/// ([`ring::offset_ring`]), a borda do traço sem tocar o `linesweeper`.
#[path = "expand_ring.rs"]
mod ring;
pub use ring::offset_ring;

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
///
/// # ⚠️ O sweep PODE entrar em pânico, e o `catch_unwind` é o que torna esta doc verdadeira
///
/// *"devolve vazio se o sweep falhar"* era uma promessa só sobre o `None` do `Region::of`. O
/// `linesweeper` 0.3.0 tem um `unwrap()` numa curva degenerada (`curve/mod.rs:302`) e **aborta o
/// processo** em vez de devolver `None` — MEDIDO com uma varredura de 200 distâncias por forma:
///
/// | forma | Miter | Round | Bevel |
/// |---|---|---|---|
/// | retângulo | 0/200 | 0/200 | 0/200 |
/// | hexágono | 8/200 | 26/200 | 26/200 |
/// | estrela 5 pontas | 0/200 | 83/200 | **118/200** |
///
/// Numa estrela com quina Bevel, **59% das distâncias derrubam o app** — e o slider de Offset da
/// seção Expand as alcança arrastando. É defeito PRÉ-EXISTENTE (nada aqui o introduziu); o
/// Contour só o torna certo, porque N anéis varrem N distâncias de uma vez.
///
/// A cura no nosso lado é a mesma do `ph2d-imageio-avif` com um decoder hostil: **isolar o pânico
/// na fronteira** e devolver o vazio que a função já promete. `AssertUnwindSafe` é honesto aqui —
/// a entrada é emprestada e só lida, e o que sai é construído dentro do bloco, então não há
/// invariante meio-escrito a vazar. A cura de VERDADE é do `linesweeper`, e não é desta linha.
#[must_use]
pub fn offset_path(path: &VecPath, d: f64, join: LineJoin, side: OffsetSide) -> Vec<VecPath> {
    if d.abs() < MIN_OFFSET || !d.is_finite() {
        return Vec::new();
    }
    // Conta cada entrada no sweep — o gate do Contour prova que o preview VIVO não o chama
    // (o offset direto cobre o caso comum), que é a raiz do FPS e do pânico. Padrão do ADR-0120
    // ("o gate que conta quantas vezes o caminho CARO dispara").
    SWEEP_CALLS.with(|c| c.set(c.get() + 1));
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        offset_path_inner(path, d, join, side)
    }))
    .unwrap_or_default()
}

thread_local! {
    /// Quantas vezes [`offset_path`] (o sweep booleano, caro e capaz de panicar) foi chamado NESTA
    /// thread. ⚠️ **Thread-local, não global**: o cozimento do Contour é síncrono na thread do
    /// teste, e o gate de FPS conta o próprio arrasto — um `AtomicU64` global era POLUÍDO pelos
    /// gates de shrink (Inner/negativo), que chamam a booleana, rodando em paralelo. Thread-local
    /// isola cada teste sem precisar de execução serial.
    static SWEEP_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// O contador de entradas no sweep booleano NESTA thread — para o gate de perf do Contour
/// (ADR-0120). Ver o `thread_local!` de [`SWEEP_CALLS`].
#[doc(hidden)]
#[must_use]
pub fn __sweep_calls() -> u64 {
    SWEEP_CALLS.with(std::cell::Cell::get)
}

fn offset_path_inner(path: &VecPath, d: f64, join: LineJoin, side: OffsetSide) -> Vec<VecPath> {
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
        .and_then(|r| drop_phantoms(r, loop_bez, d))
}

/// Remove os contornos **FANTASMAS** — o refugo do stroke com caneta maior que o laço.
///
/// Com `2|d|` rivalizando o tamanho do próprio laço, o contorno interno da banda
/// ([`penned_outline`]) degenera num laço auto-cruzado cujo winding vira ruído, e o refugo
/// atravessa o sweep: **encolher** (`base ∖ banda`) devolvia uma ILHA no meio de uma forma
/// que devia ANIQUILAR, e **crescer** (`base ∪ banda`) ganhava um FURO fantasma no miolo.
/// Medido no donut do smoke (2026-07-20): `d=−4` com Round/Bevel devolvia 12 verts de área
/// 2,52 onde o Miter (corretamente) devolvia nada — a forma *crescia de volta* conforme o
/// artista encolhia mais; e `d=+4` inflava a área de 19,8 (a resposta exata, que o Miter dá)
/// para 30,7. É o mecanismo do report *"muda para round mas não muda para Miter e Bevel"*:
/// no `d` extremo cada join produzia um refugo DIFERENTE (nada · fantasma · o MESMO
/// fantasma), então uns cliques "mudavam" e outros não.
///
/// O discriminador é **DISTÂNCIA ao laço fonte**: toda borda legítima de um offset contém
/// pontos a pelo menos `|d|` do laço (aresta offsetada = exatamente `|d|`; bico de miter =
/// além). O teste é o **MÁXIMO** do contorno, nunca o mínimo — o chord de um bevel numa
/// quina aguda AFUNDA abaixo de `|d|` no meio, mas as pontas dele estão a `|d|`, então o
/// máximo o absolve. Um fantasma vive no MIOLO: todo ponto dele fica aquém de `|d|`.
///
/// Caminho comum = **zero custo**: nada descartado devolve a região INTACTA (sem re-sweep);
/// o re-sweep só roda no caso degenerado que hoje devolve geometria errada.
fn drop_phantoms(region: Region, loop_bez: &BezPath, d: f64) -> Option<Region> {
    let segs = flat_segments(loop_bez);
    let floor = d.abs() * 0.99;
    let is_real = |contour: &BezPath| -> bool {
        let mut max_d2 = 0.0_f64;
        for el in flat_lines(contour).elements() {
            if let Some(p) = end_of(el) {
                max_d2 = max_d2.max(dist2_to_segs(p, &segs));
                if max_d2 >= floor * floor {
                    return true;
                }
            }
        }
        false
    };
    let mut kept = BezPath::new();
    let mut dropped = false;
    for group in &region.groups {
        let Some((outer, holes)) = group.split_first() else {
            continue;
        };
        if !is_real(outer) {
            dropped = true; // o grupo inteiro é refugo — os furos dele morrem junto
            continue;
        }
        kept.extend(outer.iter());
        for hole in holes {
            if is_real(hole) {
                kept.extend(hole.iter());
            } else {
                dropped = true;
            }
        }
    }
    if !dropped {
        return Some(region);
    }
    // Contornos sobreviventes já saíram orientados do sweep — NonZero os recompõe.
    Region::of(&kept, LsFillRule::NonZero)
}

/// O laço achatado como segmentos de reta — a régua do [`drop_phantoms`].
fn flat_segments(bez: &BezPath) -> Vec<kurbo::Line> {
    let flat = flat_lines(bez);
    let mut segs = Vec::new();
    let (mut start, mut prev) = (Point::ZERO, Point::ZERO);
    for el in flat.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                start = p;
                prev = p;
            }
            PathEl::LineTo(p) => {
                segs.push(kurbo::Line::new(prev, p));
                prev = p;
            }
            PathEl::ClosePath => {
                segs.push(kurbo::Line::new(prev, start));
                prev = start;
            }
            // `flat_lines` só emite MoveTo/LineTo/ClosePath.
            _ => {}
        }
    }
    segs
}

/// Distância AO QUADRADO de `p` ao segmento mais próximo (quadrado: o comparador do
/// [`drop_phantoms`] só precisa ordenar, e a raiz por-amostra seria transcendental à toa).
fn dist2_to_segs(p: Point, segs: &[kurbo::Line]) -> f64 {
    let mut best = f64::INFINITY;
    for s in segs {
        let v = s.p1 - s.p0;
        let len2 = v.hypot2();
        let t = if len2 <= 0.0 {
            0.0
        } else {
            ((p - s.p0).dot(v) / len2).clamp(0.0, 1.0)
        };
        let q = s.p0 + v * t;
        best = best.min((p - q).hypot2());
    }
    best
}

#[cfg(test)]
#[path = "expand_tests.rs"]
mod tests;
