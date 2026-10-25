# 07 — A animação num cartão só (plano, 25/10)

Ordem do dono, depois do smoke do N1: *«apenas para configurar animação temos vários componentes e seções no
inspector: Sprite Sheet, Platform Player (Auto Animate) e Animation — confuso demais».* Contexto:
[`05_N1_plano.md`](05_N1_plano.md) · [handoff](handoffs/HANDOFF_CONTINUACAO_2026-10-25_N1_ARTE.md).

## Como os outros fazem (de memória, NÃO reverificado na web)

| App | Importar a folha | Onde se edita | Troca de animação |
|---|---|---|---|
| Construct 3 | strip cols×rows | editor de Animations próprio: lista à esquerda, faixa de quadros em baixo, velocidade, loop / ping-pong / repeat | eventos («On jumped → set animation») |
| Godot | `AnimatedSprite2D` + `SpriteFrames` | painel inferior; «Add frames from sprite sheet» (grelha, clicar células por ordem); FPS + loop | script ou AnimationTree |
| Unity | Sprite Editor, fatia por grelha | arrastar quadros cria clip + Animator; grafo de estados (onde o iniciante se perde) | Animator |
| Unreal Paper2D | Extract Sprites por grelha → Flipbook | editor de Flipbook | Blueprint |

Padrão: **todos editam fora do inspector, num editor que mostra os quadros; todos usam quadros/segundo;
nenhum liga estado do personagem → animação sozinho** (o nosso diferencial).

## Desenho aprovado («siga»)

1. **Inspector: UM cartão compacto «Animation»** — pré-visualização ao vivo; lista de animações com miniatura e
   ▶; botão «Editar animações…»; secção «Quem escolhe a animação»: *Eu (manual)* · *Script* · *O personagem
   (automático)*. No automático, Parado/Correr/Saltar/Cair são DROPDOWNS das tags da lista (nunca texto livre;
   renomear uma tag tem de acompanhar), mais «correr a partir de» e «virar ao andar para trás».
2. **Editor de animações em área própria:** lista à esquerda; a folha com a grelha ao meio — arrastar ou
   clicar células por ordem adiciona quadros; faixa de quadros em baixo (arrastar reordena, tempo por
   quadro); em cima velocidade em **quadros por segundo**, Repete / Uma vez / Vai-e-volta, ▶.
3. **Largar uma folha nova** propõe cols×rows detectando calhas transparentes; o utilizador confirma.
4. **O cartão Platform Player fica só com movimento.** A secção «Sprite Sheet» deixa de existir à parte (a
   grelha H/V passa para o cartão/editor Animation; centrado/offset/flip ficam no Sprite).
5. **Ficheiros gravados continuam a abrir:** muda ONDE se edita, não o dado (`SpriteGrid`, `SpriteAnimations`,
   `SpriteAnimator`, `CharacterAnimations` ficam; `per_frame_ms` ↔ fps é só apresentação).

## MEDIR primeiro (CLAUDE.md §5.0 — confirme o código antes de crer numa ausência)

- Onde nascem hoje os cartões de `SpriteGrid` / `SpriteAnimations` / `CharacterAnimations`: catálogo
  (`FieldDesc` em `crates/ph2d-component-desc/src/catalog/*.rs`) ou secções à mão do Sprite Inspector
  («Sprite Sheet»/«Animation»: `crates/ph2d-editor-core/src/screens/hero/`,
  `shells/desktop/src/render_loop/inspector_anim.rs`).
- Se já existe painel de editor de animação (projeto Sprite Inspector: `docs/Sprite_projeto/`) — reutilize.
- Gates de costura de cada controlo novo (DIRETIVA_IMPLEMENTACAO §2, `ph2d-ui-testkit`); i18n em todo
  rótulo (HR-15); UI em tokens.

## Gates a planear

- O dropdown de estados lista exatamente as tags e acompanha um renomear.
- Round-trip fps ↔ ms.
- A detecção de grelha na folha de fábrica do herói dá **8×4**.
- O cartão Platform Player já não mostra Auto Animate.
- Projectos antigos abrem idênticos (bit a bit no documento).

## Smoke do dono (a escrever em passos numerados no fecho)

Largar/escolher o herói · «Editar animações…» · criar uma animação clicando células · pô-la em «Correr» ·
Play. Fotografar antes de mandar.

## Medido (25/10, antes de desenhar)

- **Os três nascem à mão, em três sítios:** «Sprite Sheet» = `ph2d-panel-inspector/src/sections/sprite_sheet.rs:46`
  (chamado de `paint_frame_shared.rs:237`); «Animation» (§11) = `sections/anim.rs:329` + `anim_rows.rs`
  (snapshot `shells/desktop/src/render_loop/inspector_anim.rs:123`, aplicação `:195`); «Auto Animate» =
  `sections/player_anim.rs:65` dentro do Platform Player (snapshot e aplicação em
  `ph2d-app-physics/src/inspector/player_anim.rs`). Nenhum é cartão genérico do catálogo.
- **Não existe editor de animação** (nem grelha clicável, nem faixa de quadros, nem detecção de calhas).
  Área própria = `DocumentSurface` (`ph2d-editor-core/src/doc_surface.rs`), o padrão do CodeEditor
  (`init.rs:373`). Menu com lista viva = o `Dropdown` da Sequence (`event_sequence.rs`, `popovers.rs`).
- ⛔ **Premissa derrubada — «o dado não muda»:** uma `AnimationTag` é um INTERVALO contínuo (`from..=to`) e o
  tocador anda célula a célula (`sprite_anim.rs::advance`, um só chamador: `sprite_anim_tick.rs`). Clicar
  células por ordem e arrastar para reordenar exige uma LISTA de quadros. O blob é postcard POSICIONAL ⇒ um
  campo novo na tag subiria o `PROJECT_SCHEMA` e recusaria todo projeto gravado. ⇒ componente IRMÃO
  (o precedente `VecPatternRotation`/`VecMorphMachine`): ausente = intervalo de sempre, ficheiro antigo
  abre bit a bit igual.
- ⛔ **Premissa derrubada — «a folha do herói tem calhas»:** as células da fábrica estão JUNTAS
  (`fabrica.rs:120`). A detecção usa colunas/linhas transparentes ENTRE as figuras (o boneco ocupa o meio da
  célula), não calhas desenhadas.

## Desenho técnico

1. **Dados (`ph2d-ecs`):** `SpriteAnimFrames` (por tag, pelo NOME: a lista de células na ordem tocada) e
   `SpriteAnimCursor` (o PASSO da lista que está a tocar — uma lista pode repetir células, então o passo não se
   deriva da célula). O motor passa a andar em PASSOS (`0..n`); um intervalo é a lista `lo..=hi` e o passo é
   `frame - lo` — comportamento idêntico, guardado pelos testes de `sprite_anim_tests.rs`. `per_frame_ms`
   já era indexado pelo passo.
2. **«Quem escolhe» sem dado novo:** Eu = `SpriteAnimator.autoplay` ligado (toca a escolhida ao começar) ·
   Script = autoplay desligado (o script escolhe) · O personagem = `CharacterAnimations` anexado (só se há
   Platform Player). Ficheiros antigos caem num dos três sem migração.
3. **Modo do ciclo:** Repete = `repeat None` + `Forward` · Uma vez = `repeat Some(1)` + `Forward` ·
   Vai-e-volta = `PingPong` (uma só edição, um só passo de undo). Quadros/s = `1000 / frame_ms` (apresentação).
4. **Renomear acompanha:** o renomear de uma tag reescreve o tocador, a lista de quadros e os quatro estados
   do personagem no MESMO passo.
5. **Detecção de grelha:** função pura sobre RGBA (`ph2d-sprite-sheet`): perfis de ocupação por coluna/linha,
   o maior divisor cujas fronteiras caem todas em transparência e que não parte figura nenhuma. Gate: herói = 8×4.
