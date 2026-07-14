//! **A CORRESPONDÊNCIA** — que ponto de A vira que ponto de B.
//!
//! É o problema difícil (interpolar é o fácil), e o que separa este motor de uma rotação rígida.
//! Módulo irmão do [`super`] pelo teto de LOC.
//!
//! # Por que uma ROTAÇÃO não basta (o smoke do Enio)
//!
//! A 1ª versão casava as duas formas por uma **rotação rígida**: `arco_B = arco_A − fase`. Ela
//! escolhe bem a fase, mas tem um teto **estrutural**, e o Enio bateu nele:
//!
//! > O quadrado tem quina a cada **0,25** de perímetro; a estrela, vértice a cada **0,10**.
//! > Sob uma rotação, **no máximo UMA quina do quadrado cai sobre um vértice da estrela** — as
//! > outras três caem no MEIO de uma aresta dela.
//!
//! O resultado: as pontas da estrela nascem do meio das arestas retas do quadrado, e o
//! intermediário sai amassado. Não é um bug de implementação — é o limite de uma rotação.
//!
//! # O que este motor faz
//!
//! **Só uma QUINA é candidata a nó** ([`features`]) — e essa frase é a correção do 2º smoke. Uma
//! âncora de contorno **suave** não é uma feature, é artefato da parametrização (as 4 âncoras do
//! círculo do catálogo existem porque a elipse é cozida em 4 cúbicas), e obrigar uma quina a casar
//! com uma delas **inventa uma rotação** que a geometria não pediu.
//!
//! Daí os **dois regimes**, e eles são a mesma pergunta com respostas diferentes:
//!
//! - **As duas formas têm quina** → os nós saem de uma **programação dinâmica** (o núcleo do
//!   Sederberg & Greenwood 1992): um casamento **cíclico e monótono** entre as **quinas**, que
//!   minimiza (1) a **distância entre as casadas**, cada forma normalizada (centro + escala) — é
//!   isto que faz uma quina do quadrado procurar uma PONTA da estrela, e não um vale dela; (2) o
//!   **bending** (a virada como par `(sen, cos)`) — convexa com reentrante é diferença de TIPO; e
//!   (3) a **distorção de arco**: dois nós vizinhos devem cobrir frações de perímetro parecidas nas
//!   duas formas, senão o mapa espreme metade de uma forma num canto da outra.
//! - **Um dos dois não tem quina nenhuma** (um círculo, um blob) → não há nó a escolher, e a
//!   correspondência é o que ela sempre foi ali: uma **fase contínua** ([`phase_only`]), achada por
//!   correlação e livre para cair no MEIO de um segmento.
//!
//! As âncoras que sobram (a estrela tem 10, o quadrado 4) **subdividem** a forma menor: a
//! pré-imagem de cada vértice não-casado da estrela vira um ponto novo na aresta do quadrado. É
//! exatamente a subdivisão que faltava.

use super::{COST_SAMPLES, MERGE_EPS, Outline};
use crate::BlendOpts;
use kurbo::Point;

/// A correspondência escolhida: os **nós** que amarram as duas formas, e o sentido de percurso.
///
/// Cada nó é um par `(arco em A, arco em B)`, e entre dois nós o arco é mapeado **proporcional**.
/// Ou seja: a correspondência é um **mapa monótono por partes**, não uma rotação.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Correspondence {
    /// Os nós `(u_a, u_b)`, em ordem crescente de `u_a`, cíclicos e monótonos.
    pub(crate) knots: Vec<(f64, f64)>,
    pub(crate) reversed: bool,
}

/// Teto do custo da programação dinâmica (`n · m³`). Acima dele o casamento cai para a **fase**
/// ([`phase_only`]) — uma forma com centenas de quinas não tem "a" quina a casar, e o custo
/// explodiria.
const DP_BUDGET: usize = 20_000_000;

/// Quantas fases o varrimento contínuo testa.
///
/// A resolução é 1/256 do perímetro; o refino parabólico (três amostras em volta do mínimo) desce
/// bem abaixo disso. Num círculo, 1/256 de volta é 1,4° — e o defeito que este caminho conserta
/// media **45°**.
const PHASE_STEPS: usize = 256;

#[cfg(test)]
thread_local! {
    /// Quantas vezes a correspondência foi buscada **nesta thread**.
    ///
    /// É o que deixa o gate de "uma busca por blend, não uma por passo" ser **exato** em vez de
    /// um cronômetro (que mede a máquina, não o código). Cada teste roda na sua própria thread,
    /// então o contador não tem corrida com os vizinhos.
    pub(crate) static SEARCH_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn search(oa: &Outline, ob: &Outline, opts: BlendOpts) -> Correspondence {
    #[cfg(test)]
    SEARCH_COUNT.with(|c| c.set(c.get() + 1));
    let ob_rev = ob.reversed();
    // **O SENTIDO é SEMPRE o automático** (o de menor custo de percurso), e não um toggle do
    // usuário.
    //
    // Houve um "Reverse Match" manual, e ele foi REMOVIDO por ser um bug de design: inverter o
    // sentido de percurso de B inverte o WINDING do resultado, e interpolar entre dois windings
    // opostos **colapsa a forma no meio do caminho** (a área passa por zero). Como todo o catálogo
    // nasce com o mesmo winding, o botão colapsava em 100% dos casos — provado em estrela→círculo,
    // quadrado→estrela e círculo→círculo. E ele era **redundante**: quando B de fato tem winding
    // oposto (uma forma importada/desenhada ao contrário), este `auto_reversed` já a percorre no
    // sentido certo sozinho. O gate `opposite_winding_does_not_collapse_the_middle` prova isso.
    let (cost_fwd, knots_fwd) = align(oa, ob, 0);
    let (cost_rev, knots_rev) = align(oa, &ob_rev, 0);
    let reversed = cost_rev < cost_fwd;

    if opts.offset == 0 {
        let knots = if reversed { knots_rev } else { knots_fwd };
        return Correspondence { knots, reversed };
    }
    let target = if reversed { &ob_rev } else { ob };
    let (_, knots) = align(oa, target, opts.offset);
    Correspondence { knots, reversed }
}

/// O casamento cíclico monótono entre as **quinas** de `oa` e `ob`, e o custo dele.
///
/// `offset` roda o nó inicial em B — é o **escape manual** (o `shapeIndex` do GSAP / *Map Nodes*
/// do Corel), agora com um significado exato: *"case a minha primeira quina com a k-ésima dela"*.
///
/// # Os nós são as QUINAS, não as âncoras
///
/// É a correção do 2º smoke ([`features`]). Uma âncora de contorno suave não é uma feature — é
/// artefato da parametrização —, e obrigar uma quina a casar com uma delas **inventa uma rotação**.
/// Quando um dos dois lados não tem quina nenhuma para oferecer, não há nó a escolher: a
/// correspondência é uma **fase contínua** ([`phase_only`]), e é ela que o varrimento acha.
pub(crate) fn align(oa: &Outline, ob: &Outline, offset: i32) -> (f64, Vec<(f64, f64)>) {
    let (ua_all, ub_all) = (oa.anchors(), ob.anchors());
    if ua_all.is_empty() || ub_all.is_empty() {
        return (f64::INFINITY, vec![(0.0, 0.0)]);
    }
    // Caminho aberto: as pontas são as pontas, e o mapa é a identidade de arco.
    if !oa.closed || !ob.closed {
        return (travel_cost(oa, ob, &[(0.0, 0.0)]), vec![(0.0, 0.0)]);
    }

    let (fa, fb) = (features(oa), features(ob));
    let (n, m) = (fa.len(), fb.len());
    // **Um dos dois não tem quina** (um círculo, um blob): não há nó a casar, e o mapa é a fase.
    // Também é aqui que cai a forma com quinas DEMAIS (fora do orçamento da DP): centenas de
    // quinas não são "a" quina, e o contorno volta a ser tratado como o que ele é — uma curva.
    if n == 0 || m == 0 || n.min(m).saturating_mul(n.max(m).saturating_pow(3)) > DP_BUDGET {
        return phase_only(oa, ob, offset);
    }

    // As âncoras, NORMALIZADAS (centro + escala): o custo compara FORMA, não posição no mundo. O
    // quadro (centro + escala) é o do CONTORNO — amostras de arco, não a média das âncoras (ver
    // `normalized`) —, e as quinas são peneiradas DEPOIS: o centro de uma estrela não é a média
    // das pontas dela.
    let (pa_all, pb_all) = (normalized(oa), normalized(ob));
    // E a VIRADA de cada âncora: é ela que impede uma quina convexa de casar com um vale.
    let (ta_all, tb_all) = (turns(oa), turns(ob));
    let pick = |ix: &[usize], u: &[f64], p: &[Point], t: &[(f64, f64)]| {
        (
            ix.iter().map(|&i| u[i]).collect::<Vec<f64>>(),
            ix.iter().map(|&i| p[i]).collect::<Vec<Point>>(),
            ix.iter().map(|&i| t[i]).collect::<Vec<(f64, f64)>>(),
        )
    };
    let (ua, pa, ta) = pick(&fa, &ua_all, &pa_all, &ta_all);
    let (ub, pb, tb) = pick(&fb, &ub_all, &pb_all, &tb_all);
    // A forma com MENOS quinas é a que casa todas as suas; as que sobram na outra subdividem.
    let swapped = n > m;
    let (ua, ub, pa, pb, ta, tb) = if swapped {
        (ub, ua, pb, pa, tb, ta)
    } else {
        (ua, ub, pa, pb, ta, tb)
    };
    let (n, m) = (ua.len(), ub.len());

    // O AUTOMÁTICO primeiro: qual âncora de B casa com a 1ª de A?
    let mut best: Option<(f64, usize, Vec<usize>)> = None;
    for c in 0..m {
        if let Some((cost, js)) = dp_from(&ua, &ub, &pa, &pb, &ta, &tb, c)
            && best.as_ref().is_none_or(|(bc, _, _)| cost < *bc)
        {
            best = Some((cost, c, js));
        }
    }
    let Some((mut cost, c_auto, mut js)) = best else {
        return phase_only(oa, ob, offset);
    };
    // **O escape manual é RELATIVO ao automático**, e isso não é detalhe: fosse absoluto, o dia em
    // que o automático já escolhesse aquele nó o botão **não faria nada** — e um escape que às
    // vezes é inerte é pior que escape nenhum (o artista conclui que a ferramenta travou).
    if offset != 0 {
        // A soma é em **i64**: `c_auto + offset` estoura o `i32` quando o `offset` chega perto do
        // teto (pânico em debug, wrap silencioso em release — e o `offset` é um campo PÚBLICO do
        // `BlendOpts`). Em i64 a soma de dois i32 nunca estoura, e o `rem_euclid` a traz de volta.
        let c = usize::try_from(
            (i64::from(c_auto_i32(c_auto)) + i64::from(offset))
                .rem_euclid(i64::try_from(m).unwrap_or(1)),
        )
        .unwrap_or(0);
        let Some((c2, js2)) = dp_from(&ua, &ub, &pa, &pb, &ta, &tb, c) else {
            return phase_only(oa, ob, offset);
        };
        cost = c2;
        js = js2;
    }
    let mut knots: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            if swapped {
                (ub[js[i]], ua[i])
            } else {
                (ua[i], ub[js[i]])
            }
        })
        .collect();
    knots.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
    (cost, knots)
}

/// **A FASE CONTÍNUA** — a correspondência de quem não tem quina para oferecer.
///
/// Um nó só, `(0, φ)`: o mapa vira `arco_B = arco_A + φ`. A forma da resposta é a mesma da rotação
/// que existia aqui antes — **o que muda é o conjunto onde ela é procurada.**
///
/// # O que estava errado (e o gate que o pegou)
///
/// A versão anterior só testava as fases que **alinham uma âncora de A com uma âncora de B**. Num
/// contorno suave as âncoras são artefato da parametrização, e a fase certa quase nunca é uma
/// delas: ela cai no **meio de um segmento**. Dois círculos iguais, com a origem do percurso
/// deslocada de 45°, encolhiam para **raio 0,924** no meio do caminho — os pontos atravessavam o
/// disco em vez de caminhar radialmente. Ninguém tinha visto, porque as elipses do catálogo nascem
/// todas com as âncoras nas mesmas posições de arco, e o defeito só aparece quando **não** nascem.
///
/// # Como a fase é achada
///
/// É uma **correlação circular**: as duas formas são amostradas em `PHASE_STEPS` posições de arco
/// **iguais**, e o custo de percurso de cada fase `p/N` é a soma dos deslocamentos ao quadrado —
/// que, com as duas amostradas na mesma grade, é um simples deslocamento de índice. O mínimo é
/// refinado por parábola (as três amostras em volta), então a resolução não é a da grade.
///
/// A translação entre as duas formas **não enviesa** a escolha: ela entra no custo como uma
/// constante (o termo cruzado com o deslocamento soma zero sobre o contorno inteiro, porque as
/// duas amostragens são centradas nos respectivos centroides de arco). O que sobra é a correlação
/// de FORMA — que é exatamente o que decide onde a quina do quadrado quer cair no círculo.
fn phase_only(oa: &Outline, ob: &Outline, offset: i32) -> (f64, Vec<(f64, f64)>) {
    let (sa, sb) = (oa.samples(PHASE_STEPS), ob.samples(PHASE_STEPS));
    let cost: Vec<f64> = (0..PHASE_STEPS)
        .map(|p| {
            (0..PHASE_STEPS)
                .map(|k| (sa[k] - sb[(k + p) % PHASE_STEPS]).hypot2())
                .sum()
        })
        .collect();
    let p = (0..PHASE_STEPS).fold(0, |best, i| if cost[i] < cost[best] { i } else { best });
    let phi = (p as f64 + parabolic(&cost, p)) / PHASE_STEPS as f64;

    // **O escape manual, no caso suave.** Aqui um dos dois lados NÃO tem quina, então não há nós
    // discretos a ciclar: girar a fase só torce o morph (o lado liso não oferece posição de
    // encaixe). O quantum vem das **quinas da forma que TEM quinas** — são elas que o artista
    // percebe como pontos de ajuste —, e não das âncoras arbitrárias da lisa.
    //
    // O smoke do Enio (estrela→círculo): o quantum era `1/âncoras-do-círculo` = **1/4 = 90°/toque**,
    // e o 2º toque (180°) **colapsava** a forma. Pelas 10 quinas da estrela dá 36°/toque — passos
    // finos, e o artista sente a torção crescer em vez de colapsar de uma vez. A torção em rotação
    // grande é intrínseca ao lerp de coordenadas (o gap Sederberg/Alexa, ainda aberto).
    let corners = features(oa).len().max(features(ob).len());
    let m = if corners > 0 {
        corners
    } else {
        ob.segs.len().max(1) // as duas lisas (círculo→círculo): girar não muda nada visível
    };
    let phi = wrap(phi + f64::from(offset) / m as f64);
    let knots = vec![(0.0, phi)];
    (travel_cost(oa, ob, &knots), knots)
}

/// O vértice da parábola pelas três amostras em volta de `p` (cíclicas) — em **frações de amostra**.
///
/// Sem ele a fase ficaria presa na grade (1/256 do perímetro); com ele, o mínimo é o do contínuo,
/// e não custa uma avaliação a mais.
fn parabolic(cost: &[f64], p: usize) -> f64 {
    let n = cost.len();
    let (l, c, r) = (cost[(p + n - 1) % n], cost[p], cost[(p + 1) % n]);
    let denom = l - 2.0 * c + r;
    if denom.abs() <= f64::EPSILON {
        return 0.0; // platô (duas formas iguais e concêntricas): a amostra já é o mínimo
    }
    (0.5 * (l - r) / denom).clamp(-0.5, 0.5)
}

/// O custo de percurso sob um mapa (o critério do flubber): o quanto os pontos andam.
fn travel_cost(oa: &Outline, ob: &Outline, knots: &[(f64, f64)]) -> f64 {
    (0..COST_SAMPLES)
        .map(|k| {
            let u = k as f64 / COST_SAMPLES as f64;
            (oa.at(u) - ob.at(map_forward(knots, u))).hypot2()
        })
        .sum()
}

/// A **virada** em cada âncora, como o par `(sen, cos)` do ângulo — sem `atan2`.
///
/// Comparar dois pares `(sen, cos)` é comparar dois vetores unitários: a distância entre eles é
/// monótona na diferença de ângulo, e não passa por transcendental nenhum (HR-5). Numa âncora
/// suave (um círculo) a virada é `(0, 1)` — o termo não perturba quem não tem quina.
pub(crate) fn turns(o: &Outline) -> Vec<(f64, f64)> {
    let n = o.segs.len();
    (0..n)
        .map(|i| {
            if !o.closed && i == 0 {
                return (0.0, 1.0); // ponta de caminho aberto: não há virada
            }
            let prev = o.segs[(i + n - 1) % n];
            let cur = o.segs[i];
            let (Some(tin), Some(tout)) = (unit(tangent_end(&prev)), unit(tangent_start(&cur)))
            else {
                return (0.0, 1.0);
            };
            (
                tin.x * tout.y - tin.y * tout.x, // sen (o SINAL diz de que lado a quina vira)
                tin.x * tout.x + tin.y * tout.y, // cos
            )
        })
        .collect()
}

/// **O que é uma FEATURE** — o limiar, em cosseno da virada.
///
/// `cos(16°)`. Abaixo disto a âncora é uma **quina de verdade**; acima, o contorno passa por ela
/// praticamente reto. Medido no catálogo: quadrado `cos 0,00` · estrela `cos −0,70` (ponta) e
/// `+0,46` (vale) · coração `cos −0,42`/`+0,37` — e o **círculo, `cos 1,00` em TODAS as quatro**.
/// A separação é gritante; o limiar não é um botão a calibrar, é uma cerca num vale vazio.
///
/// Ele é escrito em **cosseno** (não em seno) de propósito: uma **cúspide** vira ~180°, onde o seno
/// volta a zero. Fosse `|sen| > ε`, o bico mais afiado que existe seria classificado como suave.
///
/// # 16°, e não 15° — porque um limiar mora onde o domínio é VAZIO
///
/// A 1ª versão usou `cos(15°)`, e **um polígono regular de 24 lados vira exatamente 15°**. A
/// comparação é estrita, o `f64` erra o cosseno no último bit, e cada uma das **24 quinas
/// IDÊNTICAS** caía de um lado ou do outro conforme o arredondamento. Medido: perturbar o raio em
/// `1e-13` dava **68 resultados distintos** (13,6% de área); **transladar a cena** — só mover, sem
/// deformar nada — mudava a forma do intermediário em **5,5%**. O artista arrastava a arte pela tela
/// e o blend mudava.
///
/// Um polígono regular de N lados vira `360/N`. **Então o limiar tem de ser um ângulo cujo `360/θ`
/// NÃO seja inteiro** — e aí nenhuma quantidade de polígonos regulares consegue sentar em cima da
/// cerca. `360/16 = 22,5`: os vizinhos mais próximos são o 22-gon (16,36°) e o 23-gon (15,65°), a
/// **13 ordens de grandeza** do ruído do `f64`. O gate
/// `the_corner_threshold_does_not_sit_on_a_polygon_the_catalog_can_make` **enumera** N até o teto do
/// catálogo (`MAX_POLYGON_SIDES = 128`) e é ele que impede o próximo a mexer aqui de reintroduzir o
/// empate.
///
/// É a mesma lição que o z-order desta linha já pagou: **não se escolhe um desempate melhor — não
/// se tem empate.**
pub(crate) const FEATURE_TURN_COS: f64 = 0.961_261_695_938_318_9;

/// As âncoras que são **quinas de verdade** — as únicas candidatas a NÓ da correspondência.
///
/// # Por que isto existe (o 2º smoke do Enio: *"o porquê da rotação?"*)
///
/// Um quadrado a caminho de um círculo **rodava 45°**. A causa não estava na busca: estava no
/// **conjunto de candidatos**. As 4 âncoras do círculo existem porque a elipse é **cozida em 4
/// cúbicas** — o artista nunca as autorou, e a virada delas é `(0, 1)`, perfeitamente suave. Não há
/// nada ali para casar. Mas o motor obrigava cada quina do quadrado a casar com uma delas, e a
/// resposta certa (a quina a 45° → o ponto do círculo a 45°, que cai **no MEIO de um segmento**)
/// **não estava no conjunto**. O melhor de quatro casamentos ruins é o giro de 45°.
///
/// Com esta peneira, um contorno suave devolve **zero** nós — e a correspondência dele deixa de ser
/// uma escolha entre âncoras para ser o que ela sempre foi: uma **fase contínua** ([`phase_only`]).
pub(crate) fn features(o: &Outline) -> Vec<usize> {
    turns(o)
        .iter()
        .enumerate()
        .filter(|(_, (_, cos))| *cos < FEATURE_TURN_COS)
        .map(|(i, _)| i)
        .collect()
}

/// A tangente na SAÍDA da âncora (o 1º controle que não coincide com ela).
fn tangent_start(s: &kurbo::CubicBez) -> kurbo::Vec2 {
    for c in [s.p1, s.p2, s.p3] {
        let v = c - s.p0;
        if v.hypot() > MERGE_EPS {
            return v;
        }
    }
    kurbo::Vec2::ZERO
}

/// A tangente na CHEGADA da âncora seguinte.
fn tangent_end(s: &kurbo::CubicBez) -> kurbo::Vec2 {
    for c in [s.p2, s.p1, s.p0] {
        let v = s.p3 - c;
        if v.hypot() > MERGE_EPS {
            return v;
        }
    }
    kurbo::Vec2::ZERO
}

fn unit(v: kurbo::Vec2) -> Option<kurbo::Vec2> {
    let len = v.hypot();
    (len > MERGE_EPS).then(|| v / len)
}

/// As âncoras normalizadas: centro na origem, escala RMS 1. O custo passa a comparar **forma**.
///
/// # O centro e a escala são da FORMA — não da lista de âncoras
///
/// Eles saíam da **média das âncoras**, e âncora é **parametrização**: picar uma aresta reta em 20
/// pedaços não muda nada do que se vê, mas arrasta essa média para o lado da aresta picada. Medido:
/// o mesmo quadrado, com uma aresta subdividida, casava as quinas com **outros** vértices da
/// estrela (`0,2` → `0,0`, e com 20 pedaços a correspondência inteira embaralhava).
///
/// É um defeito real e cotidiano — um caminho traçado, importado ou passado por um `Simplify` tem
/// âncoras onde o algoritmo as deixou, não onde a forma pede. **Duas formas que se veem iguais têm
/// de blendar igual.**
///
/// O quadro agora sai de amostras **equiespaçadas em ARCO** (as mesmas de [`super::Outline::
/// samples`]): elas descrevem o contorno, e não a escolha de quem desenhou. As âncoras continuam
/// sendo as candidatas — o que mudou é o **referencial** em que elas são medidas.
fn normalized(o: &Outline) -> Vec<Point> {
    let walk = o.samples(SHAPE_FRAME_SAMPLES);
    let n = walk.len() as f64;
    let cx = walk.iter().map(|p| p.x).sum::<f64>() / n;
    let cy = walk.iter().map(|p| p.y).sum::<f64>() / n;
    let rms = (walk
        .iter()
        .map(|p| (p.x - cx).powi(2) + (p.y - cy).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
        .max(MERGE_EPS);
    o.segs
        .iter()
        .map(|s| Point::new((s.p0.x - cx) / rms, (s.p0.y - cy) / rms))
        .collect()
}

/// Quantas amostras de arco definem o **quadro** (centro + escala) de uma forma.
const SHAPE_FRAME_SAMPLES: usize = 256;

/// O peso da distorção de arco no custo. As parcelas são adimensionais e da mesma ordem
/// (distâncias normalizadas ~1; frações de perímetro ~1/n; viradas em (sen, cos) ~1), então `1.0`
/// é um empate honesto — e não um botão a calibrar.
const STRETCH_WEIGHT: f64 = 1.0;

/// O peso da **forma da quina** no custo — o termo de *bending* do Sederberg & Greenwood.
///
/// Sem ele, o custo só olha POSIÇÃO, e o smoke do Enio mostrou o preço: **duas das quatro quinas
/// do quadrado casavam com os VALES da estrela** (os vértices reentrantes), porque o vale estava
/// angularmente mais perto do que a ponta seguinte. Na tela: o quadrado **colapsa pelas quinas**
/// enquanto as pontas nascem do meio das arestas retas — o "amassado" que o Enio viu.
///
/// Uma quina **convexa** tem de casar com uma **convexa**. É diferença de TIPO, não de grau: a
/// quina do quadrado vira +90°, a ponta da estrela +144°, e o vale vira para o **outro lado**.
/// Comparar convexa com reentrante custa 4,7× mais caro do que comparar duas convexas, e é isso
/// que reordena o casamento inteiro.
const TURN_WEIGHT: f64 = 1.0;

/// A programação dinâmica, com o 1º nó fixado em `(0, c)`: escolhe `j_1 < j_2 < … < j_{n-1}`
/// (ciclicamente crescentes) minimizando distância-entre-casados + distorção-de-arco.
#[allow(clippy::too_many_arguments)] // cada entrada é um eixo distinto do custo
fn dp_from(
    ua: &[f64],
    ub: &[f64],
    pa: &[Point],
    pb: &[Point],
    ta: &[(f64, f64)],
    tb: &[(f64, f64)],
    c: usize,
) -> Option<(f64, Vec<usize>)> {
    let (n, m) = (ua.len(), ub.len());
    if n > m {
        return None;
    }
    let du = |i: usize| wrap_pos(ua[(i + 1) % n] - ua[i]);
    let dv = |o0: usize, o1: usize| wrap_pos(ub[(c + o1) % m] - ub[(c + o0) % m]);
    let pair = |i: usize, o: usize| {
        let j = (c + o) % m;
        let pos = (pa[i] - pb[j]).hypot2();
        // A distância entre as duas VIRADAS, como vetores unitários: convexa com convexa é barato,
        // convexa com reentrante é caro. É uma diferença de TIPO de quina.
        let ((sa, ca), (sb, cb)) = (ta[i], tb[j]);
        let bend = (sa - sb).powi(2) + (ca - cb).powi(2);
        pos + TURN_WEIGHT * bend
    };
    let stretch = |i: usize, o0: usize, o1: usize| {
        let d = du(i) - dv(o0, o1);
        STRETCH_WEIGHT * d * d
    };

    // dp[i][o] = melhor custo de casar a âncora `i` de A com o offset `o` de B (a partir de c).
    let inf = f64::INFINITY;
    let mut dp = vec![vec![inf; m]; n];
    let mut from = vec![vec![0usize; m]; n];
    dp[0][0] = pair(0, 0);
    for i in 1..n {
        for o in i..=(m - (n - i)) {
            for o0 in (i - 1)..o {
                let prev = dp[i - 1][o0];
                if prev.is_infinite() {
                    continue;
                }
                let cost = prev + pair(i, o) + stretch(i - 1, o0, o);
                if cost < dp[i][o] {
                    dp[i][o] = cost;
                    from[i][o] = o0;
                }
            }
        }
    }
    // Fecha o ciclo: do último nó de volta ao primeiro (a volta completa é `m` offsets).
    let mut best: Option<(f64, usize)> = None;
    for (o, cost) in dp[n - 1].iter().enumerate().skip(n - 1) {
        if cost.is_infinite() {
            continue;
        }
        let total = cost + stretch(n - 1, o, m);
        if best.as_ref().is_none_or(|(bc, _)| total < *bc) {
            best = Some((total, o));
        }
    }
    let (cost, mut o) = best?;
    let mut js = vec![0usize; n];
    for i in (0..n).rev() {
        js[i] = (c + o) % m;
        if i > 0 {
            o = from[i][o];
        }
    }
    Some((cost, js))
}

/// O índice do nó automático como `i32` (satura se, por absurdo, não couber — não pode acontecer:
/// `c_auto < m ≤ nº de âncoras`).
#[inline]
fn c_auto_i32(c_auto: usize) -> i32 {
    i32::try_from(c_auto).unwrap_or(i32::MAX)
}

/// Delta de arco cíclico, sempre **positivo** (uma volta completa vale 1).
#[inline]
fn wrap_pos(d: f64) -> f64 {
    let w = wrap(d);
    if w <= MERGE_EPS { 1.0 } else { w }
}

/// O mapa **monótono por partes** A → B: entre dois nós, o arco caminha proporcional.
///
/// Com um nó só ele é exatamente a rotação de antes (`arco_B = arco_A − fase`) — o caso
/// degradado continua sendo o comportamento que já estava provado.
pub(crate) fn map_forward(knots: &[(f64, f64)], u: f64) -> f64 {
    let k = knots.len();
    if k == 0 {
        return wrap(u);
    }
    if k == 1 {
        return wrap(u - knots[0].0 + knots[0].1);
    }
    let u = wrap(u);
    // O trecho que contém `u`: o último nó com `u_a <= u` (ciclicamente).
    let i = knots
        .iter()
        .rposition(|(ua, _)| *ua <= u + MERGE_EPS)
        .unwrap_or(k - 1);
    let (ua0, ub0) = knots[i];
    let (ua1, ub1) = knots[(i + 1) % k];
    let span_a = wrap_pos(ua1 - ua0);
    let span_b = wrap_pos(ub1 - ub0);
    let f = (wrap_pos(u - ua0) / span_a).clamp(0.0, 1.0);
    let f = if wrap(u - ua0) <= MERGE_EPS { 0.0 } else { f };
    wrap(ub0 + f * span_b)
}

/// O inverso do [`map_forward`] (B → A) — os nós lidos ao contrário.
pub(crate) fn map_backward(knots: &[(f64, f64)], v: f64) -> f64 {
    let flipped: Vec<(f64, f64)> = {
        let mut f: Vec<(f64, f64)> = knots.iter().map(|(a, b)| (*b, *a)).collect();
        f.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        f
    };
    map_forward(&flipped, v)
}

#[inline]
pub(crate) fn wrap(s: f64) -> f64 {
    let s = s % 1.0;
    if s < 0.0 { s + 1.0 } else { s }
}
