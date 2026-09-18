//! ⭐⭐⭐ **A CENA `=123` — A CADEIA** (doc 115 §18, ordem do dono: *«quero todas as possibilidades
//! possíveis, não quero limitações no sistema»*).
//!
//! # Porque esta cena existe, e porque ela NÃO podia ser a `=121` nem a `=122`
//!
//! Aquelas duas são feitas de **pares INDEPENDENTES**, e essa escolha está escrita nelas: um par é
//! um problema local, e o passe resolve-o em poucas varreduras. ⇒ *nelas a escada de varreduras é
//! invisível* — a `8`, a `64` ou a `1024` o artista vê a mesma coisa.
//!
//! Uma **CADEIA** é o oposto: cada peça encosta na seguinte, e a informação tem de viajar de uma
//! ponta à outra. Medido (peças de lado `1` a um quarto de passo, pares atravessados > `2 %`):
//!
//! | n | antes | a 64 | a 256 | a 1024 |
//! |---|---|---|---|---|
//! | 16 | 42 | 23 | 15 | **0** |
//!
//! ⇒ é **aqui**, e só aqui, que o tecto das varreduras se vê. Com o `64` herdado esta cena era
//! impossível de mostrar; é por isso que ela nasce com a wave que mediu o tecto.
//!
//! # ⚠️ Ela nasce DESARMADA, como as duas irmãs
//!
//! O primeiro quadro tem de mostrar a cadeia encavalitada, senão o artista vê o resultado e nunca
//! a causa. E o `Collide Sweeps` fica no valor de fábrica (`8`) de propósito: o passo (3) do
//! roteiro é ele **não chegar**, que é o que dá sentido ao passo (4).
//!
//! # ⛔ Porque ela NÃO treme
//!
//! A `=122` treme porque o assunto dela é *o passe corre em todo quadro*. O assunto desta é a
//! **ESCADA**, e um knob de cada vez é a lei do doc 103 — movimento aqui só acrescentaria uma
//! variável a uma leitura que já é subtil.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use ph2d_motion_doc::MotionDoc;

/// O comprimento da cadeia. ⭐ **`16` não é escolhido:** é o `n` que a medição do §18 põe
/// exactamente entre os dois tectos — `64` (o herdado) deixa-lhe `23` pares atravessados e `1024`
/// (o do slider de hoje) fecha-a, a `18,5 %` de um quadro.
pub(super) const PECAS: usize = 16;

/// O `Size` autorado no cartão da forma — a MEIA extensão de mundo (a geometria de uma
/// `source.shape` vive em raio `1`), a armadilha que a §12.1 e as duas irmãs já pagaram.
const MEIA: f32 = 0.09;
/// O lado de uma peça, em mundo.
const LADO: f32 = 2.0 * MEIA;

/// O passo entre peças vizinhas. ⭐ **Um QUARTO do lado**, que é a fixtura da medição do §18: cada
/// peça encavalita nas três vizinhas de cada lado, e é essa profundidade que faz da fila uma
/// CADEIA em vez de uma fileira de pares.
const PASSO: f32 = 0.25 * LADO;

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let quadrado = super::sim_demo::indice_de(reg, "source.shape", "kind", "Square")?;
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str| g.add_node(tipo.to_string());

    // A FORMA, com o botão que faz a peça DECLARAR a caixa dela (doc 115 W4).
    let forma = no(g, "source.shape");
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, quadrado);
    g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, MEIA);
    g.set_label(forma, "Shape");

    // Os PONTOS: UMA fila, encavalitada — a cadeia.
    let grelha = no(g, "motion.grid");
    g.set_param(grelha, "rows", 1.0);
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de pecas, 16")]
    g.set_param(grelha, "cols", PECAS as f32);
    g.set_param(grelha, "gap_x", PASSO);
    g.set_label(grelha, "Grid (a cadeia)");

    let dup = no(g, "motion.duplicator");
    let saida = no(g, "motion.output");

    for (i, n) in [forma, grelha, dup].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                x: 40.0 + i as f32 * 180.0,
                y: 240.0,
            },
        );
    }
    g.set_pos(saida, Pos { x: 640.0, y: 240.0 });

    for (a, ap, b, bp) in [
        (forma, 0u16, dup, 0u16),
        (grelha, 0, dup, 1),
        (dup, 0, saida, 0),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .ok()?;
    }
    doc.graph.validate(reg).ok()?;
    // ⚠️⚠️ **Nenhum override no sink** — a cena nasce desarmada e com o `Collide Sweeps` de
    // fábrica, logo não pode divergir do valor de fábrica no dia em que ele mudar. Quem liga e quem
    // sobe o número é o artista, e é isso que o roteiro ensina.
    Some(vec![saida])
}

/// O roteiro que o dono segue. ⚠️ Cada passo nomeia o que aparece NA TELA (§0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[cadeia] UMA FILA de {PECAS} quadrados ENCAVALITADOS — cada um metido dentro dos\n\
         vizinhos. Isto e' uma CADEIA, e nao uma fila de pares: por isso ela e' o unico sitio\n\
         onde o numero de varreduras se VE'.\n\
         \n\
         (1) Carregue no cartao `Output` (o ultimo no' da fila).\n\
         (2) Ligue `Collide`. A cadeia abre-se um pouco — e continua encavalitada.\n\
         (3) Repare que `Collide Sweeps` esta' em 8. Suba para 64 e veja: ela abre MAIS, e\n    \
         ainda nao chega. Era aqui que o sistema parava ate' 18/09.\n\
         (4) Arraste `Collide Sweeps` ate' ao fim (1024). A cadeia abre-se INTEIRA: os\n    \
         quadrados ficam encostados, sem nenhum metido dentro do outro.\n\
         (5) Quer mais? ESCREVA 4096 na caixa. O slider para em 1024 porque e' ate' ali que a\n    \
         mao trabalha; o numero que se escreve e' o numero que corre — nada e' cortado em\n    \
         silencio.\n\
         \n\
         (i) Quanto custa: 1024 varreduras nesta fila custam ~a quinta parte de um quadro.\n    \
         4096 custam ~um quadro e meio, e fecham uma cadeia do DOBRO do comprimento.\n\
         \n\
         (!) Se ligar `Collide` e nada se mexer, o defeito e' a forma nao estar a DECLARAR a\n    \
         caixa dela — abra o cartao `Shape` e confirme que o `Collide` DELE esta' ligado.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_passe_cadeia_demo_tests.rs"]
mod tests;
