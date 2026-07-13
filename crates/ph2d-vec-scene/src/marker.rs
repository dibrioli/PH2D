//! **Pontas de traço** (arrowheads / markers) — módulo irmão de [`crate::shapes`].
//!
//! ## Por que isto NÃO é uma forma do catálogo
//!
//! Já existe uma família de setas em BLOCO (`arrows.rs`): silhuetas preenchíveis, com corpo
//! e cabeça, que se desenham arrastando uma caixa. São ótimas para *apontar para algo num
//! cartaz*, e péssimas para o que se quer aqui.
//!
//! Uma ponta de traço é outra coisa: é uma **propriedade do stroke**. Ela nasce na ponta de
//! um caminho — qualquer caminho aberto: uma linha, um arco, uma espiral, uma curva
//! desenhada à mão, e (o que vem a seguir) um **conector**. Ela herda a cor e a largura do
//! traço, gira sozinha com a tangente da curva, e não tem caixa própria. Modelá-la como
//! forma obrigaria o usuário a posicioná-la e girá-la à mão a cada vez que a linha mudasse
//! — que é exatamente o trabalho que um editor existe para não fazer.
//!
//! É também a razão de ela vir ANTES dos conectores: um conector sem ponta é uma linha.
//!
//! ## O modelo (o do SVG, que é o que todo mundo usa)
//!
//! - A geometria do marcador é autorada numa caixa unitária e **escalada pela largura do
//!   traço** (`markerUnits = "strokeWidth"`, o default do SVG). Engrossar a linha engrossa a
//!   ponta na proporção — senão uma linha grossa terminaria num alfinete.
//! - Ele é **orientado pela tangente** da curva no extremo (`orient = "auto"`).
//! - A linha é **encurtada** pelo recuo do marcador ([`Marker::inset`]), senão o traço
//!   apareceria por dentro de uma ponta vazada (a `Open` e a `Bar` não têm o que esconder,
//!   mas a `Triangle` e o `Diamond` têm). É a mesma razão pela qual um conector tem de parar
//!   na borda da caixa, não no centro dela — e é o mesmo mecanismo.
//!
//! ## Os dois ajustes do usuário: `scale` e `round`
//!
//! **`scale`** ([`StrokeSpec::marker_scale`](crate::StrokeSpec::marker_scale)) multiplica o
//! tamanho que a largura já dita — é `w_efetivo = w · scale`, e mais nada. O ponto delicado é
//! que o **recuo tem de escalar junto** ([`Marker::inset`] recebe o mesmo `scale`): se a
//! cabeça cresce e a linha continua parando onde parava, o traço reaparece **por dentro** da
//! seta, atravessando-a. Recuo e geometria são a MESMA medida vista de dois lados, e o gate
//! `the_line_always_stops_exactly_at_the_back_of_the_head` amarra os dois.
//!
//! **`round`** ([`StrokeSpec::marker_round`](crate::StrokeSpec::marker_round)) arredonda as
//! quinas da ponta. Não é o `stroke-linejoin: round` do SVG — esse arredonda a junção de um
//! traço, e aqui nós GERAMOS o contorno, não o traçamos. O certo é o **filete** de CAD, o
//! mesmo de [`crate::corners`]: em cada quina recua-se `t` ao longo das duas arestas e
//! ligam-se os dois pontos por uma cúbica cujos controles apontam para a quina original.
//!
//! Aqui o filete é parametrizado por **fração**, não por raio: `round = 1` significa "o
//! máximo que esta ponta comporta", que é `t = metade da menor aresta adjacente`. É o mesmo
//! clamp de [`crate::corners`], só que promovido de rede-de-segurança a **unidade da escala**
//! — um raio absoluto não tem sentido aqui, porque a ponta muda de tamanho com a largura e
//! com o `scale`, e o usuário quer "arredondado até o talo", não "arredondado 3 px". Sem esse
//! teto o filete comeria a aresta inteira e a seta viraria uma pastilha.
//!
//! Uma ponta **sem quina** (o `Circle`) ignora o `round` — não há o que arredondar, e forçar
//! algo ali seria inventar. A `Bar` é um risco reto: os extremos dela são pontas de traço (o
//! `cap` da caneta), não quinas — também não têm filete. Já a `Open` TEM uma quina (o bico do
//! "V"), e essa arredonda.

use crate::{StrokeSpec, VecPath, VecVertex, VertexKind};

/// A ponta de um traço. **Append-only**: o discriminante é gravado no documento.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Default, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(u8)]
pub enum Marker {
    /// Sem ponta (o default — uma linha é uma linha).
    #[default]
    None = 0,
    /// Triângulo cheio: a seta clássica de fluxograma.
    Triangle = 1,
    /// Ponta ABERTA (duas riscas em "V"): a seta de diagrama de classes / UML.
    Open = 2,
    /// Losango cheio (agregação, em UML).
    Diamond = 3,
    /// Losango VAZADO (composição, em UML) — o mesmo contorno, sem preenchimento.
    DiamondOpen = 4,
    /// Círculo cheio (o "ponto" de uma âncora, ou o bullet de um diagrama ER).
    Circle = 5,
    /// Círculo VAZADO.
    CircleOpen = 6,
    /// Barra perpendicular (o "1" de uma cardinalidade; ou um fim-de-linha).
    Bar = 7,
}

/// Todos os marcadores, na ordem do seletor.
pub const ALL_MARKERS: &[Marker] = &[
    Marker::None,
    Marker::Triangle,
    Marker::Open,
    Marker::Diamond,
    Marker::DiamondOpen,
    Marker::Circle,
    Marker::CircleOpen,
    Marker::Bar,
];

/// Meia-largura padrão de uma ponta, em múltiplos da largura do traço.
const HALF_W: f64 = 2.0;
/// Comprimento padrão de uma ponta, em múltiplos da largura do traço.
const LEN: f64 = 4.0;
/// Raio de uma ponta redonda.
const R: f64 = 2.0;
/// Abaixo disto (unidades de mundo) um comprimento / recuo é tratado como zero.
const EPS: f64 = 1e-9;

impl Marker {
    /// O discriminante gravado no documento.
    #[must_use]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// O marcador de um discriminante gravado. `None` (o `Option`) = veio de uma versão
    /// futura — o chamador trata como "sem ponta" em vez de entrar em pânico.
    #[must_use]
    pub fn from_u8(v: u8) -> Option<Self> {
        ALL_MARKERS.iter().copied().find(|m| m.as_u8() == v)
    }

    /// Rótulo para a UI (inglês).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Marker::None => "None",
            Marker::Triangle => "Arrow",
            Marker::Open => "Open",
            Marker::Diamond => "Diamond",
            Marker::DiamondOpen => "Diamond (hollow)",
            Marker::Circle => "Circle",
            Marker::CircleOpen => "Circle (hollow)",
            Marker::Bar => "Bar",
        }
    }

    /// A ponta é PREENCHIDA? As vazadas (`Open`, `DiamondOpen`, `CircleOpen`) são desenhadas
    /// com o mesmo traço da linha — é o que as faz parecerem "da mesma caneta".
    #[must_use]
    pub fn is_filled(self) -> bool {
        matches!(self, Marker::Triangle | Marker::Diamond | Marker::Circle)
    }

    /// **Quanto a linha tem de RECUAR**, em múltiplos da largura do traço, para não aparecer
    /// por dentro da ponta — com a cabeça no tamanho `scale` (o mesmo que vai para o
    /// [`Marker::build`]).
    ///
    /// Uma ponta VAZADA precisa do recuo INTEIRO (o traço cruzaria o vão e se veria por
    /// dentro dela); uma ponta CHEIA esconde a linha e pode recuar um pouco menos, mas ainda
    /// recua — senão a junção do traço com a base do triângulo forma uma bolha nas larguras
    /// grandes. A `Open` e a `Bar` não fecham região nenhuma: recuo zero.
    ///
    /// **O `scale` NÃO é opcional aqui.** O recuo é a profundidade da cabeça vista do outro
    /// lado; deixá-lo para trás quando a cabeça cresce é fazer a linha reaparecer atravessando
    /// a seta. Por isso não existe um `inset()` de conveniência: quem constrói a ponta e quem
    /// encurta a linha passam o MESMO número, ou o desenho mente.
    #[must_use]
    pub fn inset(self, scale: f64) -> f64 {
        let base = match self {
            Marker::None | Marker::Open | Marker::Bar => 0.0,
            Marker::Triangle => LEN,
            Marker::Diamond | Marker::DiamondOpen => 2.0 * LEN,
            Marker::Circle | Marker::CircleOpen => 2.0 * R,
        };
        base * scale.max(0.0)
    }

    /// A geometria da ponta, em MUNDO: na ponta `tip`, apontando na direção `dir` (unitária,
    /// a tangente da curva no extremo, apontando **para fora**), com o traço de largura `w`,
    /// a cabeça no tamanho `scale` e as quinas arredondadas por `round`.
    ///
    /// A caixa é escalada por `w · scale` (o `markerUnits = "strokeWidth"` do SVG, mais o
    /// ajuste do usuário): a ponta engrossa junto com a linha. Contorno FECHADO nas cheias e
    /// nas vazadas de região (elas são preenchidas ou traçadas conforme [`Marker::is_filled`]);
    /// ABERTO na `Open` e na `Bar`, que são riscos.
    ///
    /// `round ∈ [0, 1]` (clampado): `0` = quinas afiadas, e a saída é **byte-idêntica** à de
    /// antes de existir este parâmetro; `1` = o filete máximo que a ponta comporta. Ver o
    /// módulo. `scale ≤ 0` (ou `w ≤ 0`) = ponta invisível ⇒ `None`, que é o mesmo que não ter
    /// ponta.
    #[must_use]
    pub fn build(
        self,
        tip: [f64; 2],
        dir: [f64; 2],
        w: f64,
        scale: f64,
        round: f64,
    ) -> Option<VecPath> {
        // O `scale` entra na geometria pela largura EFETIVA — e só por ela. É o que garante
        // que ele e o `inset` (que multiplica a mesma coisa) nunca possam divergir.
        // (A comparação POSITIVA também barra o NaN, que passaria por um `<=`.)
        let w = if w * scale > EPS {
            w * scale
        } else {
            return None;
        };
        let round = round.clamp(0.0, 1.0);
        let (dx, dy) = (dir[0], dir[1]);
        let len = dx.hypot(dy);
        if len < 1e-12 {
            return None; // sem tangente não há para onde apontar
        }
        let (ux, uy) = (dx / len, dy / len); // ao longo da linha, para FORA
        let (nx, ny) = (-uy, ux); // a normal
        // `(a, b)` = `a` unidades para trás da ponta + `b` unidades para o lado, tudo em
        // múltiplos da largura do traço. Um sistema de coordenadas local, para as tabelas
        // abaixo se lerem como um desenho.
        let p = |a: f64, b: f64| -> [f64; 2] {
            [
                tip[0] - ux * a * w + nx * b * w,
                tip[1] - uy * a * w + ny * b * w,
            ]
        };

        let path = match self {
            Marker::None => return None,
            // A base do triângulo é uma ARESTA reta perpendicular ao eixo: filetar as duas
            // quinas dela recua os cantos POR CIMA da base, sem mexer na profundidade do
            // encosto. Nenhuma quina aqui é a de junção — todas arredondam.
            Marker::Triangle => filleted(
                &[p(0.0, 0.0), p(LEN, HALF_W), p(LEN, -HALF_W)],
                true,
                &[round; 3],
            ),
            // Duas riscas em "V": um contorno ABERTO que passa pela ponta. Só o BICO é quina
            // (os dois extremos são pontas de traço) — e é ele que o filete arredonda.
            Marker::Open => filleted(
                &[p(LEN, HALF_W), p(0.0, 0.0), p(LEN, -HALF_W)],
                false,
                &[round; 3],
            ),
            // **A quina de TRÁS do losango não arredonda — e isso é uma decisão, não um
            // esquecimento.** Ela é o ponto em que a LINHA ENCOSTA: o recuo do traço é
            // `inset(scale)`, que por contrato depende só do `scale`, então a profundidade da
            // cabeça TEM de ser invariante ao `round`. Um filete ali recua a traseira do
            // losango (0.35·w a `round` 0.25, 1.38·w a `round` 1.0 — medido) enquanto a linha
            // continua parando onde parava: abre-se um VÃO entre o fim do traço e a cabeça,
            // maior que a própria largura da linha. Um losango com o bico de trás vivo é uma
            // troca estética; uma linha partida é um bug. As outras três quinas arredondam.
            Marker::Diamond | Marker::DiamondOpen => filleted(
                &[
                    p(0.0, 0.0),
                    p(LEN, HALF_W),
                    p(2.0 * LEN, 0.0),
                    p(LEN, -HALF_W),
                ],
                true,
                &[round, round, 0.0, round],
            ),
            // Um círculo não tem quina: o `round` não o afeta, e isso é o certo.
            Marker::Circle | Marker::CircleOpen => circle(p(R, 0.0), R * w),
            // Perpendicular à linha, centrada na ponta. Um risco reto: os extremos dele são
            // `cap` de caneta, não quina — o `round` também não o afeta.
            Marker::Bar => filleted(&[p(0.0, HALF_W), p(0.0, -HALF_W)], false, &[round; 2]),
        };
        Some(path)
    }
}

/// O contorno de arestas retas `pts`, com a quina `i` **arredondada** por `rounds[i] ∈ [0, 1]`
/// (fração do filete máximo — ver o módulo). O `round` é POR-VÉRTICE porque nem toda quina de
/// uma ponta é decorativa: a de junção com a linha não pode se mexer (ver o losango, acima).
///
/// Num contorno ABERTO os dois extremos **não são quinas**: são pontas de traço, e saem crus.
/// Só os vértices com vizinho dos dois lados arredondam.
///
/// `round ~ 0` (ou uma quina degenerada) devolve o vértice CRU: com o `round` zerado a saída
/// é byte-idêntica ao polígono de entrada — **a identidade é sagrada** (é o que mantém as
/// pontas afiadas de hoje exatamente como estão).
fn filleted(pts: &[[f64; 2]], closed: bool, rounds: &[f64]) -> VecPath {
    let n = pts.len();
    let mut verts: Vec<VecVertex> = Vec::with_capacity(n * 2);
    for i in 0..n {
        // Os vizinhos GEOMÉTRICOS: num contorno aberto, o extremo não tem um dos dois.
        let prev = (closed || i > 0).then(|| pts[(i + n - 1) % n]);
        let next = (closed || i + 1 < n).then(|| pts[(i + 1) % n]);
        let round = rounds.get(i).copied().unwrap_or(0.0);
        match prev
            .zip(next)
            .and_then(|(a, b)| fillet(a, pts[i], b, round))
        {
            Some((v_in, v_out)) => {
                verts.push(v_in);
                verts.push(v_out);
            }
            None => verts.push(VecVertex::corner(pts[i])),
        }
    }
    VecPath {
        verts,
        closed,
        ..VecPath::default()
    }
}

/// Os DOIS vértices que substituem a quina `v` (entre `a` e `b`) filetada por `round`.
/// `None` quando não há o que arredondar (`round` ~ 0, aresta degenerada, quina colinear) —
/// aí o chamador mantém o vértice cru.
///
/// Mesma construção de [`crate::corners::round_closed_corners`] — recua `t` nas duas arestas e
/// liga por uma cúbica cujos controles apontam para a quina original —, com duas diferenças:
///
/// 1. o parâmetro é a **fração do máximo** (`t = round · meia-aresta-menor`), não um raio: um
///    raio absoluto não teria sentido numa ponta que muda de tamanho com a largura e com o
///    `scale`. **O teto de meia-aresta é o que impede o filete de comer a ponta inteira** e a
///    seta virar pastilha;
/// 2. a trigonometria sai por identidades de meio-ângulo sobre `cos θ = ua·ub` — só `sqrt`,
///    sem `acos`/`tan` (HR-5). No canto reto isto reproduz exatamente o `KAPPA` canônico, e o
///    gate `the_fillet_agrees_with_the_crates_canonical_corner_rounding` prova a concordância
///    com o `corners.rs` numérico, não de boca.
fn fillet(a: [f64; 2], v: [f64; 2], b: [f64; 2], round: f64) -> Option<(VecVertex, VecVertex)> {
    if round <= EPS {
        return None;
    }
    // Versores das duas arestas, SAINDO da quina.
    let (ua, len_a) = unit(a[0] - v[0], a[1] - v[1])?;
    let (ub, len_b) = unit(b[0] - v[0], b[1] - v[1])?;
    // O recuo. **Teto = metade da aresta mais curta**: assim o filete satura em vez de
    // atravessar a aresta que duas quinas vizinhas compartilham (o mesmo clamp do
    // `corners.rs`, aqui promovido a unidade da escala).
    let t = round * 0.5 * len_a.min(len_b);
    if t <= EPS {
        return None;
    }
    // Meio-ângulo interno sem transcendental: cos θ = ua·ub, e daí
    // sin(θ/2) = √((1−cos θ)/2), cos(θ/2) = √((1+cos θ)/2).
    let c = (ua[0] * ub[0] + ua[1] * ub[1]).clamp(-1.0, 1.0);
    let half_sin = ((1.0 - c) * 0.5).max(0.0).sqrt();
    let half_cos = ((1.0 + c) * 0.5).max(0.0).sqrt();
    if half_sin <= EPS || half_cos <= EPS {
        return None; // colinear (nada a arredondar), ou aresta dobrada sobre si
    }
    // Raio do arco que passa pelos dois recuos: r = t · tan(θ/2).
    let r = t * half_sin / half_cos;
    // Comprimento de handle exato da cúbica que segue esse arco: (4/3)·tan(α/4)·r, com
    // α = π − θ o arco descrito. tan(α/4) = (1 − sin(θ/2)) / cos(θ/2) — de novo só √.
    // (No canto reto: r = t e h = 0.5522847…·t, o KAPPA.)
    let h = (4.0 / 3.0) * ((1.0 - half_sin) / half_cos) * r;
    let p_in = [v[0] + ua[0] * t, v[1] + ua[1] * t];
    let p_out = [v[0] + ub[0] * t, v[1] + ub[1] * t];
    // Handles apontam para a quina ORIGINAL (o polo do arco); o outro lado fica nulo (a
    // aresta é reta) — mesma forma do `corners.rs`, quinas independentes e editáveis.
    Some((
        VecVertex {
            anchor: p_in,
            in_handle: p_in,
            out_handle: [p_in[0] - ua[0] * h, p_in[1] - ua[1] * h],
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: p_out,
            in_handle: [p_out[0] - ub[0] * h, p_out[1] - ub[1] * h],
            out_handle: p_out,
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
    ))
}

/// Versor + comprimento; `None` se o vetor é degenerado.
fn unit(x: f64, y: f64) -> Option<([f64; 2], f64)> {
    let len = x.hypot(y);
    (len > EPS).then(|| ([x / len, y / len], len))
}

/// Círculo em quatro cúbicas exatas (o `KAPPA`) — não um polígono.
fn circle(c: [f64; 2], r: f64) -> VecPath {
    let k = crate::corners::KAPPA * r;
    let verts = [
        ([c[0] + r, c[1]], [0.0, -k], [0.0, k]),
        ([c[0], c[1] + r], [k, 0.0], [-k, 0.0]),
        ([c[0] - r, c[1]], [0.0, k], [0.0, -k]),
        ([c[0], c[1] - r], [-k, 0.0], [k, 0.0]),
    ]
    .iter()
    .map(|&(a, i, o)| VecVertex::smooth(a, [a[0] + i[0], a[1] + i[1]], [a[0] + o[0], a[1] + o[1]]))
    .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// **Encurta um caminho aberto** para dar lugar às pontas: recua `start` unidades de mundo
/// no começo e `end` no fim, ao longo da curva.
///
/// Sem isto o traço aparece por dentro de uma ponta vazada, e forma uma bolha na base de uma
/// cheia. É o análogo exato do que um conector faz ao parar na BORDA de uma caixa em vez do
/// centro dela.
///
/// Recua ao longo da **poligonal das âncoras**, não do comprimento de arco exato: o recuo é
/// de poucos múltiplos da largura do traço, e nessa escala a diferença é sub-pixel. Se o
/// caminho for mais curto que os dois recuos somados, devolve `None` — não há linha a
/// desenhar, só as pontas.
#[must_use]
pub fn trim_path(path: &VecPath, start: f64, end: f64) -> Option<VecPath> {
    if path.closed || path.verts.len() < 2 {
        return Some(path.clone()); // um contorno fechado não tem pontas
    }
    if start <= 0.0 && end <= 0.0 {
        return Some(path.clone());
    }
    let mut verts = path.verts.clone();
    if start > 0.0 && !trim_end(&mut verts, start, true) {
        return None;
    }
    if end > 0.0 && !trim_end(&mut verts, end, false) {
        return None;
    }
    Some(VecPath {
        verts,
        ..path.clone()
    })
}

/// Recua `d` a partir de uma das pontas. `false` = o caminho é curto demais.
fn trim_end(verts: &mut Vec<VecVertex>, d: f64, from_start: bool) -> bool {
    let n = verts.len();
    // Caminha do extremo para dentro, consumindo `d`.
    let mut left = d;
    let mut i = 0_usize;
    loop {
        let (a, b) = if from_start {
            (verts[i].anchor, verts[i + 1].anchor)
        } else {
            (verts[n - 1 - i].anchor, verts[n - 2 - i].anchor)
        };
        let seg = (b[0] - a[0]).hypot(b[1] - a[1]);
        if seg >= left {
            // O ponto novo cai DENTRO deste segmento.
            let t = if seg > 1e-12 { left / seg } else { 0.0 };
            let np = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            let idx = if from_start { i } else { n - 1 - i };
            // A âncora anda; os handles a acompanham (o recuo é curto, então a curva
            // praticamente não muda de forma — e uma quina continua quina).
            let v = &mut verts[idx];
            let (sx, sy) = (np[0] - v.anchor[0], np[1] - v.anchor[1]);
            v.anchor = np;
            v.in_handle = [v.in_handle[0] + sx, v.in_handle[1] + sy];
            v.out_handle = [v.out_handle[0] + sx, v.out_handle[1] + sy];
            // Descarta os vértices que ficaram para fora.
            if from_start {
                verts.drain(0..i);
            } else {
                verts.truncate(n - i);
            }
            return true;
        }
        left -= seg;
        i += 1;
        if i + 1 >= n {
            return false; // consumiu o caminho inteiro
        }
    }
}

/// A tangente de saída num extremo do caminho, apontando **para fora** — a direção em que a
/// ponta olha. `None` se não há caminho (ou se os dois pontos coincidem).
#[must_use]
pub fn end_tangent(path: &VecPath, at_start: bool) -> Option<([f64; 2], [f64; 2])> {
    let n = path.verts.len();
    if n < 2 {
        return None;
    }
    let (v, next) = if at_start {
        (&path.verts[0], &path.verts[1])
    } else {
        (&path.verts[n - 1], &path.verts[n - 2])
    };
    // A tangente da CURVA no extremo é a direção do handle — não a corda até a âncora
    // seguinte. Numa curva bem encurvada as duas divergem visivelmente, e a ponta sairia
    // torta. O handle degenerado (quina) cai de volta na corda.
    let h = if at_start { v.out_handle } else { v.in_handle };
    let (hx, hy) = (h[0] - v.anchor[0], h[1] - v.anchor[1]);
    let (dx, dy) = if hx.hypot(hy) > 1e-9 {
        (-hx, -hy) // para FORA = contra o handle, que aponta para dentro do caminho
    } else {
        (v.anchor[0] - next.anchor[0], v.anchor[1] - next.anchor[1])
    };
    let len = dx.hypot(dy);
    if len < 1e-12 {
        return None;
    }
    Some((v.anchor, [dx / len, dy / len]))
}

/// **A ponta de um extremo do traço**, na geometria do próprio caminho.
///
/// Ela NÃO faz parte do `VecPath` — é construída sob demanda a partir do caminho + do
/// [`StrokeSpec`]. E é por isso que ela mora aqui, e não no renderer: **quem desenha e quem
/// agarra têm de ler a mesma função.**
///
/// Enquanto ela viveu só no render, o hit-test do canvas não a enxergava — e a cabeça é
/// justamente a parte GORDA da seta, a que o olho mira e o mouse persegue. Clicar no
/// triângulo não selecionava nada, e a única área clicável era o fio da linha. Foi a queixa
/// do Enio ("a área da seta é fina"), e a causa não era o raio de captura: era uma metade do
/// desenho que simplesmente não existia para o mouse.
#[must_use]
pub fn stroke_head(path: &VecPath, s: &StrokeSpec, at_start: bool) -> Option<(Marker, VecPath)> {
    // Contorno fechado não tem pontas (não há extremo onde pô-las).
    if path.closed {
        return None;
    }
    let marker = if at_start {
        s.marker_start
    } else {
        s.marker_end
    };
    if marker == Marker::None {
        return None;
    }
    let (tip, dir) = end_tangent(path, at_start)?;
    let geo = marker.build(tip, dir, s.width, s.marker_scale, s.marker_round)?;
    Some((marker, geo))
}

#[cfg(test)]
#[path = "marker_tests.rs"]
mod tests;
