//! **O LIMITADOR** — a trava que impede uma ponta da corda de entrar na roldana
//! (W-RopeStop).
//!
//! Enio: *"vamos criar limitadores de modo que tem a possibilidade dos objetos
//! nunca colidirem com as polias. Os limitadores são dois por corda e são
//! desenhados em cima da corda"*.
//!
//! Módulo FILHO de [`super`] pelos mesmos dois motivos dos irmãos
//! [`super::mount`] e [`super::winch`]: ele precisa do [`End`] e do [`end`], que
//! são privados de lá — um limitador é **mais uma restrição sobre a mesma
//! corda**, e dar-lhe uma resposta própria para *"qual é a massa efetiva neste
//! ponto?"* seria a segunda porta que esta família passou waves fechando —, e o
//! `pulley.rs` está no teto de LOC.
//!
//! # O que a medição disse ANTES de uma linha ser escrita
//!
//! `tests/measure_rope_stop.rs`, um guincho recolhendo a 0,5 m/s sobre uma
//! roldana de raio 0,5:
//!
//! | t (s) | folga de tangente (m) | y da carga |
//! |---|---|---|
//! | 0,00 | 6,0092 | 2,000 |
//! | 6,00 | 2,9968 | 4,991 |
//! | 10,50 | 0,7823 | 7,241 |
//! | **12,00** | **0,0000** | 7,905 |
//! | 13,50 | 1,1998 | 7,688 |
//! | 15,00 | 1,4607 | 7,095 |
//!
//! ⚠️ **E o defeito é pior que *"colide"*:** passada a folga zero a rota fica
//! DEGENERADA (o corpo está em cima da roldana), o passe recusa a corda inteira —
//! *"meia rota é pior que nenhuma"* — e a carga **deixa de ser segurada**: as duas
//! últimas linhas são ela caindo de volta, sem erro e sem aviso.
//!
//! # A grandeza é a FOLGA DE TANGENTE, e ela não foi escolhida
//!
//! `len = √(d² − r²)` é o comprimento do trecho de corda que sobra entre a
//! amarração e o ponto em que ela encosta na roda — e ele chega a **zero
//! exatamente quando `d = r`**, isto é, quando a amarração toca o ARO. Então
//! `len ≥ s` é literalmente *"não encoste na roldana, e sobre `s` de corda"*.
//!
//! Uma distância ao CENTRO diria que ainda há meio metro de folga quando a carga
//! já está encostada numa roldana de meio metro de raio; e uma distância medida
//! **da amarração** poria a marca num lugar e a carga noutro. Medida do ponto de
//! TANGÊNCIA, a marca desenhada **é** o lugar onde a amarração vai parar — o que
//! um limitador desenhado em cima da corda tem de significar.
//!
//! # A restrição
//!
//! ```text
//! C   = s − len                      (violação, em metros; só age com C > 0)
//! ∂C/∂âncora = −g ,  ∂C/∂centro = +g ,  g = (âncora − centro)/len
//! Ċ   = v(âncora)·(−g) + v(centro)·(+g)
//! λ   = (Ċ + β·C/dt) / k             , λ ≥ 0
//! ```
//!
//! ⚠️ **`g` NÃO é o versor do trecho** — ele aponta do centro da roda para a
//! amarração, com magnitude `d/len ≥ 1`. Com raio zero os dois coincidem (`d =
//! len`), e é por isso que a forma errada passaria despercebida em toda cena de
//! roldana-ponto e mentiria por `d/len` justamente na roda grande, que é onde o
//! artista pede o limitador.
//!
//! ⚠️ **`λ ≥ 0`: um limitador só AFASTA.** Ele não puxa a carga de volta para a
//! roldana quando ela está longe — é a mesma desigualdade da corda, pelo lado
//! oposto, e é o que o deixa conviver com ela em vez de brigar.
//!
//! ⚠️ **O eixo MONTADO entra como a outra ponta da mesma restrição**, exatamente
//! como no passe da corda: sem isso um limitador numa cadernal móvel empurraria a
//! carga contra um eixo de massa infinita que ele não tem.

use rapier2d::dynamics::RigidBodySet;
use rapier2d::na::Vector2;

use super::rope_route::{RopeWheel, Tangent};
use super::{PulleyDesc, end, push};

/// **Uma ponta da rota, do ponto de vista do limitador:** onde a corda se amarra,
/// onde ela ENCOSTA na roldana, e quanta corda há entre os dois.
///
/// É a estrutura que os TRÊS consumidores leem — o passe de impulso (para saber
/// se a trava agiu), o desenho (para pôr a marca) e o arrasto (para converter um
/// cursor em número). Uma segunda derivação em qualquer um deles poria a marca
/// num lugar e a trava noutro.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StopLeg {
    /// Onde a corda se amarra no corpo, em mundo.
    pub anchor: [f32; 2],
    /// Onde ela encosta na roldana desta ponta, em mundo.
    pub touch: [f32; 2],
    /// O centro da roldana desta ponta, em mundo.
    pub centre: [f32; 2],
    /// A folga de tangente que existe AGORA, em metros — `|touch − anchor|`.
    pub len: f32,
    /// O índice, na lista VIVA de roldanas, da roda desta ponta.
    pub wheel: usize,
}

/// **A ponta `side` da rota** (`0` = A, `1` = B), ou `None` quando ela não tem
/// roldana contra a qual travar.
///
/// ⚠️ **`None` sem roldana nenhuma, e isso é a lei e não um guarda:** um limitador
/// é uma trava CONTRA uma roldana. Numa corda reta entre dois corpos há **um**
/// trecho, então as duas pontas pediriam a mesma coisa duas vezes — um controle e
/// a redundância dele ao lado. `legs.len() == wheels.len() + 1`, então `>= 2` é
/// exatamente *"existe ao menos uma roldana"*.
#[must_use]
pub fn stop_leg(legs: &[Tangent], wheels: &[RopeWheel], side: usize) -> Option<StopLeg> {
    if legs.len() < 2 {
        return None;
    }
    let (leg, wheel) = if side == 0 {
        (legs.first()?, 0)
    } else {
        (legs.last()?, wheels.len().checked_sub(1)?)
    };
    let w = wheels.get(wheel)?;
    // ⚠️ A ponta A percorre `âncora → tangência`; a ponta B, `tangência → âncora`.
    // Trocar os dois é o erro que põe a marca no ponto de tangência da roda errada
    // e o desenho a mostraria colada na roldana em vez de descer pela corda.
    let (anchor, touch) = if side == 0 {
        (leg.from, leg.to)
    } else {
        (leg.to, leg.from)
    };
    Some(StopLeg {
        anchor,
        touch,
        centre: w.centre,
        len: leg.len,
        wheel,
    })
}

/// **Onde a marca é desenhada:** `stop` metros de corda a partir do ponto de
/// tangência, andando na direção da amarração.
///
/// Porta ÚNICA com [`stop_at_point`], que é a inversa dela — a lei do seed==sample
/// que esta linha já pagou duas vezes: se o desenho e o arrasto derivarem a
/// posição por caminhos diferentes, a marca salta debaixo do dedo no instante do
/// clique.
///
/// O passo é clampado ao trecho: uma marca além da amarração ficaria atrás do
/// corpo, e uma além da tangência estaria do outro lado da roldana.
#[must_use]
pub fn stop_mark(leg: &StopLeg, stop: f32) -> [f32; 2] {
    if leg.len <= 0.0 {
        return leg.touch;
    }
    let s = stop.clamp(0.0, leg.len) / leg.len;
    [
        leg.touch[0] + (leg.anchor[0] - leg.touch[0]) * s,
        leg.touch[1] + (leg.anchor[1] - leg.touch[1]) * s,
    ]
}

/// **A inversa:** que número um ponto do mundo autora, projetado no trecho.
///
/// O cursor quase nunca está EM cima da corda, então ele é projetado nela — é o
/// que faz o arrasto seguir a corda em vez de exigir mira. Fora do trecho, clampa
/// nas pontas.
#[must_use]
pub fn stop_at_point(leg: &StopLeg, p: [f32; 2]) -> f32 {
    let d = [leg.anchor[0] - leg.touch[0], leg.anchor[1] - leg.touch[1]];
    let dd = d[0] * d[0] + d[1] * d[1];
    if dd <= 0.0 {
        return 0.0;
    }
    let t = ((p[0] - leg.touch[0]) * d[0] + (p[1] - leg.touch[1]) * d[1]) / dd;
    (t.clamp(0.0, 1.0) * leg.len).clamp(0.0, leg.len)
}

/// **Impor os dois limitadores de uma corda**, uma vez por sub-passo — e dizer
/// quais pontas ficaram **TRAVADAS**.
///
/// ⚠️ **Roda ANTES do early-out de corda frouxa**, e não é detalhe: `C ≤ 0` faz o
/// passe da corda sair sem tocar em ninguém, e uma trava tem de segurar mesmo com
/// a corda bamba — é outra coisa que ela impede (um corpo empurrado contra a
/// roldana por um contato, por uma zona de força, pela mão).
///
/// # Por que ele DEVOLVE alguma coisa — a metade que a medição exigiu
///
/// A primeira versão só empurrava, e a sonda mostrou a carga **orbitando a
/// roldana**: com a folga radial presa em `s`, o orçamento `L ≤ L0` ainda
/// encurtava pelo **ARCO**, então o guincho arrastava a carga por cima do eixo
/// (`y` final **9,545 m** com a roldana a 8,0 — medido).
///
/// A física que faltava é o que um nó travado É: ele impede a corda de **CORRER**
/// pelo bloco. O arco fica do outro lado do nó, então encurtá-lo exigiria
/// exatamente a corrida que o nó proíbe.
///
/// A expressão disso no kernel é uma linha, e ela é **derivada, não escolhida**:
/// com a corda impedida de correr, o arco fica CONSTANTE, logo `L = len + const`
/// e o Jacobiano daquela ponta deixa de ser o versor do trecho e passa a ser
/// `∂len/∂âncora`, que é o **`g` radial** — o mesmo vetor que a trava usa.
///
/// Em uma frase: *com o nó travado a corda só consegue puxar a ponta CONTRA o
/// bloco; puxá-la de lado exigiria a corda correr, que é o que o nó proíbe.*
///
/// ⚠️ **A primeira versão tirava a ponta da restrição INTEIRA, e isso foi
/// REPROVADO pelo gate da roda grande:** sem o termo da corda a ponta perde o
/// SUPORTE, cai em queda livre, destrava ao se afastar, é apanhada de volta com
/// um tranco — e o ciclo INJETA energia (medido: a carga atirada a `x = 3,976 m`
/// numa roda de raio 2,0). Substituir o Jacobiano em vez de o apagar mantém a
/// corda segurando o peso e tira **só** a componente tangencial, que é
/// exatamente a que arrastaria a carga em volta do eixo.
///
/// ⚠️ **E isso NÃO paralisa o outro lado**, que é o teste de que a lei está certa:
/// com A travado, um contrapeso em B que desça ALONGA a rota, `C > 0`, e a corda o
/// segura — que é o que uma corda com um nó preso no bloco faz.
///
/// No-op para toda corda cujos dois números são zero — o estado de tudo o que já
/// existia —, e o `continue` sai antes de qualquer leitura de corpo, então aquelas
/// cenas ficam **byte-idênticas** e a devolução é `[false, false]`.
pub(super) fn apply_stops(
    bodies: &mut RigidBodySet,
    p: &PulleyDesc,
    live: &[RopeWheel],
    legs: &[Tangent],
    dt: f32,
    bias: f32,
) -> [Option<Vector2<f32>>; 2] {
    let mut jammed = [None, None];
    for (side, (&stop, jam)) in p.stops.iter().zip(jammed.iter_mut()).enumerate() {
        // ⚠️ **Não-finito é DESLIGADO, e não `<= 0.0`:** um `NaN` compara falso
        // com tudo, então a forma curta o deixaria atravessar até virar uma pose
        // `NaN` e daí o hash do `physics_ecs_c9`. Um `∞` seria uma trava mais
        // longa que qualquer corda — travada para sempre, em silêncio.
        if !stop.is_finite() || stop <= 0.0 {
            continue;
        }
        let Some(leg) = stop_leg(legs, live, side) else {
            continue;
        };
        // ⚠️ **A trava é declarada pela GEOMETRIA, não pelo impulso.** Ela vale
        // mesmo quando o `λ` sai zero (a ponta já está se afastando sozinha):
        // *o nó está encostado no bloco* é um fato sobre onde as coisas estão, e
        // é ele que a corda tem de honrar.
        // O gradiente da folga em relação à amarração. Um trecho degenerado daria
        // `NaN` — e um `NaN` chega ao `physics_ecs_c9`.
        if leg.len <= f32::EPSILON {
            continue;
        }
        let g = Vector2::new(
            (leg.anchor[0] - leg.centre[0]) / leg.len,
            (leg.anchor[1] - leg.centre[1]) / leg.len,
        );
        if leg.len <= stop {
            *jam = Some(g);
        }
        let c = stop - leg.len;
        if c <= 0.0 {
            continue;
        }
        let (handle, local) = if side == 0 {
            (p.body_a, p.local_a)
        } else {
            (p.body_b, p.local_b)
        };
        let Some(e) = end(bodies, handle, local) else {
            continue;
        };
        let mut k = e.k(g);
        let mut c_dot = e.rate(-g);
        // A outra ponta da MESMA restrição: o eixo, quando ele é de um corpo.
        let axle = live
            .get(leg.wheel)
            .and_then(|w| Some((w.body?, w.local)))
            .and_then(|(h, l)| Some((h, end(bodies, h, l)?)));
        if let Some((_, ref ea)) = axle {
            k += ea.k(g);
            c_dot += ea.rate(g);
        }
        if k <= f32::EPSILON {
            continue;
        }
        let lambda = (c_dot + bias * c / dt) / k;
        if lambda <= 0.0 {
            continue;
        }
        push(bodies, handle, e.point, lambda, g);
        if let Some((h, ea)) = axle {
            push(bodies, h, ea.point, -lambda, g);
        }
    }
    jammed
}
