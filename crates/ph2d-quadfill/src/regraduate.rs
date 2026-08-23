//! ⭐⭐⭐ **A RE-GRADUAÇÃO DO ARCO** — onde os pontos de subdivisão caem ao longo de um
//! arco deixa de ser comprimento na PEÇA e passa a ser comprimento no DOMÍNIO.
//!
//! # ⛔ O único suspeito que sobrou, e ele está medido
//!
//! Quatro achatamentos foram construídos e medidos — valor médio, cotangente,
//! quadrilátero extremal ([`crate::rectangle`]) e LSCM ([`crate::lscm`]) — e nenhum
//! move o enviesamento. ⭐⭐ **O LSCM até o PIORA** (`18° → 28°`) enquanto leva o erro
//! conforme de `4,32` a `1,01`, e é essa medição que nomeia o culpado:
//!
//! | esfera lisa, `d = 0,55` | Tutte | LSCM convergido |
//! |---|---|---|
//! | erro conforme | `4,32` | ⭐ `1,01` |
//! | **enviesamento do DOMÍNIO**, rectângulos | `1,0°` | ⛔ **`21,4°`** |
//! | **enviesamento do DOMÍNIO**, leques | `18,7°` | ⛔ **`50,8°`** |
//!
//! ⇒ *num domínio conforme, os pontos de bordo — postos por **comprimento de arco** —
//! caem em posições muito desiguais, e a grade de Coons entre eles nasce torta.* O
//! Tutte pregado **mascarava** isso; o mapa bom deixou de o esconder.
//!
//! # ⭐ A lei
//!
//! O ponto `k` do lado 0 e o ponto `k` do lado 2 têm de estar na **mesma fracção
//! conforme** para a linha de grade que os une nascer recta. Cada patch sabe qual é
//! essa fracção — é a distância acumulada no domínio dele
//! ([`crate::param::PatchParam::side_alpha`]). ⇒ **o arco recebe a MÉDIA das duas
//! propostas** dos patches que o partilham.
//!
//! ⚠️ **O TOTAL de cada arco NÃO muda, e a restrição é deliberada.** O `arc_tau` é lido
//! por três sítios — a reamostragem, o pino da fronteira no achatamento e o `uv` dos
//! pontos de saída — e o **último valor** dele é o peso do arco perante os irmãos do
//! mesmo lado **e** perante o alvo da quantização. Mexer no total mudaria quantos
//! segmentos o F4 dá a cada arco, que é outra experiência. ⇒ *muda-se a FORMA de dentro
//! do arco e mais nada.*
//!
//! ⭐⭐ **E é por isso que uma régua só chega para os três sítios:** eles todos leem
//! `arc_tau`, logo re-graduá-lo re-gradua os três **por construção**. *Duas réguas aqui
//! rasgariam a malha ao longo de toda fronteira de patch, com um erro pequeno demais
//! para se ver num render.*

use ph2d_mesh::Mesh;
use ph2d_quantize::Quantization;
use ph2d_trace::PatchLayout;

use crate::param::PatchParam;

/// ⭐⭐⭐ **A RE-GRADUAÇÃO SHIPA?** — ver a tabela do [`crate::lscm`] para o porquê de
/// ela existir, e o `PLAN.md` para o que ela mediu.
///
/// ⚠️ **Com `false` a cadeia é byte-idêntica à de sempre**, e é assim que a tabela tem
/// um controlo.
/// ⛔⛔ **DESLIGADO — MEDIDO E REJEITADO** (2026-08-23), e a rejeição **deriva** a obra
/// seguinte em vez de a adivinhar.
///
/// # ⭐ Ela corre: `42 de 42` arcos, zero desistências
///
/// ⚠️ *Chegar aqui custou três «byte-idêntico ao controlo»* — a circularidade do Tutte,
/// um `?` a abortar a função, e um `return` que não preenchia o
/// [`crate::param::PatchParam::side_alpha`]. O que os separou foi a coluna do **motivo**
/// (`Regraduation::gave_up`), não mais raciocínio.
///
/// # ⛔ E não move o número — nem sozinha nem com o domínio conforme
///
/// Esfera lisa, `d = 0,55`:
///
/// | | controlo | só re-graduação | só LSCM | ⭐ **as duas** |
/// |---|---|---|---|---|
/// | erro conforme | `4,32` | `4,49` | `1,01` | `1,01` |
/// | enviesamento p50 | **`18°`** | `19°` | `28°` | ⛔ `28°` |
/// | **domínio dos rectângulos** | `1,0°` | `1,0°` | `21,4°` | ⛔ **`21,3°`** |
/// | **domínio dos leques** | `18,7°` | `18,7°` | `50,8°` | ⛔ **`48,2°`** |
/// | dobras | **`0`** | `0` | `68` | ⛔ `79` |
///
/// ⭐⭐⭐ **A linha do domínio é a que fala: `21,4° → 21,3°`. Zero.** Alinhar cada arco
/// com o vizinho dele **não endireita a grade**.
///
/// # ⭐⭐⭐ E o porquê DERIVA a obra seguinte
///
/// ⛔ **Esta cura emparelha o lado errado.** O enviesamento do domínio de um patch nasce
/// do desacordo entre os **lados OPOSTOS dele** — o ponto `k` do lado 0 contra o ponto
/// `k` do lado 2. A re-graduação alinha cada arco com o **vizinho do outro lado da
/// costura**, que é outro par: os lados 0 e 2 do mesmo patch são arcos diferentes, com
/// vizinhos diferentes.
///
/// ⚠️⚠️ **E os dois pedidos não se conseguem satisfazer ao mesmo tempo, localmente.**
/// A distribuição do lado 0 tem de servir *o lado 2 do meu patch* **e** *o patch do
/// outro lado do arco*. Cada arco está preso nos dois, e a cadeia de dependências
/// atravessa a peça inteira.
///
/// ⇒ ⭐⭐⭐ **É exactamente isso que uma parametrização GLOBAL resolve, e a razão deixou
/// de ser uma citação da referência:** ela não faz a média de duas propostas em
/// desacordo — ela **impõe o acordo desde o início**, pela função de transição através
/// da costura, e resolve todos os patches de uma vez.
///
/// # ⛔⛔⛔ E o PONTO FIXO foi construído e medido — a afirmação anterior estava errada
///
/// Esta nota dizia *«a média local não converge para isso porque o ponto fixo dela nem
/// sequer contrai (`21,4 → 21,3`)»*. ⚠️ **`21,4 → 21,3` é UM TERMO**, e o laço corria
/// uma vez só — *uma afirmação sobre convergência tirada de uma amostra que não a pode
/// exibir.* Ver [`conformal_arc_tau`] e a sonda `does_the_fixed_point_contract`.
///
/// ⭐ **Medido: ele contrai, e por exactamente `½` por ronda.**
///
/// | ronda | 1 | 2 | 3 | 4 | 5 | 6 |
/// |---|---|---|---|---|---|---|
/// | moveu (unidades de `τ`) | `0,185` | `0,060` | `0,030` | `0,015` | `0,0075` | `0,0038` |
/// | razão | — | `0,33` | **`0,500`** | **`0,500`** | **`0,500`** | **`0,499`** |
///
/// ⛔⛔ **E o ponto fixo NÃO endireita a grade** (esfera `24×36`, LSCM ligado — ⚠️ *não*
/// comparável com os `21,4°` das tabelas acima, que são de uma `96×144`):
///
/// | | domínio rect | domínio leque | enviesamento p50 | dobras |
/// |---|---|---|---|---|
/// | sem re-graduação | **`22,5°`** | `66,9°` | `54°` | **`41`** |
/// | 1 ronda | `24,4°` | `62,0°` | `55°` | `51` |
/// | ⭐ **8 rondas (o ponto fixo)** | ⛔ **`24,4°`** | ⛔ **`62,0°`** | `54°` | `47` |
///
/// ⭐⭐⭐ **`1` ronda e `8` dão o MESMO número** — o ponto fixo é alcançado à primeira e
/// as sete seguintes não movem nada. E contra o controlo ele **piora** o rectângulo
/// (`22,5 → 24,4`) e **melhora** o leque (`66,9 → 62,0`): *move a discordância de sítio
/// em vez de a remover*, que é a assinatura de uma cura que não é a cura.
///
/// ⇒ **A conclusão de cima fica de pé com prova muito mais forte:** os dois pedidos não
/// se satisfazem localmente — não porque o esquema não convirja, mas porque **o ponto
/// para onde ele converge não é o que se quer**.
///
/// ⚠️ **Com `false` a cadeia é byte-idêntica à de sempre**, e é assim que a tabela tem
/// um controlo.
///
/// ⭐ **O que ela já ensinou, mesmo por correr:** a primeira versão usava o achatamento
/// de **Tutte** para tirar a «fracção conforme», e isso é **circular** — o Tutte prega a
/// fronteira *pela fracção de `τ`*, logo a resposta era o `τ` de volta e o resultado saiu
/// byte-idêntico ao controlo. ⇒ *só um achatamento de fronteira LIVRE tem opinião
/// própria*, e é por isso que o [`crate::lscm`] — rejeitado como mapa — é obrigatório
/// aqui.
pub(crate) const REGRADUATE: bool = false;

/// ⭐⭐⭐ **QUANTAS RONDAS DO PONTO FIXO** — ver [`conformal_arc_tau`].
///
/// ⭐ **`8` porque é onde o ponto fixo já está** — medido: a contracção é `½` por ronda
/// e o número do produto é idêntico a `1` ronda. ⇒ *se alguém voltar a ligar o
/// [`REGRADUATE`], recebe a resposta CONVERGIDA e não uma varredura arbitrária.*
///
/// ⚠️ **Com [`REGRADUATE`] a `false` esta constante é inerte** — não custa nada shipar
/// a semântica certa.
///
/// ⛔ A versão que foi medida e rejeitada corria **uma** varredura, e a nota da
/// rejeição concluía daí que o ponto fixo «não contrai». *Uma afirmação sobre
/// convergência tirada de um termo.* A sonda `does_the_fixed_point_contract` imprime a
/// sequência inteira e o número do produto ao lado.
pub(crate) const ROUNDS: usize = 8;

/// ⭐⭐⭐ **O QUE A RE-GRADUAÇÃO DEVOLVE, com o motivo de cada desistência.**
///
/// ⛔ **A terceira vez que o mesmo sintoma mordeu neste ficheiro** — «byte-idêntico ao
/// controlo» — foi a que ensinou a lição: um numerador (`5 de 42`) diz **que** ela
/// desiste e não **onde**. *A cura é a mesma das outras duas: acrescentar a coluna que
/// falta.*
pub(crate) struct Regraduation {
    /// O `arc_tau` novo.
    pub(crate) tau: Vec<Vec<f32>>,
    /// Quantos arcos de facto mudaram.
    pub(crate) changed: usize,
    /// ⭐⭐⭐ **O maior deslocamento de um ponto entre a penúltima ronda e a última**,
    /// em unidades de `τ` — ver [`conformal_arc_tau`]. É a coluna que distingue
    /// *contrai* de *empata* de *diverge*, e ela não existia.
    pub(crate) moved: f32,
    /// **Por que desistiu**, por ordem: `0` patch sem achatamento · `1` lado sem alfa ·
    /// `2` comprimentos discordam · `3` `span` nulo · `4` balde discorda do arco.
    ///
    /// ⚠️ **Lido só pelo gate**, e é de propósito: a cadeia não precisa do motivo, mas
    /// quem voltar a ligar esta constante precisa — foi esta coluna que separou três
    /// «byte-idêntico ao controlo» com causas diferentes. *Apagá-la porque o produto não
    /// a lê é apagar a única coisa que os distinguiu.*
    #[allow(dead_code)]
    pub(crate) gave_up: [usize; 5],
}

/// ⭐⭐ **O `arc_tau` NOVO** — mesmo total por arco, distribuição interna vinda do
/// achatamento.
///
/// ⚠️ **`None` é uma resposta e não uma falha** — a cadeia segue com o `τ` de sempre.
///
/// ⛔⛔ **NENHUM `?` de dentro do laço aborta a função**, e a regra custou uma medição:
/// um `param.side_alpha.get(i)?` a falhar num patch devolvia `None` **para a cadeia
/// inteira**, e o chamador caía no `τ` de sempre — *byte-idêntico ao controlo, sem uma
/// palavra*. ⇒ o que falha num patch **salta esse patch** e a contagem devolvida diz
/// quantos arcos de facto mudaram. *Um recuo silencioso é indistinguível de uma cura
/// que não funciona.*
///
/// ⛔ **O achatamento aqui usa o `τ` VELHO para pregar a fronteira**, e isso é uma
/// aproximação declarada: a re-graduação é um ponto fixo, e isto é a primeira iteração
/// dele. *Correr uma segunda seria barato; medir se a primeira vale é mais barato ainda,
/// e é a ordem certa.*
fn one_sweep(
    indexed: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
    // ⚠️ **`from`, e não `tau`:** existe um `tau` LOCAL dentro do laço dos lados, e um
    // parâmetro com o mesmo nome fica ensombrado lá dentro **sem erro de tipo óbvio** —
    // ele compila até tocar num método que só o outro tem.
    from: &[Vec<f32>],
) -> Option<Regraduation> {
    let arcs = layout.arc_chain.len();
    // Por arco, as propostas acumuladas e quantas foram.
    let mut acc: Vec<Vec<f32>> = from.iter().map(|t| vec![0.0f32; t.len()]).collect();
    let mut votes: Vec<usize> = vec![0; arcs];
    let mut gave_up = [0usize; 5];

    // As faces de cada patch, uma passagem só — a mesma lei do [`crate::stitch`].
    let mut patch_faces: Vec<Vec<u32>> = vec![Vec::new(); layout.side_arcs.len()];
    for (f, &pp) in layout.face_patch.iter().enumerate() {
        if let Some(slot) = patch_faces.get_mut(pp as usize) {
            slot.push(u32::try_from(f).unwrap_or(0));
        }
    }

    for (p, sides) in layout.side_arcs.iter().enumerate() {
        let n = sides.len();
        // A fronteira do patch por lado, e o `τ` dela — a MESMA construção do
        // [`crate::stitch`], incluindo o espelho de um arco percorrido ao contrário.
        let mut mesh_sides: Vec<Vec<u32>> = Vec::with_capacity(n);
        let mut mesh_tau: Vec<Vec<f32>> = Vec::with_capacity(n);
        for side in sides {
            let (mut chain, mut tau): (Vec<u32>, Vec<f32>) = (Vec::new(), Vec::new());
            for &(a, rev) in side {
                let (Some(cc), Some(src)) =
                    (layout.arc_chain.get(a as usize), from.get(a as usize))
                else {
                    continue;
                };
                let mut c = cc.clone();
                let end = src.last().copied().unwrap_or(0.0);
                let mut t: Vec<f32> = if rev {
                    src.iter().rev().map(|v| end - v).collect()
                } else {
                    src.clone()
                };
                if rev {
                    c.reverse();
                }
                let base = tau.last().copied().unwrap_or(0.0);
                for v in &mut t {
                    *v += base;
                }
                if chain.is_empty() {
                    chain = c;
                    tau = t;
                } else {
                    chain.extend_from_slice(&c[1..]);
                    tau.extend_from_slice(&t[1..]);
                }
            }
            mesh_sides.push(chain);
            mesh_tau.push(tau);
        }
        let seg: Vec<u32> = sides
            .iter()
            .map(|s| s.iter().map(|&(a, _)| quant.arc[a as usize]).sum())
            .collect();
        // ⛔⛔ **`true` — o LSCM é OBRIGATÓRIO aqui, e a alternativa é circular.**
        // Ver o doc de [`PatchParam::build`]: com o Tutte, a «fracção conforme» ao longo
        // de um lado **é o `τ` de volta**, porque o Tutte prega a fronteira por `τ`.
        // *Medido: a re-graduação saiu byte-idêntica ao controlo, e o que ela mediu foi
        // a sua própria entrada.*
        let Some(faces_of) = patch_faces.get(p) else {
            continue;
        };
        let Some(param) =
            PatchParam::build(indexed, faces_of, &mesh_sides, &mesh_tau, &seg, None, true)
        else {
            gave_up[0] += 1;
            continue;
        };

        // ── A proposta deste patch, arco a arco.
        for (i, side) in sides.iter().enumerate() {
            let Some(alpha) = param.side_alpha.get(i) else {
                gave_up[1] += 1;
                continue;
            };
            if alpha.len() != mesh_sides[i].len() {
                gave_up[2] += 1;
                continue;
            }
            let mut at = 0usize;
            for &(a, rev) in side {
                let Some(len) = layout.arc_chain.get(a as usize).map(Vec::len) else {
                    break;
                };
                if at + len > alpha.len() {
                    break;
                }
                let slice = &alpha[at..at + len];
                let (lo, hi) = (slice[0], slice[len - 1]);
                let span = hi - lo;
                if span <= 0.0 {
                    gave_up[3] += 1;
                    at += len - 1;
                    continue;
                }
                // ⚠️⚠️ **O ESPELHO de um arco percorrido ao contrário**, e é a mesma
                // armadilha que o `τ` já tinha: virar a lista sem virar os VALORES daria
                // uma cadeia decrescente. Aqui são as DUAS coisas — o índice canónico e
                // a fracção:
                //
                // | | índice canónico de `slice[k]` | fracção canónica |
                // |---|---|---|
                // | a favor | `k` | `f` |
                // | ⚠️ ao contrário | `len − 1 − k` | **`1 − f`** |
                let Some(bucket) = acc.get_mut(a as usize) else {
                    break;
                };
                if bucket.len() != len {
                    gave_up[4] += 1;
                    at += len - 1;
                    continue;
                }
                for (k, w) in slice.iter().enumerate() {
                    let f = (w - lo) / span;
                    let (idx, val) = if rev { (len - 1 - k, 1.0 - f) } else { (k, f) };
                    bucket[idx] += val;
                }
                votes[a as usize] += 1;
                at += len - 1;
            }
        }
    }

    // ── A média, e o total de cada arco é o de sempre.
    let mut out = Vec::with_capacity(arcs);
    let mut changed = 0usize;
    for (a, old) in from.iter().enumerate() {
        let total = old.last().copied().unwrap_or(0.0);
        if votes[a] == 0 || total <= 0.0 {
            out.push(old.clone());
            continue;
        }
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / votes[a] as f32;
        let mut t: Vec<f32> = acc[a].iter().map(|v| v * inv * total).collect();
        // ⛔ **A monotonia é forçada, não esperada.** A média de duas cadeias monótonas
        // é monótona; o que não é garantido é o `f32` respeitá-lo no empate, e um `τ`
        // que recua faz a reamostragem devolver pontos fora de ordem.
        for k in 1..t.len() {
            t[k] = t[k].max(t[k - 1]);
        }
        if let Some(last) = t.last_mut() {
            *last = total;
        }
        out.push(t);
        changed += 1;
    }
    Some(Regraduation {
        tau: out,
        changed,
        gave_up,
        moved: 0.0,
    })
}

/// ⭐⭐⭐ **O PONTO FIXO SOBRE O LAYOUT** — a segunda das duas saídas que o `PLAN.md`
/// §4-tresetquadragies nomeou, e a que nunca tinha sido construída.
///
/// # ⛔⛔ Porque uma varredura só não respondia
///
/// A proposta de cada patch sai do **achatamento** dele, e o achatamento prega a
/// fronteira **pelo `τ`**. ⇒ *uma varredura calcula as propostas a partir do `τ` que
/// está a tentar substituir.* A segunda varredura já vê um achatamento diferente, e
/// propõe outra coisa.
///
/// ⛔⛔⛔ **E a nota que aqui estava afirmava o contrário sem medir a sequência:**
/// *«o ponto fixo dela nem sequer contrai (`21,4 → 21,3`)»*. ⚠️ **`21,4 → 21,3` é UM
/// TERMO.** Uma afirmação sobre convergência precisa de vários, e nenhum tinha sido
/// calculado — a varredura corria uma vez e parava. *É a mesma família do tecto lido
/// como defeito: uma conclusão sobre um comportamento, tirada de uma amostra que não o
/// pode exibir.*
///
/// # A régua da contracção
///
/// [`Regraduation::moved`] é o maior deslocamento de um ponto de subdivisão entre a
/// ronda anterior e esta, **em unidades de `τ`**. Contrair = essa coluna a cair
/// geometricamente; empatar = ela a estabilizar num valor > 0; divergir = a subir.
/// ⭐ *É a coluna que distingue as três, e ela não existia.*
pub(crate) fn conformal_arc_tau(
    indexed: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
) -> Option<Regraduation> {
    let mut cur: Vec<Vec<f32>> = layout.arc_tau.clone();
    let mut last = one_sweep(indexed, layout, quant, &cur)?;
    for _ in 1..ROUNDS.max(1) {
        let moved = spread(&cur, &last.tau);
        cur = std::mem::take(&mut last.tau);
        let Some(next) = one_sweep(indexed, layout, quant, &cur) else {
            // ⚠️ **Uma ronda que recusa não apaga as anteriores.** Devolve-se o que já
            // se tem, com o deslocamento da última que correu.
            return Some(Regraduation {
                tau: cur,
                changed: last.changed,
                gave_up: last.gave_up,
                moved,
            });
        };
        last = next;
    }
    last.moved = spread(&cur, &last.tau);
    Some(last)
}

/// O maior deslocamento de um ponto de subdivisão entre duas graduações.
///
/// ⚠️ **Comprimentos diferentes contam como deslocamento INFINITO**, não como zero: se
/// a contagem de pontos mudou, o par nem sequer é comparável, e um zero ali leria-se
/// como «convergiu».
fn spread(a: &[Vec<f32>], b: &[Vec<f32>]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }
    let mut worst = 0.0f32;
    for (x, y) in a.iter().zip(b) {
        if x.len() != y.len() {
            return f32::INFINITY;
        }
        for (p, q) in x.iter().zip(y) {
            worst = worst.max((p - q).abs());
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    /// ⭐⭐ **PRESENÇA: a re-graduação corre em TODOS os arcos, e diz porque não.**
    ///
    /// ⛔⛔ **Ela nasceu a `5 de 42` e ninguém sabia porquê.** Foi a coluna do MOTIVO
    /// (`Regraduation::gave_up`) que respondeu numa corrida: `sem alfa 57` — o caminho
    /// do LSCM devolvia um patch com o `side_alpha` **vazio**, porque a linha que o
    /// preenche faltava num dos quatro `return` do `PatchParam::build`. ⇒ `42/42`.
    ///
    /// ⚠️ **É a terceira vez que o sintoma foi «byte-idêntico ao controlo»** neste
    /// ficheiro, com três causas diferentes (circularidade do Tutte · um `?` a abortar a
    /// função · um `return` sem o campo). *Nenhuma delas se distinguia sem a coluna do
    /// motivo, e nenhuma dose de raciocínio as separava.*
    ///
    /// ⚠️ Custa `108 s` (o LSCM a `100 000` rondas × 16 patches) — daí o `#[ignore]`.
    #[test]
    #[ignore = "lento (108 s): o LSCM a 100 000 rondas por patch"]
    fn the_regraduation_actually_changes_the_ruler() {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
        mesh.triangulate();
        ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
        mesh.triangulate();
        let dual = ph2d_crossfield::Dual::build(&mesh);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
        let spec = layout.to_layout(0.25).expect("o layout fecha");
        let (quant, _) =
            ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
                .expect("a quantizacao fecha");
        let r = super::conformal_arc_tau(&mesh, &layout, &quant).expect("a re-graduacao responde");
        let (tau, changed) = (&r.tau, r.changed);
        eprintln!(
            "  regraduou {changed}/{} arcos · desistiu [sem achatamento {}, sem alfa {}, \
             comprimentos {}, span nulo {}, balde {}]",
            layout.arc_chain.len(),
            r.gave_up[0],
            r.gave_up[1],
            r.gave_up[2],
            r.gave_up[3],
            r.gave_up[4],
        );
        assert_eq!(tau.len(), layout.arc_tau.len(), "mudou a contagem de arcos");
        assert_eq!(
            changed,
            layout.arc_chain.len(),
            "a re-graduacao mexeu em {changed} de {} arcos -- ela esta' a recuar em silencio \
             nalgum patch, e o `gave_up` acima diz em qual passo",
            layout.arc_chain.len()
        );
        assert_eq!(
            r.gave_up, [0; 5],
            "houve desistencias: {:?} -- ver a ordem no doc de `Regraduation::gave_up`",
            r.gave_up
        );
        // ⚠️ **O TOTAL de cada arco tem de ficar igual** — é a restrição que mantém a
        // quantização fora desta experiência.
        for (a, (novo, velho)) in tau.iter().zip(&layout.arc_tau).enumerate() {
            let (x, y) = (
                novo.last().copied().unwrap_or(0.0),
                velho.last().copied().unwrap_or(0.0),
            );
            assert!(
                (x - y).abs() <= y.abs() * 1.0e-4,
                "o arco {a} mudou de TOTAL ({y} -> {x}) -- isso muda a quantizacao, que e' \
                 outra experiencia"
            );
            assert!(
                novo.windows(2).all(|w| w[1] >= w[0]),
                "o arco {a} saiu com `tau` a RECUAR -- a reamostragem devolveria pontos fora \
                 de ordem"
            );
        }
    }

    /// ⭐⭐⭐ **SONDA — O PONTO FIXO CONTRAI?**
    ///
    /// ```text
    /// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
    ///     does_the_fixed_point_contract -- --ignored --nocapture
    /// ```
    ///
    /// ⛔⛔ **A rejeição da re-graduação concluiu «o ponto fixo nem sequer contrai» a
    /// partir de UMA ronda.** A proposta de cada patch sai do achatamento dele, e o
    /// achatamento prega a fronteira pelo `τ` que se está a substituir — logo a segunda
    /// ronda vê outro achatamento e propõe outra coisa. *A sequência nunca foi
    /// calculada.*
    ///
    /// Esta sonda calcula-a: o maior deslocamento de um ponto de subdivisão de uma ronda
    /// para a seguinte. **Contrai** = a cair geometricamente · **empata** = estabiliza
    /// acima de zero · **diverge** = sobe.
    ///
    /// ⚠️ Não tem barra e **não é um gate**: ela imprime a sequência e quem lê decide.
    /// *A barra viria da sequência, e a sequência é o que falta.*
    #[test]
    #[ignore = "sonda lenta: uma cadeia de LSCM por ronda"]
    fn does_the_fixed_point_contract() {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
        mesh.triangulate();
        ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
        mesh.triangulate();
        let dual = ph2d_crossfield::Dual::build(&mesh);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
        let spec = layout.to_layout(0.25).expect("o layout fecha");
        let (quant, _) =
            ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
                .expect("a quantizacao fecha");

        // ⭐ **A escala com que comparar o deslocamento.** Um `τ` é comprimento de arco,
        // logo `moved` está nas unidades da peça; sozinho ele não diz se é grande. O
        // passo MEDIANO entre pontos de subdivisão é o que ele tem de bater para
        // significar alguma coisa.
        let mut steps: Vec<f32> = layout
            .arc_tau
            .iter()
            .flat_map(|t| t.windows(2).map(|w| w[1] - w[0]))
            .collect();
        steps.sort_by(f32::total_cmp);
        let step = steps.get(steps.len() / 2).copied().unwrap_or(1.0);
        eprintln!("  passo mediano entre pontos de subdivisao: {step:.5}");

        let mut cur: Vec<Vec<f32>> = layout.arc_tau.clone();
        for round in 1..=6usize {
            let Some(r) = super::one_sweep(&mesh, &layout, &quant, &cur) else {
                eprintln!("  ronda {round}: RECUSOU");
                break;
            };
            let moved = super::spread(&cur, &r.tau);
            eprintln!(
                "  ronda {round}: moveu {moved:.5} ({:>6.2}% do passo) · {} arcos · desistiu {:?}",
                100.0 * moved / step,
                r.changed,
                r.gave_up
            );
            cur = r.tau;
        }

        // ⭐⭐⭐ **E O NÚMERO QUE INTERESSA, na mesma sonda.**
        //
        // ⚠️ Contrair não é o mesmo que **acertar**: o ponto fixo converge para
        // *alguma* graduação, e a pergunta é se essa graduação endireita o domínio.
        // *Uma sonda que só imprimisse a contracção deixaria isso por medir e a
        // primeira leitura seria «convergiu, logo resolveu».*
        //
        // ⛔ A cadeia lê `REGRADUATE`/`ROUNDS` das constantes, então o A/B faz-se
        // trocando-as e correndo isto duas vezes. A comparação é interna a esta
        // fixtura — ⚠️ os `21,4°` das tabelas publicadas são de uma esfera `96×144`,
        // e **não** se comparam com o número desta.
        match crate::fill(&mesh, &mesh, &layout, &quant, crate::SMOOTHING_ROUNDS) {
            Ok((_, rep)) => eprintln!(
                "  ⭐⭐⭐ REGRADUATE={} ROUNDS={} | DOMINIO rect {:.1}° (n={}) leque {:.1}° (n={}) \
                 | enviesamento p50 {:.0}° | dobras {} | regraduou {}",
                super::REGRADUATE,
                super::ROUNDS,
                rep.domain_skew.0,
                rep.domain_cells.0,
                rep.domain_skew.1,
                rep.domain_cells.1,
                rep.shape.skew_p50,
                rep.folded_local,
                rep.regraduated,
            ),
            Err(e) => eprintln!("  ⛔ a montagem RECUSOU: {e:?}"),
        }
    }
}
