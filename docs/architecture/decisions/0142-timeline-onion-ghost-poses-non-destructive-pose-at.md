# ADR-0142 — O onion da TIMELINE: poses-fantasma por `pose_at` não-destrutivo

- **Status:** aceito (provisório na `line/anim`; o número renumera na integração se colidir)
- **Data:** 2026-07-25
- **Linha:** `line/anim` (continuação do ADR-0141, motion path)
- **Contexto:** Enio — *"criar o Onion definitivo da timeline (acho que ainda não temos),
  baseado no que há de melhor no mundo, no estado da arte."*

## O problema

Uma timeline de keyframes anima **poses** (Transform de objetos ao longo do tempo). Para
autorar pose-a-pose o animador precisa VER onde o objeto estava e onde estará — os
*fantasmas* das poses vizinhas sobre a pose atual. Hoje a timeline **não tem nenhum**.

O que existe é o onion do **Flip** (`ph2d-flip::onion`, GP: passado verde / futuro azul,
opacidade, `frames_before/after`, modos Absolute/Selected) — mas ele é para **quadros
DESENHADOS à mão** (`FlipObject::frames`), um domínio diferente: frames discretos, não
poses contínuas keyframadas. A timeline dirige `Transform` via `apply_from_doc`; o Flip
composita camadas de pixels. **São dois domínios, um vocabulário visual.**

## A decisão

### 1. `pose_at` é NÃO-DESTRUTIVO e reusa os primitivos do apply

Um novo primitivo público em `ph2d-timeline`:

```rust
pub fn pose_at(world: &World, doc: &TimelineDoc, entity: u64, clip_t: f64) -> Option<Transform>
```

Ele parte do `Transform` VIVO da entidade (os campos que nenhuma track dirige ficam como
estão — exatamente o que o apply faz) e **sobrepõe** cada binding da entidade, amostrado
no relógio da entidade. É a MESMA composição que `apply_active_clip` faz — `remapped_time`
→ `track.sample` → a mesma atribuição de campo — mas escrevendo num `Transform` de
rascunho, **nunca no mundo**.

⚠️ **Não é uma 2ª derivação da pose** (a doença [[feedback_derived_coordinate_seed_must_match_sample]]
que este módulo pagou 3×). É a mesma aritmética por outro destino, e um **gate de
equivalência** prova `pose_at(e,t) == { apply em t; read Transform }` campo a campo — se
alguém tocar um dos dois lados, o gate sangra. A alternativa (mutar o mundo em t', ler,
restaurar) foi **rejeitada**: é frágil (qualquer leitura entre o apply-fantasma e o
restore vê a pose errada) e mutar o mundo vivo no meio de um frame é exatamente o tipo de
efeito colateral que este projeto evita.

### 2. Um fantasma é uma SILHUETA recolorida, injetada pelo slot `extra`

O pass de sprite já aceita `extra: &[RenderInstance]` (`present.rs` → `renderer_draw` →
`sprite_collect`; o Motion já o usa). Um fantasma é o `RenderInstance` do sprite vivo
(textura/uv/tamanho/anchor) com **`world_pos`/`basis` vindos de `pose_at(t')`** e
**`tint` = a cor do onion × alfa de falloff**. Recoloração 100% (GP), não um blend —
espelha `ph2d_flip::Ghost{tint, alpha}`.

### 3. O vocabulário VISUAL é compartilhado; o código NÃO (Chesterton)

O onion da timeline mora no shell (`render_loop/timeline_onion.rs`): ele lê o `doc`/`world`
do shell e constrói `RenderInstance` do shell. O onion do Flip mora na crate Flip. As duas
fontes de pose e os dois passes de render são diferentes; extrair uma crate `ph2d-onion`
por causa de dois structs de settings pequenos seria over-engineering **agora**. Decisão:
os DEFAULTS de cor do onion da timeline **espelham** `ph2d_flip::OnionSettings` (passado
verde, futuro azul) para o app ter UM vocabulário de fantasma, e uma unificação em crate
é follow-up **se** aparecer um 3º consumidor.

### 4. Modo, escopo, falloff (estado da arte)

- **Modo `Keys` (default) e `Frames`.** `Keys` = fantasma nas keyframes vizinhas (o modelo
  pose-a-pose do animador; Blender/Maya). `Frames` = `t ± k·frame` (mostra o espaçamento
  dos inbetweens). Os dois porque a timeline serve os dois fluxos.
- **Escopo: SELECIONADO** (como o motion path e o GP: *edita-se o que está na mão*), com
  "todos animados" como toggle futuro.
- **Falloff:** alfa cai com a distância (frames/keys) a partir de `opacity`, piso
  `GHOST_MIN_ALPHA` — a mesma lei do Flip.
- **Passado frio / futuro quente** pelos defaults do Flip.

## Ondas

- **W1 (esta):** `pose_at` + gate de equivalência · `timeline_onion.rs` no shell constrói
  os fantasmas do selecionado em modo **Frames** (`t ± k`) com tint+falloff, injetados no
  `extra` do pass de sprite · toggle (flag + env de smoke) · smoke. Gates: equivalência
  `pose_at`==apply · contagem/tint/alfa dos fantasmas · nenhum fantasma no tempo VIVO ·
  off = zero fantasma.
- **W2 (LANDOU):** modo **Keys** — `ph2d_timeline::entity_key_times` (a união deduplicada
  das keyframes da entidade, sem o Time Remap) + `OnionMode{Frames,Keys}`, default **Keys**
  (pose-a-pose). O onion ghosta as keyframes vizinhas em vez de `t±k`. Gates: caem NAS
  keys · o fps não move um fantasma de modo Keys (mutação → RED). ⚠️ Sob Time Remap a
  vizinhança-por-key é aproximada (a inversa `fonte→clip` não é única — a mesma razão do
  bake recusar instante ambíguo).
- **W3 (LANDOU, o toggle):** os toggles **Onion** (on/off) e **Onion Keys** (Keys/Frames)
  na barra de transporte + i18n. `OnionSettings`/`OnionMode` mudaram-se para
  **`ph2d-timeline`** (dados puros) para o `TimelineState`, o `apply_intent`, o snapshot e o
  painel falarem UMA língua — o espelho EXATO do `SetSimulatePhysics`: o onion vive em
  `TimelineState::onion` (não serializado, fora de `TimelineFlags` porque tem `f32`), o
  painel edita por `TimelineIntent::SetOnion`, o snapshot o carrega, e o shell LÊ
  `self.timeline.onion` para o passe de fantasmas. Gates: seam que CLICA os dois toggles →
  `SetOnion` (mutação: fora do `populate` = morto sob o mouse) · `apply_intent`→estado→
  snapshot · id-collision.
- **W3b (LANDOU, o card):** contagens/opacidade/cores **não cabem na barra** (Enio:
  *"esse card deve viver num modal … mas o botão para abrí-lo fica na timeline e todas as
  edições devem ser vistas em tempo real no canvas"* · *"o modal deve ser arrastável com o
  mouse e deve ter o botão de fechar funcional"*). Card flutuante **arrastável** com **X**,
  aberto por uma **engrenagem** na barra de transporte, editando ao vivo (Opacity · Ghosts
  Before/After · Past/Future colour). **É hero chrome, espelho EXATO do `fill_modal`** — o
  shell não roteia `WidgetEvent`s da própria chrome (registra hit-rect mas não tem dispatch
  interativo; isso mora no `chrome::dispatch_all` do editor-core). ⚠️ **A `WidgetStore` é o
  quadro-negro compartilhado:** o editor-core NÃO enxerga `OnionSettings`, então os widgets
  vivem no store (dirigidos pelo dispatch genérico) e o **shell lê de volta** para
  `self.timeline.onion` a cada frame (o passe de fantasmas relê ⇒ tempo real de graça);
  `onion_modal::apply` **não encaminha nada** (não há tool; nenhum `EditorAction` nomeia
  onion), só fecha no X e consome o handle/sliders. **O mapeamento contagem↔slider vive SÓ
  no shell** (`crate::onion_modal`, uma cópia): o open-seed e o read-back o usam. O botão é
  BOTÃO (não toggle): seu `PanelEvent::Click` chega ao shell, que abre o card seeded de
  `self.timeline.onion` (shell-side porque o card mora no `hero.store`, fora do alcance do
  painel — espelho do `TIMELINE_MOTION_PATH`). A **cor** vai pelo `register_picker_swatch`
  (o `BlenderColorPicker` compartilhado; read-back por `widget_color`). O **drag** é uma
  máquina de estado no shell (`ONION_MODAL_DRAG`), byte-a-byte o `fill_drag`. As contagens
  são **sliders** (`0..MAX_GHOSTS=8`, arredondadas na leitura) — o label é estático (como o
  "Threshold" do `fill_modal`) e o preenchimento comunica o nível; mostrar o inteiro é
  refinamento (exigiria plumbing do `MAX` até o painter). Gates: paint gated em
  `onion_modal_pos()` (mutação: pinta fechado) · X fecha · handle consome sem fechar ·
  slider consome · open semeia + swatches marcadas + move desloca · a **engrenagem** pinta,
  CLICA e encaminha `PanelEvent::Click` (mutação: fora do `populate` = morto sob o mouse) ·
  o **read-back** escreve o onion enquanto aberto e é no-op fechado (mutação: `read_into`
  no-op) · contagem/cor round-trip. Smoke: `PH2D_ONION_SMOKE=1`/`=2` abrem o card semeado.
  **Refinamento aberto:** mostrar o inteiro da contagem no label; e o card não persiste
  posição (reabre no canto — deliberado, o artista arrasta).
- **Futuro (Chesterton):** rigs parenteados (compor a cadeia como a física W5 fez) ·
  "todos animados" · unificação de crate com o Flip.

## Consequências

- Foundational tocado, **aditivo**: `ph2d-timeline` ganha `pose_at` (função nova, nada
  muda de contrato; `DOC_VERSION`/`PROJECT_SCHEMA` intactos — o onion é vista, não
  documento).
- O shell ganha um passe de fantasmas que **concatena** ao `extra` do Motion (os dois
  raramente coexistem; um `Vec` de rascunho os une).
- Zero regressão quando desligado (default off): sem fantasmas, `extra` fica como está.
