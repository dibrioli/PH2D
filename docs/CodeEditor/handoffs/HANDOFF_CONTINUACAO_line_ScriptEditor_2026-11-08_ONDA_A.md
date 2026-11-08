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
