//! ⭐⭐⭐ **O QUE É UMA CURVA AQUI** — o irmão do *«o que é um ponto aqui»*, e a 2.ª saída da F26.
//!
//! # ⛔⛔⛔ O defeito que este módulo existe para curar, escrito por outra crate há meses
//!
//! O cabeçalho da [`ph2d_vec_envelope`] di-lo em duas linhas:
//!
//! > *«Só transformações **afins** comutam com a avaliação de Bézier … logo isto está errado:
//! > `for v in verts { v.anchor = warp(v.anchor) }` … A curva resultante **não é a imagem** da curva
//! > original. E erra de um jeito traiçoeiro: sob um mapa suave ela acerta a verdade em `t=0` e
//! > `t=1` **exatamente**, e no interior **nunca**.»*
//!
//! **A pele é um mapa não-afim** (os pesos variam com o ponto) e o [`crate::aplica`] deforma os
//! pontos de controlo — ⇒ ele é, à letra, o `for v in verts` daquele aviso. As consequências foram
//! as duas queixas do dono desta jornada:
//!
//! 1. *«pintar peso ENTRE os vértices não faz nada»* — porque só os NÓS são amostrados. **Medido:**
//!    uma mancha no meio de uma aresta move a arte `0,000000` pelo caminho dos pontos de controlo e
//!    `0,242375` por este.
//! 2. *«o ponto criado deforma a malha»* (F28) — o corte revelava o erro de interior que já lá
//!    estava. A F28 curou-o por compensação; aqui ele **deixa de existir**.
//!
//! # ⭐⭐ A lei: o peso varia ao longo do `t`, e nos NÓS ela é a de hoje
//!
//! Num segmento de `a` para `b`, o peso do ponto `C(t)` é a **mistura** das linhas dos dois nós,
//! `lerp(ra, rb, t)`, com as manchas pintadas somadas **no ponto** — a mesma lei que a F28 já usa
//! para dar peso a um nó novo. ⇒ em `t = 0` e `t = 1` ela é **exactamente** a de hoje, logo os nós
//! não se mexem e a paridade do repouso fica intacta.
//!
//! ⛔⛔ **É por isso que a pele NÃO é um [`ph2d_vec_envelope::Warp`]:** aquele contrato é por
//! POSIÇÃO, e a tabela do padrão-ouro é guardada **por ponto de controlo** — ela não tem forma
//! contínua no espaço. *Pedir um campo posicional obrigaria a guardar a malha do domínio no bind*
//! (degrau de schema, e peso no ficheiro) para responder à mesma pergunta que o `t` responde de
//! graça.
//!
//! # ⚠️ A derivada é por DIFERENÇA FINITA em `t`, e a razão está medida
//!
//! O fitter precisa de `(ponto, derivada)`, e a doc do [`ph2d_vec_envelope::Warp`] argumenta — com
//! razão — que uma derivada inconsistente faz o `fit_to_bezpath` **não convergir**. ⛔ Aqui a forma
//! fechada existiria mas atravessaria o `quota` de um osso que dobra e o bump de cada mancha, e
//! **cada um deles é um ramo** — a lei ficaria escrita duas vezes, e a segunda envelhece.
//!
//! ⇒ a derivada é a diferença central da **própria** função de amostragem, em `t` (um parâmetro em
//! `[0,1]`, com `h = 1e-6` ⇒ erro `~1e-12` relativo). *Ela é consistente com o `map` por
//! construção, que é o que o contrato de facto exige.* A sonda mediu que o fit converge.

use kurbo::{BezPath, CubicBez, CurveFitSample, ParamCurve, ParamCurveFit, Point, Vec2};
use ph2d_skeleton::{Correccao, Skin};
use ph2d_vec_scene::{VecPath, VecVertex};

/// ⭐ **A tolerância do refit**, em unidades do desenho (distância de Fréchet).
///
/// ⚠️ **Ela é o único knob desta lei, e o número é MEDIDO:** a `0,05` a sonda fita um rectângulo de
/// `40 × 10` em `0,163 ms` (`--release`) e devolve `16` nós de `4`. ⛔ Mais apertado paga relógio
/// **e** nós; mais largo devolve a curva errada, que é o defeito que este módulo cura.
pub const TOLERANCIA: f64 = 0.05;

/// ⭐⭐⭐ **A LEI DA CURVA ESTÁ LIGADA?** — a porta única da F30, e ela tem **dois** leitores que não
/// se conhecem: o `recook` do quadro e o gesto que acrescenta um ponto
/// ([`ph2d_skeleton_live::ponto_novo`]).
///
/// ⛔⛔ **O segundo leitor não é zelo — é uma lei que se INVERTE.** A F28 compensa o ponto novo para
/// o desenho não saltar, e essa compensação é feita contra a curva que a lei **ingénua** desenha.
/// Com a lei da curva ligada o desenho é a imagem verdadeira da fonte, logo partir a fonte **não o
/// move** (medido: `0,0000 %`) e compensar passa a **estragar** (medido: `11,11 %` da peça).
/// *Uma cura fica errada no dia em que o defeito que ela curava deixa de existir.*
///
/// ⚠️ `PH2D_SKIN_CURVE=0` volta ao caminho dos pontos de controlo, e lá a compensação volta a ser
/// obrigatória.
///
/// ⛔⛔ **Ela é lida no SÍTIO DE CHAMADA e a lei viaja como PARÂMETRO daí para baixo** — nunca num
/// estado global. A 1.ª redacção pôs um átomo com uma porta `forcar_lei` para os gates medirem o
/// outro lado, e o doc dela dizia *«o nextest corre um processo por teste»*: verdade para o
/// `nextest`, **falsa** para o `cargo test`, que corre os testes em THREADS do mesmo processo — o
/// guarda de um vazava para outro e a suíte reprovava em conjunto e passava sozinha. *Um estado
/// global posto para o teste é um canal entre testes.*
#[must_use]
pub fn lei_da_curva_activa() -> bool {
    std::env::var("PH2D_SKIN_CURVE").as_deref() != Ok("0")
}

/// Um segmento da arte visto **através** da pele — a curva paramétrica `t ↦ blend(C(t), w(t))`.
///
/// Ela nunca é materializada: o fitter amostra-a.
struct SegmentoDaPele<'a> {
    src: CubicBez,
    pele: &'a Skin,
    /// A linha de pesos do nó de PARTIDA, ou `None` para a lei derivada.
    ra: Option<&'a [f64]>,
    /// A linha de pesos do nó de CHEGADA.
    rb: Option<&'a [f64]>,
    correcoes: &'a [Correccao],
}

impl SegmentoDaPele<'_> {
    /// A imagem de `C(t)`.
    fn ponto(&self, t: f64) -> Point {
        let c = self.src.eval(t);
        let p = [c.x, c.y];
        let mut w = self.pele.scratch();
        match (self.ra, self.rb) {
            // ⭐ A MISTURA das duas linhas no mesmo `t` — a lei da F28 para o nó novo, aplicada
            // agora a **todo** ponto da curva. Em `t = 0` e `t = 1` ela é a linha do nó, ao bit.
            (Some(a), Some(b)) => {
                let mistura: Vec<f64> = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| (y - x).mul_add(t, *x))
                    .collect();
                self.pele
                    .weights_corrected(p, Some(&mistura), &mut w, self.correcoes);
            }
            // ⛔ Sem tabela a lei é a DERIVADA, e ela já é função da posição — não há o que misturar.
            _ => self.pele.weights_corrected(p, None, &mut w, self.correcoes),
        }
        let q = self.pele.blend(p, &w);
        Point::new(q[0], q[1])
    }

    /// A derivada, por diferença central em `t` — ver o cabeçalho do módulo.
    fn derivada(&self, t: f64) -> Vec2 {
        const H: f64 = 1e-6;
        let (a, b) = ((t - H).max(0.0), (t + H).min(1.0));
        let (pa, pb) = (self.ponto(a), self.ponto(b));
        (pb - pa) / (b - a)
    }
}

impl ParamCurveFit for SegmentoDaPele<'_> {
    fn sample_pt_tangent(&self, t: f64, _sign: f64) -> CurveFitSample {
        CurveFitSample {
            p: self.ponto(t),
            tangent: self.derivada(t),
        }
    }

    fn sample_pt_deriv(&self, t: f64) -> (Point, Vec2) {
        (self.ponto(t), self.derivada(t))
    }

    /// ⛔ **Nenhuma cúspide, e é uma afirmação sobre a PELE, não uma omissão.** Uma cúspide em
    /// `W(C(t))` nasce onde a jacobiana do mapa é singular — isto é, onde a pele **dobra sobre si
    /// mesma**. Isso acontece (uma junta a mais de `180°`), e ali a resposta certa **não** é
    /// aproximar melhor: um bico bem fitado continua a ser uma dobra. ⚠️ A doc do kurbo mede o custo
    /// de a perder — *«mais subdivisão, generally not a disaster»*.
    fn break_cusp(&self, _range: core::ops::Range<f64>) -> Option<f64> {
        None
    }
}

/// ⭐⭐⭐ **DEFORMA O CAMINHO PELA CURVA** — a porta desta lei.
///
/// `pesos` é a tabela guardada no bind (por ponto de controlo, três linhas por vértice; **vazia** ⇒
/// a lei derivada). `correcoes` são as manchas pintadas à mão, já no espaço da lei.
///
/// ⚠️ **Ela SUBSTITUI a geometria do caminho**, e o número de nós do desenho **cresce** — é o preço
/// declarado: a imagem de uma cúbica por um mapa não-afim não é uma cúbica, e representá-la exige
/// mais pedaços. *A FONTE não muda; o artista continua a editar os nós que desenhou.*
pub fn aplica_pela_curva(
    pele: &Skin,
    path: &mut VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    tolerancia: f64,
) {
    // ⭐⭐⭐ **A LEI DE HOJE CORRE SEMPRE, e primeiro.** Ela preserva o `kind` e o `corner_radius` de
    // cada vértice, que o refit **não** pode preservar (um join derivado de um fit não é autoria, e
    // o raio foi cozido na deformação). ⇒ onde o mapa é afim sobre o segmento — em REPOUSO, e em
    // toda aresta cujos dois nós têm o mesmo peso e que nenhuma mancha toca — a saída é
    // **byte-idêntica** à de sempre, e nada a jusante dá por isto.
    //
    // ⛔⛔ **Isto não é optimização: é a cura de um defeito medido.** A 1.ª redacção refitava
    // **sempre**, e o gate `binding_a_shape_moves_nothing` acusou `13,333…` = `40/3` em REPOUSO —
    // a elevação `(⅓, ⅔)` de uma recta, que desenha a MESMA curva com outros pontos de controlo.
    // *O desenho estava certo e a representação é que mudava*, e oito gates da casa mediam a
    // representação.
    let fonte = path.clone();
    crate::aplica_corrigido(pele, path, pesos, correcoes);

    let ossos = if pesos.is_empty() {
        0
    } else {
        pesos.len() / (fonte.verts_all().count() * 3).max(1)
    };
    let mut base = 0usize;
    for c in 0..fonte.contour_count() {
        let Some((verts, fechado)) = fonte.contour(c) else {
            continue;
        };
        let n = verts.len();
        // ⭐ **A pergunta é sobre o RESULTADO de hoje**, não sobre os pesos: *o que a lei ingénua
        // desenhou afasta-se do mapa verdadeiro mais do que a tolerância?* É a mesma grandeza que o
        // fitter usa, logo a decisão e o remédio falam a mesma língua.
        let ingenuo = path.contour(c).map(|(v, _)| v.to_vec()).unwrap_or_default();
        if precisa_de_fit(
            pele, verts, &ingenuo, fechado, pesos, ossos, base, correcoes, tolerancia,
        ) && let Some(novos) = contorno(
            pele, verts, fechado, pesos, ossos, base, correcoes, tolerancia,
        ) && let Some((alvo, _)) = path.contour_mut(c)
        {
            *alvo = novos;
        }
        base += n;
    }
}

/// ⭐⭐ **A lei ingénua afasta-se do mapa verdadeiro mais do que `tolerancia`?**
///
/// ⚠️ **As amostras são INTERIORES**, e é o que torna a pergunta honesta: nos extremos as duas leis
/// concordam **por construção** (ali a mistura é a linha do próprio nó), logo amostrar `t = 0` ou
/// `t = 1` mediria zero sempre.
#[expect(
    clippy::too_many_arguments,
    reason = "todos os argumentos são a MESMA coisa: o contorno e a tabela que o descreve"
)]
fn precisa_de_fit(
    pele: &Skin,
    verts: &[VecVertex],
    ingenuo: &[VecVertex],
    fechado: bool,
    pesos: &[f64],
    ossos: usize,
    base: usize,
    correcoes: &[Correccao],
    tolerancia: f64,
) -> bool {
    /// Quantos pontos interiores por segmento. ⚠️ **Três e não um:** um só (o meio) é cego a um
    /// desvio que se anula ali por simetria, que é exactamente o que um osso a dobrar produz.
    const AMOSTRAS: usize = 3;
    let n = verts.len();
    if ingenuo.len() != n {
        return true;
    }
    let segs = if fechado { n } else { n.saturating_sub(1) };
    for k in 0..segs {
        let s = SegmentoDaPele {
            src: cubica(verts, k, n),
            pele,
            ra: linha(pesos, ossos, base + k),
            rb: linha(pesos, ossos, base + (k + 1) % n),
            correcoes,
        };
        let ja = cubica(ingenuo, k, n);
        for j in 1..=AMOSTRAS {
            let t = f64::from(u32::try_from(j).unwrap_or(1)) / (AMOSTRAS as f64 + 1.0);
            if (s.ponto(t) - ja.eval(t)).hypot() > tolerancia {
                return true;
            }
        }
    }
    false
}

/// A linha de pesos do nó `k` (índice PLANO, na tabela guardada), ou `None` quando não há tabela.
fn linha(pesos: &[f64], ossos: usize, k: usize) -> Option<&[f64]> {
    (ossos > 0).then(|| pesos.get(k * 3 * ossos..k * 3 * ossos + ossos))?
}

/// Deforma **um** contorno. `None` quando ele não tem segmento nenhum.
#[expect(
    clippy::too_many_arguments,
    reason = "todos os argumentos são a MESMA coisa: o contorno e a tabela que o descreve"
)]
fn contorno(
    pele: &Skin,
    verts: &[VecVertex],
    fechado: bool,
    pesos: &[f64],
    ossos: usize,
    base: usize,
    correcoes: &[Correccao],
    tolerancia: f64,
) -> Option<Vec<VecVertex>> {
    let n = verts.len();
    let segs = if fechado { n } else { n.checked_sub(1)? };
    if segs == 0 {
        return None;
    }
    let mut pts: Vec<[Point; 3]> = Vec::new();
    let inicio = {
        let s = SegmentoDaPele {
            src: cubica(verts, 0, n),
            pele,
            ra: linha(pesos, ossos, base),
            rb: linha(pesos, ossos, base + (1 % n)),
            correcoes,
        };
        s.ponto(0.0)
    };
    for k in 0..segs {
        let s = SegmentoDaPele {
            src: cubica(verts, k, n),
            pele,
            ra: linha(pesos, ossos, base + k),
            rb: linha(pesos, ossos, base + (k + 1) % n),
            correcoes,
        };
        let fitado: BezPath = kurbo::fit_to_bezpath(&s, tolerancia);
        ph2d_vec_envelope::push_cubics(&fitado, &mut pts);
    }
    Some(ph2d_vec_envelope::rebuild(&pts, inicio, fechado))
}

/// O segmento `k` como cúbica do kurbo. Fechado: o último liga de volta ao primeiro.
fn cubica(verts: &[VecVertex], k: usize, n: usize) -> CubicBez {
    let (a, b) = (&verts[k], &verts[(k + 1) % n]);
    CubicBez::new(
        Point::new(a.anchor[0], a.anchor[1]),
        Point::new(a.out_handle[0], a.out_handle[1]),
        Point::new(b.in_handle[0], b.in_handle[1]),
        Point::new(b.anchor[0], b.anchor[1]),
    )
}

#[cfg(test)]
#[path = "curva_tests.rs"]
mod tests;
