//! A cena `=138` e a `PH2D_MOTION_OBJ_SMOKE=18` — **O FILTRO EM CADA FORMA, A CAIR NUM CAMPO** ([doc 132](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md) §1 e §6).
//!
//! Uma chuva SIMULADA: um `Emitter` larga peças por cima, e um CAMPO (`Falloff` em círculo no
//! meio) diz ao `Wind` para baixo onde puxar — dentro do círculo elas caem mais depressa. Antes da
//! saída, um `Filter` com `Apply To` = `Each Shape`:
//!
//! - `=138` — 4 096 DISCOS (`Shape`) com `Outer Glow`: o degrau de FORA, alcance ≈ leitura (o P1);
//! - `PH2D_MOTION_OBJ_SMOKE=18` — 2 048 cópias de uma SPRITE (o `Object`, rodadas) com `Drop
//!   Shadow`: a silhueta igual nos dois `Apply To` e nas duas rotas (o P6).
//!
//! ⚠️ **O P6 não é a `=142` do roteador de grafo** (doc 132 §0 a numerava assim): a LEI DA APARÊNCIA
//! (`lei_da_aparencia`) só desenha uma corrente que veio de uma forma ou de um OBJECTO, e o objecto
//! mora na HIERARQUIA — que uma cena de grafo não povoa. Medido: a mesma chuva carimbada com o
//! ladrilho de uma grelha desenha 0 sprites. A família do objecto (`motion_object_smoke`) povoa-a.

use super::jogo_demo::{forma, liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// Os discos da `=138`: quantos o emissor mantém vivos (`rate × life`).
pub(crate) const DISCOS: f32 = 4096.0;
/// As cópias da sprite do P6.
pub(crate) const SPRITES: f32 = 2048.0;
/// O tempo de vida de cada peça (s): ela atravessa a cena de cima a baixo.
const VIDA: f32 = 4.0;
/// O raio do CAMPO (o círculo do `Falloff`) e a força para baixo dentro dele.
pub(crate) const CAMPO: f32 = 3.5;
const PUXAO: f32 = 6.0;
/// O tamanho de cada cópia (o `size` do emissor, × o lado do `Object`).
const LADO_DA_SPRITE: f32 = 0.35;

/// Os nós que os gates leem.
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub saida: NodeId,
    pub filtro: NodeId,
    pub campo: NodeId,
}

/// A chuva: emissor (um retângulo por cima) → campo → vento para baixo → integra.
pub(crate) fn chuva(g: &mut Graph, pecas: f32, tamanho: f32) -> Option<(NodeId, NodeId)> {
    let no = no_em(0.0);
    let emissor = no(g, "motion.emitter", -880.0, 0.0);
    for (chave, v) in [
        ("rate", pecas / VIDA),
        ("life", VIDA),
        ("max", pecas),
        ("size", tamanho),
        ("size_random", 0.3),
        ("speed", 1.2),
        ("speed_random", 0.4),
        ("angle", 270.0), // para baixo, neste mundo Y-up
        ("spread", 8.0),
        ("x", 0.0),
        ("y", 5.5),
        ("shape_mode", 3.0), // um retângulo (meias-larguras)
        ("shape_w", 8.0),
        ("shape_h", 0.3),
        ("seed", 13.0),
    ] {
        g.set_param(emissor, chave, v);
    }
    // O CAMPO: um círculo no meio; fora dele o vento não puxa.
    let campo = no(g, "motion.falloff", -660.0, -110.0);
    g.set_param(campo, "shape", 0.0);
    g.set_param(campo, "curve", 0.0);
    g.set_param(campo, "radius", CAMPO);
    let vento = no(g, "force.wind", -440.0, -110.0);
    g.set_param(vento, "angle", 270.0);
    g.set_param(vento, "strength", PUXAO);
    g.set_param(vento, "gust", 0.0);
    // O laço da simulação: o emissor é o `rest`; a saída volta (`pre`) pelo campo e pelo vento às
    // `forces`.
    let integra = no(g, "motion.integrate", -220.0, 0.0);
    liga(g, (emissor, 0), integra, 0, false)?;
    liga(g, (integra, 0), campo, 0, true)?;
    liga(g, (campo, 0), vento, 0, false)?;
    liga(g, (vento, 0), integra, 1, false)?;
    Some((integra, campo))
}

/// O degradê de cor pela ordem (azul → laranja), o filtro e a saída.
fn acabamento(
    g: &mut Graph,
    de: NodeId,
    efeito: u8,
    reg: &NodeRegistry,
    campo: NodeId,
) -> Option<Cena> {
    let no = no_em(0.0);
    let tint = no(g, "motion.tint", 220.0, 0.0);
    g.set_param(tint, "mode", 1.0);
    for (chave, v) in ["r", "g", "b", "a"].into_iter().zip([0.15, 0.45, 1.0, 1.0]) {
        g.set_param(tint, chave, v);
    }
    for (chave, v) in ["r2", "g2", "b2", "a2"]
        .into_iter()
        .zip([1.0, 0.6, 0.1, 1.0])
    {
        g.set_param(tint, chave, v);
    }
    liga(g, (de, 0), tint, 0, false)?;
    // ⭐ O FILTRO em CADA forma (o `Apply To` de omissão), o tipo pelo MESMO gesto do cartão.
    let filtro = no(g, ph2d_node_fx_filter::TYPE_NAME, 440.0, 0.0);
    ph2d_node_fx_filter::switch_kind(g, filtro, efeito);
    liga(g, (tint, 0), filtro, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 0.0);
    liga(g, (filtro, 0), saida, 0, false)?;
    g.validate(reg).ok()?;
    Some(Cena {
        saida,
        filtro,
        campo,
    })
}

/// A `=138`: 4 096 discos com `Outer Glow` em cada um.
pub(crate) fn monta_discos(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let (integra, campo) = chuva(g, DISCOS, 1.0)?;
    let no = no_em(0.0);
    let disco = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Circle,
        0.16,
        [1.0, 1.0, 1.0],
        -220.0,
        160.0,
    )?;
    let pecas = no(g, "motion.duplicator", 0.0, 0.0);
    liga(g, (disco, 0), pecas, 0, false)?;
    liga(g, (integra, 0), pecas, 1, false)?;
    acabamento(g, pecas, ph2d_ecs::FxOp::GLOW, reg, campo)
}

/// O P6: 2 048 cópias da sprite `objeto` (rodadas 30°) com `Drop Shadow` em cada uma.
pub(crate) fn monta_sprites(doc: &mut MotionDoc, reg: &NodeRegistry, objeto: &str) -> Option<Cena> {
    let g = &mut doc.graph;
    let (integra, campo) = chuva(g, SPRITES, LADO_DA_SPRITE)?;
    let no = no_em(0.0);
    let fonte = no(g, "source.object", -220.0, 160.0);
    g.set_text_param(fonte, "object", objeto);
    // A escala do PONTO (o `size` do emissor) multiplica a do objecto.
    let pecas = no(g, "motion.duplicator", 0.0, 0.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (fonte, 0), pecas, 0, false)?;
    liga(g, (integra, 0), pecas, 1, false)?;
    let roda = no(g, "motion.rotate", 0.0, 110.0);
    g.set_param(roda, "angle", 30.0);
    liga(g, (pecas, 0), roda, 0, false)?;
    acabamento(g, roda, ph2d_ecs::FxOp::DROP_SHADOW, reg, campo)
}

pub(super) fn build_discos(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta_discos(doc, reg).map(|c| vec![c.saida])
}

/// A ficha, por baixo da chuva.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    // À esquerda da chuva e dentro do canvas visto: no meio os discos caíam por ela, e a -5,2 o
    // grafo tapava-a (régua `motion_demo_legend::tapa`, 15/10).
    vec![crate::motion_demo_legend::Caption::new(
        [-5.1, -4.6],
        "the middle circle is the field: inside it everything falls faster".to_string(),
    )]
}

/// **O roteiro da `=138`** (`CLAUDE.md` §0.8).
pub(super) fn announce_discos() {
    eprintln!(
        "\n[o brilho em cada forma] UMA CHUVA de 4096 discos coloridos (azul para laranja) que nascem\n\
         em cima e caem - uma SIMULACAO: no circulo do meio (o CAMPO, um cartao `Falloff`) o vento\n\
         puxa para baixo e eles ACELERAM. Antes da saida ha' um no' `Filter` com `Effect` = `Outer\n\
         Glow` e `Apply To` = `Each Shape`.\n\
         \n\
         (1) De PLAY. Cada disco tem o seu BRILHO a volta. Onde dois discos se encostam, cada um\n    \
         guarda o seu brilho inteiro (os brilhos SOMAM: o encontro fica mais claro).\n\
         (2) No cartao `Filter`, troque `Apply To` para `Whole Layer`: os discos que se tocam viram UMA\n    \
         peca e o brilho passa a contornar o grupo. Volte para `Each Shape`.\n\
         (3) O CUSTO: abra o app com `PH2D_FLUID_PROFILE=1` antes do comando. A cada 2 segundos o\n    \
         terminal imprime uma linha `[gpu]`; procure `fx_cells.pilha`, `fx.borrao_h` e\n    \
         `fx.borrao_v` (os ms da placa por quadro). Com a chuva cheia (4096 discos) o terminal diz\n    \
         `fx_cells.pilha` perto de 6 ms com `(x7.0)`: sete paginas por quadro. Antes desta versao\n    \
         eram ~10 ms e `(x10.0)` - cada disco pagava duas celulas. `render.vello` fica abaixo de 1 ms\n    \
         (antes ele somava o filtro inteiro).\n\
         \n\
         DEU ERRADO se: os discos nao cairem; nao acelerarem dentro do circulo; o brilho tremer ou\n\
         sumir num disco que se encosta a outro em `Each Shape`; ou nao aparecer `fx_cells.pilha` na\n\
         linha `[gpu]` com `PH2D_FLUID_PROFILE=1`.\n"
    );
}

/// **O roteiro do P6** (`PH2D_MOTION_OBJ_SMOKE=18`, `CLAUDE.md` §0.8).
pub(crate) fn announce_sprites() {
    eprintln!(
        "\n[a sombra em cada sprite] UMA CHUVA de 2048 copias da IMAGEM `Object` (o quadrado no\n\
         centro da cena), todas rodadas, que caem - a SIMULACAO da `=138`: no circulo do meio (o CAMPO, o cartao\n\
         `Falloff`) eles ACELERAM. Antes da saida ha' um `Filter` com `Effect` = `Drop Shadow` e\n\
         `Apply To` = `Each Shape`.\n\
         \n\
         (1) De PLAY. Cada quadrado projecta a SUA sombra, para baixo e para a direita; a sombra\n    \
         de um cai por cima do vizinho.\n\
         (2) No cartao `Filter`, troque `Apply To` para `Whole Layer`: a sombra passa a ser do grupo\n    \
         inteiro (nao cai em cima de nenhum quadrado). As BORDAS de cada quadrado ficam EXACTAMENTE\n    \
         iguais nas duas opcoes - so' a sombra muda. Volte para `Each Shape`.\n\
         (3) Pare o tempo e aproxime (roda do rato) um quadrado: a borda e' uma linha limpa; troque\n    \
         `Apply To` outra vez - a borda nao mexe um pixel.\n\
         \n\
         DEU ERRADO se: os quadrados nao cairem; a borda de um quadrado piscar ou mudar ao trocar\n\
         `Apply To`; ou a sombra sair por cima do quadrado que a projecta.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_cada_forma_demo_tests.rs"]
mod tests;
