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
- **W2:** modo **Keys** (a união das keyframes vizinhas das tracks da entidade).
- **W3:** a UI no painel da timeline (modo, contagens, opacidade, cores, escopo) + tokens
  + i18n. Estado de VISTA, **não serializado** (a classe do toggle Physics/Speed graph —
  a resposta a *"o que a tela mostra"* não deve mudar sozinha após um load).
- **Futuro (Chesterton):** rigs parenteados (compor a cadeia como a física W5 fez) ·
  "todos animados" · unificação de crate com o Flip.

## Consequências

- Foundational tocado, **aditivo**: `ph2d-timeline` ganha `pose_at` (função nova, nada
  muda de contrato; `DOC_VERSION`/`PROJECT_SCHEMA` intactos — o onion é vista, não
  documento).
- O shell ganha um passe de fantasmas que **concatena** ao `extra` do Motion (os dois
  raramente coexistem; um `Vec` de rascunho os une).
- Zero regressão quando desligado (default off): sem fantasmas, `extra` fica como está.
