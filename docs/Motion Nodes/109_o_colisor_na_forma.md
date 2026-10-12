# 109 — O COLISOR NA FORMA: as peças colidem SOZINHAS

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/109_o_colisor_na_forma.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Mais varreduras do solver (`VARREDURAS 8 → 32`) contra o tremor da pilha | REFUTADA: o pior vai de `0,0709` a `0,0792` — mais iterações PIORAM | vivo §8.2 |
| Impulso lido no CENTRO (v2) ou no PONTO (v3, auto-corrector) | O giro líquido sobe de `24,30°` (o que shipa) para `46,70°` / `51,08°`: «rotacionam como se fossem círculos» | vivo §8.4 |
| Sub-relaxação `α` do empurrão de ângulo | É uma TROCA: o termo é o restaurador E o zumbido | vivo §8.9 |
| «Mudar o contacto de dono» (projecções alternadas iteradas) | Ordenado, testado no grafo sem cirurgia e REFUTADO | vivo §8.11 |
| Alavancas de produto: atrito das peças e arrasto angular | Inertes (e uma delas prova o teorema) | vivo §8.12 |
| Manifesto de dois pontos (ordem do dono) | A medição matou-a antes da primeira linha — e a correcção posterior do doc 111 §5.10 REFUTA esta recusa | vivo §8.7 |
| `Collider Fit` e `Collider Scale` no cartão | SAÍRAM (nunca chegaram ao `main`); ficam `Collider Shape`/`Width`/`Height`/`Radius` como multiplicadores | [arquivo](../archive/motion-nodes-2026-10-12/109_o_colisor_na_forma.md) (tabela do cartão `Collision`) |

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

### §6.2 — Os dois botões novos no cartão (secção «Collision»)

| param | o que faz | default |
|---|---|---|
| **`Show Collider`** | pinta o contorno do colisor de cada peça desta forma, por cima da arte | **ligado** — quem liga o `Collide` quer ver o que declarou |
| **`Lock Rotation`** | escreve `inv_inertia = 0`: as peças desta forma não rodam no contacto | **desligado** — o dono pediu a rotação destravada |

⚠️ Os dois só aparecem com o `Collide` ligado, e nenhum entra na chave de conteúdo da geometria.

### §6.5 — Aberto

- ⏳ **Velocidade angular** (uma peça que continua a girar depois do toque) e **atrito angular**.
- ⏳ **O contorno exacto** (casco convexo), §5.5.
- ⏳ **W4** (o `motion.integrate`) e ⏸️ **W5** (o dispositivo).

---

## §7 — A peça tem MATERIAL, e é o atrito que roda um círculo (3.º report do dono, 2026-09-13)

> *«Os círculos não rotacionam com a colisão, talvez por falta de atrito. Precisamos de parâmetros
> do material.»*

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

### §7.7 — Aberto, com o mecanismo

- ⏳ **O kernel de GPU do `sim.collide` não conhece nem a forma declarada nem o material** — ele
  implementa `contact` + o sangramento. É o **mesmo** vão que o §5 já abriu (a caixa declarada
  também não chega lá), e não um vão novo: quem o fechar fecha os dois de uma vez.
- ✅ **A rotação do contacto peça×peça continua sem velocidade angular** (§6) — **FECHADO em
  2026-09-16**, e a wave está no [doc 111 §9](111_o_motor_de_contacto_com_memoria.md). A redacção
  original está acima e era verdade: *«ela é projecção de posição, e uma peça roda enquanto toca»*,
  com as duas metades do app a não dizerem a mesma coisa (o `sim.collide` já dava `spin` de
  verdade). ⭐ Hoje o impulso do par escreve `spin`, e **a projecção de posição deixou de rodar** —
  duas leis de rotação a somar-se era exactamente o que partia a pilha na 1.ª tentativa. ⚠️ O preço
  declarado: uma caixa em penetração pura **com velocidade zero** já não roda; sob gravidade isso é
  invisível.
- ⏳ **Entre CAIXAS o `Friction` quase não pesa** (tabela acima): o encaixe geométrico de duas faces
  planas já impede o deslize, e o que sobra para o atrito é pouco. Não está errado — está **medido e
  nomeado**, e quem quiser mudá-lo tem de dizer que recurso é que o limita.
- ✅ **Não há atrito de ROLAMENTO** — **FECHADO na §7.10.** A redacção original dizia: *«(o que faz
  uma bola a rolar parar sozinha num plano): com `angular_damping = 1` (o default) ela rola para
  sempre num chão infinito. **É um param, não uma lei**»* — ⚠️ e essa última metade estava **errada**:
  o `Friction` não podia fazê-lo por construção (o deslize de uma bola a rolar é exactamente zero),
  logo era preciso um impulso novo. *O diagnóstico estava certo e o preço não.*

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

### §8.4 — ⛔⛔⛔ E ela FALHA, porque aquele termo fazia DUAS coisas

| lei | balanço por tique (pior) | giro **LÍQUIDO** em `0,9 s` (pior) | veredito |
|---|---:|---:|---|
| **o que shipa** (empurrão de ângulo) | `4,16°` | **`24,30°`** | o dono vê **tremor** |
| v2 — impulso lido no CENTRO | `0,87°` | `46,70°` | ⛔ *«rotacionam como se fossem círculos»* |
| v3 — impulso lido no PONTO (auto-corrector, `v·n + ω·bn`) | `1,00°` | **`51,08°`** | ⛔ **pior ainda** |

⭐⭐⭐ **O termo que eu tirei não era só o zumbido — era a única coisa que SEGURAVA o ângulo da
peça.** Uma projecção de posição a partir da penetração é um **restaurador**: sempre que a caixa
saía da pose de repouso, ela era rodada de volta. Mal amortecido, ele zumbe; ausente, **nada** opõe
o giro que o atrito injecta, e com o `angular_damping` a nascer em `1` (sem arrasto) esse giro
acumula-se sem fim.

> ⛔⛔ **CORRECÇÃO (2026-09-15, [doc 111 §2.1](111_o_motor_de_contacto_com_memoria.md)):** este
> parágrafo dizia *«as peças são gelo entre si — a cena não escreve material»*. **Falso.** Lida do
> stream COZIDO, a coluna `friction` está **PRESENTE** com `0,500` (o default do Rapier, que o
> `source.shape` escreve sempre que o `Collide` está ligado). A tabela acima não muda — a frase que
> estava ao lado dela, sim. *É a terceira ausência que esta caça afirmou sem olhar para a coluna.* *Foi por isso que a v3 não curou: tornar o impulso auto-corrector limita o que
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

> ### ⛔⛔⛔ CORRECÇÃO (2026-09-15, doc 111 §5.10) — **ESTA RECUSA ESTÁ REFUTADA. A ordem do dono
> estava certa e fui eu que a matei com uma medição errada.**
>
> A tabela acima tem **dois** defeitos, e qualquer um deles a invalida:
>
> **1. Ela foi medida pela PORTA ERRADA.** A sonda chamava `pump.cook.cook(..)` **directamente**,
> que coze o grafo uma vez por tique e **salta os sub-passos** — o mesmo furo que invalidou uma
> tabela inteira no doc 111 §5.8.1. Corrigida para marchar pelo PUMP e com o monte a assentar
> (`substeps = 8`), ela lê **`9 %` face-com-face e `35 %` abaixo de `15°`**, com a `p10` do
> desalinho a cair de `15,8°` para `6,0°`: deixou de ser *pior que o acaso* e passou a estar **no**
> acaso.
>
> **2. E o defeito de fundo é mais fundo: o censo conta o monte ASSENTE, isto é, DEPOIS de cada
> apoio face-a-face já ter tombado.** Ele conta **SOBREVIVENTES**. Medido ao longo do tempo
> (`probe_os_apoios_face_a_face_no_tempo`):
>
> | substeps | pico de apoios face-a-face | média | **no fim** |
> |---|---|---|---|
> | 1 | **4** | `0,3` | **0** |
> | 8 | **8** | `2,4` | `2` |
>
> ⇒ **o `0 %` que matou o manifesto não dizia «esta cena não tem o caso»: dizia «esta cena DESTRÓI
> o caso».** *Um censo tirado depois do evento mede o resultado do defeito e lê-se como a ausência
> da precondição dele* — e o que o destrói é precisamente o defeito que o manifesto curaria.
>
> **E o mecanismo está medido** (doc 111 §5.10): uma caixa pousada **de face** sobre outra é um
> equilíbrio **INSTÁVEL** neste solver — o desalinho cresce `×1,4` por tique desde `1,7°` até
> travar nos `45°` (quina contra face), que é a configuração que o solver consegue segurar. *Um
> contacto de UM ponto não resiste a binário nenhum*, e é exactamente isso que o segundo ponto de
> apoio existe para dar.

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

### §8.11 — ⛔⛔⛔ «Mudar o contacto de dono» foi ORDENADO, TESTADO SEM CIRURGIA, e REFUTADO

> *«muda o contacto de dono»* (2026-09-15) — a hipótese do §8.10: o zumbido é um ciclo entre **dois
> conjuntos de restrições** resolvidos uma vez cada, em sequência, e projecções alternadas convergem
> *se a alternância for ITERADA*.

⭐ **Ela testa-se no GRAFO, sem uma linha de motor:** `passo → taça → passo → taça`. O segundo
`sim.step` lê `dt = 0` (o primeiro já escreveu o `sim_t` do tique), logo **não integra** — só volta a
separar as peças depois de a taça as mexer. Medido em 5 realizações por célula:

| voltas de alternância | balanço pior (°/tique) | giro líquido pior (°) |
|---|---|---|
| **1** (o que shipa) | `3,79 .. 5,69` | **`24,30 .. 28,36`** |
| 2 | `1,14 .. 6,31` | `32,40 .. 50,62` |
| 4 | `0,53 .. 7,05` | `27,49 .. 53,85` |

⛔ **Não converge.** A faixa do balanço **ALARGA** — em algumas quedas melhora `7×`, noutras piora —
e o giro sobe até `53,85°`. *Cada volta a mais injecta mais rotação, porque a correcção angular sai
da PENETRAÇÃO e a penetração é recriada; ela não é uma projecção sobre um conjunto de restrições, é
um ganho aplicado repetidamente.* ⇒ **a cirurgia de meio dia teria sido paga para nada, e o que a
poupou foi exprimir a hipótese no grafo.**

### §8.12 — ⛔⛔ As duas alavancas de PRODUTO são inertes — e UMA delas prova o teorema

| atrito das peças | arrasto angular | balanço pior | giro líquido pior |
|---|---|---|---|
| `0,0` | `1,0` | `3,49 .. 7,58` | `16,31 .. 40,46` |
| `0,6` | `1,0` | `3,83 .. 7,79` | `24,73 .. 38,15` |
| `0,0` | **`0,9`** | `3,49 .. 7,58` | `16,31 .. 40,46` |
| `0,0` | **`0,6`** | `3,49 .. 7,58` | `16,31 .. 40,46` |

⭐⭐⭐ **As linhas 1, 3 e 4 são IDÊNTICAS AO BIT: o `angular_damping` não muda NADA.** E o motivo é o
teorema desta secção inteira, agora medido em vez de lido: **ele amortece a coluna `spin`, e a
rotação do contacto é escrita no `rot`.** ⇒ *o único knob do módulo feito para travar uma rotação
não alcança esta rotação — não há, em todo o sistema, nada que a possa amortecer.*

⚠️ E o atrito entre peças também não (`3,83 .. 7,79` contra `3,49 .. 7,58`, dentro do ruído).

### §8.13 — ⛔ E a CÉLULA QUE NINGUÉM TINHA CORRIDO também cai

O §8.4 mediu a moeda `spin` com o `angular_damping` **no default `1`**, que é *sem arrasto* — as duas
metades foram medidas sozinhas e recusadas, e a combinação nunca foi corrida (a armadilha da célula
`(1,1)` que a `line/quadextract` registou). Corrida agora:

| lei | atrito | arrasto | balanço pior | giro líquido pior |
|---|---|---|---|---|
| `spin` (§8.4) | `0,6` | `1,0` | `0,77 .. 1,10` | `19,11 .. 56,05` |
| `spin` | `0,0` | `0,6` | `1,33 .. 2,05` | `27,04 .. 71,54` |
| `spin` | `0,6` | `0,6` | `0,85 .. 1,14` | `39,58 .. 45,75` |

⛔ **O arrasto não alcança o giro porque os contactos o RE-INJECTAM mais depressa do que ele sangra.**
Nenhuma combinação devolve o giro à banda de `24 .. 28` que a lei que shipa entrega.

### §8.14 — ⭐⭐⭐ O teorema, e o que a obra seguinte tem de SER

**Oito cadeias de cura foram construídas ou testadas e medidas, e as oito trocam uma grandeza pela
outra:**

| # | cura | veredito |
|---|---|---|
| 1 | mais varreduras peça × peça (`8 → 32`) | pior |
| 2 | a moeda `spin`, impulso lido no centro | giro `24 → 47`, **reprovada pelo dono** |
| 3 | a moeda `spin`, impulso lido no PONTO (auto-corrector) | giro `24 → 51` |
| 4 | manifesto de DOIS PONTOS (ordem do dono) | `0 %` de contactos face-a-face: **sem sujeito** |
| 5 | sub-relaxação `α` | monótona nas duas colunas: nenhum `α` compra as duas |
| 6 | alternância ITERADA (ordem do dono) | não converge; giro `24 → 54` |
| 7 | atrito entre peças · arrasto angular | **inertes** (o arrasto não alcança a coluna) |
| 8 | a célula `(spin, arrasto)` | giro nunca volta à banda |

⇒ ⭐⭐⭐ **O zumbido e o rodopio são a MESMA grandeza neste desenho**, e nenhuma alavanca deste
solver os separa. A razão é de CLASSE: **uma correcção de posição recalculada do zero a cada tique
não tem memória do impulso que aplicou no tique anterior.** O que separa as duas coisas nos motores
que assentam pilhas é o que este não tem — **impulsos sequenciais com `λ` ACUMULADO por contacto
persistente** (*warm starting*), mais um critério de **repouso**. Com `λ` acumulado a restrição
sustenta a orientação sem re-excitar, que é exactamente a combinação que as oito tentativas
procuraram e nenhuma alcançou.

⛔ **Isso não é uma wave deste doc: é um solver de contacto novo**, e a decisão é do dono.
