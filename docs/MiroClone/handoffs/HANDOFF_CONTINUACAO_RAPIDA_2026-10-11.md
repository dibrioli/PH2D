# CONTINUAÇÃO — `line/MiroClone` onda «linha RÁPIDA» (2026-10-11)

> Para a janela NOVA que abre a 1.ª onda depois da W13. Rota `/pd-linha-assumir`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current` ANTES de ler.
> Ordem do dono: [`PROMPT_LINHAS_RAPIDAS_2026-10-11.md`](../../archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md)
> — leia §1, §2 e só o §3.Miro. Esta onda vem ANTES da fila da linha.

## As regras (R1–R9, copiadas do prompt §2)

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

- **W12 + W13 FECHADAS, por integrar** (não integre, não pushe): 22 commits sobre `8f20c60de`, HEAD do fecho
  `0a5ee03f5` (o commit do handoff) — [handoff W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md). Portão final
  17 953/17 953. Smoke da W13 (cena 14) por fazer pelo dono. Esta onda vai POR CIMA na mesma branch; se o integrador já
  levou a W12+W13 (`git cherry main HEAD | grep -c '^+'` = 0), `git reset --keep main` primeiro.
- Binário `--profile smoke` quente na worktree; não o invalide sem o refazer no fim (DIRETRIZ §1.5.9 item 9).

## A tarefa (prompt §3.Miro), por ordem

1. `bash scripts/agent-loop-profile.sh 10` → os números de ANTES (o fecho da W13 mediu, 20 sessões: paralelismo 1,14 ·
   `test`:`check` 2,6× · `Edit` 30 % · contexto relido 402 mil · respostas/sessão 206).
2. `ph2d-editor-core`: `cargo nextest run -p ph2d-editor-core --status-level pass` → os 10 mais lentos. O prompt mediu
   `screens::hero::tool_bar::tests::nenhuma_face_de_pulldown_da_coluna_e_cortada` em **39 s** (de 45 s). Veja de QUEM é
   (`git log --format='%h %s' -- <ficheiro>`): se for de outra linha, registe-o no handoff e NÃO mexa; se for do
   Quadro, R9 (meça A/B pelo §2.1 antes de qualquer `opt-level`).
3. Os testes do Quadro medidos no fecho da W13 já cabem no R9: `board_bar::tests::history` 10 testes em 4,2 s;
   `the_history_scene_does_what_its_lessons_say` 0,3 s; `ph2d-board-sync` lib 50 testes; o fotógrafo é `#[ignore]`.
   Confirme com o passo 2 (filtre `board_`), não reescreva o que já está abaixo de 10 s.
4. `02_plano.md` (38 KB) foi lido 38 vezes: só por `grep -n` (R7). Nada a partir.
5. Handoff com a tabela antes × depois e este bloco de regras no topo; PARE.

## Lembretes medidos desta linha (W12–W13)

- O lento aqui é o AGENTE (o prompt §3.Miro): janelas de 500+ passos, edições por `python3 .replace()` e `.rs` inteiros
  por `Write`. A W13 fez o mesmo (as edições multi-trecho foram por script com `assert`) — é o hábito a cortar (R2).
- O shell é **zsh**: uma variável com vários argumentos não se parte (`for f in "a b"` passa UM argumento) — escreva os
  comandos por extenso.
- Comandos pesados vão por `bash scripts/ph2d-run.sh` (o guarda recusa o cru). Um filtro de teste que casa zero diz
  «0 passed»: confira a contagem.
- Delegue o portão ao `verificador` e a prova de mutação ao `mutacao`, em fundo; não edite a árvore enquanto eles correm
  (a mutação altera os mesmos ficheiros; o portão mede uma árvore fixa).
