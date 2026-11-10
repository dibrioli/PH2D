# PROMPT DE MISSÃO — linha `line/InputMap`, ONDA 3 (cole inteiro numa janela nova)

> Escrito em 10/11 pela janela da onda 2 (teclado de notebook + polimento pro + 2 rodadas pós-smoke, todas com
> smoke OK do dono). Leitor: a LLM que cumpre a onda 3 SOZINHA, do início ao fim. Estado técnico:
> [handoff da onda 2](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) (o que fica aberto está lá, §10) e
> [plano](../01_plano.md) §13 (contratos). Ordem do dono: *«Vamos corrigir tudo que for necessário de uma vez.»*

## 0 · Regras (iguais às da onda 2 — copie a lista no briefing de CADA subagente)

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap`, Modo L. Missão autónoma, sem parar e sem
perguntar; só pare nos casos do CLAUDE.md §0.2. Decisões técnicas e de acabamento são suas (estado da arte).
- ⛔ `cd X && …` (até `cd /tmp`), `bash -c`/`sh -c`, `rm -rf` com variável/glob, `kill` de PID alheio, escrever em
  `.git/`, `.claude/`, rc. ⛔ `sed -i`/`perl -i` sem `grep -c` na mesma linha; `replace()`/`re.sub()` sem `assert`.
- ⛔ Pesado cru: sempre `bash scripts/ph2d-run.sh` ou `cargo-check-narrow.sh` / `cargo-test-narrow.sh <crate> <filtro>`.
- ⛔ Laço sobre `pgrep -f`; vigia sem `timeout`; `git add -A`/`.`, `git stash`, `push`, `--force`, integrar, `ship.sh`,
  `suite-completa.sh`, `nextest-impacted.sh`; `#[allow]`; a palavra «bancada»; `spectacle`.
- O hook `commit-barato` recusa ficheiro fora do rustfmt: `rustfmt --edition 2024 <f>` num comando SEPARADO antes.

**Lições medidas (onda 1 + 2 — não as reaprenda):**
1. Teste de layout na ÁREA REAL da aba: `(308,28,988,890)`@1600×1000, `(297,28,690,604)`@1280×720 (+ uma estreita).
2. Censo de cortes nos dois modos de texto (`Default` e `CrispHeavyPlus`, o de fábrica).
3. `fotografa_cena.sh` recompila o smoke: ninguém edita `src/` enquanto se fotografa (nem mutação).
4. `mut.py`/mutação restaura com `git checkout HEAD`: comite antes de mutar.
5. `ph2d-app-input-map` está NO tecto (449/449); `shells/desktop/src/input_dispatch.rs` a 599/600; a shell só encolhe.
   Linha nova paga-se MOVENDO.
6. Agentes paralelos: ficheiros DISJUNTOS escritos no briefing; um commit apanhou hunks alheios 3× na onda 2 —
   `git -C <wt> commit -- <paths>` e confira `git diff --cached --stat` antes.
7. Hover = **D32 do dono: instantâneo, sem fade** (`docs/CartoesUI/01_decisoes_do_dono.md`). Movimento só pelo `UiMotion`.
8. Subagente tem 45 min IMPOSTOS: dê tarefas de ~30 min; retome o MESMO agente (`SendMessage`) para continuar.
9. Esta janela só decide, desenha e confere; passou de ~300 mil tokens por passo, delegue cada rodada inteira.

## 1 · Abertura

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && pwd && git branch --show-current`.
2. `git log --oneline HEAD..main | head` — se o main andou, `git rebase main 2>&1 | tee target/rebase.log`.
3. Leia o handoff da onda 2 (§10 aberto), o plano §13, `DIRETIVA_IMPLEMENTACAO.md`, e abra como IMAGEM
   `docs/InputMap/maquetes/06_teclado_notebook_dono.png` e `docs/InputMap/fotos/{b1_1600,b1_900,a1_1600,c4_1600}.png`.

## 2 · O que corrigir (tudo de uma vez; um item = teste vermelho → verde + foto quando é visual)

| # | Item | Onde | Lei / critério |
|---|---|---|---|
| 1 | **Fila F em meia altura**, como na foto | `view_keyboard/{geometry,keys,keys_fit}.rs` | Com os traços POR TRÁS da plaquinha (lei pós-smoke), meça se ~0,6 da altura cabe a legenda + ≥ 2 px de traço de cada lado; se não, o menor fator que cabe, com a medição ao lado. Nunca abaixo do piso de passo (`PITCH_FLOOR`). |
| 2 | **Traços por trás também no comando e nos chips «Also:»** | `view_keyboard/pad.rs`, `keys.rs` (Also) | A mesma lei da tecla: nome parado numa plaquinha, cores por trás, N cores nunca empurram o nome. |
| 3 | **Hover na tela «Controls» do jogo (F1)** | `ph2d-player-controls` (`host.rs`, `paint.rs`) + a porta da shell | A shell só TRADUZ o movimento do ponteiro para `host::pointer_move` (pague as linhas movendo); linha sob o ponteiro acende no quadro, Reset com hover/premido. |
| 4 | **Alt + tecla nunca liga sob a escuta partilhada** | `ph2d-editor-core` `input_map_ops.rs::capture_if_listening`, `ph2d-input/src/editor_keys.rs` | Hoje só Ctrl/F5 são recusados (o store não guarda Alt). Guarde Alt como guarda Ctrl e recuse o acorde. Teste por Mesa. |
| 5 | **× premido no tema claro mais escuro que o hover** | `ph2d-tokens` (`tokens.json` nos 4 temas) + `view_list/looks.rs` | Token novo `danger-press` (aditivo, os 4 temas, contraste medido) ou a derivação certa; premido sempre mais fundo que hover nos dois temas. |
| 6 | **Folga no `input_dispatch.rs`** (599/600) | `shells/desktop/src/input_dispatch.rs` | Mova um bloco coeso para a família certa (HOWTO `partir_uma_familia_da_shell` §2); a catraca só desce. |
| 7 | **O deslizar horizontal do trackpad nas OUTRAS abas** | `ph2d-editor-core` `board_view.rs`/`surface_input.rs` | A onda 2 passou a mandá-lo a toda aba como Shift+roda. Prove que nenhuma aba (Kanban, Code, Planner, Sprite, Pixel, SFX) regrediu: teste por aba que já tenha Shift+roda; onde não tenha, nada muda. |
| 8 | **Revisão independente lado a lado** depois de 1–7 | fotos `docs/InputMap/fotos/` (1600, 1280, 900) | Revisor sem as suas conclusões; corrija até a lista vir vazia. |

**Fica de fora por DECISÃO (não reabra):** legendas secundárias do numérico (a ligação é pela tecla física, com
ou sem Num Lock); a frase do hover da B no cabeçalho (R7b); sem ◆ em Dead/Press; o Inspector vazio (outro módulo).

## 3 · Fecho — `/pd-linha-fechar`

1. `cargo fmt` · `PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo clippy --workspace --all-targets -- -D warnings`
   · `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` · `input_map::` inteiro e os
   portões tocados por nome (`architecture_workspace_file_loc_cap`, `the_shell_only_shrinks`, `enum_order_matches_svgs`).
2. **Mutação** (agente `mutacao`): as leis novas (fila F, traços por trás no comando, hover da tela do jogo, Alt
   recusado, premido > hover) + as da onda 2 (piso de passo sem `Narrow`, painéis arrastam com a aba aberta, plaquinha
   parada) + as quatro da onda 1. Toda sobrevivente vira teste.
3. Auditoria ≥ 2 lentes (correcção e fios; aparência lado a lado) com o TEMPLATE da DIRETIVA §3.
4. `git rebase main` (log) · `bash scripts/censos-da-arvore-combinada.sh` · `collision-surface.sh` (caminho absoluto).
5. Handoff novo `docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_<data>_ONDA3.md` (≤ 15 KB, aponta o da onda 2).
6. `target/*/incremental` apagados um a um; binário smoke 2× (2.ª sem «Compiling»); `pgrep -af 'ph2d|cargo|rustc'`.
7. **Relatório ao dono** curto, PT-BR, sem jargão, smoke em passos numerados (comando com `cd` · onde clicar · o que
   acontece · como saber que deu errado), fotos como links `[foto](Worktrees/line-InputMap/docs/InputMap/fotos/…png)`.
   Depois PARE e aguarde a ordem de integração.
