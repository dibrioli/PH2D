# TOP-20 #14 — `ProjectileMotion`: o plano

> **Fila:** [levantamento §7](00_levantamento_componentes.md) item **14**, *«o mover arcade genérico
> (avança + gravidade + ricochete + alcance + homing)»*. Os #1–#13 estão fechados.
> **Entrega declarada:** [síntese de movimento §ProjectileMotion](pesquisa/sintese_movimento_fisica.md).
> **Protocolo:** `/pd-feature` — plano antes de código, oráculo CORRIDO, números MEDIDOS.

---

## §0 — O que o artista consegue FAZER quando isto fechar

Põe um objecto na cena, carrega em **+ Add Component → Projectile Motion**, e:

1. ele **avança sozinho** para onde está virado, com velocidade inicial e aceleração;
2. liga **Gravity** e ele faz o **arco** de uma pedra atirada;
3. bate numa parede e **ricocheteia pela normal**, com um tecto de saltos e perda por salto;
4. depois de **Range** metros percorridos ele **morre** — sem um `Lifetime` a adivinhar o tempo;
5. liga **Face Velocity** e a flecha **aponta para onde voa**;
6. dá-lhe um **alvo** e ele **persegue** (o *homing* embutido, o modelo do Unreal).

---

## §1 — ⭐⭐⭐ A PERGUNTA DE ANTES: a composição já o exprime? (§5.0)

**MEDIDO** por [`mede_o_que_a_composicao_ja_da.rs`](../../crates/ph2d-physics-ecs/tests/it/mede_o_que_a_composicao_ja_da.rs)
(sonda versionada, `--ignored --nocapture`), sobre um corpo **dinâmico** com `restitution = 1` dos
dois lados, `GravityScale(0)` e `InitialVelocity`:

### §1.1 — ⛔ O RICOCHETE **JÁ É EXACTO**, e isso derruba metade da entrega declarada

| incidência | rapidez antes | depois | razão | `vx` medido | `vx` do espelho |
|---|---|---|---|---|---|
| 90° | 12,000 | 12,000 | **1,000** | −12,000 | −12,000 |
| 75° | 12,000 | 12,000 | **1,000** | −11,591 | −11,591 |
| 60° | 12,000 | 12,000 | **1,000** | −10,392 | −10,392 |
| 45° | 12,000 | 12,000 | **1,000** | −8,485 | −8,485 |
| 30° | 12,000 | 12,000 | **1,000** | −6,000 | −6,000 |
| 15° | 12,000 | 12,000 | **1,000** | −3,106 | −3,106 |

⇒ *a rapidez sobrevive ao toque e a direcção é o espelho, ao terceiro decimal, em todos os ângulos.*
⛔ **A síntese vende o «ricochete em sólido pela normal» como entrega, e a casa já o tem.** Construir
uma lei de ricochete nova seria um **segundo motor** para o que o solver calcula exactamente.

⭐⭐ **O que a medição de facto compra é a CONFIANÇA NA LEI:** o espelho `v − 2(v·n)n` é o que a
`rapier` produz, logo é ele que o componente deve escrever — não uma variante.

### §1.2 — ⭐⭐⭐ E o que ela acusa é **POR QUE o componente tem de existir**

| tiro idêntico contra… | `x` final | rapidez final |
|---|---|---|
| **parede estática** | −16,649 | **12,001** |
| **caixa dinâmica leve** (densidade `0,05`) | −14,235 | **10,252** |

⇒ **um projéctil dinâmico é um PARTICIPANTE da física:** ele empurra o que toca e paga por isso em
rapidez e em trajectória. Uma bala de arcade não faz isso — ela atravessa a cena com a **mesma**
rapidez independentemente da massa do que bate. ⛔ *Isto não se afina com um knob: é a diferença
entre ter massa e não ter.*

### §1.3 — E o que a casa **não tem**, contado e não lembrado

`grep` na árvore: **alcance percorrido**, **homing**, **«a flecha aponta para onde voa»** e **tecto
de ricochetes** não são campo de componente nenhum. O `Lifetime` do #12 mata por **tempo**, que é
outra grandeza — *uma bala lenta e uma rápida com o mesmo tempo de vida têm alcances diferentes.*

### §1.4 — ⇒ A DECISÃO: o projéctil é **CINEMÁTICO**, como os dois irmãos

Ele escreve a própria pose (lei transversal 2 da síntese: *um dono do transform por vez*), o que lhe
dá de graça: massa irrelevante, determinismo cross-OS, e o mesmo laço de orçamento anti-túnel que o
#13 já paga (`move_character_from` é um **shape-cast**, logo não há túnel por construção).

---

## §2 — O ORÁCULO (§0.9)

A triagem do [plano do #13 §1.1](10_plano_topdown_player.md) continua válida e **pára na mesma porta
aberta**: o **Godot 4.7.2 (MIT)** é o único alvo instalado. A pergunta que sobra para ele é a que o
#13 já ensinou a fazer: **o que acontece ao ORÇAMENTO que resta quando o corpo bate?**

- no #13 (deslize) o `FLOATING` **conserva** o orçamento e re-emite na tangente — `1,414×` a 45°;
- aqui a hipótese é a irmã: re-emitir o resto **no espelho**.

⇒ o bloco novo da sonda mede `move_and_collide` + `get_remainder()` + `Vector2.bounce()`, que é a
receita canónica de um projéctil cinemático no Godot.

⚠️ **O `homing`, o `range` e o `face velocity` NÃO têm oráculo instalado** (o modelo é o
`ProjectileMovementComponent` do Unreal, que não está nesta máquina) — essas três metades são
desenhadas e defendidas por gates próprios, ⛔ nunca apresentadas como paridade.

---

## §3 — O desenho, com a porta ÚNICA de cada pergunta

### §3.1 — ⭐⭐ O ORÇAMENTO é o mesmo do #13 ⇒ ele SAI para uma folha partilhada

O laço «anda `budget` nesta `dir`; o mundo deixou andar `moved`; e agora?» é **letra por letra** o do
deslize — só a **re-emissão** muda (tangente contra espelho). ⇒ `SlideStep`/`first_step`/
`RESTO_MINIMO` mudam-se para a crate-folha nova **`ph2d-sweep`**, e cada lei fica com o seu
`next_step`.

⛔ **Não se duplica:** o orçamento é *a lei da wave anterior*, e escrevê-la duas vezes é exactamente
o defeito que esta linha pagou duas vezes em 2026-09-15 (a entrega do dedo, a chave das raízes).
⛔ **E a `ph2d-projectile` não depende da `ph2d-topdown`:** um projéctil não é um caso de vista de
cima, e a dependência lida ao contrário é a que envelhece.

### §3.2 — Os campos, e a porta de cada um

| campo | o que é | porta |
|---|---|---|
| `speed` · `acceleration` · `max_speed` | o avanço | `advance` |
| `gravity` | o arco, m/s² para baixo | idem |
| `bounce` (0..1) · `max_bounces` | o ricochete | `bounce::next_step` |
| `range` | metros percorridos até morrer (`0` = sem limite) | `Travelled` |
| `face_velocity` | a flecha aponta para onde voa | `rotation` (a porta do #13, **reusada**) |
| `homing_target` · `homing_accel` | a perseguição | `homing` |

⚠️ **O alvo é `stable_name_id`, nunca `Entity` bits** (lei 4 da síntese, e a lei do repo).

### §3.3 — O que NÃO entra, e porquê

⛔ **`DespawnOutside`** — o #12 já o tem. ⛔ **`WeaponFire`** (cooldown/munição) — é P1 e outro item.
⛔ **Um segundo marcador «Obstacle»** — a moeda é o `Collider` + camada (lei 1).

---

## §4 — Onde encosta em contrato congelado (§6) ou schema

| grandeza | delta previsto |
|---|---|
| `PROJECT_SCHEMA` | **+1** (componente registado novo ⇒ o load recusa em voz alta) |
| `register_physics_components` | **+1** |
| espelhos `ph2d-render` · `ph2d-script` | **0** — eles contam `ecs + render`/`ecs + script` |
| contrato congelado (§6) | **0** — nenhum dos sete aparece no desenho |

⚠️ **Conta-se o DELTA contra a árvore em que vai aterrar, nunca o literal.**

---

## §5 — As quatro condições de UI (independentes)

1. **EXISTE** — o componente e a linha na paleta *Add Component*;
2. **é PINTADO e REGISTADO** — a secção do Inspector + `LIVE_SECTIONS` (⚠️ o censo
   `architecture_every_live_section_is_in_the_table` do #13 apanha o esquecimento);
3. **o clique chega ao barramento** — `EditorAction::InspectorProjectileEdit` + a fase de dreno;
4. **a SEQUÊNCIA leva a algum lugar** — a cena de smoke.

---

## §6 — As waves

| # | o quê | fecha com |
|---|---|---|
| **W0** | as duas sondas (composição ✅ feita · oráculo do resto do orçamento) | tabelas medidas |
| **W1** | `ph2d-sweep` (extracção) + `ph2d-projectile` (lei pura) | gates da lei |
| **W2** | componente + ponte + schema | gates do produto contra a `rapier` |
| **W3** | o painel | as 4 condições |
| **W4** | smoke + mutação + fecho | o dono corre |
