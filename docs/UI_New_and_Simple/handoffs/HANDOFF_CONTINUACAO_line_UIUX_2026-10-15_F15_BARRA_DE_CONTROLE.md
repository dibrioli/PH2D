# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 15/10 — F15 (a ④ recebe Arrange/Align) feita; próximo: a barra de controle

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `0aab2c1cb6`. Árvore limpa depois deste commit.
  Janela anterior fechou por TAMANHO (~350 mil tokens/passo), não por fim de trabalho.

## 1 — O que esta janela fez (smoke do dono OK nas duas primeiras; a 3.ª está por smoke)

| commit | o quê |
|---|---|
| `e4b067baab` | **Arrange e Align saem do painel do vetor para a ④.** 1.ª linha: Duplicate · To Front · Forward · Backward · To Back \| Flip H/V · Rotate CCW/CW. **2.ª linha** (`ToolBars::actions_below`, `float_bar::strip_rows_at`/`pinned_strip_rows`): Align (sel ≥ 2) + Distribute (sel ≥ 3). 12 ícones Lucide novos. `paint_align.rs` apagado, `paint_arrange.rs` → `paint_fill_type.rs`. O campo Z do vetor e a cadeia dele na shell (`pending_vec_z`, `set_z_index`) saíram — o Z é o do Inspector no modo Objeto (mesmo `ZIndexOverride`). |
| `60b0ca86f2` | **Cena 22** (`PH2D_OBJECT_MODE_SMOKE=22`, `crates/ph2d-app-vec/src/vector_mode_alinhar.rs`, estágios 220–224): três formas, Select, as três selecionadas. |
| `9fe6f7162c` | **A ④ não se esconde mais** (dono): saem o ✕ (`SELECTION_ACTIONS_OFF`), *View ▸ Selection Actions*, `actions_off` e a chave do `layout.txt` (ficheiro antigo cai no `_` do `parse`). Fica o fixar. **Live Boolean** na ④ (sel ≥ 2), à frente das booleanas, `IconId::Link`, id do estado OPOSTO. `00_DECISOES` D10 #4 anotado. |
| `a39cfb1be6` | **Dentro da booleana viva** (1 forma selecionada): os 4 `VECTOR_BOOL_SHAPE_*` (aceso o da forma, dica `«<forma> · <verbo>»`) e o `VECTOR_BOOL_APPLY` (`IconId::Check`). A base (BSE) não tem verbo — regra da shell. **Por smoke do dono.** |

Gates novos: `the_arrange_verbs_ride_the_first_row_and_align_the_second`, `the_panel_no_longer_paints_arrange_nor_align`,
`the_live_boolean_switch_rides_in_front_of_the_booleans`, `a_shape_inside_a_live_boolean_gets_its_verbs_and_the_apply`
(`ph2d-panel-vector/tests/it/the_bars_carry_the_gesture.rs`), `the_second_row_of_the_action_bar_sits_under_the_first_centred`
(`float_bar_tests.rs`), `the_action_bar_pins_and_never_hides` (`ph2d-panel-registry-init`). Testes por NOME verdes; a
**suíte e o clippy do fecho NÃO correram** (5 commits; correm no fecho, `PH2D_SUITE=fecho`).

**Em aberto para o fecho:** CLAUDE.md §5.1 UI/UX — smoke `6..22` e o «Último:». Duas perguntas ao dono sem resposta:
(a) o Duplicate também sai da ④ (está lá e no menu da Hierarchy)? (b) o «Live: Off/On» da secção Boolean do painel sai?
Não é preciso: o `frame_margin()` (F8) conta UMA linha da ④; com o Align a ④ tem duas — com ≥ 2 formas enquadradas
ela pode cair por baixo da seleção. Medir antes de mexer.

## 2 — O PRÓXIMO pedido do dono (15/10, verbatim)

> *«Agora vamos criar a barra de ajustes no topo como no illustrator, principalmente com os ajustes do stroke.
> Primeiro investigue e depois planeje»* — referência: [`R_illustrator_barra_de_controle_2026-10-15.png`](../vetor_nova_ui/referencias/R_illustrator_barra_de_controle_2026-10-15.png)
> (a *Control bar* do Illustrator: nome do objeto · Fill · Stroke · peso `16 pt` · perfil *Uniform* · pincel *Basic* · Opacity · Style).

⛔ **Investigar e PLANEAR, não codar** — o plano vai ao dono antes da 1.ª linha de código.

Pontos de partida já verificados nesta janela (confira o CÓDIGO antes de acreditar numa ausência — §5.0):
- **A fila ② JÁ É uma barra no topo** no layout *Vetor* (foto `fotos/F15_cena22_arrange_e_alinhar_na_barra.png`):
  FILL · LINE · W · Corner round · MOVE/ROT/SCALE/PIVOT · SPACE · VIEW · UNDO/REDO · Edit Mode · Snap · Path.
  Produtor: `crates/ph2d-panel-vector/src/bars.rs` `tool_bars()` (`row_lead`/`row_tail`, `style_chips`,
  `gesture_chips`, `shape_chips`); a lei *«a ORDEM é a prioridade — o `⋯` leva a cauda; à frente o que só funciona na fila
  (campos, amostras)»* está no comentário de `tool_bars()`. iPad 12,9" dá ~730 px (F0).
  ⇒ a pergunta do plano é provavelmente *«o que a ② ganha e como»*, não *«uma barra nova»*.
- **O que o Stroke do painel tem hoje** (secção Stroke, mesma foto): Width, cor, Opacity, Cap (Butt/Round/Square),
  Join (Miter/Round/Bevel), Align (Centre/Inner/Outer), Dash, Gap, Start/End (pontas), Head Size, Head Round, Both Ends.
- Oráculos: CLAUDE.md §0.9 e `docs/_ComoInvestigarApps/`; referências já escritas: `vetor_nova_ui/referencias/R3_illustrator_curve_rive_inkscape.md`
  e `R1_affinity.md` (a *context toolbar* do Affinity é a irmã mais próxima). Plano D10: `vetor_nova_ui/03_plano.md`.
- O que a D10 já decidiu e não se re-litiga sem o dono: `00_DECISOES_DO_ENIO.md` (D10, tabela das seis decisões).
