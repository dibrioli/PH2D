---
name: feedback-context-menu-closes-on-down-repaint
description: Context-menu item click "does nothing" → the row is missing from populate_global_context_menu (not is_focusable); the repaint/close-on-Down theory is a red herring
metadata:
  node_type: memory
  type: feedback
  originSessionId: 946859c1-ddd7-4f30-9ba5-a6cc4547baac
---

Sintoma: um menu de contexto (ex.: handle Vector/Auto do falloff do Painter) abre,
mas clicar num item **não faz nada** — nenhum evento. No **Down** do item
`hit=item, menu_open=true`; no **Up** `hit=outro, menu_open=false`; ambos produzem
0 eventos.

**Causa-raiz REAL (corrigida 2026-06-21, commit `c32608a7`):** a linha do menu não
estava registrada em `screens/hero/pre_populate.rs::populate_global_context_menu`
(faltavam `CTX_MENU_FALLOFF_HANDLE_VECTOR`/`_AUTO`). TODA linha de menu simples
precisa de uma entrada `Plain` no `WidgetStore` ali — senão `is_focusable` rejeita
a linha, o **Down nunca arma `active`/`active_rect`**, e o **Up** (que resolve o
widget ativo pelo `active_rect` tirado no Down — `pointer_up.rs` ~209-217, **NÃO**
pelo hit-index vivo) não emite Click. É a gotcha [[feedback-panel-populate-register]];
as linhas irmãs `CTX_MENU_POINT_TYPE_*` (VectorPointType) estão registradas logo
acima. Fix = adicionar os 2 ids.

**O repaint era RED HERRING.** Como o Up usa o snapshot `active_rect`, o repaint
contínuo do Painter des-registrar o item do hit-index é irrelevante — o Click
dispararia se `active` tivesse sido armado no Down. Todos os outros menus
(hierarquia/inspector/tema/point-type) sobrevivem a fechar-no-Down + repaint
contínuo justamente por esse snapshot. O hit mudar entre Down e Up era coincidência
(o menu fechou), não a causa.

**NÃO** mexa no `pointer_down`/`pointer_up` global (fechar-no-Down) pra resolver
isto — a tentativa anterior (`a4456cae`+`1c182e96`: "só `hit.is_none()` dispensa no
Down") quebrou os menus do app inteiro e foi **revertida** em `d72873af` (viola
isolamento, CLAUDE.md §0.2). A correção é painter-/feature-local: registrar a linha.

**Why:** o teste `context_menu_item_click_emits_click_even_though_menu_closes_on_down`
era **falso-verde** — registrava o item como `Button` na mão e afirmava que "o paint
do menu registra" (falso: o paint só faz `hit_index.register`; a entrada de store
vem do populate como `Plain`). Por isso a omissão do populate passou batido.
Bench/unit-verde ≠ vivo ([[feedback-tool-unit-green-integration-dead]]).

**How to apply:** (1) clique de menu "não faz nada" → **PRIMEIRO** cheque se o id da
linha está em `populate_global_context_menu` (grep o id) — antes de qualquer teoria
de dispatch/repaint/geometria. Gate: `simple_row_context_menu_items_are_populate_registered`.
(2) Instrumente mouse→dispatch→chrome de baixo pra cima, mas saiba ler o sinal:
"Down produziu 0 eventos + `active` não setado" aponta pra is_focusable/registro,
não pra repaint. (3) Desconfie de teste de dispatch que registra o widget na mão em
vez de exercer o caminho real de populate. Relacionado:
[[feedback-measure-perf-symptom-scale]].
