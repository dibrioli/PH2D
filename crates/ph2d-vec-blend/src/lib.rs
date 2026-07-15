#![forbid(unsafe_code)]
//! ph2d-vec-blend — **interpolação de formas** (Morph / Blend).
//!
//! O problema difícil **não é interpolar**: é a **correspondência**. Dadas duas formas com
//! topologias diferentes, *qual ponto de A vira qual ponto de B?* A pesquisa
//! (`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md` §1.3) foi categórica: **ninguém
//! resolveu isso.**
//!
//! | Quem | Como resolve |
//! |---|---|
//! | **flubber** (a lib mais usada da web) | reamostra por arco, acolchoa, e faz **força bruta** sobre os deslocamentos. O autor: *"uma heurística cuja justificativa é que geralmente funciona bem"* |
//! | **GSAP MorphSVG** | heurísticas + **`shapeIndex` MANUAL** + uma ferramenta de debug cuja existência **admite que o automático erra** |
//! | **CorelDRAW Blend** | *"Map Nodes"*: o usuário **clica um nó em cada forma** |
//! | **Lottie / AE** · **Rive** | **nenhuma** — lerp por índice, exige a mesma contagem de pontos |
//!
//! O alvo aqui é o **melhor automático possível** — não há escape manual (ver a nota no corpo).
//!
//! # As duas decisões que separam este motor do flubber
//!
//! 1. **O corte é na UNIÃO das posições de âncora das duas formas** (em comprimento de arco
//!    normalizado), não numa reamostragem uniforme. Reamostrar uniformemente **arredonda as
//!    quinas** de quem tem quina — o quadrado vira um saco. Cortando na união, **as duas** formas
//!    mantêm cada uma das suas quinas, e ainda assim saem com a mesma contagem de peças, pareadas
//!    1-a-1. O preço é `n_a + n_b` âncoras na interpolada, e é um preço justo.
//! 2. **Cada peça é uma CÚBICA, não um segmento de reta.** Como a união inclui as âncoras
//!    originais, nenhuma peça atravessa uma âncora — então ela é sempre uma sub-cúbica exata de
//!    um segmento original (`subsegment`). A curva não é achatada em polilinha em lugar nenhum:
//!    o que sai é geometria de Bézier de verdade, editável.
//!
//! # O que ele NÃO faz (e a literatura sabe)
//!
//! É o **lerp de coordenadas**. Ele encolhe a forma no meio do caminho e pode auto-intersectar
//! numa rotação grande — é por isso que o GSAP tem um modo "rotational". O estado da arte é
//! Sederberg & Greenwood 1992 (deformação de trabalho mínimo) e Alexa 2000 (*as-rigid-as-
//! possible*). Ficam para depois, atrás do motor: a correspondência é o pré-requisito dos dois.

use kurbo::{CubicBez, ParamCurve, ParamCurveArclen, Point};
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Precisão do comprimento de arco.
///
/// Ela decide **onde** a peça é cortada — não a geometria dela (subdividir uma cúbica é exato,
/// de Casteljau). Ainda assim é apertada, e por um motivo medido: a 1e-6, `t=0` devolvia A com
/// um desvio de **2,2e-6** — pequeno, mas é **viés sistemático**, não ruído, e um viés desses
/// vira "a forma tremeu" quando o `t` é animado quadro a quadro.
const ARCLEN_EPS: f64 = 1e-11;

/// Duas posições de arco mais próximas que isto são a MESMA — cortar entre elas produziria uma
/// peça de comprimento zero (e um vértice duplicado na saída).
const MERGE_EPS: f64 = 1e-9;

/// Quantas amostras a busca de correspondência usa para medir o custo de um candidato.
const COST_SAMPLES: usize = 64;

// A correspondência é 100% AUTOMÁTICA — não há escape manual.
//
// Houve um (`BlendOpts { offset, reverse }`, o "Rotate Match" / "Reverse Match" do painel), e os
// dois foram REMOVIDOS por serem bugs de design: o **Reverse** invertia o winding e colapsava a
// forma; o **Rotate** rodava a correspondência às cegas e produzia torção. No modelo vivo (o Blend
// do Illustrator), o ajuste é feito **editando as formas-fonte** — girar a do meio adapta os
// intermediários dos dois lados —, não um botão que gira a correspondência sem o artista ver.

/// Um segmento do contorno, **com a parametrização normalizada**.
///
/// # A reta que entortava (o smoke do Enio: *"as intermediárias não representam a transição"*)
///
/// No documento, uma aresta RETA é a cúbica **degenerada** `(P0, P0, P3, P3)` — os handles
/// colapsados na âncora. Ela é geometricamente reta, mas a **parametrização dela não é uniforme**:
/// o ponto anda devagar, rápido, devagar. Duas retas cortadas em posições de arco diferentes
/// devolvem sub-cúbicas cujos pontos de controle caem em **frações diferentes** de cada aresta — e
/// o lerp de frações desalinhadas **tira o controle de cima da corda**. A aresta reta sai
/// **ondulada**.
///
/// Medido no quadrado→estrela do smoke: até **0,24 unidade** de desvio numa forma de tamanho 2 —
/// **12% da forma**. As duas pontas são polígonos, então todo intermediário TEM de ser um
/// polígono; o desvio devia ser zero.
///
/// O conserto não é "endireitar depois": é **não ter parametrização torta**. Uma reta entra aqui
/// na forma canônica (controles a ⅓ e ⅔ da corda), que é **afim em `t`** — e uma sub-cúbica de uma
/// curva afim é afim, com os controles nos mesmos ⅓ e ⅔ da sub-corda. Aí o lerp de duas retas é
/// uma reta, **por construção**, cortadas onde forem.
fn segment(a: &VecVertex, b: &VecVertex) -> CubicBez {
    let (p0, p3) = (pt(a.anchor), pt(b.anchor));
    if is_line(a, b) {
        let d = p3 - p0;
        return CubicBez::new(p0, p0 + d / 3.0, p0 + d * (2.0 / 3.0), p3);
    }
    CubicBez::new(p0, pt(a.out_handle), pt(b.in_handle), p3)
}

/// A aresta `a → b` é RETA? (os dois handles colapsados nas âncoras — a convenção do documento,
/// a mesma que a booleana usa.)
fn is_line(a: &VecVertex, b: &VecVertex) -> bool {
    close(a.out_handle, a.anchor) && close(b.in_handle, b.anchor)
}

fn close(p: [f64; 2], q: [f64; 2]) -> bool {
    (p[0] - q[0]).abs() <= COINCIDENT_EPS && (p[1] - q[1]).abs() <= COINCIDENT_EPS
}

/// Handle a esta distância da âncora conta como colapsado.
const COINCIDENT_EPS: f64 = 1e-9;

/// O contorno de uma forma como cadeia de cúbicas, com o comprimento de arco acumulado.
struct Outline {
    segs: Vec<CubicBez>,
    /// `cum[i]` = arco até o INÍCIO de `segs[i]`; `cum[n]` = o total.
    cum: Vec<f64>,
    total: f64,
    closed: bool,
}

impl Outline {
    /// A partir do contorno externo da geometria **COZIDA** (é ela que está na tela — mesma
    /// escolha da booleana: o raio de quina vivo já entra assado).
    fn of(path: &VecPath) -> Option<Outline> {
        let cooked = path.cooked();
        let verts = &cooked.verts;
        if verts.len() < 2 {
            return None;
        }
        let n = verts.len();
        let last = if cooked.closed { n } else { n - 1 };
        let mut segs = Vec::with_capacity(last);
        for i in 0..last {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            segs.push(segment(a, b));
        }
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(ARCLEN_EPS);
        }
        cum.push(total);
        if total <= MERGE_EPS {
            return None; // forma degenerada: não há o que percorrer
        }
        Some(Outline {
            segs,
            cum,
            total,
            closed: cooked.closed,
        })
    }

    /// As posições (arco normalizado, em [0,1)) das âncoras ORIGINAIS.
    fn anchors(&self) -> Vec<f64> {
        self.cum[..self.segs.len()]
            .iter()
            .map(|c| c / self.total)
            .collect()
    }

    /// O ponto no arco normalizado `s`.
    fn at(&self, s: f64) -> Point {
        let (i, t) = self.locate(s);
        self.segs[i].eval(t)
    }

    /// `n` pontos em posições de arco **igualmente espaçadas**, começando na origem do contorno.
    ///
    /// É a grade comum que torna a busca de fase uma **correlação circular**: amostradas assim, as
    /// duas formas comparam-se por um simples deslocamento de índice (`matching::phase_only`), e a
    /// fase deixa de estar presa às âncoras — que, num contorno suave, não significam nada.
    fn samples(&self, n: usize) -> Vec<Point> {
        (0..n).map(|k| self.at(k as f64 / n as f64)).collect()
    }

    /// (índice do segmento, `t` local) do arco normalizado `s`.
    fn locate(&self, s: f64) -> (usize, f64) {
        let arc = s.clamp(0.0, 1.0) * self.total;
        // O último segmento absorve o fim exato (`s == 1`).
        let i = match self.cum[1..].partition_point(|&c| c <= arc) {
            i if i >= self.segs.len() => self.segs.len() - 1,
            i => i,
        };
        let local = (arc - self.cum[i]).max(0.0);
        let seg = &self.segs[i];
        // O comprimento já está no acumulado — recalculá-lo aqui seria um `arclen` por consulta.
        let len = self.cum[i + 1] - self.cum[i];
        let t = if len <= MERGE_EPS {
            0.0
        } else {
            seg.inv_arclen(local.min(len), ARCLEN_EPS)
        };
        (i, t.clamp(0.0, 1.0))
    }

    /// A cadeia de cúbicas que sai de cortar este contorno nas posições `cuts`.
    ///
    /// **O percurso é CÍCLICO, e isso não é detalhe.** As posições de B saem de `wrap(s − fase)`,
    /// então elas são uma **rotação** da ordem crescente, não a ordem crescente: exatamente uma
    /// peça atravessa a origem do contorno. (Nunca *mais* de uma, e nunca uma âncora: a origem de
    /// B **é** uma âncora de B, e toda âncora de B está em `cuts` — é o que garante que cada peça
    /// caiba num único segmento e seja uma sub-cúbica EXATA.)
    ///
    /// Devolve **uma peça por corte** — sempre. É o que faz A e B saírem pareados 1-a-1.
    ///
    /// # O segmento se decide pelo MEIO da peça, nunca pela borda dela
    ///
    /// **Toda** borda de peça é uma âncora — é assim que este motor foi desenhado (a união inclui
    /// as âncoras das duas formas). E uma âncora é exatamente a fronteira entre dois segmentos:
    /// perguntar "de quem é este ponto?" ali é perguntar de que lado de um empate o `f64` caiu.
    ///
    /// A 1ª versão perguntava pela borda, e o smoke do Enio mostrou o preço: quando o corte era
    /// atribuído ao segmento **anterior** (`t≈1`) em vez do seguinte (`t≈0`), a peça colapsava num
    /// **ponto** — e a aresta correspondente **sumia do pareamento**. Medido no quadrado→estrela:
    /// **3 das 10 arestas da estrela viravam pontos**. As intermediárias deixavam de representar a
    /// transição, e uma delas chegou a explodir na tela.
    ///
    /// O **meio** da peça não é fronteira de nada: ele cai dentro de um único segmento, sempre.
    /// # A ORIGEM tem de ESTAR na lista — e quem depende disso é quem garante
    ///
    /// O parágrafo acima ("nenhuma peça atravessa a origem") não é uma observação: é um
    /// **invariante**, e ele vale porque a origem do contorno **é** uma âncora, e toda âncora está
    /// nos cortes. Quando ele falha, esta função **trunca** a peça que atravessaria a origem em
    /// `1.0` — e o arco entre a origem e o corte seguinte fica **sem peça nenhuma**: uma aresta
    /// inteira some do pareamento, em silêncio.
    ///
    /// E ele FALHAVA. Os cortes de B são as **imagens** dos cortes de A, e a imagem que devia cair
    /// exatamente na origem volta do ida-e-volta `map_backward` → `map_forward` como
    /// **`1 − 3,7e-14`** (medido, círculo → heptágono): o `f64` não fecha o ciclo. Aí a peça
    /// `[1−ε, 1]` é um ponto, e a aresta que começava na origem desaparece.
    ///
    /// O invariante agora é **estabelecido aqui**, e não meramente esperado do chamador: um corte a
    /// menos de `MERGE_EPS` da volta completa **é** a origem. É a mesma lição que já custou uma
    /// reprovação neste motor — *nunca deixar um `f64` decidir de que lado de um empate ele caiu* —,
    /// só que na fronteira que fecha o ciclo.
    fn cut(&self, cuts: &[f64]) -> Vec<CubicBez> {
        let m = cuts.len();
        let mut out = Vec::with_capacity(m);
        let at_origin = |s: f64| if s >= 1.0 - MERGE_EPS { 0.0 } else { s };
        for k in 0..m {
            let s0 = at_origin(cuts[k]);
            // O corte seguinte, ciclicamente. Se ele "voltou" (é a peça que fecha o contorno), o
            // fim dela é o fim do percurso.
            let next = at_origin(cuts[(k + 1) % m]);
            let s1 = if next <= s0 + MERGE_EPS { 1.0 } else { next };
            let i = self.segment_at(0.5 * (s0 + s1));
            let (t0, t1) = (self.local_t(i, s0), self.local_t(i, s1));
            out.push(self.segs[i].subsegment(t0.min(t1)..t1.max(t0)));
        }
        out
    }

    /// O segmento que contém o arco normalizado `s`. `s` tem de cair **dentro** de um segmento —
    /// os chamadores passam o MEIO de uma peça, nunca uma borda.
    fn segment_at(&self, s: f64) -> usize {
        let arc = s.clamp(0.0, 1.0) * self.total;
        self.cum[1..]
            .partition_point(|&c| c <= arc)
            .min(self.segs.len() - 1)
    }

    /// O `t` local do arco normalizado `s` **dentro do segmento `i`** (fora dele, satura em 0/1 —
    /// que é o certo: a borda da peça é a borda do segmento).
    fn local_t(&self, i: usize, s: f64) -> f64 {
        let arc = s.clamp(0.0, 1.0) * self.total;
        let len = self.cum[i + 1] - self.cum[i];
        if len <= MERGE_EPS {
            return 0.0;
        }
        let local = (arc - self.cum[i]).clamp(0.0, len);
        self.segs[i].inv_arclen(local, ARCLEN_EPS).clamp(0.0, 1.0)
    }

    /// O contorno percorrido ao CONTRÁRIO (a saída do "forma vira do avesso").
    fn reversed(&self) -> Outline {
        let segs: Vec<CubicBez> = self
            .segs
            .iter()
            .rev()
            .map(|s| CubicBez::new(s.p3, s.p2, s.p1, s.p0))
            .collect();
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(ARCLEN_EPS);
        }
        cum.push(total);
        Outline {
            segs,
            cum,
            total,
            closed: self.closed,
        }
    }
}

/// A **correspondência** (o mapa monótono entre as duas formas) — módulo irmão, pelo teto de LOC.
mod matching;
use matching::{map_backward, map_forward, search};

/// O **flow dos passos ao longo do SPINE editável** (ADR-0122) — a camada nova sobre o motor. É
/// geometria de arco pura (deslocamentos), separada da correspondência.
pub mod spine;
pub use spine::spine_offsets;

/// **O PLANO de um blend**: a correspondência já resolvida e as duas formas já cortadas, peça a
/// peça. Avaliar um `t` é, a partir daqui, só um lerp — `O(peças)`, sem busca nenhuma.
///
/// # Por que ele existe
///
/// A **correspondência é função só do par (A, B, opções)** — não do `t`. Enquanto ela era buscada
/// dentro do `morph`, um blend de 10 passos a buscava **10 vezes**, e chegava dez vezes à mesma
/// resposta. Depois que a busca de fase entrou (ela varre 256 fases contra 256 amostras), essas
/// nove buscas jogadas fora passaram a custar **5,9 ms** por blend, contra 0,18 ms do caminho da
/// DP — e o artista re-roda o blend a cada toque em Steps / Rotate.
///
/// E é a **mesma** estrutura que o **morph vivo** (o `t` animável, o próximo da fila) precisa: o
/// plano é montado quando a relação muda, e cada frame só avalia um `t`.
pub struct Plan {
    pieces_a: Vec<CubicBez>,
    pieces_b: Vec<CubicBez>,
    closed: bool,
    fill: (Option<Paint>, Option<Paint>),
    stroke: (Option<StrokeSpec>, Option<StrokeSpec>),
}

impl Plan {
    /// Resolve a correspondência entre A e B e corta as duas na união.
    ///
    /// `None` se qualquer uma das duas degenerar (menos de 2 vértices, ou comprimento nulo).
    #[must_use]
    pub fn new(a: &VecPath, b: &VecPath) -> Option<Plan> {
        let (oa, ob) = (Outline::of(a)?, Outline::of(b)?);
        let corr = search(&oa, &ob);
        let target = if corr.reversed { ob.reversed() } else { ob };
        let (pieces_a, pieces_b) = pair_up(&oa, &target, &corr.knots)?;
        Some(Plan {
            pieces_a,
            pieces_b,
            closed: oa.closed,
            fill: (a.fill.clone(), b.fill.clone()),
            stroke: (a.stroke, b.stroke),
        })
    }

    /// **A forma no meio do caminho.** `t = 0` devolve A; `t = 1`, B.
    ///
    /// Um `t` **NaN** vira `0` (devolve A), e não NaN: `f64::clamp` **propaga** NaN, e o morph vivo
    /// (§4 do handoff) vai chamar isto por frame com um `t` que sai de uma curva animada — uma
    /// singularidade nessa curva não pode virar uma forma de vértices NaN, que suja a cena e o save.
    #[must_use]
    pub fn at(&self, t: f64) -> VecPath {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let lerped: Vec<CubicBez> = self
            .pieces_a
            .iter()
            .zip(&self.pieces_b)
            .map(|(ca, cb)| {
                CubicBez::new(
                    mix(ca.p0, cb.p0, t),
                    mix(ca.p1, cb.p1, t),
                    mix(ca.p2, cb.p2, t),
                    mix(ca.p3, cb.p3, t),
                )
            })
            .collect();

        let mut out = path_from(&lerped, self.closed);
        out.fill = mix_paint(self.fill.0.as_ref(), self.fill.1.as_ref(), t);
        out.stroke = mix_stroke(self.stroke.0, self.stroke.1, t);
        out
    }
}

/// **A forma no meio do caminho.** `t = 0` devolve A; `t = 1`, B.
///
/// `None` se qualquer uma das duas degenerar (menos de 2 vértices, ou comprimento nulo).
///
/// Para mais de um `t` do MESMO par, monte um [`Plan`] — senão a correspondência é buscada de novo
/// a cada chamada, e ela não depende do `t`.
#[must_use]
pub fn morph(a: &VecPath, b: &VecPath, t: f64) -> Option<VecPath> {
    Some(Plan::new(a, b)?.at(t))
}

/// **O PAREAMENTO**: as duas formas cortadas na UNIÃO, peça a peça, na mesma ordem.
///
/// O corte é nas âncoras de A **mais a PRÉ-IMAGEM de cada âncora de B**. É a subdivisão que faz a
/// ponta da estrela nascer: cada vértice dela que não casou com uma quina do quadrado vira um ponto
/// novo na aresta do quadrado, no lugar exato que o mapa manda.
///
/// **Isto é uma função só, e de propósito.** Os gates precisam do mesmo pareamento que o produto
/// usa; enquanto eles o reconstruíam à mão, o espelho e o original podiam divergir — e divergiram
/// (o gate omitia a normalização da origem, e por sorte isso escondia um defeito em vez de inventar
/// um). Duas portas para a mesma pergunta divergem: [[feedback_two_doors_to_the_same_question_diverge]].
fn pair_up(
    oa: &Outline,
    target: &Outline,
    knots: &[(f64, f64)],
) -> Option<(Vec<CubicBez>, Vec<CubicBez>)> {
    let mut cuts: Vec<f64> = oa.anchors();
    cuts.extend(target.anchors().iter().map(|v| map_backward(knots, *v)));
    cuts.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    cuts.dedup_by(|x, y| (*x - *y).abs() <= MERGE_EPS);
    // O fim do percurso É o começo dele (fechado): um corte em ~1 é o corte em 0.
    if cuts.last().is_some_and(|&s| s >= 1.0 - MERGE_EPS) {
        cuts.pop();
    }
    if cuts.first().is_none_or(|&s| s > MERGE_EPS) {
        cuts.insert(0, 0.0);
    }

    let pieces_a = oa.cut(&cuts);
    let cuts_b: Vec<f64> = cuts.iter().map(|u| map_forward(knots, *u)).collect();
    let pieces_b = target.cut(&cuts_b);
    (!pieces_a.is_empty() && pieces_a.len() == pieces_b.len()).then_some((pieces_a, pieces_b))
}

/// Os **N passos intermediários** entre A e B — o Blend do Illustrator.
///
/// Só os do MEIO: as duas pontas são as formas que o artista já tem. `steps = 3` devolve as
/// formas em `t = ¼, ½, ¾`.
///
/// A correspondência é buscada **uma vez** ([`Plan`]) e avaliada em cada `t`: ela é função do par,
/// não do `t`.
#[must_use]
pub fn steps(a: &VecPath, b: &VecPath, n: usize) -> Vec<VecPath> {
    let Some(plan) = Plan::new(a, b) else {
        return Vec::new();
    };
    (1..=n)
        .map(|i| plan.at(i as f64 / (n + 1) as f64))
        .collect()
}

/// Os passos intermediários de uma **CADEIA** de formas — o Blend multi-forma do Illustrator.
///
/// Para N fontes (a linha aceita 2..=5), liga `fonte[0]→fonte[1]`, `fonte[1]→fonte[2]`, … — cada
/// par consecutivo é um [`Plan`] independente, com `n` passos entre eles, unidos na fonte
/// compartilhada. É a **cadeia pairwise** do Illustrator.
///
/// As **fontes NÃO entram no resultado** — elas se desenham sozinhas (são paths reais); aqui saem
/// só os intermediários **virtuais**, na ordem da cadeia. Um par que degenera (uma forma inválida)
/// é **pulado** — a cadeia não quebra por causa de um elo.
///
/// O spine é **emergente**: as fontes estão em posições diferentes do mundo, e o lerp de
/// coordenadas move os pontos, então os passos já se distribuem entre elas numa reta. O spine
/// editável (ADR-0122, Fase C) é uma re-distribuição por cima disto.
#[must_use]
pub fn chain(shapes: &[VecPath], n: usize) -> Vec<VecPath> {
    let mut out = Vec::new();
    for pair in shapes.windows(2) {
        if let Some(plan) = Plan::new(&pair[0], &pair[1]) {
            out.extend((1..=n).map(|i| plan.at(i as f64 / (n + 1) as f64)));
        }
    }
    out
}

/// Cadeia de cúbicas → `VecPath`. A âncora é o `p0` de cada peça; a alça de saída é o `p1` dela,
/// e a de entrada é o `p2` da peça ANTERIOR (é o mesmo ponto do documento, visto pelos dois
/// lados).
///
/// **Uma peça reta sai com os handles COLAPSADOS na âncora** — que é como o documento escreve uma
/// reta (e o que o resto do repo testa com `is_line`). Sem isso, o blend entre dois polígonos
/// devolveria uma forma geometricamente reta mas com alças de curva penduradas no meio de cada
/// aresta: o artista abriria o modo Node e veria controles que ele nunca pediu, a booleana a
/// trataria como curva, e o `Simplify` teria trabalho à toa.
fn path_from(segs: &[CubicBez], closed: bool) -> VecPath {
    let n = segs.len();
    let straight: Vec<bool> = segs.iter().map(is_straight).collect();
    let mut verts: Vec<VecVertex> = Vec::with_capacity(n + 1);
    for (i, s) in segs.iter().enumerate() {
        let prev_i = if i == 0 { n - 1 } else { i - 1 };
        // A alça de ENTRADA mora na âncora quando não há aresta chegando (ponta de caminho
        // aberto) ou quando a que chega é RETA — nos dois casos o documento não guarda controle.
        let in_handle = if (i == 0 && !closed) || straight[prev_i] {
            s.p0
        } else {
            segs[prev_i].p2
        };
        let out_handle = if straight[i] { s.p0 } else { s.p1 };
        verts.push(VecVertex {
            anchor: xy(s.p0),
            in_handle: xy(in_handle),
            out_handle: xy(out_handle),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    if !closed && let Some(last) = segs.last() {
        let in_handle = if straight[n - 1] { last.p3 } else { last.p2 };
        verts.push(VecVertex {
            anchor: xy(last.p3),
            in_handle: xy(in_handle),
            out_handle: xy(last.p3),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    VecPath {
        verts,
        closed,
        ..VecPath::default()
    }
}

/// A cúbica é uma RETA? (os dois controles em cima da corda — que é o que o lerp de duas retas
/// canônicas devolve, por construção).
fn is_straight(s: &CubicBez) -> bool {
    let chord = s.p3 - s.p0;
    let len = chord.hypot();
    if len <= COINCIDENT_EPS {
        return true; // peça degenerada: não há direção para desviar
    }
    [s.p1, s.p2].iter().all(|c| {
        let v = *c - s.p0;
        (v.x * chord.y - v.y * chord.x).abs() / len <= STRAIGHT_EPS
    })
}

/// O quanto um controle pode se afastar da corda e a aresta ainda contar como reta. É uma
/// tolerância **de comprimento**, e ela é frouxa de propósito: o lerp é exato, mas os `f64` que
/// chegam aqui já passaram por `inv_arclen` e por um corte — o resíduo é da ordem de 1e-12.
const STRAIGHT_EPS: f64 = 1e-9;

/// O Illustrator interpola a COR junto com a forma, e é isso que faz um blend parecer um blend
/// em vez de N cópias. Só entre dois sólidos; qualquer outra coisa (gradiente, nada) fica com o
/// lado mais próximo.
///
/// # A cor caminha em OKLab, não em sRGB
///
/// É onde superamos o Illustrator: ele interpola em **device-space** (RGB/CMYK, canal a canal), e o
/// meio de dois matizes opostos passa por um **cinza lamacento**. Em OKLab o caminho é
/// **perceptual** — a luminosidade e os eixos oponentes interpolam sem o meio-tom sujo. Preferimos
/// OKLab (cartesiano) a OKLCH (polar): o polar preserva o matiz mas força escolher o SENTIDO da
/// volta do matiz, e para um blend o caminho reto do OKLab é o esperado.
fn mix_paint(a: Option<&Paint>, b: Option<&Paint>, t: f64) -> Option<Paint> {
    match (a, b) {
        (Some(Paint::Solid(ca)), Some(Paint::Solid(cb))) => {
            Some(Paint::solid(mix_oklab(*ca, *cb, t)))
        }
        _ if t < 0.5 => a.cloned(),
        _ => b.cloned(),
    }
}

/// O TRAÇO também interpola — como a cor e a forma, é o que faz o blend parecer uma transição e não
/// N cópias. A **largura** e a **cor** caminham (largura por lerp, cor em OKLab como o fill); o resto
/// (cap/join/dash/pontas) é discreto e vem do lado mais próximo.
///
/// **Um lado só com traço** (o outro sem): a largura some **suave** (fade a 0) em vez de aparecer/
/// sumir de repente no meio — uma forma traçada que blenda para uma sem traço vê o contorno afinar.
fn mix_stroke(a: Option<StrokeSpec>, b: Option<StrokeSpec>, t: f64) -> Option<StrokeSpec> {
    match (a, b) {
        (Some(sa), Some(sb)) => Some(StrokeSpec {
            color: mix_oklab(sa.color, sb.color, t),
            width: sa.width + (sb.width - sa.width) * t,
            // cap/join/dash/pontas são discretos — o lado mais próximo (o mesmo critério do fill).
            ..(if t < 0.5 { sa } else { sb })
        }),
        // Só um lado tem traço: a largura afina até 0 (fade), com a cor/estilo do lado que existe.
        (Some(sa), None) => Some(StrokeSpec {
            width: sa.width * (1.0 - t),
            ..sa
        }),
        (None, Some(sb)) => Some(StrokeSpec {
            width: sb.width * t,
            ..sb
        }),
        (None, None) => None,
    }
}

/// Interpola duas cores no espaço **OKLab** (perceptual). O ida-e-volta sRGB→linear→OKLab→…→sRGB
/// clampa+quantiza só na fronteira do display (`to_srgb`), como manda a `ph2d-color`.
#[inline]
fn mix_oklab(a: Rgba8, b: Rgba8, t: f64) -> Rgba8 {
    use ph2d_color::{OklabColor, SrgbRgba};
    let ok = |c: Rgba8| OklabColor::from_linear(SrgbRgba([c.r, c.g, c.b, c.a]).to_linear());
    #[allow(clippy::cast_possible_truncation)]
    let m = ok(a).lerp(ok(b), t as f32).to_linear().to_srgb().0;
    Rgba8::new(m[0], m[1], m[2], m[3])
}

#[inline]
fn mix(a: Point, b: Point, t: f64) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

#[inline]
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
fn xy(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

/// Os gates da correspondência SUAVE (o giro do quadrado→círculo) — arquivo irmão pelo teto de LOC.
#[cfg(test)]
#[path = "tests_phase.rs"]
mod tests_phase;

/// Os gates da Fase A (cadeia multi-forma + cor OKLab) — arquivo irmão pelo teto de LOC.
#[cfg(test)]
#[path = "tests_chain.rs"]
mod tests_chain;
