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
> 4. **Barra curta, num MODO próprio** do editor (seletor *Mode*), com os atalhos de uma tecla só lá dentro.
>
> Pesquisa: [01](01_pesquisa_miro_excalidraw.md) · oráculo instalado: [`ferramentas/excalidraw_oracle/`](ferramentas/excalidraw_oracle/README.md).

---

## §0 — O veredito sobre as nossas formas e setas: reusar o NÚCLEO, construir o QUADRO por cima

A desconfiança do dono foi medida contra o código e contra o oráculo. Resultado:

| peça | estado | prova |
|---|---|---|
| **roteador de setas** (`ph2d-vec-connect`) — A* sobre grafo de visibilidade ortogonal, **puro** (só `serde`, 1 384 linhas) | ✅ **melhor que o oráculo** | o Excalidraw 0.18.1 corrido aqui: a seta em cotovelo vai **através** do obstáculo (`(180,205)→(500,205)` sobre a caixa `x 280..400`); desalinhada, o segmento `x=340` cai dentro dela — [README do oráculo](ferramentas/excalidraw_oracle/README.md#o-que-já-mediu). O nosso desvia (`crates/ph2d-app-vec/src/connector_walls.rs`) |
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

### §1.1 — O que é um Quadro no editor

- **Um TIPO de objecto novo**, `ObjectKind::Board` (6.ª variante em `crates/ph2d-component-desc/src/lib.rs:334`,
  `ALL` passa a 6, chave `component.object_kind.board`), com marcador próprio.
- **Um MODO**, aberto por uma `impl ModeFamily` (`crates/ph2d-editor-core/src/screens/hero/mode_drive.rs:23`;
  moldes vivos: `crates/ph2d-app-flip/src/flip_mode.rs:88`, `crates/ph2d-app-painter/src/paint_mode.rs:22`).
  Entrar no quadro (`Tab`) dá a barra curta e os atalhos de uma tecla; sair devolve o editor normal.
- **Os elementos do quadro NÃO são entidades ECS.** Um quadro com 100 mil notas como 100 mil entidades
  encheria a Hierarquia, o undo do mundo e o laço de sistemas. Vivem num documento (`BoardDoc`) que é **um**
  componente do objecto Quadro. A Hierarquia mostra o Quadro; dentro dele, um painel de **Frames** próprio.

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
- Persistência: o `BoardDoc` viaja como **blob com versão própria** (o idioma do `TimelineDoc`) ⇒ **um**
  degrau de `PROJECT_SCHEMA` para todo o módulo (hoje `183`, `shells/desktop/src/project_schema.rs:392`;
  recontado na integração com `scripts/schema-recount.py`).

### §1.3 — As crates

| crate | o quê | depende de |
|---|---|---|
| `ph2d-board-model` | documento, ops, ids, índice fraccionário, serde | — (pura) |
| `ph2d-board-layout` | texto na forma/sticky (parley: quebra, centrado, crescer), layout de mind map (árvore arrumada), arrumar em grelha | `parley`, model |
| `ph2d-board-rough` | **traço à mão** (porta do algoritmo do rough.js, MIT) e **contorno do desenho livre** (porta do perfect-freehand, MIT), com o aviso de copyright; semente por elemento ⇒ determinístico | `kurbo` |
| `ph2d-vec-connect` | **reusado**: o roteador | — |
| `ph2d-board-render` | índice espacial, recorte ao ecrã, fragmentos Vello em cache por elemento (refaz só o sujo), nível de detalhe por zoom | `vello`, model, layout, rough |
| `ph2d-app-board` | a família: modo, ferramentas, gestos, painel, roteador de smokes `PH2D_BOARD_SMOKE` | `ph2d-app-host`, tudo acima |

A shell só **compõe** (`the_shell_only_shrinks`); o registo da família é gerado (`ph2d-app-sync` →
`ph2d-app-registry-init`).

### §1.4 — Undo

Cada gesto produz um lote de `BoardOp` com o inverso; entra na **fila única** do editor como uma entrada.
⚠️ **A medir na W0:** o custo do diff do `ProjectState` com um `BoardDoc` de 100 mil elementos. Se o diff do
blob inteiro passar de ~1 ms por gesto, o quadro entrega à fila o **seu** diff (as ops), não o blob.

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

---

## §3 — As ondas (cada uma fecha com gate batched, smoke e o que o dono vê)

Ordem pensada para o quadro ser **usável cedo**: depois da W3 já se faz um brainstorm.

### W0 — Fundação e réguas
- Crates vazias com o molde; `ObjectKind::Board` + o modo; `BoardDoc` gravado e carregado; undo de uma op.
- Cena de medida gerada (§2) + a régua A/B; **primeira medição** do custo de desenhar N rectângulos.
- Teste de seam (`ph2d-ui-testkit`): Add → Quadro → `Tab` entra no modo → a barra aparece.
- **Dono vê:** menu *Add → Quadro*, um quadro vazio com grelha de pontos, entrar e sair do modo.

### W1 — Tela, formas e texto
- Pan/zoom infinito (rato, trackpad, `Espaço`+arrastar), grelha de pontos, snap, guias de alinhamento.
- Seleccionar, mover, redimensionar, rodar, duplicar (`Ctrl+D`), apagar, multi-selecção.
- Formas: rectângulo (cantos redondos), elipse, losango, triângulo, e as de fluxograma da `ShapeKind`.
- **Texto dentro da forma:** duplo-clique escreve, quebra automática, centrado, a forma cresce.
- Estilo: preenchimento, contorno, tracejado, espessura, opacidade.
- **Oráculo:** contorno das formas com `roughness 0` contra o SVG do Excalidraw (geometria dos `d=`).
- **Kill-criterion do §2 decide-se aqui.**
- **Dono vê:** desenhar caixas, escrever dentro, arrumá-las.

### W2 — Setas (a peça em que já somos melhores — terminar de a fazer brilhar)
- Gesto: arrastar a ponta sobre uma forma **realça** o alvo e liga; `Ctrl` solta; linha livre.
- Âncora ao centro × ponto fixo (a regra do Miro, §4.1 do 01), recto/cotovelo/curvo, rótulo, pontas.
- **Pontos azuis**: um clique na borda cria a forma seguinte já ligada; `Ctrl+seta` cria pelo teclado.
- Cache de rota (§2); desvio de obstáculos (o roteador reusado).
- **Oráculo:** a normalização da ligação (`fixedPoint`, lado de saída) contra o `restored.json`; o desvio
  mede-se contra o NOSSO critério (nenhum segmento dentro de uma caixa), porque nele o oráculo falha.
- **Dono vê:** um fluxograma feito só com cliques e teclado, que se reorganiza ao mover caixas.

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
| elementos como entidades ECS | ⛔ recusado (§1.1): Hierarquia, undo e laço a crescer com cada nota |
| copiar o desvio de setas do Excalidraw | ⛔ medido pior que o nosso |
| ler o código do Excalidraw para portar o render | ⛔ §0.9 — corre-se. A porta do rough.js e do perfect-freehand é a **permissiva** (MIT, com aviso) e confere-se por passo contra o oráculo |
| fontes do Excalidraw além da Excalifont/Virgil | ⚠️ não triadas — nenhuma embarca sem triagem |
| rede na Etapa 1 | ⛔ decisão do dono (Etapa 2) |
| texto rico: parley tem estilo por trecho? | ⚠️ a medir na W3 antes de desenhar |
