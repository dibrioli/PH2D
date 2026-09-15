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
- ✅ ~~**Rotação por contacto**~~ — **pedida e feita no mesmo dia**: §6.
- ⏳ **O contorno exacto** (casco convexo) — o terceiro colisor, depois da caixa e do círculo.
- ⏳ **Centro e ângulo do colisor como params** — hoje o centro é o meio da forma e o ângulo o da peça.
- ⏳ **W4** (o `motion.integrate`) e ⏸️ **W5** (o dispositivo), inalterados.

---

## §6 — O colisor VÊ-SE em todas as formas, e as peças RODAM (2.º report do dono, 2026-09-13)

> *«o collider deve aparecer na frente da shape (z-index maior) e visível em TODAS as formas.
> Coloque um botão no nó shape: Ver collider. Precisa destravar a rot. e colocar outro botão para
> travar rotação»*

### §6.1 — O dono tinha razão: o contorno ESTAVA por baixo

⛔⛔ **A minha primeira leitura foi que a ordem já estava certa** — *«a fase que desenha o gizmo corre
depois da que codifica a arte»*, lido dos sítios das chamadas — e escrevi-a aqui. A RÉGUA refutou-a no
mesmo dia: no quadro emendado (`frame_text::render_frame`, que costura cada fase no ponto em que ela
CORRE) a tinta do gizmo aparecia em **`484 970`** e a arte das formas em **`623 773`** — o gizmo era
pintado **antes**, e no Vello quem pinta primeiro fica por baixo.

⇒ a cura é o SÍTIO da chamada: os dois gizmos de nó (o do colisor e o do warp) passam a ser
desenhados na `fase_vector_overlays`, **logo a seguir** ao `motion_shape_gen::encode` e na mesma cena.
Gate novo com essa régua: `the_collider_outline_is_painted_after_the_shape_art`, que mede os DOIS.

⚠️⚠️ **E isto reabre o report de 2026-09-08 do gizmo de warp** (*«está sendo desenhado por trás das
shapes»*): a cura de então foi um **casing** escuro, sob uma nota que afirmava *«o gizmo ESTÁ por
cima»*. O casing é bom pelo outro motivo (um traço claro some sobre conteúdo claro), mas a ordem
estava errada desde aí — *uma afirmação sobre a ordem do quadro tirada do sítio do CÓDIGO é um palpite
com cara de medição; o quadro emendado é a régua.*

As outras duas metades do report:

- **o contorno das peças SEM alças era o traço fino e apagado** (o dos braços do warp), e sobre uma
  folha de quadrados brancos isso soma-se ao defeito de ordem. Hoje **todo contorno** leva o traço
  forte com casing, e o que distingue a peça com alças é **ter alças**;
- **só a forma SELECCIONADA pintava.** Hoje pinta **toda** forma com `Collide` e `Show Collider`
  ligados, esteja seleccionada ou não; as alças continuam a ser da seleccionada, porque o arrasto
  edita os params DELA.

### §6.2 — Os dois botões novos no cartão (secção «Collision»)

| param | o que faz | default |
|---|---|---|
| **`Show Collider`** | pinta o contorno do colisor de cada peça desta forma, por cima da arte | **ligado** — quem liga o `Collide` quer ver o que declarou |
| **`Lock Rotation`** | escreve `inv_inertia = 0`: as peças desta forma não rodam no contacto | **desligado** — o dono pediu a rotação destravada |

⚠️ Os dois só aparecem com o `Collide` ligado, e nenhum entra na chave de conteúdo da geometria.

### §6.3 — A rotação: a correcção reparte-se entre MOVER e RODAR

Todo contacto passa a saber **onde** toca, e a projecção de posição vira a canónica do PBD de corpo
rígido:

```text
  r = ponto − centro      c = r × n      k = w + invI · c²
  λ = penetração / (k_a + k_b)      Δp = n · λ · w      Δθ = c · λ · invI
```

⚠️ **`invI = 0` devolve a lei anterior termo a termo** (`k = w`, `λ = pen/(w_a + w_b)`), e é isso que o
`Lock Rotation` escreve. A inércia ausente DERIVA da forma: caixa `3w/(hx² + hy²)`, disco `2w/r²`.

| onde | o PONTO do contacto |
|---|---|
| disco × disco | sobre a normal, a meio do trecho sobreposto |
| caixa × caixa | o **meio do trecho recortado** da face incidente contra a de referência |
| disco × caixa | o ponto da caixa mais próximo do disco (ou o da face, com o centro dentro) |
| plano / chão | o **ponto de suporte**: o meio da face quando ela é paralela, a quina quando inclinada |
| taça | o ponto extremo — a borda do disco, o canto mais longe da caixa |

⛔ **Não há velocidade ANGULAR.** A rotação é projecção de posição, como o afastamento: uma peça roda
enquanto toca e não continua a girar no ar. É o que separa isto de um corpo rígido a sério.

⚠️⚠️ **Dois defeitos que a construção pagou, e os dois são sobre o PONTO:**

- o do plano era `centro − n · suporte`, que cai sempre **debaixo do centro** ⇒ braço zero, e uma
  caixa a `20°` nunca se endireitava (medido: giro `0,000`). O certo é o ponto de suporte — e com a
  face paralela ele tem de ser o **meio** dela, senão uma peça pousada de chapa tomba sozinha;
- o de duas caixas como *«vértice mais fundo»* faz uma **pilha parada** tombar, pela mesma razão. Daí o
  recorte, com gate próprio (`a_box_resting_flat_on_another_does_not_turn`).

### §6.4 — Medido

**A `=114`, `2,6 s`, a metade que colide** (25 peças):

| | |
|---|---|
| destravada (o default) | maior ângulo **`49,4°`**, **22 de 25** acima de `5°` |
| `Lock Rotation` ligado | a coluna `rot` **nem nasce** — a cena sai como saía antes da rotação existir |

⚠️ **E a régua de GEOMETRIA da cena passa a correr TRAVADA** (`pilha_travada`): com as peças a tombar,
um quadrado a `45°` toca o vizinho pela quina e o vão típico sobe até à DIAGONAL — uma barra de
«encostado» medida sobre isso não separaria o encosto do ar que o dono fotografou. A rotação tem gate
próprio (`the_pieces_tumble_unless_the_card_locks_the_rotation`), e os números de §5.3 continuam a ser
os da pilha travada.

**Mutações — 27 de 27 mortas** nas duas waves. As do §6, com o gate que as mata:

| mutação | quem a mata |
|---|---|
| o suporte sempre na quina (nunca o meio da face) | `a_tilted_box_turns_on_the_floor_and_flat_or_locked_does_not` |
| o contacto de duas caixas no vértice mais fundo · o giro sem o braço | `a_box_resting_flat_on_another_does_not_turn` |
| a caixa com a inércia de um disco | `the_inverse_inertia_comes_from_the_shape_and_a_pin_never_turns` |
| a coluna que não trava | `the_inertia_column_locks_and_the_absent_one_derives` |
| a massa efectiva apagada (`k = 1`) | a metade NOVA do gate do plano (*travada sobe mais que livre*) |
| o giro que não chega ao ângulo | `a_box_caught_off_centre_turns_unless_the_column_locks_it` |
| `Lock Rotation` que não escreve a coluna | `locking_the_rotation_writes_a_zero_inertia_column` |
| o `Show Collider` ignorado · só a seleccionada a pintar | `every_shape_that_shows_its_collider_is_drawn_and_only_the_selected_one_has_handles` |
| o contorno apagado do quadro | `the_collider_outline_is_painted_after_the_shape_art` |

⚠️ **Uma SOBREVIVEU à primeira volta, e o gate nasceu daí:** apagar a massa efectiva (`k = 1`) não
muda a direcção nem o travamento — muda só a REPARTIÇÃO, e nada media isso. O gate que a mata compara
quanto a peça SOBE travada e livre: com a rotação livre ela sobe menos, porque parte do empurrão virou
giro.

⚠️ **E o tecto de LOC apanhou o solver**: com a rotação a `ph2d-contact/src/lib.rs` foi a `821`
linhas. Cortada por responsabilidade — a lei de UM par (normal, profundidade, ponto) vive agora em
`par.rs` (`217`), e o `lib.rs` (`615`) fica com a NUVEM: a grelha, o Jacobi com média e a ordem das
somas.

### §6.5 — Aberto

- ⏳ **Velocidade angular** (uma peça que continua a girar depois do toque) e **atrito angular**.
- ⏳ **O contorno exacto** (casco convexo), §5.5.
- ⏳ **W4** (o `motion.integrate`) e ⏸️ **W5** (o dispositivo).

---

## §7 — A peça tem MATERIAL, e é o atrito que roda um círculo (3.º report do dono, 2026-09-13)

> *«Os círculos não rotacionam com a colisão, talvez por falta de atrito. Precisamos de parâmetros
> do material.»*

### §7.1 — A conta que dá razão ao report, e que o explica melhor do que o palpite

A correcção de não-penetração (§6) move as peças **ao longo da normal**, e a alavanca dela é
`r × n`. Num disco o ponto de contacto está **sobre a linha dos centros**, logo `r = ±R·n` e

```
r × n  =  ±R · (n × n)  =  0
```

— zero **por construção**, nas três rotas que produzem um contacto de disco (disco×disco ·
disco×caixa, por dentro e por fora · o ponto de suporte contra um plano). ⇒ **nenhum número dado
àquela metade podia rodar um círculo.** Não era um parâmetro mal afinado: era uma metade ausente.

⭐⭐ E a metade que faltava tem a alavanca **máxima exactamente onde a outra tem zero**. Com
`t = perp(n)` vale a identidade `r × t = r · n`, e num disco `r · n = ±R`:

| | alavanca da NORMAL (`r × n`) | alavanca da TANGENTE (`r · n`) |
|---|---|---|
| disco | **0** | **±R** (o raio inteiro) |
| caixa de chapa | 0 (o meio da face) | `−h` (meia altura) |
| caixa de quina | ≠ 0 | ≠ 0 |

*As duas são as duas coordenadas do mesmo vector: onde uma é zero a outra é tudo.* Gate:
`a_disc_has_no_lever_on_the_normal_and_all_of_it_on_the_tangent`
([`ph2d-contact/src/tests.rs`](../../crates/ph2d-contact/src/tests.rs)).

⚠️ **Zero em aritmética exacta, RUÍDO em `f32`.** O braço é `ponto − centro`, e numa peça longe da
origem isso subtrai dois números muito maiores do que a diferença. Medido na cena `=115` (bola a
`x ≈ 1,35`, 2 s de rampa): **`0,0000075°` acumulados**. ⛔ *Uma barra de zero-ao-bit sobre isto
mediria a aritmética e não a lei* — os gates de cena usam `1e-3°` com este número escrito ao lado.

### §7.2 — E já HAVIA atrito. Ele era do tipo errado.

⛔⛔ O `sim.collide` tem um param `friction` desde que existe, e a cena `=114` põe-no a `0,6`. A
lei dele era **um sangramento da velocidade tangencial** (`vt *= 1 − atrito`), aplicado **no centro
de massa**: ele tira a derrapagem e **não produz binário nenhum**. ⇒ *uma bola com atrito máximo
PARAVA em vez de ROLAR.*

É a família do §5.0 do `CLAUDE.md` — **o consumidor que projecta o valor fora**: o fio está
completo, o número chega ao solver, e a matemática descarta a parte que interessa. Nenhuma sonda de
*«quem lê este param?»* o vê: ele **é** lido.

### §7.3 — A lei: atrito como restrição POSICIONAL no ponto de contacto

É a restrição de atrito do PBD (Müller et al. 2007 §3.5, na forma de corpo rígido de
Macklin–Müller–Chentanez). O **deslize** é quanto os dois pontos de contacto se mexeram um em
relação ao outro ao longo de `t` **desde o início do passo**; o atrito desfá-lo, repartido entre
mover e RODAR pela massa efectiva tangencial, limitado por **Coulomb**:

```
λt = clamp( deslize / (kt_a + kt_b),  ±μ · λn )     kt = w + invI · (r · n)²
Δp = −t · λt · w                 Δθ = −(r · n) · λt · invI
```

⭐ **Numa bola pousada isto dá o rolamento de manual à primeira correcção:** `kt = w + 2w = 3w`,
logo `⅓` do deslize vai para a translação e `⅔` para a rotação, e a soma no ponto de contacto é
exactamente `−deslize`. *Ela deixa de derrapar porque começou a ROLAR, não porque travou.* Gate
`the_rolling_split_gives_the_spin_twice_the_slide` — medido `2,00×`.

⚠️ **O limite de Coulomb é o que a torna física e não cola.** Numa pilha assente `λn` é a penetração
que a gravidade fez **naquele tique** (`~g·dt²`), então o atrito por tique é `μ·g·dt²` — pequeno de
propósito. Uma bola a `1 u/s` sob gravidade `4` com `μ = 0,6` passa de derrapar a rolar em ~0,3 s.

⚠️ **O deslize inclui a rotação PRÓPRIA da peça** (`spin` integrado), e não só a do contacto: uma
bola que chega a girar derrapa contra o chão mesmo parada. O `sim.step` calcula esse ângulo antes de
chamar o contacto — o valor é o mesmo que ele já integrava, e a ordem das somas no `rot` não muda.

### §7.4 — Duas moedas, e a fronteira entre elas é DECLARADA

| onde | moeda | quem roda |
|---|---|---|
| peça × peça (`ph2d-contact::separate`, dentro do `sim.step`) | **posição** | `rot`, pela projecção |
| peça × obstáculo (`sim.collide`) | **velocidade** | `spin`, que o `sim.step` integra |

Não é uma lei escrita duas vezes: é a mesma lei nas duas moedas que cada substrato fala. O
`sim.collide` é `Effect::Pure`, **não tem `dt` nenhum**, e sem um passo um deslocamento não é
derivável de uma velocidade — ali o atrito é o impulso tangencial que anula a velocidade **do ponto
de contacto** (`v·t + ω·bt`), limitado a `μ·jn`, repartido entre travar e rodar.

⚠️⚠️ **E ele AUTO-CORRIGE**: como a velocidade lida é a do PONTO, uma bola a girar depressa demais é
travada pelo mesmo termo que a pôs a girar. *Sem o `ω·bt` ela acelerava para sempre* — o
`angular_damping` do `sim.step` nasce em `1` (sem arrasto) e nada mais no quadro a travaria. Gate
`a_disc_spinning_too_fast_is_slowed_by_the_same_friction`.

⛔ **A peça SEM forma declarada fica no sangramento de sempre, ao bit** — um ponto não tem raio, e
inventar-lhe um mudaria toda cena que este nó já shipou. Gate
`the_undeclared_path_is_the_tangential_bleed_verbatim`, que escreve a lei antiga inline.
⚠️ E `Lock Rotation` **não** troca a lei: com `invI = 0` o Coulomb continua a valer e o giro sai zero
por construção. *Um botão que diz «não rodes» não pode mudar quanto a peça TRAVA.*

### §7.5 — O material, e como um par se combina

Duas colunas reservadas novas — `friction` e `bounce` ([`attr.rs`](../../crates/ph2d-nodegraph/src/attr.rs)) —
e duas linhas na secção `Collision` do cartão: **`Friction`** (default `0,5`, o do Rapier) e
**`Bounciness`** (default `0`, o do Rapier e o do Box2D).

| grandeza | combina por | porquê |
|---|---|---|
| atrito | **média geométrica** `√(μa·μb)` | uma peça de gelo desliza contra tudo, que é o que «gelo» quer dizer; um `max` faria uma peça de lixa colar tudo ao chão |
| salto | **o maior dos dois** | uma bola saltitante salta contra uma parede morta |

(as duas leis são as do Box2D.)

⚠️⚠️ **`None` e `Some(0)` são coisas DIFERENTES, e a distinção é load-bearing.** O obstáculo do
`sim.collide` tem atrito próprio; uma peça que **não declarou nada** tem de continuar a sofrê-lo
inteiro (a lei de sempre, ao bit), enquanto uma peça que declarou `Friction 0` **é gelo** e desliza
sobre ele. Colapsar as duas faria toda cena de hoje perder o atrito do chão — é por isso que
`ph2d_contact::materiais` devolve `Option`, como a irmã `colisores`.

⚠️ **As duas colunas escrevem-se SEMPRE que o `Collide` está ligado, mesmo a zero** — aqui a lei
estrutural do `fill` (*«o default não é escrever o mesmo valor, é não escrever»*) **não vale**: `0`
não é *«como estava»*, é **gelo**, e gelo é um pedido.

### §7.6 — A cena `=115`, e o que ela mede

`PH2D_GPU_COOK_DEMO=115` — seis bolas em três fileiras, e em cada par muda **um** número do cartão
da forma (a rampa, o chão, a taça, a gravidade e o relógio são os mesmos nós com os mesmos números
**dentro de cada par**, com gate: `only_the_shape_card_differs_between_the_halves_of_a_pair`).

| fileira | par | medido |
|---|---|---|
| rampa (peça × MUNDO) | `Friction 0` | andou `1,60` · rodou **`−0,0°`** |
| | `Friction 1` | andou `1,08` · rodou **`307,5°`** — `0,97×` o rolamento puro (`316°`) |
| queda (peça × MUNDO) | `Bounciness 0` | pico `0,200` = pousada no chão (`0 + R`) |
| | `Bounciness 0,9` | pico **`0,572`** — sobe `0,372`, metade da queda |
| taça (peça × **PEÇA**) | `Friction 0` | 16 bolas, giro total **`0,0°`**, monte `1,562` de largo |
| | `Friction 1` | as mesmas 16, giro total **`469,9°`** (maior `77,5°`), monte `1,392` |

⭐⭐⭐ **A terceira fileira é a resposta ao 3.º report** (*«as propriedades entre as próprias shapes
não funcionam»*), e a decisão que a torna uma afirmação honesta é **a taça ser ESCORREGADIA nas duas
metades**: `μ` contra a parede é `√(0 · μ_bola) = 0` dos dois lados, logo **todo** o giro medido
nasce de bola contra bola. ⛔ Com um obstáculo áspero metade da rotação viria da parede e o gate
ficaria verde sem provar nada sobre o par de peças.

⚠️ **E as bolas NASCEM DENTRO da taça** (gate próprio, `1,033` contra o limite de `1,090`): um
recipiente projecta para dentro tudo o que nasce fora, e a cena abriria com um SALTO a cada volta do
laço — a armadilha que a `=114` já nomeia por escrito.

⚠️ **A bola leva um TRACEJADO no contorno**, e ele é a razão de ela ser desenhada assim: *um círculo
liso rodado é indistinguível de um círculo parado.* Uma cena que demonstra rotação tem de desenhar
uma coisa cuja rotação se veja.

⚠️ **A cena `=114` não regride**: o vão típico da pilha continua em `0,2192` (`100 %` do lado) e as
peças continuam a tombar (`54,0°` de máximo, `18` de `25` acima de `5°`, contra `49,4°`/`22` antes
do atrito). *O que mudou foi elas deixarem de escorregar umas sobre as outras.*

⛔⛔ **E porque o 3.º report era JUSTO mesmo com a lei a funcionar.** Medido na `=114` variando só o
`Friction` do cartão:

| colisor | `Friction 0` | `0,5` (o default) | `1` |
|---|---|---|---|
| **Circle** | `0°` de giro, monte `2,220` de largo | `730°`, `1,819` | `869°`, `1,665` |
| **Box** | `685°`, `2,258` | `601°`, `1,906` | `636°`, `1,960` |

⇒ *entre peças a lei age* — e **onde o dono iria olhar ela era invisível**: a `=114` é uma pilha de
**quadrados** (colisor `Box`, o default), e ali o `Friction` mexe `±8 %` num giro que a metade
NORMAL já produz sozinha. Some-se a isso o default `0,5`, que não deixa nenhum estado «desligado»
com que comparar. *Um controlo cujo default já faz a coisa, numa cena onde ele quase não pesa,
lê-se exactamente como um controlo que não funciona.* A cura foi dar-lhe o A/B que faltava.

### §7.7 — Aberto, com o mecanismo

- ⏳ **O kernel de GPU do `sim.collide` não conhece nem a forma declarada nem o material** — ele
  implementa `contact` + o sangramento. É o **mesmo** vão que o §5 já abriu (a caixa declarada
  também não chega lá), e não um vão novo: quem o fechar fecha os dois de uma vez.
- ⏳ **A rotação do contacto peça×peça continua sem velocidade angular** (§6): ela é projecção de
  posição, e uma peça roda enquanto toca. O que MUDOU é que uma bola contra um obstáculo agora
  ganha `spin` de verdade — as duas metades do app já não dizem a mesma coisa sobre isto, e
  unificá-las é uma wave própria.
- ⏳ **Entre CAIXAS o `Friction` quase não pesa** (tabela acima): o encaixe geométrico de duas faces
  planas já impede o deslize, e o que sobra para o atrito é pouco. Não está errado — está **medido e
  nomeado**, e quem quiser mudá-lo tem de dizer que recurso é que o limita.
- ✅ **Não há atrito de ROLAMENTO** — **FECHADO na §7.10.** A redacção original dizia: *«(o que faz
  uma bola a rolar parar sozinha num plano): com `angular_damping = 1` (o default) ela rola para
  sempre num chão infinito. **É um param, não uma lei**»* — ⚠️ e essa última metade estava **errada**:
  o `Friction` não podia fazê-lo por construção (o deslize de uma bola a rolar é exactamente zero),
  logo era preciso um impulso novo. *O diagnóstico estava certo e o preço não.*

### §7.8 — O salto vai ao DOBRO, e a frase que o segurava nunca tinha sido medida

> *«Quero mais capacidade de Bounciness — de zero até o dobro do máximo atual.»* (dono, 2026-09-13)

⛔⛔ **O `1` vinha de uma frase, não de uma medição.** Ela vive no `sim.collide` desde que ele
existe — *«uma batida que devolve mais do que levou é uma máquina de fazer energia, e acaba com a
cena em órbita»* — e é **verdadeira em Física e falsa sobre o que este código faz**. Medido numa
queda de `0,75` sob gravidade `4`, **sem laço**, 40 s:

| `Bounciness` | pico | `v` máxima | saltos | finito? |
|---|---|---|---|---|
| `0,90` | `0,95` | `2,40` | 16 | sim |
| `1,00` | `0,95` | `2,47` | 50 | sim — salta **para sempre**, que é o que «perfeitamente elástico» quer dizer |
| `1,25` | `39,6` | `22,2` | 10 | sim |
| `1,50` | `91,8` | `40,7` | 7 | sim |
| **`2,00`** | **`192,0`** | **`78,3`** | 5 | **sim** |
| `3,00` | `550,8` | `66,4` | 3 | sim |

⇒ **nada diverge, nada vira `NaN`, nada atravessa o chão.** O que acontece acima de `1` é a bola
subir cada vez mais — que é *exactamente* o que o artista pede ao passar de `1`. *A «órbita» da
frase era uma previsão, e a previsão estava certa sobre a trajectória e errada sobre a estabilidade.*

⚠️⚠️ **A primeira medição mediu o programa ERRADO.** Corrida na cena `=115` tal como ela shipa, a
`2,00` dava `pico 3,2` e `v 4,9` — tudo manso — porque a `sim.zone` está em **laço** e reinicia
antes de qualquer acumulação. *Uma régua corrida sobre uma cena que se reinicia mede o relógio da
cena, não a lei.* A tabela acima é com `duration = 600`.

⚠️ **O limite REAL tem recurso com nome, e não é este número:** acima de `2R/dt` (na `=115`,
`24 u/s`) a peça atravessa o próprio diâmetro num tique, e colidir com algo que não seja um plano
infinito deixa de ser fiável. Uma queda **sem laço** chega lá acima de `~1,25`. ⛔ Isso é propriedade
do **integrador**, não deste param — a cura que já existe é o `Speed Limit` do `sim.step` — e numa
cena em laço (todas as deste catálogo) não chega a nascer.

**O que mudou:**

- `ph2d_nodegraph::attr::BOUNCE_MAX = 2.0` e `FRICTION_MAX = 1.0`, **ao lado das colunas**, com
  `material_coerce` — a porta única que os DOIS lados usam. ⚠️ *Quem declara e quem consome têm de
  coagir pelo mesmo tecto, senão o cartão deixa autorar um número que o solver corta e o slider
  pára de responder a meio do curso.*
- O `max` do hint de `Bounciness` **lê** `BOUNCE_MAX`. Um literal ali seria o segundo sítio onde a
  faixa vive — e a caixa de texto de um param sem `ParamHardMax` é capada exactamente por esse
  `max`, logo ele é a faixa inteira que o artista alcança, e não só o curso do slider.
- ⚠️ **O atrito NÃO foi alargado, e ele é o controlo da mudança:** um coeficiente de Coulomb acima
  de `1` não compra nada (o impulso tangencial já está limitado ao que a normal aguenta). Sem essa
  metade, «alargar o tecto do material» teria alargado os dois e o gate ficaria verde sobre uma
  faixa que ninguém pediu.
- ⚠️ **O `restitution` do OBSTÁCULO (`sim.collide`) fica em `0..1`** — é outro controlo, o dono não
  o pediu, e o par toma o MAIOR dos dois, logo a peça alcança `2` contra qualquer parede.

Medido na cena: `Bounciness 0 → 0,200` (pousada) · `1 → 0,510` · **`2 → 3,201`**. ⚠️ No tecto a bola
**sai pelo cimo da janela** antes de voltar, e o anúncio diz isso — *é o que `2` quer dizer*.

⚠️ **Dois gates antigos reprovaram, e era o que tinham de fazer:** eles afirmavam *«e nunca passa de
1»*. A faixa mudou, e eles disseram-no alto em vez de a deixarem passar em silêncio.

### §7.9 — O `Bounce` do OBSTÁCULO chega ao mesmo tecto, e a §7.8 estava errada sobre ele

⛔⛔ **A última linha da §7.8 é uma premissa minha que a medição derrubou.** Ela dizia:

> *«O `restitution` do OBSTÁCULO (`sim.collide`) fica em `0..1` — é outro controlo, o dono não o
> pediu, e o par toma o MAIOR dos dois, logo a peça alcança `2` contra qualquer parede.»*

As duas metades da frase são verdade e a conclusão não. O par toma o maior, **e os dois números
entram no MESMO campo** (`resposta::Resposta::salto`): não são dois controlos, são o mesmo número
com duas portas de autoria. Manter `1` numa delas é o mesmo deslizante a parar a meio do curso ou
não **consoante o lado do par em que o artista lhe toca** — e o lado do obstáculo é o que o
tutorial do ciclo 5 ensina primeiro (capítulo 2).

⚠️ **E o corte estava em DOIS sítios, não no hint:** o `eval` cortava (`clamp(0.0, 1.0)`) e o
cabeçalho do módulo declarava a lei por escrito (*«Restitution ≤ 1»*). Um deslizante alargado com o
`eval` a cortar teria dado exactamente o sintoma que o dono já tinha reportado uma vez.

**Medido** (sonda `probe_the_obstacle_bounce_above_one`, nas condições da tabela do §7.8 — queda de
`0,75`, `g = 4`, `dt = 1/60`, **sem laço**, 40 s):

| `Bounce` do obstáculo | 0,00 | 0,50 | 1,00 | 1,25 | 1,50 | 2,00 | 3,00 |
|---|---|---|---|---|---|---|---|
| pico alcançado | 0,750 | 0,750 | 0,750 | **39,40** | 95,83 | 194,05 | 550,58 |

⭐ **É a MESMA curva da tabela do §7.8** (`39,6` · `91,8` · `192,0` · `550,8`, medida pelo lado da
peça): as duas portas alimentam a mesma lei, e agora têm o mesmo tecto — `ph2d_nodegraph::attr::
BOUNCE_MAX`, lido pelo hint **e** pela coerção.

**O que mudou:** o `eval` passa a coagir por `material_coerce(_, BOUNCE_MAX)` (e o `friction` por
`FRICTION_MAX`, pela mesma porta), o hint lê a const, e o cabeçalho do módulo leva a tabela.
Gate `the_obstacle_bounce_reaches_the_same_ceiling_as_the_piece` — a régua é o **produto** (a peça
volta acima de onde partiu) e não a constante, com o controlo a `1,00` no mesmo teste.

### §7.10 — O ATRITO DE ROLAMENTO: a bola pára sozinha

A ponta que a §7.7 nomeava — *«com `angular_damping = 1` (o default) ela rola para sempre num chão
infinito. É um param, não uma lei»* — estava certa no diagnóstico e errada no preço: **é uma lei**,
e a razão é a mesma conta da §7.1.

⭐⭐⭐ **Porque o `Friction` não podia fazer isto.** O atrito tangencial opõe-se ao **deslize**, que
é a velocidade do PONTO de contacto (`v·t + ω·bt`). Numa bola a rolar sem derrapar esse número é
**exactamente zero** — não há nada a que ele se oponha. *A mesma identidade que faz o atrito ser a
única coisa capaz de rodar um círculo (§7.1) faz dele incapaz de o parar.* O que trava uma bola
real é a **deformação** do contacto, que a literatura modela como um binário limitado pela normal:

```text
  |τ| ≤ μr · jn · R      ⇒     jr = clamp( ω/invI,  ±μr·jn·R )
```

— a forma do `rollingResistance` do Box2D v3 e do `rolling_resistance` do Rapier. ⚠️ O `clamp` é
sobre o **momento angular que a peça tem**, logo o pior caso é parar a rotação neste tique: ⛔ ele
nunca a inverte, e é por isso que o tecto não tem divergência a temer.

⚠️ **Ele corre DEPOIS do tangencial e lê o `ω` que aquele acabou de escrever** — os dois escrevem a
mesma grandeza. Lido do `ω` de entrada, numa bola a derrapar (onde `ω = 0`) ele seria **inerte no
tique em que o atrito a põe a rodar**. Gate `the_rolling_reads_the_spin_the_friction_just_wrote`,
com a régua a `0,5` e não a `1,0`: **a `1,0` o cancelamento é exacto** (o tecto do rolamento vale
precisamente o momento que o atrito produziu) e um gate ali mediria uma igualdade acidental.

#### O tecto, MEDIDO

Sonda `probe_the_rolling_ball_stops` — bola já **a rolar** a `1 u/s`, chão plano, `μ = 1`, nas
constantes da `=115`:

| rolamento | 0,00 | 0,02 | 0,05 | 0,10 | 0,25 | 0,50 | 1,00 | **1,50** | 2,00 | 4,00 | 8,00 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| pára em (s) | ∞ | 18,57 | 7,43 | 3,72 | 1,50 | 0,78 | 0,43 | **0,33** | 0,33 | 0,33 | 0,33 |

⭐⭐ **A `1,50` a coluna satura, e o que a segura é OUTRA lei:** com o rolamento a matar todo o `ω`
em cada tique a bola passa a ser um bloco a derrapar, e aí quem a trava é o atrito de Coulomb —
`v/(μ·g) = 0,25 s` em teoria, `0,33` medidos (o contacto não acontece em todos os tiques). ⇒
`ROLLING_MAX = 1.5`, porque acima disso o deslizante devolve sempre o mesmo número.

#### A fronteira, e porque ela NÃO se combina por par

⚠️⚠️ **É a única das três propriedades que é da PEÇA e não do par.** O atrito e o salto são de
*duas superfícies a esfregar-se*; o rolamento modela a peça a **achatar-se** contra o que toca.
⛔ E a razão de produto é mais dura que a física: nenhum obstáculo declara rolamento, logo uma média
geométrica daria `0` em **toda cena que existe** — o artista arrastaria um controlo que não faz
nada, que é literalmente o report do §7.6.

⛔⛔ **E ela vive na moeda da VELOCIDADE** (peça × obstáculo). No contacto peça × peça a rotação é
uma correcção de POSIÇÃO (§7.4) e não há `ω` que travar — *uma bola que não tem como «rolar para
sempre» também não tem o que parar*. Isto está **declarado e gateado**, não esquecido:
`the_piece_against_piece_contact_ignores_rolling_bit_for_bit` compara a saída com o rolamento a `0`
e no tecto e exige **bits iguais**. ⇒ no dia em que aquele solver ganhar velocidade angular (o §7.7
nomeia-o), ele reprova e obriga a decidir.

#### Medido na cena

⭐⭐⭐ **A bola TRAVA na própria rampa, e o número não foi escolhido:** uma bola prende num plano
inclinado quando `μr ≥ tan θ`, e a rampa da `=115` tem `12°` ⇒ `0,213`. Medido (sonda
`probe_the_rolling_ball_on_the_ramp`, 2 s, distância percorrida):

| `Rolling Friction` | 0,00 | 0,10 | 0,20 | **0,25** | 0,30 | 0,50 | 1,00 | 1,50 |
|---|---|---|---|---|---|---|---|---|
| desceu | 1,104 | 0,620 | 0,170 | **0,087** | 0,087 | 0,086 | 0,086 | 0,086 |

— ela prende entre `0,20` e `0,25`, **exactamente onde a conta manda**, e daí para cima nada muda.
⚠️ Os `0,086` que sobram não são descida: é a folga de `0,02` com que a bola nasce pousada.
Gate `the_rolling_friction_locks_the_ball_on_the_ramp`.

**O que mudou:** coluna `rolling` + `ROLLING_MAX` (`ph2d_nodegraph::attr`) · `Material::rolar` e
`atrito::rolamento` (`ph2d-contact`) · o impulso em `resposta::respond` · o param `Rolling Friction`
no cartão da forma (hint, grupo `Collision`, gate acima do `Collide`, coluna escrita pelo
`collider.rs`) · o passo 7 do anúncio da `=115`.

### §7.11 — O tutorial do ciclo 5 passa a ensinar o que o app faz

⛔⛔ **O PDF do ciclo 5 ensinava o colisor ANTIGO.** Ele foi aprovado em 10/09 e o capítulo 2 dele
manda *«clique no cartão `Collider` e arraste a caixa»* — o colisor do MUNDO. Nada do colisor na
forma, do material ou do salto ao dobro estava lá, e a tabela derivada dizia `Friction 0 a 1` sobre
uma secção que hoje tem três linhas. *Quando um comportamento muda, o smoke que o demonstra é o
último sítio a ser lembrado e o primeiro que o dono lê* (`CLAUDE.md` §5.0).

- **Capítulo 6 novo** — *«As peças colidem SOZINHAS, e de que elas são feitas»*, sobre a `=115`,
  com o comando próprio e seis passos numerados; o antigo 6 passa a 7.
- ⛔ **O comando deixa de ser `--release`** — `161 s` por correcção contra `3 s` do perfil `smoke`
  (DIRETRIZ §6.7). *Um tutorial é o sítio onde o dono COPIA o comando.* Há gate a proibir a palavra.
- **A tabela derivada ganha o cartão da FORMA**, com o `Collide` **ligado** (`derive_ligado`): é o
  estado em que o capítulo o põe, e a nota `Só noutro modo` não carrega a FAIXA de cada controlo,
  que é o que o leitor ali vai buscar.
- ⛔⛔ **E o gerador era cego a uma família inteira de gates:** a nota lia só os `ParamGate`
  (enum) e **nunca** os `ParamGateAbove` (limiar), que shipou depois dela — a secção `Collision`
  inteira não aparecia nem na tabela nem na nota. *Uma nota que promete «o que existe e o modo de
  omissão não acende» e lê metade dos gates é pior que a ausência dela: ela afirma que não há mais
  nada.* Curado, e as notas passam a ser **por param** (o `Collider Radius` tem um gate de cada
  família e saía escrito duas vezes, a prometer coisas diferentes).
- ⚠️ **Regenerar as tabelas dos ciclos 1 a 3 mudou-as** — elas estavam paradas antes da cura
  *«uma unidade, uma palavra»* e ainda imprimiam `graus` onde o app diz `deg`. Os cinco PDFs foram
  reconstruídos.
- O gate `every_row_the_sim_tutorial_names_is_on_the_card` passa a correr **sobre as DUAS cenas**,
  e procura os cartões pelo nome que o capítulo manda clicar (`Friction 1: ROLA`), que é o da
  legenda e não `Shape`.


---

## §8 — ⛔⛔⛔ O TREMOR DA PILHA: diagnóstico FECHADO, cura CONSTRUÍDA, MEDIDA e **REPROVADA** (4.º e 5.º reports, 2026-09-15)

> **4.º report:** *«as shapes que ficam embaixo no centro vibram muito após serem apertadas pelas
> shapes acima»* — foto da `=114`, três setas no fundo do monte.
>
> **5.º report, sobre a cura:** *«piorou. os mesmos blocos rotacionam como se fossem círculos.»*
>
> ⇒ **o produto está como estava** (a lei do §6 shipa, ao bit), o defeito continua **ABERTO**, e o
> que esta secção entrega é o mecanismo, as duas curas refutadas com número, e as duas réguas novas.

### §8.1 — ⛔ Nenhuma régua desta cena o via, e a cegueira é estrutural

A régua da `=114` é o **vão típico** (§3-bis), e um vão é uma FOTOGRAFIA: *uma pilha a tremer e uma
pilha parada com o mesmo espaçamento leem-se exactamente iguais nela*. Os quatro gates da cena
estavam verdes sobre o defeito.

### §8.2 — O diagnóstico, na ordem em que as hipóteses caíram

| # | pergunta | medida | veredito |
|---|---|---|---|
| 1 | reproduz? | BAIXO `p50 0,0128` · pior `0,0709` · CIMA `p50 0,0071` (deslocamento por tique / lado) | ✅ e as três piores são as do fundo-centro |
| 2 | vibração ou deslize? | `deriva = |ΣΔp|/Σ|Δp|` das duas piores: **`0,007`** e **`0,030`** | **oscilação pura**, no sítio |
| 3 | faltam varreduras? | `VARREDURAS 8 → 32`: pior `0,0709 → 0,0792` | ⛔ **REFUTADA** — mais iterações PIORAM |
| 4 | é a rotação? | `Lock Rotation`: pior `0,0709 → 0,0109` | ✅ **6,5×**, é a metade angular |
| 5 | quem vibra? | só as que oscilam: na **taça** `p50 0,0204`/pior `0,0709`; em **peças** `p50 0,0079`/pior `0,0128` | **5,5×** — é o contacto com o MUNDO |
| 6 | o sinal | `rot` da peça 2 alterna **`+3,98° / −2,25°`**, **17 trocas de sinal em 18 passos** | **ciclo de 2 tiques** |

⚠️ **A hipótese 3 é a que vale guardar:** *um solver que não converge com `4×` as iterações não está
mal afinado — ele não está a resolver um ponto fixo.*

### §8.3 — A leitura que parecia certa, e que o produto derrubou

O cabeçalho do [`resposta.rs`](../../crates/ph2d-node-sim-collide/src/resposta.rs) declara a moeda
deste nó desde o §7: *«a rotação daqui é VELOCIDADE ANGULAR (`spin`), não um empurrão de ângulo»*.
A metade do ATRITO obedece; a metade da NORMAL, escrita no §6, devolvia um **empurrão de ângulo em
graus** somado ao `rot`, a partir da **penetração** — que a gravidade recria a cada tique, e que
nada amortece porque um empurrão de ângulo não tem memória. O `respond` tem a guarda que impede
isto na metade linear, com o nome do fenómeno ao lado (*«the classic collider jitter»*), e a metade
angular nasceu depois e não a herdou.

⇒ a cura construída foi pôr as duas metades na mesma moeda: penetração devolvida INTEIRA e o
binário a sair pelo impulso normal, em `spin`.

### §8.4 — ⛔⛔⛔ E ela FALHA, porque aquele termo fazia DUAS coisas

| lei | balanço por tique (pior) | giro **LÍQUIDO** em `0,9 s` (pior) | veredito |
|---|---:|---:|---|
| **o que shipa** (empurrão de ângulo) | `4,16°` | **`24,30°`** | o dono vê **tremor** |
| v2 — impulso lido no CENTRO | `0,87°` | `46,70°` | ⛔ *«rotacionam como se fossem círculos»* |
| v3 — impulso lido no PONTO (auto-corrector, `v·n + ω·bn`) | `1,00°` | **`51,08°`** | ⛔ **pior ainda** |

⭐⭐⭐ **O termo que eu tirei não era só o zumbido — era a única coisa que SEGURAVA o ângulo da
peça.** Uma projecção de posição a partir da penetração é um **restaurador**: sempre que a caixa
saía da pose de repouso, ela era rodada de volta. Mal amortecido, ele zumbe; ausente, **nada** opõe
o giro que o atrito injecta, e com o `angular_damping` a nascer em `1` (sem arrasto) e as peças a
serem **gelo entre si** (`Material::LISO`, `μ = 0` — a cena não escreve material) esse giro
acumula-se sem fim. *Foi por isso que a v3 não curou: tornar o impulso auto-corrector limita o que
ele INJECTA e não repõe o que ele deixou de SEGURAR.*

⚠️ **E um impulso normal é uma restrição de UM LADO SÓ:** ele empurra, nunca puxa. Um contacto de
**um ponto** não pode, por construção, produzir um binário restaurador — para segurar uma
orientação é preciso um **BINÁRIO**, logo **dois** pontos de apoio.

### §8.5 — ⇒ A cura de fundo tem nome, e nenhuma das duas metades é afinação

1. ⭐ **O MANIFESTO de contacto com DOIS pontos.** Uma caixa pousada de chapa toca por um SEGMENTO,
   e o `ph2d-contact` reduz esse segmento ao **meio** dele (§ do cabeçalho: escolher um vértice
   *«daria binário a uma pilha parada — ela tombava sozinha»*). O meio dá binário **zero** quando
   está direita — certo — **e zero de resistência a inclinar-se** — que é o buraco. Resolver as
   duas pontas do trecho recortado dá as duas coisas, e é o que todo motor de corpo rígido faz;
   é isso que torna uma pilha de caixas estável.
2. ⭐ **OU o contacto com o mundo entra no MESMO laço de varreduras do contacto entre peças.** Hoje
   o `sim.step` resolve peça × peça (8 varreduras, um ponto fixo) e o `sim.collide` resolve
   peça × mundo **uma vez, a jusante**, e nenhum sabe o que o outro fez — que é exactamente por que
   só as peças do fundo (as únicas sujeitas aos dois) zumbem.

⛔ **O que NÃO é cura, medido:** mais varreduras (§8.2 nº 3) · trocar a moeda da metade normal
(§8.4, duas variantes) · atrito entre peças (`μ = 0,6` devolve `p50 0,0107`/pior `0,0176`, o mesmo
sítio da lei que shipa).

### §8.6 — O que FICA no repo, com o produto intocado

- **`the_pile_settles_instead_of_buzzing`** — a régua do 4.º report, **`#[ignore]` com o vermelho
  DECLARADO**: ela reprova hoje, nomeando as cinco peças do fundo. A barra `1,0°/tique` sai do vale
  entre os dois lados medidos (`… 0,38 · 1,24 · 2,53 · 3,50 · 3,96 · 4,16°` contra `0,87°`).
  ⛔ *Um `#[ignore]` que esconde um vermelho conhecido é uma dívida com endereço; um gate apagado é
  uma dívida sem nenhum.*
- **`the_pile_does_not_start_spinning_like_a_ball`** — ⭐ a catraca que o 5.º report comprou, e ela
  está **VERDE**: nenhuma peça pode girar mais de `29°` líquidos em `0,9 s` (a lei que shipa entrega
  `24,30°`, com `20 %` de folga). ⚠️⚠️ **É a régua que faltava:** tremer e girar leem-se **iguais**
  num `|Δrot|` por tique, e sem ela *uma cura que melhora a grandeza medida e piora a que ninguém
  mede lê-se como uma vitória* — foi exactamente o que aconteceu.
- As sondas da §8.2 (`probe_o_que_ainda_treme` · `probe_a_direcao_do_tremor` ·
  `probe_quem_vibra_toca_a_taca` · `probe_o_angulo_tique_a_tique` · `probe_gira_ou_treme` ·
  `probe_o_atrito_entre_pecas`), em
  [`motion_state_pilha_demo_tremor.rs`](../../crates/ph2d-app-motion/src/motion_state_pilha_demo_tremor.rs).

### §8.7 — ⛔⛔⛔ A ORDEM DO DONO era o MANIFESTO DE DOIS PONTOS, e a medição matou-a ANTES da primeira linha

> *«atacar o encosto de dois pontos agora»* (2026-09-15).

**Um segundo ponto de apoio só existe onde há um TRECHO, e um trecho só existe entre duas faces
quase paralelas.** Entre uma QUINA e uma face o contacto é um ponto **por geometria**, e nenhuma lei
o desdobra. Medido no monte assente da `=114` (`probe_quantos_contactos_sao_face_a_face`), sobre os
**28** contactos vivos:

| desalinho dos eixos (módulo 90°) | contactos |
|---|---|
| `< 5°` (face com face) | **`0` = 0 %** |
| `< 15°` | `2` = 7 % |
| p10 / p50 / p90 | `15,8°` / **`39,6°`** / `44,9°` |

⇒ **o manifesto não tem onde agir nesta cena.** ⚠️ E é PIOR que o acaso: um monte com desalinho
uniforme `0..45°` daria `11 %` abaixo de `5°`, e este dá **zero** — *uma pilha de quadrados soltos
assenta em quinas, não em faces.* Construí-lo seria curar um caso que esta cena não tem.

### §8.8 — ⛔⛔ E o PISO DE RUÍDO, que faltava a tudo o que veio antes

**Um monte de 25 quadrados a cair é CAÓTICO.** Deslocando só o berço por `±0,003` (a lei intocada,
7 realizações — `probe_o_ruido_entre_realizacoes`):

| grandeza | faixa | p50 | amplitude |
|---|---|---|---|
| balanço pior (°/tique) | `3,79 .. 5,69` | `4,87` | `1,5×` |
| giro líquido pior (°/0,9 s) | `24,30 .. 28,36` | `25,97` | `1,2×` |

⚠️⚠️ **Sem este número, toda varredura de constante lê RUÍDO como TENDÊNCIA** — é a armadilha que a
`line/quadextract` pagou em cinco realizações da mesma escultura. Toda medição desta secção passa a
correr as 7.

### §8.9 — ⛔⛔⛔ A SUB-RELAXAÇÃO é uma TROCA, e é a MESMA troca: o termo é o restaurador E o zumbido

Multiplicar o empurrão de ângulo por `α` (7 realizações por célula):

| `α` | balanço pior `p50` | giro líquido `p50` |
|---|---:|---:|
| **`1,0`** (o que shipa) | `4,87` | **`25,97`** |
| `0,5` | `1,81` | `35,26` |
| `0,3` | **`0,85`** | `45,00` |
| `0,15` | `0,92` | `44,34` |
| `0,05` | `0,93` | `47,70` |

⭐⭐⭐ **Monótona nas duas colunas, e `α → 0` é exactamente a cura que o dono reprovou** (§8.4):
*enfraquecer o termo tira o zumbido e solta o giro, porque é o MESMO termo a fazer as duas coisas.*
⇒ a família «afinar o ganho» está **fechada por medição**; não há `α` que compre as duas.

### §8.10 — ⇒ O que SOBRA tem UM nome, e é estrutural

O que falta a um ganho que oscila não é ganho **menor**, é **amortecimento** — um termo proporcional
à TAXA a que o ângulo está a mudar. E ele é inexprimível onde está:

⛔⛔ **o `sim.collide` é `Effect::Pure`** (`lib.rs`), logo **não tem memória**: ele não sabe onde a
peça estava no tique anterior, e sem isso não há taxa que se possa opor. *A metade do atrito
consegue amortecer porque lê o `spin`, que é uma velocidade que OUTRO nó guarda; a metade da normal
corrige uma POSIÇÃO, e a posição anterior ninguém lha dá.*

⇒ **a cura é mudar de DONO:** o contacto com o mundo tem de ser resolvido por quem tem estado — o
`sim.step`, dentro do mesmo laço de varreduras do contacto entre peças. Isso também dissolve a causa
do §8.2 nº 5 (os dois solvers a disputar a mesma peça, que é por que só o fundo zumbe), e é **uma
wave com espec própria**: o `sim.step` passaria a precisar da lista de obstáculos, que hoje vive em
nós a jusante dele.
