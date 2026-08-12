//! **QUEM ESTA ENCOSTADO** (`PH2D_GPU_COOK_DEMO=30`) — o P1 do evento de contato da folha 13 do
//! doc 89 montado como documento pronto para smoke: *uma colisão mudava `P` e `vel` e **nada a
//! jusante conseguia saber que ela aconteceu***.
//!
//! O passo muda esses dois em TODO tique, então *"tocou?"* era uma pergunta que o grafo não sabia
//! fazer. Agora o `sim.collide` escreve a coluna `hit` — quão fundo a colisão deste tique empurrou
//! o elemento de volta — e ela é **lida pelo domínio de VALOR**, o que torna a colisão componível:
//!
//! ```text
//!   sim.collide  ->  value.attribute(Hit)  ->  motion.drive(Size, Add)
//! ```
//!
//! ⚠️ **Nenhum nó novo.** A cena inteira é feita de nós que já existiam; o que nasceu foi a
//! coluna, e é por isso que a mesma cadeia com `motion.drive(Falloff)` + `motion.cull` dentro da
//! zona faz o elemento **MORRER ao tocar** (a linha 98 do doc 63) sem um nó a mais.
//!
//! ⚠️ **São DOIS colisores, e isso é o teste da lei de acumulação:** um disco no meio e um chão
//! embaixo. A coluna acumula por `max` ao longo do tique, então *o mais fundo do tique* — nunca
//! *"o último colisor da cadeia"*, que seria um fato sobre a ordem em que o artista ligou os fios.
//!
//! ⚠️ **E ela é TRANSIENTE:** a `sim.zone` a tira do estado, como faz com `accel`. Guardada, ela
//! diria "tocou" no tique seguinte ao que parou de tocar.
//!
//! ⚠️ **E o canal e' um INSTANTE, nao uma historia** — a versao de 2026-08-10 desta cena somava
//! `hit` no `size` com `motion.drive(Add)` e chamava isso de "marca". Um elemento em REPOUSO
//! continua em contato, entao a soma nunca tinha ponto fixo: medido, o maior `size` ia a 0,455 ·
//! 0,892 · 1,197 · 1,963 · **3,021** aos 8 s com a chuva ja' parada. Quanto mais tempo o artista
//! deixasse tocar, maiores as bolas — *"cada Play um resultado diferente"* (report do Enio,
//! 2026-08-11). O anuncio da cena ate' NOMEAVA o risco (*"a `sim.zone` a tira do estado, senao a
//! marca cresceria para sempre"*) e atribuia a cura ao lugar errado: o strip zera o CANAL, nunca
//! o `size` que o `Add` ja' empilhou.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Onde o obstáculo fica e quanto ele mede. Alto o bastante para a chuva se dividir: parte bate
/// nele cedo, parte segue até o chão — e é a diferença entre as duas que se vê num quadro só.
pub(super) const DISC_Y: f32 = 0.5;
pub(super) const DISC_R: f32 = 1.6;
/// A altura do chão. O `sim.collide` a lê como a forma de Hesse (distância à origem ao longo da
/// normal), e num plano sem inclinação isso É a altura.
pub(super) const FLOOR: f32 = -2.5;
/// Quanto um contato engorda o elemento — o DELTA sobre [`BASE`], nao mais uma escala de
/// acumulacao. **MEDIDO**, nao escolhido (`probe_hit_mark`): aos 3 s a chuva assentada mede
/// `0,720` contra os `0,220` de quem nao toca — **3,3x**, e o numero PARA (0,720 aos 3, 5 e 8 s).
pub(super) const MARK: f32 = 0.5;
/// O tamanho de nascimento — a porta ÚNICA: o `motion.scale` o dá e o `map_range` o devolve a
/// quem não tocou em nada. Dois números aqui seriam duas respostas para *"de que tamanho nasce?"*.
pub(super) const BASE: f32 = 0.22;
/// A profundidade de contato que ja' vale a marca CHEIA — o ponto onde o `clamp` do `map_range`
/// SATURA, e e' ele que mata a cintilacao. **MEDIDA, e o 1o valor estava errado:** com 0,004 o
/// tamanho de um elemento assentado oscilava **0,101** entre 5 s e 8 s com a posicao PARADA,
/// porque a profundidade por tique de um corpo em repouso e' um CICLO-LIMITE (ele afunda um
/// pouco e e' empurrado de volta), nao uma constante. Saturado, a leitura vira binaria — no ar
/// ou encostado — e o quadro fica IMOVEL.
pub(super) const HIT_FULL: f32 = 0.0008;
pub(super) const ROWS: f32 = 4.0;
pub(super) const COLS: f32 = 14.0;

/// **QUEM ESTA ENCOSTADO** (`PH2D_GPU_COOK_DEMO=30`).
///
/// `mark` existe para o GATE: com `0.0` a cadeia inteira continua lá — o canal é lido, o valor
/// viaja, o drive roda — e **nada** engorda, que é o controle de que o tamanho na tela veio do
/// contato e não de outra coisa da cena.
pub(super) fn build_gpu_hit_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    mark: f32,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", ROWS);
    g.set_param(grid, "cols", COLS);
    g.set_param(grid, "gap_x", 0.55);
    g.set_param(grid, "gap_y", 0.55);

    // A chuva nasce ALTA, acima do obstáculo — ver a queda é o que torna o repouso legível.
    let lift = g.add_node("motion.transform");
    g.set_param(lift, "offset_y", 4.5);

    let size = g.add_node("motion.scale");
    g.set_param(size, "amount", BASE);

    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", 5.0);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 0.9);

    // O OBSTÁCULO. `Disc` = o mundo é tudo FORA dele.
    let disc = g.add_node("sim.collide");
    g.set_param(disc, "shape", 1.0);
    g.set_param(disc, "center_x", 0.0);
    g.set_param(disc, "center_y", DISC_Y);
    g.set_param(disc, "radius", DISC_R);
    g.set_param(disc, "restitution", 0.15);
    g.set_param(disc, "friction", 0.05);

    // O CHÃO, encadeado depois dele — a coluna `hit` acumula pelos dois.
    let floor = g.add_node("sim.collide");
    g.set_param(floor, "shape", 0.0);
    g.set_param(floor, "height", FLOOR);
    g.set_param(floor, "restitution", 0.0);
    g.set_param(floor, "friction", 0.35);

    // A LEITURA — o canal de contato entrando no domínio de valor.
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, "attr", "hit");
    g.set_param(attr, "mode", 0.0);

    // …e o TAMANHO que ela decide. ⚠️ **`Set`, nunca `Add`** — o canal `hit` é um INSTANTE (a
    // profundidade que este tique empurrou para fora), e um elemento em repouso continua em
    // contato para sempre. Somar um instantâneo a cada tique é integrar sem teto: medido, o
    // maior `size` ia a 0,455 · 0,892 · 1,197 · 1,963 · **3,021** aos 8 s com a chuva já
    // PARADA — quanto mais tempo o artista deixasse tocar, maiores as bolas, e o quadro nunca
    // era o mesmo duas vezes (report do Enio, 2026-08-11).
    //
    // O `Set` sozinho zeraria quem não toca (o elemento SUMIRIA), então quem carrega a base é o
    // próprio valor: o `map_range` leva `hit` de [0, HIT_FULL] para [BASE, BASE + mark], com
    // clamp. Contato nenhum ⇒ o tamanho de nascimento, ao bit; contato ⇒ gordo, e nunca além.
    let mapped = g.add_node("value.map_range");
    g.set_param(mapped, "in_lo", 0.0);
    g.set_param(mapped, "in_hi", HIT_FULL);
    g.set_param(mapped, "out_lo", BASE);
    g.set_param(mapped, "out_hi", BASE + mark);
    g.set_param(mapped, "clamp", 1.0);

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 3.0); // Size
    g.set_param(drive, "mode", 1.0); // Set
    g.set_param(drive, "scale", 1.0);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, lift, size, zone].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 170.0,
                y: 260.0,
            },
        );
    }
    for (i, n) in [wind, step, disc, floor, mapped, drive, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 420.0 + i as f32 * 170.0,
                y: 400.0,
            },
        );
    }
    g.set_pos(
        attr,
        Pos {
            x: 420.0 + 4.0 * 170.0,
            y: 560.0,
        },
    );

    for (a, ap, b, bp, delayed) in [
        (grid, 0u16, lift, 0u16, false),
        (lift, 0, size, 0, false),
        (size, 0, zone, 0, false),
        (zone, 0, wind, 0, true),
        (wind, 0, step, 0, false),
        (step, 0, disc, 0, false),
        (disc, 0, floor, 0, false),
        // O MESMO stream por duas portas: o fio grosso (as instâncias) e a leitura do canal.
        (floor, 0, drive, 0, false),
        (floor, 0, attr, 0, false),
        (attr, 0, mapped, 0, false),
        (mapped, 0, drive, 1, false),
        (drive, 0, zone, 1, false),
        (zone, 0, out, 0, false),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_gpu_hit_demo_tests.rs"]
mod tests;
