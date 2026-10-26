# 09 — Abaixar com corpo baixo (plano, 25/10)

Pedido: segurar «down» no chão agacha — o corpo baixa (passa sob tecto baixo; ao soltar só levanta se couber),
anda mais devagar (no cartão Platform Player, em unidades) e a pose «Crouching» entra nas condições das regras.
Antecessor: [08](08_regras_de_animacao_plano.md) · [handoff 08](handoffs/HANDOFF_CONTINUACAO_2026-10-25_08_REGRAS.md).

## Medido (25/10, antes de desenhar)

- **O agachar JÁ EXISTE** (W15, `ph2d-platformer/src/crouch.rs`): `CrouchConfig { height, speed, walk_off_ledges }`,
  `crouch_step` (quer = `down && no chão`; preso = tecto por cima), `effective_crouched` (perna e caminhada), sensor
  de tecto por varredura do corpo (`bridge/player_probes.rs::probe_headroom`, `sweep_body` com a forma REAL),
  `PlayerView::crouching`. No cartão Platform Player já há a secção **Crouch** com `Height (m)` e `Speed (m/s)`
  (`player_rows.rs::CROUCH_ROWS`). Gates W15/W18 em `ph2d-physics-ecs/tests/it/platform_crouch*.rs`.
- **Nasce DESLIGADO** (`CrouchConfig::STARTING_POINT.height = 0`), e o **Add ▸ Character não o liga**
  (`ph2d-app-game/src/object_add.rs`): no smoke 2, segurar ↓ hoje não faz nada ao corpo.
- **O agachar é uma PERNA mais curta, o collider nunca muda.** A perna só desce até a base da cápsula tocar o chão
  (W18, `measure_crouch_floor.rs`: abaixo do piso *«o slider é MORTO»* — 200 mm de curso, 1 mm de resposta).
  **O herói de fábrica** (cápsula `half_height 0,15 + radius 0,2`, `float_height 0,45`): piso `0,35` ⇒ o agachar
  da perna baixa no máximo **0,10 m** de `0,80` — 12 %. Não passa sob tecto baixo nenhum que interesse.
- `down`: estado (não borda) em `player_input.rs`; outros donos no ar (`down + jump` = atravessar) — o agachar só
  vale no chão, sem conflito.
- `ph2d-physics` não tem `set_shape`: o collider é construído no spawn (`collider_build.rs`) e o `rewind_to`
  reconstrói o mundo a partir do `BodyDesc` (o `b.rest` da ponte).
- **Pose:** `CharacterPose { Idle, Run, Jump, Fall }` gravada pelo índice (append-only); `ALL` serve as 4 regras
  derivadas (`AnimRules::of_character`), o menu (`ANIM_POSES = 4`, `POSE_KEYS`) e o espelho do cartão.
- **Arte:** a folha do herói tem `idle/run/jump/fall` — **nenhuma célula de agachar** (é o plano 06). O nível 2 não
  tem tecto baixo.

## Desenho

1. **Lei (`crouch.rs`, nova `body_shrink`):** a perna desce primeiro (lei W15 intocada); o que ela NÃO consegue
   (a altura autorada abaixo do piso da forma) a forma ENCOLHE POR CIMA, com a base no mesmo sítio:
   `s = clamp(base_abaixo_do_centro − height, 0, máximo_da_forma)` quando agachado e armado. Cura o defeito W18
   (o campo `Height` passa a responder em toda a faixa) sem campo novo ⇒ **sem degrau de `PROJECT_SCHEMA`**.
   Quem autorou `height ≥ piso` (todo o banco W15) fica bit a bit igual (`s = 0`). Máximo: cápsula/estádio = o
   troço recto (`2·half_height`; no fim vira bola), caixa = a própria base; bola/elipse = 0 (só a perna).
2. **Tecto:** nada muda — a varredura usa a forma ACTUAL (encolhida) e sobe `rise = float_height − height`, que é
   exactamente o que falta da cápsula de pé (perna + encolhido). Numa rampa pergunta a mais (conservador: nunca
   levanta para dentro da pedra).
3. **Física (`ph2d-physics`, `PhysicsWorld::reshape_body`):** troca a forma e a translação do collider PRINCIPAL
   (o 1.º do corpo) mantendo handle, camada, sensor e **a massa** (agachar não deixa o corpo mais leve); repor a forma
   de pé repõe a densidade/massa autorada (bit a bit). Idempotente: compara a translação actual com a pedida — por
   isso um `rewind_to` (que reconstrói de pé) cura-se no tique seguinte, sem estado novo na ponte.
4. **Ponte (`bridge/player.rs`):** depois do passo da lei, `s` do estado novo → forma encolhida a partir de
   `b.rest.shape`/`offset` → `reshape_body` (fora do laço de leitura).
5. **Pose `Crouch` (append-only):** `CharacterPose::Crouch` no fim; `ALL` = 5, `DERIVED` = as 4 de
   `CharacterAnimations` (as regras derivadas ficam iguais); `tag()` → `Option`. A pose base (Idle/Run/Jump/Fall)
   continua a mesma enquanto agacha; «Crouching» é um facto a mais (`Factos::agachado = PlayerView::crouching`),
   para a regra derivada «Standing still» continuar a valer agachado em ficheiros antigos. i18n «Crouching»;
   `ANIM_POSES = 5`; a opção só aparece com o agachar armado (`AnimCardInfo::can_crouch`).
6. **Fábrica e smoke:** Add ▸ Character arma o agachar com `height 0,05 m` (agachado ≈ metade da altura: topo
   `0,80 → 0,40`) e `speed 2 m/s` (de pé `6`). Nível 2 ganha um **tecto baixo** «Low Ceiling» junto à parede da
   esquerda (fundo a `0,55 m`: de pé não cabe, agachado cabe). Sem arte de agachar: o smoke usa uma tag existente.

**Corrigido na construção:** a «perna desce primeiro» deu um defeito medido — perna até a base da cápsula tocar o
chão + forma a encolher pelo resto ⇒ o corpo encosta no chão e o atrito prende-o (agachado o herói ficou em
x = 0,02 m em 300 tiques). Cura (lei do Unreal): **a FORMA encolhe primeiro** (por cima, base no sítio), a perna só
desce o que a forma não der (`body_room`, `body_drop = min(rise, body_room)`, `leg_drop = rise − body_drop`);
`body_room = 0` ⇒ lei W15 bit a bit.

## Gates

- Lei: `body_shrink` (zero acima do piso, cresce abaixo, tecto da forma, desarmado = 0).
- Física: `reshape_body` mantém a massa e repõe bit a bit; a varredura vê a forma nova.
- Ponte (banco W15 com a cápsula do herói): agachado passa sob um tecto que a perna sozinha não passaria; soltar
  sob o tecto NÃO levanta; soltar a céu aberto levanta (forma de pé de volta); a base não afunda; scrub cura-se.
- Regras: «Crouching» ganha enquanto agachado; soltar devolve; as 4 derivadas iguais (bit).
- Fábrica: o Character nasce agachável; o nível 2 tem o tecto e o boneco passa por baixo agachado.
- Seam: escolher «Crouching» no menu When grava `AnimWhen::Pose(Crouch)`; sem agachar armado a opção não aparece.

## Fora desta onda

Arte de agachar (plano 06) · rastejar com animação própria · agachar no ar.

**Feito (25/10):** abaixar com corpo baixo — a forma encolhe por cima, a perna fica com o resto, pose Crouching nas regras, Add ▸ Character agachável, «Low Ceiling» no nível 2 e smoke `PH2D_GAME_SMOKE=3`; ver [handoff](handoffs/HANDOFF_CONTINUACAO_2026-10-25_09_ABAIXAR.md); smoke do dono por fazer.
