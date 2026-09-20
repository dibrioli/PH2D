//! ⭐⭐⭐ **O OSSO NUMA CADEIA** (`PH2D_GPU_COOK_DEMO=125`) — ordem do dono, 2026-09-19:
//! *«Shape:Bone em Skeleton:Duplicator ficou 180 graus rodado. corrija e crie uma simulação com
//! ele»*.
//!
//! ## O que ela mostra, e por que são TRÊS colunas
//!
//! ```text
//!   ESQUERDA   a cadeia CURVA, vestida de ossos      — onde a ORIENTAÇÃO se lê
//!   MEIO       uma cadeia recta a ONDULAR            — a SIMULAÇÃO
//!   DIREITA    a mesma curva, SEM forma              — o CONTROLO: só as posições
//! ```
//!
//! ⚠️⚠️ **A coluna do MEIO é a que decide, e a da esquerda sozinha não chegava:** numa cadeia
//! recta todos os ossos apontam para o mesmo lado, logo *«está 180° rodado»* e *«está certo»*
//! produzem a MESMA imagem espelhada — e um espelho de uma coluna vertical de formas simétricas
//! em Y é indistinguível dela própria. É na curva que a ponta de cada osso tem de apontar para a
//! junta SEGUINTE, e é isso que se vê.
//!
//! ⭐ **E a da direita é o controlo que separa duas perguntas:** *«as posições estão certas?»* e
//! *«a forma aponta para onde devia?»*. Elas são independentes — a pose já estava certa quando o
//! report chegou —, e sem as cruzes ao lado um defeito de orientação lê-se como um defeito de
//! cadeia.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Quantas juntas cada cadeia tem. ⚠️ Doze e não oito (o valor de fábrica): com oito a curva da
/// coluna do meio não fecha o suficiente para a orientação ser óbvia a olho.
const JUNTAS: f32 = 12.0;

/// O comprimento de cada osso, em unidades de mundo.
///
/// ⚠️ **`0,6` e não `0,9`, e o recurso é o ENQUADRAMENTO:** as três colunas somam
/// `2 × (COLUNA + raio)` de largura, e a `0,9` isso dá `±17,4` contra os `±12,8` que a câmera de
/// arranque mostra — *a cena abria com duas das três colunas fora do ecrã, e o dono tinha de
/// afastar a vista antes de ver o que ela existe para mostrar.*
const OSSO: f32 = 0.6;

// ⚠️⚠️ **O `±12,8` da nota acima é a metade LONGA do eixo, e isso foi medido depois** (19/09, na
// foto da cena irmã `=124`): a origem do mundo **não é o centro do canvas** — ela fica a `720 px`
// da borda esquerda e a `608` da direita —, logo uma cena centrada na origem só pode contar com
// `±10,9`. ⇒ **esta cena cabe por `0,17` unidades, e só porque a coluna curva se estende para o
// lado LONGO**; a foto de 19/09 prova que ela cabe hoje, e quem lhe mexer nos números tem de
// re-medir contra `pontos_demo::VISTA_MEIA_LARGURA`, que é o valor honesto.

/// Quanto cada junta vira em relação à anterior, na coluna CURVA.
///
/// ⚠️ **`14°` é derivado e não escolhido:** `JUNTAS × 14° ≈ 168°`, que é uma cadeia a fechar
/// quase meia-volta — o suficiente para a ponta de cada osso apontar visivelmente para a junta
/// seguinte, e pouco para ela não se cruzar consigo mesma.
const CURVA: f32 = 14.0;

/// O tamanho do osso — **METADE do vão, porque o `size` é o SEMI-eixo**.
///
/// ⛔⛔ **A primeira redacção usou o vão inteiro e a foto mostrou porquê:** a receita do
/// `source.shape` corta toda forma de uma caixa de LARGURA `2 × size`, logo o comprimento
/// desenhado é `2 × size`. Com `size = OSSO` cada osso media o DOBRO do vão entre duas juntas, e a
/// cadeia saía como uma massa branca contínua em que não se distinguia peça nenhuma — *e muito
/// menos para que lado cada uma aponta*, que é o que a cena existe para mostrar.
///
/// ⭐ **E desde o 2.º report do dono (19/09) o osso pendura-se na CABEÇA** (a caixa dele é
/// `[0, 2s]` e não `[−s, s]`), logo com este número cada peça vai **exactamente** da junta em que
/// está à seguinte — a cadeia ladrilha, em vez de cada osso montar metade do vizinho.
const TAMANHO: f32 = OSSO / 2.0;

/// ⭐⭐ **A ESBELTEZA do osso, e ela é DERIVADA da própria silhueta.**
///
/// O `aspect` multiplica o semi-eixo `y` da caixa, logo a altura do osso é `2 × aspect × size` e
/// o comprimento é `2 × size` ⇒ **`1/3` é, à letra, «três vezes mais comprido do que largo»**.
///
/// ⛔⛔ **Sem ele a foto mostra BLOCOS e não ossos:** o valor de fábrica do `aspect` é `1` — o que
/// serve um carimbo qualquer e faz desta forma um quadrado —, e a peça sai tão alta quanto longa.
/// ⚠️ *A alavanca não é nova: o cabeçalho do [`ph2d_vec_scene::symbols_rig`] já declara que o
/// `OMBRO` não é knob **porque o `aspect` é a alavanca** que muda o que o olho de facto lê.*
const ESBELTEZA: f32 = 1.0 / 3.0;

/// ⭐⭐⭐ **A ONDA da coluna do meio** — a *«simulação com ele»* que o dono pediu.
///
/// ⛔⛔ **A amplitude é DERIVADA do número de juntas, e a foto mostrou porquê.** A primeira
/// redacção somou `25°` de onda aos `14°` da curva: `39° × JUNTAS ≈ 468°`, e a cadeia **enrolou-se
/// num novelo** no centro do ecrã — *uma simulação que se cruza consigo mesma não mostra o
/// movimento que ela existe para mostrar*.
///
/// O tecto é um quarto de volta na cadeia inteira (`JUNTAS × A ≤ 90°`), e a base desta coluna passou a
/// ser RECTA para a onda ser a ÚNICA curvatura que ela tem.
const ONDA_GRAUS: f32 = 90.0 / JUNTAS;
/// Meio hertz — uma ida e volta a cada dois segundos, que é o ritmo em que o olho segue cada osso.
const ONDA_HZ: f32 = 0.5;
/// ⚠️ **O desfasamento por elemento é o que faz a onda PERCORRER a cadeia** em vez de todas as
/// juntas dobrarem juntas: sem ele a cadeia abre e fecha como um leque.
///
/// ⛔⛔ **E ele também é DERIVADO, pela mesma foto:** com `0,35` a cadeia cobria `0,35 × JUNTAS ≈
/// 4,2` voltas de fase, logo cada junta estava num ponto arbitrário do ciclo e o que saía era um
/// NOVELO, não uma onda. `1 / JUNTAS` é exactamente UMA volta ao longo da cadeia inteira — uma
/// crista e um vale, que é a forma que o olho lê como onda.
const ONDA_ATRASO: f32 = 1.0 / JUNTAS;

/// A distância entre as três colunas, **DERIVADA do raio que a cadeia curva descreve**.
///
/// ⛔ **Um número escolhido aqui sobrepõe as colunas e ninguém dá por isso** — foi o que aconteceu
/// à primeira redacção (`7,0`): a curva estende-se até `−7,16` e a coluna da esquerda acaba em
/// `−7,00`, logo as duas encostavam-se. *Uma cena em que duas colunas se tocam não consegue
/// mostrar a comparação que ela existe para mostrar*, e quem a apanhou foi o gate
/// `as_tres_colunas_ficam_separadas`.
///
/// Uma cadeia que vira `CURVA` graus por osso de comprimento `OSSO` descreve um arco de raio
/// `OSSO / CURVA_rad` — aqui `≈ 3.68` —, logo ela ocupa `2 ×` isso em largura. A folga de um
/// osso é o que impede duas colunas de se tocarem quando alguém mexer nos números.
///
/// ⚠️⚠️ **E ela tem de servir as DUAS formas de coluna, que ocupam espaço por razões
/// diferentes:** a curva ocupa `2 × raio` em largura, e a que ONDULA nasce horizontal, logo
/// ocupa **meio comprimento da cadeia** para cada lado do seu centro. A régua que reprovou foi a
/// `as_tres_colunas_ficam_separadas`, com a horizontal a invadir a vizinha.
const CURVA_RAD: f32 = CURVA * core::f32::consts::PI / 180.0;
const RAIO_DA_CURVA: f32 = OSSO / CURVA_RAD;
/// ⚠️ **O raio entra a DOBRAR:** uma cadeia que vira `CURVA` graus por osso ao longo de
/// `JUNTAS` fecha quase meia-volta, logo ela estende-se para trás do ponto onde começa — medido,
/// um raio de folga não chegava (`o meio acaba em 4,5 e a direita começa em 2,83`).
const COLUNA: f32 = JUNTAS * OSSO / 2.0 + 2.0 * RAIO_DA_CURVA + OSSO;

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let osso = super::sim_demo::indice_de(reg, "source.shape", "kind", "Bone")?;
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };

    // Uma cadeia, com o ângulo por junta e a coluna em que ela cai.
    let cadeia = |g: &mut ph2d_nodegraph::graph::Graph, angulo: f32, raiz: f32, dx: f32, y: f32| {
        let s = g.add_node("rig.skeleton".to_string());
        g.set_pos(s, Pos { x: 0.0, y });
        g.set_param(s, "joints", JUNTAS);
        g.set_param(s, "length", OSSO);
        g.set_param(s, "angle", angulo);
        // ⛔⛔⛔ **A coluna que ONDULA nasce a apontar para `0°`, e a razão é MEDIDA.** O
        // `motion.oscillator` SOMA ao canal, e o canal `rot` é hoje o ângulo de MUNDO — logo um
        // `rig.fk` a jusante relê `90 + osc` como pose LOCAL e acumula-o junta a junta:
        // `[90, 183, 280, 377…]`, mais de uma volta, e a cadeia sai **enrolada num novelo** (a
        // extensão vertical medida cai de `9,90` para `1,23`). Com a raiz a `0°` o que o `fk`
        // acumula é só a onda, e a cadeia ondula esticada.
        g.set_param(s, "root_angle", raiz);
        let m = g.add_node("motion.move".to_string());
        g.set_pos(m, Pos { x: 200.0, y });
        g.set_param(m, "dx", dx);
        g.connect(Edge {
            from: (s, 0),
            to: (m, 0),
            delayed: false,
        })
        .ok()?;
        Some(m)
    };

    let mut sinks = Vec::new();

    // ── ESQUERDA e MEIO: vestidas de ossos.
    for (i, (angulo, raiz, dx)) in [(CURVA, 90.0, -COLUNA), (0.0, 0.0, -JUNTAS * OSSO / 2.0)]
        .into_iter()
        .enumerate()
    {
        let y = 260.0 * i as f32;
        let mut corpo = cadeia(g, angulo, raiz, dx, y)?;
        // ⭐⭐⭐ **A do MEIO ONDULA — e ela é a demonstração da lei que esta jornada curou.**
        //
        // O oscilador escreve o canal `Rotation`, que é a coluna `rot`; o `rig.fk` a seguir lê-a
        // como pose LOCAL e **re-resolve a cadeia**, logo as POSIÇÕES mexem-se e não apenas o
        // ângulo das peças. ⚠️ Isto só funciona pelo degrau 2 da escada do [`fk::local`]: uma
        // corrente que já saiu de um `resolve` traz `lrot`, e sem esse degrau o `rot` reescrito
        // pelo oscilador seria **deitado fora em silêncio** — a cadeia ficava parada com os
        // números todos certos.
        if i == 1 {
            let osc = no(g, "motion.oscillator", 240.0, y + 60.0);
            g.set_param(osc, "channel", 2.0); // `Rotation`
            g.set_param(osc, "amplitude", ONDA_GRAUS);
            g.set_param(osc, "frequency", ONDA_HZ);
            g.set_param(osc, "phase_stagger", ONDA_ATRASO);
            let fk = no(g, "rig.fk", 310.0, y);
            for (de, para) in [(corpo, osc), (osc, fk)] {
                g.connect(Edge {
                    from: (de, 0),
                    to: (para, 0),
                    delayed: false,
                })
                .ok()?;
            }
            corpo = fk;
        }
        let forma = no(g, "source.shape", 0.0, y + 120.0);
        g.set_param(forma, ph2d_node_motion_shape::param::KIND, osso);
        g.set_param(forma, ph2d_node_motion_shape::param::SIZE, TAMANHO);
        g.set_param(forma, ph2d_node_motion_shape::param::ASPECT, ESBELTEZA);
        let dup = no(g, "motion.duplicator", 380.0, y);
        // ⚠️ A forma na porta `0`, os pontos na `1` — a ordem do manifesto do duplicador.
        for (de, porta) in [(forma, 0u16), (corpo, 1)] {
            g.connect(Edge {
                from: (de, 0),
                to: (dup, porta),
                delayed: false,
            })
            .ok()?;
        }
        let saida = no(g, "motion.output", 560.0, y);
        g.connect(Edge {
            from: (dup, 0),
            to: (saida, 0),
            delayed: false,
        })
        .ok()?;
        sinks.push(saida);
    }

    // ── DIREITA: o CONTROLO — a mesma curva, sem forma nenhuma.
    let nu = cadeia(g, CURVA, 90.0, COLUNA, 520.0)?;
    let saida_c = no(g, "motion.output", 560.0, 520.0);
    g.connect(Edge {
        from: (nu, 0),
        to: (saida_c, 0),
        delayed: false,
    })
    .ok()?;
    sinks.push(saida_c);

    Some(sinks)
}

/// O roteiro que o dono segue. ⚠️ **Cada passo nomeia o que aparece NA TELA** (§0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[osso] TRES cadeias de {JUNTAS:.0} juntas, lado a lado. Carregue em PLAY.\n\
         \n\
         ESQUERDA = uma cadeia CURVA, vestida de OSSOS.\n\
         MEIO     = uma cadeia recta a ONDULAR, vestida de ossos.\n\
         DIREITA  = a mesma curva, SEM forma: so' as posicoes (cruzinhas).\n\
         \n\
         (1) Olhe a do MEIO, que fica no centro do ecra: cada osso e' GROSSO na junta em que\n    \
         esta' pendurado e AFIA para a junta seguinte — como um osso de esqueleto, grosso no\n    \
         ombro e fino no cotovelo. A ponta grossa marca o ponto em que ele GIRA.\n\
         (2) Carregue em PLAY: a onda PERCORRE essa cadeia, da raiz para a ponta, e cada\n    \
         osso vira com ela. As POSICOES mexem-se, nao so' as pecas.\n\
         (3) Pause. Clique no cartao `Shape` dessa coluna e arraste `Pivot Offset X`. Enquanto\n    \
         arrasta, um ALVO (um anel com uma cruz) aparece sobre cada osso, no ponto em\n    \
         que ele gira — e o numero conta-se em TAMANHOS da forma:\n    \
         `0` = a CABECA (o valor de fabrica desta forma);\n    \
         `-1` = o MEIO, como qualquer carimbo;\n    \
         `+1` = a PONTA fina. `2` poe o ponto uma forma inteira para fora.\n    \
         Com o PLAY ligado a diferenca ve-se de uma vez. Ha' tambem `Pivot Offset Y`.\n\
         (4) Afaste a vista e olhe a da ESQUERDA (a cadeia enrolada) e a da DIREITA (as\n    \
         cruzinhas, que sao as mesmas posicoes sem forma nenhuma).\n\
         \n\
         DEU ERRADO se: a parte GROSSA de um osso ficar do lado para onde a cadeia VAI; se\n    \
         com o PLAY a do meio ficar PARADA; se ela rodar as pecas sem mexer as posicoes; ou\n    \
         se arrastar o `Pivot X` nao mudar o ponto em torno do qual cada osso roda.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_osso_demo_tests.rs"]
mod tests;
