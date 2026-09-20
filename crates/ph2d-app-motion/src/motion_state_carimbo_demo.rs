//! ⭐⭐⭐ **O CAMPO DE ESTRELAS** (cena `=126`) — a cena que MOSTRA a cura do carimbo, porque
//! **nenhuma cena do produto a mostra**.
//!
//! # Porque ela existe
//!
//! O report do dono é de 2026-09-14 (*«usando shape (exemplo: star) fps cai para 27»*) e a cura
//! shipou em 2026-09-20: a forma passa a ser encodada **uma vez** e carimbada por cópia
//! ([`ph2d_vector::VectorScene::fill_prepared`]), com as duas rotas a escreverem os **mesmos
//! bytes** (gate `o_carimbo_preparado_escreve_os_mesmos_bytes`, na `ph2d-vector`).
//! `PH2D_CARIMBO_PREPARADO=0` devolve o caminho de antes.
//!
//! ⚠️⚠️ **A diferença é SÓ tempo — a imagem é a mesma ponto por ponto** ⇒ uma cena que a mostre
//! tem de ser grande ao ponto de o RELÓGIO se ver, e **nenhuma cena do catálogo é**: medido (doc
//! 116 §5.7), o pior cartão de todas elas desenha **`190`** linhas e custa `0,017 ms`, que é
//! `0,1 %` de um quadro. *Só o grafo do próprio dono — o do grid grande — produz o fenómeno*, e é
//! esse grafo que esta cena monta.
//!
//! # A CADEIA é a do report, e nada mais
//!
//! `motion.grid` (`560 × 560`) → `motion.duplicator` ← `source.shape` (**Star**) → `motion.output`
//!
//! ⛔ **Sem oscilador, sem campo, sem simulação.** Tudo o que se acrescentasse entrava na conta do
//! quadro e a cena passaria a medir outra coisa — *o que está aqui é o mínimo que produz o
//! fenómeno*, e é por isso que ela não é uma cena de ciclo.
//!
//! # A POPULAÇÃO é DERIVADA, e a janela dela é ERRO DE COMPILAÇÃO
//!
//! O recurso é o **quadro de 60 fps** (`16,67 ms`), e o custo por cópia está MEDIDO pelas portas
//! do produto ([`crate::motion_carimbo_relogio_probe`], `audit_the_stamp_frame_split`, a
//! `102 400` cópias, `73`–`85 %` de CPU ociosa):
//!
//! | rota | desenho | cozer | CPU do quadro | **por cópia** |
//! |---|---:|---:|---:|---:|
//! | `fill` por cópia (ANTES) | `6,37 ms` | `0,99 ms` | `7,36 ms` (`44 %`) | **`0,0719 µs`** |
//! | carimbo preparado (HOJE) | `2,25 ms` | `0,99 ms` | `3,24 ms` (`19 %`) | **`0,0316 µs`** |
//!
//! ⇒ a `560 × 560` = **`313 600`** cópias, a conta dá **`22,6 ms`** pela rota antiga (mais de um
//! quadro inteiro ⇒ o número de baixo CAI) e **`9,9 ms`** pela de hoje (`59 %` de um quadro ⇒ ele
//! fica em `60`). É essa DISTÂNCIA que a cena existe para mostrar, e as duas metades dela estão
//! presas por [`_A_JANELA_DA_POPULACAO`].
//!
//! ⚠️ **A extrapolação é legítima porque o custo por cópia é PLANO**, medido sobre um intervalo de
//! `1000×` (`1 000` a `1 000 000` cópias, mesma razão `3,2×`) — não é um palpite sobre o joelho de
//! uma curva.
//!
//! # Porque o campo é MAIOR do que o ecrã
//!
//! ⛔ **As duas leis puxam em sentidos opostos e a aritmética não deixa cedermos as duas:** uma
//! estrela só se lê como estrela com `~6 px` (`0,108` de mundo a `55,5 px` por unidade, a régua
//! medida da cena `=124`), e o que a câmara de arranque mostra são `21,8 × 6,8` unidades ⇒ cabem
//! **`~10 000`** estrelas legíveis no ecrã, e a cena precisa de **`313 600`** para o relógio se
//! mexer. *Encolher a estrela até tudo caber entrega um rectângulo cinzento* — e uma cena em que o
//! dono não vê estrelas não ensina que isto são estrelas.
//!
//! ⇒ o campo é `~73 × 73` unidades e o ecrã mostra um pedaço dele. **A conta é paga pelas
//! `313 600`**, estejam elas à vista ou não: o desenho não tem recorte por câmara, e é isso que
//! faz a cena medir o que ela diz medir.
//!
//! # Como o dono compara (o roteiro está no [`announce`])
//!
//! ```text
//! cargo run -p ph2d-host-desktop --release -- ... PH2D_GPU_COOK_DEMO=126
//! env PH2D_CARIMBO_PREPARADO=0 …o mesmo comando…
//! ```
//!
//! ⚠️ **`--release` e não `--profile smoke`** (`CLAUDE.md` §5): este é um smoke de PERFORMANCE, e
//! o `smoke` não tem LTO — ali as duas colunas mediriam o perfil de build.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// **Quantos píxeis de ecrã vale uma unidade de mundo na câmara de arranque** — medido na foto da
/// cena `=124` e citado dela, nunca re-estimado.
const PX_POR_UNIDADE: f32 = 55.5;

/// **A pegada de uma estrela, em píxeis** — o que faz dela uma ESTRELA e não um ponto.
///
/// ⚠️ Abaixo disto as cinco pontas fundem-se e o campo lê-se como um granulado: a cena passaria a
/// mostrar *«um rectângulo que fica mais lento»*, que não é o assunto dela.
const ESTRELA_PX: f32 = 6.0;

/// O `size` do `source.shape` é o **raio** (metade da pegada) — ver o doc do param.
pub(super) const TAMANHO: f32 = ESTRELA_PX / (2.0 * PX_POR_UNIDADE);

/// O vão entre posições: a pegada inteira mais `20 %` de ar.
///
/// ⚠️ Com as estrelas encostadas o campo volta a ler-se como uma textura — a mesma lei que a
/// cena `=124` pagou com a foto das cruzes fundidas numa treliça.
pub(super) const VAO: f32 = 2.4 * TAMANHO;

/// **O LADO da grelha.** Ver a tabela do cabeçalho: ele não é escolhido, é o que põe as duas rotas
/// em lados opostos de um quadro de 60 fps.
pub(super) const LADO_N: u32 = 560;

/// O mesmo número para quem escreve o param (o nó lê `f32`). ⚠️ **Uma fonte só** — dois literais
/// aqui divergiriam no dia em que alguém mexesse num deles.
pub(super) const LADO: f32 = LADO_N as f32;

/// Quantas estrelas a cena carimba.
pub(super) const ESTRELAS: u64 = (LADO_N as u64) * (LADO_N as u64);

/// Um quadro de 60 fps, em nanossegundos — o RECURSO de que a população sai.
const QUADRO_NS: u64 = 16_667_000;

/// O custo de CPU de uma cópia pela rota ANTIGA, em nanossegundos (`0,0719 µs`, arredondado para
/// cima — ver a tabela do cabeçalho).
const ANTES_NS: u64 = 72;

/// O mesmo pela rota de HOJE (`0,0316 µs`).
const HOJE_NS: u64 = 32;

// ⭐⭐⭐ **A JANELA DA POPULAÇÃO, e ela é ERRO DE COMPILAÇÃO nas DUAS metades.**
//
// Uma cena grande de mais deixa de correr a `60` **dos dois lados** e o dono vê duas corridas
// lentas; uma pequena de mais corre a `60` dos dois e ele vê duas corridas iguais. **As duas
// falhas leem-se como *«a cura não faz nada»***, e é por isso que as duas estão presas aqui.
//
// ⛔ Ela **não** pode viver num teste: um `assert!` sobre constantes é dobrado pelo compilador
// antes de correr, e o clippy di-lo em voz alta (a lei que a cena `=124` já escreve).
//
// ⛔⛔ **E ela é ANÓNIMA (`const _`) e não um `const` com nome, porque a 1.ª redacção tinha nome e
// o compilador acusou `constant QUADRO_NS is never used`:** um item com nome que ninguém
// referencia é **morto** para o lint, e os usos DENTRO de um item morto não contam como usos —
// logo a cerca compilava, mordia, e deixava um aviso a apontar para a constante medida como se
// ela fosse lixo. *Um `const _` não pode ser referenciado por construção, logo é sempre vivo.*
//
// ⚠️ **Aritmética inteira de propósito** — comparar `f32` em contexto `const` é terreno que esta
// casa não precisa de pisar para prender dois números medidos.
const _: () = assert!(
    ESTRELAS * HOJE_NS <= 70 * QUADRO_NS / 100,
    "a cena nao cabe em 70% de um quadro pela rota de HOJE -- as duas corridas sairiam \
     lentas e a cura leria-se como inutil"
);
const _: () = assert!(
    ESTRELAS * ANTES_NS >= 120 * QUADRO_NS / 100,
    "a cena cabe num quadro pela rota ANTIGA -- as duas corridas dariam 60 fps e a cena \
     nao mostraria nada"
);

/// **O índice da estrela no `kind` do `source.shape`, derivado do PRÓPRIO enum.**
///
/// ⛔⛔ **E não do rótulo, que é o defeito que esta linha pagou em 2026-09-20:** os `KIND_LABELS`
/// passaram a ser chaves de i18n quando a fronteira dos motores fechou, e uma procura por `"Star"`
/// devolvia `None` — a auditoria inteira mediu um CÍRCULO julgando medir uma estrela. A porta que
/// as outras cenas usam (`sim_demo::indice_de`) já resolve a chave pelo idioma inglês e está
/// curada; **esta cena não passa por rótulo nenhum**, que é a cura mais forte: o `ALL_KINDS` está
/// alinhado ao `KIND_LABELS` **por gate** na crate do nó.
///
/// ⚠️ Ela falha **alto**: uma forma que saia do enum tem de parar a cena, não escolhê-la em
/// silêncio.
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
    // ⚠️ O registo entra na assinatura porque o roteador o passa a todas as cenas — e aqui ele
    // serve de guarda: uma cena que monte com um nó que não existe cozinha zero e desenha nada.
    for tipo in ["motion.grid", "motion.duplicator", "source.shape", "motion.output"] {
        reg.manifests()
            .find(|m| m.id == ph2d_nodegraph::node::NodeTypeId::of(tipo))?;
    }
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };

    // ── AS POSIÇÕES.
    let grade = no(g, "motion.grid", 0.0, 0.0);
    g.set_param(grade, "rows", LADO);
    g.set_param(grade, "cols", LADO);
    g.set_param(grade, "gap_x", VAO);
    g.set_param(grade, "gap_y", VAO);

    // ── A FORMA: uma estrela, e só uma. É ela que o carimbo prepara uma vez.
    let forma = no(g, "source.shape", 0.0, 220.0);
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, indice_da_estrela());
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, TAMANHO);

    // ── O CARIMBO. ⚠️ A forma na porta `0`, os pontos na `1` — a ordem que o manifesto do
    // duplicador declara, e que um censo do roteador confere em toda cena.
    let dup = no(g, "motion.duplicator", 240.0, 110.0);
    for (de, porta) in [(forma, 0u16), (grade, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }
    let saida = no(g, "motion.output", 460.0, 110.0);
    g.connect(Edge {
        from: (dup, 0),
        to: (saida, 0),
        delayed: false,
    })
    .ok()?;
    Some(vec![saida])
}

/// **O roteiro que o dono segue.** ⚠️ Cada passo nomeia o que aparece NA TELA (`CLAUDE.md` §0.8),
/// e o *«deu errado se»* é a metade que só cabe aqui.
///
/// ⛔ **Esta cena não pousa legenda no canvas de propósito:** a legenda desta casa é feita de
/// PARES (uma ficha por metade, em lados opostos — há censo a exigi-lo), e isto é UMA coisa só.
/// O que ela tem para dizer é um número que já está na tela: a barra de baixo.
pub(super) fn announce() {
    let n = ESTRELAS;
    let antes = (ESTRELAS * ANTES_NS) as f64 / 1e6;
    let hoje = (ESTRELAS * HOJE_NS) as f64 / 1e6;
    eprintln!(
        "\n[estrelas] UM CAMPO DE {n} ESTRELAS ({LADO_N} x {LADO_N}) — a cena do seu report\n\
         («usando shape star fps cai»). O `Grid` poe as posicoes e o `Duplicator` veste cada\n\
         uma com a MESMA estrela.\n\
         \n\
         (1) Olhe a BARRA DE BAIXO do ecra. Ela diz algo como `60 fps · 16.7 ms · 300 raw`.\n    \
         O 1.o numero e' a fluidez; o 3.o (`raw`) e' a FOLGA — quanto maior, mais sobra.\n\
         (2) Arraste o fundo (botao do meio) para passear pelo campo: tem de andar LISO.\n    \
         O ecra mostra um pedaco — as {n} estrelas existem e sao todas desenhadas.\n\
         (3) Feche o app e corra o MESMO comando com `PH2D_CARIMBO_PREPARADO=0` a' frente:\n    \
         e' o caminho ANTIGO, e a mesma cena passa a engasgar.\n\
         (4) Compare os dois numeros. A imagem e' a MESMA, ponto por ponto — so' o tempo muda.\n\
         \n\
         O que esta maquina mediu para este tamanho: ANTIGO ~{antes:.0} ms por quadro (abaixo de\n\
         60 fps) contra HOJE ~{hoje:.0} ms (60 fps com folga).\n\
         \n\
         DEU ERRADO se: o campo nao aparecer; se as duas corridas derem o MESMO numero;\n\
         ou se a imagem for DIFERENTE entre as duas.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_carimbo_demo_tests.rs"]
mod tests;
