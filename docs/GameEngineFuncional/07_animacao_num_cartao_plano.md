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
