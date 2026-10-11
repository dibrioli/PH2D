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

/// **A banda por PEÇAS** do Offset Bevel/Miter — módulo irmão (teto de LOC).
#[path = "expand_band.rs"]
mod band;

/// **A tangente honesta nas quinas** que a banda Bevel/Miter mede — módulo irmão (teto de LOC).
#[path = "expand_corner.rs"]
mod corner;

/// **O caçador de fantasmas** do Offset Round — módulo irmão (teto de LOC).
#[path = "expand_phantom.rs"]
mod phantom;

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
    /// A união de um conjunto com o **VAZIO** é ele mesmo — e é essa identidade que serve de porta
    /// de entrada: o sweep resolve as auto-interseções (que o traço de um caminho que se cruza
    /// sempre tem) e orienta o que sai.
    ///
    /// # ⛔ O 2.º operando é o VAZIO, e não `bez` outra vez
    ///
    /// A identidade `A ∪ A = A` também serve, e era o que estava escrito aqui. Mas ela põe **cada
    /// aresta na varredura com multiplicidade 2** — e a partir do `linesweeper` 0.4 uma
    /// multiplicidade **par** faz o motor deixar de dissolver a aresta INTERNA que dois quadriláteros
    /// vizinhos da fita partilham. O *power stroke* saía com **um contorno por quad**: 128, 60, 32,
    /// 175 peças soltas onde o produto quer **uma** fita.
    ///
    /// ⚠️ **É paridade, e foi medida como paridade:** `k` cópias coincidentes da mesma geometria dão
    /// **1** peça para `k` ímpar e **uma peça por face** para `k` par. A mesma região desenhada como
    /// um hexágono só (sem aresta interna) dá 1 para todo `k`. E é **livre de escala** — com a
    /// obliquidade varrida por **12 ordens de grandeza** (`0,5` a `1e-12`) a auto-união reprova nas
    /// nove e `A ∪ ∅` acerta em todas. ⇒ **não é tolerância**, é a regra de multiplicidade.
    ///
    /// ⚠️ Sob a 0.3 as duas formas são **byte-idênticas**, e é por isso que esta linha e o pin da
    /// versão **têm de viajar juntos**: sozinha, ela é um no-op que nenhuma mutação mata.
    ///
    /// ⛔ O que **não** era a causa (cada um medido, e cada um a hipótese óbvia): o modo de saída
    /// aproximado que a 0.4 tornou padrão (`contours_correct` dá os mesmos 128) · a inversão do
    /// sentido de winding (emitir os quads na convenção nova dá 64 grupos, ainda errado) · a
    /// `FillRule` · o `BinaryOp` · a tolerância · a nossa porta (o defeito reproduz-se no
    /// `binary_op` cru, sem uma linha nossa) · e a `Region::combine`, que compõe **dois conjuntos
    /// diferentes** e por isso nunca teve o problema.
    fn of(bez: &BezPath, rule: LsFillRule) -> Option<Self> {
        let groups = binary_grouped(bez, &BezPath::new(), rule, BinaryOp::Union)?;
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
    ///
    /// ⚠️ **Furo de área ~nula não é furo, é lasca, e sai aqui.** O `linesweeper` 0.5 devolve-as
    /// como FUROS dentro do grupo (medido: a senoide de `the_ink_is_one_connected_piece_with_no_loops`
    /// saía com dois furos de área `0,0` na fita de `8,0`; a 0.4 não os emitia), e o
    /// [`drop_slivers`] só olha a peça inteira. O piso é o mesmo dele, relativo ao total.
    fn into_paths(self, style: &VecPath) -> Vec<VecPath> {
        let total: f64 = self
            .groups
            .iter()
            .filter_map(|g| g.first())
            .map(|c| c.area().abs())
            .sum();
        self.groups
            .iter()
            .filter_map(|g| {
                let (outer, holes) = g.split_first()?;
                let mut kept = vec![outer.clone()];
                kept.extend(
                    holes
                        .iter()
                        .filter(|h| h.area().abs() > total * SLIVER_REL)
                        .cloned(),
                );
                compound_from(&kept, style)
            })
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
///
/// ⚠️ **O tracejado é AJUSTADO ao caminho que vai ser traçado** (`ph2d_vec_scene::dash_fit`) —
/// a MESMA porta que o renderer usa. Se um ajustasse e o outro não, a forma assada sairia com
/// o tracejado noutra cadência que a desenhada; e sem ajuste nenhum a emenda do contorno
/// fechado fica à vista (Enio, 2026-08-22).
pub(crate) fn line_pen(line: &VecPath, s: &StrokeSpec) -> Stroke {
    let pen = Stroke::new(s.width)
        .with_caps(match s.cap {
            LineCap::Butt => Cap::Butt,
            LineCap::Round => Cap::Round,
            LineCap::Square => Cap::Square,
        })
        .with_join(join_of(s.join))
        // ⚠️ O MESMO limite do traço desenhado — ver [`ph2d_vec_scene::MITER_LIMIT`].
        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT);
    // Os comprimentos vêm do `StrokeSpec` — a MESMA porta que o renderer usa, senão o
    // tracejado assado sairia noutra cadência que o desenhado.
    // ⚠️ A peça chega da FONTE (o `stroke_plan` recebe o objeto e encurta a linha para a ponta
    // caber), então é a porta que COZE e mede — a mesma conta que o `tess.dash` do renderer
    // cacheia sobre o cozido. Uma só lei (`dash_fit`), duas portas, e a diferença é quem cozeu.
    match ph2d_vec_scene::dash_for(line, s) {
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
    let mut out = weld_seams(&kurbo::stroke(bez, pen, &StrokeOpts::default(), tol), tol);
    append_miter_clips(&mut out, bez, pen);
    out
}

/// ⭐ O bico CORTADO (F12 A, a lei em [`ph2d_vec_scene::miter_clip`]) — o mesmo que o renderer
/// desenha (`ph2d_vec_render::miter_clip`): cada junta `Miter` além do limite ganha o quadrilátero
/// do corte, no sentido do corpo.
fn append_miter_clips(out: &mut BezPath, bez: &BezPath, pen: &Stroke) {
    use ph2d_vec_scene::miter_clip::{El, clip_patches, oriented, subpaths};
    if !matches!(pen.join, Join::Miter) || pen.width <= 0.0 {
        return;
    }
    let el = |e: kurbo::PathEl| match e {
        kurbo::PathEl::MoveTo(p) => El::Move([p.x, p.y]),
        kurbo::PathEl::LineTo(p) => El::Line([p.x, p.y]),
        kurbo::PathEl::QuadTo(a, p) => El::Quad([a.x, a.y], [p.x, p.y]),
        kurbo::PathEl::CurveTo(a, b, p) => El::Cubic([a.x, a.y], [b.x, b.y], [p.x, p.y]),
        kurbo::PathEl::ClosePath => El::Close,
    };
    let subs = if pen.dash_pattern.is_empty() {
        subpaths(bez.iter().map(el))
    } else {
        subpaths(kurbo::dash(bez.iter(), pen.dash_offset, &pen.dash_pattern).map(el))
    };
    let cuts = clip_patches(&subs, pen.width * 0.5, pen.miter_limit);
    let quads = oriented(&cuts, |p| {
        kurbo::Shape::winding(&*out, kurbo::Point::new(p[0], p[1]))
    });
    for q in quads {
        out.move_to((q[0][0], q[0][1]));
        for p in &q[1..] {
            out.line_to((p[0], p[1]));
        }
        out.close_path();
    }
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
    let Some(s) = path.stroke.as_ref() else {
        return Vec::new();
    };
    let mut acc: Option<Region> = None;
    for piece in stroke_plan(path, s) {
        let region = match piece {
            StrokePiece::Line { path: p } => {
                penned(&to_bez_with(&p, Closing::AsDrawn), &line_pen(&p, s))
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
        Some(acc) => acc.into_paths(&ink_style(s)),
        None => Vec::new(),
    }
}

/// **A SILHUETA de um path** — a região que ele de facto cobre na tela.
///
/// Sem traço é o preenchimento e mais nada; **com** traço é `preenchimento ∪ contorno-do-traço`,
/// porque é isso que o olho vê e é isso que um campo de distância tem de medir.
///
/// ⚠️ **Por que a união é obrigatória, e não bastam os dois contornos soltos.** Um traço centrado
/// cobre `[−w/2, +w/2]` em volta do caminho, então o caminho do PREENCHIMENTO fica no MEIO da
/// faixa de tinta — dentro da região. Alimentar um campo de distância com ele poria uma fronteira
/// no meio da forma, e todo consumidor (bevel, feather, contorno) desenharia uma segunda borda
/// ali. A união apaga as fronteiras internas; nada mais apaga.
///
/// ⚠️ **Isto NÃO é o `outline_stroke`**, que é um comando de EDIÇÃO e de propósito não traz o
/// preenchimento junto (traço e miolo têm cores diferentes porque são coisas diferentes). Aqui a
/// pergunta é outra — *que região esta forma ocupa?* — e a resposta é uma só.
///
/// Devolve **vazio** quando não sabe responder (traço degenerado, sweep que não fecha); o
/// chamador cai no caminho que tinha antes.
#[must_use]
pub fn silhouette_paths(path: &VecPath) -> Vec<VecPath> {
    // Sem traço a silhueta É o path — devolvê-lo evita um sweep que não decide nada.
    //
    // ⚠️ **Largura zero conta como "sem traço"** (`StrokeSpec::lays_a_band`, a porta única): é o que
    // o slider promete e o que o desenho faz. Perguntar `stroke.is_none()` levava a largura zero ao
    // `outline_stroke`, que devolve tinta VAZIA, e daí a `Vec::new()` — que o chamador lê como *"não
    // sei responder"* e converte no caminho do raster. O pente do bevel voltava numa forma sem
    // contorno nenhum (BUGS #24, 2ª rodada).
    if !path.stroke.as_ref().is_some_and(StrokeSpec::lays_a_band) {
        return vec![path.clone()];
    }
    let ink = outline_stroke(path);
    if ink.is_empty() {
        return Vec::new();
    }
    if path.fill.is_none() {
        // Só traço (uma linha, um contorno aberto): a tinta É a silhueta.
        return ink;
    }
    let mut all: Vec<&VecPath> = Vec::with_capacity(ink.len() + 1);
    all.push(path);
    all.extend(ink.iter());
    crate::apply_many(&all, crate::BoolOp::Union)
}

/// O estilo de uma forma que **É** o traço: preenchida com a cor dele, e sem traço.
///
/// Deixar o traço no resultado o desenharia uma segunda vez, agora em volta de si mesmo,
/// engordando o desenho no clique que não devia mudar nada. Porta única porque os dois
/// comandos que assam tinta ([`outline_stroke`] e [`power_stroke`]) têm de responder igual.
fn ink_style(s: &StrokeSpec) -> VecPath {
    VecPath {
        fill: Some(Paint::Solid(s.color())),
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
pub(crate) fn drop_slivers(paths: Vec<VecPath>) -> Vec<VecPath> {
    let total: f64 = paths.iter().map(crate::area).sum();
    if total <= MIN_TOL {
        return Vec::new();
    }
    paths
        .into_iter()
        .filter(|p| crate::area(p) > total * SLIVER_REL)
        .collect()
}

/// O piso de área de uma lasca, relativo à área total do resultado — ver [`drop_slivers`].
const SLIVER_REL: f64 = 1e-4;

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
/// # O `catch_unwind` é um cinto CONTADO, não um silêncio
///
/// O `linesweeper` 0.3/0.4 tinha `unwrap()`s em curvas degeneradas (`curve/mod.rs:302` na 0.3,
/// `:364` na 0.4) e **abortava o processo**: medido na 0.3 com 200 distâncias por forma, a
/// estrela de 5 pontas panicava em 83/200 (Round) e 118/200 (Bevel). A 0.5 (prompt 49, W1) zera
/// a conta — **0 pânicos** nas 2 700 combinações de `the_offset_never_takes_the_app_down` e nas
/// 600 de `the_offset_sweep_matches_the_table`, que contam cada pânico por um hook de
/// `std::panic` e reprovam se o número subir. O cinto fica (a porta única de
/// [`crate::engine`] tem o seu; este cobre também o stroker da kurbo): uma regressão do motor
/// volta a ser um vazio, nunca um app derrubado — mas um vazio que um gate VÊ.
/// `AssertUnwindSafe` é honesto: a entrada é emprestada e só lida, e a saída nasce no bloco.
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
    // Bevel/Miter medem a junção pelas retas vizinhas: junto às quinas elas seguem a tangente
    // com erro `≤ tol/|d|`, ou a junção erraria `|d|·θ` (ver `expand_corner.rs`).
    let honest = (offset && join != LineJoin::Round)
        .then(|| corner::split_at_corners(loop_bez, tolerance(loop_bez) / d.abs()));
    let base = Region::of(
        &flat_lines(honest.as_ref().unwrap_or(loop_bez)),
        LsFillRule::NonZero,
    )?;
    if !offset {
        return Some(base);
    }
    let pen = Stroke::new(2.0 * d.abs())
        .with_join(join_of(join))
        .with_caps(Cap::Butt);
    // ⚠️ Bevel/Miter: a banda é a UNIÃO das peças (ver `expand_band.rs` — o contorno do stroker
    // cancelava parte dela sob NonZero e a forma sumia sob o slider). Round: o stroker, que lá bate.
    let round = join == LineJoin::Round;
    let raw = if round {
        flat_lines(&penned_outline(&base.bez(), &pen))
    } else {
        band::pieces(&flat_lines(&base.bez()), d, join, pen.miter_limit)
    };
    let band = Region::of(&raw, LsFillRule::NonZero)?;
    if band.is_empty() {
        return Some(base);
    }
    let op = if d > 0.0 {
        BinaryOp::Union
    } else {
        BinaryOp::Difference
    };
    let r = base.combine(&band, op)?;
    if round {
        phantom::drop_phantoms(r, loop_bez, d)
    } else {
        Some(r)
    }
}

#[cfg(test)]
#[path = "expand_tests.rs"]
mod tests;
