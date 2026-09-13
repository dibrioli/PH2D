# 109 — O COLISOR NA FORMA: as peças colidem SOZINHAS

> **Ordem do dono**, em duas metades: *«Vou preferir colocar na shape»* (2026-09-10, no smoke da
> `=114`) e, com as duas leituras à frente, **«Colidem sozinhas»** (2026-09-13).
> Registo da decisão: [doc 108 W7](108_ciclo_5_simulacao.md). Protocolo da linha:
> [doc 103](103_dinamica_dos_ciclos.md) — isto entra **antes do ciclo 6**.

**O produto, numa frase:** liga-se «Colide» no cartão da forma, e dentro de uma simulação as peças
deixam de se atravessar — **sem nó nenhum na linha da simulação**.

---

## §1 — O que está medido (antes de uma linha de código)

### §1.1 — Só quem desenha sabe o tamanho do que desenha

| mídia | `size = 1` desenha | fonte |
|---|---|---|
| `source.shape` (`Circle`) | **raio `1`** — a geometria nasce em raio 1 e a coluna `size` escala-a | `motion_shape_gen::publish` + `ShapeParams::read_unit` |
| sprite (quad) | lado `1` ⇒ **raio inscrito `0,5`** | medido e registado no doc do `sim.collide` (*«a 1×1 quad on a floor at `y = −2` comes to rest with its bottom edge at −2.5»*) |

| consumidor | o raio que ele usa | numa forma de `size = 1` |
|---|---|---|
| `motion.collide` | `radius × max(|sx|, |sy|)`, `radius = 0,3` de fábrica | `0,3` contra `1` desenhado |
| `sim.collide`, `Radius From: Sprite Size` | `min(|sx|, |sy|) × 0,5 × size_scale` | `0,5` contra `1` desenhado |

⇒ **nenhum consumidor sabe que mídia recebeu, e não pode saber:** o factor entre `size` e o que
se vê é da **mídia e da geometria** (um `Circle` e uma `Star` do mesmo `size` têm contornos
diferentes). É o argumento técnico da decisão do dono.

### §1.2 — O estado da arte põe o colisor NO OBJECTO e o motor resolve

| referência | onde vive o colisor | quem resolve | fonte |
|---|---|---|---|
| **Unity** | `Collider2D` é componente do `GameObject`; *«Rigidbody2Ds can't collide with each other without Colliders»* | o motor, sozinho | [Collider2D](https://docs.unity3d.com/6000.5/Documentation/ScriptReference/Collider2D.html) · [Rigidbody2D](https://docs.unity3d.com/6000.1/Documentation/ScriptReference/Rigidbody2D.html) |
| **Godot** | `CollisionShape2D` *«should be used as a child of Area2D, StaticBody2D, RigidBody2D…»* | o motor, sozinho | [CollisionShape2D](https://docs.godotengine.org/en/stable/classes/class_collisionshape2d.html) |
| **Cinema 4D** | etiqueta de dinâmica posta no **Cloner**; com *Individual Elements* os clones colidem entre si; formas substitutas `Box · Ellipsoid · Cylinder · Convex Hull · Static Mesh · Moving Mesh` (e *Auto* só em corpos rígidos) | o motor, sozinho | [Dynamics Body Tag: Collision](https://help.maxon.net/c4d/r21/us/html/DYNRIGIDBODYTAG-RIGID_BODY_GROUP_COLLISION.html) |
| **Houdini** | o raio vive **na partícula** (`pscale`) | ⚠️ **um nó explícito**: *POP Interact* (forças) ou *POP Grains* (PBD, move as partículas) | [POP Interact](https://www.sidefx.com/docs/houdini/nodes/dop/popinteract.html) · [POP Grains](https://www.sidefx.com/docs/houdini/nodes/dop/popgrains.html) |
| **PH2D, física de corpos rígidos** | `Collider { shape: Ball · Cuboid · Capsule }` é componente da entidade; *«its absence is the off»* | a `rapier`, sozinha | `ph2d-physics-ecs/src/components.rs` |

⭐ Três das quatro referências e a **nossa própria física** fazem o que o dono pediu. O Houdini é a
excepção, e mesmo lá o **tamanho** é da partícula.

### §1.3 — O dispositivo NÃO é o caminho de uma forma, hoje

`motion_bridge_gpu::cook_gpu` recusa o documento inteiro para a CPU quando ele traz um
`source.shape` (*«CPU: o grafo traz uma FORMA vectorial viva»*): o cozinhador de GPU não tem rota
para `geometry_id`, e *«a bomba de CPU é dona do tique desde o início»*.

⇒ **para peças que são formas, a obra que entrega o pedido é a de CPU** — e ela tem de escalar
(o `motion.collide` de CPU é `O(n²·iterações)`). ⛔ O passe de contactos **no dispositivo** só passa
a ter cliente no dia em que uma fonte que o dispositivo desenha (a sprite do `source.object`)
declarar colisor, e o preço está nomeado: no cozinhador o passe de grelha está preso ao **tipo de
nó** de cada etapa (`kernels.grid(stage.ty)`), então um segundo passe dentro do `sim.step` é
substrato novo.

---

## §2 — O desenho

### §2.1 — A declaração é uma COLUNA, e a ausência dela é o «desligado»

`collider` (escalar, por elemento): **o raio de colisão na unidade da geometria do elemento**. O
raio de mundo é `collider × max(|sx|, |sy|)` — a mesma lei do `motion.collide` (*«o disco que CONTÉM
a arte»*), para que escalar a peça a jusante escale o colisor com ela, como em todo motor.

- **Ausente ⇒ o elemento não colide.** É a lei do `Collider` da nossa física, e é o que mantém toda
  cena existente **byte-idêntica** — nenhum stream de hoje tem a coluna.
- ⚠️ **Não é param do `sim.step`:** quem responde *«que tamanho tenho?»* é quem desenha (§1.1).
- ⚠️ Viaja sozinha: o `motion.duplicator` replica toda coluna da forma, e a zona guarda todo o
  estado menos os transitórios.

### §2.2 — Quem escreve: o cartão da forma (secção «Collision»)

| param | o que é | default |
|---|---|---|
| `Collide` | liga a declaração | **desligado** ⇒ a coluna não é escrita |
| `Collider Fit` | `Around` = o menor círculo **à volta** do contorno (a arte nunca invade a vizinha) · `Inside` = o maior círculo **dentro** (encostam pelo miolo) | a medir no smoke (§4) |
| `Collider Scale` | multiplicador do raio escolhido | `1` |

⚠️ **O raio sai da GEOMETRIA, e a geometria é do shell** (o nó não alcança a biblioteca vectorial,
por desenho). O shell mede, uma vez por geometria, os dois raios em unidade de geometria e
publica-os no stream externo; **o nó escolhe** pelos params dele e emite só `collider`.
⛔ **Os params de colisão NÃO entram na `shape_key`:** não mudam a geometria, e entrarem nela
re-internaria um `VecPath` a cada clique na caixa.

### §2.3 — Quem resolve: o PASSO da simulação

O `sim.step` já é *«one integration step»* — o `world.step()` de todo motor integra **e** resolve
contactos. Depois da integração, se o estado traz `collider`:

1. **projecção de posições** por Jacobi com média (a lei do `motion.collide`: `r_i + r_j`, divisão
   pelo peso `inv_mass`, média por contagem de contactos) — numa **grelha uniforme** de CPU, para não
   ser `O(n²)`;
2. **correcção de velocidade** `v += Δp / dt`: sem ela a velocidade continua a empurrar para dentro da
   vizinha a cada tique e a pilha **respira** (é o `93 %` do vão que a `=114` mede hoje com o
   `motion.collide`, que só mexe em `P`).

⚠️ **A ordem na linha fica certa por construção:** `sim.step → sim.collide`, então o recipiente (a
taça) tem a última palavra, como a `=114` já exigia.

⚠️ **O `sim.collide` passa a ler a declaração:** uma peça com colisor tem de assentar na taça pelo
raio dela e não pelo centro. `Radius From = 0` lê a coluna quando ela existe — **byte-idêntico** onde
não existe, que é toda cena de hoje.

### §2.4 — O solver é UM, numa crate-folha

`ph2d-contact`: a projecção por Jacobi com média + a grelha, função pura. Dois clientes (o
`sim.step`; o `motion.integrate` na W4). ⛔ Um nó a depender de outro nó quebraria o isolamento de
drop-crate; copiar a lei seria a segunda resposta à mesma pergunta.

---

## §3 — As waves

| wave | entrega | gates |
|---|---|---|
| **W1** a declaração | coluna reservada `collider` na fundação · os dois raios medidos pelo shell · a secção «Collision» no cartão da forma | ausente ⇒ nada escrito, ao bit · `Circle`: `Around == Inside == 1` · `Square`: `Around = √2`, `Inside = 1` · a coluna atravessa o duplicador e a zona |
| **W2** o passo resolve | `ph2d-contact` · o `sim.step` projecta e corrige a velocidade · o `sim.collide` lê a declaração | sem coluna ⇒ passo ao bit · duas peças assentam a `r_i + r_j` · uma sem colisor atravessa · pino é obstáculo · a energia cinética não sobe no contacto · a grelha dá o mesmo que todos-os-pares |
| **W3** a cena | a `=114` passa a ser formas + duplicador, «Colide» desligado à esquerda e ligado à direita, **sem `motion.collide`** · anúncio e legenda | as duas metades (borrão × pilha) · o passo que o anúncio manda fazer existe no cartão |
| **W4** o outro integrador | o `motion.integrate` resolve pela mesma porta | idem W2 |
| ⏸️ **W5** o dispositivo | a sprite do `source.object` declara colisor · passe de contactos no cozinhador de GPU | **bloqueada por falta de cliente**: nenhuma fonte residente no dispositivo declara colisor antes dela (§1.3) |

⚠️ **Enquanto a W5 não existir, só fontes que recusam o dispositivo podem escrever `collider`** —
senão a mesma cena daria pilha na CPU e borrão no dispositivo. Isto é **gate**, não nota.

---

## §3-bis — O que já está feito (2026-09-13)

### ✅ W1 — a forma declara (`e597e82c7`)

Coluna `ph2d_nodegraph::attr::COLLIDER_COLUMN`; no `source.shape` os params `collide` ·
`collider_fit` · `collider_scale` (fora da `shape_key`) numa secção **Collision**; o shell mede
`around`/`inside` no contorno de preenchimento e o nó escolhe e **retira** as colunas de raio.
Medido: `Circle` `[1, 1]` · `Square` `[√2, 1]`. **4 de 4 mutações mortas.**

### ✅ W2 — a simulação resolve (`1ddd33333` + a W2b/W2c)

- **`ph2d-contact`** — a grelha dá **os mesmos bits** que todos-os-pares (os parceiros somados em
  ordem crescente de índice). ⛔⛔ **E o gate dessa igualdade NÃO prova a lei do par:** os dois
  caminhos chamam a mesma função por par, e a mutação do eixo dos centros coincidentes SOBREVIVEU a
  ele — as duas peças eram empurradas para o mesmo lado e nunca se separavam. Gate próprio escrito.
- **`sim.step`** — separa as peças com colisor depois da integração e **só cancela a aproximação**
  (nunca acrescenta velocidade: duas peças que nascem sobrepostas separam-se paradas).
- **`sim.collide`** — o modo 0 chama-se **`Auto`** (era `Point`): pousa a peça pelo colisor que ela
  declarou, e sem declaração é o ponto **ao bit**. O WGSL faz o mesmo termo a termo, e a coluna
  **ausente não gasta buffer** (`codegen::plan_bindings` dá `ReadIdentity`). Paridade na placa:
  **30 de 30**. ⚠️ O ramo com a coluna PRESENTE no dispositivo não é exercitado por teste nenhum —
  por construção ela não chega lá. O tutorial 05 foi regenerado com o rótulo novo.
- **A recusa para a CPU** — `graph_declares_collider`: todo documento com o nome `collider` num text
  param cozinha na CPU. ⚠️ **O buraco que ela fecha é real:** o canal `Custom…` do `motion.drive`
  escreve qualquer coluna pelo nome e recua sozinho para a CPU, mas numa rota HÍBRIDA a coluna
  atravessa a fronteira e chegaria a um `sim.step` na placa. Gate da pergunta + gate da LIGAÇÃO na
  placa (`the_bridge_cooks_a_document_that_names_the_collider_on_the_cpu`).
- **Mutações: 8 de 8 mortas** (4 na W2a, 4 na W2b/W2c — entre elas desligar a chamada no `cook_gpu`,
  que só o gate da placa apanha).

## §4 — Aberto, com o que o decide

- ⏳ **O default do `Collider Fit`** — `Around` garante que a arte nunca se sobrepõe e deixa folga
  nas diagonais de um quadrado e entre as pontas de uma estrela; `Inside` encosta pelo miolo e deixa
  as pontas entrarem. Decide-se no smoke da W3, com as duas imagens.
- ⏳ **O contorno exacto** (em vez de um círculo) — o que o Cinema 4D chama *Convex Hull*; §9 do
  [handoff de 10/09](handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-10.md) já o nomeava
  como inexistente em todo o repo.
