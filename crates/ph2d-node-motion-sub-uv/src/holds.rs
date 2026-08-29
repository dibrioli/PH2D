//! **A DURAÇÃO DESIGUAL POR QUADRO** — a única das quatro leis de *cel animation* que o
//! grafo não tinha (medido: `cel_animation_laws_the_graph_already_has`).
//!
//! # O que já existia, e por que isto é o que sobra
//!
//! O plano [93](../../../docs/Motion%20Nodes/93_plano_lsystem_datasource_celanim.md) §3 dizia
//! que faltavam quatro coisas ao flipbook desta casa. Três já existiam: **inverso** é o
//! `speed` negativo (zero nós), **ping-pong** é o `value.wrap(Mirror)` e **tocar uma vez** é o
//! `value.wrap(Clamp)`. ⇒ Sobra esta: com um `speed` constante toda célula dura o mesmo, e
//! nenhum dos três modos do `wrap` muda isso — eles dobram e cortam o eixo, nunca o
//! **esticam por troços**. É o que um animador chama de *hold*: a pose de contacto de um ciclo
//! de passo fica no ecrã três vezes mais tempo do que as de passagem.
//!
//! # A forma: PESOS RELATIVOS, nunca milissegundos
//!
//! ⭐ `"1 1 3 1"` diz *"a terceira célula dura o triplo"*. O tempo total do ciclo continua a
//! ser `cells / speed` — os pesos só **redistribuem** dentro dele.
//!
//! ⚠️ **É isso que impede uma SEGUNDA resposta a «quão rápido»**. Uma lista em milissegundos
//! competiria com o `Cells / Second` que o nó já tem, e as duas discordariam: o artista
//! arrastaria o slider e nada mudaria. E de quebra evita a recusa R1 da auditoria da §11 do
//! Sprite — um `frame_ms` de `[1, 60000]` num slider dá ~600 ms de salto por pixel.
//!
//! # A tabela é a LEI, e os dois motores lêem a MESMA
//!
//! ⚠️ **Nem a CPU calcula exacto nem a GPU aproxima** — as duas amostram a mesma LUT com a
//! mesma aritmética. A alternativa (CPU exacta, GPU por tabela) é o que o `field.remap` faz
//! para uma CURVA, e ali funciona porque a curva é contínua e o erro é a inclinação a dividir
//! pela resolução. Aqui a lei é um **DEGRAU**: perto de uma fronteira, exacto e tabelado
//! diferem por uma CÉLULA INTEIRA, e a barra de paridade teria de excluir a vizinhança de
//! cada transição — que é a metade que interessa.
//!
//! ⚠️ E a resolução é **512** pela mesma razão medida que a onda `Custom` do
//! `motion.oscillator`: numa tabela uniforme, o que encolhe com a densidade é a **LARGURA da
//! banda errada**, não a altura ([`feedback_a_uniform_grid_cannot_represent_a_corner`]). Uma
//! fronteira de célula fica a menos de `0,2 %` do ciclo do sítio exacto — e, o que importa
//! mais, **ao mesmo sítio nos dois motores**.

/// A resolução da tabela — ver o cabeçalho para de onde sai o número.
pub(crate) const HOLD_LUT_RESOLUTION: u32 = 512;

/// O nome do canal de LUT: o acessor gerado chama-se `suv_hold_sample(t)`.
pub(crate) const HOLD_LUT_NAME: &str = "suv_hold";

/// O text param que carrega os pesos (o canal do doc 32 — um `ParamSpec` é um `f32`, e uma
/// lista de durações não é um número).
pub const HOLDS_KEY: &str = "holds";

/// **A SENTINELA de «nada autorado»**, escrita em toda a tabela quando o texto está vazio.
///
/// ⚠️ Ela é NEGATIVA de propósito: a saída legítima vive em `[0, 1)` (uma fracção do eixo de
/// células), então nenhum valor autorado a pode imitar. E como a tabela inteira leva o mesmo
/// número, a interpolação do acessor devolve-o intacto em qualquer `t` — os dois motores lêem
/// «uniforme» com a mesma comparação, e o caminho que já shipava corre **byte-idêntico**.
pub(crate) const NO_HOLDS: f32 = -1.0;

/// Os pesos de `text`. Vazio / malformado / todos nulos ⇒ `None` (o ciclo uniforme).
///
/// ⚠️ Um peso **não-positivo é descartado**, não coagido a zero: uma célula de duração zero
/// nunca seria desenhada, e uma lista `"1 0 1"` significaria *"salta a do meio"* — que já se
/// diz apagando-a da folha. Manter a célula e dar-lhe duração nula é estado inalcançável.
#[must_use]
pub(crate) fn weights(text: &str) -> Option<Vec<f32>> {
    let w: Vec<f32> = text
        .split([' ', ',', ';', '\t'])
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    (w.len() >= 2).then_some(w)
}

/// **A LEI**: para a fase `t ∈ [0,1)`, qual célula (como fracção do eixo uniforme).
///
/// Devolve o **centro** da célula escolhida num eixo de `m` células — `(j + ½) / m` — e não a
/// borda dela. ⚠️ O centro é o que torna a interpolação do acessor inofensiva: entre duas
/// entradas vizinhas da tabela que escolhem a mesma célula, a mistura não sai do interior
/// dela; e numa fronteira ela atravessa `(j+1)/m` exactamente a meio caminho, **nos dois
/// motores**. Com as bordas, um `floor` do lado errado de um ULP daria a célula seguinte num
/// motor e a anterior no outro.
fn cell_fraction(w: &[f32], t: f32) -> f32 {
    let m = w.len();
    let total: f32 = w.iter().sum();
    let target = t.clamp(0.0, 1.0) * total;
    let mut acc = 0.0f32;
    for (j, wj) in w.iter().enumerate() {
        acc += *wj;
        if target < acc {
            return (j as f32 + 0.5) / m as f32;
        }
    }
    (m as f32 - 0.5) / m as f32
}

/// Enche a tabela a partir do text param — a metade deste crate do canal de LUT (o
/// `ph2d-nodegraph` não sabe o que é um *hold*).
pub(crate) fn fill_hold_lut(text: &str, out: &mut [f32]) {
    let Some(w) = weights(text) else {
        out.fill(NO_HOLDS);
        return;
    };
    let last = out.len().saturating_sub(1).max(1) as f32;
    for (k, slot) in out.iter_mut().enumerate() {
        *slot = cell_fraction(&w, k as f32 / last);
    }
}

/// **O acessor, letra a letra igual ao que o codegen gera para o WGSL** — `clamp`, escala
/// pelo último índice, `floor`, vizinho, `mix`.
///
/// ⚠️ Ele é copiado de propósito e não abstraído: o original vive no gerador de código
/// (`ph2d-gpu-cook::codegen`), que uma crate de nó não pode alcançar (ADR-0075). O que os
/// mantém em acordo é o gate de paridade, e é a mesma disciplina do `rem_euclid` do
/// `cell_xform` — que também vive duas vezes, aqui e no WGSL.
#[must_use]
pub(crate) fn sample(lut: &[f32], t: f32) -> f32 {
    if lut.is_empty() {
        return NO_HOLDS;
    }
    let last = lut.len() - 1;
    let x = t.clamp(0.0, 1.0) * last as f32;
    let i0 = x.floor() as usize;
    let i1 = (i0 + 1).min(last);
    let f = x - i0 as f32;
    lut[i0.min(last)] * (1.0 - f) + lut[i1] * f
}

/// **A tabela deste nó, construída uma vez por cozimento** — a porta única da CPU.
///
/// ⚠️ Ela é reconstruída a cada `eval` e não guardada: um cache seria estado, e este nó é
/// `Effect::Pure`. O preço é 512 buscas lineares sobre uma lista de unidades — medido abaixo
/// do ruído contra o próprio laço por elemento.
#[must_use]
pub(crate) fn table(text: &str) -> Vec<f32> {
    let mut out = vec![0.0; HOLD_LUT_RESOLUTION as usize];
    fill_hold_lut(text, &mut out);
    out
}

/// **A célula que a fase `k` mostra**, já com os *holds* aplicados — a lei que os dois
/// motores correm.
///
/// `k` é a posição em CÉLULAS que o nó já calculava (`cell + speed·t + stagger·i`); o ciclo
/// tem `cells` células, então a fase é `frac(k / cells)`. Sem *holds*, `k` volta intacto e o
/// `cell_xform` faz o que sempre fez.
#[must_use]
pub(crate) fn held_index(lut: &[f32], k: f32, cells: u32) -> f32 {
    let v = sample(lut, phase(k, cells));
    if v < 0.0 {
        return k;
    }
    (v * cells as f32).floor()
}

/// A fase de `k` no ciclo de `cells` células, em `[0,1)`. `rem_euclid` pela mesma razão do
/// [`super::cell_xform`]: um `k` negativo tem de contar do fim.
fn phase(k: f32, cells: u32) -> f32 {
    if !k.is_finite() || cells == 0 {
        return 0.0;
    }
    let c = cells as f32;
    (k / c).rem_euclid(1.0)
}

#[cfg(test)]
#[path = "holds_tests.rs"]
mod tests;
