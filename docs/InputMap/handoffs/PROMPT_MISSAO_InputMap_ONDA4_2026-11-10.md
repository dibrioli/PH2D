# PROMPT DE MISSÃO — linha `line/InputMap`, ONDA 4 (cole inteiro numa janela nova)

> Escrito em 10/11 pela janela da onda 3 (smoke OK do dono em tudo, inclusive o botão Controls na barra do Play).
> Estado técnico: [handoff da onda 3](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA3.md) (§10 = o aberto) e
> [plano](../01_plano.md) §14. Ordem do dono, ao ver a lista do aberto: *«vamos resolver todos»*.

## 0 · Regras (copie a lista no briefing de CADA subagente)

As do [prompt da onda 3](PROMPT_MISSAO_InputMap_ONDA3_2026-11-10.md) §0, inteiras (formas proibidas, wrappers,
`rustfmt` antes do commit, `git -C <wt> commit -- <paths>`, sem fotografar enquanto alguém edita `src/`). E mais,
medido na onda 3:
1. Tectos no LIMITE exacto: `ph2d-app-input-map` 449/449, `ph2d-app-kanban` 32 175/32 175, `ph2d-app-sprite-anim`
   5 209/5 209, `action_bus.rs` 700/700, `color.rs` 699/700; painel tokens 2 074 (cada token de cromo = +31 px, com
   o comentário ⬆️ medido). Linha nova paga-se cortando/movendo, nunca subindo.
2. Antes de mandar a bateria, clippy `--keep-going` (lista TODOS os avisos de uma vez); os testes NOVOS também
   passam no clippy (`type_complexity`, `bool_assert_comparison`, `assertions_on_constants` apareceram 5×).
3. Um teste que só passa porque nada se move (área que cabe, aba fechada) não prova a lei: dê-lhe um controlo.
4. Widget PARTILHADO mudado (tooltip, etc.) ⇒ varredura por CONSUMIDOR antes de entregar.
5. Ponteiro numa foto: a ferramenta não move o rato — crie a cena que o põe pela porta do host
   (`crates/ph2d-input-map-scene` `rato_do_p`, cenas 20–22) e anote a foto (círculo + legenda) para o dono.
6. P1..P4 pedem `PH2D_GAME_SMOKE_DRIVE=editor`; ESCREVA-o no comando do smoke.

## 1 · Abertura

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && pwd && git branch --show-current`;
`git log --oneline HEAD..main | head` (andou ⇒ `git rebase main 2>&1 | tee target/rebase.log`); leia o handoff
da onda 3 (§2b, §7, §10) e este ficheiro.

## 2 · O que resolver (tudo de uma vez; teste vermelho → verde + foto quando é visual)

| # | Item | Onde | Lei / decisão |
|---|---|---|---|
| 1 | **Botão Controls na barra de TELA CHEIA** | `ph2d-editor-core` (os botões do topo em tela cheia; grep `TOPBAR_PLAYER_CONTROLS`, `transport_bar.rs`) | O mesmo botão, o mesmo caminho (`host::toggle`), a mesma dica que quebra; separador e moldura própria como na barra normal. |
| 2 | **Hover do botão Controls com a tela ABERTA** | shell `controles_do_jogador.rs` + `transport_bar.rs` | A tela fica com o movimento; o movimento SOBRE o botão também chega à barra (como o clique já passa). Hover instantâneo (D32). |
| 3 | **AltGr** | `ph2d-input/editor_keys.rs`, `input_map_ops.rs`, o fio dos modificadores na shell | MEÇA primeiro: que atalhos do editor usam Alt, e se o winit distingue o Alt direito/AltGr (`KeyCode::AltRight`, estado por tecla). Lei: só o acorde com o Alt que o EDITOR usa é recusado; AltGr (Alt direito a compor carácter) não bloqueia a ligação da tecla física. Teste por Mesa. |
| 4 | **Kanban: Calendário, Dashboard, gaveta da Timeline** sem deslize lateral | `ph2d-app-kanban` + `ph2d-kanban-views` | Onde o conteúdo rola na horizontal (meça cada vista), o deslize lateral rola como o Shift+roda; onde não rola, nada (e um teste com controlo). Tecto do Kanban no limite: corte. |
| 5 | **Planner: o calendário pode rolar ao contrário** | `ph2d-app-planner` (manda a roda CRUA ao Calendário, que espera a convertida) | Confirme por teste (mesmo gesto ⇒ mesmo sentido que o Kanban); se for, uma conversão só, num sítio. |
| 6 | **A face da tecla usa `KanbanCardRaised`** (token de dado de outra família) | `view_keyboard/keys_fit.rs` `REST_FACE`, `keys.rs` | Token NOVO de cromo `key-face` (aditivo, todos os temas) com o MESMO valor visual de hoje nos temas atuais (o dono aprovou o visual) e a lei medida das barras (≥ 3:1 em 12 temas × 12 cores). Foto lado a lado antes/depois: tem de ser idêntica. |
| 7a | **Segundas legendas do numérico** (End, ↓, PgDn, ←, →, Home, ↑, PgUp, Ins, Del, como na foto do dono `maquetes/06_teclado_notebook_dono.png`) | `view_keyboard/keys*.rs` | O dono decidira «fora» antes e agora pede TUDO: legenda secundária pequena, por baixo, menor e mais fraca que a principal, só onde cabe medida (nunca elidir; abaixo do piso some inteira). A ligação continua pela tecla física. Foto ao lado da do dono. |
| 7b | **Inspector vazio na aba Input Map** | o Inspector (outro módulo) + a aba | Estado da arte: com a aba Input Map ativa, o Inspector mostra a acção ABERTA (nome, grupo, quem lê, as ligações, Dead/Press) pelos mesmos cartões, ou diz numa frase o que fazer («Pick an action in the list»). Escolha, meça, foto. |
| 8 | **Revisão independente lado a lado** + auditoria 2 lentes | fotos 1600/1280/900 | Até a lista vir vazia. |

## 3 · Fecho

Como na onda 3 §3 (bateria por nome, clippy `--keep-going`, mutação das leis novas + as antigas, auditoria,
censos, `collision-surface.sh` por caminho absoluto, handoff `…_ONDA4.md` ≤ 15 KB apontando o da onda 3, incremental
apagado, binário smoke 2×). **Relatório ao dono** curto, sem jargão, smoke em passos numerados com o comando COMPLETO
(com as variáveis), fotos como links `[foto](Worktrees/line-InputMap/docs/InputMap/fotos/…png)`, anotadas quando há
rato. Depois PARE e aguarde a ordem de integração.
