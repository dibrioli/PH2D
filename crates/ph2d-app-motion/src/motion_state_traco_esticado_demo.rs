//! ⭐⭐⭐ **AS ESTRELAS ESTICADAS COM CONTORNO** (cena `=127`) — a cena que MOSTRA a W4 do doc 121,
//! porque **nenhuma cena do catálogo a mostra**.
//!
//! # Porque ela existe
//!
//! A W4 levou à placa o traço de uma cópia **esticada num eixo só** (afim NÃO conforme): antes dela
//! uma cópia assim devolvia o quadro INTEIRO ao Vello. Medido pelo censo de rota
//! (`motion_bridge_gpu_rota_das_formas_probe`), **nenhuma** das cenas com forma do catálogo tem a
//! combinação *traço + escala não-uniforme* — a `=126` tem as estrelas mas sem contorno e com a
//! escala igual nos dois eixos, logo ela desenha exactamente o mesmo antes e depois da W4.
//! *Uma cena que não contém o fenómeno ensina que a cura não faz nada* (`CLAUDE.md` §5.0).
//!
//! # A CADEIA
//!
//! `motion.grid` → `motion.integrate` (a galáxia da `=126`, a MESMA lei) → `motion.duplicator` ←
//! `source.shape` (**Star**, com preenchimento e CONTORNO) → `motion.scale` (**`uniform = 0`**: o
//! X estica e o Y encolhe) → `motion.output`.
//!
//! ⭐ **A simulação é a da `=126`, reutilizada e não copiada** ([`super::carimbo_demo::simulacao`]):
//! o disco em rotação rígida cuja derivação vive lá (o doc da `GALAXIA`). Duas cópias da lei
//! divergiriam no dia em que alguém afinasse uma delas.
//!
//! # O que o dono tem de VER
//!
//! A lei do traço da casa (`ph2d_vec_render::stroke_uniform::pen_for`, bug #27 do vector: *«quando
//! engrossa, engrossa por igual nos dois eixos»*) pede a CANETA REDONDA: o contorno de uma estrela
//! esticada tem a MESMA grossura nos lados compridos e nos curtos. ⛔ Uma caneta elíptica (o que o
//! Vello faria com o traço expandido no espaço local) daria contornos grossos em cima e finos dos
//! lados — é esse o *«deu errado»* do roteiro.
//!
//! ⚠️ **As duas rotas desenham a MESMA lei** (o Vello e a placa, gate
//! `a_rota_da_placa_desenha_o_traco_esticado_como_a_casa`): o que muda entre o comando de sempre e
//! `PH2D_FORMAS_NA_PLACA=0` é a FOLGA do quadro (o `raw`), não a imagem.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use super::carimbo_demo::PX_POR_UNIDADE;

/// **A pegada de uma estrela ANTES de esticar, em píxeis.** Maior do que a da `=126` (`6 px`) de
/// propósito: o assunto desta cena é o CONTORNO, e um contorno de dois píxeis numa estrela de seis
/// é uma mancha.
pub(crate) const ESTRELA_PX: f32 = 22.0;

/// O `size` do `source.shape` é o **raio** (metade da pegada).
pub(crate) const TAMANHO: f32 = ESTRELA_PX / (2.0 * PX_POR_UNIDADE);

/// **O esticão**: o X cresce e o Y encolhe. ⚠️ Os dois têm de ser DIFERENTES — é isso que faz a
/// cópia NÃO conforme, e há gate. O produto (`1,08`) fica perto de `1` para a caneta
/// (`w·√|det|`) ter quase a largura autorada: o que a cena mostra é a FORMA dela, não o tamanho.
pub(crate) const ESTICA_X: f32 = 1.8;
pub(crate) const ESTICA_Y: f32 = 0.6;

/// **O contorno, em píxeis de ecrã** — e a largura em unidades de mundo sai dele pela escala da
/// câmara de arranque (o param é de mundo, ver o doc do `stroke_width`).
pub(crate) const CONTORNO_PX: f32 = 1.5;
pub(crate) const CONTORNO: f32 = CONTORNO_PX / PX_POR_UNIDADE;

/// O lado da grelha. ⚠️ **Não é o tecto da `=126`:** a galáxia é um disco rígido de NÚCLEO `17 m`,
/// e fora dele o ímã ganha à órbita — o canto do campo tem de ficar dentro (`gate
/// o_campo_cabe_no_nucleo_da_galaxia`).
pub(crate) const LADO_N: u32 = 32;

/// O vão entre posições — o MESMO nos dois eixos, e igual à pegada ESTICADA inteira.
///
/// ⛔ A 1.ª redacção dava a cada eixo a pegada dele (`2,4 × tamanho × esticão`), e a foto
/// desmentiu-a: a galáxia RODA as posições e o esticão fica sempre na horizontal, logo uma fileira
/// que começa horizontal passa a diagonal e as estrelas de vão curto FUNDEM-SE numa tira contínua,
/// em que o contorno de uma corta o da vizinha. Com o vão do eixo comprido nos dois sentidos, a
/// distância entre vizinhas nunca fica abaixo da largura esticada, seja qual for o ângulo.
pub(crate) const VAO_X: f32 = 2.0 * TAMANHO * ESTICA_X;
pub(crate) const VAO_Y: f32 = VAO_X;

/// Quantas estrelas a cena desenha.
pub(crate) const ESTRELAS: u32 = LADO_N * LADO_N;

/// O índice da `Star` no enum — pela mesma porta da `=126` (nunca pelo rótulo, que é i18n).
fn indice_da_estrela() -> f32 {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
        .expect("a `Star` tem de estar no `ALL_KINDS`");
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    {
        i as f32
    }
}

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    for tipo in [
        "motion.grid",
        "motion.duplicator",
        "source.shape",
        "motion.scale",
        "motion.output",
        "motion.integrate",
        "force.vortex",
        "force.attractor",
        "force.curl",
        "motion.falloff",
    ] {
        reg.manifests()
            .find(|m| m.id == ph2d_nodegraph::node::NodeTypeId::of(tipo))?;
    }
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };

    let grade = no(g, "motion.grid", 0.0, 0.0);
    #[expect(clippy::cast_precision_loss, reason = "um lado de grelha pequeno")]
    let lado = LADO_N as f32;
    g.set_param(grade, "rows", lado);
    g.set_param(grade, "cols", lado);
    g.set_param(grade, "gap_x", VAO_X);
    g.set_param(grade, "gap_y", VAO_Y);

    // ── A FORMA: uma estrela AMARELA com CONTORNO azul-escuro. ⚠️ Sem tracejado: o tracejado sob
    // escala não-uniforme continua no Vello (doc 121 §9) e a cena mostraria a rota errada.
    use ph2d_node_motion_shape::param as p;
    let forma = no(g, "source.shape", 0.0, 220.0);
    for (chave, valor) in [
        (p::KIND, indice_da_estrela()),
        (p::SIZE, TAMANHO),
        (p::FILL, 1.0),
        (p::FILL_R, 1.0),
        (p::FILL_G, 0.82),
        (p::FILL_B, 0.25),
        (p::FILL_A, 1.0),
        (p::STROKE_WIDTH, CONTORNO),
        (p::STROKE_R, 0.06),
        (p::STROKE_G, 0.10),
        (p::STROKE_B, 0.35),
        (p::STROKE_A, 1.0),
    ] {
        g.set_param(forma, chave, valor);
    }

    // ── A SIMULAÇÃO COM CAMPOS: a galáxia da `=126` (regra do dono, doc 103 §1).
    let ig = super::carimbo_demo::simulacao(g, grade)?;

    // ── O CARIMBO. A forma na porta `0`, os pontos na `1` (o manifesto do duplicador).
    let dup = no(g, "motion.duplicator", 240.0, 110.0);
    for (de, porta) in [(forma, 0u16), (ig, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }

    // ── O ESTICÃO: um eixo cresce, o outro encolhe. É ele que faz cada cópia NÃO conforme.
    let estica = no(g, "motion.scale", 460.0, 110.0);
    g.set_param(estica, "uniform", 0.0);
    g.set_param(estica, "amount", ESTICA_X);
    g.set_param(estica, "amount_y", ESTICA_Y);
    g.connect(Edge {
        from: (dup, 0),
        to: (estica, 0),
        delayed: false,
    })
    .ok()?;

    let saida = no(g, "motion.output", 680.0, 110.0);
    g.connect(Edge {
        from: (estica, 0),
        to: (saida, 0),
        delayed: false,
    })
    .ok()?;
    Some(vec![saida])
}

/// **O roteiro que o dono segue.** Cada passo nomeia o que aparece NA TELA (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    let n = ESTRELAS;
    eprintln!(
        "\n[estrelas esticadas] {n} ESTRELAS AMARELAS COM CONTORNO AZUL, esticadas para os lados:\n\
         cada uma fica mais larga do que alta. Elas GIRAM devagar, como uma galaxia: e' uma\n\
         SIMULACAO com campos de forca (redemoinho, iman, ruido), a mesma da cena =126.\n\
         \n\
         (1) Aproxime com a roda do rato ate' uma estrela ocupar um bom pedaco do ecra.\n    \
         Olhe o CONTORNO AZUL: ele tem a MESMA grossura nas pontas compridas (dos lados) e\n    \
         nas curtas (em cima e em baixo), como um risco feito com uma caneta redonda.\n\
         (2) Arraste o fundo com o botao do meio: tem de passear LISO, e as estrelas continuam\n    \
         a girar.\n\
         (3) Olhe a BARRA DE BAIXO do ecra e ANOTE o terceiro numero, o `raw` (a folga do\n    \
         quadro: quanto MAIOR, melhor).\n\
         (4) Feche o app e corra o MESMO comando com `PH2D_FORMAS_NA_PLACA=0` a' frente: e' o\n    \
         caminho antigo, em que o processador desenhava as estrelas. A imagem tem de ser a\n    \
         MESMA (o mesmo contorno, a mesma grossura); o `raw` CAI.\n\
         \n\
         DEU ERRADO se: as estrelas nao aparecerem; se ficarem PARADAS; se o contorno for\n\
         GROSSO em cima e em baixo e FINO dos lados (ou ao contrario); se o contorno tiver\n\
         BURACOS ou DENTES nas pontas; se a imagem for DIFERENTE entre as duas corridas do\n\
         passo (4); ou se o `raw` for IGUAL nas duas.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_traco_esticado_demo_tests.rs"]
mod tests;
