//! **Auto-flip** — B foi desenhado no sentido CONTRÁRIO de A?
//!
//! Módulo irmão do [`crate::tween`]: é uma responsabilidade à parte, e erra de outro
//! jeito. O tween erra em *interpolação* (o inbetween sai torto); isto erra em
//! *geometria de decisão* (o traço dá um NÓ no meio do caminho, porque o ponto 0 de A
//! foi interpolado contra o ponto final de B).
//!
//! O teste é puramente geométrico e transcendental-free (HR-5): **qual dos dois jeitos de
//! parear as pontas percorre menos caminho?**
//!
//! ⚠️ **Traço FECHADO responde por outra pergunta** ([`opposite_winding`]) — ver lá.

use crate::stroke::FlipStroke;
use ph2d_core::Vec2;

pub(crate) fn should_flip(a: &FlipStroke, b: &FlipStroke) -> bool {
    // **Num traço fechado as "pontas" são VIZINHAS** (a costura), então a corda entre elas
    // não descreve o percurso — o teste abaixo vira ruído e inverte anéis à toa.
    //
    // ⚠️ Era um bug REAL, e a espiral o expôs: num "O" que gira 90° o flip espúrio
    // transformava a rotação limpa numa reflexão, o ajuste via `σ ≈ 0` e caía na
    // translação — o anel cortava caminho pela CORDA em vez de percorrer o arco.
    //
    // Para um anel a pergunta certa é o SENTIDO do percurso: dois windings opostos
    // percorrem o contorno em direções contrárias, e interpolar entre eles vira a forma do
    // avesso (o `ph2d-vec-blend` mediu isso — a forma COLAPSA no meio).
    if a.closed || b.closed {
        return opposite_winding(a, b);
    }
    let (Some(a0), Some(a1)) = (a.point(0), a.point(a.len().saturating_sub(1))) else {
        return false;
    };
    let (Some(b0), Some(b1)) = (b.point(0), b.point(b.len().saturating_sub(1))) else {
        return false;
    };
    let (a0, a1, b0, b1) = (a0.pos, a1.pos, b0.pos, b1.pos);

    // **A pergunta, inteira: cruzado é mais curto que direto?**
    //
    // ⚠️ Isto SUBSTITUI o par de heurísticas portadas do GP (cordas que se cruzam ·
    // direções opostas, com a distância só como desempate de cordas quase paralelas). As
    // duas eram PROXIES desta pergunta, e a de direção **quebra numa rotação grande**:
    // `da·db < 0` vale para qualquer giro além de 90°, então um braço que gira 120° era
    // lido como *"desenhado ao contrário"* — o flip colava o OMBRO de A na PONTA de B, e o
    // inbetween girava em torno do lugar errado. Medido no fixture do braço:
    // `dot = −1,62` (o GP diz inverter) enquanto **direto 3,118 < cruzado 3,600** (a
    // distância diz que não). Só a espiral tornou isso visível; com o lerp o resultado já
    // era torto de qualquer jeito.
    //
    // A distância decide os dois casos que importam, e é UMA regra em vez de três:
    // traço redesenhado ao contrário ⇒ cruzado é bem mais curto ⇒ inverte; traço que
    // girou ⇒ as pontas seguem cada uma a sua ⇒ não inverte.
    //
    // ⚠️ **Distâncias REAIS, não ao quadrado** — e a diferença decide. Somar quadrados
    // pergunta outra coisa (ela penaliza *uma* viagem longa mais que *duas* médias), e no
    // braço que gira 120° as duas respostas são OPOSTAS: por distância, direto `3,118` <
    // cruzado `3,600` (não inverte, certo); por quadrado, cruzado `6,48` < direto `9,72`
    // (inverte, errado). O `dist2` estava aqui porque a versão antiga só o usava como
    // desempate de cordas quase paralelas, onde os dois critérios concordam.
    dist(a0, b1) + dist(a1, b0) < dist(a0, b0) + dist(a1, b1)
}

/// **Os dois anéis são percorridos em sentidos contrários?** (área com sinal, shoelace).
///
/// É o teste que substitui as cordas no caso fechado. Área ZERO (um anel degenerado, ou
/// um traço marcado `closed` com dois pontos) não decide nada — e aí não inverter é a
/// resposta segura: uma inversão espúria é o defeito que este módulo existe para evitar.
///
/// ⚠️ **O que isto NÃO resolve, de propósito:** dois anéis de mesmo sentido cujo ponto 0
/// está em lugares diferentes do contorno (a FASE da costura). O tween os interpola com a
/// costura desalinhada, e a forma do meio fica torcida. A resposta é o alinhamento de fase
/// por correlação circular que o `ph2d-vec-blend` já construiu para o Blend do vetor —
/// wave própria, nomeada no handoff, e não um `if` a mais aqui.
fn opposite_winding(a: &FlipStroke, b: &FlipStroke) -> bool {
    let area = |s: &FlipStroke| -> f32 {
        let p = s.positions();
        let n = p.len();
        if n < 3 {
            return 0.0;
        }
        // Shoelace: o dobro da área com sinal (o fator 2 não muda o SINAL, que é o que
        // importa aqui, então não se paga por ele).
        (0..n)
            .map(|i| {
                let (u, v) = (p[i], p[(i + 1) % n]);
                u.x * v.y - v.x * u.y
            })
            .sum()
    };
    let (wa, wb) = (area(a), area(b));
    wa * wb < 0.0
}

/// A distância entre dois pontos (o `sqrt` é exato em IEEE — não é transcendental, e
/// nada aqui depende de aproximação de biblioteca).
fn dist(p: Vec2, q: Vec2) -> f32 {
    let d = q - p;
    (d.x * d.x + d.y * d.y).sqrt()
}
