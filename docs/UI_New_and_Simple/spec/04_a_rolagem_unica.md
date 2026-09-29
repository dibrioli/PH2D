# 04 — A rolagem é UMA: uma porta para todo painel, e inércia no arrasto (2026-09-29)

> Ordem do dono (29/09): *«vamos para Rolagem com inércia. O Scroll e a barra de scroll não estão
> padronizados e centralizados e deste modo cada painel tem um comportamento diferente do outro.
> Alguns painéis o scroll nem funciona. vamos corrigir e padronizar para todos.»*
>
> Este documento é o plano. O censo que o motivou foi feito por leitura de código (um agente
> Explore, 27 painéis) e os achados graves foram **conferidos à mão** antes de entrarem aqui
> (marcados ✔). O que não foi conferido diz-o.

## §1 — O censo, em números

- **27** `impl Panel` + quatro superfícies de chrome que rolam (Input Map, paleta de comandos,
  popovers de dropdown, a coluna de catálogos do navegador de assets).
- **Quatro** implementações de rolagem diferentes: a das tabelas do `WidgetStore` (quase todos),
  a da timeline (estado próprio, barra-`Slider`, sem suavidade), a do Input Map (campo próprio,
  barra que não arrasta) e a da paleta (campo próprio, dica de 3 px).
- **Cada painel escreve à mão a mesma sequência**: `push_clip` → corpo com `y − scroll` →
  `pop_layer` → `paint_scrollbar` → `register(<ID>, …)` → `set_panel_content_h/visible_h` → clamp.
  É nesta cópia que os defeitos moram.

## §2 — Os defeitos, com endereço

| # | defeito | onde | conferido |
|---|---|---|---|
| D1 | **Arrastar o polegar anda depressa demais** — o painel regista o POLEGAR e o `pointer_down` lê a altura do rect acertado como a da TRILHA (`track_h: rect.h`). Com o polegar no mínimo, `drag_range` colapsa a `1 px` e um pixel de arrasto salta ao fim. | `pointer_down.rs` (`begin_scrollbar_drag`) × 14 painéis (`register(<X>_SCROLLBAR_ID, thumb)`) | ✔ (padding) |
| D2 | **A Hierarquia não publica `visible_h`** ⇒ o polegar nunca arma e o arrasto no corpo está desligado; a roda usa uma 3.ª altura (`panel.h − 60`). | `ph2d-panel-hierarchy/src/paint.rs` | ✔ |
| D3 | **O painel de ossos pinta com o id do Vector** (`VECTOR_SCROLLBAR_ID`) e **não está na lista da roda** ⇒ a roda sobre ele dá zoom na câmera, e arrastar a barra dele rola o Vector. | `ph2d-panel-skeleton/src/paint.rs`; `forwarding.rs::cursor_over_hero_panel` | ✔ |
| D4 | **O Inspector perdeu o recorte do corpo** (o `push_clip` saiu em `41e6597bf`), e o `close_body` ainda faz `scene.pop_layer()` **sem par**. | `ph2d-panel-inspector/src/paint_body.rs` | ✔ |
| D5 | **O `HitIndex` só é recortado em 5 sítios** ⇒ nos outros, linhas roladas para fora continuam clicáveis debaixo do cabeçalho. ⚠️ No Inspector a cura foi tentada duas vezes e revertida (auditoria de 31/08, «A4»): *os gates de costura clicam em widgets fora do corpo visível* — curar pede reescrevê-los para **rolar antes de clicar**. | por painel | parcial |
| D6 | **Duas fontes de verdade para «este painel rola»**: a roda usa a lista À MÃO do shell (`cursor_over_hero_panel`); o arrasto no corpo deriva das alturas publicadas. D3 é o preço. | `forwarding.rs` × `panel_ops.rs::scrollable_panel_at` | ✔ |
| D7 | **A roda e o arrasto ignoram a ordem z** (`panel_at` percorre um `BTreeMap` por id) ⇒ sobre painéis flutuantes sobrepostos pode rolar o de baixo. | `panel_ops.rs::panel_at` | ✔ (código) |
| D8 | **O arrasto no corpo não é 1:1**: ele escreve o ALVO e o conteúdo segue pela mola da suavidade. | `pointer_move.rs` + `live.rs::tick_panel_scroll` | ✔ (código) |
| D9 | **Alturas velhas nunca se limpam** (ex.: o caminho de cena vazia do Sculpt3d publica o rect sem `content_h`). | `panel_ops.rs` | não |
| D10 | Entradas mortas no despacho (`MOTION_PARAMS`, `PAINTER_BRUSH_STUDIO`). | `scroll.rs`, `forwarding.rs` | não |
| D11 | A coluna de catálogos do navegador não é limpa quando o painel fecha. | `ph2d-panel-asset-browser` | não |
| D12 | **Não há inércia em lado nenhum**: soltar o dedo depois de um arrasto no corpo pára a lista no sítio. | — | ✔ |

## §3 — O desenho

### 3.1 — A porta: `widget::scroll_area`

Um par **abrir / fechar**, com o nome a dizer que há uma metade por fechar (o molde do `open_body`
do Inspector):

```text
let area = scroll_area::open(scene, hit_index, store, panel, bar_id, body);   // recorta DESENHO e HIT
… o corpo pinta em  area.top() = body.y − area.scroll  …
scroll_area::close(area, scene, hit_index, store, content_h, theme);           // desfaz os recortes,
                                                                               // publica as alturas,
                                                                               // clampa o alvo,
                                                                               // pinta a barra e
                                                                               // regista a TRILHA
```

- **A trilha é o que se regista**, nunca o polegar ⇒ D1 morre por construção, e carregar na
  trilha fora do polegar passa a ter resposta (ver 3.3).
- **As alturas são sempre publicadas pelos DOIS lados** ⇒ D2 deixa de ser exprimível.
- **O id da barra entra na porta junto com o do painel**, e a tabela `scrollbar_panel_for_id`
  passa a ser **derivada** do que as portas publicaram no quadro (`store.scroll_bar_owner`) ⇒ D3
  deixa de ser exprimível: um painel novo não tem onde escrever o id do vizinho.
- **Recorta o `HitIndex` também** — é a metade de D5. ⚠️ Os gates que clicam fora do corpo
  visível passam a reprovar, e isso é a cura a aparecer: cada um se reescreve para **rolar antes
  de clicar** (ou para pintar num viewport onde o alvo está à vista).

### 3.2 — A roda deixa de ter lista

`cursor_over_hero_panel` deixa de ser uma lista de ids: **qualquer rect de painel publicado no
quadro** (`store.panel_rects()`) é chrome, e a roda sobre chrome nunca dá zoom na câmera. O
`panel_at` passa a perguntar **de cima para baixo** pela ordem z (D7). ⇒ D3, D6 e D10 fecham
juntos, e os dois gates de lista (`every_scrollable_panel_intercepts_the_wheel` ·
`the_allowlist_has_no_stale_entries`) trocam por um: *todo painel que publica o rect come a roda*.

### 3.3 — A barra

- Carregar **no polegar** e arrastar: proporcional, como hoje, agora com a trilha certa.
- Carregar **na trilha fora do polegar**: o polegar **salta para debaixo do dedo** e o arrasto
  continua dali (o comportamento de omissão do GTK e do Blender; num tablet é o único gesto que
  faz sentido — «página a página» exige cliques repetidos).

### 3.4 — A inércia

- Só no **arrasto do CORPO** (o gesto de dedo). O polegar da barra não tem inércia em nenhum
  sistema de referência, e a roda do rato já é suavizada pela mola.
- **Velocidade de largada**: janela das últimas amostras de `Move` dentro de **100 ms**; se o
  último `Move` tem mais de **40 ms** quando o dedo sai, a velocidade é **zero** (o dedo parou
  antes de largar). Constantes do `VelocityTracker` do AOSP (Apache-2.0): `HORIZON = 100 ms`,
  `ASSUME_POINTER_STOPPED_TIME = 40 ms`.
- **Limiar para lançar**: `50 px/s`; **tecto**: `8000 px/s` — `ViewConfiguration`
  `MINIMUM_FLING_VELOCITY = 50` e `MAXIMUM_FLING_VELOCITY = 8000` (dp/s, AOSP). ⚠️ São
  constantes de **PRODUTO** (o que um dedo produz), não de recurso: o §0.0 pede que se diga isso.
- **Desaceleração exponencial**: `v(t) = v₀ · 0,998^(t/ms)`, a taxa **normal** publicada do
  `UIScrollView.DecelerationRate` (`0.998` por milissegundo) ⇒ constante de tempo `~0,5 s`, e a
  lista percorre `v₀ · 0,4995 s` antes de parar.
- **Pára** quando a deslocação por quadro fica abaixo de meio pixel, quando bate numa borda, ou
  quando o dedo volta a tocar o painel (pegar no conteúdo em voo SEGURA-o — a lei de todo SO).
- **Durante o arrasto e o voo o conteúdo é 1:1** (D8): o tique escreve o vivo = alvo e a mola
  fica de fora; ela só volta a mandar quando a roda volta a mandar.

## §4 — As waves

| wave | o quê | fecha | estado (2026-09-29) |
|---|---|---|---|
| **W1** | a porta `scroll_area` + a trilha registada + salto na trilha + tabela de dono derivada | D1, D3 (id), D6 (barra) | ✅ |
| **W2** | migrar todos os painéis do caminho do `WidgetStore` para a porta | D1, D2, D4, D5 (parcial) | ✅ **25** painéis (a galeria incluída; o Painter com as duas vistas) + a coluna do catálogo — com o Input Map e a paleta da W5, **28** chamadas à porta |
| **W3** | a roda sem lista, e o `panel_at` pela ordem z | D3 (roda), D6, D7, D10 | ✅ + a tabela `scrollbar_panel_for_id` APAGADA (o dono publicado é a única resposta) |
| **W4** | a inércia + o 1:1 | D8, D12 | ✅ |
| **W5** | as três implementações à parte (Input Map, paleta, timeline) passam a usar as mesmas tabelas e a mesma barra | §1 «quatro implementações» | ✅ Input Map e paleta · ⛔ timeline fora, por DESENHO (§4.1) |
| **W6** | o recorte do `HitIndex` onde falta, com os gates que clicavam no invisível reescritos | D5 | ✅ por construção (a porta recorta) |

### 4.1 — O que a implementação mudou no desenho

- **As formas com `PaintCtx` moram em `panel::scroll_area`**, não em `widget::scroll_area`: o
  `widget` não pode conhecer o `panel` (`panel → action_bus → interaction → widget` já existe, e
  `widget → panel` fechava o ciclo que `the_foundation_modules_form_a_dag` reprova). No `widget`
  fica a porta com as partes soltas (`open_with` · `close_with` · `close_parts` → `Pending`).
- **`panel_at` pergunta primeiro aos rects FORA da ordem z** (o overlay do áudio, o cartão do Input
  Map): eles são pintados depois de todo painel, logo estão por cima por construção. Depois a ordem z
  de cima para baixo.
- **O arrasto no corpo aceita a pressão no FUNDO do próprio painel** (`hit == panel`): uma janela
  flutuante regista o fundo para o clique não vazar ao canvas, e esse registo não é um widget.
- **A roda tem UMA lei** (`WidgetStore::wheel_panel`): o `dispatch_wheel` e a paleta (que toma a
  roda da tela inteira) chamam a mesma função.
- **A timeline fica FORA**: a roda nela é zoom do EIXO DO TEMPO (`set_timeline_canvas`), um gesto de
  canvas e não de lista — pô-la na porta trocaria o gesto.
- **O cabeçalho de um painel nunca come o corpo** (Tokens): o painel desenha-se com os tokens que
  edita, e com `spacing.* ≥ 1024` o corpo começava abaixo da janela — antes da porta o *Reset This
  Mode* ficava registado por baixo de um recorte de altura zero, **clicável às cegas**. O corpo
  guarda sempre uma linha (`ROW_H_PX`).
- **Os censos que contam o que o índice de acerto regista passaram a pintar numa janela que contém o
  painel inteiro** (`16000 px`): com o clique recortado, o que fica abaixo da janela deixa de estar
  no índice. O piso de população de cada censo é quem acusa o dia em que não chegar.

- **D10 e D11 fecharam de passagem.** As entradas mortas do despacho saíram com a tabela à mão
  (`scrollbar_panel_for_id` APAGADA; `cursor_over_hero_panel` é `panel_at(x, y).is_some()`), e a
  coluna de catálogos limpa a região dela pelas **duas** portas de desaparecer (colapsada · painel
  fechado) através de UMA função, `paint_catalog::forget_region` — escrita no `paint.rs` ao lado, a
  limpeza pôs o id num segundo ficheiro e o censo `the_painted_control_reaches_a_consumer` acusou-o
  como controlo sem consumidor. ⏳ **D9 continua aberto** (alturas velhas de um painel que publica
  o rect sem `content_h`).
- **A catraca `widget → interaction` DESCEU de `49` para `46`**: a paleta passou a receber o
  `WidgetStore` (a porta precisa dele) e as referências redundantes dos testes da paleta (o
  `HitIndex` já vinha pelo `use super::*`) saíram — a catraca só encolhe e o número é o medido.

⚠️ O painel da escultura (`ph2d-panel-sculpt3d`) é território da `line/sculpt3d` e muda todos os
dias: a migração dele é **uma linha** (a chamada à porta) e fica anotada no handoff para a
integração.
