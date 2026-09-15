# HANDOFF DE INTEGRAÇÃO — `line/components`, o PROJÉCTIL (TOP-20 #14)

> **Data:** 2026-09-15 · **Merge-base:** `1d43da737` · **Plano:** [`11_plano_projectile_motion.md`](../11_plano_projectile_motion.md)
> **Estado:** fechada. **NÃO integrada, NÃO pushada** (§0.7).

---

## §1 — O que o ARTISTA consegue fazer agora

Põe um objecto na cena, carrega em **+ Add Component → Projectile Motion**, e:

1. ele **avança sozinho** para onde está virado, com aceleração e tecto de rapidez;
2. **Gravity** faz o arco de uma pedra atirada, e **Face Velocity** faz a flecha apontar para onde voa;
3. ele **ricocheteia** pela normal, com perda por salto e um tecto de saltos;
4. **Range** mata o voo por **metros percorridos** — e não por tempo;
5. **Homing** dá-lhe um alvo (pelo NOME) e ele persegue.

---

## §2 — Os contadores, como DELTA (⛔ nunca o literal)

| grandeza | delta | de → para |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** | `131 → 132` |
| `register_physics_components` | **+1** | `34 → 35` (delta **+3** contra o `main`) |
| `LIVE_SECTIONS` | **+1** | `24 → 25` |
| espelhos `ph2d-render` · `ph2d-script` | **0** | eles contam `ecs + render`/`ecs + script` |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `FIELD_DOC_VERSION` · `DOC_VERSION` | **0** | — |
| contrato congelado (§6) | **0** | — |

---

## §3 — Crates novas: DUAS

- **[`ph2d-sweep`](../../../crates/ph2d-sweep/)** — o **ORÇAMENTO DE MOVIMENTO**, com **zero**
  dependências. É a lei da wave #13, e o ricochete gasta-a exactamente como o deslize: só a
  **re-emissão** muda (tangente contra espelho).
- **[`ph2d-projectile`](../../../crates/ph2d-projectile/)** — a lei do projéctil (`libm` + a folha
  acima).

⚠️ Foundational tocado: `ph2d-ecs` (`DeathCause::Spent`), `ph2d-topdown` (o orçamento saiu),
`ph2d-physics-ecs`, `ph2d-editor-core`, `ph2d-panel-inspector`, `ph2d-component-desc`.

---

## §4 — Os SEIS achados, com o mecanismo

### 4.1 ⭐⭐⭐ A composição JÁ ricocheteia, e é por isso que o componente existe por OUTRA razão

**MEDIDO antes de uma linha** (`mede_o_que_a_composicao_ja_da.rs`): um corpo **dinâmico** com
`restitution = 1` dá o espelho **exacto** — razão `1,000` e o `vx` ao terceiro decimal em todos os
ângulos (90°..15°). ⇒ *escrever uma lei de ricochete nova seria um segundo motor.*

⭐ A razão real está na linha seguinte da tabela: o **mesmo** tiro contra uma **caixa leve** sai a
`10,252` em vez de `12,001` e por outro caminho. **Um projéctil dinâmico é participante da física;
uma bala de arcade não tem massa.** ⇒ ele é **cinemático**, como os dois irmãos.

### 4.2 ⭐⭐ O orçamento saiu para uma folha em vez de ser escrito duas vezes

O laço é letra por letra o do deslize. ⛔ Duplicá-lo seria duplicar *a lei que a wave #13 existiu
para trazer* — e esta linha pagou **duas vezes no mesmo dia** o preço de uma lei escrita em dois
sítios. O oráculo confirmou-a para o ricochete: o orçamento parte-se e **soma ao bit**
(`3,984375 + 6,015625 = 10,000000`), e o espelho acerta a **`0,00000°`** nos seis ângulos.

### 4.3 ⛔⛔ O `pose_owner` esqueceu o segundo mover — a wave a seguir à que o previu

Os **sete** gates da ponte nasceram vermelhos com a bala parada na origem: sem o componente na
lista de *quem escreve a própria pose*, o `drive_kinematic` repõe a pose autorada em todo tique —
e o corpo fica onde nasceu **com todos os números certos**. O §4.5 do handoff do #13 escreveu isto
por extenso, e mordeu à mesma.

### 4.4 ⭐⭐⭐ A morte tem UM despachante, e o filtro protege o trabalho do artista

A ponte **anuncia** (`projectile_done`) e quem remove é o dreno que o `Lifetime` do #12 já usa, com
um motivo novo no vocabulário partilhado (`DeathCause::Spent`). ⚠️⚠️ E o filtro é
`ph2d_ecs::is_transient` — *o que nasce numa corrida não é documento*: um projéctil que o artista pôs
na cena à mão **não se apaga**; ele pára e fica.

### 4.5 ⚠️⚠️ Duas mutações SOBREVIVERAM, e as duas eram gates meus a medir a grandeza errada

- **a memória do voo:** a régua pedia *«caiu mais de `1,5 m`»*, e a bala com a memória apagada cai
  **`1,93 m`** — passa. ⭐ O mecanismo é subtil: com `face_velocity` ligado a ponte escreve o ÂNGULO
  a cada tique, e o re-nascimento seguinte lê-o ⇒ **a rotação é uma segunda memória de direcção**, e
  ela mascara a perda da primeira. A régua que separa é uma **LEI**: a gravidade não toca em `x`,
  logo o `x` é **exactamente** `v·t` (`3,000` contra `2,199`), e a queda é a soma discreta exacta.
- **o canal da morte:** ele é limpo a cada **dispatch**, e o gate lia-o no fim de 120 tiques — o
  defeito matava a bala ao tique `40` e o canal já estava vazio. *Um canal por-tique lê-se a cada
  tique.*

### 4.6 ⛔ Dois tectos de LOC curados por CORTE, nunca por isenção

A escada do `PROJECT_SCHEMA` partiu-se por **IDADE** pela terceira vez (`v99`..`v111`), o publicador
de instantâneos do Inspector largou a metade da **FÍSICA**, e o `state.rs` do painel largou as portas
dos instantâneos da fila do TOP-20.

---

## §5 — As SEIS leituras que o diff inverte

1. **A `ph2d-sweep` não tem `libm`, e isso é o desenho** — os transcendentais vivem nas leis que a
   consomem. *Uma folha que não precisa dele não o declara.*
2. **`range = 0` é SEM LIMITE e `max_speed = 0` é SEM TECTO** — um zero lido como limite mataria
   toda bala ao nascer. Os dois com gate.
3. **A `bounciness` toca nas DUAS grandezas com o MESMO número** — orçamento e velocidade são a
   mesma coisa (`orçamento = |v|·dt_restante`), e as duas saem por UMA porta (`Bounced`).
4. **A aceleração pode ser NEGATIVA** — é o que faz uma bala travar no ar; prendê-la em `0` apagaria
   metade do campo.
5. **O homing olha para onde o alvo ESTÁ, não para onde ele vai** — antecipar é outra lei, e um
   míssil que a tivesse deixaria de ser desviável.
6. **`ProjectileState` não é componente**, e a ausência é a decisão: três campos que mudam por tique
   dentro de um componente registado fariam o undo ver cada quadro como um passo.

---

## §6 — As premissas que a medição derrubou

1. *«O ricochete é a entrega»* — ele já existe, exacto. A entrega é a **ausência de massa**.
2. *«O corpus da quina mede uma quina»* — a 1.ª fixtura media o VAZIO (o corpo tocava uma parede e o
   resto nunca alcançava a outra), e lia `1 salto` sobre uma quina que não existia no alcance do
   tique.
3. *«Imprimir a saída e o espelho lado a lado compara-os»* — eram um DESLOCAMENTO e uma VELOCIDADE,
   em unidades diferentes na mesma linha. A comparação honesta é o **ângulo**.
4. *«Um gate de alcance com `range = 100` mede o alcance»* — em 120 tiques o defeito acumula `12 m`
   e nunca lá chega. *Uma fixtura que não consegue produzir o sujeito do próprio teste passa
   sempre.*

---

## §7 — O SMOKE (o que o dono corre)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_PROJECTILE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_PROJECTILE_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke
```

---

## §8 — O que fica ABERTO

1. **O `WeaponFire`** (cooldown/munição/recarga) é P1 e outro item — sem ele o projéctil é criado
   pela fábrica do #11 ou à mão.
2. **A perseguição não antecipa** (§5.5) — a mira do `CameraFollow` do #7 já tem a lei, e portá-la
   é produto novo sem lado aprovado.
3. **O custo a N projécteis não foi varrido.** O laço é o do irmão (um cast por salto por tique), e
   o `max_bounces` é a alavanca.
4. ⚠️ **Uma flake de carga NOVA para promover:**
   `glaze_layering_costs_a_ratio_not_an_order_of_magnitude` (`ph2d-wet-paint`) — reprovou no fan-out
   de 23 079 e passou **3 de 3** sozinha a `load 142–151`, com **zero** linhas do diff naquela
   crate. É um gate de RAZÃO, a forma canónica da família.
