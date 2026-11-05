# GEF 23 — A refundação: a fila, objeto por objeto (05/11/2026)

> Regras e protocolo: [`README.md`](README.md). Esta é a fila e o estado de cada item. Substitui a ordem de
> [`03_plano_jogo_de_plataforma.md`](03_plano_jogo_de_plataforma.md).

**Estados:** `a revisar` → `em conversa` → `aprovado` (ficha escrita) → `em obra` → `no smoke` → `fechado`.

**A ordem.** A Sprite é a primeira, por ordem do dono. **Depois dela, a ordem é uma proposta**: segue os lotes da crítica,
e o dono confirma o próximo item ao fechar cada um.

**Colunas «Crítica» e «Estudo»:** o nome exato da seção, para `ferramentas/estudo_secao.py`. `—` = o estudo não tem seção
própria. `pendente` = a crítica ainda não escreveu esse lote. Nesses casos, antes de propor, escreva na ficha a mesma análise
em 7 partes, ou peça ao dono que rode o lote (`fonte_critica/PROMPT_CONTINUACAO.md`, na pasta dos estudos).

## Primeiro: Sprite

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| **Sprite** (com Transform e Ordering dela, e a porta Add ▸ Sprite / Image…) | Sprite | Sprite · Transform | `a revisar` |

**Ponto de partida, a conferir** (recomendação da crítica, ainda **não aprovada** pelo dono):

- Versão B («cartões por assunto»), 10 campos à vista e 24 no total contra 37 hoje, com as miniaturas do seletor Use da versão C.
- Três trocas pequenas: convite no vazio, o nome de «Image…», apagar «key 4294967291» e «shader runtime pending».
- Três «vale rever?»: Still e a folha · Sprite, Image e Animated Sprite são o mesmo objeto · 9-Slice no seletor Use.

**Decisões vigentes a ler** (em `project-memory/`):

- `project_image_is_a_project_asset_reference_2026_10_26`
- `project_image_use_selector_under_image_2026_10_26`
- `project_isolation_by_use_and_atlas_strategy_stays_on_object_2026_10_26`
- `project_tiled_blocks_and_canvas_drop_always_creates_2026_10_26`
- `project_hierarchy_order_rules_drawing_and_click_2026_11_01`
- a regra de 27/10 sobre Still (comentário em `image_choice.rs`)

Também: `docs/Sprite_projeto/` e os docs `12`–`16` desta pasta.

## A casa (o que todo item usa)

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Animated Sprite | Animated Sprite | AnimatedSprite | `a revisar` |
| Add Object | Add Object | Objeto e Inspector | `a revisar` |
| Add Component | Add Component | Objeto e Inspector | `a revisar` |
| Inspector e Hierarchy | pendente | Objeto e Inspector | `a revisar` |
| Prefab | pendente | Prefab | `a revisar` |

## Corpo e movimento

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Physics Body | Physics Body | Corpo físico | `a revisar` |
| Static Body | Static Body | Corpo físico | `a revisar` |
| Collision Shape | Collision Shape | Colisor | `a revisar` |
| Character | Character | — | `a revisar` |
| Top-Down Player | Top-Down Player | — | `a revisar` |
| Projectile Motion | Projectile Motion | — | `a revisar` |
| Ray Sensor | Ray Sensor | — | `a revisar` |
| Motion Input | Motion Input | — | `a revisar` |
| Path Follow | Path Follow | — | `a revisar` |

## Câmera e fundo

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Camera e Game Camera | Camera e Game Camera | Câmera | `a revisar` |
| Camera Follow | Camera Follow | Câmera | `a revisar` |
| Camera Limits | Camera Limits | Câmera | `a revisar` |
| Camera Shake | Camera Shake | — | `a revisar` |
| Shake Emitter | Shake Emitter | — | `a revisar` |
| Background Layer | Background Layer | — | `a revisar` |
| Parallax, Limits, Drift, Repeat | Parallax e os outros três | — | `a revisar` |

## Regras do jogo

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Signal Actions | Signal Actions | — | `a revisar` |
| Trigger | Trigger | — | `a revisar` |
| Counter | Counter | — | `a revisar` |
| Counter Watch | Counter Watch | — | `a revisar` |
| Timers | Timers | — | `a revisar` |
| Lifetime, Destroy Outside | Lifetime e Destroy Outside | — | `a revisar` |
| Factory | Factory | — | `a revisar` |
| Weapon | Weapon | — | `a revisar` |
| Damage | Damage | — | `a revisar` |
| Health | Health | — | `a revisar` |
| Health Bar | Health Bar | — | `a revisar` |

## Comportamento

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Animation (clipes) | Animation | Animation Player | `a revisar` |
| State Machine | State Machine | Máquina de estados | `a revisar` |
| Tween | Tween | — | `a revisar` |
| Sequence Player | Sequence Player | — | `a revisar` |
| Script | Script | Script no painel | `a revisar` |
| Nav Agent | Nav Agent | — | `a revisar` |
| Nav Region | Nav Region | — | `a revisar` |
| Nav Cost Area | Nav Cost Area | — | `a revisar` |
| Nav Link | Nav Link | — | `a revisar` |

## Tela do jogo

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| HUD, HUD Canvas | HUD | Texto e UI | `a revisar` |
| Button | Button | Texto e UI | `a revisar` |
| Label | Label | Texto e UI | `a revisar` |
| Anchors | Anchors | — | `a revisar` |
| Anchor Mount | Anchor Mount | — | `a revisar` |
| Clip Children | Clip Children | — | `a revisar` |

## Aparência

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| 9-Slice | 9-Slice | — | `a revisar` |
| Blend Mode | Blend Mode | — | `a revisar` |
| Mask, Mask Interaction | Mask e Mask Interaction | — | `a revisar` |
| On-Screen Enabler | On-Screen Enabler | — | `a revisar` |
| Texture Filter | Texture Filter | — | `a revisar` |
| Texture Repeat, UV Transform | Texture Repeat e UV Transform | — | `a revisar` |
| Visibility Layer | Visibility Layer | — | `a revisar` |
| Particles | Particles | Partículas | `a revisar` |
| Tags | Tags | — | `a revisar` |
| Constraints | Constraints | — | `a revisar` |

## Som e luz (crítica: lote 9)

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Sound, Audio Source 2D, Audio Listener 2D | pendente | Áudio | `a revisar` |
| Light, Ambient Light, Light Occluder, Unlit | pendente | Luz | `a revisar` |

## Desenho e esqueleto (crítica: lote 10)

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Image | pendente | Sprite | `a revisar` |
| Vector Drawing | pendente | — | `a revisar` |
| Text | pendente | Texto e UI | `a revisar` |
| Flip Drawing | pendente | — | `a revisar` |
| Skeleton, Bone, Angle Limit, Smart Bone | pendente | — | `a revisar` |
| Motion | pendente | — | `a revisar` |
| Pixel Sprite | pendente | — | `a revisar` |
| Empty | pendente | — | `a revisar` |
| Macro | pendente | — | `a revisar` |

## Fora do escopo da crítica

| Item | Crítica | Estudo | Estado |
|---|---|---|---|
| Tilemap (não existe hoje; o projeto `docs/Tilling` tem o trabalho anterior) | — | Tilemap | `a revisar` |

## O que estava aberto em 04/11: para onde foi

| Aberto | Vai para |
|---|---|
| Placar «Coins: N» e os ajustes de HUD, Label e Text | HUD · Label · Counter · Text |
| Nome «Path 0» × «Text» | Add Object · Text |
| Som em gaveta do catálogo e remoção do projeto | Sound |
| Conta-gotas que só apanha sprites | Camera Follow |
| «Edit Animations…» na coluna do valor (o dono tinha aprovado à largura inteira) | Animated Sprite |
| A secção Animation aprovada antes do rebase e não alinhada ao visual novo | Animated Sprite |
| Inimigos (J6) e o resto do jogo (J7–J10) | saem da fila: as peças são revisadas nos seus itens |
| Velocidade do `LayerCompositor` | transversal: medir quando um item a expuser |

Os «testes de 1 minuto» e as medições da crítica entram no passo 2 do protocolo do item a que pertencem.

## O jogo de plataforma

Deixa de mandar na ordem. As cenas `PH2D_GAME_SMOKE` continuam existindo, com os seus testes, e podem servir de banco de
prova, mas cada item tem o **seu** smoke. ⚠️ Premissa desta reescrita, a confirmar com o dono: continuar ou não o jogo
como objetivo próprio.
