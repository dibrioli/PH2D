//! ⭐ **O gizmo 3D** — mover, rodar e escalar.
//!
//! Enio, 2026-08-19: *"Não há gzimo 3d para mover os objetos. Precisamos de uma como o do blender."*
//!
//! # O que este arquivo é, e o que ele NÃO é
//!
//! É **lei pura**: projeção, apontar e arrastar, sem `App`, sem ponteiro e sem GPU. Tudo o que aqui
//! entra sai de dois números — a âncora no mundo e a câmera — e por isso todo gesto é gateável sem
//! abrir janela nenhuma. A pintura e a ligação ao ponteiro são os arquivos irmãos
//! ([`crate::field3d_gizmo_paint`], [`crate::field3d_input`]).
//!
//! ⚠️ **Mora no SHELL**, e é a mesma razão do [ADR-0150] que já manda na navegação: uma janela 3D
//! não pode obrigar a mexer no `Tool=12`, que está **congelado**.
//!
//! # ⭐ A projeção é a MESMA do traçador
//!
//! [`ph2d_field_render::Screen`] e [`ph2d_field_render::Orbit::project`] são a conta que a marcha de
//! raios usa para construir os raios. Uma segunda cópia dela aqui divergiria meio pixel, e o sintoma
//! seria uma alça que **agarra ao lado da superfície que ela diz mover** — o tipo de defeito que
//! ninguém chama de bug de projeção. O gate `a_point_projects_where_the_march_actually_hits_it`
//! prende as duas metades.
//!
//! # Os eixos são os do MUNDO
//!
//! Como o default do Blender ("Global"). O nó pode estar rodado — os cilindros da cena 1 estão — e
//! nesse caso mover *ao longo do próprio eixo dele* é uma segunda orientação, que o Blender expõe
//! num seletor. Ela é item ABERTO, e não uma omissão: escolher a orientação é decisão de produto, e
//! entregar só a local seria escolher por quem não pediu.
//!
//! # ⛔ Por que o ESCALAR tem UMA alça, e não três
//!
//! [ADR-0161 §6] mediu e decidiu: a escala de um nó é **uniforme**, porque escala não-uniforme
//! **destrói a propriedade de distância** (‖∇f‖ = 1) de que tudo neste módulo depende — sem ela o
//! raio deixa de ser o raio e a marcha atravessa a superfície.
//!
//! Então três caixas por eixo, como as do Blender, seriam três controles a **prometer o que o
//! modelo não entrega**: arrastar a de X escalaria os três, e o artista concluiria que o app tem um
//! bug. A alça de escala é **uma**, é um punho de tamanho (não um eixo), e por isso não leva cor de
//! eixo nenhuma.
//!
//! [ADR-0150]: ../../../docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md
//! [ADR-0161 §6]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_field::xform::{cross, dot};
use ph2d_field_render::{Orbit, Screen};

/// **O comprimento do braço, EM PIXELS** — o gizmo tem tamanho de tela constante, como o do Blender.
///
/// ⚠️ Constante na tela e não no mundo, de propósito: um gizmo de tamanho de mundo fixo fica maior
/// do que a janela ao aproximar e some ao afastar, e é a mesma peça que se está a manipular nos dois
/// casos. O comprimento em mundo sai daqui dividido por [`Screen::px_per_world`].
pub(crate) const ARM_PX: f32 = 90.0;

/// A folga no centro. Nada é desenhado nem apontável dentro dela — é ela que separa as três setas
/// umas das outras e do disco de vista.
pub(crate) const INNER_PX: f32 = 15.0;

/// O raio de agarre: a que distância do traço um clique ainda é daquela alça.
pub(crate) const GRAB_PX: f32 = 9.0;

/// Comprimento e meia-largura da ponta da seta.
pub(crate) const HEAD_PX: f32 = 17.0;
pub(crate) const HEAD_HALF_W_PX: f32 = 5.5;

/// Espessura do traço da haste (e das argolas).
pub(crate) const SHAFT_HALF_W_PX: f32 = 1.3;

/// Onde fica o quadrado de plano, em fração do braço, e o lado dele.
pub(crate) const PLANE_AT: f32 = 0.38;
pub(crate) const PLANE_SIDE: f32 = 0.22;

/// ⚠️ **O comprimento projetado abaixo do qual uma seta deixa de ser uma alça** — e o número é
/// **derivado**, não escolhido.
///
/// Uma seta apontada para o observador projeta-se curta. A partir de certo ponto a região que a
/// agarra deixa de ser distinguível do centro: a haste começa em [`INNER_PX`] e o agarre tem
/// [`GRAB_PX`] de raio dos dois lados, então uma haste mais curta do que `INNER_PX + 2·GRAB_PX`
/// **não tem um único pixel que seja só dela**. Aí ela não é um controle — é uma lotaria entre três.
///
/// Escondê-la é o que o Blender faz, e o efeito colateral é bom: com a seta escondida sobra o
/// quadrado de plano perpendicular a ela, que é exatamente o gesto que aquele enquadramento pede.
pub(crate) const MIN_ARM_PX: f32 = INNER_PX + 2.0 * GRAB_PX;

/// Em quantos pedaços uma argola é amostrada. Ela é um **círculo do mundo**, e o que se pinta e se
/// aponta é a projeção dele — uma elipse, que só uma poligonal aproxima.
pub(crate) const RING_SEGMENTS: usize = 48;

/// ⚠️ **O quanto uma argola tem de estar virada para o observador** — também **derivado**.
///
/// Vista de perfil, uma argola projeta-se numa reta: o eixo menor da elipse mede
/// `ARM_PX · |cos θ|`, com θ o ângulo entre o eixo dela e a direção da vista. Abaixo de
/// [`GRAB_PX`] ela deixa de ser uma argola apontável e passa a ser um traço — e, pior, o arrasto
/// degenera junto (o plano de rotação fica de perfil e o raio do cursor não o encontra).
///
/// A saída existe e é a [`Handle::ViewRing`]: a argola do plano da tela nunca fica de perfil
/// consigo mesma.
pub(crate) const RING_MIN_DOT: f32 = GRAB_PX / ARM_PX;

/// ⚠️ **O piso que decide o que está «atrás», e ele nomeia o recurso: a precisão da representação.**
///
/// A argola de VISTA fica, por construção, **exatamente** no plano da câmera: a profundidade de todo
/// ponto dela é zero. Em `f32` esse zero sai como ±10⁻⁷ aleatório, e um teste `>= 0` transformaria a
/// argola numa fieira de pedaços soltos — medido (o gate `the_front_half_of_a_ring_is_one_unbroken_run`
/// apanhou-a a sair com **3 pontos** de 48).
///
/// 10⁻⁵ está duas ordens acima do ruído e cinco abaixo de qualquer fronteira real de meia-argola: o
/// pior que ele faz é deixar passar um segmento a mais na borda, que ninguém vê.
///
/// (É o irmão do `PRECISION_FLOOR` do traçador, e pelo mesmo motivo.)
const RING_FRONT_EPS: f32 = 1.0e-5;

/// O raio da argola de vista, em frações do braço. Ela fica **por fora** das três, como a branca do
/// Blender — é a de fora que se agarra sem pensar.
pub(crate) const VIEW_RING_R: f32 = 1.18;

/// Meia-aresta do punho de tamanho.
pub(crate) const GRIP_HALF_PX: f32 = 6.5;

/// ⚠️ **A direção do punho de tamanho é de TELA, e ela é cosmética.**
///
/// Ele não é um eixo — é um punho, como o canto de uma janela —, e a lei do arrasto depende só do
/// **raio** ao centro, nunca desta direção. Pô-lo em cima e à direita é a convenção de todo punho de
/// redimensionar; movê-lo para outro canto não mudaria uma linha da conta.
///
/// (`y` cresce para BAIXO em pixels, daí o sinal.)
pub(crate) const GRIP_DIR: [f32; 2] = [
    std::f32::consts::FRAC_1_SQRT_2,
    -std::f32::consts::FRAC_1_SQRT_2,
];

/// **O que o gizmo faz agora.** Os três verbos, num seletor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Mode {
    #[default]
    Move,
    Rotate,
    Scale,
}

impl Mode {
    /// A ordem em que eles aparecem no painel. ⚠️ **É a fonte da contagem** — quem acrescentar um
    /// verbo mexe aqui e o painel segue sozinho.
    pub(crate) const ALL: [Mode; 3] = [Mode::Move, Mode::Rotate, Mode::Scale];

    /// ⚠️ Uma **chave** de i18n, nunca um rótulo pronto (HR-15).
    pub(crate) fn key(self) -> &'static str {
        match self {
            Mode::Move => "panel.model3d.mode.move",
            Mode::Rotate => "panel.model3d.mode.rotate",
            // ⚠️ O rótulo desta diz **uniforme**, porque é o que o modelo entrega (ver o doc do
            // módulo). Um rótulo que promete mais do que o modelo dá é como se aprende que o app
            // tem um bug que ele não tem.
            Mode::Scale => "panel.model3d.mode.scale",
        }
    }
}

/// A alça agarrada. `usize` é o índice do eixo: 0 = X, 1 = Y, 2 = Z.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Handle {
    /// Mover ao longo de um eixo.
    Axis(usize),
    /// Mover no plano **perpendicular** a este eixo (o quadrado XY é `Plane(2)`).
    Plane(usize),
    /// Mover no plano da tela.
    View,
    /// Rodar em torno de um eixo do mundo.
    Ring(usize),
    /// Rodar em torno da direção da **vista** — a argola que nunca fica de perfil.
    ViewRing,
    /// Escalar **uniformemente**. Uma só, e o doc do módulo diz porquê.
    Grip,
}

/// **Onde o gizmo está e para onde ele aponta**, no mundo. Publicado pela ponte com a cena, que é
/// quem tem o mundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Anchor {
    /// A entidade que ele move — a identidade viaja com a âncora, senão o arrasto teria de a
    /// procurar outra vez e podia achar outra.
    pub(crate) entity: u64,
    pub(crate) origin: [f32; 3],
    /// ⭐ **Os três eixos, JÁ NO MUNDO** — a orientação escolhida ([`Frame`]).
    ///
    /// ⚠️ Eles viajam prontos de propósito: assim a lei do gizmo deixa de saber que existe uma
    /// escolha de orientação, e quem a faz é a ponte, que é quem tem a pose do nó. Sem isto, cada
    /// função daqui teria de perguntar «global ou local?» — o mesmo `if` repetido em cinco sítios,
    /// que é como um deles fica para trás.
    pub(crate) axes: [[f32; 3]; 3],
}

impl Anchor {
    /// Uma âncora nos eixos do mundo — o que a maioria dos gates quer.
    #[cfg(test)]
    pub(crate) fn global(entity: u64, origin: [f32; 3]) -> Self {
        Self {
            entity,
            origin,
            axes: WORLD_AXES,
        }
    }
}

/// **Em que referencial os eixos do gizmo apontam.**
///
/// ⚠️ A distinção não é cosmética: num nó rodado, `Global` move ao longo dos eixos da cena e `Local`
/// ao longo dos do próprio objeto — e os dois são o gesto certo, em momentos diferentes. O Blender
/// expõe exatamente esta escolha num seletor, e entregar só uma seria escolher por quem não pediu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Frame {
    #[default]
    Global,
    Local,
}

impl Frame {
    pub(crate) const ALL: [Frame; 2] = [Frame::Global, Frame::Local];

    /// ⚠️ Uma **chave** de i18n, nunca um rótulo pronto (HR-15).
    pub(crate) fn key(self) -> &'static str {
        match self {
            Frame::Global => "panel.model3d.frame.global",
            Frame::Local => "panel.model3d.frame.local",
        }
    }

    /// Os três eixos deste referencial, no mundo, dada a rotação do nó.
    pub(crate) fn axes(self, rotation: [f32; 4]) -> [[f32; 3]; 3] {
        match self {
            Frame::Global => WORLD_AXES,
            Frame::Local => WORLD_AXES.map(|a| ph2d_field::xform::quat_rotate(rotation, a)),
        }
    }
}

/// A forma de uma alça já projetada — em **pixels**, pronta a pintar e a apontar.
#[derive(Clone, Debug)]
pub(crate) enum Shape {
    /// Haste (do centro para fora) + ponta.
    Arrow { from: [f32; 2], to: [f32; 2] },
    /// Quadrilátero: os quatro cantos, já projetados.
    Quad([[f32; 2]; 4]),
    /// Disco no centro.
    Disc { center: [f32; 2], radius: f32 },
    /// ⭐ **A metade da frente de uma argola**, como poligonal.
    ///
    /// ⚠️ Só a metade da frente, como no Blender: a de trás está do outro lado da peça, e desenhá-la
    /// faz uma argola vista de lado parecer duas linhas cruzadas em vez de um anel.
    Arc(Vec<[f32; 2]>),
    /// Punho de tamanho: um quadrado no fim de um traço a partir do centro.
    Grip { from: [f32; 2], to: [f32; 2] },
}

/// Uma alça pronta. `live = false` ⇒ **nem pintada nem apontável** neste enquadramento.
#[derive(Clone, Debug)]
pub(crate) struct Projected {
    pub(crate) handle: Handle,
    pub(crate) shape: Shape,
    pub(crate) live: bool,
}

/// ⭐ **O que um arrasto pede** — e por que não é sempre um vetor.
///
/// ⚠️ Os três verbos compõem de formas diferentes: translação **soma**, rotação **compõe** e escala
/// **multiplica**. Um `[f32; 3]` para os três obrigaria quem recebe a adivinhar qual é qual pelo
/// modo em que o gizmo estava — e num quadro em que o modo mudou a meio, a adivinha erra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Motion {
    Translate([f32; 3]),
    /// Em torno de `axis` (unitário, no mundo), pelo **pivô da âncora**.
    Rotate {
        axis: [f32; 3],
        angle: f32,
    },
    /// Fator **uniforme**. Ver o doc do módulo.
    Scale(f32),
}

impl Motion {
    /// ⭐ **Acumular dois pedidos do MESMO arrasto.**
    ///
    /// ⚠️ Entre dois quadros chegam vários eventos de ponteiro, e cada verbo tem a sua lei de
    /// composição. Somar fatores de escala, por exemplo, faria dois passos de ×1,1 valerem ×2,2 em
    /// vez de ×1,21 — e o defeito só apareceria com o rato depressa, que é o mais difícil de
    /// acreditar quando alguém o reporta.
    ///
    /// Variantes diferentes não se compõem: o segundo ganha. Não pode acontecer num arrasto (a alça
    /// fixa o verbo), e inventar uma soma entre um giro e uma escala seria pior do que ceder.
    pub(crate) fn merge(self, next: Motion) -> Motion {
        match (self, next) {
            (Motion::Translate(a), Motion::Translate(b)) => {
                Motion::Translate([a[0] + b[0], a[1] + b[1], a[2] + b[2]])
            }
            (Motion::Rotate { axis, angle: a }, Motion::Rotate { angle: b, .. }) => {
                Motion::Rotate { axis, angle: a + b }
            }
            (Motion::Scale(a), Motion::Scale(b)) => Motion::Scale(a * b),
            (_, other) => other,
        }
    }

    /// ⭐ **O que falta aplicar**, dado o que já foi: `self` é o TOTAL desde a pegada e `applied` o
    /// que o mundo já recebeu.
    ///
    /// ⚠️ É a inversa exacta de [`Motion::merge`], e existe pelo mesmo motivo que ela: cada verbo
    /// compõe à maneira dele. `total.since(applied).merge(applied) == total` — que é o gate.
    pub(crate) fn since(self, applied: Motion) -> Motion {
        match (self, applied) {
            (Motion::Translate(t), Motion::Translate(a)) => {
                Motion::Translate([t[0] - a[0], t[1] - a[1], t[2] - a[2]])
            }
            (Motion::Rotate { axis, angle: t }, Motion::Rotate { angle: a, .. }) => {
                Motion::Rotate { axis, angle: t - a }
            }
            (Motion::Scale(t), Motion::Scale(a)) if a.abs() > f32::MIN_POSITIVE => {
                Motion::Scale(t / a)
            }
            (total, _) => total,
        }
    }

    /// O pedido **neutro** deste verbo — o ponto de partida de um arrasto.
    pub(crate) fn neutral(self) -> Motion {
        match self {
            Motion::Translate(_) => Motion::Translate([0.0; 3]),
            Motion::Rotate { axis, .. } => Motion::Rotate { axis, angle: 0.0 },
            Motion::Scale(_) => Motion::Scale(1.0),
        }
    }

    /// ⭐ **O mesmo pedido, preso à grelha** — o gesto de precisão (`Ctrl`).
    ///
    /// `step` é o passo da translação, em unidades de mundo, e vem **derivado do enquadramento**
    /// ([`snap_step`]). O ângulo e o fator têm passos próprios, e cada um diz por que é aquele.
    pub(crate) fn snapped(self, step: f32) -> Motion {
        let round_to = |v: f32, q: f32| -> f32 { if q > 0.0 { (v / q).round() * q } else { v } };
        match self {
            Motion::Translate(d) => Motion::Translate([
                round_to(d[0], step),
                round_to(d[1], step),
                round_to(d[2], step),
            ]),
            // ⚠️ **15°, e a razão é o que se pede pelo NOME**: é o maior passo que ainda contém 30,
            // 45, 60 e 90 — os ângulos que um artista diz em voz alta. Um passo mais fino não os
            // perde, mas obriga a mira; um mais grosso perde o 45.
            Motion::Rotate { axis, angle } => Motion::Rotate {
                axis,
                angle: round_to(angle, SNAP_ANGLE),
            },
            // ⚠️ **O passo do fator é o que a LEITURA consegue exprimir.** O número aparece com uma
            // casa decimal, então prender a 0,1 faz um valor preso ser exatamente o que se lê. Um
            // passo mais fino mostraria "×1,5" para dois tamanhos diferentes.
            Motion::Scale(f) => Motion::Scale(round_to(f, SNAP_FACTOR).max(SNAP_FACTOR)),
        }
    }

    /// Um pedido que não pede nada — o que uma alça degenerada devolve.
    pub(crate) fn is_idle(self) -> bool {
        match self {
            Motion::Translate(d) => d.iter().all(|v| v.abs() < f32::EPSILON),
            Motion::Rotate { angle, .. } => angle.abs() < f32::EPSILON,
            Motion::Scale(f) => (f - 1.0).abs() < f32::EPSILON,
        }
    }
}

/// Os três eixos do mundo.
const WORLD_AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// O passo de ângulo do gesto preso. Ver [`Motion::snapped`].
pub(crate) const SNAP_ANGLE: f32 = std::f32::consts::PI / 12.0;

/// O passo do fator de tamanho. Ver [`Motion::snapped`].
pub(crate) const SNAP_FACTOR: f32 = 0.1;

/// ⭐ **O passo da translação presa, DERIVADO do enquadramento** — o menor número redondo (1-2-5)
/// cujo comprimento na tela ainda se consegue mirar.
///
/// ⚠️ Um passo fixo em unidades de mundo é inútil nos dois extremos: aproximado, dois pontos da
/// grelha ficam a meia tela um do outro; afastado, ficam dentro do mesmo pixel. A grelha do Blender
/// subdivide com o zoom pela mesma razão.
///
/// **A condição que fixa o número:** dois pontos vizinhos da grelha têm de estar mais afastados do
/// que a tolerância do próprio ponteiro ([`GRAB_PX`]) — abaixo disso o gesto deixa de conseguir
/// escolher entre eles, e prender à grelha passa a ser sorteio. Sobe-se então a escada 1-2-5 até o
/// primeiro degrau que passa.
#[must_use]
pub(crate) fn snap_step(screen: Screen) -> f32 {
    let min_world = GRAB_PX / screen.px_per_world().max(f32::MIN_POSITIVE);
    if !min_world.is_finite() || min_world <= 0.0 {
        return SNAP_FACTOR;
    }
    let decade = 10f32.powf(min_world.log10().floor());
    for m in [1.0, 2.0, 5.0] {
        let step = m * decade;
        if step >= min_world {
            return step;
        }
    }
    decade * 10.0
}

/// **Projeta o gizmo inteiro**, no modo dado. A ordem é a de apontar: do centro para fora.
///
/// ⚠️ **A ordem é load-bearing** — [`pick`] devolve a primeira que casa, e o disco de vista está por
/// dentro da folga onde as setas não começam. Sem esta ordem, apontar o centro escolheria um eixo à
/// sorte.
pub(crate) fn project(anchor: Anchor, cam: &Orbit, screen: Screen, mode: Mode) -> Vec<Projected> {
    let px_per_world = screen.px_per_world().max(f32::MIN_POSITIVE);
    let arm = ARM_PX / px_per_world;
    let (o2, _) = cam.project(anchor.origin, screen);
    match mode {
        Mode::Move => move_handles(anchor, cam, screen, arm, o2),
        Mode::Rotate => rotate_handles(anchor, cam, screen, arm),
        Mode::Scale => vec![Projected {
            handle: Handle::Grip,
            shape: Shape::Grip {
                from: o2,
                to: [o2[0] + GRIP_DIR[0] * ARM_PX, o2[1] + GRIP_DIR[1] * ARM_PX],
            },
            // Um punho de TELA não tem como degenerar: ele não é uma direção do mundo.
            live: true,
        }],
    }
}

fn move_handles(
    anchor: Anchor,
    cam: &Orbit,
    screen: Screen,
    arm: f32,
    o2: [f32; 2],
) -> Vec<Projected> {
    let mut out = vec![Projected {
        handle: Handle::View,
        shape: Shape::Disc {
            center: o2,
            radius: INNER_PX,
        },
        // O plano da tela nunca fica de perfil consigo mesmo: esta alça é a única que não pode
        // degenerar, e é por isso que ela é a rede de segurança do enquadramento difícil.
        live: true,
    }];

    for n in 0..3 {
        let (u, v) = ((n + 1) % 3, (n + 2) % 3);
        let corner = |a: f32, b: f32| -> [f32; 2] {
            let mut p = anchor.origin;
            for (k, c) in p.iter_mut().enumerate() {
                *c += anchor.axes[u][k] * a * arm + anchor.axes[v][k] * b * arm;
            }
            cam.project(p, screen).0
        };
        let (lo, hi) = (PLANE_AT, PLANE_AT + PLANE_SIDE);
        let quad = [
            corner(lo, lo),
            corner(hi, lo),
            corner(hi, hi),
            corner(lo, hi),
        ];
        // ⚠️ **De perfil, um quadrado é um traço.** A pergunta certa não é a área: é se ele ainda é
        // largo o bastante para se apontar — o lado mais estreito tem de passar do raio de agarre.
        let narrow = (0..4)
            .map(|i| dist(quad[i], quad[(i + 1) % 4]))
            .fold(f32::INFINITY, f32::min);
        out.push(Projected {
            handle: Handle::Plane(n),
            shape: Shape::Quad(quad),
            live: narrow >= GRAB_PX,
        });
    }

    for (n, axis) in anchor.axes.iter().enumerate() {
        let tip = cam.project(offset(anchor.origin, *axis, arm), screen).0;
        let len = dist(o2, tip);
        out.push(Projected {
            handle: Handle::Axis(n),
            shape: Shape::Arrow { from: o2, to: tip },
            live: len >= MIN_ARM_PX,
        });
    }
    out
}

fn rotate_handles(anchor: Anchor, cam: &Orbit, screen: Screen, arm: f32) -> Vec<Projected> {
    let (_, _, fwd) = cam.basis();
    let mut out = Vec::with_capacity(4);
    for (n, axis) in anchor.axes.iter().enumerate() {
        out.push(Projected {
            handle: Handle::Ring(n),
            shape: Shape::Arc(front_arc(anchor.origin, *axis, arm, cam, screen)),
            live: dot(*axis, fwd).abs() >= RING_MIN_DOT,
        });
    }
    // ⭐ A argola de VISTA fica por fora e é a única que não pode ficar de perfil consigo mesma —
    // a rede de segurança do enquadramento difícil, como o disco no modo de mover.
    out.push(Projected {
        handle: Handle::ViewRing,
        shape: Shape::Arc(front_arc(
            anchor.origin,
            fwd,
            arm * VIEW_RING_R,
            cam,
            screen,
        )),
        live: true,
    });
    out
}

/// A **metade da frente** de um círculo do mundo, projetada — ou o círculo inteiro quando ele está
/// de frente para a câmera.
///
/// ⚠️ A metade da frente é um trecho **contíguo** do círculo (um plano corta uma circunferência em
/// exatamente dois pontos), mas ele pode dar a volta ao fim do vetor de amostras. Por isso a
/// travessia começa onde o trecho começa, e não no índice zero — cortar em zero partiria a argola em
/// duas no meio da tela.
fn front_arc(
    origin: [f32; 3],
    axis: [f32; 3],
    radius: f32,
    cam: &Orbit,
    screen: Screen,
) -> Vec<[f32; 2]> {
    let (u, v) = basis_of(axis);
    let (_, _, fwd) = cam.basis();
    let world = |i: usize| -> [f32; 3] {
        let t = i as f32 / RING_SEGMENTS as f32 * std::f32::consts::TAU;
        let (s, c) = t.sin_cos();
        let mut p = origin;
        for k in 0..3 {
            p[k] += (u[k] * c + v[k] * s) * radius;
        }
        p
    };
    // ⚠️ **A profundidade sai do DESLOCAMENTO, nunca de `ponto − origem`.** A subtração cancela dois
    // números grandes e o erro que sobra é da ordem da própria origem — numa peça longe do zero, o
    // sinal da conta passa a ser ruído. Aqui o deslocamento já é o que se quer, e o erro fica da
    // ordem de `radius · 10⁻⁷`.
    let (du, dv) = (dot(u, fwd), dot(v, fwd));
    let front: Vec<bool> = (0..RING_SEGMENTS)
        .map(|i| {
            let t = i as f32 / RING_SEGMENTS as f32 * std::f32::consts::TAU;
            let (s, c) = t.sin_cos();
            c.mul_add(du, s * dv) >= -RING_FRONT_EPS
        })
        .collect();
    if front.iter().all(|f| *f) {
        return (0..=RING_SEGMENTS)
            .map(|i| cam.project(world(i % RING_SEGMENTS), screen).0)
            .collect();
    }
    let Some(start) =
        (0..RING_SEGMENTS).find(|&i| front[i] && !front[(i + RING_SEGMENTS - 1) % RING_SEGMENTS])
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for k in 0..RING_SEGMENTS {
        let i = (start + k) % RING_SEGMENTS;
        if !front[i] {
            break;
        }
        out.push(cam.project(world(i), screen).0);
    }
    out
}

/// Dois vetores unitários que geram o plano perpendicular a `axis`.
///
/// ⚠️ O parceiro do produto vetorial é escolhido pelo **eixo menos alinhado** com `axis`. Um
/// parceiro fixo daria produto nulo exatamente quando `axis` fosse ele — e o sintoma seria uma
/// argola que desaparece num dos três eixos, e só nele.
fn basis_of(axis: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = normalize(axis);
    let small = (0..3)
        .min_by(|&i, &j| a[i].abs().total_cmp(&a[j].abs()))
        .unwrap_or(0);
    let mut helper = [0.0f32; 3];
    helper[small] = 1.0;
    let u = normalize(cross(a, helper));
    (u, cross(a, u))
}

/// **De quem é este ponto?** — `None` quando nenhuma alça o reclama.
pub(crate) fn pick(projected: &[Projected], p: [f32; 2]) -> Option<Handle> {
    projected
        .iter()
        .find(|h| h.live && hits(&h.shape, p))
        .map(|h| h.handle)
}

fn hits(shape: &Shape, p: [f32; 2]) -> bool {
    match shape {
        Shape::Disc { center, radius } => dist(*center, p) <= *radius,
        Shape::Quad(q) => point_in_quad(*q, p),
        // ⚠️ A haste começa DEPOIS da folga: sem isto as três setas disputariam o centro com o
        // disco, e qual ganha dependeria da ordem da lista em vez da geometria.
        Shape::Arrow { from, to } => {
            let d = [to[0] - from[0], to[1] - from[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if len <= INNER_PX {
                return false;
            }
            let u = [d[0] / len, d[1] / len];
            let start = [from[0] + u[0] * INNER_PX, from[1] + u[1] * INNER_PX];
            dist_to_segment(start, *to, p) <= GRAB_PX
        }
        Shape::Arc(pts) => pts
            .windows(2)
            .any(|w| dist_to_segment(w[0], w[1], p) <= GRAB_PX),
        // O punho é o quadrado do fim; o traço até ele é decoração e não se agarra.
        Shape::Grip { to, .. } => {
            (p[0] - to[0]).abs() <= GRIP_HALF_PX + GRAB_PX * 0.5
                && (p[1] - to[1]).abs() <= GRIP_HALF_PX + GRAB_PX * 0.5
        }
    }
}

/// ⭐ **O arrasto**: o que o nó faz quando o ponteiro vai de `from_px` a `to_px`.
///
/// Devolve um pedido **inerte** ([`Motion::is_idle`]) quando a alça não é utilizável neste
/// enquadramento — a mesma condição que [`project`] usa para a esconder, porque uma alça invisível
/// não pode arrastar.
pub(crate) fn drag(
    handle: Handle,
    anchor: Anchor,
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> Motion {
    let px_per_world = screen.px_per_world().max(f32::MIN_POSITIVE);
    let arm = ARM_PX / px_per_world;
    match handle {
        // A conta é uma projeção escalar: `d` é quanto o braço inteiro mede na tela, e a fração do
        // movimento do rato ao longo dele é a fração do braço que a peça anda.
        //
        // ⚠️ **Sem divisão por zero possível**: `dot(d,d)` só é nulo quando o eixo aponta ao
        // observador, e aí a alça já não está viva.
        Handle::Axis(n) => {
            let (o2, _) = cam.project(anchor.origin, screen);
            let tip = cam
                .project(offset(anchor.origin, anchor.axes[n], arm), screen)
                .0;
            let d = [tip[0] - o2[0], tip[1] - o2[1]];
            let dd = d[0].mul_add(d[0], d[1] * d[1]);
            if dd < MIN_ARM_PX * MIN_ARM_PX {
                return Motion::Translate([0.0; 3]);
            }
            let m = [to_px[0] - from_px[0], to_px[1] - from_px[1]];
            let t = m[0].mul_add(d[0], m[1] * d[1]) / dd * arm;
            Motion::Translate([
                anchor.axes[n][0] * t,
                anchor.axes[n][1] * t,
                anchor.axes[n][2] * t,
            ])
        }
        // Num plano, o deslocamento é a diferença entre dois pontos do plano — cada um o encontro do
        // raio do cursor com ele. É a mesma conta do gizmo 2D, com o plano a vir do mundo.
        Handle::Plane(n) => Motion::Translate(plane_delta(
            anchor.axes[n],
            anchor.origin,
            cam,
            screen,
            from_px,
            to_px,
        )),
        // O plano da tela: a normal é a direção da vista, e o denominador vale 1 — nunca degenera.
        Handle::View => {
            let (_, _, fwd) = cam.basis();
            Motion::Translate(plane_delta(fwd, anchor.origin, cam, screen, from_px, to_px))
        }
        Handle::Ring(n) => spin(anchor.axes[n], anchor.origin, cam, screen, from_px, to_px),
        Handle::ViewRing => {
            let (_, _, fwd) = cam.basis();
            spin(fwd, anchor.origin, cam, screen, from_px, to_px)
        }
        // ⭐ Tamanho é **razão de raios**, e não diferença: é o que faz duas metades de um arrasto
        // valerem o produto e não a soma — a mesma lei que um zoom de roda usa.
        Handle::Grip => {
            let (o2, _) = cam.project(anchor.origin, screen);
            let r0 = dist(o2, from_px);
            let r1 = dist(o2, to_px);
            // ⚠️ O piso é do RAIO INICIAL, não do fator: agarrar em cima do centro daria uma razão
            // infinita e a peça saltaria num pixel. O punho vive a `ARM_PX` do centro, então este
            // piso só morde se alguém arrastar **para dentro** do centro.
            if r0 < GRAB_PX || !r1.is_finite() {
                return Motion::Scale(1.0);
            }
            Motion::Scale((r1 / r0).max(f32::MIN_POSITIVE))
        }
    }
}

/// ⭐ **O ângulo que o cursor varreu em torno de um eixo** — medido **no plano de rotação**, e não
/// na tela.
///
/// ⚠️ A alternativa (medir o ângulo em pixels em torno do centro projetado) é a que muitos editores
/// usam, e ela **mente fora do eixo da vista**: a projeção de um círculo é uma elipse, e o ângulo na
/// elipse não é o ângulo no círculo. O gesto ficaria rápido de um lado e lento do outro, e uma volta
/// inteira não fecharia. Aqui os dois pontos do cursor são levados ao plano real e o ângulo sai do
/// produto vetorial — exato, e com o sinal já certo.
fn spin(
    axis: [f32; 3],
    origin: [f32; 3],
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> Motion {
    let axis = normalize(axis);
    let idle = Motion::Rotate { axis, angle: 0.0 };
    let (Some(a), Some(b)) = (
        ray_plane(cam, screen, from_px, origin, axis),
        ray_plane(cam, screen, to_px, origin, axis),
    ) else {
        return idle;
    };
    let d0 = [a[0] - origin[0], a[1] - origin[1], a[2] - origin[2]];
    let d1 = [b[0] - origin[0], b[1] - origin[1], b[2] - origin[2]];
    // Em cima do eixo o ângulo é ruído: um pixel de rato varreria meia volta.
    if len3(d0) < f32::EPSILON || len3(d1) < f32::EPSILON {
        return idle;
    }
    let angle = dot(cross(d0, d1), axis).atan2(dot(d0, d1));
    Motion::Rotate { axis, angle }
}

fn plane_delta(
    normal: [f32; 3],
    origin: [f32; 3],
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> [f32; 3] {
    let a = ray_plane(cam, screen, from_px, origin, normal);
    let b = ray_plane(cam, screen, to_px, origin, normal);
    match (a, b) {
        (Some(a), Some(b)) => [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        _ => [0.0; 3],
    }
}

/// Onde o raio de um pixel encontra o plano que passa por `p0` com normal `n`. `None` de perfil.
fn ray_plane(
    cam: &Orbit,
    screen: Screen,
    px: [f32; 2],
    p0: [f32; 3],
    n: [f32; 3],
) -> Option<[f32; 3]> {
    let (o, dir) = cam.ray(px[0], px[1], screen);
    let denom = dot(dir, n);
    // ⚠️ O limiar não é folclore: abaixo dele um pixel de rato vale um salto arbitrário no plano, e
    // o gesto deixa de ser manipulação para ser sorteio. É a mesma razão de `MIN_ARM_PX`.
    if denom.abs() < 1.0e-3 {
        return None;
    }
    let t = dot([p0[0] - o[0], p0[1] - o[1], p0[2] - o[2]], n) / denom;
    Some([o[0] + dir[0] * t, o[1] + dir[1] * t, o[2] + dir[2] * t])
}

fn offset(p: [f32; 3], dir: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k]
}

fn len3(v: [f32; 3]) -> f32 {
    dot(v, v).sqrt()
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let n = len3(v);
    if n <= 0.0 || !n.is_finite() {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / n, v[1] / n, v[2] / n]
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn dist_to_segment(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let dd = d[0].mul_add(d[0], d[1] * d[1]);
    if dd <= f32::MIN_POSITIVE {
        return dist(a, p);
    }
    let t = ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / dd;
    let t = t.clamp(0.0, 1.0);
    dist([a[0] + d[0] * t, a[1] + d[1] * t], p)
}

/// ⚠️ **Por produto vetorial, e não por «está dentro da caixa»**: o quadrilátero é um quadrado do
/// MUNDO já projetado, então ele é um losango qualquer na tela. Um teste de caixa alinhada
/// reclamaria pixels que não são dele — e como as três alças de plano se tocam nos cantos, o gesto
/// escolheria a errada exatamente onde a diferença importa.
fn point_in_quad(q: [[f32; 2]; 4], p: [f32; 2]) -> bool {
    let mut positive = false;
    let mut negative = false;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        if cross > 0.0 {
            positive = true;
        }
        if cross < 0.0 {
            negative = true;
        }
    }
    !(positive && negative)
}

#[cfg(test)]
#[path = "field3d_gizmo_tests.rs"]
mod tests;
