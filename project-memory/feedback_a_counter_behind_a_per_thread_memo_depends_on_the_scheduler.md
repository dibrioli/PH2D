---
name: feedback_a_counter_behind_a_per_thread_memo_depends_on_the_scheduler
description: "«Contador e não relógio, logo imune à carga» é FALSO quando o contado passa por estado POR THREAD (memo, arena): quantas threads o rayon põe a tocar a coisa muda com o fan-out — o gate da superfórmula leu morno 0/4/8 e reprovou o ship sem uma linha de Rust mudada"
metadata:
  type: feedback
---

Integração da refatoração final (2026-09-13). O `ship.sh` reprovou, sem uma linha de Rust mudada desde
a ponta verde, o `the_shape_constants_are_computed_once_per_shape_not_once_per_tile`
(`ph2d-field-render`): *«quadro MORNO tinha de pagar ZERO varreduras e pagou 4»*. O gate contava
`ops_gielis::SCANS`, e o doc dele E o do próprio contador juravam *«uma contagem é imune à carga»*.

**Mecanismo:** o memo das constantes da forma é `thread_local` e o quadro corre em rayon (32 workers + a
thread que chama = 33). O produto garante **uma conta por forma POR THREAD**; o gate afirmava **zero no
2.º quadro**, o que só vale se nenhuma thread que o 1.º quadro não usou tocar a forma no 2.º — e isso
decide-o o escalonador. Medido: sozinho `frio 132 = 33×4, morno 0` dez vezes em dez; 24 cópias
concorrentes → `morno 4` e `morno 8` em 3–4 de cada 24; o ship leu `frio 128, morno 4` (uma thread faltou
ao frio e pagou no morno).

**Why:** um contador só é imune ao RELÓGIO. Quando o que se conta passa por estado POR THREAD, a
contagem depende de QUAIS threads trabalham — exactamente o que o fan-out muda. É a família de flakes de
fan-out do CLAUDE.md §5.0 com um contador no lugar do relógio.

**How to apply:**
1. A **lei exacta** corre numa pool de UMA thread (`rayon::ThreadPoolBuilder::new().num_threads(1)` +
   `install`): aí `frio == 4` e `morno == 0` são igualdades, e a regressão (conta por ladrilho) lê `2 568`.
2. No **caminho do produto** afirme o invariante ESTRUTURAL, não o observado: dois quadros
   `≤ 4 × (rayon::current_num_threads() + 1)` — nenhuma thread paga duas vezes. (Medido no tecto exacto,
   `132`, em 24 de 24: um tecto estrutural pode ter folga zero e continuar certo.)
3. Antes de chamar flake, **reproduza sob contenção** (N cópias concorrentes do binário com `--exact`) e
   conte `test result: (ok|FAILED)` com controlo do filtro — o `ok` é minúsculo, e um `[A-Z]+` leu 72
   passagens como NADA.
4. Corrija também o doc que prometia imunidade — é ele que o autor do próximo gate lê.

Relacionado: [[feedback_a_constant_folded_into_a_tree_is_recomputed_wherever_the_tree_is]],
[[reference_flip_fit_cache_ratio_is_a_load_flake]], [[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]].
