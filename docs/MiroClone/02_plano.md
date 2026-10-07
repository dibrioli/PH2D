# 02 — PLANO: o Quadro (MiroClone) dentro do PH2D

> **Ordem do dono (2026-10-05):** *«Aceito suas sugestões. Talvez nossas formas e setas NÃO funcionem tão
> bem como nos apps referência. Talvez precisemos criar algo melhor para esse módulo. Se quiser baixar e
> instalar o Excalidraw para usar como ref, faça isso. Depois crie o plano.»*
>
> **Decisões do dono que este plano assume** (as quatro do [01 §8](01_pesquisa_miro_excalidraw.md)):
> 1. **Equipa em duas etapas** — Etapa 1 (este plano): o quadro no computador, com comentários e histórico
>    **no ficheiro**; Etapa 2 (plano próprio, depois): ao vivo. O modelo de dados da Etapa 1 já nasce pronto
>    para a 2 (§1.2), para ela não pedir migração.
> 2. **Sticky cresce na vertical** (FigJam), não encolhe a letra (Miro).
> 3. **Aparência com botão «Rascunho ↔ Final»** — traço à mão (rough) ou limpo, por quadro e por elemento.
> 4. **Barra curta**, com os atalhos de uma tecla só dentro do quadro.
> 5. **Cada quadro é uma ABA na barra superior do app e não aparece na Hierarquia** (ordem posterior do
>    mesmo dia — substitui o «modo próprio no seletor *Mode*» do 01 §8.5; §1.1).
>
> Pesquisa: [01](01_pesquisa_miro_excalidraw.md) · oráculo instalado: [`ferramentas/excalidraw_oracle/`](ferramentas/excalidraw_oracle/README.md).

---

## §0 — O veredito sobre as nossas formas e setas: reusar o NÚCLEO, construir o QUADRO por cima

A desconfiança do dono foi medida contra o código e contra o oráculo. Resultado:

| peça | estado | prova |
|---|---|---|
| **roteador de setas** (`ph2d-vec-connect`) — A* sobre grafo de visibilidade ortogonal, **puro** (só `serde`, 1 384 linhas) | ✅ **melhor que o oráculo** | o Excalidraw 0.18.1 corrido aqui: a seta em cotovelo vai **através** do obstáculo (`(180,205)→(500,205)` sobre a caixa `x 280..400`); desalinhada, o segmento `x=340` cai dentro dela — [README do oráculo](ferramentas/excalidraw_oracle/README.md#o-que-já-mediu). O nosso desvia (`crates/ph2d-app-vec/src/connector_walls.rs`). ⚠️ **Ordem do dono (06/10): as setas do quadro NÃO desviam** (o idioma do Miro) — o roteador fica só para o cotovelo contornar as SUAS duas formas (§6) |
| âncora ao centro (`Anchor::Floating`) × ponto fixo (`Anchor::Port`), recto/cotovelo/curvo, rótulo, 8 pontas, alvo apagado ⇒ ponta solta | ✅ | `crates/ph2d-ecs/src/vec_connector.rs:38-72` · `crates/ph2d-vec-scene/src/marker.rs` |
| geometria das formas (`ShapeKind`, ~50 tipos incl. fluxograma) | ✅ | `crates/ph2d-vec-scene/src/kind.rs:38-98` |
| **a rota recalcula-se a CADA QUADRO, para cada seta**, sem cache de rota nem marca de sujo (só a histerese do lado, `SideCache`) | ⚠️ risco a escala | `crates/ph2d-app-vec/src/connector_live.rs:46,409-452` — nunca medido com centenas de setas |
| criar a forma seguinte JÁ LIGADA (pontos azuis) · `Ctrl` para não ligar | ⭐ | `shells/desktop/src/connector_gesture.rs:86-119` |
| texto DENTRO da forma com quebra, centrado, forma que cresce | ⭐ | o texto é objecto à parte (`VecShape::Text`); o rótulo só existe no conector |
| traço à mão | ⭐ | nenhum no vectorial |
| o ciclo de vida (cozer ao vivo, efeitos, blends, nós de bezier) | pensado para **ilustração** | puxá-lo para um quadro de notas é atrito e custo por elemento |

⇒ **Decisão técnica (delegada ao padrão-ouro):** o Quadro **reusa** o roteador puro (`ph2d-vec-connect`) e a
geometria das formas, e **constrói** um núcleo próprio de quadro — modelo de documento, gestos, texto em
forma, cache de rota, desenho — sem herdar o modo Vector. ⛔ Corrigido do inventário: o `VecConnector` **não**
está em contrato congelado (o congelado é o modelo vectorial antigo, ADR-0056..0068); reusá-lo seria
permitido, só não é a melhor forma.

---

## §1 — Arquitectura

### §1.1 — O que é um Quadro no editor: um DOCUMENTO numa ABA, nunca um objecto da cena

> **Ordem do dono (2026-10-05):** *«Os quadros não devem aparecer na hierarquia mas devem aparecer como abas
> na barra superior do APP.»*

- **Um quadro não é objecto da cena:** não aparece na Hierarquia, não é entidade, não tem `ObjectKind` nem
  `ModeFamily`. É um **documento** do projecto, ao lado da cena.
- **Abas de documento na barra superior:** `[Cena] [Quadro 1] [Quadro 2] [+]`. Clicar troca a área central;
  `+` cria um quadro; duplo-clique renomeia; arrastar reordena; botão direito: Renomear · Duplicar · Apagar
  (com confirmação). Hoje a barra pinta os grupos da esquerda (`clusters[..split]`) e os da direita
  (`crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs:276-355`) e **não há abas de documento** — as
  abas que existem são de PAINÉIS (`screens/hero/slot_tabs.rs`). A fila de abas nasce num módulo **irmão**
  (`topbar/document_tabs.rs`, ids num ficheiro novo em `ids/chrome/`), no espaço entre os dois grupos, com o
  idioma visual das abas de painel (mesma altura, mesmo arrastar).
- **Trocar de aba troca o conteúdo, não a tela.** O precedente vivo é o modo das ferramentas de imagem, que
  troca o lado direito da barra e as ferramentas visíveis (`topbar/mod.rs:343-355`,
  `topbar/image_action_row.rs`, `tool.rs:395`). Com uma aba de quadro activa: a área central desenha o quadro
  (câmara própria por quadro — pan/zoom gravados), a **barra curta** de ferramentas flutua à esquerda da área
  (o idioma do Miro/Excalidraw), e os painéis laterais da cena dão lugar aos do quadro (Frames, Estilo,
  Comentários). Voltar à aba `Cena` devolve tudo como estava.
- **Atalhos de uma tecla** só valem com uma aba de quadro activa e a área com foco — o despacho já dá
  prioridade ao widget com foco sobre a área (`crates/ph2d-editor-core/src/interaction/dispatch/key.rs:33`).
- **Os elementos do quadro também não são entidades.** Vivem no `BoardDoc` (§1.2); um quadro com 100 mil
  notas não toca no ECS, na Hierarquia nem no laço de sistemas.

### §1.2 — O modelo de documento (`ph2d-board-model`, crate pura: sem ECS, sem render)

- `BoardDoc { elements: BTreeMap<ElementId, Element>, … }` — `BTreeMap` (determinismo, HR).
- Cada `Element`: `id` estável · `kind` (Sticky, Shape, Text, Connector, Freehand, Image, Frame, Comment,
  MindNode) · geometria · estilo · `z` por **índice fraccionário** · `parent_frame` · `group_ids` ·
  `locked` · `link` · **`version: u64` + `nonce`** · **lápide `deleted`**.
  ⇒ os três últimos são o que a Etapa 2 (ao vivo) precisa — é o esquema do Excalidraw (união por elemento,
  ganha a versão maior) e deixa aberta a troca por CRDT sem migrar ficheiros.
- **Operações** (`BoardOp`: inserir, apagar, mudar campos, mover z) — o undo e, na Etapa 2, a rede consomem
  as MESMAS operações.
- Cores do documento = **dado do utilizador** (RGBA no documento); a paleta por omissão vem de `ph2d-tokens`
  — zero hex no código (HR-15).
- Persistência: um campo novo no `ProjectFile` (`shells/desktop/src/project.rs:16`), `boards: Vec<u8>` — um
  **`BoardSet`** (os `BoardDoc`, a ordem das abas, o nome e a câmara de cada quadro) como **blob com versão
  própria**, o idioma do campo `timeline: Vec<u8>` ⇒ **um** degrau de `PROJECT_SCHEMA` para todo o módulo
  (hoje `183`, `shells/desktop/src/project_schema.rs:392`; recontado na integração com
  `scripts/schema-recount.py`). Os quadros viajam **com o projecto**, não com o layout do utilizador
  (`~/.ph2d/layout.txt` é por máquina).

### §1.3 — As crates

| crate | o quê | depende de |
|---|---|---|
| `ph2d-board-model` | documento, ops, ids, índice fraccionário, serde | — (pura) |
| `ph2d-board-layout` | texto na forma/sticky (parley: quebra, centrado, crescer), layout de mind map (árvore arrumada), arrumar em grelha | `parley`, model |
| `ph2d-board-rough` | **traço à mão** (porta do algoritmo do rough.js, MIT) e **contorno do desenho livre** (porta do perfect-freehand, MIT), com o aviso de copyright; semente por elemento ⇒ determinístico | `kurbo` |
| `ph2d-vec-connect` | **reusado**: o roteador | — |
| `ph2d-board-render` | índice espacial, recorte ao ecrã, fragmentos Vello em cache por elemento (refaz só o sujo), nível de detalhe por zoom | `vello`, model, layout, rough |
| `ph2d-app-board` | a família: a área do quadro (câmara, ferramentas, gestos, barra curta, painéis), roteador de smokes `PH2D_BOARD_SMOKE` | `ph2d-app-host`, tudo acima |
| `ph2d-editor-core` (foundational, aditivo) | a fila de abas de documento (`topbar/document_tabs.rs` + ids próprios) e o estado «documento activo» | — |

A shell só **compõe** (`the_shell_only_shrinks`); o registo da família é gerado (`ph2d-app-sync` →
`ph2d-app-registry-init`).

### §1.4 — Undo

Cada gesto produz um lote de `BoardOp` com o inverso. **Regra de produto:** com uma aba de quadro activa,
`Ctrl+Z` desfaz o último gesto **daquele quadro** — nunca algo da cena nem de outro quadro, e vice-versa
(documentos separados, como as abas do Miro/Figma).
⚠️ **A decidir na W0, medindo:** (a) entradas na fila única do editor (`shells/desktop/src/undo.rs`) marcadas
com o documento e filtradas pela aba activa, ou (b) histórico próprio por quadro — conferindo primeiro, no
código, como o Motion e a Timeline (que também vivem fora do ECS) fazem hoje. Em qualquer dos dois, o que
entra é o lote de ops, nunca o blob inteiro (100 mil elementos por gesto não cabem num diff).

---

## §2 — Velocidade: o tecto é o do hardware (§0.0)

**Contra quem:** o Miro pede < **5 000** objectos; o Excalidraw cai a ~30 fps com 10–14 mil, e trava acima
de 5 mil (sem índice espacial). **Alvo de trabalho:** 100 000 elementos com pan e zoom a 60 Hz nesta
máquina — o número final **mede-se**, não se escolhe.

- **Régua** (regra do dono de 05/10): as variantes são caminhos do MESMO processo escolhidos em execução
  (flag no objecto ou `override` do WGSL); 7 rodadas × 20 quadros intercaladas com ordem rodada; vale o
  **mínimo** (mediana ao lado como controlo); relógio da placa (`ph2d_gpu::pass_profiler::drain`, ou a soma
  por janela do perfilador até ele chegar); uma compilação só em `--profile smoke`; `loadavg` anotado.
- **Cena de medida gerada** (W0): N notas + N/5 formas + N/10 setas + texto, N ∈ {1k, 10k, 100k}.
- ⛔ **Kill-criterion (antes do código):** se depois da 2.ª tentativa o quadro de **10 mil** elementos
  (formas + texto) passar de **8 ms** por quadro na placa, o desenho por fragmentos em cache não existe nesta
  forma — pára-se e prova-se o modelo de render antes da 3.ª.
- Setas: rota **em cache**, refeita só quando uma ponta ou um obstáculo próximo muda (corrige o risco do §0).

### §2.1 — Medido na W0 (2026-10-05)

**CPU — encodar o quadro inteiro numa `VectorScene`** (`ph2d_board_render::paint`, todos os
elementos visíveis, grelha de pontos incluída), `--release`, 7 rodadas × 20 quadros intercaladas com
ordem rodada, mínimo e mediana; `loadavg 12.45 21.16 20.65`. Régua versionada:
`crates/ph2d-board-render/tests/it/measure_encode_cost.rs` (`#[ignore]`, à mão).

| elementos | mínimo ms/quadro | mediana |
|---|---|---|
| 1 000 | 0,093 | 0,094 |
| 10 000 | 0,450 | 0,460 |
| 100 000 | 3,441 | 3,617 |

⇒ o encode cresce linear (~34 ns por rectângulo) e não é o tecto: 100 mil cabem em 3,4 ms de CPU
**reconstruindo tudo a cada quadro**.

**PLACA — o Vello rasteriza a mesma cena** (`VelloPass::render_to_intermediate` do produto, 1920×1080),
RTX 5060 Ti / Vulkan, `TIMESTAMP_QUERY` à volta das submissões do Vello (com enchimento na fila para não
contar a placa ociosa durante o encode do Vello; `descobertos` = 0 em todas as corridas). Mesma régua (7×20,
ordem rodada, mínimo e mediana), um aquecimento por N fora dela, e um controlo de cobertura (área laranja
lida de volta ≈ a esperada: os buffers fixos do Vello não transbordaram a 100 mil). Régua:
`crates/ph2d-board-render/tests/it/measure_gpu_raster_cost.rs` (`#[ignore]`, à mão com `PH2D_GPU=1`).
Três corridas com `loadavg` 10–22 (outras linhas a compilar): a coluna da placa quase não mexe
(10 mil: 0,364 → 0,366 → 0,370); as colunas de CPU sim. Corrida 1, `loadavg 10.53 13.02 14.75`:

| rectângulos | placa mín ms | mediana | Vello CPU mín | parede mín (chamada → fila vazia) |
|---|---|---|---|---|
| 1 000 | 0,257 | 0,258 | 0,137 | 0,439 |
| 10 000 | 0,364 | 0,367 | 0,192 | 0,607 |
| 100 000 | 1,465 | 1,469 | 0,689 | 2,288 |

⇒ **com rectângulos a placa não é o tecto**: 10 mil ficam 22× abaixo dos 8 ms do kill-criterion, e a
100 mil o encode da CPU (4–6 ms sob esta carga) já pesa mais que o raster. ⚠️ O kill-criterion fala de
**formas + texto** e a cena ainda só tem rectângulos: ele continua a decidir-se na W1, com esta régua
(a cena de medida ganha formas e texto).

⛔ **Dois defeitos que a montagem desta régua apanhou** (curados no mesmo dia): o `z_on_top` era
O(n) por chamada (100 mil elementos = 10¹⁰ comparações; a régua não terminava) e a chave de z só com
fracção crescia ~1 carácter a cada 6 acrescentos. Cura: o esquema publicado do
`fractional-indexing` (parte inteira de comprimento variável) e o topo de z lembrado — gates com os
17 vectores publicados e 100 mil acrescentos com chaves ≤ 5 caracteres.

---

### §2.2 — Medido na W1 (2026-10-06): o kill-criterion PASSA

Mesmas réguas (CPU `measure_encode_cost.rs`, placa `measure_gpu_raster_cost.rs`), agora com três cenas:
`Rects` (a da W0), `Shapes` (rectângulo/elipse/losango com contorno, metade com cantos redondos, um em
cinco rodado, uma palavra em cada — a vista enquadra o quadro inteiro) e `ShapesNear` (as mesmas, com a
vista a pôr a letra a 14 px no ecrã). `loadavg ~3`, RTX 5060 Ti / Vulkan, mínimo de 7×20:

| cena | N | CPU encode ms | placa ms | Vello CPU ms |
|---|---|---|---|---|
| Rects | 10 000 | 0,52 | 0,37 | 0,22 |
| Rects | 100 000 | 6,09 | 1,46 | 0,68 |
| Shapes (longe) | 10 000 | 6,07 | **1,31** | 0,76 |
| Shapes (longe) | 100 000 | 8,20 | 1,48 | 0,65 |
| ShapesNear | 10 000 | 0,90 | 0,45 | 1,13 |
| ShapesNear | 100 000 | 5,40 | 0,48 | 1,25 |

⇒ **10 mil formas + texto = 1,31 ms de placa**, 6× abaixo dos 8 ms do kill-criterion; o pior quadro
inteiro (100 mil de longe) cabe em ~10 ms (CPU + placa) — 60 Hz com folga. Três curas medidas no mesmo
dia (cada uma com o número de antes): letra abaixo de **6 px** desenha-se como traço (*greeking*: com o
tecto a 2 px, 10 mil palavras de 2 px custavam **14,9 ms** de placa); contorno GUARDADO por forma e o
rectângulo simples pelo `fill_rect` (refazer o contorno a cada quadro levou 10 mil rectângulos de 0,45
a **3,1 ms**); forma com menos de **4 px** é um ponto de cor (a 2 px, 100 mil formas de 2,5 px custavam
**26,6 ms**). ⛔ Rejeitado e desfeito: o índice de z mantido pelas ops (100 mil rectângulos 7,8 ms contra
6,1 a ordenar a cada quadro — §6).

### §2.3 — Medido na W2 (2026-10-06): as setas

Cena `Flow` (partilhada pelas três réguas): N formas 160×100 em grelha com o vão de nascença (80) e
N/10 setas CURVAS entre vizinhas, presas ao centro, vista a enquadrar tudo. `loadavg ~5`, RTX 5060 Ti
/ Vulkan, mínimo de 7×20 (mediana entre parênteses). Réguas: `measure_route_cost` (cache),
`measure_encode_cost` (CPU), `measure_gpu_raster_cost` (placa).

| N formas | setas | 1.ª sincronização | parado | arrastar 1 forma (revistas / refeitas) | encode CPU | placa |
|---|---|---|---|---|---|---|
| 1 000 | 100 | 0,39 ms | 0 | 0,011 ms (1 / 1) | 0,18 ms | 0,29 ms |
| 10 000 | 1 000 | 3,7 ms | 0 | 0,15 ms (1 / 1) | 1,19 ms | 0,56 ms |
| 100 000 | 10 000 | 43 ms | 0 | 3,9 ms (1 / 1) | 8,9 ms | 1,77 ms |

⇒ **10 mil formas com mil setas = ~1,3 ms de CPU e 0,56 ms de placa por quadro a arrastar**; o pior caso
(100 mil + 10 mil setas, TUDO à vista, a arrastar) ~13 ms de CPU — 60 Hz cabe. O «arrastar» de 100 mil é
quase todo o passeio da diferença (O(N) pelas versões). Nível de detalhe (seta < 4 px no ecrã = traço de
ponta a ponta): o encode de 100 mil + 10 mil setas passou de **11,2** para **8,9 ms**.

**História medida desta onda** (recusas no §6): a 1.ª versão desviava das formas com a lei do vectorial
(o ponto fixo de obstáculos sem tecto) e cada seta via o quadro INTEIRO — 1 000/1 000 e 10 000/10 000
formas, a régua a 100 mil ficou 12 min sem acabar; um tecto de região (`DETOUR_K`) trouxe-a a ~28
obstáculos por seta e a 1.ª sincronização de 10 mil setas a **207 ms**. O dono recusou o desvio (06/10)
e a curva-de-cotovelo; sem A\* nas curvas, a mesma sincronização custa **43 ms**.

## §3 — As ondas (cada uma fecha com gate batched, smoke e o que o dono vê)

Ordem pensada para o quadro ser **usável cedo**: depois da W3 já se faz um brainstorm.

### W0 — Fundação e réguas
- Crates com o molde; **abas de documento** na barra superior (`[Cena] [+]`); `+` cria «Quadro 1»; trocar,
  renomear, reordenar, apagar; `BoardSet` gravado e carregado com o projecto; undo de uma op por quadro (§1.4).
- Cena de medida gerada (§2) + a régua A/B; **primeira medição** do custo de desenhar N rectângulos.
- Teste de seam (`ph2d-ui-testkit`): clique em `+` → nasce a aba → clique nela → a área central é o quadro e a
  barra curta aparece → clique em `Cena` → a cena volta intacta; e a Hierarquia **não** ganha linha nenhuma.
- **Dono vê:** a barra de cima com `Cena` e `+`; criar dois quadros, alternar entre eles e a cena, fechar e
  reabrir o projecto e encontrar os quadros lá.
- **Estado em 2026-10-05 (W0a, commits `4694a5273..`):** ✅ abas `Scene · Board n · +` na barra de menus
  (clique real com gate), área do quadro (grelha de pontos de densidade constante, zoom à volta do cursor,
  arrastar), a fila de chips da cena some num quadro, gravar/abrir no `.ph2dproj` (v184, recusa de blob
  ilegível), cena `PH2D_BOARD_SMOKE=1` fotografada, régua de encode (§2.1). ✅ **Smoke aprovado pelo dono.**
- **W0b (2026-10-05, 2.ª janela):** ✅ renomear NO LUGAR (duplo-clique ou *Rename*; `Enter`/clicar fora
  grava, `Esc` desiste), menu do botão direito (*Rename · Duplicate · Delete…*, apagar pergunta antes),
  reordenar arrastando (com a marca de onde cai), régua da PLACA (§2.1). ⏳ smoke do dono; transbordo
  das abas (medir) e undo por quadro (§1.4, nasce na W1) — ver o handoff de continuação mais recente em
  [`handoffs/`](handoffs/).

### W1 — Tela, formas e texto
- Pan/zoom infinito (rato, trackpad, `Espaço`+arrastar), grelha de pontos, snap, guias de alinhamento.
- Seleccionar, mover, redimensionar, rodar, duplicar (`Ctrl+D`), apagar, multi-selecção.
- Formas: rectângulo (cantos redondos), elipse, losango, triângulo, e as de fluxograma da `ShapeKind`.
- **Texto dentro da forma:** duplo-clique escreve, quebra automática, centrado, a forma cresce.
- Estilo: preenchimento, contorno, tracejado, espessura, opacidade.
- **Oráculo:** contorno das formas com `roughness 0` contra o SVG do Excalidraw (geometria dos `d=`).
- **Kill-criterion do §2 decide-se aqui.**
- **Dono vê:** desenhar caixas, escrever dentro, arrumá-las.
- **Estado em 2026-10-06 (W1, commits `c7557f5b5..`):** ✅ 18 formas (básicas + fluxograma ISO), estilo
  (preenchimento, contorno, espessura, traço, cantos, opacidade, letra), texto dentro (duplo-clique/`Enter`,
  quebra, centrado, a forma cresce), seleccionar/mover/redimensionar/rodar/duplicar/apagar/copiar/colar,
  guias de alinhamento, `Espaço`/mão para a vista, desfazer POR quadro (teclado e menu), barra curta e barra
  de estilo; oráculo dos cantos (§4) e kill-criterion (§2.2) fechados. Smoke `PH2D_BOARD_SMOKE=2`.
  ✅ **Smoke aprovado pelo dono (06/10).** O que fica aberto está no handoff de continuação da W1 em [`handoffs/`](handoffs/).

### W2 — Setas (a peça em que já somos melhores — terminar de a fazer brilhar)
- Gesto: arrastar a ponta sobre uma forma **realça** o alvo e liga; `Ctrl` solta; linha livre.
- Âncora ao centro × ponto fixo (a regra do Miro, §4.1 do 01), recto/cotovelo/curvo, rótulo, pontas.
- **Pontos azuis**: um clique na borda cria a forma seguinte já ligada; `Ctrl+seta` cria pelo teclado.
- Cache de rota (§2); desvio de obstáculos (o roteador reusado).
- **Oráculo:** a normalização da ligação (`fixedPoint`, lado de saída) contra o `restored.json`; o desvio
  mede-se contra o NOSSO critério (nenhum segmento dentro de uma caixa), porque nele o oráculo falha.
- **Dono vê:** um fluxograma feito só com cliques e teclado, que se reorganiza ao mover caixas.
- **Estado em 2026-10-06 (W2, commits `3e7c3564b..`):** ✅ seta no documento (`ElementKind::Connector`,
  no fim do enum: formato 2 não sobe), ferramenta Seta (`A`/`5`, botão na barra curta): arrastar de
  uma forma a outra realça o alvo e liga (`Ctrl` solta); miolo = centro, faixa junto ao contorno =
  ponto fixo colado ao meio do lado; arrastar a ponta de uma seta seleccionada religa-a; **curva do
  Miro de nascença** (sai/entra perpendicular, braço medido), recta, cotovelo (contorna só as suas
  formas); **pontos de ajuste** (bolinha a meio de um trecho cria, círculo oco arrasta, duplo-clique
  apaga); 5 pontas na barra (8 no documento), rótulo (`Enter`/duplo-clique) sobre recorte do fundo;
  pontos azuis (selecção ou passar o rato) e `Ctrl+seta` criam a forma seguinte já ligada; apagar
  solta as pontas onde estão; copiar/duplicar religa as cópias. ⛔ **Nenhuma seta reage a outra forma**
  (ordem do dono, 06/10). Cache de rota por diferença (§2.3). Oráculo (§4): `fixedPoint`, Z a meio do
  vão, volta com recuo 40 medido, caixa do meio ignorada como no oráculo. Smoke `PH2D_BOARD_SMOKE=3`.
  ✅ **Smoke aprovado pelo dono (06/10).**

### W3 — Notas adesivas (o coração do brainstorm)
- Sticky (`N`): paleta de cores, três tamanhos, **cresce na vertical**; `Tab` cria a seguinte à direita
  (mesma cor e tamanho); `Enter` em massa; pilha de onde se arrastam; **colar uma planilha = uma nota por
  célula**; pega que arruma uma selecção em grelha.
- Texto com **negrito/itálico/cor por trecho** (o Excalidraw não tem — queixa dele).
- **Dono vê:** encher o quadro de ideias depressa e arrumá-las.

### W4 — Caneta e o botão «Rascunho ↔ Final»
- Desenho livre com pressão da mesa digitalizadora; borracha; ponteiro laser (`K`).
- Traço à mão com semente por elemento; o botão troca o quadro (ou a selecção) entre rascunho e final.
- **Oráculo:** caminhos do rough.js por elemento com a MESMA semente; contorno do perfect-freehand — por passo.
- **Dono vê:** o mesmo diagrama em «guardanapo» e em «apresentação» com um clique.

### W5 — Organizar
- **Frames**: levam os filhos, nome e cor, aninháveis, pertença visível; painel de Frames.
- Agrupar, travar, alinhar/distribuir, ordem (frente/trás), minimapa (`M`), busca (`Ctrl+F`) com filtro por
  etiqueta, links entre elementos.
- **Nível de detalhe por zoom** (queixa aberta no Miro): longe, notas viram blocos de cor e frames mostram só o título.
- **Dono vê:** um quadro grande que continua legível e rápido de navegar.

### W6 — Planear
- **Mapa mental**: `Tab` filho, `Enter` irmão, setas navegam, ramos recolhem, layout automático.
- **Modelos**: retrospectiva, brainstorm, kanban, mapa mental, fluxograma, mapa de jornada.
- **Dono vê:** começar um planeamento a partir de um modelo e expandi-lo pelo teclado.

### W7 — Facilitação (a parte que funciona num ecrã só)
- Cronómetro visível no quadro; carimbos/pontos de voto; agrupar notas por cor, etiqueta ou palavra.
- ⚠️ A votação **anónima** e o modo **privado** precisam de várias pessoas ao mesmo tempo ⇒ Etapa 2.

### W8 — Equipa assíncrona e saída
- **Comentários** em fio, presos a um elemento ou a um ponto, com autor e «resolvido», gravados no ficheiro.
- **Versões nomeadas** dentro do ficheiro; restaurar como cópia (o idioma do Miro).
- **Apresentação** pelos frames, na ordem do painel.
- Exportar PNG/SVG/PDF do quadro, de um frame ou da selecção.
- **Importar e exportar `.excalidraw`** — traz quadros de quem já usa o Excalidraw, e fecha o laço do
  oráculo (ida e volta medida).

### Etapa 2 — Ao vivo (plano próprio, depois da W8)
Rede, presença, cursores, seguir/«trazer todos», votação anónima, modo privado. O modelo (§1.2) já tem
versão, nonce, lápide e operações — a Etapa 2 escolhe transporte e reconciliação sem migrar ficheiros.

---

## §4 — Oráculos por onda

| onda | o quê se compara | com quê |
|---|---|---|
| W1 | contorno das formas (`roughness 0`) | `exportToSvg` (geometria dos `d=`, depois pixel) |
| W2 | normalização da ligação; forma do cotovelo | `restored.json` / `editor.json` — ⛔ não o desvio |
| W4 | traço à mão (mesma semente); contorno do desenho livre | caminhos do rough.js / perfect-freehand, por passo |
| W8 | importação e exportação `.excalidraw` | ida e volta pelo oráculo |
| todas | comportamento (atalhos, gestos) | Miro e FigJam **observados à mão** — referência, nunca fixture |

As fixtures novas vão para `ferramentas/excalidraw_oracle/saidas/` com o `meta.json`; cada corrida vira gate.

---

## §5 — Smokes

Roteador `PH2D_BOARD_SMOKE=<n>` na crate da família. Uma cena por onda, cada uma que ENSINA o que a onda
trouxe (§5.0: cena que ensina o contrário é pior que nenhuma), fotografada antes de mandar ao dono
(`docs/Components/ferramentas/fotografa_cena.sh`). Smoke de performance (100 mil elementos) em `--release`.

---

## §6 — Riscos e recusas

| assunto | estado |
|---|---|
| herdar o modo Vector inteiro para o quadro | ⛔ recusado (§0): ciclo de ilustração, sem cache de rota |
| índice de z mantido pelas ops (`BTreeSet<(FracKey, id)>`) em vez de ordenar a cada quadro | ⛔ **medido e desfeito** (06/10): 100 mil rectângulos 7,8 ms contra 6,1 — as buscas por id custam mais que a ordenação de chaves curtas (§2.2) |
| «escrever com uma forma seleccionada começa o texto» | ⛔ recusado na W1 (06/10): as letras soltas são atalhos (`R`, `O`, `D`, `V`, `H`); com a regra, desenhar uma forma e carregar `R` para a seguinte escrevia «r» nela. Escreve-se com `Enter` ou duplo-clique (o idioma do Excalidraw). Volta a pôr-se nas notas (W3), onde é o idioma do Miro |
| quadro como objecto da cena (`ObjectKind::Board` + `ModeFamily`, a 1.ª versão deste plano) | ⛔ **ordem do dono 05/10**: abas na barra superior, fora da Hierarquia (§1.1) |
| elementos como entidades ECS | ⛔ recusado (§1.1): Hierarquia, undo e laço a crescer com cada nota |
| copiar o desvio de setas do Excalidraw | ⛔ medido pior que o nosso — e hoje o quadro não desvia de todo (linha abaixo) |
| ⭐ setas que DESVIAM das formas no caminho (o roteador com as outras formas por obstáculo) | ⛔ **ordem do dono (06/10)**: *«setas não se reajustam sozinhas»* — o idioma do Miro: a rota depende só da seta, das suas duas formas e dos pontos de ajuste do artista. O cotovelo usa o roteador só para contornar as SUAS duas formas |
| braço nos pontos de ajuste de ⅓ (o do vectorial) ou de ½ (escolhido a OLHO no 3.º smoke), e braço da ponta = ½ do afastamento AO LONGO da saída | ⛔ **medido** (06/10) na captura do Miro com pontos (`ferramentas/capturas_miro/`, régua `mede_curva_miro.py`): as tangentes do Catmull-Rom estão certas (< 1,1 px por trecho com braços livres); o melhor braço global é **0,4** (3,44 px) — ⅓ 3,78, ½ 3,98; Catmull-Rom uniforme / centrípeto / cordal piores. A lei «ao longo da saída» achatava a volta de duas pontas para o mesmo lado ⇒ 0,45 da DISTÂNCIA (a 1.ª captura serve as duas) |
| recuar a linha para a ponta de seta pela poligonal das âncoras (o `trim_path` do vectorial), ou recuando a âncora com os braços ao longo da tangente | ⛔ 2.º e 3.º smokes do dono: o 1.º entortava a curva (a linha entrava de lado no triângulo); o 2.º separava o traço da GUIA da selecção ⇒ a linha é a PRÓPRIA rota cortada onde fica à distância da cabeça (`chord_cut`) e a cabeça aponta da ponta para esse corte |
| a cabeça da seta orientada pela curva nos seus últimos píxeis | ⛔ 4.º smoke do dono (06/10): uma curva que dobra à chegada virava o «V» de lado ⇒ debaixo de cada cabeça a curva acaba numa HASTE recta perpendicular à forma, do comprimento da cabeça (`curved_with_stems`); a cabeça e o traço assentam sempre nela |
| a curva a encaixar na direcção do LADO da caixa | ⛔ 5.º smoke do dono (06/10): num círculo ou numa ponta arredondada a seta deve encaixar na NORMAL do contorno ⇒ `ph2d_board_geom::outline_normal` (média numa junção lisa; num canto vivo, como o vértice do losango, fica a do lado) |
| o A\* do roteador a ordenar pelo custo CRU (`f64` com todos os dígitos) | ⛔ **medido** (6.º smoke do dono, 06/10: «as setas rectangulares tremem»): duas rotas de custo igual (a dobra a meio do vão e no recuo) diferiam por ~1e-13 e o arredondamento escolhia — 846 saltos > 5 un. em 3 600 passos de arrasto. O custo ordena-se ARREDONDADO a 1e-6 (`f_key`, `ph2d-vec-connect`, partilhado com o Vector) e o desempate pela centralidade decide: 2 saltos |
| o recuo fixo de 40 num vão menor que 80 entre saídas que se olham; o trecho a contornar a forma da OUTRA ponta | ⛔ medido no mesmo smoke: 8 mudanças de forma num vão curto; 24 saltos com um ponto de ajuste ⇒ `facing_jetty` (metade do vão) e só a forma da ponta do trecho por obstáculo — 2 e 0. Gate `the_elbow_does_not_jitter_while_a_box_is_dragged` |
| a curva = o cotovelo suavizado (o `RouteKind::Curved` do vectorial) | ⛔ **ordem do dono (06/10)**: «curvas exageradas»; a curva é a do Miro — uma cúbica perpendicular às faces com o braço medido na captura dele (`END_ARM`) |
| ler o código do Excalidraw para portar o render | ⛔ §0.9 — corre-se. A porta do rough.js e do perfect-freehand é a **permissiva** (MIT, com aviso) e confere-se por passo contra o oráculo |
| fontes do Excalidraw além da Excalifont/Virgil | ⚠️ não triadas — nenhuma embarca sem triagem |
| rede na Etapa 1 | ⛔ decisão do dono (Etapa 2) |
| texto rico: parley tem estilo por trecho? | ⚠️ a medir na W3 antes de desenhar |
| a região de obstáculos de uma rota SEM tecto (a lei do vectorial) | ⛔ medido (06/10): num quadro denso cada seta via o quadro INTEIRO (1 000/1 000, 10 000/10 000). Morreu com o desvio (linha acima); fica a lição para quem o quiser de volta: precisa de tecto |
| o `0.5001` do `fixedPoint` (e o `105.009` da rota) do oráculo | ⛔ desempate dele, não lei: o nosso meio de lado é `0.5` exacto (gate com tolerância 0,01) |
| a ponta `arrow` com a abertura do Excalidraw (~20°) | ⛔ fica a do catálogo de pontas do vectorial (26,6°): uma lei de pontas para o app; o TAMANHO é o medido (`HEAD_SCALE`) |
| rever TODAS as setas a cada quadro (o `recook` do vectorial) | ⛔ recusado para o quadro (§0): a cache revê só a seta que mudou ou cuja forma de ponta mudou |
| um ponto fixo a contar para o afastamento das paralelas | ⛔ foto da cena 3 (06/10): empurrava a seta do centro para fora do vértice do losango; só contam setas presas ao centro nas duas pontas |
