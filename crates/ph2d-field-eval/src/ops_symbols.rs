//! ⭐⭐ **OS SÍMBOLOS** (W120) — o raio, o escudo, a etiqueta, o visto, a faixa e a chave.
//!
//! # ⚠️ Três receitas, e escolher a errada custa uma medição
//!
//! 1. **INTERSECÇÃO** (escudo, etiqueta) — peças exactas em `max`. É a mais barata e a mais segura:
//!    o `max` de funções 1-Lipschitz é 1-Lipschitz, por definição.
//! 2. **SUBTRACÇÃO** (faixa, etiqueta) — `max(corpo, −buraco)`. ⚠️ O que se subtrai tem de **passar
//!    de largo** pelas faces do corpo que não quer comer: com as fronteiras coincidentes a
//!    intersecção arredondada **come** aquelas faces.
//! 3. **UNIÃO** (raio, visto, chave) — só quando a forma não é convexa. ⚠️ Aqui manda a lei da
//!    [`crate::ops_arrows`]: duas peças que se **tocam sem se sobrepor** dão `0` num ponto interior,
//!    e duas cujas fronteiras **coincidem ao longo de uma face** fazem a união **inchar** para fora
//!    dela. Toda união deste arquivo sobrepõe-se numa área, e nenhuma partilha face.

use fidget::context::Tree;

use crate::ops::{half_plane, slab_and_walls};
use crate::ops_joint::{Edge, intersection_joint, union_joint};
use crate::ops_plate2d::{arco, disco_em, faixa, paredes, paredes_faixa, quinas, rect_round_em};

/// Um triângulo de três semiplanos, com as quinas arredondadas duas a duas.
///
/// ⚠️ **Os vértices em ordem ANTI-HORÁRIA** — o [`half_plane`] é negativo à esquerda de `a → b`, e
/// invertê-los dá o complemento.
fn triangulo(v: [[f64; 2]; 3], e: Edge) -> Tree {
    let l0 = half_plane(v[0], v[1]);
    let l1 = half_plane(v[1], v[2]);
    let l2 = half_plane(v[2], v[0]);
    // ⚠️ **OS TRÊS CANTOS NUMA MISTURA SÓ**, e não duas juntas encaixadas: encaixadas, a segunda
    // recebe a composta da primeira e leva a costura dela para a aresta seguinte — o raio lia
    // `37,2 %` da superfície sobre um vinco com o filete a metade do limite.
    crate::ops_joint::intersection_joint_n(
        &[l0.clone(), l1.clone(), l2.clone()],
        &[(l0.clone(), l1.clone()), (l1, l2.clone()), (l2, l0)],
        e,
    )
}

/// ⭐ **RAIO** — a união de dois triângulos que se **cruzam numa banda**, nunca que se tocam.
///
/// ⛔ A decomposição óbvia (metade de cima até `y = 0`, metade de baixo a partir dali) é uma
/// **partição**: as duas bases ficam sobre a mesma recta, o `min` vale zero ao longo dela, e nasce
/// uma superfície fantasma **dentro** do sólido — a lição que a [`crate::ops::sd_star`] pagou.
///
/// ⭐ Aqui cada triângulo **atravessa** o meio: o de cima desce a `−0,1 h` e o de baixo sobe a
/// `+0,1 h`, e a banda entre os dois tem área. ⚠️ E ela não é um remendo — é exactamente onde o
/// raio muda de direcção, então a silhueta que a união desenha ali **é** a do símbolo.
pub fn sd_bolt(
    half_width: f64,
    half_span: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, h) = (half_width, half_span);
    let cima = triangulo([[0.40 * w, h], [-w, -0.10 * h], [0.20 * w, -0.10 * h]], e);
    let baixo = triangulo([[-0.40 * w, -h], [w, 0.10 * h], [-0.20 * w, 0.10 * h]], e);
    slab_and_walls(&union_joint(&cima, &baixo, e), half_height, e)
}

/// ⭐ **ESCUDO** — o topo reto e dois arcos que se encontram numa ponta em baixo.
///
/// ⭐⭐ **Os dois lados são DISCOS, e a conta fecha sozinha:** um círculo tangente à vertical em
/// `(±w, s)` tem o centro à mesma altura, e passar por `(0, −s)` fixa o raio —
/// `c = (4s² − w²)/2w`, `R = c + w`. ⇒ a forma sai de uma **intersecção de três peças exactas**, sem
/// união nenhuma e sem um número escolhido à mão.
///
/// ⚠️ **`2·half_span > half_width` é a cerca**, e o documento recusa fora dela: abaixo disso o
/// centro cai do lado errado e os arcos curvam ao contrário.
pub fn sd_shield(
    half_width: f64,
    half_span: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    let c = (4.0 * s * s - w * w) / (2.0 * w);
    let r = c + w;
    let (dl, dr) = (disco_em(-c, s, r), disco_em(c, s, r));
    let topo = Tree::y() - Tree::constant(s);
    if chamfer <= 0.0 {
        let lados = intersection_joint(&dl, &dr, e);
        return slab_and_walls(&intersection_joint(&lados, &topo, e), half_height, e);
    }
    // ⭐ Com chanfro as peças entram INTEIRAS — um perfil já composto leva a costura dele para o
    // aro. Ver [`crate::ops::plate_joint_n`].
    crate::ops::plate_joint_n(
        &[dl.clone(), dr.clone(), topo.clone()],
        &[(dl.clone(), dr.clone()), (dl, topo.clone()), (dr, topo)],
        half_height,
        e,
    )
}

/// ⭐ **ETIQUETA** — o rectângulo que afila numa ponta a `+X`, com o furo do cordel.
///
/// ⚠️ **É uma INTERSECÇÃO seguida de uma SUBTRACÇÃO**, e nenhuma união: o corpo é convexo (um
/// rectângulo cortado por dois planos), e o furo sai por `max(corpo, −disco)`.
pub fn sd_tag(
    half_width: f64,
    half_span: f64,
    point: f64,
    hole: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    let corpo = rect_round_em(0.0, 0.0, w, s, round);
    // As duas rectas que fecham a ponta, de `(w, 0)` até `(w − point, ±s)`.
    let sup = half_plane([w, 0.0], [w - point, s]);
    let inf = half_plane([w - point, -s], [w, 0.0]);
    if chamfer > 0.0 {
        // ⭐ Com chanfro o corpo entra em quatro paredes, e a ponta nas duas rectas dela.
        let g = paredes(0.0, 0.0, w, s);
        let mut arestas = quinas(&g);
        arestas.push((sup.clone(), inf.clone()));
        arestas.push((g[2].clone(), sup.clone()));
        arestas.push((g[3].clone(), inf.clone()));
        let pecas = [
            g[0].clone(),
            g[1].clone(),
            g[2].clone(),
            g[3].clone(),
            sup.clone(),
            inf.clone(),
            -disco_em(-w * 0.7, 0.0, hole),
        ];
        return crate::ops::plate_joint_n(&pecas, &arestas, half_height, e);
    }
    let bico = intersection_joint(&intersection_joint(&corpo, &sup, e), &inf, e);
    // ⚠️ **O furo é um círculo LISO** — ele não faz quina 2D com nada, então não há aresta a
    // declarar: quem arredonda a boca dele é o aro da chapa, como em toda a família.
    // ⚠️ **O centro sai da LARGURA, e não da altura** — a cerca que o documento impõe é
    // `hole < 0,3 × half_width`, e um centro derivado de `half_span` poria as duas a discordar sobre
    // onde o furo cabe. *Duas respostas à mesma pergunta, e a que o artista vê é a que envelhece.*
    let furo = -disco_em(-w * 0.7, 0.0, hole);
    slab_and_walls(&bico.max(furo), half_height, e)
}

/// ⭐ **VISTO** — duas faixas que se cruzam no vértice de baixo.
///
/// ⚠️ **Cada braço passa do vértice pela própria espessura**, e é isso que lhes dá a sobreposição:
/// cortadas exactamente no vértice, as duas pontas quadradas encostam-se ao longo de rectas que se
/// cruzam ali, e a área comum degenera com o ângulo.
pub fn sd_check(
    half_width: f64,
    half_span: f64,
    thickness: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s, t) = (half_width, half_span, thickness * 0.5);
    let v = [-0.25 * w, -s];
    let alonga = |p: [f64; 2]| {
        let (dx, dy) = (v[0] - p[0], v[1] - p[1]);
        let l = (dx * dx + dy * dy).sqrt().max(f64::MIN_POSITIVE);
        [v[0] + dx / l * t, v[1] + dy / l * t]
    };
    let curto = [-w, 0.15 * s];
    let longo = [w, s];
    if chamfer > 0.0 {
        // ⭐ Cada braço é uma chapa de meios-planos no referencial dele — a receita da cruz.
        let braco = |p: [f64; 2], q: [f64; 2]| {
            let w4 = paredes_faixa(p, q, t);
            crate::ops::plate_joint_n(&w4, &quinas(&w4), half_height, e)
        };
        return union_joint(
            &braco(alonga(curto), curto),
            &braco(alonga(longo), longo),
            e,
        );
    }
    let a = faixa(alonga(curto), curto, t, round);
    let b = faixa(alonga(longo), longo, t, round);
    slab_and_walls(&union_joint(&a, &b, e), half_height, e)
}

/// ⭐ **FAIXA / fita** — o rectângulo com um entalhe em «V» em cada ponta.
///
/// ⚠️ **Os entalhes SUBTRAEM-SE, e cada um é uma cunha INFINITA para fora** — assim ela nunca toca
/// a face de trás do rectângulo, e a intersecção arredondada não a come. ⭐ O vértice de dentro de
/// cada entalhe é uma quina **côncava** da faixa, e arredondá-lo é arredondar o **ápice da cunha**:
/// *um vinco côncavo de um sólido é uma quina convexa do vazio*, a mesma lei do chevron.
pub fn sd_banner(
    half_width: f64,
    half_span: f64,
    notch: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    let corpo = rect_round_em(0.0, 0.0, w, s, round);
    let entalhe = |sinal: f64| {
        let apex = [sinal * (w - notch), 0.0];
        let cima = [sinal * (w + notch), s * 2.0];
        let baixo = [sinal * (w + notch), -s * 2.0];
        // ⚠️ A ordem dos vértices vira com o sinal: o [`half_plane`] é negativo à ESQUERDA.
        // ⚠️⚠️ **A ORDEM dos vértices é o SINAL da cunha**: o [`half_plane`] é negativo à esquerda
        // de `a → b`, então trocá-los devolve o **complemento**. A 1.ª redacção tinha as duas
        // orientações ao contrário e a faixa saía VAZIA — o entalhe comia a fita inteira.
        let (p, q) = if sinal > 0.0 {
            (half_plane(cima, apex), half_plane(apex, baixo))
        } else {
            (half_plane(apex, cima), half_plane(baixo, apex))
        };
        -intersection_joint(&p, &q, e)
    };
    // ⚠️⚠️ **A SUBTRACÇÃO é uma JUNTA, e não um `max` duro** — as quinas onde o entalhe encontra o
    // contorno da fita são arestas a sério, e com o `max` cru elas ficavam **vivas**: a sonda leu
    // `3,5 %` da superfície sobre um vinco de `75,2°`. *Um `max` é uma intersecção sem raio, e uma
    // intersecção sem raio é uma quina.*
    let (ed, ee) = (entalhe(1.0), entalhe(-1.0));
    if chamfer > 0.0 {
        // ⭐ Com chanfro as peças entram INTEIRAS — o corpo em quatro paredes e cada entalhe como a
        // peça que ele é.
        let g = paredes(0.0, 0.0, w, s);
        let mut arestas = quinas(&g);
        for lado in [&ed, &ee] {
            arestas.push((g[2].clone(), lado.clone()));
            arestas.push((g[3].clone(), lado.clone()));
        }
        let pecas = [
            g[0].clone(),
            g[1].clone(),
            g[2].clone(),
            g[3].clone(),
            ed.clone(),
            ee.clone(),
        ];
        return crate::ops::plate_joint_n(&pecas, &arestas, half_height, e);
    }
    let perfil = intersection_joint(&intersection_joint(&corpo, &ed, e), &ee, e);
    slab_and_walls(&perfil, half_height, e)
}

/// ⭐ **CHAVE `{`** — quatro quartos de arco, dois por metade.
///
/// ⚠️ **A proporção é a identidade da forma**, então ela não tem controle de largura: o raio é
/// `half_span/2` e o alcance sai dele. Uma chave mais larga que alta deixa de se ler como uma chave,
/// e um knob que só a estraga é um knob a menos.
///
/// ⚠️ **Os arcos passam uns dos outros em ÂNGULO** (`OVERLAP`), e sem isso as pontas encostavam-se
/// com as secções coincidentes — a união arredondada incharia ali. Ver o cabeçalho do módulo.
pub fn sd_brace(
    half_span: f64,
    thickness: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    use std::f64::consts::{FRAC_PI_2, PI};
    /// A folga angular que dá área à sobreposição de dois arcos tangentes.
    const OVERLAP: f64 = 0.12;
    let e = Edge::square(round, chamfer);
    let r = half_span * 0.5;
    let t = thickness * 0.5;
    let pecas = [
        arco(2.0 * r, r, r, t, FRAC_PI_2 - OVERLAP, PI + OVERLAP, e),
        arco(0.0, r, r, t, -FRAC_PI_2 - OVERLAP, OVERLAP, e),
        arco(0.0, -r, r, t, -OVERLAP, FRAC_PI_2 + OVERLAP, e),
        arco(2.0 * r, -r, r, t, PI - OVERLAP, 1.5 * PI + OVERLAP, e),
    ];
    // ⚠️ **A mistura é UM QUARTO da meia-espessura** — os arcos encontram-se TANGENTES, então um
    // raio pequeno basta para alisar a junta; a metade fechava o vão da curva, e um vão fechado
    // deixa de ser uma chave.
    //
    // ⚠️⚠️ **E ela é uma união com JUNTA, não a crua**: o nariz da chave é o encontro dos dois arcos
    // do meio, e é uma aresta a sério. Com a [`crate::ops::union_round_n`] o chanfro **não lhe
    // chegava** — ele entra pela junta, e uma união crua não tem nenhuma; a sonda leu `33` pontos
    // por cortar, **todos no nariz**.
    // ⛔⛔⛔ **A UNIÃO ENTRA CRUA NAS DUAS ROTAS, e o preço está declarado.**
    //
    // O **nariz** — onde os dois arcos do meio se encontram — é uma quina a sério (`55,6°`), e a
    // sonda localizou-a: `33` de `87` pontos de vinco, **todos** em `x = +0,148`. Sem junta na
    // união, o chanfro não lhe chega: ele corta `62,1 %` das arestas da chave contra os `~90 %` da
    // barra, e a forma está no [`CHANFRO_APICE`] do `measure_sharp_edges` com esse número.
    //
    // ⛔ **A alternativa foi MEDIDA e recusada:** com [`crate::ops_joint::union_joint_n`] o chanfro
    // alcança o nariz **e** a marcha passa a `passo × ‖∇f‖ = 1,36` — acima do `1` em que ela
    // atravessa a superfície, isto é, a peça sai **furada**. ⚠️ E declarar **um** vale em vez de
    // três dá **exactamente o mesmo número** (`1,9290` nos dois): o que domina não são os planos a
    // mais, é o plano do corte estar **negativo** junto a um encontro quase-tangente, onde ele
    // ganha o `min` da mistura e entra no `length` dela.
    //
    // ⇒ *entre uma quina que o chanfro não corta e uma peça furada, fica a quina.* A cura de fundo
    // é outra: um vale que sabe que os vizinhos dele são tangentes.
    slab_and_walls(&crate::ops::union_round_n(&pecas, t * 0.25), half_height, e)
}
