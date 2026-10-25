# N1 — os objectos do jogo de plataforma no menu Add (plano, 25/10)

> Onda seguinte a R0+N0 ([04](04_R0_N0_plano.md), smoke do dono OK 25/10), passo 3 do
> [plano 03](03_plano_jogo_de_plataforma.md). Smoke = **J2 + J3**: um nível montado SÓ com os objectos novos do
> menu Add — o personagem anda, salta e cai sobre chão e plataformas, e a animação troca entre parado, correr,
> saltar e cair pelo estado dele. Jogado por *Run ▸ Play in a Window*.

## O que JÁ existe (medido antes de desenhar, §5.0)

| Peça | Onde | O que dá / o que falta |
|---|---|---|
| Menu Add | `ph2d-editor-core/src/object_add.rs` (`AddEntry`, `build`) · famílias em `shells/desktop/src/render_loop/fase_object_add.rs:13` | o modal é a `command_palette`; o grupo *Game* tem Camera, Physics Body, Sound, HUD (`ph2d-app-components/src/object_add.rs`) — cada um = vazio + UM componente por `attach_by_name` |
| A porta de anexar | `component_attach::attach_by_name` (`:209`) | cascata dos `requires` + semente; no-op sobre o que já está lá. `Sprite` **não** é anexável (sem `Default`) |
| Sementes | `ph2d-app-physics/src/physics_seed.rs:139` | `Collider` neutro ⇒ `Cuboid` com metade do `Sprite.size`; `PlatformPlayer` mede o `Collider` para pairar |
| Personagem | `PlatformPlayer` (`ph2d-physics-ecs/src/components/player.rs:58`) · `common::spawn_player` | o controlador completo (andar, saltar, coyote, buffer…). Precisa de `RigidBody` (Dynamic) + `Collider` + `LockRotation` |
| Estado do personagem | `PhysicsBridge::player_view` → `PlayerView` (`ph2d-platformer/src/contract.rs:190`) | `footing` (ar/rampa/chão), `velocity`, `ground_velocity`, `facing` — **nenhum consumidor o liga à animação**; o Luau não o vê |
| Animação por intervalos | `SpriteGrid` + `SpriteAnimations` (tags `from..=to`, `frame_ms`) + `SpriteAnimator { current }` | troca-se escrevendo `current`. Espelho: `Sprite.flip_x` |
| Peças de colisão | `ph2d-physics-ecs/src/bridge/parts.rs` | um filho com `Collider` e **sem** `RigidBody` é uma forma do corpo ancestral — é o `CollisionShape2D` da Godot, já pronto |
| Imagens de fábrica | `ph2d-render/src/atlas/mod.rs` (`WHITE_TILE_KEY`, `DISC_TILE_KEY`, `insert_reserved_tiles`) | chaves reservadas no topo, registadas antes de qualquer sprite; gravam-se pela chave |
| Dica (tooltip) | `WidgetStore::set_tooltip` + `paint_hover_tooltip` | **uma linha de texto**, pintada ANTES da paleta (fica por baixo do modal). Sem imagem, sem dica nos itens da paleta |
| Tipo do objecto | `ObjectKind` (7, por marcador) · Hierarquia mostra um ícone POR FAMÍLIA de componente | um personagem lê-se *Image* (tem `Sprite`) e mostra os ícones de Física e Imagem — chega; **não** se acrescenta `ObjectKind` (mexeria nos filtros da paleta e nos modos) |
| Inspector | cartões gerados do `FieldDesc` do catálogo (`ph2d-component-desc/src/catalog/*.rs`) | um componente novo com `FieldDesc` ganha cartão sem pintura à mão |
| Desenhos | `assets/sprites/*.png` | quadrados de cor, não versionados — **não há** folha de personagem no repo |

## O desenho

**1. Os cinco objectos** — família do jogo, `crates/ph2d-app-game/src/object_add.rs` (entra em `FAMILIES`),
grupo *Game*. Cada um nasce pela porta do `spawn_empty_root`; o que tem `Default` entra por `attach_by_name`
(cascata + sementes), a `Sprite` e os valores próprios do papel entram antes, à mão:

| Objecto | Componentes |
|---|---|
| **Sprite** | `Sprite` (quadrado branco 1×1 m) |
| **Animated Sprite** | `Sprite` (o boneco de fábrica) + `SpriteGrid` + `SpriteAnimations` (idle·run·jump·fall) + `SpriteAnimator` (toca `idle`) |
| **Static Body** | `Sprite` (bloco 3×0,5 m) + `RigidBody { Static }` + `Collider` (a semente copia o tamanho da sprite) |
| **Character** | o Animated Sprite + `Collider` cápsula + `LockRotation` + `RigidBody` + `PlatformPlayer` (semente: altura de pairar) + **`CharacterAnimations`** |
| **Collision Shape** | `Collider` sozinho (caixa 1×1): posto DENTRO de um corpo é uma forma dele |

**2. O boneco de fábrica** — `HERO_SHEET_KEY = u32::MAX - 2`, a 3.ª imagem reservada do atlas: uma folha
procedimental (função pura `hero_sheet_pixels`, determinística, sem ficheiro binário) de 4 linhas × 6 células
— parado (respira), correr (pernas em ciclo), saltar, cair — com margem transparente por célula (o filtro
bilinear não sangra da vizinha).

**3. A ligação estado → animação** — componente novo `CharacterAnimations` (em `ph2d-physics-ecs`, ao lado do
`PlatformPlayer`; gravado ⇒ PROJECT_SCHEMA +1, recontado): `idle`, `run`, `jump`, `fall` (nomes das tags),
`run_from` (m/s relativos ao chão) e `flip` (espelha pelo `facing`). A lei é uma função pura
`estado(view, run_from) → Idle|Run|Jump|Fall` (chão: corre se |vx − vx_chão| ≥ `run_from`; ar: sobe = salta,
desce = cai); a ponte (`ph2d-app-game`) corre depois do passo da física e escreve `SpriteAnimator.current`
(só quando muda — reinício da tag) e `Sprite.flip_x` no próprio objecto ou no 1.º filho com animador. Um nome
que a lista não tem: o animador fica em `NoTag` e o Inspector já o diz.

**4. A dica ilustrada** (Fusion 360) — `ph2d-editor-core`:
- `AddEntry` ganha `tip: Option<AddTip { body: TextKey, picture: &'static [Mark] }>` (`AddEntry::with_tip`);
  o `build` passa-as ao modelo (`PaletteModel.tips`, por id de item — campo novo é erro de compilação nos
  construtores, de propósito).
- `Mark` = um vocabulário mínimo de desenho em DADOS (rect, disco, linha/seta, contorno tracejado), em
  coordenadas da moldura (0..1) e cores por `ColorToken` — testável, sem imagens, segue o tema.
- A paleta pinta, por cima dos cartões, o cartão da dica do item sob o rato: título (o nome simples), a
  ilustração, a explicação completa em várias linhas; ao lado do item, virando para o lado que cabe.
- Recebem dica os cinco novos **e** os quatro do grupo *Game* que já existiam (Camera, Physics Body, Sound,
  HUD) — o mesmo grupo não pode ter metade com explicação. Nunca o nome da Godot (decisão 1).

**5. O smoke cresce — `PH2D_GAME_SMOKE=2`** (`crates/ph2d-app-game/src/smoke.rs`): o nível 2 monta-se SÓ por
`object_add::add` (a mesma porta do menu) e depois põe posição/tamanho: chão e plataformas = *Static Body*,
uma parede invisível = *Collision Shape* dentro do chão, o jogador = *Character*, uma tocha/moeda animada =
*Animated Sprite*, uma placa = *Sprite*; a câmara de jogo do nível 1. O condutor ganha o horário do nível 2
(correr, saltar, cair) para as fotos.

## Gates

- **A porta é a do menu:** o nível 2 só cria entidades por `object_add::add` (teste conta as entradas usadas).
- **Cada objecto tem o conjunto fixo:** por entrada, os componentes presentes depois do `add` (registo real,
  sementes reais); mutação: tirar um ⇒ vermelho.
- **J2:** o Character do nível 2 anda (x cresce com a seta), salta e aterra numa plataforma, a física do app
  (`PhysicsBridge::dispatch`) a correr como no smoke 1.
- **J3:** a lei pura (4 estados, chão móvel incluído); na cena, a sequência parado → correr → saltar → cair →
  parado aparece em `SpriteAnimator.current` ao longo do horário; `flip_x` segue o `facing`.
- **A dica:** seam test `ph2d-ui-testkit` — rato sobre o item ⇒ o cartão pinta título + texto + ilustração
  (hit-rect e caixa dentro do viewport); item sem dica ⇒ nada. Toda entrada do grupo *Game* tem dica.
- **Imagem de fábrica:** a folha é determinística (hash) e cada célula tem margem transparente.

## ⛔ Recusas

| Recusado | Porquê |
|---|---|
| `ObjectKind::Character/Body` | os `ObjectKinds` filtram a paleta de componentes e decidem os modos; um personagem é uma imagem que se move, e a Hierarquia já mostra o que ele FAZ pelos ícones das famílias |
| Convenção de nomes (tags `idle`/`run` mudas, sem componente) | uma regra invisível: quem chamar à tag `walk` não vê nada nem sabe porquê |
| Folha do personagem em PNG versionado | precisa de registo de asset por projecto; a imagem reservada procedimental grava-se pela chave e é testável |
| Dica com imagem em pixels (PNG por item) | precisaria de texturas no chrome; o desenho em dados segue o tema e o HR-15 |
| Repetir a árvore da Godot (Node2D → CollisionObject2D → …) no menu | spec/06 §5 e plano 03 |
