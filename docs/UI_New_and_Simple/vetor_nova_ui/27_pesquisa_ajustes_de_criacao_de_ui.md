# 27 — Os ajustes de CRIAÇÃO DE UI do painel do vetor: o que temos, como o Figma arruma, e o botão da coluna ①

> Pedido do dono (16/10): *«os ajustes no inspector do vector estão bagunçados. Veja como figma faz para
> arrumar os ajustes responsáveis por criação da UI. Veja se é possível criar uma tool para isso na barra
> lateral que sirva para ativar/mostrar os ajustes de criação da UI.»* Este doc é a PESQUISA; a decisão do
> dono e o plano entram no §5 quando ele escolher.

## §1 — O que o painel do vetor tem hoje (medido no código, 16/10)

- **33 secções**, ordem em `crates/ph2d-panel-vector/src/paint_body_plan.rs:194-287`. As de criação de UI
  são as **8 últimas** e nascem **recolhidas** (`COLLAPSED_BY_DEFAULT`, `:167-180`): Frame · Clip · Layout ·
  Constraints (`ANCHORS`) · Prefab (`COMPONENT`) · Widget Skin · States · Morph States.
- As 25 de cima são de DESENHO (Transform, Snap, Appearance, Fill, Fill Type, Pattern, Stroke, Stroke
  Pattern, Brush, Connector, Text, Font, Paragraph, Text on Path, Axes, Vertex, Effects, Filters, Expand,
  Blend, Morph, Envelope, Pattern on Path, Contour, Boolean). Quase todas com escopo `Always`; o filtro só
  lê a CONTAGEM da seleção (`covers`, `:79`) — o filtro por ferramenta saiu em D10 (`section_scope.rs`).
- ⇒ a bagunça medida: a ordem é a da HISTÓRIA (cada onda pôs a sua secção no fim), não a do objecto; as
  secções de UI vivem sempre no fundo, fechadas, mesmo quando o objecto seleccionado É um componente; e
  as de desenho avançado (Blend, Morph, Envelope, Contour…) ficam à frente delas.
- O **Inspector** (`ph2d-panel-inspector`) NÃO tem ajustes de UI do vetor: é o dos comportamentos de jogo.
  Não há duplicação entre ele e o painel do vetor (a Hierarquia por confirmar).
- **Nenhum mecanismo** liga hoje um conjunto de secções a um botão da coluna ①. O único botão que muda o
  painel é o do `DrawMode` — que mistura 2 modos + 12 ferramentas (D3, `00_DECISOES_DO_ENIO.md:205-209`).
- Catraca de altura: `a_altura_de_abertura_de_um_painel_so_encolhe`
  (`quantas_entradas_tem_cada_painel.rs:2180`), vetor = `887` px.
- Decisões do dono que valem aqui: **D3** (Layout = o utilizador escolhe, por tarefa · Modo = o TIPO do
  objecto · Ferramenta = o gesto) · **D10** (cinco lugares; a ① flutua com 0 px de layout) · ⛔ «uma faixa
  nova de cabeçalho» recusada (31/08, tablets) · ⛔ painéis flutuantes livres (`03_plano.md` §5).

## §2 — Como os outros arrumam (fontes oficiais; [I] = inferência)

| app | o painel | o mecanismo de modo/aba |
|---|---|---|
| **Figma UI3** (jun/2024) | componente/instância/variantes **em cima** («component controls deserved top billing above attributes like color and size»); **Layout** = tamanho + auto layout numa secção («aligns with how products are built in code»); Position com Constraints (só para filho de frame); Appearance · Fill · Stroke · Effects · Export; secções opcionais com «+» [I: a ordem exacta] | abas **Design / Prototype** (Prototype = interacções, scroll, fluxos); o seletor de modo da barra troca o espaço inteiro (Dev Mode, Draw); **nenhuma ferramenta de desenho muda o painel** (só a Frame mostra os tamanhos-modelo) |
| **Rive** | inspector contextual; Layout vs Node no topo; data binding no próprio campo (fica verde) | **Design / Animate** (Tab), porque no Animate a edição grava chaves; cor de palco própria por modo |
| **Penpot** | Flex/Grid layout como CSS; bloco do filho de layout | abas Design / Prototype / Inspect |
| **Adobe XD** | um inspector, com Repeat Grid e Fixed Position | abas Design / Prototype / Share |
| **Framer** | menos propriedades por breakpoint; o campo SOBRESCRITO destacado | — |

Fontes: [figma.com/blog/behind-our-redesign-ui3](https://www.figma.com/blog/behind-our-redesign-ui3/) ·
[our-approach-to-designing-ui3](https://www.figma.com/blog/our-approach-to-designing-ui3/) (Constraints
abertas por omissão; «Clip content» voltou a caixa — o clique a mais atrasava; painéis flutuantes
revertidos) · [help: properties panel](https://help.figma.com/hc/en-us/articles/360039832014) ·
[help: toolbar](https://help.figma.com/hc/en-us/articles/360041064174) ·
[help: constraints](https://help.figma.com/hc/en-us/articles/360039957734) ·
[help: auto layout](https://help.figma.com/hc/en-us/articles/360040451373) ·
[rive: design vs animate](https://rive.app/docs/editor/fundamentals/design-vs-animate-mode) ·
[rive: layouts](https://rive.app/docs/editor/layouts/layouts-overview) ·
[penpot: interface](https://help.penpot.app/user-guide/the-interface/) ·
[XD: workspace](https://helpx.adobe.com/xd/help/workspace-basics.html) ·
[NN/g: modes](https://www.nngroup.com/articles/modes/) (modo esconde o que vive só nele; ≥ 2 indicadores
visuais do modo).

⚠️ **O preço que o Figma pagou** (fórum, out/2024 e abr/2025, sem resposta deles): com o bloco do
componente em cima, Position e Layout mudam de altura conforme o tipo seleccionado e um componente com
muitas propriedades empurra-as para fora do ecrã ⇒ a ORDEM tem de ser a mesma para todo tipo; esconder o
que não se aplica, nunca reordenar; o bloco do componente recolhível.

## §3 — Princípios a adoptar

1. A ORDEM das secções é uma só, para todo objecto; o que não se aplica some, nunca muda de lugar.
2. O que aparece depende do OBJECTO e do contexto (Constraints só dentro de um Frame; Layout só num
   contentor ou filho de layout; o bloco do componente só num componente/instância).
3. Opcional fica fechado até ser usado; desenho avançado num grupo próprio, recolhido.
4. Nomes e grupos como o código pensa: Componente · Posição · Layout · Aparência · Preenchimento · Traço ·
   Texto · Efeitos.
5. Token e sobrescrita vivem NO CAMPO (vínculo colorido; a sobrescrita destacada com «repor»).
6. COMPORTAMENTO (Estados, Morph States, interacções) é outra tarefa sobre a MESMA seleção ⇒ aba
   própria, como o Prototype do Figma.
7. Um modo de verdade só onde a edição se comporta diferente (o Animate do Rive grava chaves); com dois
   indicadores (cor do palco + o interruptor aceso).

## §4 — O botão da coluna ① (a pergunta do dono)

- **É possível**: a coluna é uma tabela (`crates/ph2d-panel-vector/src/bars.rs`, `TOOLS` + `column()`), e
  um botão novo é uma linha + id + ícone + chave i18n; os gates que o censam são
  `every_painted_column_and_action_button_answers_a_real_pointer` e os do `the_vector_control_bar.rs`.
- **Como FERRAMENTA, não** (nenhum dos apps faz; D3: ferramenta é o gesto). Como **interruptor de vista**
  sim — é o que o seletor de modo do Figma faz na barra dele (Dev Mode, Draw): fica no FIM da coluna,
  separado das ferramentas, não muda o que o clique faz, só o que o painel mostra (a aba «UI»), com o
  palco tingido como segundo indicador.
- Pela D3 a «vista de criação de UI» é um **Layout por tarefa** (o utilizador escolhe), e o botão da ① é
  o atalho para ele — não um `DrawMode` novo (que repetiria a mistura que a D3 mediu).

## §5 — Decisão do dono (16/10)

1. **Abas + botão.** O painel do vetor ganha duas abas NO PRÓPRIO TÍTULO (nada de faixa nova — a recusa de
   31/08): **«Desenho»** e **«Interface»**. Um **interruptor no fim da coluna ①**, separado das ferramentas,
   troca para «Interface» e **tinge levemente o palco** (segundo indicador, NN/g). Não muda o que o clique faz.
2. **Arrumar a ordem como o Figma**, uma só para todo objecto: Componente → Posição (+ Constraints) → Layout →
   Aparência → Preenchimento → Traço → Texto → Efeitos → **Avançado** (recolhido: Blend, Morph, Envelope,
   Pattern on Path, Contour, Boolean, Expand…). O que não se aplica some; nada muda de lugar.

## §6 — Esboço do plano (para a janela que implementa — `/pd-feature`, plano primeiro)

- **Quem vai para cada aba** (proposta, a fechar no plano): «Interface» = Componente (Prefab), Frame, Clip,
  Layout, Constraints, Widget Skin, States, Morph States; «Desenho» = o resto. A Posição/Transform aparece nas
  duas (é do objecto). Medir antes: abrir cada secção de UI e confirmar que nenhuma tem knob morto (§5.0
  «controlo morto»).
- **O estado da aba é do HERÓI/VISTA** (D3: Layout por tarefa, o utilizador escolhe), não um `DrawMode` novo;
  o botão da ① e o clique na aba escrevem o MESMO campo; o tinto do palco lê-o.
- **Contextual sem botão**: seleccionar um componente/instância com a aba «Desenho» aberta mostra um
  aviso-chip «Interface ▸» no título (descoberta, NN/g: o que vive só num modo esconde-se).
- **Portas/gates a respeitar**: `paint_body_plan.rs` (a ordem é uma tabela — reordenar é a mesma lista),
  `COLLAPSED_BY_DEFAULT`, `a_altura_de_abertura_de_um_painel_so_encolhe` (`887` deve DESCER: as secções da
  outra aba saem da conta), `every_painted_column_and_action_button_answers_a_real_pointer` (o interruptor),
  censos de i18n/tooltip, `architecture_panel_wiring_parity`. Zero hex / literal de UI (HR-15).
- **Smoke**: cena nova `PH2D_OBJECT_MODE_SMOKE=25` com um componente de UI (botão com estados) e uma forma
  comum, fotografada nas duas abas.
