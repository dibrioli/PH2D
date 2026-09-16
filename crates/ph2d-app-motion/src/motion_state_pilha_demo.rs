//! **PEÇAS QUE NÃO SE ATRAVESSAM** (`PH2D_GPU_COOK_DEMO=114`) — o colisor NA FORMA
//! ([doc 109](../../docs/Motion%20Nodes/109_o_colisor_na_forma.md)).
//!
//! ## A ordem do dono que a reescreveu
//!
//! A cena nasceu (10/09) com um `motion.collide` na linha da simulação, e o smoke dela devolveu a
//! pergunta: *pôr o `Collide` ali, sem referência à forma, não é contra-intuitivo?* — *«Vou preferir
//! colocar na shape.»* Com as duas leituras à frente (13/09), o dono escolheu **«colidem sozinhas»**:
//! liga-se `Collide` no cartão da forma e as peças deixam de se atravessar, **sem nó nenhum** na
//! linha da simulação. ⛔ E há gate a dizê-lo.
//!
//! ## O que se vê
//!
//! Duas taças, os MESMOS quadrados a cair pela MESMA lei, e a cadeia difere numa CAIXA:
//!
//! ```text
//!   ESQUERDA   shape(Collide OFF) → duplicator ← grid → … → zona   um BORRÃO no fundo
//!   DIREITA    shape(Collide ON)  → duplicator ← grid → … → zona   uma PILHA
//! ```
//!
//! ⚠️ **A taça é o que torna a diferença visível.** Num chão plano as peças espalham-se e quase
//! não se sobrepõem sozinhas; uma taça junta-as todas no mesmo ponto baixo.
//!
//! ⚠️ **A taça pousa cada peça pelo colisor dela** (`Radius From: Auto`, o default do
//! `sim.collide`): à direita pela FACE da caixa, à esquerda — que não declarou nada — pelo centro.
//!
//! ⚠️ **A forma é um QUADRADO de propósito:** é a forma em que o círculo à volta (a 1.ª redacção,
//! `√2 · LADO`) deixava `41 %` de ar, e o dono viu-o na foto (doc 109 §5). A caixa declarada é o
//! próprio quadrado, e a pilha encosta.
//!
//! ⚠️ **Ela precisa de Play** — é uma simulação, como a `=99` e a `=113`.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_motion_shape::param;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças caem em cada taça: `5 × 5`. Poucas o bastante para se **contar** as da
/// direita, muitas o bastante para as da esquerda serem um borrão.
const COLS: f32 = 5.0;
const ROWS: f32 = 5.0;
/// O meio-lado de cada quadrado (o `size` do `source.shape`: a geometria nasce em raio 1).
const LADO: f32 = 0.11;
/// O vão de partida. ⚠️ **Maior que o diâmetro do colisor à volta** (`2 · √2 · LADO ≈ 0,311`):
/// peças que nascessem sobrepostas separavam-se no primeiro tique, e a cena mostraria um salto
/// antes da queda.
const GAP: f32 = 0.32;
/// De que altura elas partem — ⚠️ **DENTRO da taça**: um recipiente projecta para dentro tudo o que
/// nasce fora, e a peça de canto (a `(0,64; 1,49)` do centro da taça) tem de caber na parede menos
/// o raio dela (`1,8 − 0,156 = 1,644`, contra `1,62`).
const ALTURA: f32 = -0.35;

/// A taça: onde fica o fundo dela e o raio.
const TACA_Y: f32 = -1.2;
const TACA_R: f32 = 1.8;
/// Quanto as duas metades se afastam — o suficiente para as taças não se tocarem.
const VAO: f32 = 2.0;

/// A gravidade (`force.wind` apontado para baixo, sem rajada — o doc dele diz que assim ele
/// **é** gravidade).
const GRAVIDADE: f32 = 4.0;
/// ⭐⭐⭐ **Os SUB-PASSOS da zona** — a cura medida do zumbido da pilha. A tabela e o porquê do `8`
/// estão no sítio onde ele é escrito; o mecanismo inteiro no doc 111 §5.8.
const SUBSTEPS: f32 = 8.0;
const RAJADA: f32 = 0.0;

/// Quanto tempo dura cada queda, e a pausa antes da seguinte.
const DURACAO: f32 = 3.0;
const PAUSA: f32 = 0.6;

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![
        Caption::new(
            [-VAO, TACA_Y - TACA_R - 0.45],
            "Collide desligado: um borrao",
        ),
        Caption::new([VAO, TACA_Y - TACA_R - 0.45], "Collide ligado: uma pilha"),
    ]
}

/// Monta as duas taças. Devolve os dois sinks (esquerda, direita).
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};

    // ⚠️ Os índices de enum são PERGUNTADOS ao registo, nunca digitados — a porta da cena `=113`.
    let taca = super::sim_demo::indice_de(reg, "sim.collide", "shape", "Bowl")?;
    let quadrado = super::sim_demo::indice_de(reg, "source.shape", "kind", "Square")?;
    let em_laco = super::sim_demo::indice_de(reg, "sim.zone", "mode", "Loop")?;

    let mut metade = |x: f32, colide: bool, y_linha: f32| -> Option<NodeId> {
        let g = &mut doc.graph;
        let forma = g.add_node("source.shape");
        g.set_param(forma, param::KIND, quadrado);
        g.set_param(forma, param::SIZE, LADO);
        // ⭐ A pergunta inteira da cena — só nesta metade.
        if colide {
            g.set_param(forma, param::COLLIDE, 1.0);
        }

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", ROWS);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP);
        g.set_param(grid, "gap_y", GAP);

        let carimbo = g.add_node("motion.duplicator");

        let alto = g.add_node("motion.transform");
        g.set_param(alto, "offset_x", x);
        g.set_param(alto, "offset_y", ALTURA);

        let zone = g.add_node("sim.zone");
        // `Loop` para a queda recomeçar sozinha — sem isso o monte assenta uma vez e a cena
        // deixa de ter o que mostrar depois do primeiro olhar.
        // ⭐⭐⭐ **OS SUB-PASSOS — a cura do 4.º report do dono** (*«as shapes que ficam embaixo no
        // centro vibram muito»*, 2026-09-15). Doc 111 §5.8.
        //
        // ⚠️⚠️ **O defeito não estava no motor: estava neste número.** Oito cadeias de cura foram
        // construídas e refutadas (doc 109 §8.14) e mais três na obra encomendada (doc 111) antes de
        // alguém medir o knob que já existia. Partir o tique em `N` passos, cada um com a sua
        // integração E o seu contacto, é o que faz um solver assentar uma pilha **à rigidez plena**
        // — sem baixar o ganho, sem filtrar e sem amolecer, que são as três que caíram.
        //
        // ⚠️⚠️ **A TABELA FOI RE-MEDIDA em 2026-09-15, depois do ENCOSTO DE DOIS PONTOS** (doc 111
        // §5.11) — a primeira versão dela foi levantada com o contacto de UM ponto, e o manifesto
        // moveu a curva inteira. *Uma tabela medida sobre um substrato que mudou é uma mentira com
        // números* (`CLAUDE.md` §0.0: quem move o número tem de reconferir a nota).
        //
        // ⭐ **O `8` é o JOELHO da curva** (5 realizações por célula, as CINCO réguas):
        //
        // ⚠️⚠️ **RE-MEDIDA OUTRA VEZ em 2026-09-15**, depois do IMPULSO DO PAR (doc 111 §5.12) —
        // é a TERCEIRA versão desta tabela no mesmo dia, e as duas primeiras ficaram erradas
        // exactamente por o substrato ter mudado debaixo delas.
        //
        // ```text
        //   substeps | tremor (°/tique) |  rodopio   | altura |   vão  | salto | cozimento
        //          1 |   42,5 .. 48,0   | 15,5..63,4 | −2,61  | 0,2005 |   —   |  0,99 ms
        //          2 |    2,06.. 13,9   | 10,0..20,0 | −2,50  | 0,2081 |   —   |  2,07 ms
        //          4 |    1,01..  3,47  | 17,2..17,6 | −2,46  | 0,2186 |   —   |  3,51 ms
        //          8 |    0,246.. 0,327 |  2,6.. 2,8 | −2,43  | 0,2206 | 1,27° |  7,05 ms
        //         16 |    0,091.. 0,236 |  3,2.. 3,5 | −2,42  | 0,2202 |   —   | 14,05 ms
        // ```
        //
        // ⚠️ **O recurso é o QUADRO**, e é ele que escolhe o `8`: o `16` custa o dobro para comprar
        // um tremor que a `8` já tem `35×` abaixo da barra do gate. ⛔ Acima daqui o preço cresce
        // linear e o ganho não.
        //
        // ⛔⛔ **E o `4` DEIXOU de ser uma opção** — ele era viável com o contacto de um ponto
        // (`1,01..3,47` de tremor hoje, contra `0,150..0,217` antes). *Uma alternativa medida
        // sobre um substrato que mudou tem de ser re-medida antes de ser oferecida outra vez*, e
        // esta linha escreveu-a como viável de manhã. Com a física certa, a `8` é o PISO.
        //
        // ⚠️ **E as duas réguas de FORMA ficam intactas** — a pilha não congela (`y = −2,42`, longe
        // da altura de nascimento `−0,35`) nem colapsa: o vão `0,2203` é o face-a-face **exacto**
        // (`2 × LADO = 0,2200`), isto é, a pilha passou a encostar de CHAPA em vez de assentar em
        // quinas. As duas curas anteriores falharam exactamente aí.
        g.set_param(zone, "substeps", SUBSTEPS);
        g.set_param(zone, "mode", em_laco);
        g.set_param(zone, "duration", DURACAO);
        g.set_param(zone, "loop_delay", PAUSA);

        let vento = g.add_node("force.wind");
        g.set_param(vento, "angle", 270.0);
        g.set_param(vento, "strength", GRAVIDADE);
        g.set_param(vento, "gust", RAJADA);

        let passo = g.add_node("sim.step");

        let bowl = g.add_node("sim.collide");
        g.set_param(bowl, "shape", taca);
        g.set_param(bowl, "center_x", x);
        g.set_param(bowl, "center_y", TACA_Y);
        g.set_param(bowl, "radius", TACA_R);
        g.set_param(bowl, "restitution", 0.05);
        g.set_param(bowl, "friction", 0.6);

        let out = g.add_node("motion.output");

        let fila = [forma, grid, carimbo, alto, zone];
        for (i, n) in fila.into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                    x: 40.0 + i as f32 * 176.0,
                    y: y_linha,
                },
            );
        }
        for (i, n) in [vento, passo, bowl, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                    x: 260.0 + i as f32 * 176.0,
                    y: y_linha + 250.0,
                },
            );
        }

        // ⚠️ A aresta `zone -> vento` é `delayed`: é a entrada de estado que fecha o laço.
        // ⚠️ E NENHUM `motion.collide` entre o passo e a taça: quem separa é o `sim.step`, que lê o
        // colisor que a forma declarou.
        for (a, ap, b, bp, delayed) in [
            (forma, 0, carimbo, 0, false),
            (grid, 0, carimbo, 1, false),
            (carimbo, 0, alto, 0, false),
            (alto, 0, zone, 0, false),
            (zone, 0, vento, 0, true),
            (vento, 0, passo, 0, false),
            (passo, 0, bowl, 0, false),
            (bowl, 0, zone, 1, false),
            (zone, 0, out, 0, false),
        ] {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed,
            })
            .ok()?;
        }
        g.set_label(forma, if colide { "Shape (Collide)" } else { "Shape" });
        Some(out)
    };

    let esquerda = metade(-VAO, false, 120.0)?;
    let direita = metade(VAO, true, 640.0)?;
    doc.graph.validate(reg).ok()?;
    Some(vec![esquerda, direita])
}

#[cfg(test)]
#[path = "motion_state_pilha_demo_tests.rs"]
mod tests;

/// ⭐ O irmão que mede o MOVIMENTO da pilha (doc 109 §8) — ver o cabeçalho dele.
#[cfg(test)]
#[path = "motion_state_pilha_demo_tremor.rs"]
mod tremor;

/// ⭐ E o irmão que mede AS CURAS (doc 109 §8.7–§8.14) — ver o cabeçalho dele.
#[cfg(test)]
#[path = "motion_state_pilha_demo_curas.rs"]
mod curas;

/// ⭐ E o que mede os factos que DESENHAM a obra encomendada (doc 111) — ver o cabeçalho dele.
#[cfg(test)]
#[path = "motion_state_pilha_demo_obra.rs"]
mod obra;

/// ⭐ E o que mede O SALTO depois de assentar (5.º report do dono) — ver o cabeçalho dele.
///
/// ⚠️ Irmão próprio e não mais uma secção do [`tremor`]: um salto é um **EXTREMO num intervalo
/// curto**, e aquele ficheiro mede medianas e somas — *as duas grandezas não partilham uma régua*.
#[cfg(test)]
#[path = "motion_state_pilha_demo_salto.rs"]
mod salto;

/// ⭐ E as sondas que nomearam a CAUSA do salto — ver o cabeçalho delas.
#[cfg(test)]
#[path = "motion_state_pilha_demo_salto_diag.rs"]
mod salto_diag;
