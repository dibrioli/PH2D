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
/// ⛔⛔ **DESLIGADO — a construção está inteira e o gate de PRESENÇA está VERMELHO**
/// (2026-08-23): ela re-gradua **`5` de `42`** arcos e recua em silêncio nos outros.
/// Ver `the_regraduation_actually_changes_the_ruler`.
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
pub(crate) fn conformal_arc_tau(
    indexed: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
) -> Option<(Vec<Vec<f32>>, usize)> {
    let arcs = layout.arc_chain.len();
    // Por arco, as propostas acumuladas e quantas foram.
    let mut acc: Vec<Vec<f32>> = layout
        .arc_tau
        .iter()
        .map(|t| vec![0.0f32; t.len()])
        .collect();
    let mut votes: Vec<usize> = vec![0; arcs];

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
                let (Some(cc), Some(src)) = (
                    layout.arc_chain.get(a as usize),
                    layout.arc_tau.get(a as usize),
                ) else {
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
            continue;
        };

        // ── A proposta deste patch, arco a arco.
        for (i, side) in sides.iter().enumerate() {
            let Some(alpha) = param.side_alpha.get(i) else {
                continue;
            };
            if alpha.len() != mesh_sides[i].len() {
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
    for (a, old) in layout.arc_tau.iter().enumerate() {
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
    Some((out, changed))
}

#[cfg(test)]
mod tests {
    /// ⛔⛔ **VERMELHO, com endereço — a re-graduação RECUA em 37 dos 42 arcos.**
    ///
    /// ⭐ **Medido 2026-08-23: `5 de 42`.** A construção está inteira e a cadeia
    /// consome-a; o que falta é descobrir **em que patch** ela desiste. Os suspeitos,
    /// por ordem de custo: o `PatchParam::build` a devolver `None` com o `force_lscm`
    /// (o `locals(...)?` no fim de dois caminhos), o `span <= 0` num arco, ou o
    /// `bucket.len() != len` (`arc_tau` e `arc_chain` com contagens diferentes).
    ///
    /// ⚠️ **Fica `#[ignore]` e não afrouxado.** A barra é *«mais de metade dos arcos»*,
    /// que é o mínimo para a média entre dois patches significar alguma coisa — baixá-la
    /// para `5` tornaria o gate verde sobre exactamente o defeito que ele nomeia.
    /// *A barra é do fenómeno; o `#[ignore]` é da agenda.*
    ///
    /// ⚠️ Ele custa `88 s` (o LSCM a `100 000` rondas × 16 patches), e é outra razão para
    /// não correr no lote.
    #[test]
    #[ignore = "VERMELHO -- a re-graduacao recua em 37 dos 42 arcos; ver o doc"]
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
        let (tau, changed) =
            super::conformal_arc_tau(&mesh, &layout, &quant).expect("a re-graduacao responde");
        assert_eq!(tau.len(), layout.arc_tau.len(), "mudou a contagem de arcos");
        assert!(
            changed > layout.arc_chain.len() / 2,
            "a re-graduacao so' mexeu em {changed} de {} arcos -- ela esta' a recuar em \
             silencio na maioria dos patches",
            layout.arc_chain.len()
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
}
