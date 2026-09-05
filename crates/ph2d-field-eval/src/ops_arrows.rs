//! ⭐⭐ **A FAMÍLIA DA SETA** (W119) — a seta (de uma ou duas pontas), o chevron e a seta dobrada.
//!
//! # Por que um arquivo irmão das chapas
//!
//! O [`crate::ops_plates`] responde *«que contorno cada chapa tem»*; este responde a mesma pergunta
//! para as três que partilham **a ponta** — e é a ponta que as junta: a mesma cunha de três
//! semiplanos aparece nas três, e escrevê-la uma vez por forma seria a mesma lei em três sítios,
//! que este módulo já pagou no `apothem_ratio`.
//!
//! # ⚠️⚠️ A LEI DA SOBREPOSIÇÃO, e é aqui que uma seta nasce partida
//!
//! Uma seta é a **união** da haste com a ponta, e a [`crate::ops::sd_star`] já escreveu o preço
//! disso: `min` de duas peças que se **tocam sem se sobrepor** vale **exactamente zero** na costura,
//! que é um ponto **interior** ao sólido — e um `0` interior é lido como fronteira por quem amostra
//! numa grade. A decomposição óbvia (haste até `xb`, ponta de `xb` em diante) é precisamente uma
//! partição.
//!
//! ⭐ **A cura sai da própria geometria, e não de um épsilon:** a haste avança até ao `x` onde a
//! ponta tem **exactamente a largura dela** ([`frente_da_haste`]). Ali as duas quinas da frente da
//! haste pousam sobre os flancos da ponta, a sobreposição tem área positiva, e a haste **nunca**
//! espreita para fora — que é o que um recuo escolhido à mão não garante.

use fidget::context::Tree;

use crate::ops_joint::Edge;

/// ⭐ **Até onde a haste avança para dentro da ponta** — ver a lei da sobreposição no cabeçalho.
///
/// A ponta tem meia-largura `head` na base (`tip − head_length`) e `0` no bico, logo ela vale
/// `shaft` em `tip − head_length·(shaft/head)`. ⚠️ Com `shaft ≥ head` o documento já recusou a peça.
fn frente_da_haste(tip: f64, head: f64, head_length: f64, shaft: f64) -> f64 {
    tip - head_length * (shaft / head).clamp(0.0, 1.0)
}

/// ⭐ **A CUNHA de uma ponta**, em três semiplanos **normalizados** — os dois flancos e a base.
///
/// `eixo` é a coordenada ao longo da qual a ponta aponta (o `x` de uma seta, o `y` de uma seta
/// dobrada) e `lado` a transversal. ⚠️ **Recebe as duas árvores em vez de as construir**, e é isso
/// que deixa a seta DUPLA dobrar o eixo por `|x|` e receber a segunda ponta de graça.
///
/// Devolve `(flanco_mais, flanco_menos, base)`, por essa ordem — quem chama decide que arestas
/// declara.
fn cunha(eixo: &Tree, lado: &Tree, tip: f64, head: f64, head_length: f64) -> [Tree; 3] {
    // A recta do flanco passa por `(tip, 0)` e `(tip − head_length, head)`; a normal exterior
    // unitária é `(head, head_length)/L`.
    let l = (head * head + head_length * head_length).sqrt();
    let (nx, ny) = (head / l, head_length / l);
    let c = nx * tip;
    [
        eixo.clone() * Tree::constant(nx) + lado.clone() * Tree::constant(ny) - Tree::constant(c),
        eixo.clone() * Tree::constant(nx) - lado.clone() * Tree::constant(ny) - Tree::constant(c),
        // A base olha para trás: `−eixo + (tip − head_length)`, com gradiente unitário.
        Tree::constant(tip - head_length) - eixo.clone(),
    ]
}

/// Um rectângulo 2D **exacto** de meias-extensões `(hx, hy)` centrado em `(cx, cy)`.
fn rect_em(cx: f64, cy: f64, hx: f64, hy: f64) -> Tree {
    let dx = (Tree::x() - Tree::constant(cx)).abs() - Tree::constant(hx);
    let dy = (Tree::y() - Tree::constant(cy)).abs() - Tree::constant(hy);
    crate::ops::length2(&dx.max(0.0), &dy.max(0.0)) + dx.max(dy).min(0.0)
}

/// O mesmo rectângulo **com as quatro quinas arredondadas** — encolher uma distância EXACTA e
/// deslocá-la, que é a única receita que funciona (ver [`crate::ops_plates`], lei da W104).
fn rect_round_em(cx: f64, cy: f64, hx: f64, hy: f64, r: f64) -> Tree {
    let r = r.min(hx * 0.999).min(hy * 0.999).max(0.0);
    crate::ops::offset(&rect_em(cx, cy, hx - r, hy - r), r)
}

/// As quatro paredes de um rectângulo, em semiplanos **separados** — a receita do braço da cruz.
///
/// ⚠️ Ela existe porque a mistura do aro precisa das peças **inteiras**: um rectângulo já composto
/// carrega a bissectriz das quinas para dentro do aro, que é o pior desalinho que este catálogo já
/// mediu (ver [`crate::ops_plates`]).
fn paredes(cx: f64, cy: f64, hx: f64, hy: f64) -> [Tree; 4] {
    [
        Tree::x() - Tree::constant(cx + hx),
        Tree::constant(cx - hx) - Tree::x(),
        Tree::y() - Tree::constant(cy + hy),
        Tree::constant(cy - hy) - Tree::y(),
    ]
}

/// Os quatro pares de quinas de um rectângulo dado em [`paredes`].
fn quinas(p: &[Tree; 4]) -> Vec<(Tree, Tree)> {
    vec![
        (p[0].clone(), p[2].clone()),
        (p[0].clone(), p[3].clone()),
        (p[1].clone(), p[2].clone()),
        (p[1].clone(), p[3].clone()),
    ]
}

/// ⭐⭐ **SETA** — a haste unida à ponta (ou às duas), puxada em Z.
///
/// ⚠️ **`heads == 2` dobra o EIXO por `|x|`**, e a seta dupla sai da mesma fórmula: uma seta de duas
/// pontas é simétrica **por construção**, e a dobra de uma forma simétrica é exacta. É a receita que
/// a [`crate::ops_box::sd_box_frame`] já usa.
///
/// ⚠️ **O bico fica VIVO**, e é a lei que a [`crate::ops_plates::sd_pie`] pagou: encaixar mais uma
/// intersecção arredondada dentro das outras compõe a inflação de cada uma, e o traçador atravessa a
/// superfície. Num bico de seta isso é também o que se quer ver — *uma seta de ponta redonda não
/// aponta*.
// ⚠️ **A assinatura espelha os campos da primitiva UM-A-UM, e é isso que a mantém honesta:** o
// despacho em `primitive_tree.rs` desestrutura a variante pelo nome, então um campo novo é **erro de
// compilação** lá. ⛔ Agrupar `round`+`chamfer` num [`Edge`] tiraria o argumento a mais e partiria a
// forma da família — as outras treze chapas recebem os dois soltos, e uma família com duas
// assinaturas é onde a próxima passa os parâmetros trocados.
#[allow(clippy::too_many_arguments)]
pub fn sd_arrow(
    heads: u32,
    half_length: f64,
    shaft: f64,
    head: f64,
    head_length: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let duplo = heads >= 2;
    let eixo = if duplo { Tree::x().abs() } else { Tree::x() };
    let ponta = cunha(&eixo, &Tree::y(), half_length, head, head_length);
    let frente = frente_da_haste(half_length, head, head_length, shaft);
    // A haste vai da cauda até dentro da ponta; na seta dupla ela é simétrica.
    let (x0, x1) = if duplo {
        (-frente, frente)
    } else {
        (-half_length, frente)
    };
    let (cx, hx) = ((x0 + x1) * 0.5, (x1 - x0) * 0.5);
    // ⭐⭐ **O BICO ARREDONDA**, e a medição é que o mandou.
    //
    // ⛔ A 1.ª redacção fechava a cunha por `max` DURO, citando a lei da [`crate::ops_plates::sd_pie`]
    // (*«o ápice corta a seco»*). ⚠️ **Aquela recusa responde outra pergunta:** ali o ápice estava
    // encaixado dentro de **três** intersecções arredondadas (os dois semiplanos · o disco · a laje)
    // e a inflação compunha-se; aqui a cunha é a peça de fora, e o que se lhe segue é uma união com
    // a haste. A sonda de arestas mediu a diferença — o pior vinco da seta dobrada cai de `86,3°`
    // para dentro da barra — e o censo continua abaixo de `1`. *Uma recusa medida responde UMA
    // pergunta; reconfira-a quando a sua for outra.*
    let bico = crate::ops_joint::intersection_joint(&ponta[0], &ponta[1], e);
    // ⚠️ **As farpas são quinas CÔNCAVAS** — é delas que uma seta vive, e um `min` duro deixá-las-ia
    // vivas. A união entra arredondada, como na cruz.
    if chamfer <= 0.0 {
        // ⭐⭐⭐ **A ESTRUTURA DA CRUZ, e ela é load-bearing** (medida em 2026-09-04): a 1.ª redacção
        // desta função dava a cada peça a **própria laje** (`plate_joint_n`) e unia as duas chapas
        // já fechadas. ⛔ Duas coisas correm mal ao mesmo tempo, e o censo apanhou as duas:
        //
        // 1. **`‖∇f‖ = 1,9076`** (`passo × ‖∇f‖ = 1,35`, acima de `1` ⇒ a marcha atravessa a
        //    superfície) — cada nível de mistura soma um quadrado na lei de Cauchy–Schwarz, que é
        //    exactamente o que a [`crate::ops::union_round_n`] existe para não fazer;
        // 2. **a peça saía da própria caixa em Z** (`0,1088` contra `0,1000`): as duas chapas
        //    partilham as tampas, e a **união arredondada de duas faces coplanares INCHA** — o
        //    filete da farpa empurrava material para fora da laje.
        //
        // ⇒ o perfil compõe-se **em 2D** e a laje entra **uma vez**, no fim.
        let haste2d = rect_round_em(cx, 0.0, hx, shaft, round);
        let cabeca2d = crate::ops_joint::intersection_joint(&bico, &ponta[2], e);
        return crate::ops::slab_and_walls(
            &crate::ops_joint::union_joint(&haste2d, &cabeca2d, e),
            half_height,
            e,
        );
    }
    // ⭐ Com chanfro cada peça é uma chapa de meios-planos, pela razão da cruz: um rectângulo já
    // composto carrega a bissectriz das quinas para dentro do aro.
    let haste_p = paredes(cx, 0.0, hx, shaft);
    let haste = crate::ops::plate_joint_n(&haste_p, &quinas(&haste_p), half_height, e);
    let cabeca = crate::ops::plate_joint_n(
        &[bico.clone(), ponta[2].clone()],
        &[(bico, ponta[2].clone())],
        half_height,
        e,
    );
    crate::ops_joint::union_joint(&haste, &cabeca, e)
}

/// ⭐ **CHEVRON** — a banda em «V», que é a **diferença de duas cunhas paralelas**.
///
/// ⚠️ **A cunha de dentro é a de fora DESLOCADA de `thickness` ao longo da própria normal**, e não
/// uma segunda cunha com outro ápice: somar a espessura ao semiplano já normalizado dá a paralela
/// exacta, e a conta do ápice deslocado (`t/sin θ`) fica por fazer — *uma lei que a normalização já
/// entrega não se escreve à mão*.
pub fn sd_chevron(
    half_length: f64,
    half_span: f64,
    thickness: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    // Os flancos exteriores: de `(half_length, 0)` a `(−half_length, ±half_span)`.
    let fora = cunha(
        &Tree::x(),
        &Tree::y(),
        half_length,
        half_span,
        2.0 * half_length,
    );
    let dentro = [
        fora[0].clone() + Tree::constant(thickness),
        fora[1].clone() + Tree::constant(thickness),
    ];
    // ⭐⭐⭐ **O ENTALHE DE DENTRO TEM DONO** — e sem esta linha ele não tinha nenhum.
    //
    // ⛔ A sonda de arestas mediu **`12,0 %` da superfície sobre um vinco de `83,1°`** com o filete a
    // metade do limite, e o chanfro cortava só `73,3 %` das arestas. ⚠️ É a mesma pedra que a cruz e
    // o coração pagaram na W106: *uma divisão de responsabilidade copiada de outra forma é uma
    // aresta órfã quando o segundo dono não existe*. O `slab_and_walls` arredonda o **aro**
    // (parede↔tampa); o vértice da cunha interior não é aro nenhum.
    //
    // ⭐ A cura é arredondar o **ápice da cunha de dentro**: o buraco encolhe ali, e o entalhe do
    // sólido fica redondo. *Um vinco côncavo de um sólido é uma quina convexa do vazio.*
    let cunha_interior = crate::ops_joint::intersection_joint(&dentro[0], &dentro[1], e);
    let tras = Tree::constant(-half_length) - Tree::x();
    // O corpo é `fora ∩ ¬dentro ∩ atrás`.
    let vazio = -cunha_interior;
    // ⭐⭐ **A BANDA fecha por `max` DURO, e só a parede de trás entra na mistura.**
    //
    // ⛔ A 1.ª redacção entregava as quatro peças a uma mistura n-ária e o censo mediu
    // `passo × ‖∇f‖ = 1,34` — ver a nota da [`crate::ops_plates::sd_rhombus`]. ⚠️ E aqui a n-ária
    // não comprava nada: **as duas faces da banda são PARALELAS** e nunca se encontram, o bico
    // exterior fica vivo pela lei da `sd_pie`, e o entalhe de dentro é o que faz um chevron
    // parecer um chevron. *As únicas quinas que este perfil tem são as da face de trás.*
    let cunha_exterior = crate::ops_joint::intersection_joint(&fora[0], &fora[1], e);
    let banda = cunha_exterior.max(vazio.clone());
    crate::ops::slab_and_walls(
        &crate::ops_joint::intersection_joint(&banda, &tras, e),
        half_height,
        e,
    )
}

/// ⭐ **SETA DOBRADA** — a haste em «L» (de `−X` a `+X`, depois a `+Y`) acabada numa ponta a `+Y`.
///
/// ⚠️ **Os dois braços SOBREPÕEM-SE no cotovelo por construção** (o quadrado de lado `2·shaft` na
/// quina), e é isso que a lei da sobreposição pede — aqui ela sai de graça, ao contrário da junção
/// haste↔ponta.
// ⚠️ **A assinatura espelha os campos da primitiva UM-A-UM, e é isso que a mantém honesta:** o
// despacho em `primitive_tree.rs` desestrutura a variante pelo nome, então um campo novo é **erro de
// compilação** lá. ⛔ Agrupar `round`+`chamfer` num [`Edge`] tiraria o argumento a mais e partiria a
// forma da família — as outras treze chapas recebem os dois soltos, e uma família com duas
// assinaturas é onde a próxima passa os parâmetros trocados.
#[allow(clippy::too_many_arguments)]
pub fn sd_bent_arrow(
    run: f64,
    rise: f64,
    shaft: f64,
    head: f64,
    head_length: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    // O braço de pé fica encostado à direita; o cotovelo é o quadrado em `(run − shaft, −rise +
    // shaft)`.
    let ex = run - shaft;
    let frente = frente_da_haste(rise, head, head_length, shaft);
    // ⭐⭐⭐ **O «L» É UMA SUBTRACÇÃO, e não a união de dois braços** (medido em 2026-09-05).
    //
    // ⛔⛔ A 1.ª redacção unia um rectângulo deitado a um de pé. Os dois **partilham a face de
    // baixo** (`y = −rise`), e a união arredondada de duas peças cuja fronteira COINCIDE ao longo de
    // uma face **INCHA** para fora dela: o censo da caixa por eixo achou peça a `−0,3459` com a
    // meia-extensão em `0,3400` — `0,0059`, que é o `r·(√2 − 1)` da mistura.
    //
    // ⚠️ **E as duas varreduras densas desta linha NÃO o viam**, porque guardavam o `y` MÁXIMO e o
    // excesso estava no `y` mínimo. *Duas réguas que discordam da terceira não a refutam — foi a
    // que mede o módulo que tinha razão.*
    //
    // ⭐ Com a subtracção não há união nenhuma no corpo: o rectângulo grande vai da base até
    // [`frente`] e perde o **bloco de cima à esquerda**. A quina côncava do «L» nasce do canto do
    // bloco removido, e arredonda como qualquer outra.
    //
    // ⚠️ **O bloco removido tem de PASSAR de largo** pelos lados esquerdo e de cima do rectângulo:
    // com a fronteira dele a coincidir com a do grande, a intersecção arredondada **comeria** aquelas
    // duas faces — é o mesmo mecanismo, do outro lado.
    let grande = rect_round_em(
        0.0,
        (frente - rise) * 0.5,
        run,
        (frente + rise) * 0.5,
        round,
    );
    let (xr, yb) = (run - 2.0 * shaft, -rise + 2.0 * shaft);
    let a = Tree::constant(xr) - Tree::x();
    let b = Tree::y() - Tree::constant(yb);
    // ⭐ A quina CÔNCAVA do «L» é o vértice do bloco removido — e é ela que dá o carácter da forma.
    let entalhe = crate::ops_joint::union_joint(&a, &b, e);
    let ponta = cunha(
        &Tree::y(),
        &(Tree::x() - Tree::constant(ex)),
        rise,
        head,
        head_length,
    );
    // ⭐ O bico arredonda — ver a nota da [`sd_arrow`].
    let bico = crate::ops_joint::intersection_joint(&ponta[0], &ponta[1], e);
    let cabeca2d = crate::ops_joint::intersection_joint(&bico, &ponta[2], e);
    if chamfer > 0.0 {
        // ⭐ Com chanfro as peças entram INTEIRAS — um perfil já composto carrega a costura interna
        // dele para dentro do aro. Medido: `16,0°` só com filete contra `60,8°` com chanfro
        // (`3,80×`, barra `2,60×`). Ver [`crate::ops::plate_joint_n`].
        let g = paredes(0.0, (frente - rise) * 0.5, run, (frente + rise) * 0.5);
        let mut arestas = quinas(&g);
        arestas.push((g[2].clone(), entalhe.clone()));
        arestas.push((g[0].clone(), entalhe.clone()));
        let pecas = [
            g[0].clone(),
            g[1].clone(),
            g[2].clone(),
            g[3].clone(),
            entalhe,
        ];
        let corpo = crate::ops::plate_joint_n(&pecas, &arestas, half_height, e);
        let cabeca = crate::ops::plate_joint_n(
            &[bico.clone(), ponta[2].clone()],
            &[(bico, ponta[2].clone())],
            half_height,
            e,
        );
        return crate::ops_joint::union_joint(&corpo, &cabeca, e);
    }
    let corpo = crate::ops_joint::intersection_joint(&grande, &entalhe, e);
    let cabeca = cabeca2d;
    // ⚠️ **A ponta encosta ao corpo em DOIS PONTOS, nunca ao longo de uma face** — em `y = frente` a
    // meia-largura dela vale exactamente `shaft`, por construção da [`frente_da_haste`]. É isso que
    // deixa esta união livre do inchaço que a de baixo tinha.
    crate::ops::slab_and_walls(
        &crate::ops_joint::union_joint(&corpo, &cabeca, e),
        half_height,
        e,
    )
}
