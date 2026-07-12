//! **A orientação é DECLARADA, nunca presumida** — o gate que fecha a classe de bug
//! "a forma nasceu de cabeça para baixo".
//!
//! O mundo é Y-para-CIMA; toda referência de onde as formas vêm (SVG, a geometria
//! pré-definida do OOXML, os stencils do draw.io) é Y-para-BAIXO. Escrever a fórmula
//! "como está na referência" direto em coordenadas de mundo é um espelhamento silencioso:
//! foi assim que metade do catálogo nasceu invertida (cone e pirâmide de ponta-cabeça,
//! coração de bico para cima, cilindro com a barriga no teto). **Uma causa, vinte bugs.**
//!
//! A causa foi removida ([`crate::space`]: a forma é autorada em `(u, v)` com `v = 0` no
//! topo, e [`crate::space::Unit::p`] é a ÚNICA inversão do catálogo). Sobrou o buraco:
//! **nada obriga o autor da forma #49 a dizer para que lado ela fica de pé.**
//!
//! Este arquivo é essa obrigação, e ela é do COMPILADOR: [`declared`] é um `match`
//! exaustivo sobre `ShapeKind` **sem braço `_`**. Variante nova = não compila até o autor
//! escolher entre "não tem cima" ([`YOrient::Symmetric`]) e "tem, e é PARA CÁ"
//! ([`YOrient::Asymmetric`], com o predicado que prova).
//!
//! ## O predicado VAZIO é o inimigo
//!
//! Um predicado que também passa na forma ESPELHADA não distingue cima de baixo: ele fica
//! verde enquanto a forma renderiza invertida. Não é hipótese — quem reescreveu os
//! símbolos reportou que o primeiro rascunho das asserções do coração e da gota era vazio
//! (passariam espelhadas), e só pegou espelhando cada forma na mão. É exatamente o que a
//! asserção **(c)** deste gate (`!pred(mirror(shape))`) automatiza, para as 48, para
//! sempre.
//!
//! ## Como os predicados são escritos
//!
//! Sempre como **ordem RELATIVA em coordenadas de MUNDO** ("o ápice é o ponto mais alto e
//! está no eixo", "o bico é o ponto mais baixo do caminho"), nunca como constante mágica —
//! uma constante de mundo se torna mentira assim que a caixa do gesto muda.

use std::collections::BTreeMap;

use crate::{ALL_SHAPES, ShapeKind, ShapeValues, VecPath, cook, cubic_at};

// ─────────────────────────────────────────────────────────────────────────────
// O quadro do gate
// ─────────────────────────────────────────────────────────────────────────────

/// A caixa do gesto. **Centrada na origem** de propósito: o espelho em Y vira a negação
/// exata de `y` (zero erro de arredondamento introduzido pelo próprio teste), e o centro
/// da caixa É o centro de toda forma fechada do catálogo (todas terminam em
/// [`crate::space::fit`], que faz a bbox SER a caixa). Não-quadrada de propósito também:
/// uma caixa quadrada esconde a troca de eixos.
const BOX_A: [f64; 2] = [-2.0, -1.0];
const BOX_B: [f64; 2] = [2.0, 1.0];
/// O centro da caixa — o "eixo" e a "meia-altura" de todo predicado daqui.
const CX: f64 = (BOX_A[0] + BOX_B[0]) * 0.5;
const CY: f64 = (BOX_A[1] + BOX_B[1]) * 0.5;

/// Amostras por segmento cúbico, **pontas inclusas**. A inclusão importa: o espelho manda
/// o segmento `i` no segmento `j` PERCORRIDO AO CONTRÁRIO, e `t ↦ 1 − t` só leva a grade
/// `{0, 1/n, …, 1}` nela mesma se as duas pontas estiverem lá.
const STEPS: usize = 16;

/// Tolerância de igualdade geométrica (mundo). O ruído real é da ordem de 1e-16 (a forma
/// e o seu espelho saem da MESMA conta, com o sinal de `y` trocado — e trocar sinal é
/// exato em IEEE); 1e-9 é folga de sete ordens de grandeza sem chegar perto de qualquer
/// detalhe de verdade do catálogo.
const TOL: f64 = 1e-9;

// ─────────────────────────────────────────────────────────────────────────────
// A declaração
// ─────────────────────────────────────────────────────────────────────────────

/// Como a forma se comporta sob espelhamento em Y.
enum YOrient {
    /// Espelhar em Y é um NO-OP — não existe "cima" para errar (retângulo, elipse, a lua
    /// — cujas cornetas apontam para a direita: assimetria em **X**, não em Y).
    Symmetric,
    /// A forma TEM um cima. O predicado diz para que lado, em coordenadas de MUNDO.
    Asymmetric(fn(&VecPath) -> bool),
}

/// **O `match` que o compilador cobra.** Sem braço `_`: uma `ShapeKind` nova não compila
/// até o autor declarar para que lado ela fica de pé.
fn declared(k: ShapeKind) -> YOrient {
    use YOrient::{Asymmetric, Symmetric};
    match k {
        // ── Sem "cima": espelhar não muda nada ──────────────────────────────
        ShapeKind::Rectangle | ShapeKind::RoundRect | ShapeKind::Pill => Symmetric,
        // Volta INTEIRA no default (`sweep = 360`). O `sweep` parcial — que É assimétrico
        // — é o que Pie/Segment/Arc cozinham, e esses três estão gateados abaixo.
        ShapeKind::Ellipse => Symmetric,
        ShapeKind::Diamond | ShapeKind::HexagonFlat | ShapeKind::Junction => Symmetric,
        // A haste e as cabeças são centradas na linha do meio; apontam em X.
        ShapeKind::ArrowRight | ShapeKind::ArrowDouble | ShapeKind::Chevron => Symmetric,
        // Assimétricos em **X** apenas: a tampa do "D" e o bico do "display" à esquerda,
        // as barras da sub-rotina, o colchete que abraça pela esquerda.
        ShapeKind::Delay | ShapeKind::Display => Symmetric,
        ShapeKind::PredefinedProcess | ShapeKind::NoteBracket => Symmetric,
        // A lua: as cornetas abrem para a DIREITA e a barriga fica na meia-altura — o
        // limbo e o terminador compartilham o semi-eixo VERTICAL, então a forma é a mesma
        // de cabeça para baixo. Assimetria em X.
        ShapeKind::Moon => Symmetric,
        // A etiqueta: o bico aponta para a ESQUERDA, o ilhó fica na meia-altura, os dois
        // cantos direitos são arredondados igual. Assimetria em X.
        ShapeKind::Tag => Symmetric,
        // A cruz é simétrica nos dois eixos.
        ShapeKind::Cross => Symmetric,
        // A engrenagem no default tem 8 dentes (PAR) e a fase põe um dente às 12h — logo
        // há um dente às 6h também, e o espelho é a própria engrenagem. Um número ÍMPAR
        // de dentes a tornaria assimétrica, mas aí o "cima" seria decorativo: engrenagem
        // não tem cima. Declarar `Symmetric` aqui é a verdade, não uma dispensa.
        ShapeKind::Gear => Symmetric,

        // ── Tem cima, e o predicado prova para que lado ─────────────────────
        // Lado ÍMPAR: uma ponta no topo, duas âncoras na base (a estrela idem).
        ShapeKind::Polygon | ShapeKind::Star => Asymmetric(one_point_up),
        // A espiral cresce do centro e a ponta LIVRE acaba às 12h.
        ShapeKind::Spiral => Asymmetric(free_end_up),
        // A reta é a DIAGONAL da caixa: o espelho dela é a anti-diagonal.
        ShapeKind::Line => Asymmetric(rises_to_the_right),
        // Meia-volta a partir das 3h: o arco abaúla para CIMA (as duas pontas são o fundo).
        ShapeKind::Arc => Asymmetric(bulges_up),
        // Mesma meia-volta, fechada: o lado RETO (corda/centro) fica embaixo.
        ShapeKind::Pie | ShapeKind::Segment => Asymmetric(flat_side_down),
        // A seta em L entra rente à base e sai para CIMA: a ponta é única lá em cima.
        ShapeKind::ArrowBent => Asymmetric(one_point_up),
        // Dados: o topo desliza para a DIREITA em relação à base.
        ShapeKind::Parallelogram => Asymmetric(leans_right),
        // Operação manual: o topo é o lado CURTO. O `flip` (entrada manual) inverte.
        ShapeKind::Trapezoid => Asymmetric(top_narrower),
        ShapeKind::TrapezoidFlip => Asymmetric(top_wider),
        // Base de dados: a TAMPA (o sub-contorno) fica no topo, a barriga embaixo.
        ShapeKind::Cylinder => Asymmetric(cap_on_top),
        // Papel: topo RETO (duas quinas), base ONDULADA (o fundo não é âncora nenhuma).
        ShapeKind::Document => Asymmetric(flat_top_wavy_bottom),
        // Conector fora-de-página: o bico aponta para BAIXO.
        ShapeKind::OffPage => Asymmetric(one_point_down),
        // O rabo aponta para quem fala — e o default fala de BAIXO (`tip_v = 0,97`).
        ShapeKind::SpeechRect | ShapeKind::SpeechOval => Asymmetric(tail_points_down),
        // A corrente de bolhas DESCE da nuvem até o pensador.
        ShapeKind::Thought => Asymmetric(bubbles_descend),
        // A primeira ponta da explosão olha para CIMA (`BURST_PHASE_DEG = −90`).
        ShapeKind::Burst => Asymmetric(first_spike_up),
        // A nuvem não tem um cima SEMÂNTICO — mas a irregularidade dela é uma tabela fixa,
        // e uma tabela fixa não é simétrica. Ver [`mass_sits_high`] (o caso honesto).
        ShapeKind::Cloud => Asymmetric(mass_sits_high),
        // A chave é SIMÉTRICA no default (`pinch = 0,5`) — o `{` de manual. O gate a
        // cozinha com o bico empurrado para a base (ver [`values`]).
        ShapeKind::Brace => Asymmetric(beak_below_middle),
        // O bico do coração aponta para BAIXO.
        ShapeKind::Heart => Asymmetric(tip_is_the_lowest_point),
        // O raio bate no canto INFERIOR DIREITO (a diagonal da tabela do ECMA).
        ShapeKind::Bolt => Asymmetric(strikes_bottom_right),
        // O bico da gota aponta para CIMA (e é uma QUINA: os flancos são as tangentes).
        ShapeKind::Drop => Asymmetric(sharp_peak_up),
        // A ponta do escudo aponta para BAIXO; o topo é reto.
        ShapeKind::Shield => Asymmetric(one_point_down),
        // O cotovelo do check fica EMBAIXO à ESQUERDA.
        ShapeKind::Check => Asymmetric(elbow_bottom_left),
        // A dobra do banner fica em CIMA (as caudas alcançam as duas bordas lá); o painel
        // pendura embaixo, estreito.
        ShapeKind::Banner => Asymmetric(top_wider),
        // O cubo é visto DE CIMA: das três arestas internas, uma DESCE (a quina da frente)
        // e duas SOBEM (os ombros).
        ShapeKind::IsoCube => Asymmetric(one_spoke_down),
        // Cone e pirâmide: ápice para CIMA. (O cone prova pelo bico afiado; a pirâmide
        // NÃO pode — o bico dela espelhado continua sendo um bico afiado no eixo, e o
        // predicado sairia VAZIO. Ela prova pela contagem: só o ápice fica acima do meio.)
        ShapeKind::IsoCone => Asymmetric(sharp_peak_up),
        ShapeKind::IsoPyramid => Asymmetric(apex_alone_above),
    }
}

/// Os valores com que o gate cozinha cada forma: o **default do catálogo**, exceto onde o
/// default esconde a assimetria — um caso só, anotado aqui.
fn values(k: ShapeKind) -> ShapeValues {
    let mut v = k.defaults();
    if k == ShapeKind::Brace {
        // `pinch = 0,5` (o default) põe o bico do `{` na meia-altura: a chave fica
        // SIMÉTRICA e o gate ficaria cego para o eixo dela. Cozinhamos com o bico
        // empurrado para a BASE (`v` do espaço de autoria cresce para baixo), que é onde
        // um eixo invertido apareceria na hora.
        v[1] = 0.7;
    }
    v
}

// ─────────────────────────────────────────────────────────────────────────────
// Ferramental compartilhado (os predicados são uma ou duas linhas por causa dele)
// ─────────────────────────────────────────────────────────────────────────────

/// A CURVA amostrada (todos os contornos, pontas inclusas) — não as âncoras. Uma Bézier
/// passeia fora do casco dos seus pontos de controle: o bico do coração e a barriga da
/// gota vivem justamente aí.
fn curve(p: &VecPath) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    let mut eat = |verts: &[crate::VecVertex], closed: bool| {
        let n = verts.len();
        if n < 2 {
            return;
        }
        let segs = if closed { n } else { n - 1 };
        for i in 0..segs {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            for s in 0..=STEPS {
                let t = s as f64 / STEPS as f64;
                out.push(cubic_at(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
            }
        }
    };
    eat(&p.verts, p.closed);
    for c in &p.subpaths {
        eat(&c.verts, c.closed);
    }
    out
}

/// As âncoras de todos os contornos.
fn anchors(p: &VecPath) -> Vec<[f64; 2]> {
    p.verts_all().map(|v| v.anchor).collect()
}

fn top_y(pts: &[[f64; 2]]) -> f64 {
    pts.iter().map(|q| q[1]).fold(f64::MIN, f64::max)
}

fn bot_y(pts: &[[f64; 2]]) -> f64 {
    pts.iter().map(|q| q[1]).fold(f64::MAX, f64::min)
}

/// O ponto mais BAIXO.
fn lowest(pts: &[[f64; 2]]) -> [f64; 2] {
    let y = bot_y(pts);
    *pts.iter().find(|q| q[1] <= y).expect("a forma tem pontos")
}

/// Quantos pontos estão na altura `y` — a contagem que separa "uma ponta" de "um lado
/// reto" sem citar coordenada nenhuma.
fn count_at_y(pts: &[[f64; 2]], y: f64) -> usize {
    pts.iter().filter(|q| (q[1] - y).abs() < TOL).count()
}

/// A largura da forma NA altura `y` (a distância entre a âncora mais à esquerda e a mais
/// à direita que vivem lá).
fn span_x_at(pts: &[[f64; 2]], y: f64) -> f64 {
    let xs: Vec<f64> = pts
        .iter()
        .filter(|q| (q[1] - y).abs() < TOL)
        .map(|q| q[0])
        .collect();
    let lo = xs.iter().copied().fold(f64::MAX, f64::min);
    let hi = xs.iter().copied().fold(f64::MIN, f64::max);
    if xs.is_empty() { 0.0 } else { hi - lo }
}

/// O `x` mais à esquerda entre as âncoras que vivem na altura `y`.
fn left_x_at(pts: &[[f64; 2]], y: f64) -> f64 {
    pts.iter()
        .filter(|q| (q[1] - y).abs() < TOL)
        .map(|q| q[0])
        .fold(f64::MAX, f64::min)
}

/// A média de `y` de um contorno — o "onde ele mora", para comparar corpo × bolhas.
fn mean_y(verts: &[crate::VecVertex]) -> f64 {
    verts.iter().map(|v| v.anchor[1]).sum::<f64>() / verts.len() as f64
}

/// **O espelho em Y**, em torno do centro da CAIXA. Como a caixa é centrada na origem,
/// isto é a negação exata de `y`: o teste não injeta erro nenhum.
fn mirror_y(p: &VecPath) -> VecPath {
    let mut m = p.clone();
    m.for_each_vert_mut(|v| {
        for q in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
            q[1] = 2.0 * CY - q[1];
        }
    });
    m
}

/// Lado da célula do índice espacial. Grosso em relação a [`TOL`] de propósito: o vizinho
/// de um ponto na borda da célula está, no máximo, na célula ao lado.
const CELL: f64 = 1e-3;

fn cell_of(p: [f64; 2]) -> (i64, i64) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "coordenada de teste, limitada"
    )]
    ((p[0] / CELL).floor() as i64, (p[1] / CELL).floor() as i64)
}

/// Índice espacial dos pontos, para casar dois conjuntos sem varredura quadrática.
///
/// `BTreeMap` e não `HashMap`: o `RandomState` da std é não-determinístico cross-platform e
/// está banido no workspace (ADR-0022 / HR-5). Aqui o mapa é só uma grade — ordenado serve.
///
/// **Ordenar-e-comparar seria FRÁGIL, e o modo de falhar é traiçoeiro:** numa forma
/// simétrica dois pontos distintos compartilham o mesmo `x` e só se distinguem pelo `y` —
/// e o espelho troca o sinal do `y`. Bastaria 1 ulp de ruído no `x` para o par sair
/// trocado na ordenação, e o teste acusaria diferença onde não há.
fn index(pts: &[[f64; 2]]) -> BTreeMap<(i64, i64), Vec<[f64; 2]>> {
    let mut m: BTreeMap<(i64, i64), Vec<[f64; 2]>> = BTreeMap::new();
    for &q in pts {
        m.entry(cell_of(q)).or_default().push(q);
    }
    m
}

fn on_curve(q: [f64; 2], idx: &BTreeMap<(i64, i64), Vec<[f64; 2]>>) -> bool {
    let (cx, cy) = cell_of(q);
    for dx in -1..=1 {
        for dy in -1..=1 {
            let Some(bucket) = idx.get(&(cx + dx, cy + dy)) else {
                continue;
            };
            if bucket
                .iter()
                .any(|r| (r[0] - q[0]).hypot(r[1] - q[1]) < TOL)
            {
                return true;
            }
        }
    }
    false
}

/// **A MESMA forma?** Comparação como CONJUNTO de pontos, nos dois sentidos. Espelhar
/// inverte o sentido de percurso e roda o vértice inicial, então comparar vértice a
/// vértice estaria errado — o que importa é a tinta na tela.
fn same_shape(a: &VecPath, b: &VecPath) -> bool {
    let (ca, cb) = (curve(a), curve(b));
    let (ia, ib) = (index(&ca), index(&cb));
    ca.iter().all(|&q| on_curve(q, &ib)) && cb.iter().all(|&q| on_curve(q, &ia))
}

// ─────────────────────────────────────────────────────────────────────────────
// Os predicados — cada um é uma ordem RELATIVA no mundo Y-para-cima
// ─────────────────────────────────────────────────────────────────────────────

/// Uma ponta ÚNICA no topo, mais de uma âncora na base (polígono/estrela de lado ímpar; a
/// seta em L). Espelhada, a contagem troca de lado.
fn one_point_up(p: &VecPath) -> bool {
    let a = anchors(p);
    count_at_y(&a, top_y(&a)) == 1 && count_at_y(&a, bot_y(&a)) > 1
}

/// O espelho do anterior: bico único embaixo, lado reto em cima (fora-de-página, escudo).
fn one_point_down(p: &VecPath) -> bool {
    let a = anchors(p);
    count_at_y(&a, bot_y(&a)) == 1 && count_at_y(&a, top_y(&a)) > 1
}

/// A ponta LIVRE da espiral (o último vértice, na borda) é o ponto mais alto dela.
fn free_end_up(p: &VecPath) -> bool {
    let a = anchors(p);
    let end = p.verts.last().expect("a espiral tem vertices").anchor;
    (end[1] - top_y(&a)).abs() < TOL && count_at_y(&a, top_y(&a)) == 1
}

/// A ponta de maior `x` é também a de maior `y`: a reta é a DIAGONAL da caixa (o espelho
/// dela é a ANTI-diagonal).
fn rises_to_the_right(p: &VecPath) -> bool {
    let (a, b) = (p.verts[0].anchor, p.verts[1].anchor);
    (a[0] > b[0]) == (a[1] > b[1])
}

/// O arco abaúla para CIMA: as duas pontas do traço são o ponto mais baixo dele, e a
/// curva sobe entre elas.
fn bulges_up(p: &VecPath) -> bool {
    let c = curve(p);
    let ends = [
        p.verts[0].anchor,
        p.verts.last().expect("o arco tem vertices").anchor,
    ];
    let floor = bot_y(&c);
    ends.iter().all(|e| (e[1] - floor).abs() < TOL) && top_y(&c) > floor + TOL
}

/// O lado RETO (a corda, ou as duas bordas radiais da pizza) fica embaixo: lá moram várias
/// âncoras; em cima, só a crista do arco.
fn flat_side_down(p: &VecPath) -> bool {
    let a = anchors(p);
    count_at_y(&a, bot_y(&a)) > count_at_y(&a, top_y(&a))
}

/// O paralelogramo de dados inclina para a DIREITA: o topo é deslocado no sentido positivo
/// de `x` em relação à base.
fn leans_right(p: &VecPath) -> bool {
    let a = anchors(p);
    left_x_at(&a, top_y(&a)) > left_x_at(&a, bot_y(&a))
}

/// Topo mais CURTO que a base (operação manual).
fn top_narrower(p: &VecPath) -> bool {
    let a = anchors(p);
    span_x_at(&a, top_y(&a)) < span_x_at(&a, bot_y(&a))
}

/// Topo mais LARGO que a base (entrada manual; e o banner, cujas caudas alcançam as duas
/// bordas em cima enquanto o painel pendura estreito embaixo).
fn top_wider(p: &VecPath) -> bool {
    let a = anchors(p);
    span_x_at(&a, top_y(&a)) > span_x_at(&a, bot_y(&a))
}

/// A TAMPA do cilindro (a meia-elipse da frente do topo, um contorno próprio) vive inteira
/// na metade de CIMA. É ela que faz o cilindro parecer cilindro — e é o único detalhe que
/// distingue a forma do seu espelho (a silhueta, sozinha, é simétrica).
fn cap_on_top(p: &VecPath) -> bool {
    !p.subpaths.is_empty()
        && p.subpaths
            .iter()
            .flat_map(|c| c.verts.iter())
            .all(|v| v.anchor[1] > CY)
}

/// Papel: o topo é RETO (duas quinas na altura máxima) e a base é a ONDA — cujo ponto mais
/// baixo é o miolo de uma cúbica, âncora nenhuma.
fn flat_top_wavy_bottom(p: &VecPath) -> bool {
    let (a, c) = (anchors(p), curve(p));
    count_at_y(&a, top_y(&c)) == 2 && count_at_y(&a, bot_y(&c)) == 0
}

/// O bico do rabo é a AGULHA: o único vértice cujos dois handles coincidem FORA da âncora
/// (a cúspide de 180° que o rabo de quadrinho é). Ele tem de ser o ponto mais baixo do
/// balão — o rabo aponta para quem fala, não para o céu.
fn tail_points_down(p: &VecPath) -> bool {
    let mut needle = None;
    for v in p.verts_all() {
        if v.in_handle == v.out_handle && v.in_handle != v.anchor {
            if needle.is_some() {
                return false; // a agulha deixou de ser única: o landmark morreu
            }
            needle = Some(v.anchor);
        }
    }
    needle.is_some_and(|t| (t[1] - bot_y(&curve(p))).abs() < TOL)
}

/// A corrente de bolhas do pensamento DESCE: cada bolha (um sub-contorno) mora abaixo do
/// corpo da nuvem.
fn bubbles_descend(p: &VecPath) -> bool {
    let body = mean_y(&p.verts);
    !p.subpaths.is_empty() && p.subpaths.iter().all(|c| mean_y(&c.verts) < body)
}

/// A PRIMEIRA ponta da explosão olha para cima (`BURST_PHASE_DEG = −90`): o vértice 0 é o
/// ponto mais alto da forma.
fn first_spike_up(p: &VecPath) -> bool {
    (p.verts[0].anchor[1] - top_y(&curve(p))).abs() < TOL
}

/// **A nuvem é o caso honesto e desconfortável do catálogo.** Ela NÃO tem um "cima"
/// semântico — de ponta-cabeça ainda é uma nuvem perfeitamente boa. Mas também não é
/// simétrica: a irregularidade dela é uma **tabela FIXA** (as 11 razões `wR` medidas do
/// `cloud` do ECMA-376, mais a fase de 200°), e uma tabela fixa desenha um blob específico,
/// cujo espelho é outro blob. `Symmetric` seria mentira (a asserção (1) cai na hora).
///
/// Então o predicado prende o que existe de verdade: **o centro de MASSA da nuvem não fica
/// no centro da caixa** — a tabela do ECMA a deixa pesada em cima (`+3%` da meia-altura,
/// medido). É um hash da tabela, não uma silhueta reconhecível, e está anotado como tal —
/// mas é uma ordem relativa no mundo, é determinística, e o espelho a reprova: um eixo `v`
/// invertido na autoria da nuvem não passa por aqui.
///
/// Centroide de ÁREA (shoelace), não a média das amostras: a média pesaria por número de
/// segmentos, e uma bolha a mais mudaria o número sem mudar a forma.
fn mass_sits_high(p: &VecPath) -> bool {
    let c = curve(p);
    let (mut a2, mut sy) = (0.0, 0.0);
    for i in 0..c.len() {
        let (u, v) = (c[i], c[(i + 1) % c.len()]);
        let cross = u[0] * v[1] - v[0] * u[1];
        a2 += cross;
        sy += (u[1] + v[1]) * cross;
    }
    a2.abs() > TOL && sy / (3.0 * a2) > CY + TOL
}

/// O bico da chave, com o `pinch` empurrado para a base: o ponto mais à ESQUERDA fica
/// abaixo da meia-altura.
fn beak_below_middle(p: &VecPath) -> bool {
    let c = curve(p);
    let beak = c.iter().fold(
        [f64::MAX, 0.0],
        |acc, q| if q[0] < acc[0] { *q } else { acc },
    );
    beak[1] < CY - TOL
}

/// O bico do coração é o ponto mais baixo da CURVA e é uma ÂNCORA, no eixo. Espelhado, a
/// âncora mais baixa passa a ser o vale — e os lóbulos (que não são âncoras) descem
/// abaixo dele.
fn tip_is_the_lowest_point(p: &VecPath) -> bool {
    let (a, c) = (anchors(p), curve(p));
    (bot_y(&a) - bot_y(&c)).abs() < TOL && (lowest(&a)[0] - CX).abs() < TOL
}

/// O raio bate no canto INFERIOR DIREITO: o ponto mais baixo é também o mais à direita.
fn strikes_bottom_right(p: &VecPath) -> bool {
    let c = curve(p);
    let right = c.iter().map(|q| q[0]).fold(f64::MIN, f64::max);
    (lowest(&c)[0] - right).abs() < TOL
}

/// Bico AFIADO para cima: o ponto mais alto da curva é uma âncora de QUINA (os dois
/// handles colados nela — os flancos são retas tangentes), e ela está no eixo. Espelhada,
/// o ponto mais alto passa a ser a barriga do bulbo/da base — que não é âncora, e muito
/// menos quina.
fn sharp_peak_up(p: &VecPath) -> bool {
    let c = curve(p);
    let top = top_y(&c);
    p.verts_all().any(|v| {
        (v.anchor[1] - top).abs() < TOL
            && v.in_handle == v.anchor
            && v.out_handle == v.anchor
            && (v.anchor[0] - CX).abs() < TOL
    })
}

/// O cotovelo do check é o ponto mais baixo, e fica à ESQUERDA do eixo (espelhado, o mais
/// baixo passa a ser a ponta do braço longo, que fica à direita).
fn elbow_bottom_left(p: &VecPath) -> bool {
    lowest(&curve(p))[0] < CX - TOL
}

/// O cubo é visto DE CIMA: das três arestas internas (todas incidentes no vértice central),
/// **uma desce** — a quina da frente — e **duas sobem**, os ombros. Espelhado, dois descem.
fn one_spoke_down(p: &VecPath) -> bool {
    let pts: Vec<[f64; 2]> = p
        .subpaths
        .iter()
        .flat_map(|c| c.verts.iter().map(|v| v.anchor))
        .collect();
    let same = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) < TOL;
    // O centro é a âncora COMPARTILHADA (as três arestas partem dele).
    let Some(hub) = pts
        .iter()
        .copied()
        .find(|&q| pts.iter().filter(|&&r| same(q, r)).count() >= 2)
    else {
        return false;
    };
    let spokes: Vec<[f64; 2]> = pts.iter().copied().filter(|&q| !same(q, hub)).collect();
    let down = spokes.iter().filter(|q| q[1] < hub[1]).count();
    let up = spokes.iter().filter(|q| q[1] > hub[1]).count();
    down == 1 && up == 2
}

/// A pirâmide: só o ÁPICE fica acima da meia-altura; os três cantos da base ficam abaixo.
/// (O bico afiado não serve aqui — espelhado, o vértice da frente também é um bico afiado
/// no eixo, e o predicado sairia VAZIO. A contagem não sai.)
fn apex_alone_above(p: &VecPath) -> bool {
    let above = p.verts.iter().filter(|v| v.anchor[1] > CY).count();
    let below = p.verts.iter().filter(|v| v.anchor[1] < CY).count();
    above == 1 && below == 3
}

// ─────────────────────────────────────────────────────────────────────────────
// O gate
// ─────────────────────────────────────────────────────────────────────────────

/// **O gate.** Para TODA forma do catálogo, nos valores de [`values`]:
///
/// 1. `Symmetric` ⇒ a forma e o seu espelho em Y são a MESMA forma (como conjunto de
///    pontos: espelhar inverte o winding e roda o vértice inicial);
/// 2. `Asymmetric(pred)` ⇒ (a) `pred` vale; (b) o espelho **não** é a mesma forma (uma
///    forma declarada assimétrica que na verdade é simétrica é uma MENTIRA); e (c) — o
///    coração disto tudo — `!pred(espelho)`: um predicado que também passa espelhado não
///    distingue cima de baixo, é VAZIO, e ficaria verde com a forma de ponta-cabeça.
#[test]
fn every_shape_declares_which_way_is_up_and_proves_it() {
    for &k in ALL_SHAPES {
        let p = cook(k, BOX_A, BOX_B, &values(k));
        let m = mirror_y(&p);
        match declared(k) {
            YOrient::Symmetric => assert!(
                same_shape(&p, &m),
                "{k:?}: declarada Symmetric, mas o espelho em Y e OUTRA forma. Ou ela tem \
                 um cima (declare Asymmetric com o predicado que prova), ou a geometria \
                 esta torta."
            ),
            YOrient::Asymmetric(pred) => {
                assert!(
                    pred(&p),
                    "{k:?}: o predicado de orientacao FALHOU na forma cozida. Isto e um bug \
                     de geometria de verdade (a forma nasceu espelhada?) — nao enfraqueca o \
                     predicado."
                );
                assert!(
                    !same_shape(&p, &m),
                    "{k:?}: declarada Asymmetric, mas espelhar em Y devolve a MESMA forma. \
                     A declaracao mente: ela nao tem cima — use Symmetric."
                );
                assert!(
                    !pred(&m),
                    "{k:?}: PREDICADO VAZIO. Ele passa TAMBEM na forma espelhada, entao nao \
                     distingue cima de baixo: ficaria verde com a forma de ponta-cabeca na \
                     tela. Reescreva-o como uma ordem RELATIVA que so vale de pe."
                );
            }
        }
    }
}

/// **O espelho do teste é o teste do espelho.** Se `mirror_y` fosse um no-op (ou se
/// `same_shape` achasse tudo igual), o gate inteiro passaria vazio. Aqui a régua é
/// aferida: uma forma que TEM cima difere do espelho; uma que não tem, não.
#[test]
fn the_mirror_and_the_ruler_actually_work() {
    let up = cook(
        ShapeKind::IsoCone,
        BOX_A,
        BOX_B,
        &ShapeKind::IsoCone.defaults(),
    );
    assert!(
        same_shape(&up, &up.clone()),
        "a regua nao reconhece a si mesma"
    );
    assert!(
        !same_shape(&up, &mirror_y(&up)),
        "o cone espelhado tem de ser outra forma — o espelho esta inerte"
    );
    let box_ = cook(
        ShapeKind::Rectangle,
        BOX_A,
        BOX_B,
        &ShapeKind::Rectangle.defaults(),
    );
    assert!(
        same_shape(&box_, &mirror_y(&box_)),
        "o retangulo espelhado E o retangulo — a regua esta sensivel demais"
    );
}
