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
//! # A restrição — SENTE pelo gradiente, EMPURRA pela corda
//!
//! ```text
//! C   = s − len                      (violação, em metros; só age com C > 0)
//! g   = (âncora − centro)/len        (o GRADIENTE da folga; |g| = d/len ≥ 1)
//! u   = (âncora − tangência)/len     (a direção da CORDA, unitária)
//! Ċ   = v(âncora)·(−g) + v(eixo)·(+g)
//! k   = gᵀ M⁻¹ u                     (bilinear — `End::k2`)
//! λ   = (Ċ + β·C/dt) / k             , λ ≥ 0
//! impulso = +λ·u na âncora, −λ·u no eixo
//! ```
//!
//! ⚠️ **Os dois vetores são DIFERENTES, e cada um responde a uma pergunta que só
//! ele pode responder.**
//!
//! **Empurrar** tem de ser por `u`: *uma corda puxa ao longo de si mesma*, e a
//! parte de `g` perpendicular a ela é força que nenhum fio transmite. Decompondo,
//! `g = u + (r/len)·n` com `n ⊥ u`. Foi essa parte que o smoke do Enio viu como
//! *"uma força bizarra que empurra o objeto na direção x das polias"* — medido,
//! **1,3964 m** de deriva lateral em 3 s sobre uma carga que devia estar parada, e
//! `atan(r/s)` de desvio: de 9,5° a **76°** na tabela de `measure_stop_sideways`.
//!
//! **Sentir** tem de ser por `g`: a folga muda com o balanço PERPENDICULAR à corda
//! à taxa `r/len`, que numa roda de raio 2,0 com 0,5 m de folga é **4×** o que a
//! corda percebe. Uma trava que só sente pela corda **não vê a carga chegando** —
//! medido, folga mínima **0,0000** naquela roda contra **0,3685**.
//!
//! ⚠️ **E isso é a formulação padrão, não um remendo:** `g·u = 1` **identicamente**
//! (porque `n ⊥ u` e `|u| = 1`), então empurrar pela corda sempre corrige a folga,
//! e `λ = (Ċ + β·C/dt)/(gᵀM⁻¹u)` zera `Ċ` exatamente. Jacobiano `g`, direção de
//! impulso `u` — o impulso sequencial não-simétrico.
//!
//! ⚠️ **A CORDA nunca muda de direção.** A v1 desta wave fazia o oposto — mandava a
//! CORDA falar `g` quando a ponta travava —, e era isso que punha a força fora do
//! eixo dela. Hoje quem se adapta é a trava, e só na metade em que ela pode.
//!
//! ⚠️ **`λ ≥ 0`: um limitador só AFASTA.** Ele não puxa a carga de volta para a
//! roldana quando ela está longe — é a mesma desigualdade da corda, pelo lado
//! oposto, e é o que o deixa conviver com ela em vez de brigar.
//!
//! ⚠️ **O eixo MONTADO entra como a outra ponta da mesma restrição**, exatamente
//! como no passe da corda: sem isso um limitador numa cadernal móvel empurraria a
//! carga contra um eixo de massa infinita que ele não tem.

use crate::rmath::Vector;
use rapier2d::dynamics::RigidBodySet;

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
/// # O nó travado — a órbita, e por que ela NÃO pede uma lei própria
///
/// A primeira versão empurrava pelo **gradiente radial**, e a sonda mostrou a
/// carga **orbitando a roldana**: a trava empurrava por `g`, a corda puxava por
/// `u`, e o resíduo entre duas direções que discordam é `λ·(r/len)·n` — lateral. O
/// guincho arrastava a carga por cima do eixo (`y` final **9,545 m** com a roldana
/// a 8,0 — medido).
///
/// A cura da época foi obrigar a **CORDA** a falar `g` quando a ponta travava. Ela
/// matava a órbita — e comprava, pelo mesmo preço, o defeito que o smoke reportou
/// em seguida: passava a ser a corda que puxava **23,76° fora de si mesma**.
///
/// **A cura que fica é a inversa:** quem cede é a TRAVA, e só na metade em que ela
/// pode — ela **empurra** por `u`, a mesma direção da corda, e segue **sentindo**
/// por `g`. Os dois impulsos da ponta ficam colineares, o resíduo perde o eixo
/// lateral para onde apontar, e a órbita morre **sem** ninguém mexer no Jacobiano
/// da corda.
///
/// ⚠️ **Ceder a metade errada foi MEDIDO e reprovado:** fazer a trava sentir por
/// `u` também deixa a folga mínima da roda grande em **0,0000** — ela para de ver
/// o balanço, que é 4× mais rápido que a corda ali.
///
/// ⚠️ **Isto NÃO é apagar a lei do nó travado — é descobrir que ela era um
/// SINTOMA.** Apagar só a substituição, mantendo o `g` da trava, **reproduz a
/// órbita** (controle medido, mesma sonda: `x` final **1,686 m ao LADO** da
/// roldana, `y` **7,73 m** com o eixo a 6,0). O que importava nunca foi *quem*
/// fala `g` — era *que os dois falem a mesma coisa*.
///
/// ⚠️ **E o outro lado não paralisa:** com A travado, um contrapeso em B que desça
/// ALONGA a rota, `C > 0`, e a corda o segura — que é o que uma corda com um nó
/// preso no bloco faz.
///
/// No-op para toda corda cujos dois números são zero — o estado de tudo o que já
/// existia —, e o `continue` sai antes de qualquer leitura de corpo, então aquelas
/// cenas ficam **byte-idênticas**.
pub(super) fn apply_stops(
    bodies: &mut RigidBodySet,
    p: &PulleyDesc,
    live: &[RopeWheel],
    legs: &[Tangent],
    dt: f32,
    bias: f32,
) {
    for (side, &stop) in p.stops.iter().enumerate() {
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
        // Um trecho degenerado daria `NaN` — e um `NaN` chega ao `physics_ecs_c9`.
        if leg.len <= f32::EPSILON {
            continue;
        }
        // **A direção da CORDA** — da tangência para a amarração, unitária, e a
        // MESMA que a rota entrega ao passe da corda (`RopeRoute::dir_a`/`dir_b`).
        // O gradiente exato da folga é o radial, e é exatamente por ele que uma
        // corda não consegue puxar: ver o cabeçalho.
        // **O GRADIENTE** da folga — radial, do centro da roda para a amarração,
        // com magnitude `d/len ≥ 1`. É por ele que a trava SENTE.
        let g = Vector::new(
            (leg.anchor[0] - leg.centre[0]) / leg.len,
            (leg.anchor[1] - leg.centre[1]) / leg.len,
        );
        // **A DIREÇÃO DA CORDA** — da tangência para a amarração, unitária, e a
        // MESMA que a rota entrega ao passe da corda (`RopeRoute::dir_a`/`dir_b`).
        // É por ela que a trava EMPURRA.
        let u = Vector::new(
            (leg.anchor[0] - leg.touch[0]) / leg.len,
            (leg.anchor[1] - leg.touch[1]) / leg.len,
        );
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
        let mut k = e.k2(g, u);
        let mut c_dot = e.rate(-g);
        // A outra ponta da MESMA restrição: o eixo, quando ele é de um corpo.
        let axle = live
            .get(leg.wheel)
            .and_then(|w| Some((w.body?, w.local)))
            .and_then(|(h, l)| Some((h, end(bodies, h, l)?)));
        if let Some((_, ref ea)) = axle {
            k += ea.k2(g, u);
            c_dot += ea.rate(g);
        }
        if k <= f32::EPSILON {
            continue;
        }
        let lambda = (c_dot + bias * c / dt) / k;
        if lambda <= 0.0 {
            continue;
        }
        push(bodies, handle, e.point, lambda, u);
        if let Some((h, ea)) = axle {
            push(bodies, h, ea.point, -lambda, u);
        }
    }
}
