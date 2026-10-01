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
//! - **A UNIÃO só corre quando o contorno se cruza** ([`crosses_itself`]) — fora do contacto ela
//!   não toca na forma, e o custo é o de uma varredura de segmentos.
//! - ⚠️ **A silhueta da pele ([`silhueta_da_pele`]) já NÃO é só a união** (F41): ela rola SEMPRE
//!   a bola de [`RAIO_DO_VINCO`] por fora do contorno ([`crate::bola`]), antes e depois do
//!   contacto. Fora de um vinco apertado ela devolve `None` e a forma sai ao bit.
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

/// A distância, em fracção da diagonal, abaixo da qual o fecho de um contorno achatado **é** o
/// vértice de partida ([`crosses_itself`]). Um ULP de uma coordenada de `f64` perto de `10` vale
/// `~2e-15`; `1e-12` fica três ordens acima do arredondamento e nove abaixo da tolerância do
/// achatamento — nenhum segmento de verdade é tão curto.
pub const FECHO_EXACTO: f64 = 1e-12;

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
    let mut contornos = groups
        .iter()
        .flatten()
        .filter_map(crate::verts_from_bez)
        .map(|v| solda_os_segmentos_curtos(v, solda))
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

/// ⭐⭐⭐⭐ **A SILHUETA DA PELE** — a porta que o desenho da pele chama em todo quadro: a união
/// quando o contorno se CRUZA ([`resolve_overlap`]) e, SEMPRE, a bola de raio [`RAIO_DO_VINCO`] a
/// rolar por fora do contorno de fora ([`crate::bola::rola_a_bola`]) — nenhum canto interno fica
/// mais apertado que ela, antes e depois do contacto (report do dono de 2026-09-30, a F41).
///
/// `quinas` são as QUINAS DO ARTISTA — cada nó que ele pôs, onde caiu no desenho deformado, com a
/// viragem que tinha EM REPOUSO ([`quinas_de`] para um desenho que é o seu próprio repouso). Só elas
/// ficam em quina; todo o resto do contorno é da bola.
///
/// `None` quando nada muda: o desenho sai como estava, ao bit.
#[must_use]
pub fn silhueta_da_pele(path: &VecPath, quinas: &[([f64; 2], f64)]) -> Option<VecPath> {
    if !path.closed || path.subpaths.iter().any(|c| !c.closed) || path.verts.len() < 3 {
        return None;
    }
    let caixa = crate::to_bez(path).bounding_box();
    let diagonal = caixa.width().hypot(caixa.height());
    if !diagonal.is_finite() || diagonal <= 0.0 {
        return None;
    }
    let solda = SOLDA_DA_QUINA * diagonal;
    let raio = RAIO_DO_VINCO * diagonal;
    // ⭐ F43: os ganchos que só existem na TANGENTE saem ANTES da união. ⛔ Depois dela é tarde
    // (medido a `(110°, 17,5°)`): um zigue-zague de `0,0016` de largura conta como cruzamento, a união
    // reescreve-o num dardo REAL de `~0,03`, e esse já não é ruído.
    // ⭐ F45: e TAMBÉM depois dela. A 1.ª redacção desta passagem saiu porque a mutação que a
    // apagava sobreviveu — mas a varredura que a julgou só dobrava em Z. Com o braço em C e as duas
    // juntas a somar `~238°` (`(130°,108°)` … `(170°,70°)`), a união corta uma cúbica do assado
    // DENTRO da dobra dela e o nó novo sai com a alça `0,035` além dele: uma meia-volta de
    // `171°`–`179°` no contorno de fora, e o traço desenha a meia-lua por cima. ⚠️ Só o contorno de
    // FORA: em `3 540` poses em C nenhuma ilha deixou gancho, e uma passagem sem caso medido seria
    // uma lei sem régua.
    let mut desenho = path.clone();
    desenho.verts = crate::gancho::desfaz_os_ganchos(path.verts.clone(), quinas, solda);
    let unido = resolve_overlap(&desenho).map(|mut u| {
        u.verts = crate::gancho::desfaz_os_ganchos(std::mem::take(&mut u.verts), quinas, solda);
        u
    });
    let base = unido.as_ref().unwrap_or(&desenho);
    let rolado = crate::bola::rola_a_bola(base.verts.clone(), quinas, raio, solda);
    if unido.is_none() && rolado == path.verts {
        return None;
    }
    let mut out = base.clone();
    // ⛔ SEM solda depois da bola: ela fundia o toque com um nó do desenho a `3,3 mm` dele (a
    // solda é `1e-3` da diagonal) e tirava o arco do sítio — o 1.º pedaço saía com raio `0,94 r`,
    // e rolar a bola outra vez lia-o como apertado (medido na dobra em Z a `80°`). A solda existe
    // para os RESTOS da união, que já foram soldados antes; um pedaço curto RECORTADO de uma curva
    // lisa tem as tangentes dela, não uma arbitrária.
    out.verts = rolado;
    // ⭐ F44: as ILHAS que a união deixa (um membro a fechar-se sobre outro) são buracos, e o
    // vinco de um buraco é o canto dele — a bola rola por DENTRO de cada uma. Medido antes da
    // cura: com o contorno de fora a `1,4°` no pior nó, as ilhas viravam até `153°`. ⚠️ Uma ilha
    // onde a bola não cabe em sítio nenhum é cheia INTEIRA pelo fecho — sai ([`crate::ilha`]).
    out.subpaths
        .retain(|c| crate::ilha::a_bola_cabe_dentro(&c.verts, raio));
    if out.subpaths.is_empty() {
        out.fill_rule = path.fill_rule;
    }
    for c in &mut out.subpaths {
        c.verts =
            crate::bola::rola_a_bola_por_dentro(std::mem::take(&mut c.verts), quinas, raio, solda);
    }
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
/// ⛔ **RECUSA MEDIDA (F42): a bola NÃO cresce até à meia-largura do traço.** Hipótese: um arco
/// côncavo de raio `r` traçado com meia-largura `h > r` tem a curva paralela do lado do centro
/// invertida, e o laço dela lê enrolamento zero na regra não-zero (um buraco em fatia). Construída
/// (`raio = max(r, 1,05·h)`) e fotografada na cena `=4` a `80°`–`105°` com e sem ela: as DUAS saem
/// limpas — o traçador do Vello desenha estes arcos (`r ≈ 0,04`, `h = 0,15` da espessura) sem fenda,
/// e as fatias da foto do dono eram o GANCHO da dobra protegido como quina (ver [`quinas_de`]). E
/// ela custava o que o dono tinha acabado de aprovar: o castanho/laranja do lado de dentro passava de
/// raio `r + h` a `2,05·h`.
///
/// ⚠️ O número: pequeno o bastante para o PREENCHIMENTO continuar com o «V» da imagem presa (`4 cm`
/// numa diagonal de `4 m`, `4 %` da espessura da barra da cena), e muito acima da solda (`1e-3`), senão
/// o arco nasceria soldado. O traço desenha-se por cima dele como um arco de raio `r + ½·largura`:
/// arredondado com qualquer junta.
///
/// ⛔⛔ **A F40 arredondava com um filete de TAMANHO fixo, só no cruzamento — e o dono reprovou-o
/// com três fotos** (2026-09-30: *«arredonda demais, não é progressivo … ainda produz artefatos
/// circulares»*). Desde a F41 este raio é o de uma BOLA que rola por fora do contorno
/// ([`crate::bola::rola_a_bola`]): um canto só é tocado onde ela não cabe, o arco cresce com o
/// ângulo em vez de saltar, e o vinco que nasce ANTES do encosto (onde a união não corre) é
/// tratado pela mesma lei. ⚠️ **Divergência declarada:** uma curva côncava LISA do desenho mais
/// apertada que a bola também é alargada — não há correspondência entre o repouso e o assado que
/// permita poupá-la.
pub const RAIO_DO_VINCO: f64 = 1e-2;

/// Abaixo desta viragem um vértice é uma curva que continua, não uma quina — `1°`.
pub(crate) const VINCO_MINIMO: f64 = 1.0;

/// A viragem a partir da qual um nó do desenho é uma QUINA que o arco não engole — `15°`. Medido:
/// o assado deixa micro-quinas de `1,4°`–`1,7°` nos nós (a costura das tampas), que um arco de
/// `4 cm` pode engolir sem ninguém ver; tratá-las como parede prendia o arco a `45 %` de um
/// segmento de `0,0017` e ele saía minúsculo.
pub const PAREDE_MINIMA: f64 = 15.0;

/// ⭐⭐ **As quinas de um desenho que é o seu próprio REPOUSO** — cada nó com a viragem que tem.
///
/// ⛔⛔ **Não sirva isto a um desenho DEFORMADO** (F42, report do dono de 2026-09-30 com duas fotos:
/// *«restam os artefatos de imagem»*). O assado da pele tem nós que o artista nunca pôs, e na DOBRA
/// do mapa um deles vira `180°` (medido na cena `=4` a `85°`: um gancho de `0,015` no vinco, de
/// raio `0,005`). Lido aqui, o gancho passava por quina desenhada, a bola não lhe tocava, e o traço
/// desenhado sobre a meia-volta abria fatias de cinzento e de laranja no castanho. A viragem de
/// uma quina é a do REPOUSO, e quem a sabe é quem assou (o `assa_a_pele_com_nos` da `ph2d-vec-skin`).
#[must_use]
pub fn quinas_de(path: &VecPath) -> Vec<([f64; 2], f64)> {
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
pub(crate) fn tangentes_do_vertice(verts: &[VecVertex], i: usize) -> Option<([f64; 2], [f64; 2])> {
    tangentes(verts, i)
}

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
///
/// ⛔⛔ **A excepção é o passo de UM ULP, e ele acusava um cruzamento que não existe** (medido F41,
/// a pose do dono a `98°`): o achatamento emite o ponto CALCULADO da cúbica perto de `t = 1` e
/// depois o vértice GUARDADO, e os dois podem diferir por **um ULP** — nasce um segmento de
/// `1e-15`, os dois vizinhos dele deixam de ser consecutivos e de partilhar os bits, os testes de
/// lado deixam de dar zero exacto, e dois segmentos quase colineares que se TOCAM leem-se como um
/// par que se ATRAVESSA. O mesmo vale para o FECHO (o último ponto calculado contra o `ini`
/// guardado). ⇒ um passo a menos de [`FECHO_EXACTO`] da diagonal **cola** a ponta do segmento
/// anterior, em vez de nascer como segmento. ⚠️ A primeira cura tratou só o fecho e o gate do dono
/// continuou vermelho: o nó onde o passo nasceu era um vértice LISO no meio do contorno.
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
    let mut n0 = 0usize;
    let cola = diag * FECHO_EXACTO;
    let fecha = |segs: &mut Vec<([f64; 2], [f64; 2])>, n0: usize, ult: [f64; 2], ini: [f64; 2]| {
        if ult == ini {
            return;
        }
        if segs.len() > n0 && (ult[0] - ini[0]).hypot(ult[1] - ini[1]) <= cola {
            if let Some(ultimo) = segs.last_mut() {
                ultimo.1 = ini;
            }
            return;
        }
        segs.push((ult, ini));
    };
    kurbo::flatten(bez.iter(), diag * DETECTION_TOLERANCE, |el| match el {
        PathEl::MoveTo(p) => {
            if aberto {
                fecha(&mut segs, n0, ult, ini);
            }
            aberto = true;
            n0 = segs.len();
            ini = [p.x, p.y];
            ult = ini;
        }
        PathEl::LineTo(p) => {
            let q = [p.x, p.y];
            if q != ult {
                // Um passo de um ULP COLA ao segmento anterior em vez de nascer como segmento —
                // senão os dois vizinhos dele deixam de partilhar os bits.
                let colar = segs.len() > n0 && (q[0] - ult[0]).hypot(q[1] - ult[1]) <= cola;
                match segs.last_mut() {
                    Some(ultimo) if colar => ultimo.1 = q,
                    _ => segs.push((ult, q)),
                }
                ult = q;
            }
        }
        PathEl::ClosePath => {
            if aberto {
                fecha(&mut segs, n0, ult, ini);
                aberto = false;
            }
        }
        // O `flatten` só emite rectas.
        PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
    });
    if aberto {
        fecha(&mut segs, n0, ult, ini);
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
