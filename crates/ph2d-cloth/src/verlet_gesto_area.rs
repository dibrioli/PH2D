//! **A NORMAL E O CENTRO DA ÁREA** (espec §4.2-bis e §4.4) — irmão (`#[path]`)
//! do [`super::verlet_gesto`], cortado por ASSUNTO.
//!
//! ⭐ **Uma varredura, DUAS grandezas, e elas não são a mesma coisa:** a normal
//! da área é a direcção do Push, o `ẑ` do referencial local e a normal do plano
//! de queda; o centro da área é o PONTO por onde esse plano passa. Os dois saem
//! do mesmo disco de meio raio e do mesmo desempate por baldes — e é por saírem
//! juntos que vivem aqui e não no ficheiro do gesto.
//!
//! ⚠️ **Nenhum caminho de chamador muda:** o `verlet_gesto` re-exporta as três
//! coisas (a constante e as duas funções).

use crate::V3;
use crate::verlet::{dist, unit};

/// **O «Normal Radius» dos presets de tecido** (espec §4.2-bis (3), proveniência
/// `A` — lido dos treze pincéis da biblioteca do alvo).
///
/// ⚠️⚠️ **A normal da área é amostrada num disco de METADE do raio do pincel.**
/// Um port que a tire do disco inteiro tem uma direcção diferente assim que a
/// superfície deixa de ser plana ou de estar em repouso — foi essa a diferença
/// que deixou o `plano_empurrar_plano_local` a errar `0,944`.
pub const RAIO_DA_NORMAL: f64 = 0.5;

/// **A NORMAL DA ÁREA** (espec §4.2-bis) — um vector por passo, e a mesma
/// grandeza que o Push usa como direcção, o referencial local usa como `ẑ` e o
/// falloff de plano usa como normal (§4.4).
///
/// ⛔ **A regra de desempate NÃO é uma média:** os vértices repartem-se em dois
/// baldes pelo sinal de `n̂ · v̂`, e a resposta é a soma normalizada do PRIMEIRO
/// balde que esteja **não-vazio E com soma de comprimento não-nulo**, nesta
/// ordem fixa — nunca a mistura, nunca «o balde com mais vértices». ⇒ basta UM
/// vértice virado para a vista para que os virados ao contrário não contem.
///
/// ⚠️ **E o teste é não-vazio E soma não-nula, balde a balde:** se as normais do
/// balde da frente se cancelarem, a resposta é a do balde de trás. ⛔ Não é
/// «escolher o balde e só depois olhar para a soma».
///
/// Sem resposta nenhuma ⇒ **vector NULO**, e o Push desse passo é força zero,
/// sem `NaN` e sem direcção de reserva (espec §4.2-bis (5)).
#[must_use]
pub fn normal_da_area(
    posicoes: &[V3],
    normais: &[V3],
    dentro: &[u32],
    cursor: V3,
    raio: f64,
    vista: V3,
) -> V3 {
    normal_e_centro_da_area(posicoes, normais, dentro, cursor, raio, vista).0
}

/// **A NORMAL e o CENTRO DA ÁREA, da MESMA varredura** (espec §4.2-bis e §4.4).
///
/// ⭐⭐⭐ **O centro da área NÃO é o centroide do disco.** Cada vértice entra na
/// média já **puxado para o cursor**:
///
/// ```text
/// contribuição(v) = c + (p_v − c) · (1 − a_v)      a_v = 3p² − 2p³
/// ```
///
/// ⇒ o peso `1 − a` vale **zero no cursor** e cresce para a borda do disco: quanto
/// mais perto do cursor um vértice está, mais completamente ele é **substituído**
/// por ele. Numa folha em repouso o centro da área é praticamente o cursor; numa
/// folha já cavada ele fica muito mais perto do cursor do que o centroide.
///
/// ⚠️⚠️ **É por isso que a medição de 06/09 — «o plano pelo cursor reproduz o alvo
/// e o plano pelo centro da área afasta-o» — não refuta esta lei: o que foi medido
/// foi um CENTROIDE, e o alvo não usa um centroide** (`empurrar 0,944 → 1,250`,
/// `arrastar 0,233 → 0,716`). *Uma recusa medida responde a UMA pergunta, e aquela
/// respondeu «o centroide não serve», não «o centro da área não serve».* O plano
/// pelo cursor é a aproximação de **primeira ordem** desta lei, e é por isso que
/// ele passava quase.
///
/// ⛔ **O balde é escolhido UMA vez, pela regra da normal** (§4.2-bis (4)) — as
/// duas grandezas saem do mesmo balde, e não de dois desempates independentes.
/// Sem balde nenhum, o centro é o **cursor** e a normal é o vector nulo.
///
/// ⚠️ **Só o plano de queda lê o centro da área.** A origem do referencial local
/// é o cursor (§4.4), e a localização da área *Local* (§2.1) é outra coisa ainda
/// — essa fica no pen-down durante todo o traço, e mora em
/// [`PincelTecido::localizacao_da_area`].
#[must_use]
pub fn normal_e_centro_da_area(
    posicoes: &[V3],
    normais: &[V3],
    dentro: &[u32],
    cursor: V3,
    raio: f64,
    vista: V3,
) -> (V3, V3) {
    let alcance = raio * RAIO_DA_NORMAL;
    let mut baldes = [[0.0f64; 3]; 2];
    let mut centros = [[0.0f64; 3]; 2];
    let mut contas = [0usize; 2];
    for &v in dentro {
        let vi = v as usize;
        let p = posicoes[vi];
        let d = dist(p, cursor);
        if d > alcance {
            continue;
        }
        let n = normais[vi];
        // O peso é a mesma smoothstep da banda, sobre `1 − d/alcance`.
        let t = (1.0 - d / alcance.max(1e-30)).clamp(0.0, 1.0);
        let w = t * t * (3.0 - 2.0 * t);
        let frente = usize::from(n[0] * vista[0] + n[1] * vista[1] + n[2] * vista[2] <= 0.0);
        contas[frente] += 1;
        for c in 0..3 {
            baldes[frente][c] += n[c] * w;
            centros[frente][c] += cursor[c] + (p[c] - cursor[c]) * (1.0 - w);
        }
    }
    // ⚠️⚠️ **A metade «soma não-nula» da regra é carregada pelo [`unit`], e não
    // por um `if` aqui — e isso é MEDIDO, não suposto.** Um `if` que a testasse
    // sobrevive a toda mutação, porque:
    //
    // - o balde da FRENTE, se não estiver vazio, **nunca** tem soma nula: ela é
    //   `Σ wᵢ n̂ᵢ` com todos os `wᵢ > 0` e todos os `n̂ᵢ · v̂ > 0`, logo o produto
    //   dela com `v̂` é positivo e o vector não pode ser zero;
    // - o de TRÁS pode cancelar-se (ali `n̂ · v̂ ≤ 0` admite o zero), e o [`unit`]
    //   de um vector nulo é o vector nulo — que é exactamente o que a espec
    //   §4.2-bis (5) manda devolver.
    //
    // *Uma linha que nenhuma mutação mata é redundante ou falta-lhe um gate; esta
    // era redundante, e a prova está no gate `sem_balde_valido_a_normal_da_area_e_nula`.*
    for b in 0..2 {
        if contas[b] > 0 {
            let k = contas[b] as f64;
            let c = [centros[b][0] / k, centros[b][1] / k, centros[b][2] / k];
            return (unit(baldes[b]), c);
        }
    }
    ([0.0; 3], cursor)
}
