# CONTINUAÇÃO — `line/MiroClone` depois da onda «linha RÁPIDA» (2026-10-11, fechada)

> Para a próxima janela da linha. Rota `/pd-linha-assumir`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current` ANTES de ler.
> A onda «linha RÁPIDA» ([continuação](HANDOFF_CONTINUACAO_RAPIDA_2026-10-11.md), prompt §3.Miro) está FECHADA:
> mediu, não mudou código. A próxima janela volta à fila da linha.

## As regras (R1–R9, do prompt `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` §2 — valem daqui em diante)

- **R1 — Janela curta.** Uma onda = uma janela. Passou de ~150 chamadas ou fechou a onda: commit, handoff de
  continuação ≤ 8 KB, PARE.
- **R2 — Edite pela ferramenta `Edit`.** Arquivo novo = `Write`; existente = `Edit`, nunca `Write` por cima. Script só
  para a MESMA troca em N arquivos, com `assert` de contagem.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`; só então teste.
- **R4 — Teste dirigido por NOME no laço.** `bash scripts/cargo-test-narrow.sh <crate> <filtro>`. Suíte inteira e
  `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- **R5 — Nada de `until … sleep` à frente.** > 2 min: `run_in_background` e trabalho independente.
- **R6 — Chamadas independentes na MESMA mensagem.**
- **R7 — Doc > 40 KB nunca inteiro:** `grep -n` e `Read` com `offset`/`limit`.
- **R8 — Escreva menos.** Comentário: a lei em 1–3 linhas. Resposta ao Enio: curta.
- **R9 — Nenhum teste da suíte normal passa de 10 s em `dev`.** Cura: (a) `opt-level = 2` na crate quente (prompt
  §2.1, medido A/B); (b) fixture menor que mantém a lei (prova de mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão
  para `#[ignore]`. Lei afirmada nunca vai para `#[ignore]` por ser lenta.

## Estado da linha

- **W12 + W13 FECHADAS, por integrar** (não integre, não pushe): [handoff W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md),
  portão final 17 953/17 953. Smoke da cena 14 por fazer pelo dono. Base = `main` `8f20c60de` (o `git rebase main`
  desta janela: «atualizado»). Se o integrador já levou a linha (`git cherry main HEAD | grep -c '^+'` = 0),
  `git reset --keep main` primeiro.
- Binário `--profile smoke` quente (2.ª corrida desta janela: `Finished \`smoke\` profile [optimized] target(s) in 0.44s`,
  zero `Compiling`).
- Esta onda NÃO mudou código: só este handoff.

## Antes × depois (o laço do agente, `bash scripts/agent-loop-profile.sh`)

| régua | antes (10 sessões) | esta janela (1 sessão, onda de medição) | alvo |
|---|---|---|---|
| chamadas por passo | 1,13 (9 % com 2+) | 1,28 (28 % com 2+) | ≥ 1,5 |
| respostas por sessão | 254 | 20 | ≤ 800 |
| `test` : `check` | 268 : 139 (1,9×) | 4 : 0 (onda só de medição, zero edições) | ≤ 1,0 |
| edições por `Edit` | 33 % (494 por script) | — (0 edições de código; 0 por script) | ≥ 80 % |
| contexto relido por passo | 422 mil | 98 mil | ≤ 250 mil |
| contexto no início | 62 mil | 63 mil | ≤ 80 mil |

A régua `test : check` e a das edições só dizem algo numa onda que edita: a 1.ª onda de código depois desta é a
medição de verdade de R2/R3.

## Os testes, medidos nesta worktree (carga anotada: a máquina esteve a 50–120 de `loadavg` toda a janela)

**`ph2d-editor-core`** (`cargo nextest run -p ph2d-editor-core --status-level pass`): 2 171 passed, 18 skipped, 61,6 s
(carga 98 no fim). Os que passam de 10 s — **nenhum é do Quadro**; registados, NÃO mexidos (R9 é da linha dona):

| s | teste | dono (commit que o criou) |
|---|---|---|
| 51,8 | `screens::hero::tool_bar::tests::nenhuma_face_de_pulldown_da_coluna_e_cortada` | UI/UX — `17268ddb5` (coluna F9, 02/10) |
| 13,0 | `it architecture_as_chaves_partilhadas_nao_ganham_leitor_partilhado::a_mesma_regua_ve_o_precedente_substeps_na_arvore` | Motion — `b541c9157` (ciclo 6, 07/10) |
| 12,6 | `screens::hero::global_palette::tests::every_global_palette_item_is_dispatched_by_somebody` | UI — `0f54f3505` (paleta Ctrl+K, 12/08) |
| 12,3 | `it architecture_as_chaves_partilhadas_…::nenhuma_das_tres_chaves_e_lida_por_nome_fora_do_dono` | Motion — `b541c9157` |

O do Quadro mais lento ali: `board_bar::tests::history::the_list_pages_when_it_does_not_fit` — 9,4 s na suíte
(carga 98), **5,6–5,9 s sozinho** (3 corridas, carga 113–122). Cabe no R9; não reescrito. 144 testes `board_` na crate.

**As 13 crates do Quadro** (`ph2d-app-board` + `ph2d-board-*`): 564 passed, 29 skipped, 8,4 s (carga 84→64).
Os dois mais lentos, medidos sozinhos 2× (carga 60 e 50):

- `ph2d-app-board live::tests::a_vanished_host_ends_the_guest_soon_and_its_old_code_never_locks_the_next_room` —
  **8,28–8,30 s, independente da carga = relógio**. É o `caio_after = 8 s` de
  [`live_tests.rs:227`](../../../crates/ph2d-app-board/src/live_tests.rs). ⛔ **Não troque por «o Caio chega quando a
  Bia acabou»:** a sala trava depois de `MAX_FAILS = 5` códigos errados (`ph2d-board-net/src/code.rs`,
  `LOCKOUT = 30 s`) e o reenvio é a cada `RECONNECT_EVERY = 2 s` (`live.rs`); os 8 s dão a um reenviador FANTASMA com
  o código velho tempo de somar os 5 erros antes de o Caio chegar. Chegando aos ~2 s, essa regressão passaria
  verde. Cura só se um dia passar de 10 s: tornar o intervalo de reenvio injetável no teste, mantendo «5 tentativas
  antes do Caio».
- `ph2d-board-sync session::tests::a_star_room_with_drops_ends_with_one_board` — 4,9–5,2 s sozinho (7,2 s na suíte):
  CPU, cabe.

Os de `ph2d-board-render` são ~2,9 s cada (o `frames_paint`), `ph2d-board-net` ponta-a-ponta 2,7 s.

## O que fica para a próxima janela

1. A fila da linha (o ABERTO está no [handoff W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md)).
   `02_plano.md` (38 KB) só por `grep -n` (R7).
2. Na 1.ª onda de código: edições por `Edit` (R2) e `check` antes de cada teste (R3) — a régua acima mostra se pegou.
3. Os quatro testes > 10 s do `ph2d-editor-core` são das linhas UI/UX e Motion: o integrador/dono decide quem os cura.
