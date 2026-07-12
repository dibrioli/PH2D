# 54 — **Scroll (e barra arrastável) no add-menu** — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor
**Status:** implementado, testado (4 guardas), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** · **Foundational tocado:** nenhum

---

## 1. O problema

> *"precisamos de um scroll e barra de scroll arrastável na lista de nós"*

A biblioteca chegou a **86 tipos de nó**. O popup era **tão alto quanto a lista** — corria para fora da tela, e os
últimos quarenta nós eram **inalcançáveis** (a captura do Enio mostra a lista cortada na borda de baixo).

## 2. O conserto

- **O painel é CAPADO ao canvas** (`add_menu_panel`), e o que não cabe **rola**.
- **Roda do mouse sobre o menu = scroll da lista** — e **não** zoom do canvas. Uma roda que desse zoom no grafo
  **por baixo da lista que você está lendo** seria a coisa mais irritante do editor. Fora do menu, a mesma roda
  segue dando zoom como sempre.
- **Barra com thumb ARRASTÁVEL**, proporcional (a altura do thumb é a fração da lista que cabe na janela) e com um
  mínimo agarrável. **A trilha inteira é clicável** — um clique na trilha vazia salta o thumb até o cursor e passa a
  arrastar dali, que é o que toda barra de rolagem do mundo faz.
- **Sem barra quando tudo cabe.** Uma barra numa lista que não rola é um controle que **mente** dizendo que há mais.

## 3. As três armadilhas (cada uma virou guarda)

1. **A barra fechava o próprio menu.** A regra do popup era *"qualquer clique fora de uma linha fecha"* — então
   **pegar a barra de rolagem matava a lista que você queria rolar**. O press na barra é capturado **antes** da
   regra de dismissal (`grab_menu_thumb`), e o release deixa o menu **aberto e rolado**. Guarda:
   `dragging_the_thumb_scrolls_and_does_not_close_the_menu`.
2. **O thumb pulava ao ser pego.** Guardar **onde dentro do thumb** o cursor o agarrou (`grab`) é a diferença entre
   arrastar e ver a barra saltar para pôr o topo dela sob o cursor — o bug de scrollbar que todo mundo já encontrou.
3. **A linha que se vê é a linha que se clica.** O hit é a linha **interseccionada com a janela** — o **mesmo** rect
   a que a pintura se clipa. Linha rolada para fora da faixa **não é clicável**, por mais que o rect (não clipado)
   cobrisse o canvas. Guarda: `the_hit_follows_the_scroll_and_stops_at_the_band`.

## 4. Superfície nova

| Onde | O quê |
|---|---|
| `geom` | `add_menu_list` · `add_menu_max_scroll` · `add_menu_track` · `add_menu_thumb` · `add_menu_scroll_at` · `add_menu_row(panel, i, **scroll**)` |
| `state` | `AddMenu.scroll` · `Interaction::MenuScroll { grab }` |
| painel | **`interact_menu.rs`** (módulo novo: roda, thumb, resolve da linha) — o `interact.rs` estourou o cap de 600 LOC e **split, nunca allowlist** |

## 5. A lição

**Uma lista que cresce é um limite que ninguém testou.** O menu funcionou por 40 nós e quebrou em 86 — e não com um
crash: com **silêncio** (os nós simplesmente não estavam lá). Todo popup dimensionado pelo conteúdo é essa bomba-
relógio; o conserto é sempre o mesmo, e é barato: **cape ao container, role o resto**.
