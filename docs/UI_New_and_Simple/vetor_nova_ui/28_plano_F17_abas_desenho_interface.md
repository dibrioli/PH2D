# 28 — Plano F17: as abas «Drawing / Interface» do painel do vetor, o interruptor da ①, a ordem do Figma

> Implementa a decisão do dono de 16/10 ([`27`](27_pesquisa_ajustes_de_criacao_de_ui.md) §5). Pesquisa do
> estado da arte: o `27` §2–§3 (Figma UI3 Design/Prototype, Rive Design/Animate, Penpot, XD; o preço que o
> Figma pagou com o bloco do componente a mudar de altura). Aqui só o desenho, as portas, os gates e a cena.

## §1 — O que foi medido antes de desenhar (16/10, no código)

| facto | onde |
|---|---|
| O título é `paint_panel_title` (só texto, `Lg`); não há porta para widgets no título | `ph2d-editor-core/src/widget/panel_chrome.rs:120`, chamado em `ph2d-panel-vector/src/paint.rs:340` |
| Existe o widget de abas (`Tabs`, `Role::TabList`/`Tab`, grupo composto) — usado só na Timeline, pintar e registar pela MESMA lista | `widget/tabs.rs:49`, `ph2d-panel-timeline/src/transport_tabs.rs:100` |
| Largura mínima da doca `210` px ⇒ sobra `210 − 2·18 − 42 = 132` px entre o X e a borda: «Vector» (`Lg`) + duas abas não cabem | `dock_width_ops.rs:189`, `panel_chrome.rs:98` |
| A ordem que o artista ARRASTOU grava-se em `~/.ph2d/sections.txt` e manda sobre a natural; **a do dono tem `fill, stroke, snap, blend` do vetor** (descodificado 16/10) ⇒ sem migração, a ordem nova não lhe chega | `section_prefs_ops.rs:152` (`ordena_seccoes`), `:208` |
| O corpo é uma lista plana, sem grupos; «Avançado» é só um comentário | `paint_body_plan.rs:262` |
| As 8 secções de UI pintam só com o `Option` que a shell publica (devolvem `y` intacto sem ele); a Component aparece para QUALQUER caminho (oferece *Create*) | `paint_components.rs:24`, `fase_vector_selection_frame_panel.rs:119` |
| A abertura do vetor (`887` px) mede-se SEM seleção e com as dobras de omissão; o gate reprova nos dois sentidos | `quantas_entradas_tem_cada_painel.rs:2156`, `:2180` |
| Os cliques da ① chegam ao painel pelo barramento (`ToolPanelEvent`) e ao `apply_event` do painel | `the_vector_bars_answer_a_real_pointer.rs:77`, `event.rs:177` |
| Não há tinto de modo em família nenhuma; o palco no produto é o `game_rt`; tokens translúcidos derivam-se em `derive.rs` (`graph-marquee` = accent a `0,18`) | `screens/hero/canvas.rs:31`, `ph2d-tokens/src/derive.rs:542` |

## §2 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | a porta única | quem escreve · quem lê |
|---|---|---|
| **Que aba está aberta?** | `state::aba() -> Aba` (`Drawing` / `Interface`), thread-local da crate do painel, sessão (nasce `Drawing`) | escreve SÓ `state::set_aba`, chamada pelo `apply_event` do painel para os TRÊS ids (as duas abas e o interruptor da ①) · lêem: o plano do corpo, a face da aba, o `on` do interruptor, o tinto |
| **De que aba é cada secção?** | o 3.º argumento de `Seccoes::seccao(id, scope, aba, …)` — `Onde::Drawing` · `Onde::Interface` · `Onde::Ambas` (Transform, Snap) | declarado na própria linha do corpo (o compilador cobra) |
| **Em que ordem?** | a lista `corpo()` — UMA ordem para todo objecto (princípio 1 do `27` §3); cada aba é um filtro dela | §3 |
| **O que é «Avançado»?** | um GRUPO: `Seccoes::grupo(VECTOR_SECTION_ADVANCED, […membros])` — cabeçalho dobrável (recolhido por omissão); dobrado ⇒ os membros não pintam; aberto ⇒ pintam LOGO a seguir ao cabeçalho, pela ordem do artista entre eles (um membro nunca foge do grupo por um arrasto) | o laço `corre` |
| **O palco tinge?** | `ToolBars::stage_tint: Option<ColorToken>` — o módulo publica, o herói pinta (por cima da arte, por baixo das réguas e gizmos). Token NOVO `StageTintUi` (`stage-tint-ui`) em `derive.rs` | vetor publica `Some(StageTintUi)` em Interface · `paint_canvas_overlays` lê |
| **A ordem gravada do artista e a ordem nova** | época por painel no `sections.txt`: `epoch.<painel>=N`; `WidgetStore::adopt_natural_order(painel, N, ids)` esquece UMA vez as posições gravadas desses ids quando a época gravada é menor | o `populate` do vetor chama-a com `VECTOR_ORDER_EPOCH = 1` |
| **A seleção tem coisa de interface?** (descoberta, NN/g) | `state::seleccao_tem_interface()` = OU dos `Option` já publicados (componente mestre/instância, pele, estados, morph states, moldura, item de layout) | marca a aba «Interface» com um ponto quando a aba aberta é a outra |

Recusas deste desenho (e porquê):
- ⛔ `DrawMode` novo — repetiria a mistura que a D3 mediu (`27` §4).
- ⛔ o estado no `hero.view` com espelho para o painel — o único leitor que DECIDE e os três escritores são da
  crate do painel; um espelho é a 2.ª porta que envelhece. O herói lê a projecção do quadro (`ToolBars`), que
  só existe com o vetor na mão — e é exactamente quando o tinto vale.
- ⛔ o chip «Interface ▸» SEPARADO no título (`27` §6) — com as abas no título seria um 2.º botão para o
  mesmo gesto (duas portas); o ponto na própria aba diz o mesmo sem custar largura.
- ⛔ «Vector» + abas na mesma linha — não cabe nos `210` px (§1). As abas SÃO o título, como no Figma; o
  nome «Vector» fica no `TabList` (a11y) e na aba da doca.

## §3 — A ordem (uma só; `I` = Interface, `D` = Drawing, `A` = ambas)

1. Component `I` · 2. Transform `A` · 3. Anchors (Constraints) `I` · 4. Snap `A` (recolhido) · 5. Frame `I` ·
6. Clip `I` · 7. Layout `I` · 8. Appearance `D` · 9. Fill · Fill Type · Pattern `D` · 10. Stroke · Stroke
Pattern · Brush · Connector `D` · 11. Text · Font · Paragraph · Text on Path `D` · 12. Axes · Vertex `D` ·
13. Effects · Filters `D` · 14. **▸ Advanced** `D` (grupo recolhido): Expand · Blend · Morph · Envelope ·
Pattern on Path · Contour · Boolean · 15. Widget Skin · States (recolhido) · Morph States (recolhido) `I`.

Na aba Interface as de UI deixam de nascer recolhidas (o artista foi lá para elas; Figma: «Constraints
abertas por omissão»), salvo States e Morph States (as maiores: 4 linhas `Rec` e 11 curvas).

## §4 — Contrato congelado / formato

- Nenhum. `ToolBars`, `WidgetStore`, `ColorToken` não são superfície de `architecture_tool_contract_surface`
  (grep 16/10: zero ocorrências de `ToolBars|section|ColorToken|interaction` nesse gate); `PanelEvent` não muda.
- Formato: `PROJECT_SCHEMA` intocado (a aba é sessão). O `sections.txt` ganha a linha `epoch.<id>=N` — o
  leitor antigo IGNORA chaves que não conhece (a confirmar no parser, com teste de ida-e-volta).

## §5 — A UI: as quatro condições, por controlo

| controlo | existe | pintado + registado | o clique chega | a SEQUÊNCIA leva a algum lugar |
|---|---|---|---|---|
| abas «Drawing»/«Interface» | `Tabs` (Ghost) no rect do título | `paint_tabs_with_hover` + `hit_index.register` pela mesma lista; `store.register` no `populate` (paridade) | `apply_event` → `set_aba` | o corpo troca de secções; a altura e a rolagem recomeçam |
| interruptor da ① | grupo próprio NO FIM da coluna (depois da cor), `BarButton.on = aba()==Interface`, ícone `Prefab` | `column()` | barramento → `apply_event` → `set_aba(toggle)` | o painel troca de aba + o palco tinge; clicar de novo volta |
| grupo «Advanced» | `section_header` com id novo | populate marca-o dobrável e `COLLAPSED_BY_DEFAULT` | dispatch genérico da dobra | os sete membros aparecem/somem juntos |
| ponto na aba Interface | rótulo da aba + marca | — | (o clique é o da aba) | leva à aba onde a seleção se edita |

## §6 — Gates, vermelhos primeiro (cada um visto VERMELHO antes da cura)

1. `a_ordem_natural_e_a_do_figma` (`paint_body_plan`) — a lista e a aba de cada secção, por extenso.
2. `cada_aba_pinta_so_as_suas_seccoes` (seam, `MockPanelHost`, clique REAL na aba): com um componente
   publicado, em Drawing a Component não pinta e o Fill pinta; depois do clique em «Interface», o inverso.
3. `as_duas_portas_escrevem_a_mesma_aba` — clique na aba e clique no interruptor da ① (pelo ponteiro, como o
   `every_painted_column…`) movem o MESMO `aba()`; o `on` do interruptor e o `stage_tint` seguem-no.
4. `o_grupo_avancado_dobra_os_sete_juntos` — fixture: ordem gravada que separa Blend de Contour; recolhido ⇒
   nenhum dos sete pinta; aberto ⇒ os sete logo a seguir ao cabeçalho.
5. `a_ordem_gravada_antes_da_epoca_e_esquecida_uma_vez` — fixture = a linha `order=` REAL do dono (16/10,
   com `vector.section.fill/stroke/snap/blend`): depois do populate a ordem pintada é a do §3; um arrasto
   depois disso SOBREVIVE a um 2.º populate; ida-e-volta do `sections.txt`.
6. `o_palco_tinge_so_na_aba_interface` (herói) — a lavagem cobre a área de desenho e nada fora dela.
7. A catraca `a_altura_de_abertura_de_um_painel_so_encolhe`: o `887` DESCE (Blend, Morph, Pattern on Path e
   Contour saem para o grupo recolhido) — o número medido entra no mesmo commit.
8. Os censos que já mordem: i18n do painel, `architecture_panel_wiring_parity`, `every_painted_column_and_
   action_button_answers_a_real_pointer`, `a_lista_do_corpo_e_toda_seccao_do_painel`, rótulo cortado a `210` px.

## §7 — Smoke

- `PH2D_OBJECT_MODE_SMOKE=25`: um BOTÃO de interface (rectângulo arredondado feito componente pelo *Create* do
  painel, com a pele *Button*) e uma estrela comum ao lado; o botão seleccionado; aba **Drawing** aberta (o
  ponto acende na aba Interface).
- `=26`: a 25, e depois o clique no interruptor da ① ⇒ aba **Interface**, palco tingido.
- Duas cenas porque o roteiro de foto fotografa o estado FINAL; cada uma passa pelo clique real.
- Números a medir na sonda antes da mensagem ao dono: altura de abertura nas duas abas, secções pintadas em
  cada uma, e o `887` novo.
