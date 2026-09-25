//! ⭐⭐⭐ **A MARCHA DE VISIBILIDADE** — a sombra de uma luz e o cone da oclusão. Saiu do
//! [`super`] por tecto de LOC; a lei do passo é a mesma (`t += d · step`, com o orçamento derivado).

use super::Scene;
use crate::MAX_STEPS;

/// ⭐⭐⭐ **A MARCHA DA SOMBRA** — de cada ponto da peça na direcção de uma luz.
///
/// # ⚠️ Porque ela é uma função PRÓPRIA e não um modo da [`super::march_slabs`]
///
/// Ela responde outra pergunta e paga outro preço. A marcha do quadro quer **onde** o raio parou e
/// com que **normal**, sai de um plano de câmera, e especializa a árvore por ladrilho × fatia porque
/// os raios dela são **coerentes no frustum**. Esta quer **se alguma coisa está pelo caminho**,
/// parte de pontos espalhados pela superfície, e os raios dela apontam todos à luz — logo a
/// especialização da outra não vale aqui.
///
/// ⭐ **O que as duas PARTILHAM é a lei do passo**, e ela é a mesma linha: `t += d · step`, com o
/// orçamento derivado do passo (`MAX_STEPS · shrink / step`). *Um segundo passo escrito aqui seria o
/// raio a andar por duas leis no mesmo documento.*
///
/// # A VISIBILIDADE, e porque ela não é um booleano
///
/// Devolve `0` (tapado) a `1` (livre). O termo macio é o estimador clássico `min(k·d/t)` — *quanto
/// mais perto de um obstáculo o raio passou, e mais cedo, mais penumbra* —, e o `k` é a dureza: ele
/// é o inverso do tamanho angular da fonte.
///
/// ⚠️ **O `t` começa AFASTADO da superfície**, e o afastamento é derivado: o ponto de partida foi
/// produzido por uma marcha que pára quando o campo desce abaixo de [`crate::Sharpness::hit`], logo ele
/// está **ligeiramente fora** — começar em `0` faria toda superfície acertar em si própria e a peça
/// sair **preta**. É a mesma tolerância que o `pick` já cita por escrito.
#[allow(dead_code)]
pub(crate) fn march_shadow(
    scene: &Scene<'_>,
    origins: &[[f32; 3]],
    dirs: &[[f32; 3]],
    t_max: f32,
    hardness: f32,
) -> Vec<f32> {
    let cerca = vec![t_max; origins.len()];
    march_shadow_to(scene, origins, dirs, &cerca, hardness)
}

/// A marcha de sombra com a cerca **POR RAIO** — a forma que o produto usa.
///
/// ⭐⭐⭐ **A cerca é o que decide o preço, e não a lei do passo.** Medido em 2026-09-14 sobre a peça
/// de três cilindros cruzados: os raios **de costas** para a luz custam quase nada (terminam no
/// primeiro passo, dentro da própria peça), logo filtrá-los corta `55 %` da POPULAÇÃO e `6 %` do
/// relógio — *o caro é o raio que VIAJA sem encontrar nada*. O que corta a viagem é a cerca:
/// - **nada além da lâmpada tapa** ⇒ `t_max` é a distância ao ponto de luz, por raio;
/// - **um raio que saiu da bola que contém a peça nunca lá volta** ⇒ corta-se na saída da bola.
///
/// As três cercas têm de devolver a MESMA imagem; só a mais apertada o faz mais depressa.
pub(crate) fn march_shadow_to(
    scene: &Scene<'_>,
    origins: &[[f32; 3]],
    dirs: &[[f32; 3]],
    t_max: &[f32],
    hardness: f32,
) -> Vec<f32> {
    march_shadow_counted(scene, origins, dirs, t_max, hardness, BIAS).0
}

/// Quantas tolerâncias de acerto o raio anda **antes de começar a perguntar**.
pub(crate) const BIAS: f32 = 4.0;

/// A mesma marcha, devolvendo **quantas amostras de campo** custou — a régua que separa «o raio
/// viaja longe» de «o raio rasteja à saída».
pub(crate) fn march_shadow_counted(
    scene: &Scene<'_>,
    origins: &[[f32; 3]],
    dirs: &[[f32; 3]],
    t_max: &[f32],
    hardness: f32,
    bias: f32,
) -> (Vec<f32>, u64) {
    march_visibility(scene, origins, dirs, t_max, |_| hardness, bias, true)
}

/// ⭐⭐⭐ **A MESMA marcha com a dureza POR RAIO** — o que o cone da oclusão precisa.
///
/// # Porque a dureza deixou de poder ser um escalar
///
/// A [`crate::occlusion::occlusion_slice_with_reach`] traça um **CONE por direcção**, e o cone certo
/// para a oclusão é o que **roça o plano tangente**: meio-ângulo `θ` com `tan θ = n·d`. Isso põe a
/// dureza a `1/(n·d)` — **um número por raio**, porque cada direcção do conjunto faz um ângulo
/// diferente com a normal daquele pixel.
///
/// ⚠️ **É essa dureza que faz um corpo CONVEXO ler exactamente `1,0`**: num plano a distância ao
/// longo do raio é `t·(n·d)` por identidade, logo `hardness · d / t = 1` — e numa esfera é
/// *maior* que `1`. Uma dureza escalar não tem como saber isto, e foi por não a ter que a
/// primeira tentativa de cone (`hardness = 1`) leu uma **esfera mais ocluída que uma cruz**.
pub(crate) fn march_cone_to(
    scene: &Scene<'_>,
    origins: &[[f32; 3]],
    dirs: &[[f32; 3]],
    t_max: &[f32],
    hardness: &[f32],
) -> Vec<f32> {
    debug_assert_eq!(origins.len(), hardness.len());
    // ⚠️ O CONE não acaba na cerca: a lei dele foi calibrada contra `1 024` direcções com a
    // paragem de sempre (ver `crate::occlusion::OCCLUSION_PASSES`), e a cerca dele é a bola.
    march_visibility(scene, origins, dirs, t_max, |i| hardness[i], BIAS, false).0
}

/// ⭐⭐⭐⭐ `ate_a_cerca`: o último passo é encurtado até à cerca e AMOSTRADO lá, em vez de parar no
/// primeiro passo que a ultrapassa — ver a `visivel` do dispositivo, que é a gémea desta. Parar no
/// passo deixa a última amostra num sítio que depende da FASE dos passos, e com a luz encostada à
/// peça (report do dono, 2026-09-24) o chão desenhava um ANEL por salto de fase.
fn march_visibility<H: Fn(usize) -> f32>(
    scene: &Scene<'_>,
    origins: &[[f32; 3]],
    dirs: &[[f32; 3]],
    t_max: &[f32],
    hardness: H,
    bias: f32,
    ate_a_cerca: bool,
) -> (Vec<f32>, u64) {
    debug_assert_eq!(origins.len(), t_max.len());
    let mut amostras = 0u64;

    let n = origins.len();
    let mut vis = vec![1.0f32; n];
    if n == 0 {
        return (vis, 0);
    }
    let t0 = scene.sharp.hit * bias;
    let mut t = vec![t0; n];
    let mut ultimo = vec![false; n];
    let mut cur: Vec<u32> = (0..n as u32).collect();
    // ⚠️ **O caminho NÃO especializado**, e de propósito: a árvore por ladrilho é do frustum da
    // câmera, e estes raios não vivem nele. O `fork` é o que o `march_slabs` já faz quando não há
    // fatia — a `shape` é partilhada entre as threads do lote.
    let mut eval = scene.shape.fork();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let budget = ((MAX_STEPS as f32) * scene.shrink.max(1.0) / scene.step.clamp(f32::EPSILON, 1.0))
        .ceil() as usize;
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..budget {
        if cur.is_empty() {
            break;
        }
        xs.clear();
        ys.clear();
        zs.clear();
        for &i in &cur {
            let i = i as usize;
            xs.push(origins[i][0] + dirs[i][0] * t[i]);
            ys.push(origins[i][1] + dirs[i][1] * t[i]);
            zs.push(origins[i][2] + dirs[i][2] * t[i]);
        }
        let Ok(out) = eval.eval(&xs, &ys, &zs) else {
            break;
        };
        amostras += cur.len() as u64;
        let mut next = Vec::with_capacity(cur.len());
        for (j, &i) in cur.iter().enumerate() {
            let iu = i as usize;
            let d = out[j];
            if d < scene.sharp.hit {
                vis[iu] = 0.0;
                continue;
            }
            vis[iu] = vis[iu].min(hardness(iu) * d / t[iu]);
            if ultimo[iu] {
                continue;
            }
            t[iu] += d * scene.step;
            if t[iu] >= t_max[iu] {
                if !ate_a_cerca {
                    continue;
                }
                t[iu] = t_max[iu];
                ultimo[iu] = true;
            }
            next.push(i);
        }
        cur = next;
    }
    (vis, amostras)
}
