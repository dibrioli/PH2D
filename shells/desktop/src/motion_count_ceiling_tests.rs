//! **O TETO DE CONTAGEM de um param se MEDE** (doc 88 A1 · CLAUDE.md §0).
//!
//! O slider dual separa duas perguntas que estavam colapsadas num número só: o **soft** (`ParamUiHint.max`)
//! é a faixa confortável que o arrasto cobre, e o **hard** (`ParamHardMax`) é **onde o disfuncional
//! começa** — o número que a caixa de texto ainda aceita. O §0 é explícito sobre o segundo: *antes de
//! escrever qualquer teto, MEÇA — e escreva o número que a medição deu, com a tabela ao lado dele.*
//!
//! ⚠️ **E o spread de hoje é o próprio sintoma que o §0 descreve:** `motion.voronoi` traz **165.000**
//! (um número que a linha da GPU mediu) enquanto `motion.grid` traz **20** por lado e `motion.clone`
//! **32** — não porque alguém mediu a grade e a achou 8.000× mais cara que o voronoi, mas porque
//! ninguém mediu. *O caminho mais lento definiu o teto do mais rápido.*
//!
//! Esta sonda existe para trocar esses palpites por números. Ela cozinha o nó pela porta REAL (`Cook`
//! sobre o registry do produto) numa varredura de contagens e reporta o custo por cook — **e a tabela
//! que ela imprime é o que tem de aparecer ao lado de cada teto que esta wave escrever**.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins measure_the_count_ceiling -- --ignored --nocapture
//! ```

#[cfg(test)]
mod tests {
    use crate::motion_state::MotionState;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// Um cook do nó `type_name` com o param de contagem em `count`, ligado a um `motion.output`.
    /// Devolve `(ms do cook mediano, instâncias que saíram)` — ou `None` se o nó não cozinha
    /// sozinho (precisa de entrada que esta sonda não monta).
    ///
    /// ⚠️ **Um `Cook` NOVO por amostra, e a 1ª corrida é a que conta** — o oposto do redutor que
    /// um laço quente pede, e a 1ª versão desta sonda errou exatamente aqui: o `Cook` **MEMOIZA**,
    /// então descartar a 1ª corrida (a cautela de *first-touch* que o pen-up do Painter ensinou)
    /// deixava a mediana medir **quatro acertos de memo**. A tabela saía `0,00 ms` em toda célula
    /// enquanto o teste levava **1402 s** — *números que não reconciliam com o relógio de parede
    /// são a assinatura desta doença*, e é o relógio que denuncia, não a tabela.
    fn cook_cost(
        type_name: &str,
        count_param: &str,
        count: f32,
        fixed: &[(&str, f32)],
    ) -> Option<(f64, usize)> {
        let st = MotionState::new();
        let mut g = Graph::new();
        let node = g.add_node(type_name);
        let out = g.add_node("motion.output");
        // ⚠️ Os params FIXOS existem para que o eixo medido seja o pedido: a grade multiplica
        // `rows` por `cols`, então sem `cols = 1` a coluna "1000" da tabela mede 3000 instâncias
        // e o número deixa de ser sobre o param que a linha nomeia.
        for (k, v) in fixed {
            g.set_param(node, *k, *v);
        }
        g.set_param(node, count_param, count);
        g.connect(Edge {
            from: (node, 0),
            to: (out, 0),
            delayed: false,
        })
        .ok()?;
        g.validate(&st.registry).ok()?;
        let mut ms = Vec::new();
        let mut n = 0usize;
        for _ in 0..3 {
            let mut cook = Cook::new(); // memo VAZIO: este cook faz o trabalho inteiro.
            let t0 = std::time::Instant::now();
            let set = cook.cook(&g, &st.registry, out, 0.5).ok()?;
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            n = set.iter().next().map_or(0, |s| s.as_stream().count());
        }
        ms.sort_by(f64::total_cmp);
        Some((ms[1], n))
    }

    /// **A TABELA** — o custo de um cook contra a contagem pedida, por nó de contagem.
    ///
    /// O que ler nela: a coluna `inst` diz quantas instâncias de fato saíram (um nó pode recusar,
    /// clampar ou multiplicar o número pedido), e a coluna `ms` diz o que um quadro paga. O
    /// orçamento de referência é **16,6 ms** (um quadro de 60 fps) — o teto HARD honesto de um param
    /// de contagem é a última potência que fica confortavelmente abaixo dele, e é ESSE número que
    /// deve ser escrito no `ParamHardMax`, com esta tabela ao lado.
    #[test]
    #[ignore = "sonda de medição, não gate"]
    fn measure_the_count_ceiling() {
        // (nó, param de contagem, soft max de hoje) — os nós cuja contagem carrega um RECURSO.
        // ⚠️ `motion.path` fica FORA: cozinha sozinho e devolve **0 instâncias** (a curva dele vem
        // de um text param que esta sonda não semeia) — medir zero seria medir a fixture.
        // ⚠️ **Cada nó leva a PRÓPRIA varredura**, e não é conveniência: varrer o `scatter` até
        // 400k custou **208 SEGUNDOS** num número que ninguém vai usar, e varrer a grade só até
        // 10k esconderia que ela é barata. A faixa de cada linha é a faixa em que a resposta
        // daquele nó de fato muda.
        /// Um sujeito da varredura: `(nó, param de contagem, soft de hoje, params fixos, faixa)`.
        type Subject = (
            &'static str,
            &'static str,
            f32,
            &'static [(&'static str, f32)],
            &'static [f32],
        );
        let subjects: &[Subject] = &[
            // Os LINEARES e baratos — a varredura vai fundo porque eles aguentam.
            (
                "motion.grid",
                "rows",
                20.0,
                &[("cols", 1.0)],
                &[100_000.0, 400_000.0, 1_000_000.0],
            ),
            (
                "motion.fibonacci",
                "count",
                2_000.0,
                &[],
                &[100_000.0, 400_000.0, 1_000_000.0],
            ),
            (
                "motion.distribute_radial",
                "count",
                2_000.0,
                &[("rings", 1.0)],
                &[100_000.0, 400_000.0, 1_000_000.0],
            ),
            // O QUADRÁTICO — best-candidate blue noise (Mitchell): `nearest_sq` varre todos os
            // pontos já postos, logo O(count² × CANDIDATES). A varredura cerca o quadro de 60 fps.
            (
                "motion.scatter",
                "count",
                2_000.0,
                &[],
                &[2_000.0, 3_000.0, 4_000.0, 6_000.0],
            ),
            // O da GPU — o cook de CPU aqui é o caminho de REFERÊNCIA, não o que manda no teto
            // (§0). A varredura fica pequena de propósito: é para NOMEAR a distância, não para
            // derivar um cap dela.
            (
                "motion.voronoi",
                "count",
                165_000.0,
                &[],
                &[1_000.0, 4_000.0, 8_000.0],
            ),
        ];
        println!("\n=== custo de UM cook (ms) x contagem pedida — orçamento 16,6 ms/quadro ===");
        for (ty, param, soft, fixed, counts) in subjects {
            let mut row = format!("{ty:26} soft={soft:<9.0} ");
            let mut first = None;
            let mut last = None;
            for &c in counts.iter() {
                match cook_cost(ty, param, c, fixed) {
                    Some((ms, n)) => {
                        row.push_str(&format!("{c:.0}:{ms:.3}ms/{n} "));
                        first.get_or_insert(ms);
                        last = Some(ms);
                    }
                    None => row.push_str(&format!("{c:.0}:-- ")),
                }
            }
            // ⚠️ O CONTROLE. Um custo que NÃO sobe com a contagem significa que a sonda não está
            // medindo o cook — foi exatamente assim que a 1ª versão saiu `0,00 ms` em toda célula
            // sobre um teste de 1402 s. A tabela só vale se esta razão for maior que 1.
            if let (Some(a), Some(b)) = (first, last) {
                row.push_str(&format!("| x{:.0}", if a > 0.0 { b / a } else { 0.0 }));
            }
            println!("{row}");
        }
    }
}
