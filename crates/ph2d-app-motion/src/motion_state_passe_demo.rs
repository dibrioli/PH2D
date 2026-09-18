//! ⭐⭐⭐ **A CENA DO PASSE AUTOMÁTICO** (doc 115 W5, cena `=121`) — ordem do dono (2026-09-17):
//! *«tirar o collide e deixar tudo pela Shape»* · *«o app separa sozinho»*.
//!
//! # O que ela mostra, e porque a metade que IMPORTA é o grafo estar VAZIO de colisão
//!
//! Uma fileira de formas com o botão **`Collide`** ligado, carimbadas por um duplicador num
//! amontoado onde elas se atravessam — e **nenhum nó de colisão em lado nenhum**. O artista abre o
//! cartão do **Output** e liga **`Collide`**: elas assentam.
//!
//! ⛔⛔ **É por isso que ela não podia ser a `=48`.** Aquela cena é o demo do `motion.collide` e as
//! peças dela são uma `motion.grid` de pontos — **elas não DECLARAM forma nenhuma** e vivem do
//! recuo de raio do cartão daquele nó, que o passe automático deliberadamente não tem
//! (`ph2d_contact::passe`: *inventar um raio seria afirmar que uma peça colide quando ninguém o
//! disse*). Ligar o interruptor ali não separaria nada, e a cena ensinaria o contrário do que
//! acontece — a espécie que o `CLAUDE.md` §5.0 chama de **pior que uma cena ausente**.
//!
//! ⭐ E esse mesmo facto é o preço da W6 já medido: *a `=48` não migra por troca directa*.
//!
//! # ⚠️ A cena nasce DESARMADA, de propósito
//!
//! O primeiro quadro tem de mostrar o amontoado. Se ela abrisse já separada, o artista veria o
//! resultado e não a CAUSA — e o interruptor, que é o assunto inteiro da wave, seria invisível.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use ph2d_motion_doc::MotionDoc;

/// Quantas peças a cena carimba — doze PARES.
pub(super) const PECAS: usize = 2 * COLUNAS;

/// ⭐⭐⭐ **A cena são PARES, e não uma fileira — e a razão está MEDIDA.**
///
/// ⛔⛔ Separar uma FILEIRA é uma **cadeia**, e o solver é Jacobi com médias: a informação viaja um
/// elo por varredura, logo uma cadeia de `n` peças precisa de `~n²` varreduras. Medido nesta cena:
/// com `8` peças por fileira e `20 %` de sobreposição, `32` varreduras deixavam `0,0055` de
/// penetração residual; com `6` peças e `15 %`, ainda ficavam `12` pares de `20`. *Não é o motor
/// que falha — é a topologia do problema.*
///
/// ⇒ **doze PARES independentes**: cada um resolve-se sozinho, o número de FÁBRICA chega, e o olho
/// lê doze coisas a acontecer em vez de uma fileira a esticar.
const COLUNAS: usize = 12;

/// O `Size` autorado no cartão da forma. ⚠️ **Ele não é o LADO** — ver [`MEIA`] abaixo.
const TAMANHO: f32 = 0.09;

/// ⭐⭐ **A MEIA EXTENSÃO de mundo de uma peça — MEDIDA, não suposta.**
///
/// ⛔⛔ A 1.ª redacção desta cena tratou o `size` da forma como o LADO, e as peças ficaram com o
/// dobro do que ela supunha: a geometria de uma `source.shape` vive em **raio 1** (doc 89, folha
/// 14), logo a meia de mundo **é** o `size` e o lado é o dobro. Medido pela porta do produto:
/// `size = 0,09` ⇒ `Caixa { meia: [0,09; 0,09] }`. *É a mesma família do erro que a §12.1 deste doc
/// registou na W4 — confundir uma extensão de GEOMETRIA com uma de MUNDO.*
const MEIA: f32 = TAMANHO;

/// O lado de uma peça, em mundo.
const LADO: f32 = 2.0 * MEIA;

/// O vão ENTRE pares — maior que o lado, logo **dois pares nunca se tocam**. É isto que faz de cada
/// par um problema independente.
const VAO_ENTRE_PARES: f32 = 1.35 * LADO;

/// O passo DENTRO de um par: as duas peças entram uma na outra em `30 %` do lado.
///
/// ⚠️ **Uma fracção do lado e não um número solto** — é o que impede a próxima pessoa a mexer no
/// tamanho de partir a cena sem reparar.
const SOBREPOSICAO_DO_PAR: f32 = 0.70 * LADO;

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let quadrado = super::sim_demo::indice_de(reg, "source.shape", "kind", "Square")?;
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };

    // A FORMA, com o botão que o dono nomeou. ⚠️ É ele que faz a forma DECLARAR a caixa dela —
    // sem ele a corrente não traz colisor e o passe não teria o que separar.
    let forma = no(g, "source.shape", 0.0, 0.0);
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, quadrado);
    g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, TAMANHO);

    // Os PONTOS: doze colunas bem afastadas, duas linhas que se atravessam ⇒ doze PARES.
    let grade = no(g, "motion.grid", 0.0, 160.0);
    g.set_param(grade, "rows", 2.0);
    g.set_param(grade, "cols", COLUNAS as f32);
    g.set_param(grade, "gap_x", VAO_ENTRE_PARES);
    g.set_param(grade, "gap_y", SOBREPOSICAO_DO_PAR);

    // O DUPLICADOR: a forma na porta `0`, os pontos na `1` (doc 115 §11.2, a 1.ª mentira do arnês).
    let dup = no(g, "motion.duplicator", 220.0, 80.0);
    let out = no(g, "motion.output", 420.0, 80.0);
    for (de, porta) in [(forma, 0u16), (grade, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }
    g.connect(Edge {
        from: (dup, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    // ⚠️⚠️ **O interruptor NÃO se escreve, e isso é mais forte do que escrevê-lo a zero:** a cena
    // nasce desarmada porque não autora override nenhum, logo ela não pode divergir do valor de
    // fábrica no dia em que ele mudar. O artista é que o liga.
    Some(vec![out])
}

/// O roteiro que o dono segue. ⚠️ **Cada passo nomeia o que aparece NA TELA** (§0.8) — e o passo
/// (1) é o amontoado, que é a metade sem a qual o (3) não quer dizer nada.
pub(super) fn announce() {
    eprintln!(
        "\n[passe] {COLUNAS} PARES ({PECAS} quadrados): em cada par um quadrado entra no outro — \
         e NENHUM no' de colisao no grafo.\n\
         \n\
         (1) Repare no amontoado: as pecas atravessam-se.\n\
         (2) Carregue no cartao `Output` (o no' do fim, a direita).\n\
         (3) Ligue `Collide`. As pecas assentam, cada uma pela CAIXA dela.\n\
         (4) `Collide Sweeps` (que so' aparece com o `Collide` ligado) diz quantas passagens\n    \
         por quadro: menos = mais rapido e menos arrumado.\n\
         (5) Desligue `Collide` outra vez: elas voltam ao amontoado.\n\
         \n\
         (!) Se ligar `Collide` e nada se mexer, o defeito e' a forma nao estar a DECLARAR a\n    \
         caixa dela — abra o cartao `Shape` e confirme que o `Collide` DELE esta' ligado.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_passe_demo_tests.rs"]
mod tests;
