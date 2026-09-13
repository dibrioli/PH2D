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
(o `motion.collide` de CPU é `O(n²·iterações)`). ⛔ O passe de contatos **no dispositivo** só passa
a ter cliente no dia em que uma fonte que o dispositivo desenha (a sprite do `source.object`)
declarar colisor, e o preço está nomeado: no cozinhador o passe de grelha está preso ao **tipo de
nó** de cada etapa (`kernels.grid(stage.ty)`), então um segundo passe dentro do `sim.step` é
substrato novo.

---

## §2 — O desenho

> ⚠️ **A §2.1/§2.2 descrevem o colisor como um DISCO (`Collider Fit` · `Collider Scale`), e o
> report do dono do mesmo dia refê-lo: o colisor tem FORMA — caixa ou círculo — e uma alça no canvas.
> O desenho em vigor é o [§5](#5--o-colisor-tem-forma-caixa-e-círculo-e-uma-alça-no-canvas-report-do-dono-2026-09-13);
> a coluna, a porta, o passo e a recusa desta secção continuam valendo.

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
contatos. Depois da integração, se o estado traz `collider`:

1. **projecção de posições** por Jacobi com média (a lei do `motion.collide`: `r_i + r_j`, divisão
   pelo peso `inv_mass`, média por contagem de contatos) — numa **grelha uniforme** de CPU, para não
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
| ⏸️ **W5** o dispositivo | a sprite do `source.object` declara colisor · passe de contatos no cozinhador de GPU | **bloqueada por falta de cliente**: nenhuma fonte residente no dispositivo declara colisor antes dela (§1.3) |

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

### ✅ W3 — a cena `=114` reescrita (⏳ smoke do dono)

As duas taças são agora **quadrados do `source.shape`** carimbados por um duplicador, com a caixa
`Collide` ligada só à direita, e **nenhum `motion.collide`** na linha (gate). A taça pousa cada peça
pelo colisor dela (`Auto`). Medido pelos gates da cena (vão típico = mediana da distância ao vizinho
mais próximo, 2,6 s):

| | vão típico |
|---|---:|
| `Collide` desligado | `0,0000` — borrão total |
| `Collide` ligado (`Around`, raio `√2 · 0,11 = 0,1556`) | **`0,3094` = 99 % do diâmetro** |
| `Collider Scale` `0,6` · `1,0` · `1,4` | `0,184` · `0,309` · `0,433` |
| `Collider Fit` `Inside` (raio `0,11`) | `0,216` — os quadrados encostam pelos lados |

⭐ **O `99 %` é o ganho da correcção de velocidade:** a mesma cena com o `motion.collide` (que só mexe
em `P`) media `93 %` — a pilha respirava contra a gravidade; com a aproximação cancelada ela assenta.

⏳ **O default do `Collider Fit`** continua em `Around` até o dono ver as duas imagens no smoke.

## §4 — Aberto, com o que o decide

- ✅ ~~**O default do `Collider Fit`**~~ — **dissolvido pelo report do dono** (§5): nenhum dos dois
  círculos é a forma, e o `Collider Fit` saiu. O colisor é uma caixa ou um círculo adaptados à forma.
- ⏳ **O contorno exacto** (depois da caixa e do círculo) — o que o Cinema 4D chama *Convex Hull*; §9
  do [handoff de 10/09](handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-10.md) já o nomeava
  como inexistente em todo o repo.

---

## §5 — O colisor tem FORMA: caixa e círculo, e uma alça no canvas (report do dono, 2026-09-13)

> *«collider impreciso, o collider não é gerado conforme a forma da Shape. No mínimo precisamos de
> colliders circulares e retangulares que tentam se adaptar às dimensões da shape e que tenham
> ajustes de tamanho com gizmo visível para o usuário»* — com a foto da `=114` e uma seta num vão
> entre dois quadrados.

### §5.1 — O que a foto mede

O colisor da W1 era sempre um DISCO, e o default `Around` é o círculo pelos cantos. Num quadrado de
meio-lado `0,11` ele declarava raio `0,156`, e a pilha assentava com **`0,3094`** entre centros onde o
lado é `0,22` — **`141 %` do lado, `41 %` de ar**. O `Inside` (`0,2162`, `98 %`) encostava pelos lados e
deixava os cantos entrarem. *Nenhum dos dois é a forma*: um disco não descreve um quadrado.

### §5.2 — O desenho

| peça | o quê |
|---|---|
| **fundação** | duas colunas reservadas ao lado do `collider`: **`collider_box`** (as MEIAS extensões, `Vec2`, unidade de geometria) e **`collider_offset`** (o centro relativo à origem da peça). A caixa válida GANHA ao raio; `[0, 0]`/`0` — o que a união do `motion.combine` preenche — é «não declara» |
| **o shell** | mede a **caixa envolvente** do contorno de preenchimento (Bernstein, `64` amostras por curva) uma vez por geometria e publica centro + meias; o nó retira-as sempre |
| **o cartão** (`Collision`) | `Collide` · **`Collider Shape`** `Box` (default) / `Circle` · `Collider Width` + `Collider Height` (só com `Box`) · `Collider Radius` (só com `Circle`) — multiplicadores, `1` = a forma. ⛔ `Collider Fit` e `Collider Scale` SAÍRAM (nunca chegaram ao `main`) |
| **as duas formas** | `Box` = a caixa envolvente · `Circle` = o círculo que toca os lados MAIORES dela (num círculo é ele próprio, num quadrado o inscrito). As duas centram-se no meio da caixa: uma estrela de 5 pontas não está centrada na origem |
| **a porta** | `ph2d_contact::declarado` / `colisores` — lê as três colunas mais `size` e `rot` e devolve o colisor de MUNDO. **Três** clientes: `sim.step`, `sim.collide` e o gizmo |
| **o solver** | disco × disco (ao bit o de antes) · caixa × caixa pelo eixo separador (4 eixos, o de menor sobreposição) · disco × caixa pelo ponto mais próximo, e pela face de menor penetração com o centro dentro. O par calcula-se sempre do índice menor para o maior, e o maior recebe o simétrico |
| **o `sim.collide`** (`Auto`) | plano pelo SUPORTE da forma · disco e caixa sólidos pela porta do par · taça pelo CANTO mais longe. Um disco centrado continua no caminho de sempre, **ao bit** |
| **o dispositivo** | a recusa para a CPU cobre as três colunas pelo nome |
| **o gizmo** | com o cartão da forma seleccionado (tool Motion): o contorno do colisor em cada peça que ela carimbou, lido do SINK pela porta do solver; oito alças (quatro no círculo) na peça MAIS PRÓXIMA do cursor; o arrasto escreve os params do cartão pela porta do slider, simétrico à volta do centro, e é **um** passo de undo |

⚠️ **As caixas giram com a peça, mas nada aqui PRODUZ rotação** — o solver projecta posições, e uma
caixa pousada numa quina fica na quina. É o corpo rígido, e fica aberto (§5.5).

### §5.3 — Medido

**A `=114`** (vão típico = mediana da distância ao vizinho mais próximo, `2,6 s`; lado `2 · 0,11 = 0,22`):

| cartão da direita | vão típico | |
|---|---:|---|
| `Collide` desligado | `0,0000` | borrão |
| **`Box` (o default)** | **`0,2192`** | **`100 %` do lado** — era `141 %` |
| `Width` + `Height` `0,6` · `1,0` · `1,4` | `0,1318` · `0,2192` · `0,3071` | monótono |
| só `Width` `1,0` → `1,6` | largura da pilha `1,280` → `1,437` | a pilha ALARGA |
| `Circle` | `0,2162` (`98 %`) | toca os lados |
| `Circle` + `Radius 1,4` | `0,3064` | |

⭐ **A barra de CIMA entrou no gate** (`only_the_half_whose_shape_collides_keeps_the_pieces_apart`,
`≤ 1,15 × lado`): sem ela o gate ficava verde sobre o defeito da foto.

**O custo da tinta do gizmo** (`measure_the_outline_paint_cost`, perfil `dev`, a `load 6,3` — ⚠️ acima
da barra de `5` do `CLAUDE.md` §5.0, logo é um TECTO e não uma medida):

| contornos | codificar |
|---:|---:|
| 256 | 0,212 ms |
| 1 024 | 0,815 ms |
| 2 048 | 1,587 ms |
| 4 096 | 3,172 ms |
| 16 384 | 12,686 ms |

⇒ `MAX_CONTORNOS = 2048`, os mais próximos do cursor: ~`10 %` de um quadro no perfil mais lento, e a
vizinhança da mão fica sempre inteira.

**Paridade na placa:** `gpu_cpu_parity_sim` **30 de 30** — o `sim.collide` mudou o caminho de CPU, e
um disco centrado continua nele ao bit.

**Mutações — 15 de 15 mortas**, cada uma pelo gate nomeado:

| mutação | quem a mata |
|---|---|
| a caixa ignora a altura · o círculo toca o lado MENOR · o centro nunca é escrito | os gates do `declare` (`ph2d-node-motion-shape`) |
| o raio do eixo separador sem o eixo da caixa | `a_turned_box_collides_along_its_own_axes` |
| o eixo de MAIOR sobreposição | `a_stacked_box_is_pushed_along_the_axis_of_least_overlap` |
| o índice maior empurrado para o mesmo lado | `two_boxes_side_by_side_touch_face_to_face` |
| o disco dentro sai pela face de MAIOR penetração | `a_disc_that_entered_a_box_leaves_by_the_nearest_face` |
| a porta nunca lê a caixa | `the_declaration_door_reads_box_first_then_radius_and_carries_the_offset` |
| o plano sem o suporte · `Auto` não lê a forma | `a_declared_box_rests_on_the_floor_by_its_face` |
| a taça pelo canto mais PERTO | `a_declared_box_fits_whole_inside_the_bowl` |
| a alça sem o sinal do lado | `dragging_a_box_edge_resizes_that_axis_from_where_it_was` |
| o arrasto sem o `begin` do undo | `a_handle_drag_writes_the_card_param_and_is_one_undo_step` |
| as alças na peça mais LONGE | `the_gizmo_finds_every_piece_the_selected_shape_stamped` |
| a shell sem o `up` do colisor | `the_collider_gizmo_is_wired_like_the_warp` |

⚠️ **O `delayed` saltado no `sink_of` do colisor NÃO tem gate, e não é defeito:** o passeio é em
largura com memória de visitados, então ele acha o sink mesmo seguindo a aresta atrasada — saltá-la diz
*«o fio de estado não é o caminho do stream»* e poupa a volta ao laço, mas nenhuma resposta muda.

**Suítes:** `ph2d-app-motion` 1071 · 122 ignorados · `ph2d-contact` 15 · `source.shape` 18 ·
`sim.collide` 55 · `sim.step` 26 · `ph2d-node-registry-init` 146 · arquitectura 65 · costura da shell 8 ·
clippy `-D warnings` em 7 crates · fmt · typos.

### §5.4 — Armadilhas que a construção pagou

- ⛔ **O `sink_of` do gizmo de warp não serve numa simulação**: ele segue a PRIMEIRA aresta de cada nó,
  e a zona tem duas saídas — a primeira é a entrada atrasada do laço (`zone → wind`), e o passeio dava
  a volta ao laço até ao limite de passos. O do colisor anda em largura e não segue as atrasadas.
- ⚠️ **A estrela do manifesto tem SEIS pontas** e é simétrica nos dois eixos (centro medido
  `[1e-16, 0]`): a fixtura do centro deslocado precisou de `sides = 5` — a de omissão não tem o fenómeno.
- ⚠️ **O `Graph::set_param` recusa um não-finito à entrada**: a guarda do nó existe para o valor
  CONDUZIDO por fio, e por isso não se testa pelo grafo.

### §5.5 — Aberto

- ⏳ **O smoke do dono** da `=114` com as alças.
- ⏳ **Rotação por contacto** (corpo rígido) — uma caixa em quina fica em quina.
- ⏳ **O contorno exacto** (casco convexo) — o terceiro colisor, depois da caixa e do círculo.
- ⏳ **Centro e ângulo do colisor como params** — hoje o centro é o meio da forma e o ângulo o da peça.
- ⏳ **W4** (o `motion.integrate`) e ⏸️ **W5** (o dispositivo), inalterados.
