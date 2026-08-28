//! ⭐⭐⭐ **O ACABAMENTO DA CADEIA DE EXTRACÇÃO** — uma porta, uma lei, dois chamadores.
//!
//! # Por que ele é uma PORTA e não uma linha em cada chamador
//!
//! ⛔ Em 2026-08-28 mediu-se: a `ph2d-quadchain` (a ordem canónica da cadeia) entregava a
//! malha **crua**, e o shell da escultura corria `6` rondas de Laplaciano. *Dois caminhos
//! para o mesmo botão, com acabamentos diferentes* — e a constante `6` vinha da montagem por
//! patches, medida noutra conectividade e nunca reconferida aqui.
//!
//! # A lei, em três frases
//!
//! 1. **A ronda zero é o que shipava** — as [`crate::SMOOTHING_ROUNDS`] de Laplaciano
//!    tangencial. ⚠️ É ela que mata a face extrema: medido na `sculpt_hooked` fina, as faces
//!    com canto pior que `60°` caem de `7` para `1`, e nenhuma quantidade de ajuste de
//!    quadrado faz isso.
//! 2. Depois corre o **ajuste de quadrado alinhado ao relevo**
//!    ([`crate::square_relax_aligned`]), que ataca a metade que o Laplaciano não vê: o
//!    **ângulo**. Medido, ele move a mediana do enviesamento de `6,5°` para `1,8°`–`3,3°`.
//! 3. ⭐⭐⭐ **E a saída é a MELHOR ronda, não a última.**
//!
//! # ⛔⛔ A terceira frase não é elegância: é a cura de uma regressão MEDIDA
//!
//! A relaxação melhora a mediana e, passado o joelho, **redistribui** o erro para menos
//! faces e piores. Na `sculpt_hooked` fina ela leva o `>60°` de `1` (o que shipa hoje) para
//! `6`–`10`, e o Laplaciano à frente **não** o impede (`lap 6` + relaxação dá `10`; `lap 20`
//! dá `8`). ⇒ *iterar até assentar entrega uma malha pior na coluna em que hoje batemos o
//! oráculo* (`1` contra os `4` da saída alisada dele).
//!
//! ⭐ Guardar a melhor ronda torna a regressão **inexprimível**: a ronda `0` é o produto de
//! hoje, e a saída só muda se alguma ronda for estritamente melhor que ela. *Não se escolhe
//! um número de rondas melhor; deixa-se de ter de escolher.*
//!
//! ⚠️ **A comparação é de PARETO nas três colunas que a barra do oráculo nomeia** — faces
//! péssimas, enviesamento mediano e aspecto mediano —, e não uma soma pesada: ver
//! [`better`].

use ph2d_mesh::Mesh;

use crate::shape::{QuadShape, quad_shape};

/// ⭐ **Quando a relaxação desiste** — o maior movimento de uma ronda, em fracções da
/// aresta **mediana da saída**.
///
/// ⛔ **Não é um número de rondas, e a razão é medida:** a taxa de convergência depende do
/// tamanho da malha. `320` rondas mal se notam a `4 500` quads e quase cegam uma malha de
/// `531` — *um tecto de rondas é uma cerca cujo tamanho muda com a peça.*
///
/// # ⛔⛔ A tabela que escolheu `1e-3`, e a PRIMEIRA que estava errada
///
/// ⚠️ **A 1.ª escolha (`1e-2`) saiu de uma varredura sem a ronda zero à frente** e o número
/// não sobreviveu ao produto: através da porta ele deu **`23` rondas** em vez das `93` da
/// tabela, e `7,8° → 6,8°` em vez de `7,8° → 4,5°`. *O Laplaciano pré-condiciona a malha, o
/// movimento começa menor, e o mesmo limiar relativo chega muito mais cedo.*
///
/// A tabela abaixo é medida **através de [`finish_extracted_with`]**, na densidade que o
/// botão usa:
///
/// | peça · alvo | `settle` | rondas | ms | aspecto p50 | envies. p50 |
/// |---|---|---|---|---|---|
/// | `wrinkled` · 2 | — (ronda zero) | 0 | 35 | 1,19 | `7,8°` |
/// | | `1e-2` | 23 | 94 | 1,16 | `6,8°` |
/// | | `3e-3` | 107 | 286 | 1,12 | `5,3°` |
/// | | ⭐ **`1e-3`** | 308 | **564** | **1,10** | **`4,5°`** |
/// | | `3e-4` | 1 200 | 1 539 | 1,08 | `4,3°` |
/// | `eared` · 2 | — (ronda zero) | 0 | 23 | 1,14 | `10,4°` |
/// | | ⭐ **`1e-3`** | 350 | **464** | **1,07** | **`3,8°`** |
/// | | `3e-4` | 660 | 836 | 1,07 | `3,5°` |
/// | `hooked` · 2 | — (ronda zero) | 0 | 8 | 1,17 | `7,7°` |
/// | | ⭐ **`1e-3`** | 283 | **216** | **1,09** | **`4,3°`** |
///
/// ⭐ **`3e-4` custa `1,5`–`3×` mais para comprar `0,2`–`0,3°`** — é aí que a escada deixa de
/// pagar, e por isso `1e-3` é o número. O acabamento fica em `0,2`–`0,6 s` sobre uma cadeia
/// de `4`–`10 s`, e o botão corre a cadeia **duas** vezes.
pub const EXTRACT_SETTLE: f32 = 1.0e-3;

/// A rede, para o caso de o assentamento não chegar. ⚠️ Medido: `1e-3` gasta `248`–`350`
/// rondas nas células do corpus, então este número **não** é o que manda.
pub const EXTRACT_MAX_ROUNDS: usize = 1_200;

/// ⭐⭐⭐ **QUANTAS RONDAS SEM MELHORIA ANTES DE DESISTIR.**
///
/// ⛔⛔ **Ela existe por uma medição, não por prudência.** Na `sculpt_hooked` **fina** o
/// alinhamento ao relevo nunca bate a ronda zero — aquela peça tem `1` face péssima depois
/// do Laplaciano, e a relaxação alinhada sobe-a para `2` logo na primeira ronda, o que a
/// ordem de comparação recusa **para sempre**. Sem esta rede a corrida gastava `1 200`
/// rondas e `8,3 s` para entregar exactamente a malha com que começou.
///
/// ⚠️ **Ela é um limite de DESPERDÍCIO, não de qualidade:** nas peças que melhoram, a última
/// melhoria chega perto do fim (`302` de `308`, `273` de `283`), sempre com intervalos muito
/// menores que este número entre melhorias sucessivas.
pub const EXTRACT_PATIENCE: usize = 128;

/// ⭐⭐ **Quanto a direcção da superfície roda o quadrado** — multiplica a anisotropia.
///
/// ⭐ **`1` significa «o peso É a confiança», sem constante nenhuma por cima**, e é o único
/// valor que não é um knob a defender. Medido na `sculpt_wrinkled` grossa: sem alinhamento o
/// relevo vai de `11,9°` para `18,8°` (com `22,5°` = cega) e com `pull = 1` fica em `11,2°`
/// — **melhor que a malha crua** — pelo mesmo enviesamento. ⚠️ Subir para `2`, `4` ou `8`
/// move o relevo mais `0,3°` e paga `p99`; ver o `ACHADO_o_acabamento_e_a_regua_da_densidade`.
pub const EXTRACT_RELIEF_PULL: f32 = 1.0;

/// O que o acabamento mediu de si próprio.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FinishReport {
    /// Rondas de ajuste de quadrado que correram.
    pub rounds: usize,
    /// ⭐ **Qual delas ficou** — `0` significa que nenhuma bateu o Laplaciano.
    pub kept: usize,
    /// A forma logo após a ronda zero (o produto de 2026-08-26).
    pub before: QuadShape,
    /// A forma que sai.
    pub after: QuadShape,
}

/// ⭐⭐⭐ **DESISTIR: a janela conta da MELHOR ronda, nunca do início.**
///
/// `round` é o índice `0`-based da ronda que acabou de correr e `kept` é a **melhor** até
/// agora (`0` = a ronda zero, o Laplaciano).
///
/// ⛔ **Um contador que arrancasse do início mataria a corrida à ronda
/// [`EXTRACT_PATIENCE`] ainda a melhorar** — e nenhuma fixtura de malha o apanha de forma
/// robusta, porque para o separar é preciso uma peça cuja última melhoria caia depois da
/// janela, e isso depende da malha. *Uma lei que a fixtura não separa testa-se onde ela é
/// declarada.*
fn give_up(round: usize, kept: usize) -> bool {
    (round + 1).saturating_sub(kept) >= EXTRACT_PATIENCE
}

/// ⭐ **Ruído de vírgula flutuante abaixo do qual duas formas são a MESMA forma** — em graus
/// para o enviesamento e em razão para o aspecto. Sem ele, uma ronda que muda a mediana em
/// `1e-6` conta como melhoria e a corrida nunca desiste.
const SAME: f32 = 1.0e-3;

/// ⭐⭐⭐ **A COMPARAÇÃO É DE PARETO nas três colunas que a barra do oráculo nomeia** —
/// faces péssimas, enviesamento mediano e aspecto mediano.
///
/// Uma ronda ganha quando **não é pior em nenhuma** e é **estritamente melhor em pelo
/// menos uma**. ⚠️ *Não há peso nenhum aqui, de propósito:* as três grandezas não têm
/// unidade comum, e somá-las seria uma opinião com números por cima.
///
/// ⛔⛔ **A 1.ª redacção era lexicográfica `(faces péssimas, mediana)` e recusava melhorias
/// REAIS** (medido 2026-08-28): numa esfera-UV crua a relaxação leva o aspecto de `1,384`
/// para `1,251` e mexe a mediana em `+0,2°` — a ordem lexicográfica lia isso como «pior» e
/// a corrida entregava a ronda zero. *Uma ordem total obriga a escolher uma coluna
/// vencedora; a de Pareto só recusa o que é pior em alguma.*
///
/// ⭐ E a garantia sai mais FORTE: como a melhor ronda é uma cadeia de melhorias de Pareto
/// a partir da ronda zero, a saída **nunca é pior que ela em nenhuma das três**.
fn better(a: &QuadShape, b: &QuadShape) -> bool {
    let no_worse = a.skew_over_60 <= b.skew_over_60
        && a.skew_p50 <= b.skew_p50 + SAME
        && a.aspect_p50 <= b.aspect_p50 + SAME;
    let strictly = a.skew_over_60 < b.skew_over_60
        || a.skew_p50 < b.skew_p50 - SAME
        || a.aspect_p50 < b.aspect_p50 - SAME;
    no_worse && strictly
}

fn median_edge(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    e.get(e.len() / 2).copied().unwrap_or(0.0)
}

/// ⭐⭐⭐ **ACABA A MALHA EXTRAÍDA** — ver o doc do módulo.
///
/// `surface` é a malha em que a saída pousa: **a escultura**, nunca a remalhada do F1.
/// *Reprojectar sobre a remalhada somaria os dois erros* — a mesma lei que o
/// [`crate::fill`] escreve com o defeito de 2026-08-21 ao lado.
pub fn finish_extracted(mesh: &mut Mesh, surface: &Mesh) -> FinishReport {
    finish_extracted_with(mesh, surface, EXTRACT_RELIEF_PULL, EXTRACT_SETTLE)
}

/// A mesma lei com o alinhamento ao relevo **à vista** — ver [`EXTRACT_RELIEF_PULL`].
///
/// ⚠️ **Existe porque a afirmação «o alinhamento preserva o relevo» é uma COMPARAÇÃO**, e
/// uma comparação precisa dos dois lados: sem esta porta, uma mutação que pusesse o `pull` a
/// zero passava a suíte inteira (medido, 2026-08-28).
///
/// ⛔⛔ **E o `settle_frac` é aqui, e não numa variável de ambiente, por uma razão medida:**
/// a 1.ª redacção deste módulo escolheu `EXTRACT_SETTLE` de uma tabela varrida **sem** a
/// ronda zero à frente, e a mesma fracção deu `23` rondas em vez das `93` da tabela — *o
/// Laplaciano pré-condiciona a malha, o movimento começa menor, e o mesmo limiar chega mais
/// cedo.* Um limiar só se escolhe na configuração em que ele corre.
pub fn finish_extracted_with(
    mesh: &mut Mesh,
    surface: &Mesh,
    pull: f32,
    settle_frac: f32,
) -> FinishReport {
    // ── Ronda zero: exactamente o que shipava em 2026-08-26.
    crate::finish::smooth(mesh, surface, crate::SMOOTHING_ROUNDS);
    let before = quad_shape(mesh);
    let mut rep = FinishReport {
        before,
        after: before,
        ..FinishReport::default()
    };
    // ⚠️ **A guarda é sobre a MALHA, não sobre o limiar.** Ela existe para a malha sem
    // arestas; um `settle_frac` de `0` é uma escolha legítima («só pára por paciência ou
    // pela rede»), e confundir as duas tornava essa escolha inexprimível — que é como o
    // gate da paciência ficaria sem sujeito.
    let unit = median_edge(mesh);
    if unit <= 0.0 {
        return rep;
    }
    let settle = unit * settle_frac;
    let hint = if pull > 0.0 {
        crate::quality::surface_hint(surface, mesh)
    } else {
        Vec::new()
    };
    let origin: Vec<[f32; 3]> = mesh.positions().to_vec();
    let floor = crate::finish::bbox_seed(surface);
    // ⚠️ **A melhor ronda guarda-se por POSIÇÕES, não por malha inteira** — a topologia não
    // muda numa relaxação, e clonar a `Mesh` a cada melhoria pagaria as faces outra vez.
    let mut best_pos = origin.clone();
    for r in 0..EXTRACT_MAX_ROUNDS {
        let seed = if r == 0 { floor } else { 1.0e-6 };
        let mv = crate::relax::square_once(
            mesh,
            surface,
            seed,
            &origin,
            f32::INFINITY,
            &hint,
            pull,
        );
        rep.rounds = r + 1;
        let s = quad_shape(mesh);
        if better(&s, &rep.after) {
            rep.after = s;
            rep.kept = r + 1;
            best_pos.copy_from_slice(mesh.positions());
        }
        // ⚠️ **Assentar OU desistir** — ver [`give_up`].
        if mv <= settle || give_up(r, rep.kept) {
            break;
        }
    }
    // ⚠️ **Repor é incondicional**, e não «se a última não foi a melhor»: `kept == 0` quer
    // dizer que a saída é a da ronda zero, e essa também tem de ser reposta.
    mesh.positions_mut().copy_from_slice(&best_pos);
    mesh.rebuild();
    rep
}

#[cfg(test)]
#[path = "finish_extract_tests.rs"]
mod tests;
