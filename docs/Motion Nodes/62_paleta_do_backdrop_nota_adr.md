# 62 — A paleta do backdrop (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-13).
> **Gesto novo:** **R-click no cabeçalho de um backdrop** abre os 8 tons. Contrato congelado
> **intocado**.

## 1. Comece pela correção: a cor **já dava pra trocar**

O Enio pediu "cor do backdrop" porque **eu disse a ele que faltava**. Estava errado.

Selecionar um backdrop **já mostrava as linhas Title e Color no painel de params** o tempo todo
(`motion_bridge_backdrops::params_snapshot` → `apply_param_intent`), e o card de grupo já tinha a
linha **Name**. O que estava morto eram os `GraphIntent::SetBackdropTitle/SetBackdropColor` —
handlers **sem emissor**, **duplicatas** de um caminho vivo. Eu greppei o símbolo, não achei quem o
emitisse, e concluí *"a feature não existe"*. Provei o contrário **rodando o seam** e imprimindo as
rows.

A lição está em [[feedback_stale_comment_and_dead_code_lie]] (3º caso): **cace a CAPACIDADE, não o
símbolo** — *"quem emite X?"* e *"o usuário consegue fazer X?"* são perguntas diferentes, e só a
segunda importa. Um sistema com N caminhos pra mesma ação tem N−1 candidatos a **parecerem** mortos.

## 2. Então o que esta fatia entrega

Não uma capacidade — um **gesto**, e dois bugs.

**O gesto:** trocar a cor deixa de ser *"selecione o backdrop, vá até o painel de params, abra o
enum, leia oito nomes"* e passa a ser **R-click no cabeçalho dele** → os oito tons, ali, com o
**ponto de cada linha pintado no próprio tom** (a paleta É a sua própria pré-visualização: você
escolhe **olhando**, não lendo) e a linha do tom **atual contornada** — uma paleta que não diz onde
você está te obriga a clicar pra descobrir.

**O que um R-click SIGNIFICA agora depende do que está embaixo dele.** Sobre o canvas, um nó, um
socket, um fio: a biblioteca. Sobre o cabeçalho de um backdrop: a paleta. Antes, o botão direito era
curto-circuitado no topo do `apply` e abria a **biblioteca de nós sobre qualquer coisa** — inclusive
sobre um backdrop, onde uma lista de 88 tipos de nó é a resposta a uma pergunta que ninguém fez.

## 3. Os dois bugs que caíram junto

1. **O campo de busca era de TODO popup, não só da biblioteca.** O menu de portas de card (doc 57
   §6.1) vinha pintando uma caixa de busca que **não filtra nada** e que **tomava o teclado** —
   desde que a busca entrou (doc 59). Agora `Menu::has_search()` diz quem tem, e a geometria mede a
   partir disso: **88 tipos de nó é uma lista que você BUSCA; oito tons e um punhado de portas são
   listas que você LÊ.**
2. **O `SetBackdropColor` não empurrava passo de undo por conta própria** (contava com o bracket de
   sessão do painel de params). A paleta commita **uma vez**, no clique — então o passo de undo é
   empurrado **no braço do intent**, não dentro do `set_color`: o caminho do painel de params entra
   no mesmo setter e um push lá dentro contaria aquele em dobro.

## 4. Uma lista, três corpos

O `MenuRow` deixou de carregar uma `NodeUiCategory` e passou a carregar o **token da cor**: uma linha
não precisa saber o que a cor dela SIGNIFICA — num nó é a categoria, num tom é o próprio tom. E o
`menu_rows` continua sendo **a única fonte das linhas** (pintura + hit + geometria), que é o que
impede a linha que você vê de ser diferente da linha que você clica.

Os nomes dos tons saem das **matizes que os tokens de fato andam** (`graph-backdrop-1..8` é uma roda
OKLCH: 20°, 60°, 110°, 150°, 200°, 250°, 300° e um quase-neutro) — "Colour 5" não é um nome que
alguém use.

## 5. Os gates

`tests/the_backdrop_palette_actually_tints_it.rs` — pinta o painel de verdade e despacha o clique
como uma mão o faz: **aperta, desliza um pixel, solta** ([[feedback_a_click_is_a_press_that_drifted]]).
Prova que o R-click abre a paleta (e não edita nada), que a linha solta é o tom que pousa, que a
paleta **não toma o teclado** (e que a biblioteca **continua tomando**), e que o botão esquerdo segue
selecionando e arrastando.

**3 mutações mortas:** o R-click voltando a abrir a biblioteca · a paleta voltando a ter busca · a
linha resolvendo com off-by-one.

## 6. Superfície (para o integrador)

| Arquivo | O quê |
|---|---|
| `.../state.rs` | `MenuBody::BackdropTints { backdrop, current }` + **`Menu::has_search()`** |
| `.../snapshot_menu.rs` | `MenuRow.category` → **`MenuRow.dot: ColorToken` + `.selected`**; `TINT_NAMES` |
| `.../geom.rs` | `menu_chrome_h(menu)`; **`menu_list`/`menu_row`/`menu_max_scroll`/`menu_track`/`menu_thumb`/`menu_scroll_at` ganharam `&Menu`** (assinatura mudou — muitos call-sites) |
| `.../interact_menu.rs` | **`open_on_right_press`** (o Secondary saiu do `interact.rs`, que estava a 616/600) + o braço da paleta no `resolve_menu` |
| `.../paint_menu.rs` | header "Backdrop Colour"; a busca só na biblioteca; o ponto usa o token da linha; a linha atual é contornada |
| `.../snapshot_intent.rs` | o comentário do `SetBackdropColor` deixou de mentir |
| `shells/.../motion_bridge_intents.rs` | o braço do `SetBackdropColor` empurra o undo (o `set_color` não) |

**Aberto:** o `SetBackdropTitle` **não voltou** — o F2 (doc 61) é o gesto de nome, e o painel de
params continua sendo o outro caminho pro título.
