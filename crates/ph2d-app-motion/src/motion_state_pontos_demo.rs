//! ⭐⭐⭐ **A CENA DAS POSIÇÕES E DA MARCA** (cena `=124`) — a resposta ao report do dono de
//! 2026-09-19: *«Grid está por padrão com 360x360 objetos. deveria ser 20x20»* e *«coloque gizmos
//! de pequenos pontos visíveis para as posições dos nós»*.
//!
//! # Porque ela existe, e porque a `=2` não servia
//!
//! ⛔⛔ **Eu mandei-lhe a cena errada.** A `=2` é o demo de PERFORMANCE do dispositivo — uma
//! `motion.grid` de `360 × 360 = 129 600` elementos, escolhida para provar que o device aguenta —,
//! e eu chamei-lhe *«o cartão do Grid»* num report. Ele abriu-a, contou os objectos e leu o número
//! como sendo o padrão do nó. ⚠️ **O padrão do nó é `3 × 3`**, e o `360` é daquela cena e só dela.
//!
//! ⇒ *baixar a `=2` para `20 × 20` apagaria a razão de ela existir.* Esta cena é o que o report
//! pedia: um grid pequeno, sozinho, onde as posições e o cartão se vêem.
//!
//! # As DUAS metades, lado a lado
//!
//! | metade | o que tem | o que se vê |
//! |---|---|---|
//! | esquerda | `motion.grid` → `motion.output` | **marcas** — cruzes, uma por posição, e mais nada |
//! | direita | o MESMO grid → `motion.duplicator` ← `source.shape` (**Bone**) | as **peças** |
//!
//! ⚠️⚠️ **A metade da direita é o CONTROLO, e sem ela a outra não ensina nada:** *«não desenha»* e
//! *«está partido»* têm exactamente o mesmo aspecto no ecrã, e o que os separa é ver a mesma nuvem
//! de posições a virar coisas assim que uma forma chega.
//!
//! ⛔⛔ **E elas ficam LADO A LADO porque empilhadas não cabiam** — ver [`VISTA_MEIA_ALTURA`]. A
//! foto da cena é que o disse: *um controlo que o dono não vê sem procurar não é um controlo.*

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use ph2d_motion_doc::MotionDoc;

/// **A PEGADA de uma marca, em unidades de MUNDO.**
///
/// A marca é derivada da pegada do elemento (`ponto_gizmo_overlay::glifo_px`), e uma corrente sem
/// coluna `size` — que é o que uma grelha nua é — cai na IDENTIDADE dela. ⇒ *o tamanho da cruz não
/// é um número desta cena: é aquele.*
const MARCA: f32 = ph2d_nodegraph::attr::SIZE_IDENTITY[0];

/// O vão entre posições: a pegada de uma marca mais `20 %` de ar.
///
/// ⛔⛔ **Abaixo da pegada as cruzes ENCOSTAM e a nuvem lê-se como uma GRADE** — não como
/// posições. A foto de 19/09 mostrou-o com todas as letras: a `0,4` de vão elas fundiam-se numa
/// treliça azul contínua, e o passo (1) do roteiro (*«são cruzinhas, uma por posição»*) ensinava
/// o contrário do que estava na tela.
const VAO: f32 = 1.2 * MARCA;

/// ⭐⭐⭐ **QUANTAS POSIÇÕES CABEM numa extensão — a régua que decide o tamanho desta cena.**
///
/// Duas leis puxam em sentidos opostos e **o vão não pode ceder** (ver acima): uma fila de `n`
/// posições mede `(n − 1) × VAO` mais uma pegada de marca a transbordar, e o todo tem de caber na
/// extensão pedida. Com `10 %` de margem isso dá `n ≤ 1 + (0,9 × extensão − MARCA) / VAO`.
///
/// ⛔⛔⛔ **A 1.ª redacção cravou `20 × 20` aqui, «o número que o dono pediu», e as duas frases
/// dele eram sobre OUTRA coisa:** ele viu `360 × 360` na cena `=2` — o demo de PERFORMANCE — e
/// leu-a como o valor de fábrica do nó (que é `3 × 3`). *Nenhuma das duas frases era sobre esta
/// cena*, e cravar aqui um número que não cabe entrega uma grelha que o dono não consegue ler.
///
/// ⚠️ E ele continua **muito** abaixo do tecto do gizmo (`ponto_gizmo::MAX_PONTOS`, `4 096`), logo
/// a amostra não engata e o que se vê é a grelha inteira, posição a posição.
///
/// ⚠️ **O `as u32` não é cosmético:** o lado de uma grelha é uma CONTAGEM, e o nó arredonda-o em
/// silêncio — um gate que multiplicasse o `f32` cru esperaria uma população que a cena não monta.
/// *Quem trunca é quem escreve o número, não quem o lê.*
///
/// ⛔⛔ **E a grelha é RECTANGULAR de propósito: a vista é `23,6 × 6,8` e uma grelha QUADRADA é
/// governada pelo lado curto**, o que deitaria fora dois terços da largura — cinco marcas por
/// metade e o ecrã vazio dos dois lados.
const fn quantas(extensao: f32) -> f32 {
    // O `− MARCA` está lá porque a marca da ponta transborda a nuvem por meia pegada de cada lado.
    (1.0 + (0.9 * extensao - MARCA) / VAO) as u32 as f32
}

/// Quantas posições ao longo da largura de UMA metade.
pub(super) const COLUNAS: f32 = quantas(VISTA_MEIA_LARGURA);
/// Quantas ao longo da altura.
pub(super) const LINHAS: f32 = quantas(2.0 * VISTA_MEIA_ALTURA);

/// ⚠️⚠️ **O PISO da derivação, e ele é ERRO DE COMPILAÇÃO de propósito.** A grelha sai de uma
/// divisão pelo vão: quem subisse o vão (ou apertasse a vista) levava-a a `2 × 2` **sem uma linha
/// vermelha**, e uma cena com quatro marcas não mostra a NUVEM de posições que é o assunto dela.
///
/// ⛔ Ele **não** pode viver num teste: um `assert!` sobre duas constantes é dobrado pelo
/// compilador antes de correr — o clippy recusa-o em voz alta, e o `ponto_gizmo_overlay` já
/// escreve a mesma lei por extenso. *O que morde é esta linha.*
const _: () = assert!(
    COLUNAS >= 5.0 && LINHAS >= 4.0,
    "a grelha derivou para menos do que se le' como uma nuvem de posicoes"
);

/// **O que a câmera de arranque MOSTRA**, em unidades de mundo, medido na foto da cena.
///
/// ⛔⛔⛔ **A 1.ª redacção desta cena punha as duas metades EMPILHADAS com o vão a `1,6`, e a foto
/// mostrou que ela era impossível:** um bloco de `20 × 20` a esse vão mede `30,4` unidades de
/// lado, a de baixo descia mais `38,4`, e **o que a câmera de arranque mostra são `23,6 × 8,6`**
/// ⇒ *o dono abria a cena e via CRUZES e mais nada* — a metade de baixo, que é o CONTROLO sem o
/// qual a de cima não ensina nada, ficava três ecrãs abaixo.
///
/// ⚠️ **O número saiu da régua do próprio app** (a barra do topo lê `-1200 .. 1000`, com `100`
/// dela por unidade de mundo), e a razão vertical é a do canvas — *não é um palpite de
/// enquadramento, é o rectângulo em que a cena tem de caber.*
/// ⚠️⚠️ **A ORIGEM DO MUNDO NÃO É O CENTRO DO CANVAS, nos DOIS eixos** (medido na foto, a
/// `55,5 px` por unidade): o zero fica a `720 px` da borda esquerda e a `608` da direita; a `190`
/// do topo e a `287` da base — a faixa do grafo e a timeline comem o resto. ⇒ **uma cena centrada
/// na origem só pode contar com a metade CURTA de cada eixo**, e foi por contar com a média que a
/// 1.ª redacção cortou a fileira de cima e a 2.ª cortou a coluna da direita.
pub(super) const VISTA_MEIA_LARGURA: f32 = 10.9;
pub(super) const VISTA_MEIA_ALTURA: f32 = 3.4;

/// A largura de um bloco, em unidades de mundo.
const BLOCO: f32 = (COLUNAS - 1.0) * VAO;

/// **Quanto a metade com forma se afasta da outra** — um bloco mais dois vãos de intervalo.
///
/// ⚠️ Cada metade desloca-se METADE disto, em sentidos opostos, para o PAR ficar centrado: mover
/// só uma delas empurraria a cena inteira para um lado da vista.
const AFASTAMENTO: f32 = BLOCO + 2.0 * VAO;

/// O tamanho da peça na metade com forma — **METADE do vão, porque o osso se pendura na CABEÇA**
/// e mede `2 × size`: assim cada peça vai exactamente de uma posição à seguinte.
pub(super) const TAMANHO: f32 = VAO / 2.0;

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    // ⭐ **O OSSO e não um círculo** — ordem do dono (2026-09-19): *«no caso dos ossos e
    // segmentos de corda, criaremos no nó shape formas similares para isso»*. A fileira de baixo
    // é a mesma nuvem da de cima, vestida com a forma que substituiu o gizmo retirado ⇒ *a cena
    // mostra as duas metades da ordem dele de uma vez*: em cima as posições nuas, em baixo o que
    // um `Duplicator` faz com elas quando há uma forma.
    let osso = super::sim_demo::indice_de(reg, "source.shape", "kind", "Bone")?;
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };
    let grade = |g: &mut ph2d_nodegraph::graph::Graph, n: NodeId, dx: f32, y: f32| {
        g.set_param(n, "rows", LINHAS);
        g.set_param(n, "cols", COLUNAS);
        g.set_param(n, "gap_x", VAO);
        g.set_param(n, "gap_y", VAO);
        // O deslocamento HORIZONTAL da metade, autorado no nó que o produz.
        if dx != 0.0 {
            let m = g.add_node("motion.move".to_string());
            g.set_pos(m, Pos { x: 200.0, y });
            g.set_param(m, "dx", dx);
            let _ = g.connect(Edge {
                from: (n, 0),
                to: (m, 0),
                delayed: false,
            });
            return m;
        }
        n
    };

    // ── A metade DA ESQUERDA: só posições.
    let so_posicoes = no(g, "motion.grid", 0.0, 0.0);
    let cabeca = grade(g, so_posicoes, -0.5 * AFASTAMENTO, 0.0);
    let saida_a = no(g, "motion.output", 420.0, 0.0);
    g.connect(Edge {
        from: (cabeca, 0),
        to: (saida_a, 0),
        delayed: false,
    })
    .ok()?;

    // ── A metade DA DIREITA: o MESMO grid, vestido.
    let com_forma = no(g, "motion.grid", 0.0, 260.0);
    let corpo = grade(g, com_forma, 0.5 * AFASTAMENTO, 260.0);
    let forma = no(g, "source.shape", 0.0, 380.0);
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, osso);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, TAMANHO);
    // ⭐ A mesma esbelteza da cena do osso: com o `aspect` de fábrica (`1`) a peça sai tão alta
    // quanto longa, e uma grelha deles lê-se como um mosaico de blocos, não como ossos.
    g.set_param(forma, ph2d_node_motion_shape::param::ASPECT, 1.0 / 3.0);
    let dup = no(g, "motion.duplicator", 220.0, 320.0);
    let saida_b = no(g, "motion.output", 420.0, 320.0);
    // ⚠️ A forma na porta `0`, os pontos na `1` — a ordem que o manifesto do duplicador declara.
    for (de, porta) in [(forma, 0u16), (corpo, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }
    g.connect(Edge {
        from: (dup, 0),
        to: (saida_b, 0),
        delayed: false,
    })
    .ok()?;
    Some(vec![saida_a, saida_b])
}

/// O roteiro que o dono segue. ⚠️ **Cada passo nomeia o que aparece NA TELA** (§0.8).
pub(super) fn announce() {
    let n = (COLUNAS * LINHAS) as u32;
    eprintln!(
        "\n[pontos] DUAS metades do MESMO grid de {COLUNAS:.0}x{LINHAS:.0} ({n} posicoes cada),\n\
         lado a lado — as duas cabem no ecra' sem mexer na camera.\n\
         \n\
         A' ESQUERDA = so' posicoes: o grid vai direito ao Output. Nao ha' forma nenhuma.\n\
         A' DIREITA  = as MESMAS posicoes com uma forma, por um Duplicator.\n\
         \n\
         (1) Olhe a metade da ESQUERDA: sao CRUZINHAS, uma por posicao — nao ha' quadrado\n    \
         nenhum. Elas sao do EDITOR: nao entram no que o app entrega.\n\
         (2) Olhe a da DIREITA: as mesmas posicoes, agora com OSSOS. E' o que um Duplicator faz.\n\
         (3) Clique no cartao `Grid` da esquerda. Ele tem um (!) no canto — carregue e leia.\n\
         (4) No mesmo cartao, mexa em `Gap X` / `Gap Y`: as cruzes AFASTAM-SE, e o centro da\n    \
         nuvem fica parado. (Era isto que estava quebrado no report do `gap y`.)\n\
         (5) Carregue no cartao `Duplicator` da direita e no `Shape`: sao eles que fazem pixels.\n\
         \n\
         DEU ERRADO se: a metade da esquerda tiver QUADRADOS em vez de cruzes; se as cruzes nao\n    \
         aparecerem de todo; se so' aparecer UMA das duas metades; ou se mexer no `Gap` fizer a\n    \
         nuvem VIAJAR em vez de espacar.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_pontos_demo_tests.rs"]
mod tests;
