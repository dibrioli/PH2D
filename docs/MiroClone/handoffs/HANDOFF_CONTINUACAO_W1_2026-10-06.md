# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W1 fechada → W2 (2026-10-06)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessores:
> [`HANDOFF_CONTINUACAO_W0b2_2026-10-05.md`](HANDOFF_CONTINUACAO_W0b2_2026-10-05.md) (abas) e
> [`HANDOFF_CONTINUACAO_W0b_2026-10-05.md`](HANDOFF_CONTINUACAO_W0b_2026-10-05.md) (§1 decisões do dono).
> Plano: [`../02_plano.md`](../02_plano.md) — §2.2 (medidas da W1), §3 W1 (estado), §6 (recusas medidas).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`, base `a46c4c200` (rebase sem conflito; o
  `main` não andou nesta janela). Commits da W1: `c7557f5b5..c0a16b1a2` (6) + o deste handoff.
- O dono respondeu «ok» ao relatório da W0b e mandou seguir; o smoke da W0b não foi relatado como feito.
- ✅ **Smoke da W1 APROVADO pelo dono (2026-10-06)** — passos no §5.

## §1 — As peças (todas novas ou reescritas nesta janela)

| crate | o quê |
|---|---|
| `ph2d-board-model` | `ElementKind::Shape(Shape { kind: ShapeType (18), style: Style, text })`, `Element.angle`, **formato 2** com leitor do 1 (`legacy.rs`, provado com bytes REAIS do build W0), `History` por quadro (`apply`/`record`/`undo`/`redo`, 500 passos), `Rgba::readable_ink` (WCAG) |
| `ph2d-board-geom` (nova) | contorno por tipo: o catálogo `cook` do vectorial (**virado: ele é Y para cima**) + rectângulo/losango/triângulo daqui; canto redondo **medido no oráculo** (`oracle_tests`); caixa do texto por tipo (`text_frame`), `height_for_text`, `text_origin`, `hit` rodado |
| `ph2d-board-layout` (nova) | texto de DOCUMENTO (não passa pelo estilo da interface; `TextSystem::contexts`), quebra + centrado + quebra a meio da palavra, `TextCache`, `TextEdit` sobre o `PlainEditor` do parley, `line_bars` (greeking) |
| `ph2d-board-edit` (nova) | `Editor`: ferramentas, selecção (clique/Shift/rectângulo que CONTÉM), mover (Shift eixo, **Alt duplica**), redimensionar (Shift proporção, Alt centro, rodado mantém o canto oposto, vários escalam e os rodados forçam uniforme), rodar (Shift 15°), guias (`snap.rs`, Ctrl desliga), texto na forma, comandos (apagar, duplicar, tudo, Esc em 3 degraus, empurrar, desfazer, copiar/cortar/colar internos), `Overlay` |
| `ph2d-board-render` | forma + estilo + texto em cache; `RenderCache` (contorno guardado); caminho rápido do rectângulo simples; letra < 6 px = traço; forma < 4 px = ponto; `paint_overlay`; `style_stroke` (a ÚNICA lei do tracejado) |
| `ph2d-editor-core` | `board_view` (porta do rato, `Input { text, mods, now_ns }`, duplo-clique próprio, `Espaço`/meio/mão = vista, `select` para smokes), `board_keys` (porta do teclado: atalhos `V/1 H R/2 D/3 O/4`, `Delete`, `Ctrl+D/A/C/X/V/Z/Y`, setas, `Enter`; texto em edição), `board_bar` (barra curta + grelha de formas + barra de estilo; ids `SALT ^ (i+1)` com SALT de meio distinto), `documents::BoardLive` (editor, histórias, cache, espaço) |
| shell | `input_dispatch/keyboard_board.rs` (teclado do quadro DEPOIS dos modais e ANTES dos atalhos da cena; ponte da área de transferência para texto), `undo_route.rs` (`UndoOwner::Board` primeiro), `input_dispatch.rs` (a letra e os modificadores chegam ao quadro) |
| `ph2d-app-board` | `PH2D_BOARD_SMOKE=2`: fluxograma com texto + catálogo das 18 formas com o nome dentro, losango já seleccionado |

## §2 — Prova à saída

- Gate batched sobre o diff acumulado (base `a46c4c200`): `nextest-impacted` **16 701/16 701**, clippy
  workspace `-D warnings` (features do `ship.sh`), `check --workspace --all-targets` warnings=deny, fmt,
  machete, standalone-optional, workflow-packages, censos-da-árvore-combinada — verdes (o 1.º corrida deu 4
  vermelhos de literais/`gap`/`HashMap`, curados em `a836c43b4`; o pontilhado e o anel, achados na foto DEPOIS
  do gate, em `c0a16b1a2`, com clippy + gates de literais re-corridos verdes).
- Prova de mutação **13/13** (gestos + teclas + barra); a M2 (Shift no canto com o vertical a mandar)
  sobrevivia e ganhou teste. W0b continua 10/10.
- Oráculo Excalidraw: entradas `formas_canto_*`, `formas_contorno_elipse`, `formas_arredondamento_padrao`;
  `mede_cantos.py` reimprime as tabelas; gate `ph2d-board-geom::oracle_tests` (4 mutações sangram).
- Fotos da cena 2 (`fotografa_cena.sh`) apanharam 5 defeitos que nenhum gate via — todos curados com gate
  red-first onde a lei é testável (orientação do catálogo, pega de rodar tapada): ver os commits `4681dbdff`
  e `c0a16b1a2`.

- `target/*/incremental` reclamado (33 GB). Binário `smoke` quente, 2.ª corrida depois do reclamo:
  `Finished \`smoke\` profile [optimized] target(s) in 0.20s`, zero `Compiling`.

## §3 — ⏳ O que fica aberto (por ordem)

1. ~~Smoke do dono~~ ✅ aprovado 06/10.
2. **W2 — setas** (plano §3): o roteador `ph2d-vec-connect` reusado, cache de rota, pontos azuis.
3. Herdado da W0b: transbordo das abas (medir com 15–30 quadros); desfazer das operações de ABA (renomear,
   duplicar, apagar, reordenar quadros não se desfazem).
4. Pequenos, da W1, não pedidos pelo plano: o cursor do rato não muda sobre as pegas; copiar/colar de FORMAS
   é interno (entre quadros funciona, para fora do app não); trackpad: dois dedos dão zoom (a roda), não
   deslocam; o fundo do quadro é o do tema (`Bg1`) e a tinta de nascença a do tema — num tema claro um
   quadro feito no escuro tem a letra clara (a cor é dado do documento). O botão direito numa forma é
   tomado e não faz nada (sem menu ainda).
5. ⚠️ Medido e não feito: `100 mil` formas de longe custam ~8 ms de CPU por quadro; o índice de z mantido
   pelas ops foi MEDIDO pior (§6 do plano) — não o refazer sem outra medição.

## §4 — O que custou e não se repete

- ⛔ Dois ids derivados por XOR com saltos que só diferem nos bits de BAIXO colidem (`…_0003 ^ 1` = `…_0005
  ^ 7`): o «mais formas» era a aba do quadro 1. Saltos novos: meio distinto + gate de não-colisão.
- ⛔ O catálogo vectorial (`ph2d-vec-scene`) é **Y para cima** (`space.rs`, `1 − 2v`); quem o usa num mundo Y
  para baixo vira-o. Só a foto o mostrou.
- ⛔ Um traço de comprimento ZERO com ponta redonda não se desenha (o pontilhado sumia).
- ⛔ A cor de documento herdada do TEMA (tinta clara no tema escuro) some num preenchimento claro — escolher
  o fundo escolhe a tinta (`readable_ink`).
- ⛔ «Escrever com a forma seleccionada» colide com os atalhos de uma letra — recusado na W1 (§6 do plano).
- `screens/hero.rs` está a **699/700**: o próximo gancho lá pede corte antes.

## §5 — Smoke do dono (W1)

`cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=2 ./target/smoke/ph2d-host-desktop`

1. Abre o **Board 1** com um fluxograma (Start → Collect ideas → Good idea? → Build it → Done) e, por baixo,
   as 18 formas com o nome dentro. O losango «Good idea?» já está seleccionado: moldura azul com pegas e, por
   cima, a **barra de estilo**.
2. **Mover:** arraste o losango; perto da borda de outra forma aparece uma **linha vermelha** e ele «cola».
   (Com `Ctrl` não cola.)
3. **Tamanho e rodar:** puxe um canto (com `Shift` mantém a proporção); puxe a bolinha **acima** da moldura
   para rodar (com `Shift` roda aos saltos de 15°).
4. **Escrever:** duplo-clique numa forma, escreva uma frase comprida — a frase quebra e a forma **cresce para
   baixo**; `Esc` termina.
5. **Estilo:** com uma forma seleccionada, carregue nas bolinhas da barra (fundo, contorno), no traço,
   tracejado/pontilhado, cantos, opacidade e S/M/L/XL.
6. **Criar:** na barra curta à esquerda escolha o rectângulo (ou carregue **R**) e arraste no quadro; o «…»
   abre as 18 formas.
7. **Desfazer:** `Ctrl+Z` desfaz o último gesto DESTE quadro (e `Ctrl+Shift+Z` refaz); a cena (aba Scene) não
   é tocada.
8. **Vista:** segure **Espaço** e arraste para mover a vista; a roda aproxima.
9. **Errado se:** uma tecla mexer na cena por baixo, o `Ctrl+Z` desfizer outro quadro, a letra sumir numa forma
   colorida, ou um contorno pontilhado não aparecer.
