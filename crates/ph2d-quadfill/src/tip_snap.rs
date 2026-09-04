//! ⭐⭐⭐ **O BICO ENCOSTA NO ÁPICE — o remate que torna as pontas UNIFORMES.**
//!
//! Irmão de [`crate::untangle`] por RESPONSABILIDADE: aquele desfaz faces do avesso, este
//! fecha a **fase** com que a grade chega ao bico. Os dois correm no fim das três saídas de
//! [`crate::finish_extracted_travel`], pela mesma razão: *uma cura que vive em duas das três
//! saídas é uma cura que o produto às vezes não corre.*
//!
//! # ⛔⛔⛔ O report do dono (2026-09-04) e o que a medição respondeu
//!
//! *«temos bons resultados em muitas pontas no mesmo mesh onde apenas uma tem resultado ruim.
//! Isso é muito estranho.»* Com a tabela por ponta ([`crate::tip_rows`]) na mão, a peça dele a
//! `Detail 1` lê, em células do alvo:
//!
//! | | ponta `8042` (a mais longa) | as outras quatro |
//! |---|---|---|
//! | a saída que ele exportou (`sculpt003`) | ⛔ `gap 0,47` | `0,08`–`0,27` |
//! | a anterior (`sculpt-Pos-Remesh`) | ⛔⛔ **comida** (`gap ≥ 3`) | `0,05`–`0,26` |
//! | a nossa corrida de 04/09 | `0,18` | `0,08`–`0,20` |
//!
//! ⭐⭐⭐ **E a MESMA extracção, acabada de duas maneiras, dá as duas leituras:** as `21 914`
//! faces com a cerca de viagem apertada lêem `gap 0,18` naquele bico, e com a cerca larga
//! lêem **`0,51`** — *o defeito nasce no ACABAMENTO, não na grade* (a grade do bico é a mesma:
//! `0,46` e `0,88`, as duas dentro da barra).
//!
//! # ⭐⭐ O MECANISMO, e ele não é sorte
//!
//! O acabamento pousa cada vértice na **escultura**, e a projecção ao ponto mais próximo de
//! uma superfície **nunca escolhe o ápice**: o bico é um ponto de medida nula, e o pé da
//! perpendicular cai sempre no FLANCO. ⇒ cada ronda embota a ponta um pouco, e *quanto* ela
//! embota depende da cerca de viagem que aquela saída calhou de usar. **É por isso que uma
//! ponta sai boa e a vizinha não.**
//!
//! ⚠️ **A distinção que faz esta cura ser legítima: o que falta é FASE, não CÉLULA.** Com a
//! calota da fase zero ([`ph2d_remesh_iso::Cap`]) a grade do bico chega a `0,46`–`0,88` do
//! alvo — *ela TEM resolução*, e o que sobra é meia célula de desalinhamento entre a última
//! volta da grade e o ápice. ⛔ Em 2026-08-31, quando a grade do bico ainda estava a `3,85 ×`
//! o alvo, puxar o vértice mais avançado foi **medido e refutado** — mas ali o deslocamento
//! era `0,6198` em unidades de mundo, **`23` células**, e levava o aspecto a `12,11`. Aqui ele
//! é de **meia célula**, e é a mesma recusa a responder a outra pergunta.
//!
//! ⛔ **A cerca é a barra da própria régua** ([`crate::TIP_GAP_MAX`]): fecha-se a fase dentro
//! da banda que a régua já chama *«não amputada»*, e **nunca** acima dela. Uma ponta comida a
//! `3` células continua acusada — *uma correcção de posição não pode apagar do selector um
//! defeito que é de células.*

use ph2d_mesh::Mesh;

use super::apex::apices;
use super::local::dist;

/// ⚠️ **Até que distância do ápice o bico se fecha** — em células, e é a **barra da régua**
/// ([`crate::TIP_GAP_MAX`]), nunca um número escolhido. Ver o doc do módulo.
pub const TIP_SNAP_GAP: f32 = crate::TIP_GAP_MAX;

/// ⚠️ **Quanto o vértice pode VIAJAR** — uma célula. O `gap` mede ponto→FACE e o vértice mais
/// próximo pode estar mais longe que ele; sem esta segunda cerca, uma ponta cuja superfície
/// passa perto **por uma face** arrastaria um vértice de longe para o bico.
pub const TIP_SNAP_TRAVEL: f32 = 1.0;

/// ⭐⭐⭐ **FECHA A FASE DE CADA PONTA** — devolve quantos bicos encostaram.
///
/// Para cada espinho afiado da escultura ([`apices`]), o vértice da saída mais próximo do
/// ápice muda-se **para o ápice**, se as duas cercas o permitirem ([`TIP_SNAP_GAP`],
/// [`TIP_SNAP_TRAVEL`]).
///
/// ⚠️ **Uma ponta de cada vez, com o censo GLOBAL a decidir** — a mesma disciplina do
/// [`crate::untangle_bowties`]: o reparo aceita-se se as faces péssimas (`> 60°`) e as faces
/// do avesso não subirem na malha **inteira**. ⛔ Aceitar as cinco de uma vez faria uma ponta
/// má vetar quatro boas; medir só as faces vizinhas deixaria o reparo empurrar o defeito para
/// o lado.
///
/// ⛔ **Sem ápices, `unit` não positiva ou malha vazia ⇒ `0` e a malha fica ao bit.**
pub fn snap_tips(mesh: &mut Mesh, surface: &Mesh, unit: f32) -> usize {
    if !unit.is_finite() || unit <= 0.0 || mesh.positions().is_empty() {
        return 0;
    }
    let (_, apex) = apices(surface, unit);
    if apex.is_empty() {
        return 0;
    }
    let spos = surface.positions();
    let mut base = crate::quad_shape(mesh);
    let mut base_avesso = avesso(mesh);
    let mut encostados = 0usize;
    for &a in &apex {
        let Some(alvo) = spos.get(a).copied() else {
            continue;
        };
        let opos = mesh.positions();
        let Some(v) =
            (0..opos.len()).min_by(|&i, &j| dist(alvo, opos[i]).total_cmp(&dist(alvo, opos[j])))
        else {
            continue;
        };
        let viagem = dist(alvo, opos[v]);
        if viagem <= 0.0 || viagem > TIP_SNAP_TRAVEL * unit {
            continue;
        }
        // ⛔ **A ponta AMPUTADA não se remata**: acima da barra o que falta é célula, e mudar
        // um vértice esconderia do selector um defeito que ele tem de ver.
        let tris = super::tip_rows::fan_tris(mesh);
        let near =
            super::tip_rows::near_tris(&tris, alvo, 2.0 * super::tip_rows::DEV_RADIUS * unit);
        if super::tip_rows::gap_to(&near, alvo) > TIP_SNAP_GAP * unit {
            continue;
        }
        let antes = opos[v];
        mesh.positions_mut()[v] = alvo;
        let agora = crate::quad_shape(mesh);
        let avesso_agora = avesso(mesh);
        if agora.skew_over_60 <= base.skew_over_60 && avesso_agora <= base_avesso {
            base = agora;
            base_avesso = avesso_agora;
            encostados += 1;
        } else {
            mesh.positions_mut()[v] = antes;
        }
    }
    if encostados > 0 {
        mesh.rebuild();
    }
    encostados
}

/// **As faces do AVESSO** — gravatas mais dobras, o mesmo censo que o selector do botão conta
/// ([`crate::untangle`]). ⚠️ *As duas espécies contam juntas de propósito: uma relaxação local
/// troca a espécie de um defeito tanto quanto o remove.*
fn avesso(mesh: &Mesh) -> usize {
    crate::local_shape(mesh).0.bowties + crate::quality::folded_faces_by_neighbours(mesh).len()
}

#[cfg(test)]
#[path = "tip_snap_tests.rs"]
mod tests;
