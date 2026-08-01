//! **A SILHUETA — que forma o traço tem, visto deste pixel.**
//!
//! Irmão de [`super::binning`], e o corte entre os dois é de RESPONSABILIDADE: lá se responde
//! *quais segmentos alcançam este ladrilho*; aqui, *que forma eles desenham no pixel*. As duas
//! perguntas têm oráculos diferentes (uma é conservadora por construção, a outra é exata), e lê-las
//! juntas foi o que deixou o teto de LOC daquele arquivo estourar.

use crate::binning::{BinSeg, ScreenSpace, closest_on_seg};
use crate::pack::FlipGpuData;

/// O que este pixel vê da silhueta do traço.
///
/// ⚠️ **`sd` e `planes` respondem perguntas diferentes e as duas são precisas.** O `sd` é a
/// distância com sinal (negativa dentro) da passagem MAIS PRÓXIMA — é ele que diz onde amostrar o
/// perfil ao longo da normal. O `planes` é o conjunto das passagens que de fato cruzam este pixel, e
/// é dele que sai a ÁREA coberta; num pixel de quina o `sd` de uma passagem não sabe da outra.
pub(crate) struct Silhouette {
    pub sd: f32,
    pub near: [f32; 2],
    pub dist: f32,
    pub planes: crate::pixel_area::PlaneSet,
}

/// A silhueta do traço vista deste pixel — a distância com sinal é **negativa dentro**.
///
/// É o `min` sobre as passagens que o `flip.wgsl` também faz — só que aqui **EXATO**, porque o
/// percurso tem os segmentos na mão. O shader precisa estimar o tamanho de um pixel em unidades de
/// `dn` (`aa = fwidth(dn)`) e o próprio comentário dele registra o preço: sobre a UNIÃO o `fwidth`
/// mede o gradiente de um `min`, que **salta na costura**, e por isso o AA de lá é por-PASSAGEM.
/// Aqui não há derivada de tela envolvida, e por isso não há costura para saltar.
///
/// ⚠️ **Com o tip pontilhado a silhueta é a das CONTAS**, e é a MESMA fórmula com outra lista: um
/// disco é uma cápsula degenerada. Sem isto o `edge` mediria a borda da FITA — 1 em toda a extensão
/// dela — e as contas sairiam sem anti-aliasing, com o `p_eval` empurrado para a linha-de-centro em
/// vez de para dentro do carimbo.
///
/// ⚠️ **E é aqui que a TAMPA CHATA mora** ([`crate::dabs::flat_caps`]): no rasterizador ela é a
/// ausência de geometria (o quad não estende), e o percurso não tem quad — então ela é a interseção
/// com um semi-plano, um `max` sobre o `sd`. Só o PRIMEIRO e o ÚLTIMO segmento a honram; os do meio
/// cobrem o que cobrem, e é isso que deixa um traço que se enrola de volta pintar sobre o próprio
/// começo cortado.
/// **UM PLANO DE TAMPA, já resolvido** — onde ele passa (px), para onde aponta o FORA, e em que
/// ARCO (mundo) do traço ele está.
///
/// ⚠️ **Ele é um fato do TRAÇO, não do segmento**, e essa distinção é o bug de 2026-07-31: o corte
/// era aplicado só ao PRIMEIRO segmento, então o disco de raio `r` na ponta do SEGUNDO ficava
/// inteiro e espiava `r − |p1 − p0|` **além** do plano. Com pontos esparsos isso vale zero; com o
/// ajuste denso que esta linha shipou o primeiro segmento tem poucos px e o resquício é quase `r`.
/// Medido (r = 20): 1º segmento de 8 px ⇒ **11,50 px** de tinta passando; de 1 px ⇒ **18,50**.
#[derive(Copy, Clone)]
struct CapPlane {
    q: [f32; 2],
    n: [f32; 2],
    arc: f32,
}

/// Os dois planos de tampa deste traço, resolvidos UMA vez (posição, normal para fora, arco).
///
/// A normal é perpendicular ao PRIMEIRO (ou ÚLTIMO) segmento — onde o `miter_a` do rasterizador cai
/// quando não há vizinho —, e ela sai dos pontos do traço, **nunca do segmento que o laço está
/// vendo**: um segmento do meio não sabe para onde a ponta olha.
fn resolve_caps(data: &FlipGpuData, screen: &ScreenSpace, run: &[BinSeg]) -> [Option<CapPlane>; 2] {
    let (head, tail) = crate::dabs::flat_caps(data, run);
    let plano = |pt: u32, viz: u32, para_tras: bool| -> Option<CapPlane> {
        let q = screen.point_px(data.points[pt as usize].pos);
        let o = screen.point_px(data.points[viz as usize].pos);
        let v = if para_tras {
            [q[0] - o[0], q[1] - o[1]]
        } else {
            [o[0] - q[0], o[1] - q[1]]
        };
        let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
        (len > 1e-6).then(|| CapPlane {
            q,
            n: [v[0] / len, v[1] / len],
            arc: data.arc_len[pt as usize],
        })
    };
    [
        head.and_then(|pt| {
            plano(pt, pt + 1, false).map(|c| CapPlane {
                n: [-c.n[0], -c.n[1]],
                ..c
            })
        }),
        tail.and_then(|pt| plano(pt, pt - 1, true)),
    ]
}

pub(crate) fn stroke_silhouette(
    run: &[BinSeg],
    data: &FlipGpuData,
    screen: &ScreenSpace,
    tip: crate::tau::TipShape,
    p: [f32; 2],
) -> Option<Silhouette> {
    let tail = crate::dabs::tail_point(data, run);
    let caps = resolve_caps(data, screen, run);
    let mut best: Option<(f32, [f32; 2], f32)> = None;
    let mut planes = crate::pixel_area::PlaneSet::default();
    let mut keep = |sd: f32, near: [f32; 2], dist: f32| {
        // ⚠️ **O plano é OFERECIDO por passagem, não pelo vencedor** — é a única diferença que
        // importa num pixel de quina, onde o vencedor sozinho descreve meia verdade. O `offer`
        // descarta o que não alcança o pixel, então o custo de oferecer é uma comparação.
        //
        // ⚠️ **Com o pixel EM CIMA do eixo (`dist ≈ 0`) a normal é indefinida — e não importa:** ali
        // a silhueta é um disco centrado no pixel, equidistante em toda direção, então qualquer
        // normal unitária dá a mesma área. (E na prática o plano nem entra: `sd = −r`, fora do
        // alcance para qualquer traço de raio ≥ 0,71 px.)
        let n = if dist > 1e-6 {
            [(p[0] - near[0]) / dist, (p[1] - near[1]) / dist]
        } else {
            [1.0, 0.0]
        };
        planes.offer(crate::pixel_area::OutsidePlane { n, sd });
        if best.is_none_or(|(prev, _, _)| sd < prev) {
            best = Some((sd, near, dist));
        }
    };
    for seg in run {
        let (pa, pb) = (data.points[seg.a as usize], data.points[seg.b as usize]);
        let sa = screen.point_px(pa.pos);
        let sb = screen.point_px(pb.pos);
        let (t, cx, cy) = closest_on_seg(p, sa, sb);
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        let ra = screen.radius_px(pa.width);
        let rb = screen.radius_px(pb.width);
        // O CORTE deste segmento, em px (`NEG_INFINITY` = sem tampa, e `max` com ele é a identidade
        // exata ⇒ todo traço de tampa redonda é byte-intocado). A normal aponta para FORA: `−dir` no
        // começo, `+dir` no fim — perpendicular ao PRIMEIRO/ÚLTIMO segmento, que é onde o `miter_a`
        // do rasterizador cai quando não há vizinho.
        // ⚠️ **O corte alcança todo segmento a menos de `r` de ARCO da tampa, não só o primeiro.**
        // É o arco — e não a distância geométrica — que preserva a razão do desenho por-segmento
        // que o `flat_caps` documenta: um traço que se ENROLA de volta sobre o próprio começo está
        // geometricamente perto e a ARCOS de distância, então ele segue pintando ali (um semi-plano
        // global apagaria essa tinta). Perto em arco é o vizinho imediato, cujo disco de ponta é
        // exatamente o que espiava para fora.
        let mut cut = f32::NEG_INFINITY;
        let dw = [pb.pos[0] - pa.pos[0], pb.pos[1] - pa.pos[1]];
        let arc_lo = data.arc_len[seg.a as usize];
        let arc_hi = arc_lo + (dw[0] * dw[0] + dw[1] * dw[1]).sqrt();
        for cp in caps.iter().flatten() {
            let fora = (cp.arc - arc_hi).max(arc_lo - cp.arc).max(0.0) * screen.px_per_world;
            if fora < ra.max(rb) {
                cut = cut.max(crate::dabs::cap_sd(p, cp.q, cp.n));
            }
        }
        if let crate::tau::TipShape::Beads { pitch, square } = tip {
            // A conta mais próxima DESTE segmento é uma das duas que cercam o arco do ponto mais
            // próximo, clampadas às que o segmento possui — as de fora são de um vizinho, que
            // também está na lista e responde por elas.
            let arc_a = data.arc_len[seg.a as usize];
            let dw = [pb.pos[0] - pa.pos[0], pb.pos[1] - pa.pos[1]];
            let wlen = (dw[0] * dw[0] + dw[1] * dw[1]).sqrt();
            let (o0, o1) = crate::dabs::bead_range(arc_a, arc_a + wlen, pitch, tail == Some(seg.b));
            if o0 > o1 {
                continue;
            }
            let base = ((arc_a + t * wlen) / pitch).floor() as i32;
            for k in [base.clamp(o0, o1), (base + 1).clamp(o0, o1)] {
                let bead = crate::dabs::bead_at((sa, sb), (ra, rb), (arc_a, wlen), (k, pitch));
                // A "distância" é `dn·r`: no disco isso É `|p − c|`, e no quadrado é a Chebyshev no
                // frame da tangente — a mesma grandeza que o `dn` normaliza, então o `edge` e o
                // empurrão do `p_eval` falam a unidade certa nos dois.
                let dist = crate::dabs::bead_dn(p, bead, square) * bead.r;
                keep((dist - bead.r).max(cut), bead.c, dist);
            }
            continue;
        }
        let r = ra * (1.0 - t) + rb * t;
        keep((dist - r).max(cut), [cx, cy], dist);
    }
    let (sd, near, dist) = best?;
    Some(Silhouette {
        sd,
        near,
        dist,
        planes,
    })
}
