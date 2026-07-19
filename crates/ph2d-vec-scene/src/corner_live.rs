//! Raio de quina **vivo** sobre um caminho AUTORADO qualquer — módulo irmão de
//! [`crate::corners`] (que arredonda uma POLILINHA e é o gerador das Live Shapes).
//!
//! A diferença é a razão deste módulo existir. O [`crate::corners`] recebe pontos
//! (`&[[f64; 2]]`) e **produz** uma forma: é o que o polígono, a estrela e o round-rect
//! chamam para nascer. Aqui a entrada é um caminho já desenhado — com **curvas** — e a
//! saída é o mesmo caminho com as quinas arredondadas. É o "Live Corners" do
//! Illustrator / o Corner Tool do Affinity: o raio é uma **propriedade da âncora**, a
//! quina afiada continua sendo o que o documento guarda, e o arredondamento é
//! re-cozido a partir dela.
//!
//! ## A construção, e por que ela é a MESMA de sempre
//!
//! Numa quina entre duas RETAS, o [`crate::corners`] recua `t = r / tan(θ/2)` ao longo
//! das duas arestas e liga os dois pontos por um arco de handle `(4/3)·tan(α/4)·r`.
//! Aqui a mesma conta é feita com dois generalizações, e só duas:
//!
//! 1. **A direção da aresta** vira a **tangente da curva** na quina (com a cascata de
//!    handle degenerado — um handle nulo cai no controle anterior, como no `cubic_bezier`
//!    do Chromium). Numa reta a tangente É a aresta.
//! 2. **O recuo** vira uma distância ao longo da curva, resolvida por bisseção no
//!    parâmetro. Numa reta o parâmetro é proporcional à distância.
//!
//! O handle da cúbica de ligação é `k · d`, com `k = (4/3)·tan(α/4)` e `d` = distância
//! do ponto de recuo até a **interseção das duas tangentes**. Numa quina reta a
//! interseção é o vértice original e `d = t`, então `k·d = (4/3)·tan(α/4)·t` — que é
//! **exatamente** o `h` do [`crate::corners`]. **A identidade é sagrada:** o caso reto
//! sai byte-a-byte igual, e uma quina que curva de verdade ganha um arco circular
//! exato (é o mesmo arco). Quando um dos lados é curvo, a ligação é um blend
//! **tangente (G1)** — que é o que Illustrator e Affinity de fato entregam; o filete
//! circular exato entre duas cúbicas é um problema de curva de grau 10, e ninguém o
//! resolve em forma fechada.
//!
//! ## A saturação (a lição do Bug #1)
//!
//! Duas quinas vizinhas dividem o segmento entre elas, e as duas querem recuar nele. O
//! reflexo é clampar cada recuo contra o outro — e é aí que mora o pânico: um piso e um
//! teto derivados da MESMA conta são iguais em ℝ e se **cruzam por 1 ulp** em `f64`
//! (`docs/Vector Module/BUGS_vector.md` #1). Aqui não há piso nem teto: quando os dois
//! recuos se sobrepõem, eles colapsam num **único número** (o ponto de encontro), o
//! segmento vira um ponto, e as duas quinas se encostam. É o caso saturado legítimo —
//! não um erro a evitar —, e ele **não tem fronteira** para inverter.

use crate::{VecPath, VecVertex, VertexKind};

/// Abaixo disto (unidades de mundo) um raio / recuo / vetor é tratado como zero.
const EPS: f64 = 1e-9;

/// O raio da bolinha da alça de raio, em PIXELS de tela.
///
/// **Mora aqui, e não na crate de desenho nem na de edição, porque as DUAS o leem.** Quem
/// desenha o círculo e quem o captura têm de usar o mesmo número: se divergirem, o usuário
/// clica exatamente no meio da bolinha e não pega nada — e a tela está certa, que é o
/// sintoma mais enlouquecedor que existe. (O `connector.rs` já escreveu essa lição; esta é
/// a mesma, e a casa comum das duas pontas é esta crate.)
///
/// Em pixels e não em mundo: uma alça que encolhesse com o zoom-out ficaria impegável
/// exatamente quando a forma fica grande.
pub const CORNER_HANDLE_R_PX: f64 = 6.0;

/// Onde a alça de uma quina AFIADA estaciona, em pixels ao longo da bissetriz. Longe o
/// bastante da âncora para não brigar com o hit-test dela, perto o bastante para ler como
/// "esta alça é desta quina".
pub const CORNER_HANDLE_PARK_PX: f64 = 14.0;

/// Uma alça de raio pronta para desenhar: onde ela está **no mundo**, e se a quina já
/// está arredondada (bolinha cheia) ou ainda afiada (vazada, só estacionada).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CornerHandleView {
    /// O centro da bolinha, em coordenadas de MUNDO.
    pub pos: [f64; 2],
    /// A âncora da quina, no mundo — um fio fino liga uma à outra, e é ele que diz a
    /// QUEM a bolinha pertence quando várias quinas estão próximas.
    pub anchor: [f64; 2],
    /// A quina já tem raio? (decide o preenchimento da bolinha)
    pub rounded: bool,
    /// O índice PLANO do vértice (o mesmo espaço de índice do hit-test e da seleção).
    pub vert: usize,
}

/// Uma cúbica: os quatro pontos de controle, na ordem do traçado.
type Cubic = [[f64; 2]; 4];

impl VecVertex {
    /// **O TAMANHO da quina** — o recuo pedido, sempre `>= 0`. O recuo e a alça de raio
    /// trabalham com esta magnitude; o SINAL (o estilo) é assunto do [`Self::is_chamfer`].
    #[must_use]
    pub fn corner_size(&self) -> f64 {
        self.corner_radius.abs()
    }

    /// **A quina é um CHANFRO** (reta) em vez de arredondada? Convenção: `corner_radius < 0`.
    #[must_use]
    pub fn is_chamfer(&self) -> bool {
        self.corner_radius < 0.0
    }

    /// Muda o TAMANHO da quina preservando o estilo (o sinal). É o que o arrasto da alça usa —
    /// arrastar redimensiona, nunca converte arredondado em chanfro por acidente.
    pub fn set_corner_size(&mut self, size: f64) {
        self.corner_radius = if self.is_chamfer() { -size } else { size };
    }

    /// Muda o ESTILO (chanfro/arredondado) preservando o tamanho. É o que o toggle Chamfer usa.
    pub fn set_chamfer(&mut self, chamfer: bool) {
        let size = self.corner_size();
        self.corner_radius = if chamfer { -size } else { size };
    }
}

/// Arredonda as quinas de um caminho autorado, cada uma pelo `corner_radius` do seu
/// vértice. `None` quando **não há nada a arredondar** — e aí o chamador mantém a fonte,
/// byte-a-byte (é o que garante que um caminho sem raio nenhum não passe por conta de
/// ponto flutuante alguma).
///
/// Num caminho ABERTO as pontas não têm quina (falta um dos dois lados), então o raio
/// delas é ignorado. Os vértices que saem daqui têm `corner_radius = 0`: o cozimento já
/// aconteceu, não há nada pendente — o que também faz cozinhar duas vezes ser o mesmo
/// que cozinhar uma. Ver o módulo para a construção e a saturação.
#[must_use]
pub fn round_authored_corners(verts: &[VecVertex], closed: bool) -> Option<Vec<VecVertex>> {
    let n = verts.len();
    if n < 3 || !verts.iter().any(|v| v.corner_size() > EPS) {
        return None; // nada a arredondar → a fonte é a saída
    }
    // Os segmentos, na convenção do caminho: `seg[i]` vai do vértice `i` ao `i + 1`
    // (fechado dá a volta). Um caminho aberto tem um segmento a menos.
    let seg_count = if closed { n } else { n - 1 };
    let segs: Vec<Cubic> = (0..seg_count).map(|i| segment(verts, i, n)).collect();

    // O RECUO pedido por cada quina, em distância, para cada uma das duas pontas do
    // segmento que ela toca. `None` = a quina não arredonda (raio ~0, colinear, ponta
    // de caminho aberto) e o vértice sai cru.
    let mut want: Vec<Option<f64>> = vec![None; n];
    for (i, w) in want.iter_mut().enumerate() {
        // A magnitude decide o RECUO (o mesmo para arredondado e chanfro); o sinal só escolhe
        // o traçado da ligação (arco vs reta), lá embaixo na emissão.
        *w = corner_setback(&segs, i, n, closed, verts[i].corner_size());
    }

    // Recuo → parâmetro, por ponta de segmento. `u0[i]` corta o COMEÇO de `seg[i]` (a
    // quina do vértice `i`); `u1[i]` corta o FIM (a quina do vértice `i + 1`).
    let mut u0 = vec![0.0_f64; seg_count];
    let mut u1 = vec![1.0_f64; seg_count];
    for i in 0..seg_count {
        if let Some(t) = want[i] {
            u0[i] = param_at_dist_from_start(&segs[i], t);
        }
        if let Some(t) = want[(i + 1) % n] {
            u1[i] = param_at_dist_from_end(&segs[i], t);
        }
        (u0[i], u1[i]) = resolve_trims(u0[i], u1[i]);
    }

    // Cada segmento vira o seu PEDAÇO entre os dois cortes (de Casteljau exato). O
    // pedaço carrega os controles novos — é por isso que até um vértice NÃO arredondado
    // pode ter os handles trocados: o vizinho encurtou o segmento que eles dividem.
    let subs: Vec<Cubic> = (0..seg_count)
        .map(|i| sub_cubic(&segs[i], u0[i], u1[i]))
        .collect();

    let mut out: Vec<VecVertex> = Vec::with_capacity(n * 2);
    for i in 0..n {
        // O segmento que CHEGA no vértice `i` e o que SAI dele. Num caminho aberto as
        // pontas têm só um dos dois.
        let inc = incoming(i, n, closed).map(|s| &subs[s]);
        let out_seg = outgoing(i, n, closed, seg_count).map(|s| &subs[s]);
        match (want[i], inc, out_seg) {
            // Quina com recuo: o vértice único vira DOIS, nos MESMOS pontos de recuo `p_in`/
            // `p_out`. O estilo escolhe só a LIGAÇÃO entre eles — arco (arredondado) ou reta
            // (chanfro). O recuo, os handles de chegada/saída e o resto são idênticos.
            (Some(_), Some(a), Some(b)) => {
                let (p_in, p_out) = (a[3], b[0]);
                let (out1, in2) = if verts[i].is_chamfer() {
                    // Chanfro: a reta na forma canônica (⅓, ⅔) — afim em `t`, sem curvatura,
                    // e a mesma forma que o resto do motor usa para um segmento reto.
                    let third = [(p_out[0] - p_in[0]) / 3.0, (p_out[1] - p_in[1]) / 3.0];
                    (
                        [p_in[0] + third[0], p_in[1] + third[1]],
                        [p_out[0] - third[0], p_out[1] - third[1]],
                    )
                } else {
                    // Arredondado: o arco de handle `(4/3)·tan(α/4)·r` de sempre.
                    let (t_in, t_out) = (tangent_at_end(a), tangent_at_start(b));
                    fillet_handles(p_in, t_in, p_out, t_out)
                };
                out.push(VecVertex {
                    anchor: p_in,
                    in_handle: a[2],
                    out_handle: out1,
                    kind: VertexKind::Corner,
                    corner_radius: 0.0,
                });
                out.push(VecVertex {
                    anchor: p_out,
                    in_handle: in2,
                    out_handle: b[1],
                    kind: VertexKind::Corner,
                    corner_radius: 0.0,
                });
            }
            // Vértice cru: a âncora e o `kind` são os da FONTE; os handles saem dos
            // pedaços (que o vizinho pode ter encurtado). Sem vizinho, o handle da fonte.
            _ => {
                let v = &verts[i];
                out.push(VecVertex {
                    anchor: v.anchor,
                    in_handle: inc.map_or(v.in_handle, |a| a[2]),
                    out_handle: out_seg.map_or(v.out_handle, |b| b[1]),
                    kind: v.kind,
                    corner_radius: 0.0,
                });
            }
        }
    }
    Some(out)
}

impl VecPath {
    /// A geometria **COZIDA** deste path: a que se renderiza, se aponta e se corta —
    /// com as quinas já arredondadas pelos `corner_radius` dos vértices.
    ///
    /// É o funil. O que o documento GUARDA (`self.verts`) é a quina afiada + o raio; o
    /// que o mundo CONSOME é isto. Quem edita (caneta, seleção, os overlays de âncora)
    /// continua vendo a fonte — senão o usuário veria dois vértices onde autorou um.
    ///
    /// **Sem raio nenhum e sem efeito ativo, devolve a própria fonte emprestada**
    /// (`Cow::Borrowed`): custo zero, byte-idêntico, nem uma alocação. Como todo path nasce
    /// sem raio e com a pilha vazia, um documento que nunca usou nenhuma das duas features
    /// passa por aqui sem sentir.
    ///
    /// A quina é o **estágio zero** e a pilha de efeitos ([`crate::effect`], ADR-0132) corre
    /// depois dela — nessa ordem, sempre: o raio é estado do vértice AUTORADO, e todo efeito
    /// a jusante resampleia.
    #[must_use]
    pub fn cooked(&self) -> std::borrow::Cow<'_, VecPath> {
        let rounds = self.verts_all().any(|v| v.corner_size() > EPS);
        if !rounds {
            // Sem quina a arredondar: a pilha (se houver) come a fonte diretamente.
            return match crate::effect::run_stack(self, &self.effects) {
                Some(p) => std::borrow::Cow::Owned(p),
                None => std::borrow::Cow::Borrowed(self),
            };
        }
        let mut out = self.clone();
        if let Some(v) = round_authored_corners(&self.verts, self.closed) {
            out.verts = v;
        }
        for (i, s) in self.subpaths.iter().enumerate() {
            if let Some(v) = round_authored_corners(&s.verts, s.closed) {
                out.subpaths[i].verts = v;
            }
        }
        if let Some(p) = crate::effect::run_stack(&out, &self.effects) {
            out = p;
        }
        std::borrow::Cow::Owned(out)
    }
}

/// O recuo (em distância) que a quina do vértice `i` pede, ou `None` se ela não
/// arredonda: raio ~0, ponta de caminho aberto, ou quina colinear (não há o que
/// arredondar) / dobrada sobre si.
///
/// É também o predicado de "aqui EXISTE uma quina" — a alça de raio do canvas usa este
/// mesmo teste para decidir onde aparecer, então a alça nunca promete um arredondamento
/// que o cozimento vai recusar.
fn corner_setback(segs: &[Cubic], i: usize, n: usize, closed: bool, r: f64) -> Option<f64> {
    if r <= EPS {
        return None;
    }
    let seg_count = segs.len();
    let a = &segs[incoming(i, n, closed)?];
    let b = &segs[outgoing(i, n, closed, seg_count)?];
    // Versores SAINDO da quina, os dois — exatamente como o `crate::corners` faz com as
    // arestas retas. Para trás pela curva que chega, para frente pela que sai.
    let ua = neg(tangent_at_end(a));
    let ub = tangent_at_start(b);
    if ua == [0.0, 0.0] || ub == [0.0, 0.0] {
        return None;
    }
    let theta = dot(ua, ub).clamp(-1.0, 1.0).acos();
    let half = theta * 0.5;
    if half <= EPS || (std::f64::consts::PI - theta) <= EPS {
        return None; // colinear, ou dobrada sobre si
    }
    // O MESMO teto que a alça enxerga (`CornerFrame::max_setback`) — cozimento e gizmo
    // têm de ser a mesma função, ou a alça anda com a forma parada.
    let t = (r / half.tan()).min(0.5 * chord(a).min(chord(b)));
    (t > EPS && t.is_finite()).then_some(t)
}

/// Existe quina arredondável no vértice `i` deste contorno? (o predicado acima, sem o
/// raio — é o que decide se a **alça** de raio aparece). `None` = ponta de caminho
/// aberto, ou quina colinear.
#[must_use]
pub fn corner_at(verts: &[VecVertex], closed: bool, i: usize) -> Option<CornerFrame> {
    let n = verts.len();
    if n < 3 || i >= n {
        return None;
    }
    let seg_count = if closed { n } else { n - 1 };
    let segs: Vec<Cubic> = (0..seg_count).map(|s| segment(verts, s, n)).collect();
    let a = &segs[incoming(i, n, closed)?];
    let b = &segs[outgoing(i, n, closed, seg_count)?];
    let ua = neg(tangent_at_end(a));
    let ub = tangent_at_start(b);
    let (ua, ub) = (unit(ua)?, unit(ub)?);
    let theta = dot(ua, ub).clamp(-1.0, 1.0).acos();
    let half = theta * 0.5;
    if half <= EPS || (std::f64::consts::PI - theta) <= EPS {
        return None;
    }
    // A bissetriz aponta para DENTRO da quina (a média dos dois versores que saem dela).
    let bisector = unit([ua[0] + ub[0], ua[1] + ub[1]])?;
    // O recuo não pode passar de METADE do lado mais curto: além disso a quina invadiria
    // a vizinha e o cozimento satura (as duas se encostam). É o MESMO teto do
    // `crate::corners`, medido pela corda de cada segmento.
    let max_setback = 0.5 * chord(a).min(chord(b));
    Some(CornerFrame {
        anchor: verts[i].anchor,
        bisector,
        // A alça mede a MAGNITUDE (um chanfro recua igual a um arredondado do mesmo tamanho);
        // o estilo não muda onde a bolinha pousa.
        setback: verts[i].corner_size() / half.tan(),
        max_setback,
        half_angle: half,
    })
}

/// A quina vista pela ALÇA: onde ela ancora, para onde ela corre, o quanto o raio atual
/// já recuou, e até onde ela pode ir.
///
/// Converter alça ⟷ raio é `r = setback · tan(θ/2)`, e é **a mesma conta do cozimento na
/// direção inversa** — de propósito. Uma coordenada derivada que é SEMEADA por uma
/// fórmula e LIDA por outra já custou três bugs neste repo (a memória
/// `feedback_derived_coordinate_seed_must_match_sample`): o arrasto e o cozimento
/// precisam ser a mesma função, senão a alça foge do cursor.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CornerFrame {
    /// A âncora da quina (espaço LOCAL do path, como tudo no documento).
    pub anchor: [f64; 2],
    /// Versor da bissetriz, apontando para DENTRO da quina — a direção em que a alça
    /// corre e em que o arredondamento come.
    pub bisector: [f64; 2],
    /// O recuo que o `corner_radius` atual produz (0 = quina afiada).
    pub setback: f64,
    /// O recuo máximo que a geometria absorve (metade do lado mais curto). Passar disso
    /// não arredonda mais — a alça **para aqui**, senão ela continuaria andando com a
    /// forma parada, que é o sintoma mais enlouquecedor que existe.
    pub max_setback: f64,
    /// Metade do ângulo interno da quina. `raio = setback · tan(half_angle)`.
    pub half_angle: f64,
}

impl CornerFrame {
    /// O raio que um recuo `setback` produz nesta quina — o inverso exato do cozimento,
    /// já clampado ao que a geometria absorve. É o que o arrasto da alça usa.
    ///
    /// Abaixo do EPS o resultado é **exatamente zero**, e não a poeira de ponto flutuante
    /// que a ida-e-volta `raio → recuo → raio` deixa (o `tan(π/4)` do `f64` não é 1). Uma
    /// quina "afiada" com raio `4e-16` cozinharia igual — o motor a trata como zero — mas
    /// ficaria guardada no documento, apareceria num diff de save e faria um
    /// `raio == 0.0` mentir para quem viesse depois. Afiada é afiada.
    #[must_use]
    pub fn radius_for_setback(&self, setback: f64) -> f64 {
        let t = setback.clamp(0.0, self.max_setback);
        let r = (t * self.half_angle.tan()).max(0.0);
        if r <= EPS { 0.0 } else { r }
    }

    /// O ponto da bissetriz a `setback` da âncora (espaço local). É **cru**: não clampa.
    ///
    /// O clamp é do RAIO (`radius_for_setback`), não do ponto — e essa distinção é a
    /// diferença entre a alça ficar sob o dedo e escorregar dele. A alça é desenhada a
    /// `park + setback`: o estacionamento é um deslocamento CONSTANTE, presente tanto no
    /// desenho quanto na leitura do arrasto (`setback = projeção − park`), e é isso que
    /// faz a bolinha pousar exatamente no cursor. Se este ponto clampasse, a bolinha
    /// pararia antes do dedo assim que o raio saturasse.
    #[must_use]
    pub fn handle_at(&self, setback: f64) -> [f64; 2] {
        let t = setback.max(0.0);
        [
            self.anchor[0] + self.bisector[0] * t,
            self.anchor[1] + self.bisector[1] * t,
        ]
    }
}

/// A corda da cúbica (ponta a ponta). É a medida de "tamanho do lado" que o teto do
/// recuo usa — conservadora numa curva (o arco é sempre ≥ a corda), que é o lado certo
/// para errar num clamp.
fn chord(c: &Cubic) -> f64 {
    hypot(sub(c[3], c[0]))
}

/// Os dois controles da cúbica que liga `p_in` (chegando na direção `t_in`) a `p_out`
/// (saindo na direção `t_out`), tangente aos dois lados.
///
/// Handle = `k · d`, com `k = (4/3)·tan(α/4)` (α = o quanto a quina vira) e `d` = a
/// distância até a INTERSEÇÃO das duas tangentes. Numa quina reta a interseção é o
/// vértice original e `d` = o recuo, o que reproduz o arco de [`crate::corners`] —
/// exato, não aproximado.
///
/// Quando as tangentes são quase paralelas não há interseção utilizável (`d` iria a
/// infinito e `0 · ∞` é **`NaN`** — e um `NaN` é pior que um pânico: ele contamina a
/// bbox e a forma "some" três telas depois). Aí a ligação cai no blend padrão de dois
/// pontos com tangentes dadas: handle = um terço da corda.
fn fillet_handles(
    p_in: [f64; 2],
    t_in: [f64; 2],
    p_out: [f64; 2],
    t_out: [f64; 2],
) -> ([f64; 2], [f64; 2]) {
    let alpha = dot(t_in, t_out).clamp(-1.0, 1.0).acos();
    let k = (4.0 / 3.0) * (alpha * 0.25).tan();
    // Interseção das retas `p_in + s·t_in` e `p_out − w·t_out`. Resolvendo
    // `s·t_in + w·t_out = d` por Cramer (`a × b` = `a.x·b.y − a.y·b.x`):
    //   s = (d × t_out) / (t_in × t_out)      w = (d × t_in) / (t_out × t_in)
    // Note que os DENOMINADORES são opostos — `t_out × t_in = −(t_in × t_out)`. Usar o
    // mesmo para os dois dá um `w` NEGATIVO numa quina perfeitamente boa, o guard de
    // "à frente" a reprova, e todo filete cai no fallback: o arco vira um blend frouxo,
    // e a quina reta deixa de bater com o `crate::corners`.
    let cross = t_in[0] * t_out[1] - t_in[1] * t_out[0];
    let d = sub(p_out, p_in);
    let s_in = (d[0] * t_out[1] - d[1] * t_out[0]) / cross;
    let s_out = (d[1] * t_in[0] - d[0] * t_in[1]) / cross;
    // A interseção só serve se ela existe (tangentes não-paralelas), é finita, e fica
    // À FRENTE dos dois pontos (senão o "arco" apontaria para trás e cruzaria a forma).
    let usable =
        cross.abs() > EPS && s_in.is_finite() && s_out.is_finite() && s_in > 0.0 && s_out > 0.0;
    let (h_in, h_out) = if usable {
        (k * s_in, k * s_out)
    } else {
        let third = hypot(d) / 3.0;
        (third, third)
    };
    (
        [p_in[0] + t_in[0] * h_in, p_in[1] + t_in[1] * h_in],
        [p_out[0] - t_out[0] * h_out, p_out[1] - t_out[1] * h_out],
    )
}

/// A cúbica do segmento `i` (do vértice `i` ao `i + 1`, dando a volta no fechado).
pub(crate) fn segment(verts: &[VecVertex], i: usize, n: usize) -> Cubic {
    let a = &verts[i];
    let b = &verts[(i + 1) % n];
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

/// O índice do segmento que CHEGA no vértice `i` (`None` = ponta inicial de um aberto).
fn incoming(i: usize, n: usize, closed: bool) -> Option<usize> {
    if closed {
        Some((i + n - 1) % n)
    } else {
        i.checked_sub(1)
    }
}

/// O índice do segmento que SAI do vértice `i` (`None` = ponta final de um aberto).
fn outgoing(i: usize, n: usize, closed: bool, seg_count: usize) -> Option<usize> {
    if closed {
        Some(i % n)
    } else {
        (i < seg_count).then_some(i)
    }
}

/// Versor da tangente no COMEÇO da cúbica, com a cascata de controle degenerado: um
/// handle nulo cai no controle seguinte, e dois handles nulos caem na corda (é a mesma
/// cascata do `cubic_bezier.cc` do Chromium — sem ela, um handle nulo dá tangente zero
/// e a quina não sabe para onde apontar).
fn tangent_at_start(c: &Cubic) -> [f64; 2] {
    for q in [c[1], c[2], c[3]] {
        if let Some(u) = unit(sub(q, c[0])) {
            return u;
        }
    }
    [0.0, 0.0]
}

/// Versor da tangente no FIM da cúbica (direção do traçado), mesma cascata.
fn tangent_at_end(c: &Cubic) -> [f64; 2] {
    for q in [c[2], c[1], c[0]] {
        if let Some(u) = unit(sub(c[3], q)) {
            return u;
        }
    }
    [0.0, 0.0]
}

/// O parâmetro em que a cúbica se afasta `t` (distância reta) do PONTO INICIAL. Satura
/// em `1.0` quando a cúbica inteira é mais curta que `t`.
fn param_at_dist_from_start(c: &Cubic, t: f64) -> f64 {
    param_at_dist(c, t, 0.0)
}

/// Idem, medindo a partir do PONTO FINAL (satura em `0.0`).
fn param_at_dist_from_end(c: &Cubic, t: f64) -> f64 {
    param_at_dist(c, t, 1.0)
}

/// Bisseção do parâmetro em que a cúbica dista `t` da ponta `from` (0 = início, 1 =
/// fim). A distância cresce monotonicamente ao sair da ponta em toda curva sã; a
/// bisseção devolve SEMPRE um valor em `[0, 1]`, então nem uma curva patológica produz
/// coisa fora da faixa (e o colapso `u0 >= u1` do chamador cobre o resto).
fn param_at_dist(c: &Cubic, t: f64, from: f64) -> f64 {
    let origin = eval(c, from);
    if hypot(sub(eval(c, 1.0 - from), origin)) <= t {
        return 1.0 - from; // a cúbica toda cabe no recuo: satura na outra ponta
    }
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64); // fração do caminho, saindo de `from`
    for _ in 0..60 {
        let mid = (lo + hi) * 0.5;
        let u = from + (1.0 - 2.0 * from) * mid;
        if hypot(sub(eval(c, u), origin)) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mid = (lo + hi) * 0.5;
    (from + (1.0 - 2.0 * from) * mid).clamp(0.0, 1.0)
}

/// Os dois cortes de um segmento, **garantidamente em ordem**: o do começo nunca passa o
/// do fim. Pós-condição: `saída.0 <= saída.1`.
///
/// Quando as duas quinas vizinhas pedem mais do que o segmento tem, os cortes se
/// encontram — e o reflexo é clampar um contra o outro. É aí que mora o pânico do Bug #1
/// (`docs/Vector Module/BUGS_vector.md`): um piso e um teto derivados da MESMA conta são
/// iguais em ℝ e se ordenam de forma imprevisível em `f64`. Aqui não há piso nem teto —
/// os dois **colapsam num único número**, o segmento vira um ponto, e as quinas se
/// encostam. Um só número não tem fronteira para 1 ulp inverter.
///
/// Na prática o clamp de meia-corda (`corner_setback`) já faz os dois PARAREM no meio em
/// vez de se atropelarem, então esta função quase nunca colapsa nada. O "quase" é o
/// ponto: os dois recuos saem de **bisseções que partem de pontas opostas**, e nada
/// obriga as duas a devolverem o mesmo bit no meio exato. Sem esta rede, esse ulp
/// inverteria o segmento.
fn resolve_trims(u0: f64, u1: f64) -> (f64, f64) {
    if u0 >= u1 {
        let m = ((u0 + u1) * 0.5).clamp(0.0, 1.0);
        return (m, m);
    }
    (u0, u1)
}

/// Uma aresta RETA, na convenção deste modelo: os dois handles coincidem com as âncoras
/// (é o que [`VecVertex::corner`] produz). Igualdade EXATA de propósito — é uma convenção
/// de representação, não uma medida de geometria, e um afim leva handle nulo em handle
/// nulo.
fn is_straight(c: &Cubic) -> bool {
    c[1] == c[0] && c[2] == c[3]
}

/// A cúbica de `u0` a `u1` (de Casteljau, exato). `u0 == 0 && u1 == 1` devolve a cúbica
/// **verbatim** — sem isso um segmento intocado seria reconstruído por ponto flutuante e
/// deixaria de ser byte-idêntico à fonte.
///
/// **A aresta reta é cortada como RETA**, e não por de Casteljau. Ela não é uma reta
/// uniforme: `[(0,10), (0,10), (0,0), (0,0)]` é um *smoothstep* — geometricamente o
/// segmento, mas com velocidade zero nas pontas. De Casteljau nela devolve pontos de
/// controle **no meio da aresta** em vez de nulos. A curva traçada seria idêntica, mas a
/// REPRESENTAÇÃO deixaria de ser a convenção de reta — e é exatamente ela que o `to_bez`
/// da booleana testa para emitir `line_to`. Sem este caso, arredondar uma quina faria a
/// booleana devolver as quinas vizinhas como `Smooth` com handles que ninguém pediu (o
/// bug que aquele código já documenta e contorna).
fn sub_cubic(c: &Cubic, u0: f64, u1: f64) -> Cubic {
    if u0 <= 0.0 && u1 >= 1.0 {
        return *c;
    }
    if is_straight(c) {
        let (a, b) = (eval(c, u0), eval(c, u1));
        return [a, a, b, b];
    }
    // Corta o fim primeiro (o parâmetro de `u0` não muda com isso), depois o começo.
    let head = split(c, u1).0;
    if u0 <= 0.0 {
        return head;
    }
    // `u0` re-parametrizado dentro do pedaço que sobrou.
    let s = if u1 > EPS { u0 / u1 } else { 0.0 };
    split(&head, s.clamp(0.0, 1.0)).1
}

/// Divide a cúbica em `u` (de Casteljau): (o pedaço `[0, u]`, o pedaço `[u, 1]`).
fn split(c: &Cubic, u: f64) -> (Cubic, Cubic) {
    let l = |p: [f64; 2], q: [f64; 2]| [p[0] + (q[0] - p[0]) * u, p[1] + (q[1] - p[1]) * u];
    let (a, b, cc) = (l(c[0], c[1]), l(c[1], c[2]), l(c[2], c[3]));
    let (d, e) = (l(a, b), l(b, cc));
    let f = l(d, e);
    ([c[0], a, d, f], [f, e, cc, c[3]])
}

/// A cúbica em `u` (Bernstein direto — a mesma conta do `boundary::cubic`).
fn eval(c: &Cubic, u: f64) -> [f64; 2] {
    let v = 1.0 - u;
    let (b0, b1) = (v * v * v, 3.0 * v * v * u);
    let (b2, b3) = (3.0 * v * u * u, u * u * u);
    [
        c[0][0] * b0 + c[1][0] * b1 + c[2][0] * b2 + c[3][0] * b3,
        c[0][1] * b0 + c[1][1] * b1 + c[2][1] * b2 + c[3][1] * b3,
    ]
}

fn sub(p: [f64; 2], q: [f64; 2]) -> [f64; 2] {
    [p[0] - q[0], p[1] - q[1]]
}

fn neg(p: [f64; 2]) -> [f64; 2] {
    [-p[0], -p[1]]
}

fn dot(p: [f64; 2], q: [f64; 2]) -> f64 {
    p[0] * q[0] + p[1] * q[1]
}

fn hypot(p: [f64; 2]) -> f64 {
    p[0].hypot(p[1])
}

fn unit(v: [f64; 2]) -> Option<[f64; 2]> {
    let len = hypot(v);
    (len > EPS).then(|| [v[0] / len, v[1] / len])
}

#[cfg(test)]
#[path = "corner_live_tests.rs"]
mod corner_live_tests;
