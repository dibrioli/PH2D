# Continuação — line/ScriptEditor, Onda A da Bancada (08/11/2026)

> Plano: [13_plano_bancada.md](../13_plano_bancada.md) (aprovado 08/11; decisão 10 em
> [00_decisoes_do_dono.md](../00_decisoes_do_dono.md)). Worktree `Worktrees/line-ScriptEditor`, base `652b506b58`.
> Estado: **Onda A construída, aguarda o smoke do dono (cena 11).** Próximo: Onda B, que espera a GEF02 integrar
> (tokens `ColorToken::layer(n)` e `paint_layer_outline` vivem só lá).

## Commits
`83194981e0` plano 13 · `f8951fa1d2` decisões · `787f345d4b` A5 helpers · `65546ce6fb` Onda A · `631ec556e7` clippy · `a91498c37e` a aba em CARTÕES (dono: «tudo fica mais legal dentro
de molduras»; `cards.rs`) · `86de17d93a` o erro do Play some ao CONSERTAR a linha (âncora pelo `map_byte` + texto
da linha; volta no Ctrl+Z), não ao gravar.

Smoke do dono 08/11: **OK** (re-smoke das molduras e do «sumir ao consertar» também OK).

## Onda C (08/11, construída — smoke do dono OK 08/11)
`2f5a7adba5` menu da aba (`tab_menu.rs`, `DocMenu::Tab` append em editor-core — o File/Edit da app deixa de
mudar com a aba), caminho até o cursor, `options_card.rs`, Ctrl+K com funções (`PaletteScopes.named`, coluna de
atalho), escolha ao soltar (`drop_doc.rs offer_drop`), `promote.rs` «Make it a property» · `b3c559f9b8` botão
direito (`context_menu.rs`), acabamentos (grelha do cartão, Esc, Toggle Fold, largura das listas de acção, prévia
com «…», a soltura dá o teclado à aba em `surface_input.rs`). Cena 11 ganhou passos de foto 1–5.
Aberto: Cut/Copy/Paste fora do menu do botão direito (o clique não chega à área de transferência); alfinete do
cartão é próprio, não o do sistema de cartões; menus sem nós de acessibilidade; arrastar real da Hierarchy com
setas não provado no ecrã (só pelo caminho da shell no teste). Próximo: Onda B (espera a GEF02) e D (medir antes).

## O que a Onda A fez (mecanismo nos commits)
- **Um problema, um lugar** — `ph2d-app-code/src/play_problems.rs` (`merge`), `console.rs` (`Store::push(l, who)`,
  grupos `PlayProblem` por espécie+linha+mensagem), `doc.rs` (`set_play`, chave da cache dos problemas). Limpa ao
  recarregar bem (`frame.rs` `hear`) e no Stop (`ph2d-script` `Console::rewind`, só com o relógio a andar; `play_run`).
- **Print com quem e quando** — `ConsoleLine.at_us` (relógio da cena em `scene_tick_with`), etiqueta `ConsoleRow.tag`.
- **Ctrl+/** `edit.rs` `toggle_comment` · **Alt+F** `folds.rs` `toggle_fold_at_cursor` (+ Ctrl+Shift+[ ] pela tecla
  FÍSICA US em `shells/desktop/src/input_dispatch/keyboard_board.rs`, +4 linhas; no ABNT2 os colchetes físicos
  trocam, por isso o Alt+F é o atalho mostrado).
- **Contagem na lista** `list.rs` `users_by_script` (cena gravada — a mesma fonte da recusa de apagar).
- **Helpers ao lado do binário** `scripts/pinned-helpers-next-to-exe.sh` (play.command e run-shell.sh chamam-no);
  `Pin::resolve_from` testável.
- Cena **11** do `PH2D_CODE_SMOKE` (`ph2d-app-components/src/code_smoke.rs`). Foto conferida:
  1 linha «self.sped is never set — did you mean speed? — on Circle, Star, Hexagon · ×3 in Play», contador ⊗1, «3» na lista.

## Provas (por nome, verdes)
`onda_a` 5 · `um_lugar` 5 (inclui `parar_o_jogo_limpa_os_problemas_do_play`) · `console::` 3 · `problemas_lista` 8 ·
`smoke::` 1 · `paleta` 5 · `lista::` 11 · `teclado::` 3 · `despacho::` 8 · panel-code `estado` 5 · ph2d-script
`a_recarga_e_o_console` 11 · app-components `o_nivel_11` 1 · code-lsp `onde_mora` 5. clippy `-D warnings` das 6
crates: limpo. fmt: limpo.

## Aberto / para o integrador
- `CLAUDE.md §5.1` diz `PH2D_CODE_SMOKE=1..10` → passa a `1..11` (integração).
- O Console perdeu as linhas com número (nenhuma linha dele leva a uma linha do script agora); o clique-para-saltar
  do console ficou sem casos — rever na Onda B (o print pode levar a linha de onde veio).
- As linhas dos problemas do Play são do texto GRAVADO; com edições por gravar podem desalinhar.
- Teste do mapeamento físico `/ [ ]` na shell: não existe (só os testes do caminho da aba).
- Os 6 textos `code.console.runtime_error*/refused*/warning*` saíram do i18n (sem leitor).
- Item 5 do plano, botão «Install» na app: NÃO feito — pergunta de produto ao dono (descarregar da internet com
  um clique vs só no pacote); com os helpers ao lado do binário o caso de hoje está coberto.

## Onda D — 1.º passo: MEDIR (08/11, feito; o depurador NÃO foi construído)
GEF02 ainda não estava no main (`camadas_de_colisao.rs` ausente) → Onda D antes da B. Gancho confirmado no
`mlua::ffi` (Luau 0.740): `lua_breakpoint`, `lua_singlestep`, `debugbreak`/`debugstep`/`debugprotectederror`.
Crate nova **`ph2d-script-debug`** (excepção de `unsafe` nomeada no gate
`architecture_every_member_inherits_the_workspace_lints`: `src/ffi.rs` + a sonda) · `SceneScripts::hook(id,name)`
em `ph2d-script`. Sonda `tests/sonda_gancho_de_linha.rs` (`--ignored`, perfil smoke). Tabela e proposta:
[plano 13 §4, Onda D medida](../13_plano_bancada.md). Resumo: tudo ≤ 2 % (pior +1,13 % com todos a passo-a-passo
lendo locais); locais exigem `debugLevel = 2` (custo 0). **Aguarda o dono** decidir o âmbito da D18 antes de construir.
Provas: sonda verde (controlos), gate dos lints 3/3, clippy `-D warnings` ph2d-script + ph2d-script-debug limpo, fmt limpo.
Integrador: crate nova → `Cargo.lock` muda (+9 linhas).

## Onda D — o depurador (08/11, construída; aguarda o smoke do dono)

Decisão 11 do dono ([00_decisoes_do_dono.md](../00_decisoes_do_dono.md)): valores ao vivo SÓ do script da aba
aberta, só no Play; F9 / Continue / Next line / Pause on Error como propostos.

**Commits (mecanismo neles)**
- `1db668d254` motor. `ph2d-script-debug`: `Debugger` por VM; `debugbreak` cede a corrotina do gancho da cena
  (Luau «yielded inside a hook»); a fotografia linha/locais/self/pilha é tirada DENTRO do gancho (de fora o
  `savedpc` desconta uma instrução); Next line por profundidade; live só no fio vigiado. `ph2d-script`:
  `Park::Held`, ganchos seguintes esperam em `then`, fila pela identidade, `resume_held` numa passagem com lane,
  Pause on Error lê a pilha da corrotina morta, topo do módulo guardado e armado (desce às filhas), `debugLevel 2`,
  `host_debug.rs`.
- `52cfafd840` reexporta tipos · `ae95a79aed` congela o jogo: `script_bridge::frame` não tica com `is_held`;
  `script_bridge::retem` antes de `impacto.retem` na shell (net 0 linhas); `Relogio::retido_por` nos motores;
  Play fica aceso, Pause mantém, Stop rebobina e solta.
- `e06f285161` UI (`debug.rs`, `debug_marks.rs`): clique na marca/F9; cartão Stopped; F8 Continue só com cartão,
  F10 Next line; menu Run + Ctrl+K (Toggle Breakpoint, Continue, Next Line, Pause on Error, Remove All
  Breakpoints); toast de paragem bloqueada.
- `ee9aa99666` cena 12 do `PH2D_CODE_SMOKE` · `a6fd4273a2` tirar o ponto solta os Breakpoint em fila cujo
  ponto já não está armado (Step/Error nunca).
- `72368118c1`/`310f7a0442` acabamento: ponto vermelho (60 % da linha), seta âmbar, faixa âmbar `BAND_INK`,
  cartão numa linha com chip e botões, grelha de variáveis, Call stack.
- `4261dd56f9` tectos de LOC: crates novas `ph2d-code-viewmodel`/`-assist`/`-smoke`/`-scenes` (Folha);
  `ph2d-app-code` 30 582→26 314 (tecto 26 628); `ph2d-app-components` 81 110→80 568 (tecto 80 869).
- `85378e1ce6` o crumb deixa de reparsar (cache por geração da árvore + linha; era a causa dos vermelhos de
  `gramatica`/`dobras` desde `2f5a7adba5` da Onda C) e `um_lugar` desliga Pause on Error no setup.

**Fotos conferidas** (cena 12, passos 1–3): parado no Circle com variáveis; Continue → Star; tirar o ponto +
Continue → valores ao vivo «swing = 0.495 ×2», «Live values: Circle».

**Smoke:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=12 cargo run --profile smoke -p ph2d-host-desktop` (sem F5: a cena corre sozinha).

**Provas por nome, verdes:** `ph2d-script` `depurador_` 14; `ph2d-script` inteira 172 (motivo: deps);
`ph2d-app-code` `depurador` 5, `barra_da_aba`, `onda_a`, `teclado`, `despacho`, `um_lugar`, `gramatica`, `dobras`;
`ph2d-panel-code` `clique`; `ph2d-app-components` `o_ponto_de_parada_congela_e_o_continue_retoma`, `script_bridge` 14;
`ph2d-code-assist` 21; gates `the_shell_only_shrinks*`, `no_dependency_climbs_a_layer`,
`every_member_inherits_the_workspace_lints`. clippy `-D warnings` limpo nas crates tocadas.

**Aberto**
- Grelha de variáveis com colunas iguais (melhor: largura por coluna); cor de aviso do tema sai cáqui; F8/F10 ao lado do botão (o botão só tem um rótulo).
- Ponto de parada é linha do texto por gravar (pode desalinhar com o gravado).
- Quebra de `break` no laço de ticks sem prova (fixture 1 tick/quadro).
- `tests/it/onda1.rs` não está fmt-limpo (anterior); `ph2d-app-components` só 301 abaixo do tecto.
- Onda B continua à espera da GEF02.

**Integrador:** CLAUDE.md §5.1 CodeEditor: smoke passa a `1..12`; 5 crates novas (`ph2d-script-debug` + as 4) →
`Cargo.lock`.
