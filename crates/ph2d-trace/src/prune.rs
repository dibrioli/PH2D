//! ⭐⭐⭐ **A PODA DOS TOCOS** — arcos que morrem num vértice REGULAR.
//!
//! # ⛔ O defeito, medido (2026-08-23)
//!
//! O passeio pára ao bater numa separatriz já traçada
//! ([`crate::walk::Walker::walk_from`]), e isso deixa um **nó em T num vértice
//! qualquer**. Medido nas quatro fixturas:
//!
//! | peça | nós do traçado | em singularidade | ⛔ **fora dela** | irregulares na saída | ⭐ o oráculo |
//! |---|---|---|---|---|---|
//! | esfera **lisa** | 29 | 7 | ⛔ **22** | **18** | **8** (o piso) |
//! | enrugada | 24 | 5 | ⛔ **19** | 14 | **8** (o piso) |
//! | orelha | 28 | 6 | ⛔ **22** | 18 | 12 |
//! | gancho | 41 | 9 | ⛔ **32** | 26 | 14 |
//!
//! ⚠️ **O campo está ilibado:** nas duas esferas ele tem `8` singularidades, tal
//! como o dele — o mínimo de Poincaré–Hopf. ⇒ *dez dos nossos irregulares são
//! fabricados entre o traçado e o preenchimento, e a única razão legítima para um
//! canto existir é uma singularidade.*
//!
//! # ⚠️ E cada toco paga-se DUAS vezes
//!
//! Um nó a mais parte um **lado** em vários **arcos** — `37 %` dos lados da esfera
//! lisa, com um lado da orelha a levar **seis**. E é precisamente aí que nasce o
//! outro defeito desta linha: *dentro de um arco a reamostragem por `τ` é
//! proporcional, logo não há desvio; a discordância conforme entre lados opostos
//! aparece quando um lado tem vários arcos com densidades diferentes*
//! ([`ph2d_quadfill::rectangle`]). **Uma causa, dois sintomas.**
//!
//! # A lei
//!
//! Um arco é **candidato** quando pelo menos uma das pontas dele está num vértice
//! **regular**. Remover as arestas dele funde os dois patches que ele separava.
//!
//! ⛔ **A remoção só é adoptada se passar em SEIS guardas**, e cada uma entrou porque a
//! ausência dela foi medida a partir:
//!
//! 1. **A topologia não piora** — a distância entre `V − E + F` do complexo e o `χ`
//!    da peça, a mesma régua da limpeza que já existia. *Uma cura que muda o género
//!    devolve uma malha com 100 % de quads e a forma errada.*
//! 2. **Nenhum patch degenerado nasce** — [`PatchLayout::degenerate`].
//! 3. ⭐ **O número de nós desce ESTRITAMENTE.** É o que garante terminação sem um
//!    teto de rondas inventado: cada passo aceite consome pelo menos um nó, e os nós
//!    são finitos. *A cerca do teto de rondas já foi medida e rejeitada nesta crate —
//!    a paciência é do fenómeno, não uma constante.*
//! 4. **Nenhuma singularidade perde grau** — [`singular_degrees`]. ⛔ *Sem ela a poda
//!    levou as quatro fixturas a **2 patches**.*
//! 5. **Uma singularidade que era nó continua a ser nó** — ⛔ *sem ela a poda
//!    **agravava** o segundo defeito enquanto curava o primeiro.*
//! 6. **O F4 ainda resolve** — [`still_quantizes`]. ⛔ *Sem ela a orelha saía
//!    `Infeasible`, e nenhum predicado local o soube prever.*
//!
//! ⚠️ **A ordem dos candidatos é determinística e geométrica** (pontas regulares
//! primeiro, depois o arco mais curto, e o desempate pela chave da primeira aresta) —
//! ⛔ **nunca o índice do arco**, que muda a cada decomposição e faria a mesma peça
//! sair diferente conforme o histórico.
//!
//! ⚠️ **Clean-room:** a simplificação de um grafo de separatrizes por remoção guardada
//! é descrita em QuadWild 2021 §6 e na literatura do *motorcycle graph*. Nenhuma linha
//! vem de fonte GPL — ver ADR-0162.

use std::collections::BTreeSet;

use ph2d_mesh::Mesh;

use crate::TraceReport;
use crate::patches::{PatchLayout, decompose};
use crate::walk::Walls;

/// ⛔⛔⛔ **DESLIGADO — MEDIDO E REJEITADO** (2026-08-23), e a rejeição é o achado mais
/// importante desta jornada: ela **confirma o diagnóstico e nomeia a ordem da obra**.
///
/// ⚠️ **Com `false` o traçado é byte-idêntico ao de sempre**, e é assim que a tabela
/// tem um controlo.
///
/// # ⭐⭐⭐ Ela faz EXACTAMENTE o que o diagnóstico previa
///
/// | | sem poda | ⭐ com poda | o oráculo | o piso |
/// |---|---|---|---|---|
/// | patches (esfera lisa) | 16 | **6** | 8 | — |
/// | nós fora de singularidade | 22 | **4** | — | 0 |
/// | **irregulares** esfera lisa | 18 | ⭐ **9** | 8 | **8** |
/// | **irregulares** enrugada | 14 | ⭐ **9** | 8 | 8 |
/// | **irregulares** orelha | 18 | ⭐⭐ **12** | **12** | 8 |
/// | **irregulares** gancho | 26 | ⭐ **19** | 14 | 8 |
///
/// ⭐⭐ **A orelha passa a empatar com o oráculo.** ⇒ *a cadeia causal está fechada:
/// os nós inventados eram, de facto, os irregulares a mais.*
///
/// # ⛔⛔⛔ E a GEOMETRIA colapsa, na mesma peça
///
/// | esfera lisa, `d = 0,55` | sem poda | ⛔ com poda |
/// |---|---|---|
/// | aspecto p50 | `1,26` | ⛔ **`2,10`** |
/// | enviesamento p50 | `18°` | ⛔ **`38°`** |
/// | faces `> 60°` | 141 | ⛔ **1 442** |
/// | dobras | **0** | ⛔ **29** |
///
/// ⛔ Mais: `the_layout_we_produce_is_quantized_with_proof` perde a prova de ótimo na
/// esfera 48×72 (o F4 resolve, mas o orçamento deixa de chegar para **provar**), e o
/// `no_face_folds_back_on_itself` sobe para `1,6 %` e `7,4 %`.
///
/// # ⭐⭐⭐ O que a queda ENSINA, e é o valor da rejeição
///
/// ⛔ **Não é o mapa** (ligar o [`ph2d_quadfill::rectangle`] em cima da poda dá
/// `38° → 36°`) e ⛔ **não é a forma do domínio** (ligar o `PROPORTIONAL_DOMAIN` dá
/// `38° → 38°`, idêntico). ⇒ **é o TAMANHO do patch.** Um achatamento de Tutte de um
/// terço de esfera sobre um polígono unitário está distorcido de forma que nenhum
/// operador e nenhum polígono corrigem.
///
/// ⚠️ **E o oráculo enche 8 patches numa esfera com `6°`.** ⇒ *o nosso F5 não é viável
/// nessa escala*, e a razão é a mesma que o [`ph2d_quadfill::rectangle`] já tinha
/// nomeado por outro caminho: ele resolve **cada patch em separado** contra um domínio
/// plano, enquanto a referência tem **uma parametrização global** de onde cada patch
/// herda um `(u,v)` consistente.
///
/// ⇒ ⭐⭐⭐ **A ORDEM DA OBRA está medida:** o preenchimento tem de aguentar um patch
/// grande **antes** de o traçado poder emitir poucos. Ligar esta constante antes disso
/// troca um defeito por outro maior.
pub const PRUNE_STEMS: bool = false;

/// Quantas rondas de poda, no máximo.
///
/// ⚠️ **Ele NÃO é o critério de paragem** — a guarda 3 (o número de nós desce
/// estritamente) é, e ela termina sozinha. Isto é só a rede contra um bug que faça a
/// contagem oscilar, e atingi-lo é um defeito a reportar, não um resultado.
const MAX_ROUNDS: usize = 256;

/// **OS NÓS do traçado** — as pontas de todos os arcos.
fn nodes(layout: &PatchLayout) -> BTreeSet<u32> {
    layout
        .arc_chain
        .iter()
        .filter_map(|c| Some((*c.first()?, *c.last()?)))
        .flat_map(|(a, b)| [a, b])
        .collect()
}

/// A chave de ordenação de um arco candidato — ver o aviso sobre determinismo.
fn key(layout: &PatchLayout, a: usize, index: &[i32]) -> (i32, u32, (u32, u32)) {
    let chain = &layout.arc_chain[a];
    let regular = chain
        .first()
        .into_iter()
        .chain(chain.last())
        .filter(|&&v| index.get(v as usize).copied().unwrap_or(0) == 0)
        .count();
    // Duas pontas regulares primeiro (o arco que o campo menos pediu), depois o mais
    // curto, e a chave da primeira aresta como desempate estável.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let len = (layout.arc_length[a] * 1024.0) as u32;
    let first = layout.arc_edges[a].iter().next().copied().unwrap_or((0, 0));
    #[allow(clippy::cast_possible_wrap)]
    (-(regular as i32), len, first)
}

/// A topologia e os degenerados — a mesma leitura que a limpeza usa.
fn health(layout: &PatchLayout) -> (i64, usize) {
    (
        (layout.complex_euler() - layout.mesh_chi).abs(),
        layout.degenerate().len(),
    )
}

/// ⭐⭐⭐ **A GUARDA QUE FALTAVA, e a falta dela apagou o traçado inteiro.**
///
/// ⛔ **Medido na primeira corrida (2026-08-23):** com as três guardas de cima e mais
/// nenhuma, a poda levou as quatro fixturas a **2 patches** e enterrou **6 das 8
/// singularidades DENTRO deles**. ⚠️ E as três guardas ficaram todas contentes — uma
/// esfera cortada por **um** laço fechado dá dois discos, `χ = 2`, zero degenerados, e
/// o número de nós desce a cada passo. *Um critério que só olha para a topologia
/// aprova a decomposição trivial.*
///
/// ⭐ **A guarda certa vem do CAMPO, e não é heurística:** uma singularidade de índice
/// `k` emite `4 − k` separatrizes — é a mesma lei do [`crate::ring::Ring::seeds`]. Logo
/// o grau dela no traçado tem de se manter.
///
/// ⚠️ **A exigência é «não piorar», não «atingir `4 − k`»:** o traçado já entrega
/// singularidades abaixo da conta (uma separatriz descartada por não fechar), e exigir
/// o ideal recusaria toda poda numa peça que já nasceu com dívida. *A cura não é o
/// sítio para cobrar uma dívida de outra fase.*
fn singular_degrees(walls: &Walls, index: &[i32]) -> Vec<(u32, usize)> {
    let branching = walls.branching();
    index
        .iter()
        .enumerate()
        .filter(|(_, k)| **k != 0)
        .map(|(v, k)| {
            let v = u32::try_from(v).unwrap_or(u32::MAX);
            let want = usize::try_from(4 - k).unwrap_or(0);
            (v, branching.get(&v).copied().unwrap_or(0).min(want))
        })
        .collect()
}

/// ⛔⛔⛔ **GUARDA 6: NENHUM ARCO PODE TER O MESMO PATCH DOS DOIS LADOS.**
///
/// # A medição que a encontrou
///
/// Com as guardas 1 a 5 a poda deu, em três fixturas, layouts que a cadeia inteira
/// aceitou — e na **orelha** o F4 devolveu **`Infeasible`**, que nesta crate significa
/// *«nenhuma quantização regular existe»* e **não** «acabou o orçamento». ⚠️ *Uma cura
/// que entrega ao passo seguinte um problema provadamente impossível não é uma cura.*
///
/// ⭐ **E o número dizia qual era, antes de qualquer depuração:**
///
/// | fixtura podada | valências | `Σ lados` | o F4 |
/// |---|---|---|---|
/// | esfera lisa | `{3:4, 4:2}` | `20` (par) | ⭐ resolve |
/// | enrugada | `{3:4, 6:1}` | `18` (par) | ⭐ resolve |
/// | gancho | `{3:8, 4:2, 8:1}` | `40` (par) | ⭐ resolve |
/// | ⛔ **orelha** | `{3:2, 4:2, 5:1}` | **`19` (ÍMPAR)** | ⛔ **`Infeasible`** |
///
/// **Cada lado é partilhado por dois patches**, logo `Σ lados` é par por construção
/// numa decomposição sã. Um total ímpar quer dizer que **algum arco tem o mesmo patch
/// dos dois lados** — a poda fundiu dois vizinhos e o arco que sobrou passou a
/// encostar a região a si própria. ⇒ o F4 não tem como exprimir `L_i = e_{i−1} + e_{i+1}`
/// para esse lado, e a recusa dele é correcta.
///
/// ⛔⛔ **E ESTA EXPLICAÇÃO ESTAVA ERRADA.** Construí a guarda sobre a auto-adjacência e
/// a orelha **continuou** a sair `Infeasible`, ainda com `Σ lados = 19`. ⚠️ *O argumento
/// «cada lado é partilhado por dois patches, logo a soma é par» não se aplica aqui*: um
/// **lado** é um agrupamento **por-patch** de arcos, e os dois patches que confinam podem
/// agrupar a mesma fronteira em números de lados diferentes. ⇒ a soma **não tem** de ser
/// par, e a correlação perfeita nas quatro fixturas era coincidência de quatro amostras.
/// *Uma tabela de quatro linhas com correlação perfeita continua a ser quatro linhas.*
///
/// ⭐⭐⭐ **A guarda que FICA é a única honesta: perguntar à FASE SEGUINTE.**
///
/// ⚠️ **O alvo da sonda é FINO de propósito.** A viabilidade de `L_i = e_{i−1} + e_{i+1}`
/// com `e ≥ 1` depende das **razões** entre os lados, que não mudam com o alvo; o que o
/// alvo muda é a folga do `≥ 1`. Um alvo fino dá contagens grandes e folga máxima, logo o
/// que ele reprova é **estrutural** — e é só isso que a poda tem de evitar. *Reprovar por
/// falta de folga seria julgar a poda por uma densidade que o artista ainda não escolheu.*
///
/// ⚠️ **O orçamento é pequeno de propósito**: aqui a pergunta é *«existe?»* e não *«qual
/// é o ótimo?»* — e esta guarda corre uma vez por candidato.
fn still_quantizes(layout: &PatchLayout, target: f32) -> bool {
    layout.to_layout(target).is_ok_and(|spec| {
        ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(64, 128)).is_ok()
    })
}

/// **O ALVO DA SONDA DE VIABILIDADE** — fino, ver [`still_quantizes`].
///
/// A mediana das arestas da malha: ela existe, é da peça, e não é uma constante.
fn probe_target(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let mut lens: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            lens.push(
                (a[0] - b[0])
                    .mul_add(
                        a[0] - b[0],
                        (a[1] - b[1]).mul_add(a[1] - b[1], (a[2] - b[2]).powi(2)),
                    )
                    .sqrt(),
            );
        }
    }
    lens.sort_by(f32::total_cmp);
    lens.get(lens.len() / 2).copied().unwrap_or(1.0).max(1.0e-6)
}

/// Nenhuma singularidade perdeu grau abaixo do que já tinha — ver [`singular_degrees`].
fn keeps_the_field(before: &[(u32, usize)], after: &Walls, index: &[i32]) -> bool {
    let now = singular_degrees(after, index);
    before
        .iter()
        .zip(&now)
        .all(|(&(_, was), &(_, is))| is >= was)
}

/// ⭐⭐⭐ **PODA os arcos que morrem em vértice regular** e devolve o layout novo.
///
/// `index` é o índice do campo por vértice ([`ph2d_crossfield::vertex_index`]): `0` é
/// regular. `walls` é consumido e devolvido já podado, para o chamador ficar com a
/// parede que corresponde ao layout.
///
/// ⚠️ **Não recebe teto de qualidade nenhum.** Ela pára quando nenhuma remoção passa
/// nas três guardas — ver o doc do módulo.
#[must_use]
pub fn prune_stems(
    mesh: &Mesh,
    mut walls: Walls,
    base: &TraceReport,
    index: &[i32],
) -> (Walls, PatchLayout, usize) {
    let mut layout = decompose(mesh, &walls, base.clone());
    let probe = probe_target(mesh);
    let mut pruned = 0usize;
    for _ in 0..MAX_ROUNDS {
        let before = health(&layout);
        let all_nodes = nodes(&layout);
        let n_before = all_nodes.len();
        let sing_nodes_before: BTreeSet<u32> = all_nodes
            .into_iter()
            .filter(|&v| index.get(v as usize).copied().unwrap_or(0) != 0)
            .collect();
        let sing_before = singular_degrees(&walls, index);
        // Os candidatos: arco com pelo menos uma ponta em vértice regular.
        let mut cands: Vec<usize> = (0..layout.arc_chain.len())
            .filter(|&a| {
                let c = &layout.arc_chain[a];
                c.first()
                    .into_iter()
                    .chain(c.last())
                    .any(|&v| index.get(v as usize).copied().unwrap_or(0) == 0)
            })
            .collect();
        cands.sort_by_key(|&a| key(&layout, a, index));

        let mut taken: Option<(Walls, PatchLayout)> = None;
        for a in cands {
            let mut trial = walls.clone();
            let mut touched = false;
            for e in &layout.arc_edges[a] {
                touched |= trial.edges.remove(e);
            }
            if !touched {
                continue;
            }
            // ⛔ **A guarda do CAMPO corre PRIMEIRO porque é a mais barata** (só lê o
            // grau das arestas de parede) **e é a que apanha o caso catastrófico** —
            // ver [`singular_degrees`].
            if !keeps_the_field(&sing_before, &trial, index) {
                continue;
            }
            let next = decompose(mesh, &trial, base.clone());
            let after = health(&next);
            if after.0 > before.0 || after.1 > before.1 {
                continue;
            }
            // ⛔ **GUARDA 6** — ver [`still_quantizes`]. Ela é a MAIS CARA e corre por
            // último de propósito: as cinco de cima já filtraram quase tudo.
            if !still_quantizes(&next, probe) {
                continue;
            }
            let n_next = nodes(&next);
            // ⛔⛔ **GUARDA 5: uma singularidade que era NÓ continua a ser nó.**
            //
            // ⚠️ **Ter grau não é ser nó**, e a diferença mordeu: podar um nó em T funde
            // os dois arcos que ele separava num só, e uma singularidade que estava na
            // junta passa a ficar **no MEIO** do arco fundido. Ela mantém o grau — a
            // guarda 4 fica contente — e deixa de ser canto de patch nenhum. ⇒ *o
            // canto deixa de estar onde a grade tem de virar*, que é a definição do
            // defeito que esta poda existe para curar. Medido: sem esta linha a
            // orelha ia de `2` para `3` singularidades sem nó, e a enrugada de `3`
            // para `4` — a poda **agravava** o segundo defeito enquanto curava o
            // primeiro.
            if !sing_nodes_before.is_subset(&n_next) {
                continue;
            }
            if n_next.len() >= n_before {
                continue;
            }
            taken = Some((trial, next));
            break;
        }
        let Some((w, l)) = taken else {
            break;
        };
        walls = w;
        layout = l;
        pruned += 1;
    }
    (walls, layout, pruned)
}
