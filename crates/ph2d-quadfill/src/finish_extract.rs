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

/// ⭐⭐⭐ **A CERCA DE VIAGEM DESTE CAMINHO**, em unidades da aresta mediana da malha extraída.
///
/// ⛔⛔⛔ **Até 2026-09-01 este caminho passava `f32::INFINITY` e a cerca estava DESLIGADA no
/// produto** — enquanto [`crate::square_relax_capped`], a porta cujo doc se intitula *«a porta
/// do produto»*, ficava **sem um único chamador**. A tabela que justifica a cerca está escrita
/// lá e mede exactamente a configuração que corria aqui: sem cerca, `1280` rondas levam o
/// relevo de `11,9°` a `19,1°` — e `22,5°` é o valor de uma grade que ignora o relevo. Este
/// laço corre até [`EXTRACT_MAX_ROUNDS`] `= 1 200`.
///
/// ⚠️ **E a aceitação não podia apanhá-lo:** [`acceptable`] e [`better`] lêem enviesamento e
/// aspecto, que é precisamente o que a relaxação sem cerca **melhora** enquanto desliza a grade
/// para fora do sítio. *Uma ronda que come a ponta e endireita os quads é aceite por unanimidade.*
///
/// ⚠️ **A unidade é a aresta, e isso é a lei da porta**, não uma conveniência: a taxa de
/// convergência depende do tamanho da malha, então um tecto de rondas seria uma cerca cujo
/// tamanho muda com a peça.
pub const EXTRACT_TRAVEL: f32 = f32::INFINITY;

/// ⭐⭐⭐ **A CERCA DA TENTATIVA DE SOCORRO** — meia aresta, e o número saiu de uma varredura.
///
/// ⚠️ **Ela NÃO é o valor de omissão, e isso é uma decisão medida.** Na peça do dono a
/// `Detail 1,00` não tem ponta partida, e ali a cerca **piora** a forma sem comprar nada
/// (enviesamento mediano `4,22° → 6,7°`); a `Detail 0,75`, onde uma ponta é comida, ela leva o
/// desvio da ponta de `2,39` para **`0,67`** quads — abaixo da barra. ⇒ o preço só se paga onde
/// há defeito, e quem chama arma-a **só** quando [`crate::tip_deviation`] acusa.
///
/// ⭐ **A varredura completa e a leitura vivem no sítio da chamada**
/// (`sculpt3d_history_retopo_extract.rs`), ao lado da condição que a arma — *uma tabela que
/// justifica uma escolha tem de estar onde a escolha é feita.*
pub const EXTRACT_TRAVEL_RESCUE: f32 = 0.5;

/// ⭐⭐⭐ **QUANTAS RONDAS SEM ACEITAR NADA ANTES DE DESISTIR** — ver [`give_up`].
///
/// ⛔⛔ **Ela existe por uma medição, não por prudência.** Na `sculpt_hooked` **fina** o
/// alinhamento ao relevo nunca bate a ronda zero — aquela peça tem `1` face péssima depois
/// do Laplaciano, e a relaxação alinhada sobe-a para `2` logo na primeira ronda, o que a
/// aceitação recusa **para sempre**. Sem esta rede a corrida gastava `1 200` rondas e
/// `8,3 s` para entregar exactamente a malha com que começou.
///
/// # ⚠️ O número é `1,8×` a maior PRIMEIRA ACEITAÇÃO medida
///
/// | peça · alvo | 1.ª ronda aceite | melhor | caiu para a lei cega? |
/// |---|---|---|---|
/// | `wrinkled` · 2 | 1 | 302 de 308 | não |
/// | `eared` · 2 | 9 | 350 de 350 | não |
/// | `hooked` · 2 | 1 | 273 de 283 | não |
/// | `sphere_uv` · 2 | 1 | 248 de 248 | não |
/// | `wrinkled` · 0,667 | 209 | 901 de 902 | não |
/// | ⚠️ `eared` · 0,667 | **418** | 762 de 793 | não |
/// | `hooked` · 0,667 | 312 | 830 de 830 | ⭐ **sim** |
/// | `sphere_uv` · 0,667 | 1 | 408 de 408 | não |
///
/// ⛔⛔⛔ **A 1.ª redacção pôs `128` e cortava trabalho real** — com ela a `sculpt_hooked`
/// fina saía **intocada** em vez de ir a `1,04 / 2,0° / p99 22,8` com zero faces péssimas.
/// ⭐ **E só UMA das oito células cai para a lei cega** — é isso que faz o relevo ficar
/// guardado nas outras sete.
pub const EXTRACT_PATIENCE: usize = 768;

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
    /// ⭐⭐ **A PRIMEIRA ronda aceite** (`0` = nenhuma). ⚠️ Ela e a [`FinishReport::kept`] são
    /// grandezas diferentes e é a distância entre elas que precifica a paciência: *desistir
    /// enquanto nada foi aceite* é barato, *desistir depois* corta trabalho real.
    pub first: usize,
    /// A forma logo após a ronda zero (o produto de 2026-08-26).
    pub before: QuadShape,
    /// A forma que sai.
    pub after: QuadShape,
    /// ⭐ **A saída veio da lei CEGA** — a alinhada não conseguiu mexer-se. Ver
    /// [`finish_extracted`].
    pub blind: bool,
    /// ⭐⭐⭐ **Quantas GRAVATAS o acabamento desfez** — ver [`crate::untangle_bowties`].
    pub untangled: usize,
}

/// ⭐⭐⭐ **DESISTIR: só enquanto NADA foi aceite ainda.**
///
/// `round` é o índice `0`-based da ronda que acabou e `first` é a **primeira** ronda aceite
/// (`0` = nenhuma até agora).
///
/// ⛔⛔⛔ **A 1.ª redacção media «rondas desde a MELHOR» e cortava trabalho real.** Medido
/// 2026-08-28 na `sculpt_hooked` fina: a primeira ronda aceite é a **`312`** e a melhor é a
/// **`830`** — com uma janela de `128` desde a melhor, a corrida morria à ronda `128` e a
/// peça saía intocada (`1,11 / 6,5° / p99 33,0`), quando ela chega a
/// **`1,04 / 2,0° / p99 22,8` com zero faces péssimas**.
///
/// ⭐ *Desistir enquanto nada foi aceite é barato; desistir depois corta trabalho real* — as
/// duas grandezas são diferentes e a distância entre elas (`312` e `830` na mesma corrida) é
/// o preço de as confundir.
fn give_up(round: usize, first: usize) -> bool {
    first == 0 && round + 1 >= EXTRACT_PATIENCE
}

/// ⭐ **Ruído de vírgula flutuante abaixo do qual duas formas são a MESMA forma** — em graus
/// para o enviesamento e em razão para o aspecto. Sem ele, uma ronda que muda a mediana em
/// `1e-6` conta como melhoria e a corrida nunca desiste.
const SAME: f32 = 1.0e-3;

/// ⭐⭐⭐ **A RONDA É ACEITÁVEL se não for pior que a RONDA ZERO em nenhuma das três
/// colunas que a barra do oráculo nomeia** — faces péssimas, enviesamento mediano e aspecto
/// mediano.
///
/// ⚠️ **Contra a ronda ZERO, nunca contra a melhor até agora.** ⛔⛔ A 1.ª redacção
/// comparava com a melhor e isso é um **catraca**: a relaxação mergulha antes de subir, e
/// bastava uma ronda inicial melhorar uma coluna e piorar outra para todas as seguintes
/// ficarem sem forma de a dominar. Medido em 2026-08-28: na densidade fina a corrida
/// entregava a ronda zero em **todas** as peças do corpus, enquanto a mesma lei com esta
/// aceitação chega a `1,04 / 2,0° / p99 22,8` numa peça em que a ronda zero dá
/// `1,11 / 6,5° / p99 33,0`. *Uma ordem parcial usada como catraca não é conservadora: é
/// cega ao que vem depois do mergulho.*
///
/// ⭐ A garantia fica a mesma e mais simples de dizer: **o que sai nunca é pior que o que
/// shipava em nenhuma das três.**
pub(crate) fn acceptable(s: &QuadShape, base: &QuadShape) -> bool {
    s.skew_over_60 <= base.skew_over_60
        && s.skew_p50 <= base.skew_p50 + SAME
        && s.aspect_p50 <= base.aspect_p50 + SAME
        // ⚠️ **A CAUDA também conta, e ela entrou tarde:** sem esta linha a `sculpt_eared`
        // fina saía com o `p99` do enviesamento em `28,2°` contra os `27,2°` da ronda zero
        // — *uma coluna que o relatório mostra e a garantia não cobria.*
        && s.skew_p99 <= base.skew_p99 + SAME
        && s.aspect_p99 <= base.aspect_p99 + SAME
}

/// ⭐ **Entre as aceitáveis, decide o ENVIESAMENTO MEDIANO** — a coluna que o artista
/// fotografa e a manchete da barra do oráculo —, com o aspecto a desempatar.
///
/// ⚠️ *Não há peso nenhum aqui:* a aceitação já garantiu que nenhuma coluna anda para trás,
/// então esta escolha não pode trocar uma coisa por outra. Somar as três seria uma opinião
/// com números por cima.
fn better(a: &QuadShape, b: &QuadShape) -> bool {
    a.skew_p50 < b.skew_p50 - SAME
        || ((a.skew_p50 - b.skew_p50).abs() <= SAME && a.aspect_p50 < b.aspect_p50 - SAME)
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
    finish_extracted_travel(mesh, surface, EXTRACT_TRAVEL)
}

/// A mesma lei com a **cerca de viagem** à vista — ver [`EXTRACT_TRAVEL`].
///
/// ⚠️ **Existe pela mesma razão que [`finish_extracted_with`]:** a afirmação *«a cerca protege
/// a ponta»* é uma COMPARAÇÃO, e uma comparação precisa dos dois lados. ⛔ E a escolha é do
/// **chamador** — uma variável de ambiente lida aqui dentro alcançaria a cadeia de bancada, os
/// gates e o produto de uma vez.
pub fn finish_extracted_travel(mesh: &mut Mesh, surface: &Mesh, travel: f32) -> FinishReport {
    // ── A lei ALINHADA primeiro: onde ela se mexe, o relevo fica guardado.
    let mut aligned = mesh.clone();
    let mut ra = finish_extracted_with(
        &mut aligned,
        surface,
        EXTRACT_RELIEF_PULL,
        EXTRACT_SETTLE,
        travel,
    );
    if ra.kept > 0 {
        *mesh = aligned;
        // ⭐⭐⭐ **AS TRÊS SAÍDAS CURAM, e esta faltava** (2026-09-03): as faces do avesso saíam
        // intactas por aqui, e foi este o caminho que a peça do dono tomou. *Uma cura que vive
        // em duas das três saídas é uma cura que o produto às vezes não corre.*
        ra.untangled = crate::untangle_bowties(mesh, surface, crate::untangle::UNTANGLE_TRAVEL)
            + crate::untangle::remove_flaps(mesh, surface);
        return ra;
    }
    // ── ⭐⭐⭐ **E SE ELA NÃO CONSEGUIU MEXER-SE, A CEGA TEM A SUA VEZ.**
    //
    // ⛔⛔ **Medido 2026-08-28, e é a razão de isto existir:** na densidade FINA a lei
    // alinhada não bate a ronda zero em peça nenhuma do corpus — a relaxação sobe uma das
    // três colunas logo à primeira ronda e a comparação recusa-a para sempre. A mesma peça,
    // com o alinhamento desligado, vai de `1,11 / 6,5° / p99 33,0 / >60 1` para
    // `1,04 / 2,0° / p99 22,8 / >60 0`.
    //
    // ⚠️ **O PREÇO está medido e é o relevo** (`17,7° → 19,3°` no gancho fino,
    // `11,8° → 13,6°` na enrugada). ⭐ A troca faz-se com os números na mão: o oráculo
    // entrega `13,3°` de relevo naquela peça — *já estamos atrás dessa coluna com ou sem
    // isto* — e as três colunas que a barra dele nomeia são onde a troca nos põe à frente.
    // ⛔ **A ordem importa e não é uma preferência:** a cega só corre onde a alinhada
    // **não teve nada a dizer**, então onde o relevo estava em jogo ele fica guardado.
    //
    // ⚠️ E a suavização do campo de direções (`quality::HINT_SMOOTH_ROUNDS`) foi construída
    // para curar isto **e não curou** — a hipótese do ruído por face está REFUTADA.
    let mut blind = mesh.clone();
    let rb = finish_extracted_with(&mut blind, surface, 0.0, EXTRACT_SETTLE, travel);
    if use_blind(&ra, &rb) {
        *mesh = blind;
        let mut rep = FinishReport { blind: true, ..rb };
        rep.untangled = crate::untangle_bowties(mesh, surface, crate::untangle::UNTANGLE_TRAVEL)
            + crate::untangle::remove_flaps(mesh, surface);
        return rep;
    }
    // Nenhuma das duas bateu o Laplaciano: fica ele.
    *mesh = aligned;
    // ⭐⭐⭐ **AS GRAVATAS SAEM NO FIM, e as TRÊS saídas passam por aqui** — ver
    // [`crate::untangle_bowties`]. ⛔ Uma delas em falta seria o defeito de sempre: a cura a
    // viver no caminho que o produto não corre.
    ra.untangled = crate::untangle_bowties(mesh, surface, crate::untangle::UNTANGLE_TRAVEL)
        + crate::untangle::remove_flaps(mesh, surface);
    ra
}

/// ⭐⭐⭐ **QUANDO A LEI CEGA GANHA A VEZ** — a lei da escolha, separada de quem a executa.
///
/// ⚠️ **Ela é `pub(crate)` e testada sem malha nenhuma de propósito.** Medido em
/// 2026-08-28: nem a esfera com bico nem o toro sacudido conseguem encenar as duas
/// respostas — na primeira **nenhuma** das leis se mexe, na segunda mexem-se as duas. *Uma
/// escolha que a fixtura não separa testa-se onde ela é declarada, e afinar a fixtura até
/// ela separar seria escolher a resposta.*
///
/// ⛔ **A cega só entra onde a alinhada não teve nada a dizer** — é isso que garante que o
/// relevo fica guardado onde ele estava em jogo.
pub(crate) fn use_blind(aligned: &FinishReport, blind: &FinishReport) -> bool {
    aligned.kept == 0 && blind.kept > 0
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
///
/// ⚠️ **`travel_frac` é a cerca de viagem** ([`EXTRACT_TRAVEL`]), em unidades da aresta
/// mediana. Um valor não-finito ou `<= 0` desliga-a — que é o que este caminho fazia sem o
/// saber até 2026-09-01.
pub fn finish_extracted_with(
    mesh: &mut Mesh,
    surface: &Mesh,
    pull: f32,
    settle_frac: f32,
    travel_frac: f32,
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
    // ⚠️ **A cerca desliga-se por valor, e o `NaN` cai do lado seguro** — `!(x > 0.0)` é
    // verdadeiro para `NaN`, o que devolveria a cerca **fechada a zero** e congelaria a malha.
    // Por isso a pergunta é feita ao contrário, e o não-finito vai para `INFINITY`.
    let max_travel = if travel_frac.is_finite() && travel_frac > 0.0 {
        unit * travel_frac
    } else {
        f32::INFINITY
    };
    let hint = if pull > 0.0 {
        crate::quality::surface_hint(surface, mesh)
    } else {
        Vec::new()
    };
    // ⭐⭐⭐ **O LAÇO CORRE SOBRE BUFFERS, e a `Mesh` só se escreve no fim.**
    //
    // ⛔⛔ Medido 2026-08-28: com `Mesh::rebuild()` por ronda, o acabamento era `11,5 s` de
    // `17,7 s` na `sculpt_eared` — e o `rebuild` reconstrói a adjacência, a curvatura e a
    // **octree** da saída, que uma relaxação não muda e não lê. ⚠️ *A porta única do
    // `rebuild` continua a valer:* a malha nunca fica publicada meio-derivada, porque ela só
    // é escrita **uma vez**, no fim.
    let topo = crate::relax_rounds::Topology::of(mesh);
    let faces = mesh.faces().to_vec();
    let mut pos = mesh.positions().to_vec();
    let origin: Vec<[f32; 3]> = pos.clone();
    let floor = crate::finish::bbox_seed(surface);
    let mut best_pos = origin.clone();
    // ⚠️ Os dois buffers de normais vivem FORA do laço — realocá-los por ronda é o mesmo
    // desperdício que o `rebuild` era, num tamanho menor.
    let (mut fnorm, mut vnorm) = (Vec::new(), Vec::new());
    for r in 0..EXTRACT_MAX_ROUNDS {
        let seed = if r == 0 { floor } else { 1.0e-6 };
        let mv = crate::relax_rounds::round(
            &mut pos, &faces, &topo, surface, &hint, pull, &origin, max_travel, seed, &mut fnorm,
            &mut vnorm,
        );
        rep.rounds = r + 1;
        let s = crate::shape::quad_shape_of(&pos, &faces);
        if acceptable(&s, &rep.before) && better(&s, &rep.after) {
            rep.after = s;
            rep.kept = r + 1;
            if rep.first == 0 {
                rep.first = r + 1;
            }
            best_pos.copy_from_slice(&pos);
        }
        // ⚠️ **Assentar OU desistir** — ver [`give_up`].
        if mv <= settle || give_up(r, rep.first) {
            break;
        }
    }
    // ⚠️ **Repor é incondicional**, e não «se a última não foi a melhor»: `kept == 0` quer
    // dizer que a saída é a da ronda zero, e essa também tem de ser reposta. ⭐ E é **aqui**
    // que a malha é publicada — uma vez, com o `rebuild` inteiro.
    mesh.positions_mut().copy_from_slice(&best_pos);
    mesh.rebuild();
    rep
}

#[cfg(test)]
#[path = "finish_extract_tests.rs"]
mod tests;
