//! ⭐⭐⭐ **A SILHUETA DE UMA FORMA QUE SE SOBREPÕE A SI MESMA** — o contacto resolvido pela regra
//! do preenchimento.
//!
//! # O defeito, medido
//!
//! Numa dobra forte (`100°`–`150°`) a parte de DENTRO de dois membros presos a ossos passa uma por
//! cima da outra — é geometria, não erro da lei: dois pedaços rígidos que rodam em torno de uma
//! junta sobrepõem-se do lado côncavo assim que `tan(θ/2)` passa a distância da junta ao início do
//! pedaço rígido sobre a meia-espessura. O PREENCHIMENTO (regra não-zero) pinta a região certa —
//! a união dos dois membros, com um canto em «V» no cotovelo —, e o TRAÇO desenha o contorno
//! inteiro, **com o «olho» da sobreposição por dentro**. Medido na barra da cena, dobra em S
//! (`skinned_mesh_arap_sonda_tests`): o contorno cruza-se `2` vezes de `100°` a `150°` e a
//! silhueta tem `0` cruzamentos, com a área do preenchimento intacta onde não há contacto.
//!
//! ⭐ **A imagem presa já faz isto de graça** (um membro desenha-se por cima do outro, como num
//! boneco de recorte, e o canto fica em «V») — esta porta é o que põe o desenho vectorial a
//! concordar com ela.
//!
//! # O estado da arte, e porque é ESTA a resposta
//!
//! O que a literatura faz no contacto de um cotovelo é pôr a pele na FRONTEIRA DA UNIÃO dos
//! membros (*Implicit Skinning*, Vaillant et al. 2013: cada vértice é projectado na iso-superfície
//! da composição dos campos por osso, e o contacto vira um vinco). Para um caminho vectorial a
//! fronteira da união **é exacta e barata**: é a união do caminho com o VAZIO, pelo motor que esta
//! crate já tem — cúbicas de verdade, sem malha nenhuma.
//!
//! ⛔ **Duas curas foram construídas, medidas e RECUSADAS antes desta** (sonda citada acima):
//! - **ARAP onde a lei esmaga** (Sorkine–Alexa 2007, a família do Plastic do OpenToonz): desfaz os
//!   triângulos virados e **cria** laços a `90°` — ela resiste à compressão, e o lado de dentro de
//!   um cotovelo TEM de comprimir.
//! - **A menor correcção sem inversão** (barreira sobre `det J`, à la IPC/Garanzha 2021): zero
//!   triângulos virados em toda a dobra, e o contorno continua a cruzar-se — **o defeito não é a
//!   pele virar do avesso, é o contacto**. Acrescentar a barreira do ângulo da borda trava o
//!   optimizador (colapsa triângulos a área zero).
//!
//! # A lei desta porta
//!
//! - **Só corre quando o contorno se cruza** ([`crosses_itself`]) — fora do contacto a forma sai
//!   **byte-idêntica**, e o custo é o de uma varredura de segmentos.
//! - **O estilo é o da forma** (preenchimento, traço, opacidade, mistura, camadas); só a geometria
//!   muda. ⚠️ Não é o `compound_from` da booleana, que devolve o estilo MÍNIMO de um resultado
//!   de edição: aqui a forma é a mesma, desenhada.
//! - **Um caminho ABERTO não tem silhueta** (não há interior) ⇒ `None`, e quem chama desenha-o
//!   como sempre.

use kurbo::{BezPath, PathEl, Shape};
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{Contour, FillRule, VecPath, VecVertex};

/// ⭐ **A tolerância do achatamento na detecção, em fracção da diagonal da forma** — `1e-4`.
///
/// ⚠️ Ela decide só se a varredura CORRE, nunca a geometria que sai (essa é exacta, do motor). Uma
/// corda de uma curva sem auto-intersecção só cruza outra quando os dois fios estão a menos da
/// tolerância um do outro — e aí o contacto existe à escala do desenho.
pub const DETECTION_TOLERANCE: f64 = 1e-4;

/// ⭐⭐⭐ **A silhueta de `path`** quando ele se sobrepõe a si mesmo; `None` quando não se
/// sobrepõe, é aberto, ou o motor recusa (e aí quem chama desenha a forma como estava).
#[must_use]
pub fn resolve_overlap(path: &VecPath) -> Option<VecPath> {
    if !path.closed || path.subpaths.iter().any(|c| !c.closed) {
        return None;
    }
    let bez = crate::to_bez(path);
    if !crosses_itself(&bez) {
        return None;
    }
    let rule = match path.fill_rule {
        FillRule::NonZero => LsFillRule::NonZero,
        FillRule::EvenOdd => LsFillRule::EvenOdd,
    };
    // ⭐ `A ∪ ∅` e NÃO `A ∪ A` — a regra de multiplicidade do `linesweeper` 0.4 (ver
    // `expand.rs`, `Region::of`): uma aresta com multiplicidade par não se dissolve.
    let groups = crate::binary_grouped(&bez, &BezPath::new(), rule, BinaryOp::Union)?;
    let caixa = bez.bounding_box();
    let diagonal = caixa.width().hypot(caixa.height());
    let solda = SOLDA_DA_QUINA * diagonal;
    let raio = RAIO_DO_VINCO * diagonal;
    let originais = nos_do_desenho(path);
    let mut contornos = groups
        .iter()
        .flatten()
        .filter_map(crate::verts_from_bez)
        .map(|v| solda_os_segmentos_curtos(v, solda))
        .map(|v| arredonda_os_vincos(v, &originais, raio))
        .filter(|v| v.len() >= 3);
    let outer = contornos.next()?;
    let resto: Vec<Contour> = contornos.map(Contour::new_closed).collect();
    let mut out = path.clone();
    out.verts = outer;
    out.closed = true;
    // ⚠️ Os contornos do motor saem ORIENTADOS (de fora primeiro, os buracos dentro), logo as
    // duas regras concordam; `EvenOdd` é a que não depende da orientação de nenhum deles.
    out.fill_rule = if resto.is_empty() {
        FillRule::NonZero
    } else {
        FillRule::EvenOdd
    };
    out.subpaths = resto;
    Some(out)
}

/// ⭐⭐⭐ **A solda da QUINA, em fracção da diagonal da forma** — `1e-3`.
///
/// ⛔ **O motor deixa pedaços MINÚSCULOS junto do ponto de cruzamento** (medido, report do dono de
/// 2026-09-29: *«a depender do ângulo a quina fica inconsistente»*): na barra da cena, dobrada de
/// `100°` a `150°`, a silhueta traz segmentos de comprimento `0` a `~4e-3` colados à quina nova, e um
/// vértice que VIRA `180°` sobre um deles. A junta do traço é calculada sobre as TANGENTES dos dois
/// segmentos que se encontram — e a tangente de um segmento degenerado é arbitrária ⇒ a mesma quina
/// saía em bico, cortada ou com um dente conforme o ângulo, em vez de seguir a junta escolhida no
/// painel. ⇒ um segmento cujos QUATRO pontos cabem nesta distância é fundido no vizinho, e a quina
/// fica UM vértice com as duas tangentes reais.
///
/// ⚠️ O número: acima do ruído da varredura (os pedaços medidos vão até `~1e-3` da diagonal) e
/// milhares de vezes abaixo de qualquer geometria que um artista desenhe ou veja — na barra da cena
/// são `4 mm` numa forma de `4 m`, contra um traço de `60 mm`.
pub const SOLDA_DA_QUINA: f64 = 1e-3;

/// ⛔⛔ **Um ponto de controlo a DISTÂNCIA DE RUÍDO de outro é posto EXACTAMENTE sobre ele.**
///
/// A junta de um traço é calculada sobre a tangente de cada segmento na ponta, e a tangente de uma
/// cúbica `(p0, p1, p2, p3)` na ponta de saída é `p1 − p0`, ou `p2 − p0` se `p1` coincide, ou
/// `p3 − p0` se `p2` também — com coincidência EXACTA. Medido (report do dono de 2026-09-29): o
/// motor e o bake devolvem segmentos cuja alça está a `~1e-6`–`1e-9` da ponta, às vezes do lado de
/// TRÁS dela, e aí a «tangente» é ruído — a junta via uma meia-volta de `180°` onde a geometria
/// tem uma quina de `~40°`, e a quina saía diferente a cada ângulo. ⇒ `p1` a `≤ tol` de `p0` passa
/// a `p0`; `p2` a `≤ tol` de `p3` passa a `p3`; e os dois cruzados (`p2` sobre `p0`, `p1` sobre
/// `p3` — uma recta cuja alça caiu na ponta de LÁ). Um ponto a `≤ tol` muda a curva em `≤ tol`.
pub fn limpa_as_alcas(verts: &mut [VecVertex], tol: f64) {
    let perto = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) <= tol;
    let n = verts.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let (p0, p3) = (verts[i].anchor, verts[j].anchor);
        let (mut p1, mut p2) = (verts[i].out_handle, verts[j].in_handle);
        if perto(p1, p0) {
            p1 = p0;
        } else if perto(p1, p3) {
            p1 = p3;
        }
        if perto(p2, p3) {
            p2 = p3;
        } else if perto(p2, p0) {
            p2 = p0;
        }
        verts[i].out_handle = p1;
        verts[j].in_handle = p2;
    }
}

/// Funde todo segmento cujos quatro pontos de controlo cabem em `tol` do vértice onde ele começa —
/// o vértice que fica leva a alça de ENTRADA dele e a de SAÍDA do fundido. Cíclico (o contorno é
/// fechado), e nunca abaixo de três vértices.
#[must_use]
pub fn solda_os_segmentos_curtos(mut verts: Vec<VecVertex>, tol: f64) -> Vec<VecVertex> {
    let perto = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) <= tol;
    let mut i = 0;
    while i < verts.len() && verts.len() > 3 {
        let n = verts.len();
        let (c, q) = (verts[i], verts[(i + 1) % n]);
        let minusculo = perto(c.anchor, q.anchor)
            && perto(c.anchor, c.out_handle)
            && perto(c.anchor, q.in_handle);
        if minusculo {
            let j = (i + 1) % n;
            verts[i].out_handle = q.out_handle;
            verts[i].kind = crate::classify(&verts[i]);
            verts.remove(j);
            // Não avança: o novo vizinho pode ser minúsculo também. Se o removido vinha ANTES
            // (o fecho), o índice deslizou um para trás.
            if j < i {
                i -= 1;
            }
        } else {
            i += 1;
        }
    }
    // ⚠️ A limpeza das alças corre DEPOIS de soldar, e só aí: o vértice que fica herda a alça de
    // saída do fundido, que mora na âncora DELE — a `~1e-6` da nova (medido: era essa a meia-volta
    // que sobrava a `150°`). ⛔ Uma passagem ANTES da solda foi escrita e a mutação que a apagava
    // SOBREVIVEU: a solda não decide nada a partir das alças limpas (um ponto a ruído passa no
    // mesmo `perto`), e o ruído medido (`1e-9`–`1e-6`) está três ordens abaixo da tolerância.
    limpa_as_alcas(&mut verts, tol);
    verts
}

/// ⭐⭐⭐ **O raio do VINCO, em fracção da diagonal da forma** — `1e-2`.
///
/// Decisão do dono (2026-09-30, com cinco fotos: *«além de inconsistente, fica tão pontudo que
/// perfura o outro lado da forma»*): **o vinco que a dobra cria fica ARREDONDADO, em todo ângulo e
/// em toda junta.** O vinco não é uma quina que o artista desenhou — ele nasce no ponto onde os dois
/// membros se encostam —, e medido na pose das fotos dele ele vira `161°` logo depois do encosto e
/// desce a `124°` a fechar: com a junta `Miter` o bico mede `1/sin((180 − v)/2)` meias-larguras, `3`
/// larguras de traço a `161°` e até `5` perto do encosto (com o limite `10`), o que ATRAVESSA a
/// peça; com o limite `4` o bico vira chanfro a partir de `~151°` e a quina muda de forma com o
/// ângulo. ⇒ nenhum limite serve as duas queixas, e a cura é o vinco deixar de ser quina.
///
/// ⚠️ O número: pequeno o bastante para o PREENCHIMENTO continuar com o «V» da imagem presa (`4 cm`
/// numa diagonal de `4 m`, `4 %` da espessura da barra da cena), e muito acima da solda (`1e-3`), senão
/// o arco nasceria soldado. O traço desenha-se por cima dele como um arco de raio `r + ½·largura`:
/// arredondado com qualquer junta.
pub const RAIO_DO_VINCO: f64 = 1e-2;

/// ⭐⭐⭐ **Arredonda os VINCOS da silhueta**: todo vértice CÔNCAVO que VIRA e que NÃO é um nó do
/// desenho (`originais`) nasceu no cruzamento — é um vinco — e é trocado por um arco tangente aos
/// dois lados, de raio `raio`.
///
/// Três leis, cada uma MEDIDA na barra dobrada (F39, 2026-09-30):
/// - **Um vinco é um vértice que VIRA MAIS do que virava no desenho.** Os nós saem da união com as
///   coordenadas exactas (distância `0e0`) — ⛔ mas o motor ENCAIXA o cruzamento num nó vizinho
///   quando ele cai dentro da precisão dele: na pose das fotos o vinco de `161°` sai EXACTAMENTE
///   sobre um nó que no desenho era liso. Nem a proximidade (o assado tem nós a `~0,01` junto da
///   junta, e um caía dentro da solda) nem a igualdade ao bit servem sozinhas: a pergunta é se o nó
///   JÁ virava assim.
/// - **Só o CÔNCAVO:** o vinco de uma dobra é sempre côncavo (dois membros que se unem); uma quina
///   convexa nascida no cruzamento fica como está — e a régua da ÁREA o prova (arredondar um
///   côncavo só ACRESCENTA área).
/// - **O corte mede-se ao longo do CONTORNO:** junto da junta o assado tem segmentos de `~0,01`,
///   e um arco preso ao primeiro deles saía minúsculo — o defeito dos pedaços soltos outra vez. Os
///   nós lisos dentro do arco são engolidos; ⛔ uma quina do ARTISTA nunca (o arco encolhe antes).
#[must_use]
pub fn arredonda_os_vincos(
    verts: Vec<VecVertex>,
    originais: &[([f64; 2], f64)],
    raio: f64,
) -> Vec<VecVertex> {
    use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point};
    let n = verts.len();
    if n < 3 || raio <= 0.0 {
        return verts;
    }
    // A viragem que o nó tinha no DESENHO, se o vértice é (ao bit) um nó dele.
    let antes = |a: [f64; 2]| {
        originais
            .iter()
            .find(|(o, _)| {
                (o[0] - a[0]).abs() <= 1e-12 * (1.0 + a[0].abs())
                    && (o[1] - a[1]).abs() <= 1e-12 * (1.0 + a[1].abs())
            })
            .map(|(_, v)| *v)
    };
    // A orientação do contorno (a área com sinal pelas âncoras chega para o SINAL).
    let area2: f64 = (0..n)
        .map(|i| {
            let (a, b) = (verts[i].anchor, verts[(i + 1) % n].anchor);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    let concavo = |i: usize| {
        tangentes(&verts, i).is_some_and(|(e, s)| (e[0] * s[1] - e[1] * s[0]) * area2 < 0.0)
    };
    let vira = |i: usize| viragem(&verts, i).unwrap_or(0.0);
    // Vira mais do que virava: um cruzamento novo, ou um nó liso sobre o qual o motor o encaixou.
    let novo = |i: usize| antes(verts[i].anchor).is_none_or(|v| vira(i) > v + VINCO_MINIMO);
    let vinco: Vec<bool> = (0..n)
        .map(|i| vira(i) > VINCO_MINIMO && concavo(i) && novo(i))
        .collect();
    if !vinco.contains(&true) {
        return verts;
    }
    // Onde o arco NÃO pode entrar: outro vinco, ou uma quina a sério do desenho (as micro-quinas
    // de `1,4°`–`1,7°` que o assado deixa nos nós são engolidas).
    let parede: Vec<bool> = (0..n)
        .map(|i| vinco[i] || (vira(i) > PAREDE_MINIMA && antes(verts[i].anchor).is_some()))
        .collect();
    let pt = |a: [f64; 2]| Point::new(a[0], a[1]);
    let arr = |p: Point| [p.x, p.y];
    let seg = |j: usize| {
        let (c, q) = (&verts[j], &verts[(j + 1) % n]);
        CubicBez::new(
            pt(c.anchor),
            pt(c.out_handle),
            pt(q.in_handle),
            pt(q.anchor),
        )
    };
    let dist = |j: usize, v: Point| pt(verts[j].anchor).distance(v);
    // O parâmetro do segmento onde ele está a `d` de `v`, entre a ponta `perto_t` (a menos de `d`)
    // e a `longe_t` (a `d` ou mais).
    let corte = |c: &CubicBez, v: Point, d: f64, mut perto_t: f64, mut longe_t: f64| {
        for _ in 0..52 {
            let m = 0.5 * (perto_t + longe_t);
            if c.eval(m).distance(v) < d {
                perto_t = m;
            } else {
                longe_t = m;
            }
        }
        0.5 * (perto_t + longe_t)
    };
    // Anda a partir do vinco `i` num sentido até ao 1.º nó a `d` ou mais; devolve o nó, ou a PAREDE
    // onde parou.
    let anda = |i: usize, d: f64, passo: usize| -> (usize, bool) {
        let v = pt(verts[i].anchor);
        let mut u = i;
        for _ in 1..n {
            u = (u + passo) % n;
            if parede[u] {
                return (u, false);
            }
            if dist(u, v) >= d {
                return (u, true);
            }
        }
        (u, false)
    };
    let mut t_ini = vec![0.0; n];
    let mut t_fim = vec![1.0; n];
    let mut some = vec![false; n];
    let mut sb_de = vec![0; n];
    let mut sf_de = vec![0; n];
    for i in (0..n).filter(|&i| vinco[i]) {
        let v = pt(verts[i].anchor);
        let alfa = viragem(&verts, i).unwrap_or(0.0).to_radians();
        // O corte fica a `r·tan(α/2)` do vinco, e NUNCA a menos de `r`: um vinco que mal vira
        // (medido na dobra em C a `95°`: poucos graus) dava um arco de `0,0013`, dentro da solda, e
        // a junta seria calculada sobre uma tangente arbitrária. Com o piso, um vinco raso ganha
        // um arco largo e suave (raio `r/tan(α/2)`), e a partir de `90°` o piso não pesa.
        let mut d = raio * (0.5 * alfa).tan().max(1.0);
        // Uma parede no caminho encolhe o arco para não passar de metade da distância até ela; um
        // nó liso que fica LOGO ALÉM do corte é engolido (o arco cresce até o deixar a `folga`), senão
        // sobra entre os dois um segmento do tamanho da solda — medido na pose das fotos a `150°`:
        // o 1.º nó além do corte ficava a `0,0008` dele.
        let folga = FOLGA_DO_CORTE * raio;
        let mut tecto = f64::INFINITY;
        for _ in 0..16 {
            let (b, okb) = anda(i, d, n - 1);
            let (f, okf) = anda(i, d, 1);
            for (u, ok) in [(b, okb), (f, okf)] {
                if !ok {
                    tecto = tecto.min(0.45 * dist(u, v));
                }
            }
            if d > tecto {
                d = tecto;
                continue;
            }
            let cresce = [(b, okb), (f, okf)]
                .into_iter()
                .filter(|&(u, ok)| ok && dist(u, v) - d < folga)
                .map(|(u, _)| dist(u, v) + folga)
                .fold(d, f64::max);
            if cresce <= d || cresce > tecto {
                break;
            }
            d = cresce;
        }
        let (ub, _) = anda(i, d, n - 1);
        let (uf, _) = anda(i, d, 1);
        let (sb, sf) = (ub, (uf + n - 1) % n);
        t_fim[sb] = corte(&seg(sb), v, d, 1.0, 0.0);
        t_ini[sf] = corte(&seg(sf), v, d, 0.0, 1.0);
        sb_de[i] = sb;
        sf_de[i] = sf;
        // Os nós estritamente entre `ub` e `uf` saem (o vinco é trocado por dois).
        let mut k = (ub + 1) % n;
        while k != uf {
            if k != i {
                some[k] = true;
            }
            k = (k + 1) % n;
        }
    }
    let corta = |j: usize| {
        let c = seg(j);
        if t_ini[j] == 0.0 && t_fim[j] == 1.0 {
            c
        } else {
            c.subsegment(t_ini[j]..t_fim[j])
        }
    };
    let unit = |w: kurbo::Vec2, reserva: kurbo::Vec2| {
        let w = if w.hypot() > 1e-12 { w } else { reserva };
        w / w.hypot().max(1e-300)
    };
    let mut out = Vec::with_capacity(n + 8);
    for i in 0..n {
        if some[i] {
            continue;
        }
        if !vinco[i] {
            let mut v = verts[i];
            v.in_handle = arr(corta((i + n - 1) % n).p2);
            v.out_handle = arr(corta(i).p1);
            out.push(v);
            continue;
        }
        let (sb, sf) = (sb_de[i], sf_de[i]);
        let (a, d) = (corta(sb), corta(sf));
        let (pa, pb) = (a.p3, d.p0);
        let ta = unit(seg(sb).deriv().eval(t_fim[sb]).to_vec2(), pa - a.p0);
        let tb = unit(seg(sf).deriv().eval(t_ini[sf]).to_vec2(), d.p3 - pb);
        // O arco circular de varrimento `θ` entre `A` e `B`: alça `(4/3)·tan(θ/4)·r`, com
        // `r = corda / (2·sin(θ/2))`.
        let theta = ta.dot(tb).clamp(-1.0, 1.0).acos();
        let corda = pa.distance(pb);
        let h = if theta > 1e-9 {
            (4.0 / 3.0) * (0.25 * theta).tan() * corda / (2.0 * (0.5 * theta).sin())
        } else {
            corda / 3.0
        };
        let mut va = verts[i];
        va.in_handle = arr(a.p2);
        va.anchor = arr(pa);
        va.out_handle = arr(pa + ta * h);
        let mut vb = verts[i];
        vb.in_handle = arr(pb - tb * h);
        vb.anchor = arr(pb);
        vb.out_handle = arr(d.p1);
        out.push(va);
        out.push(vb);
    }
    for v in &mut out {
        v.kind = crate::classify(v);
    }
    out
}

/// A distância mínima, em raios, entre o corte do arco e o nó liso seguinte — `0,2`: o dobro da
/// solda (`SOLDA_DA_QUINA / RAIO_DO_VINCO = 0,1`), e abaixo do passo dos nós do assado junto da junta
/// (`~0,01` numa peça de `4 m`, ou `0,25` raio), senão o arco cresceria nó a nó até à parede.
const FOLGA_DO_CORTE: f64 = 0.2;

/// Abaixo desta viragem um vértice é uma curva que continua, não uma quina — `1°`.
const VINCO_MINIMO: f64 = 1.0;

/// A viragem a partir da qual um nó do desenho é uma QUINA que o arco não engole — `15°`. Medido:
/// o assado deixa micro-quinas de `1,4°`–`1,7°` nos nós (a costura das tampas), que um arco de
/// `4 cm` pode engolir sem ninguém ver; tratá-las como parede prendia o arco a `45 %` de um
/// segmento de `0,0017` e ele saía minúsculo.
const PAREDE_MINIMA: f64 = 15.0;

/// Os nós do desenho com a viragem que cada um tinha — o que a união NÃO pode tocar.
fn nos_do_desenho(path: &VecPath) -> Vec<([f64; 2], f64)> {
    std::iter::once(&path.verts)
        .chain(path.subpaths.iter().map(|c| &c.verts))
        .flat_map(|vs| (0..vs.len()).map(move |i| (vs[i].anchor, viragem(vs, i).unwrap_or(0.0))))
        .collect()
}

/// A direcção unitária `a → b`, ou `None` se os dois pontos coincidem.
fn direccao(a: [f64; 2], b: [f64; 2]) -> Option<[f64; 2]> {
    let (x, y) = (b[0] - a[0], b[1] - a[1]);
    let l = x.hypot(y);
    (l > 1e-12).then(|| [x / l, y / l])
}

/// Quanto o contorno VIRA no vértice `i`, em graus — pelas tangentes que o traço usa (a alça, e na
/// falta dela o ponto de controlo seguinte, e na falta deste o vizinho).
#[must_use]
pub fn viragem_do_vertice(verts: &[VecVertex], i: usize) -> Option<f64> {
    viragem(verts, i)
}

/// As tangentes de ENTRADA e de SAÍDA do vértice `i`, unitárias, como o traço as lê.
fn tangentes(verts: &[VecVertex], i: usize) -> Option<([f64; 2], [f64; 2])> {
    let n = verts.len();
    let (p, c, q) = (&verts[(i + n - 1) % n], &verts[i], &verts[(i + 1) % n]);
    let ent = direccao(c.in_handle, c.anchor)
        .or_else(|| direccao(p.out_handle, c.anchor))
        .or_else(|| direccao(p.anchor, c.anchor))?;
    let sai = direccao(c.anchor, c.out_handle)
        .or_else(|| direccao(c.anchor, q.in_handle))
        .or_else(|| direccao(c.anchor, q.anchor))?;
    Some((ent, sai))
}

fn viragem(verts: &[VecVertex], i: usize) -> Option<f64> {
    let (ent, sai) = tangentes(verts, i)?;
    Some(
        (ent[0] * sai[0] + ent[1] * sai[1])
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees(),
    )
}

/// ⭐⭐ **O contorno cruza-se?** — os contornos achatados, e um par de segmentos que se ATRAVESSA.
/// Poda por varredura em `x`, para não pagar o quadrado inteiro.
///
/// ⚠️ **Não há salto de vizinhos, e não é descuido:** dois segmentos consecutivos partilham o
/// extremo com os MESMOS bits, logo um dos quatro testes de lado subtrai um ponto de si mesmo e dá
/// **zero exacto** — e o teste estrito de [`atravessa`] recusa-o. Um salto explícito foi escrito e
/// uma mutação que o apagava SOBREVIVEU: *uma linha que a mutação não consegue matar não é lei.*
#[must_use]
pub fn crosses_itself(bez: &BezPath) -> bool {
    let caixa = bez.bounding_box();
    let diag = caixa.width().hypot(caixa.height());
    if !diag.is_finite() || diag <= 0.0 {
        return false;
    }
    let mut segs: Vec<([f64; 2], [f64; 2])> = Vec::new();
    let (mut ini, mut ult) = ([0.0; 2], [0.0; 2]);
    let mut aberto = false;
    let fecha = |segs: &mut Vec<_>, ult: [f64; 2], ini: [f64; 2]| {
        if ult != ini {
            segs.push((ult, ini));
        }
    };
    kurbo::flatten(bez.iter(), diag * DETECTION_TOLERANCE, |el| match el {
        PathEl::MoveTo(p) => {
            if aberto {
                fecha(&mut segs, ult, ini);
            }
            aberto = true;
            ini = [p.x, p.y];
            ult = ini;
        }
        PathEl::LineTo(p) => {
            let q = [p.x, p.y];
            if q != ult {
                segs.push((ult, q));
                ult = q;
            }
        }
        PathEl::ClosePath => {
            if aberto {
                fecha(&mut segs, ult, ini);
                aberto = false;
            }
        }
        // O `flatten` só emite rectas.
        PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
    });
    if aberto {
        fecha(&mut segs, ult, ini);
    }
    segs.sort_by(|a, b| a.0[0].min(a.1[0]).total_cmp(&b.0[0].min(b.1[0])));
    for (i, s) in segs.iter().enumerate() {
        let xmax = s.0[0].max(s.1[0]);
        for t in &segs[i + 1..] {
            if t.0[0].min(t.1[0]) > xmax {
                break;
            }
            if atravessa(s.0, s.1, t.0, t.1) {
                return true;
            }
        }
    }
    false
}

/// Dois segmentos atravessam-se de verdade (os extremos de cada um em lados estritamente opostos
/// do outro). ⚠️ Um toque num extremo NÃO conta — dois contornos que se tocam num ponto não
/// pedem silhueta.
fn atravessa(a1: [f64; 2], a2: [f64; 2], b1: [f64; 2], b2: [f64; 2]) -> bool {
    let lado = |o: [f64; 2], u: [f64; 2], v: [f64; 2]| {
        (u[0] - o[0]).mul_add(v[1] - o[1], -((v[0] - o[0]) * (u[1] - o[1])))
    };
    let (d1, d2) = (lado(a1, a2, b1), lado(a1, a2, b2));
    let (d3, d4) = (lado(b1, b2, a1), lado(b1, b2, a2));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

#[cfg(test)]
#[path = "overlap_tests.rs"]
mod tests;
