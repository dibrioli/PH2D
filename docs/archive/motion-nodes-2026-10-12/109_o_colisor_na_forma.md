# ARQUIVO — 109_o_colisor_na_forma.md (história, 479 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`109_o_colisor_na_forma.md`](../../Motion%20Nodes/109_o_colisor_na_forma.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-206,282-307,335-343,415-427,459-540,588-609,808-837,851-892,910-1083` do original.
>
> ⚠️ **A única alteração ao corpo:** 4 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

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
([`ph2d-contact/src/tests.rs`](../../../crates/ph2d-contact/src/tests.rs)).

⚠️ **Zero em aritmética exacta, RUÍDO em `f32`.** O braço é `ponto − centro`, e numa peça longe da
origem isso subtrai dois números muito maiores do que a diferença. Medido na cena `=115` (bola a
`x ≈ 1,35`, 2 s de rampa): **`0,0000075°` acumulados**. ⛔ *Uma barra de zero-ao-bit sobre isto
mediria a aritmética e não a lei* — os gates de cena usam `1e-3°` com este número escrito ao lado.

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

> ✅ **ACTUALIZAÇÃO 2026-09-16:** a fronteira que esta secção declara (*«só peça × obstáculo; entre
> peças não há velocidade angular a travar»*) **fechou** — ver o
> [doc 111 §10](../../Motion%20Nodes/111_o_motor_de_contacto_com_memoria.md). A lei entre peças é a desta secção, pela
> mesma porta, e as duas metades do app param um disco no mesmo décimo de segundo.

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

### §8.3 — A leitura que parecia certa, e que o produto derrubou

O cabeçalho do [`resposta.rs`](../../../crates/ph2d-node-sim-collide/src/resposta.rs) declara a moeda
deste nó desde o §7: *«a rotação daqui é VELOCIDADE ANGULAR (`spin`), não um empurrão de ângulo»*.
A metade do ATRITO obedece; a metade da NORMAL, escrita no §6, devolvia um **empurrão de ângulo em
graus** somado ao `rot`, a partir da **penetração** — que a gravidade recria a cada tique, e que
nada amortece porque um empurrão de ângulo não tem memória. O `respond` tem a guarda que impede
isto na metade linear, com o nome do fenómeno ao lado (*«the classic collider jitter»*), e a metade
angular nasceu depois e não a herdou.

⇒ a cura construída foi pôr as duas metades na mesma moeda: penetração devolvida INTEIRA e o
binário a sair pelo impulso normal, em `spin`.

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
  [`motion_state_pilha_demo_tremor.rs`](../../../crates/ph2d-app-motion/src/motion_state_pilha_demo_tremor.rs).

