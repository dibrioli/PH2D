//! **A TINTA POUSA SOBRE O CHÃO** (`PH2D_GPU_COOK_DEMO=28`) — o 2º P0 da folha 13 do doc 89
//! montado como documento pronto para smoke: *o colisor colidia um PONTO*.
//!
//! Um elemento que repousa põe o **centro** sobre a linha do chão, então o sprite que o
//! renderizador desenha ali **afunda por metade da própria altura** — e a compensação óbvia
//! (subir o `height`) só funciona com tamanho UNIFORME, porque `size` é uma coluna por
//! elemento e `height` é um param, um número por tique mesmo quando dirigido.
//!
//! **A cena mostra as duas metades lado a lado**, porque a única forma de ver um raio é ver o
//! que acontece sem ele:
//!
//! ```text
//!   ESQUERDA   Radius From = Point         os discos afundam, cada um pela sua metade
//!   DIREITA    Radius From = Sprite Size   os discos POUSAM, e as bordas de baixo alinham
//! ```
//!
//! ⚠️ **Os tamanhos VARIAM de propósito, e é isso que torna a cena um oráculo em vez de uma
//! foto.** Com discos todos iguais, um `height` subido à mão daria o mesmo desenho — a fileira
//! ficaria certa e ninguém saberia se o raio funciona. Com cinco tamanhos, *"as bordas de baixo
//! formam uma linha reta"* é uma afirmação que **só** um raio por-elemento consegue satisfazer.
//!
//! ⚠️ **E é a MESMA cadeia dos dois lados** — mesma grade, mesma gravidade, mesmo chão, mesmo
//! `sim.step`: os dois `sim.collide` diferem em UM param. Se a diferença viesse de qualquer
//! outra coisa, a comparação não diria nada sobre o raio.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// A altura do chão, nas duas metades. Um número só: as duas fileiras têm de pousar na MESMA
/// linha, senão o olho compara duas coisas que já eram diferentes.
pub(super) const FLOOR: f32 = -2.6;
/// Quantos discos por fileira. **Cinco**, porque o oráculo é *"as bordas de baixo alinham"* e
/// duas alinham por acidente; e mais que cinco começa a encostar um no outro nesta escala.
pub(super) const COLS: f32 = 5.0;
/// O menor e o maior disco da fileira. **MEDIDO** (`probe_radius_rest`): com estes números o
/// afundamento do lado esquerdo vai de **0,15 a 0,60 unidade** — o pequeno mal se nota, o
/// grande está meio enterrado, e é o CONTRASTE dentro da mesma fileira que denuncia o ponto.
/// Tamanhos todos iguais tornariam o defeito indistinguível de um chão mal posicionado.
pub(super) const SIZE_MIN: f32 = 0.3;
/// Ver [`SIZE_MIN`].
pub(super) const SIZE_MAX: f32 = 1.2;
/// A largura que a fileira ocupa, em unidades de mundo — `(COLS − 1) × gap_x`, e o `motion.grid`
/// centra-se, então ela vai de `−ROW_SPAN/2` a `+ROW_SPAN/2` em torno do deslocamento.
/// É a régua da rampa: o feather do campo TEM de medir exatamente isto, senão os discos das
/// pontas caem no plateau (todos iguais) ou fora dele (todos zero).
const ROW_SPAN: f32 = (COLS - 1.0) * GAP_X;
/// O espaçamento da fileira. Uma const porque [`ROW_SPAN`] a lê — dois números que têm de
/// concordar são um número.
const GAP_X: f32 = 1.5;

/// **A TINTA POUSA SOBRE O CHÃO** (`PH2D_GPU_COOK_DEMO=28`).
///
/// O que o artista vê: duas fileiras de discos de tamanhos diferentes caindo sobre o mesmo
/// chão. À esquerda cada um **afunda pela própria metade** — os centros alinham e as formas
/// não. À direita eles **pousam**: os centros ficam em cinco alturas distintas e as bordas de
/// baixo formam uma linha.
pub(super) fn build_gpu_radius_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // Uma metade: grade -> rampa de tamanho -> zona -> gravidade -> passo -> colisor.
    // `radius_from` é o ÚNICO param que difere entre as duas chamadas.
    let mut half = |x_offset: f32, radius_from: f32, row: f32| -> Option<NodeId> {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP_X);
        // A fileira nasce ALTA e cai — ver a queda é o que torna o repouso legível como pouso.
        let lift = g.add_node("motion.transform");
        g.set_param(lift, "offset_x", x_offset);
        g.set_param(lift, "offset_y", 3.2);

        // A RAMPA DE TAMANHO, em três nós que já existem. ⚠️ O `falloff` é uma COLUNA que
        // viaja no stream (não uma entrada lateral): o campo a escreve e todo nó a jusante a
        // lê, então a rampa é uma CADEIA e a ordem é a lei —
        //
        //   base   `motion.scale(SIZE_MIN)` com `falloff` AUSENTE (lê 1) ⇒ a fileira inteira
        //          encolhe de SIZE_IDENTITY para SIZE_MIN;
        //   campo  `field.box` escreve `falloff` = 0..1 pela POSIÇÃO em x (ver abaixo);
        //   rampa  `motion.scale(SIZE_MAX / SIZE_MIN)` — o fator é `1 + (A-1)·f`, então o
        //          posto 0 fica em SIZE_MIN e o último em SIZE_MAX, exatos.
        //
        // Inverter base e rampa daria a base modulada pelo campo e a rampa sobre um tamanho
        // que já variava: os dois `motion.scale` LEEM a mesma coluna.
        let base = g.add_node("motion.scale");
        g.set_param(base, "amount", SIZE_MIN);
        // ⚠️ **O campo é `field.box`, e NÃO o `field.index_range`** — este é o irmão ordinal e
        // seria a escolha óbvia, mas ele é uma BANDA: `start=0, end=1, soft=0` é literalmente
        // o neutro documentado dele (máscara 1 em toda parte), e nenhum ajuste o torna
        // monotônico — um trapézio com as duas rampas dá um TRIÂNGULO, não uma escada.
        //
        // O `field.box` tem `soft` em unidades de MUNDO, então uma caixa cuja borda esquerda
        // cai no início da fileira e cujo feather cobre os 6 de largura dela deixa os cinco
        // discos inteiramente dentro da SUBIDA: um por degrau, monotônico e linear.
        let band = g.add_node("field.box");
        g.set_param(band, "width", 20.0);
        g.set_param(band, "height", 40.0); // plateau em y: a máscara é função só do x
        g.set_param(band, "soft", ROW_SPAN);
        g.set_param(band, "center_x", x_offset + 10.0 - ROW_SPAN * 0.5);
        g.set_param(band, "center_y", 0.0);
        g.set_param(band, "curve", 0.0); // Linear: cinco discos igualmente espaçados
        let scale = g.add_node("motion.scale");
        g.set_param(scale, "amount", SIZE_MAX / SIZE_MIN);

        let zone = g.add_node("sim.zone");
        let wind = g.add_node("force.wind");
        g.set_param(wind, "angle", 270.0); // para baixo
        g.set_param(wind, "strength", 3.0);
        let step = g.add_node("sim.step");
        g.set_param(step, "damping", 0.6);
        let ground = g.add_node("sim.collide");
        g.set_param(ground, "shape", 0.0); // Floor
        g.set_param(ground, "height", FLOOR);
        g.set_param(ground, "restitution", 0.0); // pousa, não quica: o repouso é o oráculo
        g.set_param(ground, "friction", 0.6);
        g.set_param(ground, "radius_from", radius_from);

        for (i, n) in [grid, lift, base, band, scale, zone]
            .into_iter()
            .enumerate()
        {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 160.0,
                    y: row,
                },
            );
        }
        for (i, n) in [wind, step, ground].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 260.0 + i as f32 * 160.0,
                    y: row + 130.0,
                },
            );
        }

        for (a, ap, b, bp, delayed) in [
            (grid, 0u16, lift, 0u16, false),
            (lift, 0, base, 0, false),
            (base, 0, band, 0, false),
            (band, 0, scale, 0, false),
            (scale, 0, zone, 0, false),
            // A entrada de estado que o motor gerencia.
            (zone, 0, wind, 0, true),
            (wind, 0, step, 0, false),
            (step, 0, ground, 0, false),
            (ground, 0, zone, 1, false),
        ] {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed,
            })
            .ok()?;
        }
        Some(zone)
    };

    // 0 = Point (o colisor de ontem) · 2 = Sprite Size (o desta wave).
    let left = half(-4.6, 0.0, 200.0)?;
    let right = half(2.4, 2.0, 620.0)?;

    let combine = g.add_node("motion.combine");
    let out = g.add_node("motion.output");
    g.set_pos(
        combine,
        Pos {
            x: 1180.0,
            y: 400.0,
        },
    );
    g.set_pos(
        out,
        Pos {
            x: 1340.0,
            y: 400.0,
        },
    );
    for (a, b, bp) in [(left, combine, 0u16), (right, combine, 1)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, bp),
            delayed: false,
        })
        .ok()?;
    }
    g.connect(Edge {
        from: (combine, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_gpu_radius_demo_tests.rs"]
mod tests;
