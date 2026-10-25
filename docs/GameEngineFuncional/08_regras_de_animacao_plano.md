# 08 — Regras de animação no cartão Animation (plano, 25/10)

Decisão do dono, 25/10 (memória `project_animation_one_card_owner_decision_2026_10_25`, decisão 2): os 4 estados
fixos de «The Character», mais o resto feito por script, são o MESMO conceito. Aprovado: no cartão Animation, uma
LISTA DE REGRAS «Quando ___ → toca ___», com os dois lados em MENUS. Escolher «The Character» pré-preenche as 4 regras
de hoje; «+ Rule» acrescenta; a regra de cima ganha; arrastar reordena; o script continua para os casos difíceis.
Antecessor: [`07_animacao_num_cartao_plano.md`](07_animacao_num_cartao_plano.md) ·
[handoff 07](handoffs/HANDOFF_CONTINUACAO_2026-10-25_07_ANIMACAO.md).

## Medido (25/10, antes de desenhar)

- **`CharacterAnimations`** (`ph2d-physics-ecs/src/components/character_anim.rs:15`): 4 nomes de tag + `run_from` +
  `flip`, postcard POSICIONAL. `CharacterPose::of(view, run_from)` (`:57`) decide Idle/Run/Jump/Fall a partir de
  `footing`, `velocity`, `ground_velocity`. **Um leitor por quadro:** `ph2d-app-game/src/personagem.rs:33`
  (`anima_personagens`): pose → tag → `troca` no `SpriteAnimator` (próprio ou do 1.º filho), e escreve o flip.
  Não lê teclas nem sinais. Registos: `ph2d-physics-ecs/src/lib.rs:223`, catálogo `ph2d-component-desc/.../physics.rs:199`,
  `ph2d-app-macros/src/mundo/portas.rs:156`, `ph2d-app-components/src/property_laws.rs:115`.
- **Teclas:** `ph2d-input`. As ações são NOMES (`"move_left"`, `"jump"`, `"down"`, …, `map.rs:12-33`), o mapa é por
  herói (`hero.input_map`). Consulta `ActionState::pressed(name)` (`resolve.rs:142`); o Platform Player resolve-o
  numa porta só (`ph2d-app-physics/src/player_input.rs:70`).
- **Sinais:** `ph2d-runtime`. O contrato é o NOME (`Signal::is`), com origem (`SignalOrigin`) e emissor; publicar
  `SignalOutbox::publish`, ler com um `SignalReader` próprio (cursor por consumidor). Script: `ph2d.emit`
  (`ph2d-script/src/api_table.rs:193`).
- **Dano:** não há um sinal «dano» com nome fixo. O facto existe: `HealthEvent { target, source, kind }` com
  `HealthEventKind::{Damaged, Shielded, Dodged, Died, Healed…}` (`ph2d-physics-ecs/src/bridge/health.rs:76,118`).
  ⇒ «levou dano» é um facto do PRÓPRIO objeto, lido do `HealthEvent` com `target == self`, sem nome a escrever.
- **Cartão:** os 4 menus de estado são `Dropdown`s (`INSP_ANIM_STATE_PICK: [NodeId;4]`,
  `ids/inspector_anim.rs:294`; pintura `anim_card.rs:330-382`; evento `event_anim_card.rs:113`; aplicação
  `ph2d-app-sprite-anim/src/card.rs:174-224`, por `queue_set` = undo por diff). Tectos: ficheiro de painel ≤ 600
  linhas (`anim_card.rs` tem 436), função ≤ 200 (`apply_anim_event` tem 221 linhas com comentário — conferir antes).
- **Molde de componente irmão** (sem degrau de schema): `SpriteAnimFrames` em `ph2d-ecs` — 5 registos
  (`ph2d-ecs` lib + registry + contador do teste, `ph2d-render` e `ph2d-script` registry).

## Desenho

1. **Dado: componente IRMÃO `AnimRules`** em `ph2d-physics-ecs`, ao lado de `CharacterAnimations` (é quem conhece a
   pose): `Vec<AnimRule { when: AnimWhen, play: String }>`. `AnimWhen` (enum fechado, append-only):
   - `Pose(Idle | Run | Jump | Fall)`: o que `CharacterPose` já sabe;
   - `Action(String)`: uma ação do mapa de teclas premida (o menu lista as ações do herói: «Down», «Jump», …);
   - `Event(Damaged | Died | Healed | Dodged)`: um facto de vida DESTE objeto;
   - `Signal(String)`: um sinal com esse nome ouvido neste quadro (o menu lista os nomes que o projeto já autorou:
     sinais das animações, dos contactos, `ph2d.emit` literais; mais «New signal…»).
   O **lado «toca»** é o nome da tag (menu das tags; renomear acompanha, como os 4 estados de hoje).
2. **Ficheiro antigo abre igual:** `CharacterAnimations` presente e `AnimRules` ausente ⇒ as 4 regras DERIVADAS
   (`idle/run/jump/fall` dos campos), sem migrar nada. A 1.ª edição da lista grava `AnimRules`, que passa a mandar;
   `run_from`/`flip` continuam no `CharacterAnimations`. Sem degrau de `PROJECT_SCHEMA`; contadores de registo +1
   (o integrador CONTA).
3. **Avaliação (uma porta, `personagem.rs`):** a cada quadro, a 1.ª regra cuja condição vale escolhe a tag. Facto
   instantâneo (evento, sinal) **segura** a sua animação até ela acabar um ciclo (senão «levou dano» duraria um
   quadro); enquanto segura, só uma regra ACIMA a pode interromper. Nenhuma regra vale ⇒ mantém a atual.
4. **Sem Platform Player**, as regras continuam úteis (tecla, evento, sinal); as condições de pose só aparecem no
   menu com Platform Player. «Who Picks» passa a «Me · Script · Rules»; escolher «Rules» num personagem pré-preenche
   as 4.
5. **UI no cartão:** cada regra é uma linha `[When ▾] → [Play ▾] ×` com pega de arrasto à esquerda; «+ Rule» no fim.
   Menus = o `Dropdown` que já serve os 4 estados (ids em tabela, como `INSP_ANIM_STATE_OPT`). Reordenar por arrasto:
   a lei única do marcador/largada já escrita na faixa do editor (`drop_gap`) — copiá-la como função partilhada, não
   como segunda conta. Tokens e i18n em tudo.

## Gates a planear

- Ficheiro antigo (`CharacterAnimations` sem `AnimRules`) avalia exactamente as 4 tags de hoje, quadro a quadro, no
  nível 2 (o mesmo bit que o `anima_personagens` de agora).
- A de cima ganha; reordenar muda o vencedor; «levou dano» segura a animação até ao fim do ciclo.
- Ação premida (`"down"`) escolhe a sua tag; soltá-la devolve à regra seguinte.
- Renomear uma tag reescreve as regras no mesmo passo de undo.
- Seam (`ph2d-ui-testkit`) por controlo: abrir/escolher cada menu, «+ Rule», ×, arrastar; populate/paint/evento.
- Projectos antigos abrem idênticos (bit a bit no documento).

## Smoke do dono (no fecho, em passos numerados)

`PH2D_GAME_SMOKE=2`: Player ▸ Animation ▸ Who Picks ▸ Rules (aparecem as 4) · «+ Rule»: Quando «Down» premida → toca
«anim_0» · arrastá-la para cima · Play in a Window, segurar ↓. Fotografar antes de mandar.

## Fora desta onda

«Abaixar» com corpo baixo (peça à parte) · condições compostas (E/OU) — o script cobre · transições com mistura.
