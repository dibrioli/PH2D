# 03 — O plano: a UI do Vector em cinco lugares

> 2026-10-07 · **PLANO DECIDIDO** — as cinco perguntas do §6 foram respondidas pelo Enio no mesmo dia
> (registo: [`../00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md) D10). Descende de
> [`01_o_estado_atual.md`](01_o_estado_atual.md) (o que temos, medido) e de
> [`02_estudo_os_apps_amados.md`](02_estudo_os_apps_amados.md) (o que os outros fazem, com fontes).
> **F0–F3 entregues em 07/10** (`line/UIUX`) — o que foi medido e decidido na obra está no
> [handoff](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md) §1–§2.
> **F4 e F5 entregues em 08/10** — ⛔ **a roda (F4.1) foi RETIRADA pelo dono no smoke** (D10.2) — [handoff de 08/10](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F4_F5_ALCANCE_E_TOQUE.md);
> **F6** (navegar com os dedos) e **F7** (o QuickShape, com o oráculo corrido) também em 08/10 — §4.

## §0 — A ideia numa frase

**Cada controlo vai para o lugar cuja pergunta ele responde.** As ferramentas vão à **esquerda**,
as opções da ferramenta vão **em cima**, o que a seleção é fica **à direita**, as ações sobre a
seleção vão **perto dela** e o que as teclas fazem vai **no rodapé**. É o desenho do Affinity, com
as lições dos apps de iPad.

⭐ **E sem pagar faixa nova** — a recusa de 31/08 (*«não podemos ir perdendo espaço»*) vale. Quatro
dos cinco lugares **já existem e já são pagos**:

| lugar | custa | porquê |
|---|---|---|
| ① barra de ferramentas (esq.) | **0 px de layout** | flutua dentro da área, como a barra curta do Quadro (precedente vivo, decisão do dono 4 do MiroClone) |
| ② barra contextual (cima) | **0 px novos** | é a **fila de ferramentas** que já existe (`tool_bar.rs`) — muda o CONTEÚDO, não a altura |
| ③ propriedades (dir.) | **0 px novos** | é o painel Vector de hoje, emagrecido |
| ④ ações da seleção | **0 px de layout** | flutua junto da seleção, como a barra de estilo do Quadro; fixável e desligável |
| ⑤ rodapé | **0 px novos** | é a pílula que já existe (`bottom_hud.rs`) |

## §1 — O desenho alvo

### Desktop (`1930 × 1040`, layout *Vetor*, forma em Edit, caneta na mão)

> Os rótulos da ② (`Pen·Smart·Polígono·Linha`) são os do **Affinity**, como ilustração do lugar —
> a caneta do PH2D ainda não tem esses sub-modos (§2).

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ File Edit View Window Run │ Scene +                          Draw [Vector] Flip Animate… │ menu (igual)
├────────────┬─────────────────────────────────────────────────────────────┬───────────────┤
│ Hierarchy  │ ② Edit Mode▾ │ Pen·Smart·Polígono·Linha │ ■Fill ▢Stroke 2px ⚙ │ ⋯ │ 🧲▾ ⊞ │ Propriedades  │
│            │ ─────────────── régua ─────────────────────────────────────  │ ③             │
│ Path 0     │┌──┐                                                         │ ┌ Alinhar ─┐  │
│ Path 1     ││▲ │ ①                                                       │ Transform     │
│ …          ││✒ │                     ╭──────╮                            │ Aparência     │
│            ││✎ │                     │ arte │                            │ Fill          │
│            ││▢◢│                     ╰──────╯                            │ Stroke        │
│            ││T │              ┌ ④ ∪ ∩ − ⊕ │ ⧉ 🗑 │ ⇄ ┐                  │ (tipo: Shape/ │
│            ││✂ │              └─────────────────────┘                    │  Text/Conect.)│
│            ││⌒ │                                                         │ Efeitos       │
│            ││… │                                                         │ ▸ Contêiner   │
│            ││■▢│ ← Fill/Stroke                                           │ ▸ Interação   │
│            │└──┘                                                         │ ▸ Avançado    │
│            │   ⑤ Caneta · clique = canto · arrastar = curva · Alt = alça solta · Shift = 45°  │
│            │      x 312  y −48 · encaixe: grelha, nós · 100 %                                │
└────────────┴─────────────────────────────────────────────────────────────┴───────────────┘
```

### iPad 11" (`1194 × 834`)

```
┌──────────────────────────────────────────────────────────────┐
│ ☰ │ Scene +                               Vector ▾ │ ⋯  (iPadOS) │  ⛔ nada no centro do topo
├──┬──────────────────────────────────────────────────────┬──┤
│▲ │ ② Edit▾ │ Pen·Smart │ ■ ▢ 2px ⚙ │ ⋯ │ 🧲             │▤ │  ③ recolhida = faixa de ícones
│✒ │                                                       │◐ │     (Studio do Affinity iPad)
│✎ │                    ╭──────╮                            │⊞ │
│▢ │                    │ arte │                            │  │
│T │             ┌ ④ ∪ ∩ − │ ⧉ 🗑 ┐                         │  │
│… │                                                       │  │
│■▢│ [⌘ modificadores]                                     │  │  ← segurar / travar (Affinity, Adobe)
├──┴──────────────────────────────────────────────────────┴──┤
│ ⑤ Caneta · toque = canto · arrastar = curva     (fora do centro: indicador de Home)  │
└──────────────────────────────────────────────────────────────┘
```

## §2 — A tabela de destino (as 41 secções e os 16 modos)

A coluna **escopo** é o que o [`section_scope.rs`](../../../crates/ph2d-panel-vector/src/section_scope.rs)
já diz hoje. ⇒ **a regra de destino é derivável, não escolhida a dedo**: GESTO → ②, SELEÇÃO → ③
(o frequente também em ②/④), DOCUMENTO → ② à direita.

| hoje no painel | escopo | vai para | nota |
|---|---|---|---|
| **Tool** (16 botões de texto) | ALWAYS | **①** barra de ferramentas | os MESMOS ids `VECTOR_MODE_*`, agora com ícone; ⛔ a secção sai do painel no mesmo trabalho (um comando, uma porta) |
| Tool → fileiras por modo (marquee do Node, botões do Cut) | gesto | **②** | já só aparecem no modo delas |
| **Shape** (catálogo) | CATALOG | **①** flyout do botão Forma + **②** parâmetros | o Affinity faz assim (flyout + Sub Tools) |
| **Pencil** | PENCIL | **②** | Fidelity e Stabilizer são opções do gesto |
| **Symmetry** | gesto | **②** | é uma opção das ferramentas de desenho |
| **Snap** | documento | **②** à direita (ímã + popover) | o Affinity põe o snapping na toolbar; o Inkscape 1.2 em popover Simples/Avançado |
| **Stroke**, **Fill** (a cor e a largura) | seleção | **②** amostras + **③** completo | ⭐ o padrão nº 1 do Affinity: amostra que abre a cor ali mesmo |
| Fill Type, Texture Pattern ×2, Brush | seleção | **③** sob Fill/Stroke | |
| **Transform**, **Appearance** | seleção | **③** no topo | |
| Shape Params, Connector, Text, Font, Paragraph, Text on Path, Axes, Vertex | seleção | **③** «específico do tipo» | só a do tipo seleccionado aparece (já é assim) |
| **Boolean**, **Arrange**, **Path**, **Align** | WHEN_SELECTED | **④** os verbos + **③** a linha de Alinhar no topo | Rive/Penpot/Sketch: alinhar no topo do painel, só com seleção |
| Expand, Effects, Filters, Envelope, Pattern on Path, Contour, Blend | seleção | **③** «Efeitos» | |
| Frame, Clip, Layout, Anchors | seleção | **③** «▸ Contêiner», recolhido | hoje abrem ANTES do Stroke (foto `antes_01`) |
| Component, Widget, States, Morph, Morph States | seleção | **③** «▸ Interação», recolhido | idem — States, com 4 linhas `Rec` e 11 curvas, ocupa o meio do painel |

⚠️ **«Recolhido» é recolhido por omissão, não escondido.** O artista abre-o e a dobra é lembrada
(já é assim: `ordena_seccoes` + a dobra por secção).

### O que muda na fila de cima (②) conforme o modo

| modo / ferramenta | à esquerda (o gesto) | ao centro (a seleção) | à direita (fixo) |
|---|---|---|---|
| **Object** (forma seleccionada) | `MOVE ROT SCALE PIVOT · SPACE` (os chips de hoje, que são do gizmo de objecto) | ■Fill ▢Stroke largura ⚙ | ⋯ · 🧲 encaixe · vista |
| **Edit + Select/Node** | os ids que **já existem**, só realojados: nó `VECTOR_VERT_CORNER/SMOOTH/SYMMETRIC` · caminho `VECTOR_PATH_JOIN/CLOSE/REVERSE/WELD` · `⚙` com `SMOOTH/SHARPEN/SIMPLIFY/SUBDIVIDE` (o *Smart* do Affinity não temos) | ■ ▢ largura ⚙ | idem |
| **Edit + Pen** | ⚠️ hoje a caneta **não tem** opções no painel (o *Spiro/Hyperbezier* do doc 12 nunca foi construído — `rg -i spiro` devolve zero). O Affinity põe aqui Pen · Smart · Polygon · Line; a F0 decide se algum entra | ■ ▢ largura | idem |
| **Edit + Shape** | parâmetros da forma (lados, raio interno…) | ■ ▢ largura | idem |
| **Edit + Pencil** | Fidelity · Stabilizer · Simetria | ■ ▢ largura | idem |
| **Edit + Cut/Trim/Fillet/…** | as opções desse gesto | — | idem |

⛔ **Hoje, em Edit, a fila mostra `MOVE/ROT/SCALE`** (foto `antes_01`): o espaço mais caro da tela
fala de outro modo. ⚠️ Mas **não apagar sem medir**: a 01/09 esses chips foram LIGADOS ao gizmo
(D2), e em Edit a forma ainda mostra gizmo. ⇒ a fase F2 mede quem os lê em Edit antes de os tirar.

## §3 — As peças, e onde vivem (sem tabela paralela)

| peça | de onde nasce | onde vive | a lei que a protege |
|---|---|---|---|
| **barra curta** (①) | ⭐ **GENERALIZAR** a do Quadro (`board_bar.rs`: `toolbar_rects`, `paint`, ids derivados de UMA tabela) + o flyout do trilho do Painter (`left_rail.rs`) | widget genérico em `ph2d-editor-core`; a TABELA do vector em `ph2d-panel-vector` | ⛔ **não copiar o `board_bar`** — *«copiar UM componente deixa para trás os IRMÃOS»* e *«"é a mesma conta que X" ao lado de uma cópia é o cheiro da lei duplicada»* (memória). Um widget, duas tabelas |
| **ícones** das 16 ferramentas | `docs/design/icons/*.svg` (Lucide, ISC; codegen em `icons.rs`) — já há `Select`, `VectorPen`, `VectorPencil`, `VectorShape`, `Text`, `Knife`, `Polygon` | idem | faltam ~8 (Node, Connect, Fillet, Chamfer, Width, Trim, Bucket, Frame); **forma distinta + cor por família** (a armadilha do Affinity V3) |
| **barra contextual** (②) | a **fila** que já existe (`tool_bar.rs`) — ganha um *produtor* por modo | o conteúdo vem da tabela `section_scope` do vector | ⛔ a fila tem **um** dono por quadro (como o `MOVE` do 3D, D2: *«a mesma bandeira decide quem é dono dos chips do trilho»*) |
| **amostras Fill/Stroke** | o picker OKLCH partilhado (`register_picker_swatch`) | — | os mesmos ids da secção Stroke/Fill: *um comando com dois ids tem dois sítios a apodrecer* |
| **ações da seleção** (④) | generalizar a **barra de estilo** do Quadro (`style_rects`) | widget genérico + tabela do vector | fixar · desligar · nunca sobre a seleção (posição âncora) |
| **rodapé** (⑤) | a pílula (`bottom_hud.rs`) | a dica vem de **cada `DrawMode`** (uma linha por modo, em `ph2d-i18n`) | censo: todo `DrawMode` tem dica (senão o rodapé fica mudo nessa ferramenta) |
| **teclas** | `keyboard_cadeia.rs` (o bloco «atalhos do vetor») | — | censo: a tabela do estudo §5 (padrão Illustrator) é a ÚNICA fonte; toda ferramenta que lá tem tecla tem-na no app, nenhuma tecla tem dois donos, e nenhuma tecla é inventada fora do idioma |
| **radial** | `radial.rs` (já projecta «as ferramentas» do trilho) | — | passa a projectar a ① do vector — e o oitavo sector é o `Ctrl+K` (já é a lei) |

⛔ **A shell só encolhe:** nada disto entra em `shells/desktop/src`. O dispatch dos ids novos segue a
família (`crates/ph2d-app-vec`), como o HOWTO de partir famílias manda.

## §4 — As fases

> `CLAUDE.md` §0.10: **problemas equivalentes num bloco só.** F1–F3 são a MESMA obra — realojar
> secções segundo uma tabela, incluindo a ④ (que é o destino de `Boolean/Arrange/Path/Align`) — e
> fazem-se numa linha, com UMA rodada de fotos e UM gate batched. F4 e F5 dependem do que o Enio
> achar no smoke de F1–F3, e por isso vêm depois.

### F0 — Decisões e medição (antes de qualquer código)

- O Enio responde ao §6.
- **Medir o orçamento** nos três iPads com ① e ④ abertas: quanto da área a barra curta tapa, e se
  ② cabe numa linha com o conteúdo mais longo (Edit + Node: 7 verbos + ■ ▢ + largura) — mesma régua
  de [`medicoes/06`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md) e do orçamento de chips (`3`).
  Se não couber, o que transborda vai ao `⋯` **por prioridade declarada**, nunca truncado.

### F1 + F2 + F3 — O realojamento (uma linha)

1. **①** a barra curta genérica, a tabela do vector (16 modos em famílias, com flyout para as
   formas), os ícones em falta, as amostras Fill/Stroke em baixo. A secção `Tool` **sai** do painel.
2. **Teclas do Illustrator** (a tabela do §5 do estudo: `V A P N M L T K W C` · `Shift+M/W/O` ·
   `X`/`Shift+X`/`D` da cor), mais a ferramenta **temporária** (segurar e soltar volta à anterior). As
   booleanas mudam de `U/I/D/X` para `Ctrl+Alt+U/I/D/X`.
3. **②** a fila ganha o produtor do vector: as opções do gesto por modo, as amostras Fill/Stroke e a
   largura, o ímã de encaixe com popover, e o `⚙ Settings` para o avançado. `Pencil`, `Symmetry`,
   `Snap` e o catálogo `Shape` **saem** do painel.
4. **③** o painel reordenado: Alinhar no topo (com seleção), Transform, Aparência, Fill, Stroke, o
   específico do tipo, Efeitos, e **Contêiner / Interação / Avançado recolhidos por omissão**.
5. **④** a barra de ações perto da seleção (generalizando a barra de estilo do Quadro), **ligada por
   omissão no desktop e no iPad** (decisão 4), com **fixar** e **desligar** e a posição âncora gravada
   no layout. Recebe os verbos de `Boolean`, `Arrange`, `Path` e `Align`.
6. **⑤** o rodapé, no layout *Vetor*, mostra a dica da ferramenta, as coordenadas, o encaixe e o
   zoom. As estatísticas de motor ficam nos outros layouts, ou num clique.

**Gates (batched, no fecho):**
- cada `DrawMode` tem **exactamente um** botão em ① e uma dica em ⑤, e a tecla que o padrão Illustrator lhe dá (§5 do estudo);
- nenhuma secção é pintada em dois lugares (*uma porta*);
- todo botão pintado em ① e ② é despachado (a família `every_painted_rail_button_is_dispatched`);
- o orçamento dos tablets (F0) vira teste;
- o censo de i18n e HR-15 continua verde;
- `the_shell_only_shrinks` continua verde.

**Smoke:** uma cena `PH2D_OBJECT_MODE_SMOKE=<próximo>` com **formas e uma simulação de uso** —
caneta na mão, uma forma em Edit, a fila mostrando as opções da caneta —, **fotografada** antes de
ir ao Enio.

### F4 — O radial e a paleta que ensina

- O radial projectando a ① do vector.
- A paleta `Ctrl+K` passando a **mostrar onde o comando mora**, fazendo piscar o botão (o *Feature
  Finder* do Affinity): ensina a UI nova a quem conhecia a antiga.

### F5 — iPad: o toque sem teclado

- **Modificadores na tela**: segurar e travar, sempre visíveis quando travados. Primeiro o
  dedo-extra, sem chrome nenhum; o widget vem a seguir.
- **Desenhar e segurar = forma perfeita**, com nós editáveis (QuickShape). Este é um gesto com LEI
  (reconhecimento de forma) ⇒ precisa de oráculo CORRIDO (`CLAUDE.md` §0.9), e não de leitura.
- 2/3/4 dedos (desfazer/refazer/foco), e tocar e segurar = menu de contexto.
- ③ recolhida em faixa de ícones no iPad 11" e no mini.
- Barras laterais espelháveis para **canhotos** (com esse nome).
- ⛔ Todo gesto com uma porta visível ao lado.

### F6 — Navegar com os dedos (entregue 08/10)

- **2 dedos que ANDAM = arrastar a vista; pinça = zoom à volta do ponto entre os dedos** (R4 #11,
  universal). Pelas portas do rato — o botão do MEIO e a RODA — nunca uma 2.ª câmera; o zoom à
  volta de um ponto é um pan pela porta do pan (`Camera2d::anchor_pan_px`).
- A fronteira com o toque de 2 dedos (desfazer) é a FOLGA do UIKit: um toque que andou não desfaz,
  um pan que não andou não move. A navegação publica-se **uma vez por quadro**, com todos os dedos
  já no sítio (publicar por evento compõe zooms à volta de dois pontos — medido na foto).
- ⛔ Sem rotação da vista (a câmera 2D não tem rotação — medido) e sem «quick-pinch encaixa no ecrã»
  (não existe *enquadrar tudo* no app — pergunta ao dono, não invenção).
- Smoke: cena `PH2D_OBJECT_MODE_SMOKE=11` (contactos sintéticos sobre as formas da cena 9),
  foto [`fotos/f6_01`](fotos/f6_01_cena11_dois_dedos_arrastam_e_aproximam.png).

### F7 — QuickShape (entregue 08/10)

- **Desenhar e SEGURAR 0,5 s = a forma perfeita, com nós de quina editáveis** (o gesto do Procreate):
  linha, polilinha aberta, triângulo e quadrilátero como desenhados, rectângulo (quinas a 90 ± 8°,
  endireitado aos eixos até 5°), polígono de N lados, elipse, círculo. Sem soltar, arrastar ESCALA e
  RODA à volta do início (linha) ou do centro; `Shift` (= o 2.º dedo) regulariza e roda a 15°.
  Soltar comete a forma (um passo de undo); `Esc` devolve o traço livre.
- A lei (`ph2d-vec-edit::quickshape`) bate as **65** medições do Xournal++ ou, onde o Procreate manda
  mais, a geometria que gerou o traço — [`oraculo_quickshape/00_resultados.md`](oraculo_quickshape/00_resultados.md) §3b–§3c.
- Porta visível: **QuickShape** na fila do Lápis (ligado por omissão) e a dica ⑤ «Hold at the end:
  perfect shape». Smoke: cena `=12`, foto [`fotos/f7_01`](fotos/f7_01_cena12_tres_tracos_seguros_viram_formas.png).
- **F7b (09/10, depois do smoke do dono):** formas PEQUENAS (piso 94 → 12 px, medido sob tremor), o
  «parado» com a folga do toque (10 px) e a elipse no meio da faixa medida — a «pouca sensibilidade»;
  um S deixa de passar por zigue-zague. Medições em [`00_resultados.md`](oraculo_quickshape/00_resultados.md) §3d;
  foto [`fotos/f7b_01`](fotos/f7b_01_cena12_quatro_tracos_com_um_circulo_pequeno.png).

## §5 — O que NÃO entra (e porquê)

| recusado | porquê |
|---|---|
| uma faixa nova de cabeçalho | recusada pelo Enio a 31/08 (tablets) |
| painéis flutuantes livres | D1 (ancorados, flutuação declarada); a Figma **reverteu** os dela |
| a barra contextual **colada** à seleção, sempre visível | a armadilha nº 1 (Illustrator CTB); a ④ só existe fixável e desligável |
| customizar a barra de ferramentas por arrasto (Affinity, Illustrator) | é bom, mas é uma 2.ª obra (gravar/repor por layout); entra depois, se o Enio pedir |
| modo *Paint* no vetor | D6: um modo que não pinta é controlo morto (`pesquisa/05`) |
| copiar UI do Affinity pixel a pixel | o oráculo ensina a PERGUNTA; a resposta é da nossa entrada (memória) |

## §6 — As decisões do Enio (2026-10-07)

| # | pergunta | decisão | como |
|---|---|---|---|
| 1 | a barra de ferramentas à esquerda flutua ou ocupa coluna? | **flutua** sobre a área, como a do Quadro | *«as demais como vc sugere»* |
| 2 | as opções da ferramenta ficam em cima ou embaixo? | **em cima** (a fila que já existe); embaixo como opção de layout para tablet | idem |
| 3 | que padrão de teclas? | ⭐ **Illustrator** — *«qual mais aceito (…)? Vamos com o mais bem aceito.»* A prova está no estudo §5: o próprio Inkscape instalado traz o perfil do Illustrator e nenhum do Affinity | pesquisa + perfil medido |
| 4 | a barra de ações perto da seleção nasce ligada? | **ligada no desktop E no iPad** (contra a recomendação, que era desligada no desktop) | *«4- ligada para ambos»* |
| 5 | a Hierarquia fica à esquerda? | **fica** | *«as demais como vc sugere»* |

⚠️ **A consequência da decisão 4:** com a ④ ligada em todo lado, ela deixa de ser opcional e entra
no bloco F1–F3 (§4). A lição do Illustrator continua obrigatória — *uma barra que tapa a arte e não
se pode fixar nem desligar é desligada pela maioria* (CTB, R3) —, então **fixar** e **desligar**
fazem parte da entrega, não de uma fase seguinte.
