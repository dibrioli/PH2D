//! **O CORTE por uma LINHA** — uma forma fechada cortada dá formas **FECHADAS** (plano 25 §7).
//!
//! Esta é a lei do produto, e ela substituiu a anterior: as peças ficavam **abertas** (a escolha
//! do Affinity), e o veredito do Enio em 2026-07-31 foi *"o corte de formas fechadas está
//! produzindo formas abertas. Isso não pode acontecer."*
//!
//! # Por que isto mora na crate da booleana, e não na do editor
//!
//! Porque **não há geometria nova aqui**. A pesquisa: o Inkscape tem as duas leis como comandos
//! SEPARADOS — `Division` (Ctrl+/) dá peças fechadas, `Cut Path` (Ctrl+Alt+/) dá peças abertas
//! sem preenchimento — e a de peças fechadas é feita sobre o motor de arranjo (livarot), não
//! sobre um "abrir o contorno". O Illustrator faz o mesmo na faca e no *Divide Objects Below*.
//!
//! Nós já temos o motor exato (`linesweeper`), e ele **exige contornos fechados**
//! (`Topology::from_paths` devolve `Result<_, NonClosedPath>` — medido). Logo o trabalho inteiro
//! desta função é UM: transformar a linha aberta que o artista desenhou num **cortador FECHADO**
//! cuja fronteira, DENTRO da forma, é exatamente a linha. Feito isso, quem calcula as
//! interseções curva-curva é o motor, com a robustez que ele já tem.
//!
//! # As peças são as componentes dos DOIS lados
//!
//! `S ∩ H` e `S − H` particionam `S` para **qualquer** `H` cuja fronteira dentro de `S` seja a
//! linha — e é por isso que este desenho não se importa com o que acontece com o `H` longe da
//! forma. De que lado cada peça caiu não é informação que alguém use: o que se quer é o
//! CONJUNTO das peças, e a união das componentes dos dois lados É esse conjunto. Um `H` que
//! ficasse com winding estranho lá fora trocaria peças de lado sem mudar a resposta.
//!
//! # Os dois casos em que a forma NÃO é cortada, e nenhum deles é limitação
//!
//! 1. **A linha não atravessa** (uma ponta dela cai dentro da forma). Uma região menos uma
//!    fenda continua CONEXA — não há duas peças. Isto é topologia, não uma pendência.
//! 2. **A extensão da linha cruzaria a forma** (a ponta ficou presa atrás de um braço côncavo).
//!    Aí a fronteira de `H` dentro de `S` teria um pedaço que ninguém desenhou, e o corte sairia
//!    por uma linha inventada. **Recusar em voz alta é melhor que cortar errado.**

use kurbo::Shape;
use ph2d_vec_scene::{VecPath, VecVertex, blade_crossings, contains_point};

/// Quantos segmentos aproximam o arco de fecho, lá fora. É desenho de FECHO, não de arte: ele
/// nunca entra na forma, então a única exigência é ser um polígono fechado sem auto-interseção.
const CLOSURE_ARC_STEPS: usize = 24;

/// Por que a forma não foi cortada — o que o chamador diz ao artista.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CutRefusal {
    /// A linha não chega a esta forma (nem a toca, ou toca sem a atravessar).
    Missed,
    /// Uma ponta da linha ficou DENTRO da forma: um corte que não atravessa não divide nada.
    DoesNotCrossThrough,
    /// A extensão da linha atravessaria a forma — cortar aqui seria cortar por uma linha que
    /// ninguém desenhou.
    Trapped,
    /// O motor recusou a entrada (degenerada: sem área, sem pontos, coordenada não-finita).
    Degenerate,
}

/// **A PORTA ÚNICA do corte** — corta `source` com `line`, tudo em MUNDO.
///
/// Ela decide pela **topologia da fonte**, e a decisão não pode viver no chamador: uma forma
/// fechada e uma fita aberta têm respostas diferentes para *"o que sobra depois do corte?"*, e
/// duas portas para essa pergunta divergiriam no dia em que uma das leis mudasse.
///
/// - **fechada** ⇒ peças **FECHADAS** ([`cut_closed`]), a lei do produto;
/// - **aberta** ⇒ peças **abertas** ([`cut_open`]), que é a única resposta possível: uma fita não
///   tem interior, então não há região a dividir — o que o corte faz é PARTI-LA.
///
/// # Errors
/// Ver [`CutRefusal`].
pub fn cut_with_line(source: &VecPath, line: &VecPath) -> Result<Vec<VecPath>, CutRefusal> {
    if source.closed {
        cut_closed(source, line)
    } else {
        cut_open(source, line)
    }
}

/// **Corta a fita ABERTA `source` com a linha `line`** — tudo em MUNDO. Devolve as peças, todas
/// abertas, na ordem em que percorrem a fita original.
///
/// # Por que aqui não entra o motor de arranjo
///
/// Uma fita não tem interior: não há região a dividir, e o `linesweeper` — que responde por
/// REGIÕES — não tem o que dizer sobre ela. O que um corte faz a uma fita é **parti-la nos
/// cruzamentos**, que é aritmética de curva e não de conjunto.
///
/// ⚠️ **Os cruzamentos saem da linha ACHATADA contra a curva EXATA da fonte.** A aproximação fica
/// toda de um lado — o da lâmina —, e o lado que decide *onde* o vértice novo nasce é exato
/// ([`blade_crossings`] refina por bisseção sobre a cúbica de verdade). Achatar os dois lados
/// poria o corte a meio texel de onde o artista o vê.
///
/// # Errors
/// Ver [`CutRefusal`]. Uma fonte fechada, compound ou degenerada é recusada aqui — a fechada tem
/// porta própria.
pub fn cut_open(source: &VecPath, line: &VecPath) -> Result<Vec<VecPath>, CutRefusal> {
    if source.closed || source.is_compound() || source.verts.len() < 2 {
        return Err(CutRefusal::Degenerate);
    }
    let crossings = crossings_along(source, line);
    if crossings.is_empty() {
        return Err(CutRefusal::Missed);
    }
    // ⚠️ **De trás para a frente, E com o `t` RE-PARAMETRIZADO.** Duas correções, não uma:
    //
    // 1. cada corte INSERE um vértice, então um cruzamento mais adiante deslocaria os índices dos
    //    anteriores — descendo, os que faltam continuam válidos;
    // 2. **o `t` é relativo ao SEGMENTO, e cortar encolhe o segmento.** Depois de partir em `t`, o
    //    segmento que fica na cabeça cobre `[0, t]` do original: um cruzamento seguinte em `t'`
    //    (menor) vive agora em `t' / t`. Sem isto, dois cortes na MESMA curva pousam no lugar
    //    errado — e nada denuncia: sai o número certo de peças, em ordem, com as fronteiras
    //    deslocadas. Foi assim que a mutação "cortar de frente para trás" sobreviveu ao 1º gate.
    let mut head = source.clone();
    let mut out: Vec<VecPath> = Vec::new();
    let mut prev_seg = usize::MAX;
    let mut upper = 1.0_f64;
    for &(seg, t) in crossings.iter().rev() {
        if seg != prev_seg {
            prev_seg = seg;
            upper = 1.0;
        }
        let local_t = if upper > f64::EPSILON { t / upper } else { t };
        upper = t;
        let Some(idx) = ph2d_vec_scene::split_segment(&mut head, seg, local_t) else {
            continue;
        };
        // ⚠️ Nenhum guard de ponta aqui, e é medido: o `split_segment` **INSERE** um vértice, então
        // o índice é sempre interior (`1 ..= len-2`). Um `if idx == 0` seria código morto que
        // nunca dispara — a mutação que o remove não sangra nenhum gate, porque não há como.
        let tail = head.verts[idx..].to_vec();
        head.verts.truncate(idx + 1);
        out.push(VecPath {
            verts: tail,
            closed: false,
            stroke: source.stroke,
            fill: source.fill.clone(),
            effects: source.effects.clone(),
            ..VecPath::default()
        });
    }
    if out.is_empty() {
        return Err(CutRefusal::Missed);
    }
    out.push(head);
    out.reverse();
    Ok(out)
}

/// Onde a lâmina cruza a fonte, em `(segmento PLANO, t)`, crescente e sem repetições.
///
/// A lâmina é achatada em segmentos retos e cada um deles pergunta à [`blade_crossings`], que é
/// exata do lado da fonte. A tolerância do achatamento é **relativa à fonte** — uma constante
/// absoluta seria fina demais numa forma minúscula e grosseira numa enorme.
fn crossings_along(source: &VecPath, line: &VecPath) -> Vec<(usize, f64)> {
    let bb = crate::to_bez(source).bounding_box();
    let tol = (bb.width().hypot(bb.height()) * 1e-4).max(1e-9);
    let mut pts: Vec<[f64; 2]> = Vec::new();
    kurbo::flatten(
        crate::to_bez_with(line, crate::Closing::AsDrawn).iter(),
        tol,
        |el| match el {
            kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => pts.push([p.x, p.y]),
            _ => {}
        },
    );
    let mut out: Vec<(usize, f64)> = Vec::new();
    for w in pts.windows(2) {
        out.extend(blade_crossings(source, w[0], w[1]));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    out.dedup_by(|a, b| a.0 == b.0 && (a.1 - b.1).abs() < 1e-7);
    out
}

/// **Corta a forma FECHADA `source` com a linha `line`** — tudo em MUNDO. Devolve as peças, todas
/// fechadas, ou o motivo de não ter cortado.
///
/// As peças herdam o estilo da fonte (o `apply_many` doa o estilo do path do TOPO, que aqui é o
/// cortador — um objeto sem estilo nenhum; sem esta re-estampagem as peças sairiam transparentes).
///
/// # Errors
/// Ver [`CutRefusal`]. Nenhuma delas é erro de programação: são as respostas honestas para
/// geometrias em que o corte não existe.
pub fn cut_closed(source: &VecPath, line: &VecPath) -> Result<Vec<VecPath>, CutRefusal> {
    if !source.closed || source.verts.len() < 2 || line.verts.len() < 2 {
        return Err(CutRefusal::Degenerate);
    }
    let cutter = build_cutter(source, line)?;
    let mut pieces = crate::apply(source, &cutter, crate::BoolOp::Intersect);
    pieces.extend(crate::apply(source, &cutter, crate::BoolOp::Subtract));
    // ⚠️ **Lascas.** Toda booleana produz fatias de área ~0 nas bordas partilhadas (o resíduo de
    // tolerância que o `drop_slivers` do Shape Builder já nomeou). Aqui elas seriam peças
    // fantasma na Hierarquia; o piso é RELATIVO à fonte, como lá.
    //
    // ⚠️ **DEFESA EM CAMADA, e não observável nesta suíte** — está medido: a mutação que remove
    // este `retain` sobrevive aos 11 gates, incluindo o corte COLINEAR com uma aresta, que é o
    // caso canônico de lasca. Ela fica porque o motivo dela foi medido NOUTRO consumidor do mesmo
    // motor (o Shape Builder, resíduo de 0,30% da área contra arte a partir de 6,5%), e porque o
    // modo de falha é caro e mudo: uma peça de área nula tem contorno LONGO, então aparece como
    // uma LINHA solta — nem uma forma pequena que se veja, nem nada que um gate de contagem pegue.
    // Herdar a proteção sem herdar a prova é o preço, e ele está escrito aqui em vez de fingido.
    let src_area = crate::area(source);
    if src_area <= 0.0 {
        return Err(CutRefusal::Degenerate);
    }
    pieces.retain(|p| crate::area(p) > src_area * 0.005);
    if pieces.len() < 2 {
        return Err(CutRefusal::Missed);
    }
    for p in &mut pieces {
        p.fill.clone_from(&source.fill);
        p.stroke = source.stroke;
        p.effects.clone_from(&source.effects);
    }
    Ok(pieces)
}

/// A linha aberta vira um **cortador fechado**: linha + as duas extensões + um fecho roteado
/// **por fora** da caixa da forma.
///
/// Uma linha JÁ fechada é o próprio cortador — e isso não é caso especial, é o caso geral com
/// zero passos: cortar um disco com um círculo desenhado por cima é a mesma pergunta.
fn build_cutter(source: &VecPath, line: &VecPath) -> Result<VecPath, CutRefusal> {
    if line.closed {
        return Ok(line.clone());
    }
    let verts = &line.cooked().verts;
    let (Some(first), Some(last)) = (verts.first(), verts.last()) else {
        return Err(CutRefusal::Degenerate);
    };
    // (1) Um corte que não atravessa não divide nada.
    if contains_point(source, first.anchor) || contains_point(source, last.anchor) {
        return Err(CutRefusal::DoesNotCrossThrough);
    }

    let bb = crate::to_bez(source).bounding_box();
    if !bb.width().is_finite() || !bb.height().is_finite() {
        return Err(CutRefusal::Degenerate);
    }
    let centre = [bb.center().x, bb.center().y];
    let diag = bb.width().hypot(bb.height()).max(1e-9);
    // Longe o bastante para a extensão sair da caixa venha de onde vier, e o fecho ainda mais.
    let reach = diag * 4.0;
    let closure_r = diag * 8.0;

    // As direções de saída: a tangente de cada ponta, tomada contra o vizinho dela. O handle é
    // preferido ao vizinho porque é ele que descreve para onde a curva ia.
    let start_dir = away(
        first.anchor,
        first.out_handle,
        verts.get(1).map(|v| v.anchor),
    );
    let end_dir = away(
        last.anchor,
        last.in_handle,
        verts.get(verts.len().wrapping_sub(2)).map(|v| v.anchor),
    );
    let e_start = step(first.anchor, start_dir, reach);
    let e_end = step(last.anchor, end_dir, reach);

    // (2) A extensão não pode atravessar a forma.
    for (a, b) in [(first.anchor, e_start), (last.anchor, e_end)] {
        if !blade_crossings(source, a, b).is_empty() {
            return Err(CutRefusal::Trapped);
        }
    }

    let mut out: Vec<VecVertex> = verts.clone();
    out.push(VecVertex::corner(e_end));
    // Do fim, radialmente para FORA até o raio do fecho, o arco, e radialmente para DENTRO até a
    // extensão do começo. ⚠️ Um raio que sai de um ponto já fora de uma caixa CONVEXA, afastando-se
    // do centro dela, nunca mais a toca — é isso que garante que o fecho não corta nada.
    let p_out = on_circle(centre, e_end, closure_r);
    let p_in = on_circle(centre, e_start, closure_r);
    out.push(VecVertex::corner(p_out));
    for k in 1..CLOSURE_ARC_STEPS {
        let t = k as f64 / CLOSURE_ARC_STEPS as f64;
        out.push(VecVertex::corner(lerp_on_circle(
            centre, p_out, p_in, closure_r, t,
        )));
    }
    out.push(VecVertex::corner(p_in));
    out.push(VecVertex::corner(e_start));

    Ok(VecPath {
        verts: out,
        closed: true,
        ..VecPath::default()
    })
}

/// A direção de SAÍDA por uma ponta: do handle, se ele diz alguma coisa; senão do vizinho.
/// Normalizada; `[1,0]` num caso degenerado (uma ponta sem direção nenhuma é um ponto).
fn away(anchor: [f64; 2], handle: [f64; 2], neighbour: Option<[f64; 2]>) -> [f64; 2] {
    for cand in [handle, neighbour.unwrap_or(anchor)] {
        let d = [anchor[0] - cand[0], anchor[1] - cand[1]];
        let len = d[0].hypot(d[1]);
        if len > 1e-12 {
            return [d[0] / len, d[1] / len];
        }
    }
    [1.0, 0.0]
}

fn step(p: [f64; 2], dir: [f64; 2], d: f64) -> [f64; 2] {
    [p[0] + dir[0] * d, p[1] + dir[1] * d]
}

/// O ponto na circunferência de raio `r` em torno de `centre`, na direção de `p`.
fn on_circle(centre: [f64; 2], p: [f64; 2], r: f64) -> [f64; 2] {
    let d = [p[0] - centre[0], p[1] - centre[1]];
    let len = d[0].hypot(d[1]).max(1e-12);
    [centre[0] + d[0] / len * r, centre[1] + d[1] / len * r]
}

/// Um passo do arco de fecho: interpola as duas direções e re-projeta no raio. Sem `atan2` (o
/// caminho curto sai da própria interpolação linear das direções).
fn lerp_on_circle(centre: [f64; 2], a: [f64; 2], b: [f64; 2], r: f64, t: f64) -> [f64; 2] {
    let mid = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    // Antipodais: a interpolação passa pelo centro e a direção some. Desvia pela perpendicular,
    // que dá exactamente a meia-volta pretendida.
    let d = [mid[0] - centre[0], mid[1] - centre[1]];
    if d[0].hypot(d[1]) <= 1e-9 {
        let perp = [-(b[1] - a[1]), b[0] - a[0]];
        return on_circle(centre, [centre[0] + perp[0], centre[1] + perp[1]], r);
    }
    on_circle(centre, mid, r)
}

#[cfg(test)]
#[path = "cut_tests.rs"]
mod tests;
