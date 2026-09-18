//! ⭐⭐⭐ **A CENA DO PASSE EM MOVIMENTO** (doc 115 W6, cena `=122`) — ordem do dono (2026-09-17):
//! *«resolva o que está em aberto»* + *«crie uma cena de simulação para smoke»*.
//!
//! Duas fileiras de PARES de quadrados que tremem sem parar. Na de cima, com o interruptor ligado,
//! eles nunca se atravessam — quadro após quadro. Na de baixo atravessam-se com o interruptor
//! igualmente ligado, porque um campo lhes disse que eles não colidem.
//!
//! # ⛔⛔⛔ Porque ela NÃO é uma simulação de física, e a medição é que o decidiu
//!
//! O pedido do dono foi *«uma cena de simulação»*. **Três famílias foram construídas e MEDIDAS, e
//! as três foram refutadas** (doc 115 §15.3) — nenhuma delas pode honestamente ensinar o que esta
//! wave entrega:
//!
//! | família | o que a medição disse |
//! |---|---|
//! | `sim.zone` + `sim.step` | o passe é **redundante**: o `sim.step` é um dos três leitores do colisor declarado e já separa DENTRO do tique |
//! | `motion.integrate` + atractor | o passe **não aguenta**: o atractor esmaga `25` peças até `0,007` de largura e ficam `41`–`80` pares atravessados **a 32 varreduras** |
//! | `motion.verlet_rope` | uma corda é uma **CADEIA**: `15` pares atravessados ficam em `10` a 8 varreduras e ainda em **`3` a 64**, que é o topo do knob |
//!
//! ⭐⭐⭐ **A lei que as três deram junta-se numa frase:** *o passe automático é um ACABAMENTO para
//! ARRANJOS com sobreposições LOCAIS e INDEPENDENTES — não é uma lei de contacto.* Sem
//! realimentação ele não segura um solver que empurre as peças umas para dentro das outras, e a
//! `~n²` da lei de Jacobi (doc 115 §13.6) põe uma CADEIA fora do alcance de qualquer número que o
//! artista consiga escrever.
//!
//! ⇒ é por isso que esta cena tem a topologia da `=121` — **PARES independentes** — e não um monte.
//! Ali `8` varreduras limpam tudo, e a promessa da cena é verdadeira em todo quadro.
//!
//! # ⚠️ Ela nasce DESARMADA, como a `=121`
//!
//! O primeiro quadro tem de mostrar os pares a atravessarem-se, senão o artista vê o resultado e
//! nunca a causa.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use ph2d_motion_doc::MotionDoc;

/// Quantas peças cada fileira carimba — oito PARES.
pub(super) const PECAS: usize = 2 * PARES;
const PARES: usize = 8;

/// O `Size` autorado no cartão da forma. ⚠️ **É a MEIA extensão de mundo** (a geometria de uma
/// `source.shape` vive em raio `1`) — a armadilha que a §12.1 e a `=121` já pagaram.
const MEIA: f32 = 0.09;
/// O lado de uma peça, em mundo.
const LADO: f32 = 2.0 * MEIA;

/// O vão ENTRE pares. ⚠️ **Maior que o lado MAIS o tremor**, senão um par empurrava o vizinho e a
/// fileira deixava de ser feita de problemas independentes — que é exactamente a propriedade de
/// que esta cena depende (ver o cabeçalho).
const VAO_ENTRE_PARES: f32 = 1.6 * LADO;
/// O passo DENTRO de um par: as duas peças entram uma na outra em `30 %` do lado.
const SOBREPOSICAO_DO_PAR: f32 = 0.70 * LADO;

/// Quanto cada peça treme, e com que rapidez. ⚠️ **A amplitude é uma fracção do LADO** e não um
/// número solto: é o que impede a próxima pessoa a mexer no tamanho de partir a independência dos
/// pares sem reparar.
const TREMOR: f32 = 0.22 * LADO;
const RITMO: f32 = 0.9;

/// Onde cada fileira mora.
const ALTURA_DA_FILEIRA: f32 = 0.85;

/// ⭐ Onde mora o campo da fileira de baixo. ⚠️ **Longe de tudo, de propósito, e MEDIDO:** o
/// `motion.falloff` dá `1` dentro do raio e **`0` exacto** fora dele, logo um campo que peça
/// nenhuma alcança põe a fileira inteira a zero **ao bit**. ⛔ A 1.ª redacção usou `invert` sobre um
/// raio grande e leu `0,68 · 0,51 · 0,16 …` — *o `invert` dá uma RAMPA, não um interruptor*, e a
/// fileira ficava parcialmente separada, que é o pior dos dois mundos.
const CAMPO_LONGE: f32 = 40.0;
const CAMPO_RAIO: f32 = 1.0;

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let quadrado = super::sim_demo::indice_de(reg, "source.shape", "kind", "Square")?;
    let pos_xy = super::sim_demo::indice_de(reg, "motion.wiggle", "channel", "Position XY")?;

    let mut fileira = |y: f32, protegida: bool, y_linha: f32, semente: f32| -> Option<NodeId> {
        let g = &mut doc.graph;
        let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str| g.add_node(tipo.to_string());

        // A FORMA, com o botão que faz a peça DECLARAR a caixa dela (doc 115 W4).
        let forma = no(g, "source.shape");
        g.set_param(forma, ph2d_node_motion_shape::param::KIND, quadrado);
        g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
        g.set_param(forma, ph2d_node_motion_shape::param::SIZE, MEIA);

        // Os PONTOS: duas linhas que se atravessam, `PARES` colunas bem afastadas ⇒ pares.
        let grelha = no(g, "motion.grid");
        g.set_param(grelha, "rows", 2.0);
        g.set_param(grelha, "cols", PARES as f32);
        g.set_param(grelha, "gap_x", VAO_ENTRE_PARES);
        g.set_param(grelha, "gap_y", SOBREPOSICAO_DO_PAR);

        let dup = no(g, "motion.duplicator");

        // ⭐ O MOVIMENTO — e é ele que faz desta cena outra coisa que a `=121`: o passe deixa de ser
        // uma conta feita uma vez e passa a correr em TODO quadro, sobre peças que nunca param.
        let tremor = no(g, "motion.wiggle");
        g.set_param(tremor, "channel", pos_xy);
        g.set_param(tremor, "amplitude", TREMOR);
        g.set_param(tremor, "frequency", RITMO);
        g.set_param(tremor, "seed", semente);

        let mover = no(g, "motion.transform");
        g.set_param(mover, "offset_y", y);
        let saida = no(g, "motion.output");

        // ⭐ O knob da cena, e o único.
        let campo = protegida.then(|| {
            let c = no(g, "motion.falloff");
            g.set_param(c, "center_x", CAMPO_LONGE);
            g.set_param(c, "center_y", CAMPO_LONGE);
            g.set_param(c, "radius", CAMPO_RAIO);
            g.set_label(c, "Falloff (esta fileira NAO colide)");
            c
        });

        for (i, n) in [forma, grelha, dup, tremor, mover].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                    x: 40.0 + i as f32 * 180.0,
                    y: y_linha,
                },
            );
        }
        g.set_pos(
            saida,
            Pos {
                x: 1140.0,
                y: y_linha,
            },
        );

        let mut arestas = vec![
            (forma, 0u16, dup, 0u16),
            (grelha, 0, dup, 1),
            (dup, 0, tremor, 0),
            (tremor, 0, mover, 0),
        ];
        match campo {
            Some(c) => {
                g.set_pos(
                    c,
                    Pos {
                        x: 950.0,
                        y: y_linha,
                    },
                );
                arestas.push((mover, 0, c, 0));
                arestas.push((c, 0, saida, 0));
            }
            None => arestas.push((mover, 0, saida, 0)),
        }
        for (a, ap, b, bp) in arestas {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed: false,
            })
            .ok()?;
        }
        g.set_label(forma, "Shape");
        Some(saida)
    };

    // ⚠️ Sementes DIFERENTES: com a mesma, as duas fileiras tremeriam em uníssono e o olho leria
    // uma cópia em vez de duas amostras da mesma lei.
    let cima = fileira(ALTURA_DA_FILEIRA, false, 80.0, 1.0)?;
    let baixo = fileira(-ALTURA_DA_FILEIRA, true, 620.0, 7.0)?;
    doc.graph.validate(reg).ok()?;
    // ⚠️⚠️ **O interruptor NÃO se escreve em NENHUM dos dois sinks** — a cena nasce desarmada
    // porque não autora override nenhum, logo não pode divergir do valor de fábrica no dia em que
    // ele mudar. Quem liga é o artista.
    Some(vec![cima, baixo])
}

/// O roteiro que o dono segue. ⚠️ Cada passo nomeia o que aparece NA TELA (§0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[passe-mov] DUAS FILEIRAS de {PARES} PARES ({PECAS} quadrados cada) que TREMEM sem\n\
         parar — e NENHUM no' de colisao em lado nenhum.\n\
         \n\
         (1) Carregue em PLAY. Sem isto nada treme. Repare: em cada par, um quadrado esta'\n    \
         metido dentro do outro, e eles continuam metidos enquanto tremem.\n\
         (2) Carregue no cartao `Output` de CIMA (o ultimo no' da fileira de cima).\n\
         (3) Ligue `Collide`. A fileira DE CIMA separa-se e fica separada — quadro apos quadro,\n    \
         com as pecas a tremer o tempo todo. A de baixo continua metida uma na outra.\n\
         (4) Ligue `Collide` tambem no `Output` de BAIXO. A fileira de baixo NAO muda — e e'\n    \
         esse o assunto do passo seguinte.\n\
         (5) Na fileira de baixo, clique no cartao `Falloff (esta fileira NAO colide)` e ponha\n    \
         `Center X` e `Center Y` a zero. Agora ela tambem se separa.\n    \
         (i) E' assim que se diz a um objecto que ele NAO colide: nem a forma nem o sink\n        \
         mudaram — mudou um campo, que e' a mesma coisa que um Sprite ou um vector podem ter.\n\
         \n\
         (!) Se ligar `Collide` e a fileira de cima nao se separar, o defeito e' a forma nao\n    \
         estar a DECLARAR a caixa dela — abra o cartao `Shape` e confirme que o `Collide` DELE\n    \
         esta' ligado.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_passe_sim_demo_tests.rs"]
mod tests;
