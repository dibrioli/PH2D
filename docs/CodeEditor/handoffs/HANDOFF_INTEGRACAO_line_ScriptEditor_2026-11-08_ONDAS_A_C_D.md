# HANDOFF DE INTEGRAÇÃO — line/ScriptEditor: a Bancada, Ondas A, C e D (08/11/2026)

> Leitor: o agente integrador. Antecessor integrado: [d5, 8b, 8c e 1b](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-23_D5_8B_8C_1B.md).
> Continuação (mecanismo por commit): [ONDA_A](HANDOFF_CONTINUACAO_line_ScriptEditor_2026-11-08_ONDA_A.md) (cobre A, C e D).
> Plano: [13_plano_bancada.md](../13_plano_bancada.md) · decisões do dono 10 e 11: [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor` · branch `line/ScriptEditor` · HEAD `fa6e4fd6e0` (o commit deste handoff vem a seguir, só docs).
- Merge-base = `main` actual = `652b506b58` (conferido `git merge-base HEAD main`): **sem rebase hoje**; se o `main` andar, reconta-se o abaixo.
- 25 commits de `83194981e0` (plano 13) a `fa6e4fd6e0`; `git log --oneline 652b506b58..HEAD`. Agrupados: plano+decisões (2) · Onda A (7) · Onda C (4) · Onda D (12, dos quais 3 de fecho/auditoria: `bf658df9d0`, `b9538a9b6f`, `fa6e4fd6e0`).
- **Onda B NÃO feita**: espera a GEF02 no `main` (`ColorToken::layer(n)` e `paint_layer_outline` vivem só lá). A D foi feita antes da B por isso.

## 2. Superfície de colisão

```
Contratos §6 (nodegraph, tool*, vec*, imageio): git diff 652b506b58..HEAD --stat nessas pastas = VAZIO → intocados
PROJECT_SCHEMA: git diff 652b506b58..HEAD | grep PROJECT_SCHEMA = VAZIO → sem recontagem
Cargo.lock: +58 linhas (5 crates novas, nenhum pacote externo novo) → regenera-se
ADR: nenhum
```

**Crates novas (5, membros do workspace):**
- `ph2d-script-debug` — motor do depurador por `mlua::ffi` (Luau debug). **A única excepção de `unsafe` acrescentada**: `unsafe_code = "deny"` na crate, listada em `EXCECOES` de `architecture_every_member_inherits_the_workspace_lints`.
- `ph2d-code-viewmodel`, `ph2d-code-assist`, `ph2d-code-smoke`, `ph2d-code-scenes` — código MOVIDO de `ph2d-app-code`/`ph2d-app-components` para a catraca de LOC.
- Tabela-irmã nova `TETOS_DAS_CRATES_DA_FAMILIA` em `architecture_the_shell_only_shrinks.rs`: assist 3925 · scenes 666 · smoke 614 · viewmodel 304. `no_literal_color` passa a varrer também `ph2d-code-*` (excepto `-help`/`-format`).

**Foundational tocado (tudo aditivo):**
- `ph2d-editor-core`: campo novo `palco_largado` em `screens/hero/state.rs`; testes dos gates alargados; `DocMenu::Tab` (append) para o menu da própria aba.
- `ph2d-i18n`: chaves novas em `code_editor.rs` (6 textos `code.console.*` sem leitor saíram).
- `shells/desktop`: `keyboard.rs` — Ctrl/Meta seguem o carácter digitado (o `;` do ABNT2 já não comenta); `janela_do_jogo.rs` — fecha a janela do jogo quando o palco é devolvido numa paragem; `fase_fixed_step_clocks.rs` + `script_bridge::retem` (congela o jogo parado) — **shell com saldo 0 linhas**; `input_dispatch/keyboard_board.rs` +4 linhas (Ctrl+Shift+[ ] pela tecla física US).
- `ph2d-app-components`: `script_bridge` (`retem`, `ticks_run`), motores `Relogio::ate`/`retido_por`, scripts de cena no censo do formatador.
- `ph2d-script`: `debugLevel 2` para todo script; API do depurador em `host_debug.rs`; `defer_heard`; reexporta tipos do depurador; `SceneScripts::hook`.
- `scripts/pinned-helpers-next-to-exe.sh` (play.command e run-shell.sh chamam-no).

**O que um merge pode partir:** a lista de membros e o `Cargo.lock`; os tectos de LOC (somam entre linhas: `ph2d-app-code` 26 528 de 26 628, `ph2d-app-components` src 77 713 de 80 869 — contagem `wc -l` sobre `src/**/*.rs`, feita pelo agente em 08/11); a cena 12 do `PH2D_CODE_SMOKE` e o intervalo `1..12`; testes que contam menus/atalhos da shell (ordem do `DocMenu`).

## 3. O que a linha entrega

| Onda | O quê |
|---|---|
| A | Um problema, um lugar (Play e editor fundem; some ao consertar a linha); print com quem e quando; Ctrl+/ · Alt+F; contagem de usos na lista; a aba em cartões; luau-lsp/StyLua ao lado do binário. Cena 11. Smoke do dono OK |
| C | Menu da própria aba; caminho até o cursor; cartão Script options; Ctrl+K com funções; escolha ao soltar; «Make it a property»; botão direito com os consertos primeiro. Smoke do dono OK |
| D | O depurador: ponto de parada (clique/F9), cartão Stopped com valores, Next line, Pause on Error, valores ao vivo do script da aba aberta, menu Run, o jogo inteiro congela e o Continue retoma. Cena 12. **Smoke do dono PENDENTE** |

## 4. Números medidos (Onda D, gancho de linha do Luau — [plano 13 §4](../13_plano_bancada.md))

Kill-criterion escrito antes: **≤ 2 %** do quadro de 60 Hz, 50 objectos com script, variantes intercaladas no MESMO processo. Veredito: **PASSA em todas**.

| nível | variante | ms/quadro (mín) | + sobre hoje | % quadro |
|---|---|---|---|---|
| 1 | desarmado (hoje) | 0,1249 | — | — |
| 2 | desarmado | 0,1246 | −0,0003 | −0,00 % |
| 2 | F9 posto, não atingido | 0,1245 | −0,0004 | −0,00 % |
| 2 | parada em CADA linha do `update` | 0,1310 | +0,0061 | +0,04 % |
| 2 | passo-a-passo, gancho vazio | 0,1901 | +0,0651 | +0,39 % |
| 2 | passo-a-passo + ler locais a cada linha | 0,3134 | +0,1884 | **+1,13 %** |

Custo é por trabalho, não por quadro; por isso o passo-a-passo é só no fio parado. Sonda: `ph2d-script-debug/tests/sonda_gancho_de_linha.rs` (`--ignored`, perfil smoke).

## 5. Prova de fecho (08/11) — verde, por nome

- fmt + clippy `-D warnings`, 12 crates: `ph2d-script-debug`, `ph2d-script`, `ph2d-app-components`, `ph2d-app-code`, `ph2d-panel-code`, `ph2d-code-viewmodel`, `ph2d-code-assist`, `ph2d-code-scenes`, `ph2d-code-smoke`, `ph2d-i18n`, `ph2d-editor-core`, `ph2d-host-desktop`.
- `ph2d-app-code` inteira 253 passed / 3 ignored · `ph2d-script` inteira 182 / 5 · `ph2d-code-assist` 21 · `ph2d-panel-code` verde · `script_bridge` 14 · censo do formatador 1.
- editor-core: `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, `no_literal_color`, `hr15_no_hardcoded_ui_strings`, `architecture_every_member_inherits_the_workspace_lints`, `architecture_no_dependency_climbs_a_layer`, `arch_safe_clamp_only`, `architecture_claude_md_cabe_no_orcamento`. shells/desktop `file_loc_caps` 4.
- **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono: é do integrador).
- Build `--profile smoke` OK; cena 12 fotografada depois das curas da auditoria (parado na linha 14 no Circle, cartão com valores, «2 more waiting»).

**Auditoria (2 lentes, 19 achados), curas:**
- `b9538a9b6f` motor: `exit` corre sem paragens (`Debugger::quiet`); estado por fio esquecido; esperas do objecto retido não retomam; módulo velho desarmado na recarga; fio retido retomado por fora não falha; `held_error` solto quando o objecto sai; tiques e sinais do quadro da paragem; Next line e supressão por fio (laço `while` de uma linha pára a cada volta); singlestep herdado desligado.
- `fa6e4fd6e0` UI: Fullscreen/Window Play devolve o palco ao editor ao parar; clique na goteira abaixo da última linha não põe ponto; selo do problema sobre o ponto; marca oca quando o motor não armou; dicas da barra (engrenagem, função do caminho, alfinete do cartão); Continue/Next Line só activos parado, F8 = Continue parado, F10 não no erro; Ctrl+/ pelo carácter; censo do StyLua varre todos os `*_LUAU` (`JOGO_LUAU` reformatado, +8 linhas); catracas de LOC/cor cobrem `ph2d-code-*`.
- `bf658df9d0` fmt e `complete.rs` → `complete_usage.rs` (tecto de 700 linhas).

## 6. ABERTO (honesto)

1. No quadro em que há paragem, os relógios de passo fixo da shell (cabeça de reprodução, clips/containers, sprite anim, timers, vidas) ainda avançam todos os ticks desse quadro, porque correm antes dos scripts: zero excesso a 1 tick/quadro (60 fps), até ticks−1 ao recuperar atraso. Cura = reordenar `fase_fixed_step_clocks`.
2. Ponto de parada em linha vazia depois do último código arma no fim implícito do chunk (só corre no carregamento): ponto cheio que nunca pára no Play.
3. Redesenho do tooltip com o rato parado: não verificado à vista.
4. Sem teste ponta-a-ponta (precisa de winit): paragem em Fullscreen/Window e Ctrl+/ pelo carácter.
5. Da continuação: larguras por coluna na grelha de variáveis; âmbar sai cáqui; F8/F10 ao lado dos botões; pontos de parada seguem o texto por gravar; Cut/Copy/Paste fora do botão direito; alfinete do cartão é próprio; menus sem nós de acessibilidade; arrastar real da Hierarchy com setas não provado no ecrã; botão «Install» dos helpers é pergunta de produto; `break` no laço de ticks sem prova. (`tests/it/onda1.rs` fmt: curado no fecho — conferir `cargo fmt --check -p ph2d-app-components` e riscar.)
6. **Onda B** espera a GEF02 (cor por objecto, contorno de camada); o clique-para-saltar do Console fica sem casos até lá.

## 7. Para o integrador

- `CLAUDE.md` §5.1 CodeEditor: smoke `PH2D_CODE_SMOKE=1..12` (já posto nesta linha; num conflito, manter a frase e o intervalo).
- 5 membros novos; `Cargo.lock` regenera-se.
- O smoke do dono da Onda D (cena 12) acontece DEPOIS da integração (ordem do dono, 08/11).

## 8. Smokes (comando inteiro)

1. O depurador (Onda D): `cd /home/enio/Documentos/Projetos/PH2D && PH2D_CODE_SMOKE=12 cargo run --profile smoke -p ph2d-host-desktop` — a cena corre sozinha, sem F5. Deve parar no Circle com o cartão «Stopped» e as variáveis; Continue (F8) vai à Star; tirar o ponto e Continue mostra valores ao vivo («swing = 0.495 ×2»). Deu errado se o jogo não parar, ou se o editor ficar tapado em ecrã cheio.
2. Ondas A e C: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_CODE_SMOKE=11 cargo run --profile smoke -p ph2d-host-desktop` — um erro «self.sped is never set — did you mean speed?» numa linha só, «×3 in Play», contador 1, «3» na lista; passos de foto 1–5 mostram o menu da aba, o caminho até o cursor, Script options e o botão direito.
