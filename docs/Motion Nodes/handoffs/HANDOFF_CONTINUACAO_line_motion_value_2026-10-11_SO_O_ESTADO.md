# CONTINUAÇÃO — `line/motion-value`, o quadro lento sem placa: só o tique desenhado compõe (11/10)

## As regras (R1–R9, de `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` §2 — valem para toda janela da linha)

- **R1 — Janela curta.** Uma onda = uma janela. Passou de ~150 chamadas ou fechou a onda: commit,
  handoff de continuação ≤ 8 KB, PARE.
- **R2 — Edite pela ferramenta `Edit`.** Novo = `Write`; existente = `Edit`. Script só para a MESMA
  troca em N arquivos, com `assert` de contagem.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`;
  só então teste.
- **R4 — Teste dirigido por NOME no laço:** `bash scripts/cargo-test-narrow.sh <crate> <filtro>`. Suíte
  inteira e `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- **R5 — Nada de `until … sleep`.** Comando > 2 min: `run_in_background` e espere o aviso.
- **R6 — Chamadas independentes na MESMA mensagem.**
- **R7 — Doc > 40 KB nunca se lê inteiro:** `grep -n` + `Read` com `offset`/`limit`.
- **R8 — Escreva menos** (comentário 1–3 linhas; resposta ao Enio curta; prompt de subagente: objetivo,
  arquivos, formato da resposta).
- **R9 — Nenhum teste da suíte normal passa de 10 s em `dev`.** Cura: (a) `opt-level = 2` na crate
  quente (§2.1 do prompt); (b) fixture menor que mantém a lei, provada por mutação; (c) só o que não
  afirma nada (`diag_*`, `sonda_*`, `measure_*`) vai a `#[ignore]`. ⛔ Lei afirmada nunca a `#[ignore]`.

## Onde a linha está
## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base `8f20c60de` (= `main`).
  **Nada integrado nem enviado.** Onda anterior:
  [`…_COLLIDE_VIZINHOS.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_COLLIDE_VIZINHOS.md).
- Commits desta onda: `70a5b6a24` (só o estado nos tiques não desenhados) · `c2521297b` (tomadas de
  gizmo só no desenho) · o deste handoff.

## A causa (duas, em série — a 2.ª só apareceu no app)

1. **Na bomba** (`ph2d-eval-motion`): a shell recupera um tique em dívida por cozimento
   (`ticks_owed`) e o scrub re-simula do checkpoint até ao alvo; só o último chega ao ecrã, mas o
   braço `Sinks` compunha todos. Na cena `=8` (`grid → collide → output`, `value.lfo` Temporal no
   `spread`, **nenhuma aresta `pre`**) o `collide` (Pure) recozia-se em cada um, porque o `lfo` muda
   o fingerprint a cada tique. Cura: `so_o_estado.rs` — com `separa_o_desenho == false` a bomba coze
   só os nós **não-`Pure`** do cone dos sinks (o estado no `memo`, ex. `sim.step`) e as tomadas; as
   fontes de `pre` continuam na marcha. Consumidor de `pre` não precisa: o estado dele é a saída da
   fonte, que a marcha avança (gate `quem_consome_o_pre_le_o_tique_anterior_sem_os_intermedios`).
   O scrub marca só o alvo como desenhado e devolve a bandeira à saída.
2. **No app** (medido com registo temporário no nó, já retirado): mesmo com (1) o `collide` corria
   em CADA tique — o **gizmo de posições** (`ponto_gizmo::taps_for`) pede o SINK como tomada, e as
   tomadas cozem em todo tique. Só as dos **sinais** precisam do livro-razão por tique. Cura: a ponte
   arma a mesma lista (começa por `signal_taps`) e chama `set_taps_por_tique(signal_taps.len())`; a
   bomba salta as restantes (warp, colisor, posições, pivô) num tique não desenhado.

## Medido

| régua | antes | depois |
|---|---|---|
| HUD, cena `=8`, `PH2D_GPU_COOK=0`, smoke, `fotografa_cena.sh` (load ~20) | `2 fps · 402 ms` | **`23 fps · 41,9 ms`** |
| registo no `collide`, 25 s de app | 1 avaliação por tique; 7 s simulados em 23,6 s | 1 por quadro; 23,9 s em 23,6 s |
| `sonda_quadro_lento_da_cena_8` (release, 10 tiques/quadro, intercalada, mínimo, load 20) | 599 ms | **64 ms**, mesmos bits (32 761 instâncias) |

O que sobra por quadro é UM `collide` (~42 ms no smoke): é o custo do nó, não da repetição.

## Gates e provas

- `so_o_estado_tests.rs` (8): o `Pure` coze 1× por quadro (controlo `1+4+4`); mesmos bits com
  estado por `pre` (fonte e consumidor), por `memo`, no scrub; o livro-razão das tomadas por tique;
  a tomada de gizmo só no desenho (controlo 9); o scrub que sai cedo devolve a bandeira.
- `passe_no_pump_tests::um_sink_tomada_nao_separa_os_tiques_recuperados` (a separação da tomada
  passa a contar em `separacoes()`).
- `motion_bridge_clock::a_ponte_diz_quais_tomadas_sao_por_tique` (texto da ponte).
- **Mutações 7/7 sangram:** guarda do braço, efeito → nada, scrub sem alvo, tomada-sink a separar,
  `tap_streams.clear()`, restauro da bandeira, salto das tomadas de gizmo (3 → 9).
- Suíte `ph2d-eval-motion` 114/114 (antes do 2.º commit: 111). Portão de fecho: ver abaixo.

**Portão de fecho** (`verificador`, sobre `5a3e47c0d..c2521297b`): `nextest-impacted` 4 642/4 642;
clippy `-D warnings` e fmt limpos nas duas crates; gate-fonte da shell
`the_graph_only_shouts_while_the_clock_plays_forward` 4/4; tectos de LOC ok (`ph2d-eval-motion/src/lib.rs`
698/700 — **2 de folga**: o próximo campo da bomba tem de ir para um irmão); censos 117/117. Binário
do smoke refeito por último, sem o registo temporário.

## ABERTO (nomeado)

- O `Boundaries` (rota híbrida) não foi tocado: o sufixo na placa pode precisar de cada tique da
  fronteira — por medir antes de cortar.
- `cook_taps_only` (rota toda na placa) também coze todas as tomadas; lá não há tiques recuperados.
- Os abertos anteriores continuam: `opt-level` da `ph2d-contact` por medir com a máquina livre; docs
  > 40 KB por partir (`115_…`, 170 KB); §8–§9 do handoff de integração do ciclo 20.

## Próxima janela

Integrar e enviar só por ordem do Enio. O §10 do
[handoff de integração](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20.md) já resume esta onda.
