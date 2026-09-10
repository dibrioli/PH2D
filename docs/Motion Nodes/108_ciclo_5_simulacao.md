# 108 — CICLO 5: SIMULAÇÃO — deixar a física decidir

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — um grupo por ciclo, params **no cartão**,
> e o entregável (e o smoke) é um **tutorial em PDF**.
> **Premissa do tutorial:** *«Deixar a física decidir»*.

Nos quatro ciclos anteriores o artista disse **onde** cada coisa está, **como** ela anda e **quem**
é afectado. Este grupo entrega a última decisão a uma **lei** — e o que ele pede em troca é um
**relógio**.

---

## §1 — Passo 2: A AUDITORIA

### §1.0 — O grupo é meio LISTA e meio FAMÍLIA, e a família é DERIVADA

`sim.zone` · `sim.spawn` · `sim.step` · `sim.lifetime` · `sim.collide` · `motion.integrate` — mais
**toda** a família `force.*`, pedida ao registry (`motion_ciclo_probe::familia("force.")`).

⚠️ **Uma lista de família escrita à mão envelhece em silêncio** no dia em que um nó dela nasce, e o
ciclo fecharia com ele por auditar. ⭐ Gate `the_force_family_is_derived_and_not_empty`, com piso:
sem ele, um prefixo mal escrito deixaria as cinco sondas a auditar **seis** nós em vez de doze —
todas verdes, todas caladas.

### §1.1 — ⛔ O catálogo já está FECHADO (as folhas 02, 03 e 13)

Como no ciclo 4: a conferência auditou estes nós contra Houdini/C4D/MOPs e as três folhas fecharam
a **zero P1**. ⇒ a auditoria do ciclo pergunta o que mudou desde então — e o que mudou é a
**superfície**.

### §1.2 — O RETRATO (`audit_the_sim_group`)

| nó | params | no cartão | device | portas | efeito |
|---|---:|---:|:---:|:---:|---|
| `sim.zone` | 5 | 3 | sim | 2→1 | Temporal |
| `sim.spawn` | 6 | 6 | sim | 2→1 | Temporal |
| `sim.step` | 4 | 4 | sim | 1→1 | Temporal |
| `sim.lifetime` | 3 | 3 | sim | 1→3 | Pure |
| `sim.collide` | **15** | **7** | sim | 1→1 | Pure |
| `motion.integrate` | 1 | 1 | sim | 2→1 | Temporal |
| `force.attractor` | 11 | 10 | sim | 2→1 | Pure |
| `force.buoyancy` | 8 | 8 | sim | 1→1 | Temporal |
| `force.curl` | 11 | 11 | sim | 1→1 | Temporal |
| `force.drag` | 3 | 3 | sim | 1→1 | Pure |
| `force.vortex` | 8 | 7 | sim | 1→1 | Pure |
| `force.wind` | 12 | 9 | sim | 1→1 | Temporal |

⭐ **Os doze estão no dispositivo.**

### §1.3 — O VOCABULÁRIO: dezasseis params partilhados, UMA divergência

`air_resist` · `angle` · `center_x/y` · `curve` · `lacunarity` · `loop_period` · `octaves` ·
`radius` · `roughness` · `seed` · `strength` · `substeps` · `type` — **todos** com o mesmo rótulo
em todos os nós que os declaram.

⚠️ A excepção é o **`mode`**: `sim.zone` chama-lhe **`Life Cycle`** (`Forever | Once | Loop`) e o
`force.vortex`/`force.wind` chamam-lhe **`Mode`** (`Force | Target Velocity`).

⛔⛔ **E o `Mode` vago é agora a SEGUNDA aparição em dois ciclos** (no 4 foi o `field.combine`).
Aqui é pior, porque os dois forces fazem **a mesma pergunta** — *«isto empurra, ou impõe uma
velocidade?»* — e ela tem um nome, que os doc-comments deles já citam da referência (o
*Treat as Wind* do POP Axis Force do Houdini). ⇒ **W1**.

### §1.4 — ⭐ E o `target_*` do attractor NÃO é o sexto vocabulário do pivô

O `force.attractor` chama ao seu ponto de mundo **`target_x`/`target_y`**, enquanto o
`force.vortex` e o `sim.collide` chamam **`center_x`/`center_y`**. Isso **parece** o achado §2.3 do
ciclo 3 e **não é**:

| palavra | o que ela quer dizer | quem a usa |
|---|---|---|
| `center_*` | **onde esta coisa está** | `field.box` · `field.radial_sweep` · `motion.falloff` · `force.vortex` · `sim.collide` |
| `target_*` | **aquilo para onde eu aponto** — e pode vir de um STREAM | `force.attractor` (`target_mode`) · `motion.look_at` (`Aim At`) |

⇒ são duas perguntas, e a casa já as separa de forma consistente. ⛔ Unificar destruiria sentido —
é uma cerca de Chesterton com a razão visível no `target_mode`.

### §1.5 — ⛔⛔ O ACHADO: TRÊS nós põem coisas no mundo e nenhum tem alça

`which_sim_nodes_have_a_place`, derivado (quem declara o vocabulário espacial contra o que o
`field_gizmo::spec_for` conhece):

| nó | centro | raio | ângulo | alça de canvas |
|---|:---:|:---:|:---:|:---|
| `sim.collide` | sim | sim | sim | ⛔ **NÃO** |
| `force.vortex` | sim | sim | — | ⛔ **NÃO** |
| `force.attractor` | `target_*` | sim | — | ⛔ **NÃO** |

⚠️ **É o achado do ciclo 4 um grupo adiante** — e aqui morde mais: o **colisor** é literalmente
uma coisa que se põe num sítio, e pôr um chão com dois sliders é o idioma que o gizmo existe para
substituir.

---

## §2 — O PLANO, wave a wave

### ✅ W1a — A ALÇA DO COLISOR, e a `spec_for` que passou a ver os PARAMS (2026-09-09)

O `sim.collide` é **literalmente uma coisa que se põe num sítio** — um chão, um prato, uma caixa —
e punha-se com dois sliders. Hoje tem caixa no canvas: mover · girar (`angle`) · redimensionar.

⚠️⚠️ **Ele é o primeiro nó cuja spec depende dos PARAMS e não só do TIPO:** as quatro formas
(`Plane · Disc · Bowl · Box`) medem-se com params **diferentes**, e o `motion.falloff` do ciclo 4
escapou a isso porque uma `Disk` servia as três formas dele. ⇒ `spec_for(type_id, &params)`.

⚠️ **O `Plane` recebe a caixa do `Box` de propósito:** um plano é infinito e não tem extensão para
agarrar, mas o `angle` e o `height` **são** o que a alça move. A alça de tamanho escreve num param
que aquela forma não lê — **inerte, não mentiroso**. ⛔ Não lhe dar spec nenhuma tirava também o
mover e o girar, que é exactamente o que ele precisa.

⛔ **Duas mutações, ambas nomeadas:** tirar o braço do colisor, ou congelar a leitura da forma, dão
*«a forma `Box` mede-se pelas duas extensões»*.

### ⛔ W1b — AS OUTRAS DUAS ALÇAS estão BLOQUEADAS, com o preço medido

O `force.vortex` e o `force.attractor` põem-se no mundo e **não têm param de ângulo** — girar um
vórtice em torno do próprio centro não move um texel. Uma spec deles teria `rotation: None`, e o
artista arrastaria a argola de rodar **sem nada acontecer**.

⇒ a cura certa é a `GizmoView` saber **suprimir** a argola, e o preço está **medido: 28 sítios de
construção dela**, em crates de **outras linhas** (`ph2d-editor-core`, `ph2d-tool-painter`). Um
campo obrigatório num struct partilhado é o oposto de *«projecte o foundational para isolamento»*
(CLAUDE.md §0.2), e colide na integração.

⛔ **Nomeado, não contrabandeado — e nada morto shipa.** O `rotation` da spec já é `Option`, então
o dia em que a `GizmoView` souber suprimir, as duas entram com um braço cada.

### W2 — ⭐ O `Mode` que não diz nada (o achado da §1.3)

### W3 — O CARTÃO do grupo, e os 8 de 15 do `sim.collide`

### W4 — A MEDIÇÃO (passo 5) e o TUTORIAL (passo 6)
