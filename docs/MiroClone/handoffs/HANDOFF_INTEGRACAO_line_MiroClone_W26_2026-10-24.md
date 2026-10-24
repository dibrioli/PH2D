# INTEGRAÇÃO — `line/MiroClone` W26: o aviso do topo sem lugar parte-se em duas linhas (2026-10-24)

> Leitor: o integrador. Antecessores (por integrar, na MESMA linha): [W25](HANDOFF_INTEGRACAO_line_MiroClone_W25_2026-10-24.md) →
> [W24](HANDOFF_INTEGRACAO_line_MiroClone_W24_2026-10-23.md) — integra-se a linha inteira (W24 + W25 + W26) de uma vez. **A linha PAROU
> aqui: não integrou nem fez push.**

## 0 — Base e commits

- A linha foi rebaseada sobre o `main` `b48f8b9512` (limpo, zero ficheiros em comum com o que o `main` andou desde `972f411a6f`).
- W26: `30132d4f65` (a cura + os gates) e este handoff. 3 ficheiros de código, todos em `crates/ph2d-editor-core/src/screens/hero/`.

## 1 — Feito

O ABERTO 2 do W25 §7, por ordem do dono (24/10, «vamos acabar tudo que está em aberto»): numa janela onde nenhuma altura leva o aviso do
topo numa linha (800/600 px com os painéis da direita), ele ficava no topo POR CIMA do «Frames».

- `board_alert::banner(ts, (area, keep), text, extra) -> (Rect, Option<usize>)` substitui `banner_rect`: tenta uma linha; sem vão,
  `split_in_two` parte no espaço que MINIMIZA a linha mais larga e procura outra vez com `(largura dessa linha + extra, Xl2 + Sm × LineHeight::Normal)`;
  só sem lugar nenhum fica no topo, por cima, numa linha. A procura do lugar (`spot`) é a do W25, sem mudança.
- `board_alert::paint_lines` pinta a frase numa linha ou nas duas, cada uma centrada na sua banda. As duas portas usam-nas: o aviso com
  botão (`layout` devolve agora `(faixa, botão, split)`; o botão continua à direita, com a altura da faixa) e o da recusa (`organize::paint_notice`).

## 2 — Contadores e contratos

`FORMAT_VERSION` 17 · `PROTOCOL` 11 · cenas `max_level` 29 — DELTA zero. No `main` de hoje: `PROJECT_SCHEMA` **193** (a linha não o toca) e
ADR livre **0190** (o 0189 é da caneta). Sem i18n nova, sem `Cargo.*`, nada de contrato §6.

## 3 — O que um merge pode partir

- `board_alert::banner_rect` saiu (único chamador fora do ficheiro: `paint_notice`); `board_alert::layout` devolve um 3.º campo.
- O aviso numa janela estreita com os painéis da direita fica mais ALTO (duas linhas) e mais estreito — gates de posição de outras ondas reconferem.

## 4 — Gates e provas de mutação

- `the_top_notice_is_never_under_the_chrome` passa a exigir `no_room` vazio (era só `eprintln!`) — VERMELHO antes da cura: `["800 direita", "600 direita", "600 direita"]`.
- Novo `a_notice_without_room_breaks_in_two_lines_that_fit` (`board_bar_overlay_tests.rs`): 1366 sem painéis = uma linha; 800/600 com os
  painéis = duas, a mais larga ≤ `label_budget(faixa)` (sem elisão) e as duas recompõem a frase.
- Mutações (à mão, restauradas, `grep -c MUTACAO` = 0): M8 sem o ramo das duas linhas · M9 parte no PIOR espaço (`max_by`) — 2/2 vermelhas nos DOIS gates.

## 5 — Bateria (desta onda)

| o quê | resultado |
|---|---|
| `cargo fmt -p ph2d-editor-core --check` | ✓ |
| clippy `-p ph2d-editor-core --all-targets -D warnings` | ✓ |
| testes por nome: os 2 gates · `notice` (5) · `alert` (3) · `send_to_board` (5) | ✓ |
| tecto de LOC: `board_alert.rs` 243 · `board_bar_overlay_tests.rs` 352 · `board_bar_organize.rs` 361 (tecto 700) | ✓ |

A suíte inteira (`nextest-impacted`) fica para antes da integração (dono, 23/10) — a última verde da linha é a 3.ª corrida do W25.

## 6 — Smoke (o dono; da pasta da LINHA)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=29 ./target/smoke/ph2d-host-desktop`
2. Abra os painéis da direita (Frames, Comments, Minimap) e estreite a janela até uns 800 px.
3. Selecione uma forma e use «Move to board…» (enviar para outro quadro): aparece o aviso com «Undo».
4. Deve acontecer: o aviso desce para um lugar livre; se não houver lugar para ele numa linha, aparece em DUAS linhas, inteiras, fora do «Frames».
5. Deu errado se: o aviso tapa o painel «Frames», ou uma das linhas aparece cortada com «…».

## 7 — ABERTO depois da linha

1. ~~Smoke do dono do §6~~ — ✅ **APROVADO pelo dono em 24/10** («smoke OK»). Ordem seguinte do dono: integrar ao `main`.
2. `a_cena_3_conduzida_passo_a_passo` (`shells/code-scenes`): é da linha do CodeEditor (W25 §7.3), não desta.
