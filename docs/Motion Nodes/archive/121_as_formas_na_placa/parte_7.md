# ARQUIVO — parte_6.md (história, 362 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_6.md`](parte_6.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-345` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §9.20 — O CONTACTO DAS FORMAS PELO MOTOR DA CASA (`rapier2d`): o desenho (2026-10-06, escrito ANTES de construir)

Ordem do dono (06/10): *«usar o motor da casa»*. ⚠️ **A recusa do doc 111 §3** (*«o oráculo não pode virar o
motor»*, `6 400` caixas `54` ms) **fica vencida por medição e não por ordem:** a premissa dela era que a lei por
colunas correria no dispositivo; ela nunca correu, e na CPU custa `48`–`93` ms a `4 096` peças contra `1,5` do
rapier (abaixo). O tecto do módulo deixa de ser o do caminho lento — é a mesma lei do `CLAUDE.md` §0.0 que a recusa
invocava, agora a favor do rapier.

**Três perguntas do desenho, medidas ANTES no oráculo** ([`oraculo_rapier_pilha_quente_frio/`](../../ferramentas/oraculo_rapier_pilha_quente_frio):
a pilha da `=114`, `release`, `enhanced-determinism`, `2,95` s — o último instante antes do recomeço —, carga `1`–`2`):

| `k × k` caixas, `1` passo, `4` iterações | tique med · p95 (ms) | vizinho med | velocidade média | par mais junto (% do lado) |
|---|---|---|---|---|
| `1 024` mundo PERSISTENTE («quente») | **`0,39`** · `0,50` | `0,2163` | **`0,195`** | `96 %` |
| `1 024` mundo RECONSTRUÍDO a cada tique («frio») | `1,71` · `3,27` | `0,2102` | `0,629` | `61 %` |
| `1 024` quente, a taça FORA do mundo (a projecção do `sim.collide` depois do passo) | `0,28` · `0,68` | `0,2210` | **`1,619`** | **`8 %`** |
| `4 096` quente | **`1,54`** · `3,67` | `0,1922` | `1,011` | `47 %` |
| `4 096` frio | `14,9` · `21,8` | `0,1500` | `3,106` | `4 %` |
| `4 096` quente, taça fora | `1,04` · `3,59` | `0,1820` | `1,949` | `1 %` |
| `4 096` quente, `8` passos × `1` iteração (os sub-passos da zona) | `6,32` · `12,5` | `0,1970` | `0,746` | `72 %` |
| `1 024` · `4 096` · `16 384`: a CÓPIA do mundo (o que um ponto de recuo pagaria) | `0,25` · `1,26` · `6,8` | — | — | — |

⇒ (a) **o mundo é PERSISTENTE** — reconstruído a cada tique ele custa `10×` (`14,9` contra `1,5` ms a `4 096`) e a
pilha fica `3×` mais agitada: o ganho do motor É a memória (contactos persistentes, aquecimento). (b) **A taça vive
DENTRO do mundo como colisor fixo** — fora dele (projectada depois do passo, como hoje) o peso do monte não passa
pelo solver e a pilha fica `8×` mais agitada, com peças a afundar umas nas outras (`8 %`, `1 %` do lado).
(c) **Um passo do motor por tique** — os `8` sub-passos da zona (a cura do zumbido da NOSSA lei, doc 111 §5.8.2)
custam `4×` e não compram pilha: a cena passa a `1` sub-passo; o nó continua certo com qualquer número.

**O desenho:**

1. **Onde vive o mundo:** na MEMÓRIA DO NÓ que o `Cook` guarda por `(nó, escopo)` — uma porta nova, apendada
   (`EvalCtx::memo`/`guarda_memo`; o ADR-0039 congela `NodeOp`/`OpResolver`/`NodeManifest`, não o `EvalCtx`). ⭐ **E a
   memória viaja no `CookCheckpoint`**: o anel do recuo (ADR-0137) diz *«para um grafo fixo a sim é função pura do
   tique»*, e uma memória fora do ponto de recuo quebraria isso — um recuo seguido de Play mostraria OUTRA queda (a
   família *«o que o undo não fotografa»*). Os bytes do mundo entram no orçamento do anel (`approx_bytes`), e — porque
   copiar o mundo custa `~0,8` passo — um grafo com memória regista o ponto de recuo **a cada `8` tiques** (`MEMO_A_CADA`):
   `+10 %` amortizado, e um recuo refaz no máximo `7` tiques. Entre dois registos o mundo é mutado no lugar (o
   `Arc` é único).
2. **Sincronia com o stream, por `id`:** o stream continua a VERDADE do estado (`P`, `vel`, `rot`, `spin`, `size`, as
   colunas do colisor e do material); o mundo é a verdade da MEMÓRIA dos contactos. Por passo: corpos novos para ids
   novos, removidos os que morreram (`BTreeMap<id, corpo>`, ordem do stream — determinístico), e o estado do stream
   escrito no corpo só onde os bits mudaram (quem mexe numa peça entre dois passos — uma força, um `drive` — é ouvido
   sem acordar os outros). **Recomeço** (`Loop`) e qualquer stream que não continue o mundo (o `sim_t` ausente, ou
   diferente do instante em que o mundo parou) ⇒ mundo novo a partir do stream; uma reavaliação no MESMO instante com
   a MESMA entrada devolve a saída guardada (o nó não avança duas vezes).
3. **Uma integração só:** o `sim.step` faz a metade da VELOCIDADE como sempre (`accel`, `damping`, `max/min_speed`,
   `angular_damping` no `spin`); o rapier recebe essa velocidade, resolve os contactos e integra a POSIÇÃO e o ângulo
   (gravidade `0` no mundo — a gravidade é o `force.wind` da cena). Peças sem colisor válido não entram no mundo e
   integram como sempre. O `rot` sai ACUMULADO (o ângulo de antes mais o quanto o corpo rodou), não o do rapier
   (`(−π, π]`); sem rotação nenhuma a coluna não nasce (o gate do `Lock Rotation`).
4. **O mapeamento:** caixa (com `desvio` no referencial da peça) → `cuboid`; disco → `ball`; massa `1/inv_mass` e
   inércia `1/inv_inertia` pela MESMA porta de hoje (`ph2d_contact::inv_inercias`) como propriedades de massa
   explícitas; `inv_inertia = 0` (o `Lock Rotation`) → rotação travada; `inv_mass = 0` → corpo cinemático conduzido
   pelo stream; Friction → `GeometricMean` (a nossa `√(a·b)`, que o rapier `0.35` tem); Bounce → `Max`.
   **Rolling:** o rapier `0.35.3` NÃO tem resistência ao rolamento (lido no fonte: só o `ray_cast_vehicle_controller`) —
   a lei que o substitui é a NOSSA, a mesma porta (`ph2d_contact::atrito::rolamento`): depois do passo, por ponto de
   contacto de cada manifesto (o impulso normal resolvido pelo rapier e o braço da peça), o giro de cada peça é
   travado contra o próprio momento, com o próprio `μr`, sem nunca o inverter — medida contra
   `the_rolling_on_the_card_calms_the_pile`.
5. **A taça e os outros obstáculos (`sim.collide`)** entram no mundo por um aperto de mão em colunas (os systems não
   se chamam — ADR-0075): o `sim.collide` DECLARA o seu obstáculo em colunas com a chave do nó; o `sim.step` consome a
   declaração, põe o obstáculo no mundo como colisor fixo (plano → `halfspace`, disco → `ball`, caixa → `cuboid`,
   taça → polilinha fechada com a flecha `≤ 10⁻³` u) e devolve um RECIBO; o `sim.collide` que lê o recibo não projecta
   as peças que o mundo resolve (só escreve o `hit`, por detecção). Sem recibo — um `sim.collide` fora da zona, ou o
   1.º passo — ele projecta como sempre. Um `sim.collide` apagado deixa de declarar e o obstáculo sai do mundo no passo
   seguinte (o `sim.step` consome a declaração, nunca a deixa a circular). O salto aleatório por peça (`Randomness`)
   chega ao solver pela coluna da declaração.
6. **O que sai:** o `sim.step` deixa de chamar `ph2d_contact::separate`/`impulsos`. O `ph2d-contact` fica para quem
   ainda o chama (o `motion.collide`, o passe do sink, o `sim.collide` para a geometria do toque e as portas do
   colisor e do material); o que ficar sem chamador SAI com o código, e as portas de prova de 05/10
   (`PH2D_CONTACT_JACOBI`/`PH2D_CONTACT_CORES`) saem.

**Kill-criteria (os do prompt, escritos antes):** tique a `4 096` por taça `≤ 5` ms na CPU (`release`); a pilha de
`25` e a de `1 024` dentro da banda da ordem contra o produto de hoje (vizinho mediano `±5 %`, velocidade
`≤ 1,5×` + `0,01` u/s, sobreposição mais funda `≤` a de hoje) OU a diferença ao dono no smoke; os gates da pilha
verdes; determinismo: duas corridas iguais ao bit; ⭐ e dois novos, do desenho: **um recuo seguido de Play dá os bits
da 1.ª passagem**, e **a 2.ª volta do `Loop` dá os bits da 1.ª**.

**§9.20 — o resultado: CONSTRUÍDO** (`e78b2c096` · `594a859fd` · `e21fd3195`). O mundo de contacto é a crate
[`ph2d-contact-world`](../../../../crates/ph2d-contact-world/src/lib.rs); a memória do nó e o ponto de recuo são
`ph2d-nodegraph` (`cook_checkpoint.rs`: `Memo`, `EvalCtx::take_memo`/`keep_memo`) e `ph2d-eval-motion`
(`CheckpointRing::regista`, `MEMO_A_CADA = 8`); o aperto de mão é `ph2d_contact::obstaculo` e
`ph2d-node-sim-collide/src/mundo.rs`; a `=114` passa a `1` sub-passo.

**A prova, no MESMO processo, pela porta do app** (`prova_do_mundo_de_contacto`, `release`, intercaladas por tique,
carga `5`–`7`; a lei de antes atrás de uma chave POR FIO em `e78b2c096`, retirada em `594a859fd`;
`target/prova/onda5/prova_do_mundo.txt`). Tique do cozimento (mediana · p95 com a pilha formada) e, no `2,95` s, a
taça da direita:

| peças por taça | lei | tique (ms) | vizinho mediano | velocidade média | sobreposição mais funda |
|---|---|---|---:|---:|---:|
| `25` | antes (`8` sub) | `0,52` · `0,57` | `0,2214` | `0,029` | `1,3 %` |
| `25` | **rapier (`1` sub)** | **`0,05` · `0,07`** | `0,2198` (`−0,7 %`) | `0,016` | `1,4 %` |
| `1 024` | antes | `10,8` · `13,4` | `0,2141` | `0,220` | `41,1 %` |
| `1 024` | **rapier** | **`1,06` · `2,17`** | `0,2129` (`−0,6 %`) | `0,144` | **`9,9 %`** |
| `4 096` | antes | `17,9` · `44,1` | `0,1939` | `2,41` | `86,7 %` |
| `4 096` | **rapier** | **`4,30` · `7,51`** | `0,1856` | `1,26` | `47,6 %` |
| `16 384` | antes | `36,3` · `72,6` | — (a cair) | `8,5` | `76,3 %` |
| `16 384` | **rapier** | **`14,3` · `24,4`** | — (a cair) | `8,1` | `65,3 %` |

A base sem `Collide`: `0,02` · `0,13` · `0,45` · `1,68` ms. Duas corridas do rapier: os MESMOS bits em todas as
células. **Os kill-criteria:** `4 096` por taça mediana `4,30 ≤ 5` ✓, p95 `7,51` ✗ (o próprio passo do rapier é
`~2,3` ms em média a `4 096`, e o p95 dele no oráculo `3,6`–`5,2`; a cola é `0,15` ms depois de alinhar os corpos às
linhas do stream — media `0,55` com uma busca por `id`) — no app o quadro fica nos `60` fps (abaixo); a banda da ordem a `25` e `1 024`: vizinho
`−0,7 %`/`−0,6 %` ✓, velocidade `0,55×`/`0,65×` ✓, sobreposição `1,4 %` contra `1,3 %` a `25` (`+0,0001` u: a FOLGA que
o rapier admite de propósito, `0,005`) e `9,9 %` contra `41 %` a `1 024` ✓ — **a diferença vai ao dono no smoke**; os
gates da pilha verdes; determinismo ✓; o recuo seguido de Play dá os bits da 1.ª passagem ✓.

**No app** (`[motion-quadro]`, `release`, a pior janela de cada ciclo; `target/prova/onda5/app/`):

| peças por taça | RTX | iGPU |
|---|---|---|
| `1 024` | `60` fps em todas as janelas (antes `56`–`60`, recomeços `16`–`18`) | `60` fps |
| `4 096` | **`59`–`60` fps em todas** (antes `13`–`25`) | **`59`–`60` fps** |
| `16 384` | `12`–`31` fps (picos do cozimento `280` ms nos recomeços) | `10`–`36` fps |

⇒ **o alvo (`60` fps a `4 096` por taça) foi atingido nas duas placas.**

**O que a medição corrigiu no desenho (cada um com o gate novo):**

1. ⛔ **«A 2.ª volta do `Loop` dá os bits da 1.ª» — a premissa estava ERRADA:** o passo é `playhead − sim_t` em
   `f32`, e `3,6 + k/60` arredonda diferente de `k/60`. A lei de antes também não repetia (`1,5e-2` no tique `150`);
   o mundo repete a `1,06e-3`. Gate: `the_second_lap_of_the_loop_repeats_the_first` (barra `2e-3`) — o mundo NOVO no
   recomeço tem gate próprio ao bit (`a_stream_that_does_not_continue_the_world_starts_a_new_one`).
2. ⛔⛔ **As sondas e os gates que andam por `cook` + `advance_tick` NÃO correm os sub-passos da zona** (só a porta
   do app, `substep_declared_zones`): a 1.ª redacção da prova deu os MESMOS bits a `8` e a `1` sub-passo. A
   `prova_dos_impulsos_da_placa` de 05/10 mediu assim a lei de antes a `1` passo por tique — a tabela de 05/10
   (Gauss–Seidel `0,0796`/`0,266`/`109 %` a `1 024`) é a da lei SEM os `8` sub-passos que a cena shipava; pela porta do
   app ela dá `0,2141`/`0,220`/`41 %` (a linha «antes» acima).
3. **Duas caixas que nascem sobrepostas saíam a VOAR** com a omissão do rapier (`0,84` u/s mesmo paradas) — a lei
   aprovada *«quem nasce sobreposto e parado não ganha velocidade»*. ⇒ `ESTABILIZACOES = 4` (`0,009` u/s; o custo cabe:
   `4 096` `1,79 → 1,83` ms). Gate: `the_contact_never_adds_speed` (agora também com caixas, `120` tiques).
4. **O salto de uma CAIXA que bate de FACE é `0,88` do pedido** — a lei da família do Box2D (o salto aplicado no fim do
   passo, UMA passagem pelos DOIS pontos da face: `11/25 ÷ 1/2`); um disco dá a conta exacta, e o momento conserva-se
   sempre. Gates: `two_equal_boxes_share_the_blow…` e `an_obstacle_is_still_a_wall…` medem discos (exacto) e caixas
   (`0,88`). ⚠️ Vai ao dono: um `Bounciness 1` numa caixa de face devolve `88 %`.
5. **O ROLAMENTO — seis leis medidas, uma aceite** (a tabela das recusas no cabeçalho de
   [`rolar.rs`](../../../../crates/ph2d-contact-world/src/rolar.rs)): cortar o giro depois do passo, o binário (ganho `1` e
   `½`), o amortecimento do rapier, o travão relativo do Box2D v3 e o contacto ACHATADO pelo gancho — todos fazem a
   pilha tremer, rastejar ou acelerar. **Aceite: Coulomb em DUAS fases** — parada ⇒ a rotação TRANCADA no solver até
   o binário pedido passar a capacidade `Σ μr·λn·|braço|` (segura na rampa enquanto `tg θ ≤ μr`); a rolar ⇒ um
   binário constante da capacidade (o rapier integra-o EXACTO: medido `1,00×` a teoria). Rodopio na queda
   (`120..180`, a cena a `1` sub-passo): caixas `5,55 → 2,18/2,70/0,23` (`Rolling` `0,1/0,25/0,75`), discos
   `10,69 → 61,70/15,28/0,00`; a bola da `=115` trava na rampa de `12°` com `0,5` ✓; a bola que rola pára nos tempos
   da taça (`±2 %`) ✓. ⛔ **ABERTO:** um `Rolling` BAIXO numa PILHA de discos agita-a na queda (`61,7` contra `10,7`):
   a trava alterna trancar e destrancar (cada troca muda a inércia efectiva sob o aquecimento). A `8` sub-passos o
   mesmo perfil saía limpo (`84,5 → 47,6/0,30/0,00`) — a decisão é a uma resolução mais fina. A cura de raiz é o
   rolamento DENTRO do solver (uma restrição angular, como o Box2D v3), que o rapier `0.35` não tem.
   ⚠️ E a ARMADILHA do relógio dos testes: um `sim_t = k·dt` difere do `playhead` anterior (`t + dt`) no último bit,
   e o mundo RENASCE a meio — o gate da bola que rola lia `0,74×` o travão por isso (o produto grava `sim_t =
   playhead`, ao bit); os gates do passo acumulam o relógio.
6. **Gates da pilha que mediam a lei de antes, re-expressos com o porquê escrito neles:** o do `Rolling` mede a
   QUEDA (`120..180`), caixas e discos (a pilha de caixas sem o botão já pára antes do tique `240`: `0,05`, abaixo do
   piso); o da largura passa a ter UM lado (alargar abre a pilha `+12 %`; estreitar não a fecha — com a rotação
   travada as caixas caem em COLUNAS e caixas alinhadas com atrito não se espremem de lado); os cinco do passo
   (`overlapping…`, `approach…`, `two_boxes…`) marcham com a MEMÓRIA do mundo e a folga do rapier (`0,005`); o ruído
   do `Friction 0` da `=115` (`0,0018°`, a de `Friction 1` roda `313°`).
7. **O que saiu sem chamador** (`e21fd3195`): `ph2d_contact::impulsos`/`Leis`/`Movimento`, a variante Jacobi, o
   deslize da separação e os testes deles (`−1 414` linhas); as portas `PH2D_CONTACT_JACOBI`/`PH2D_CONTACT_CORES`. Fica
   o `separate` (o `motion.collide`), o `passe` (o sink), as portas do colisor e do material, e o `obstaculo` novo.

**O item da placa (o prompt de 05/10), reavaliado:** a CPU com o rapier NÃO chega a `16 384` por taça com folga
(`14` ms de mediana, `10`–`36` fps) ⇒ o item NÃO fecha por esta medição. Mas a pergunta mudou outra vez: a alavanca
seguinte medida no oráculo é o rapier `parallel` (`16 384`: `15,8`–`18,5` → `5,8`–`7,0` ms; `4 096`: `3,0` → `1,7`),
e falta provar o DETERMINISMO dele (a feature é unificada pelo cargo com a `ph2d-physics`, cujos gates de replay
entre sistemas operativos o pedem) — antes de qualquer contacto no dispositivo. E o recomeço a `16 384` paga a
construção de um mundo novo (`280` ms no quadro do recomeço).

### §9.21 — O QUE O §9.20 DEIXOU, num bloco (2026-10-06, depois do smoke aprovado: *«siga com o que está em aberto»*)

Os três itens abertos medidos juntos antes de construir (oráculo `oraculo_rapier_pilha_quente_frio/`, `release`):

**(1) O rapier em VÁRIOS núcleos (`parallel`) — CONSTRUÍDO.** O determinismo, a pergunta que o §9.20 deixou: a pilha
de `4 096` e a de `16 384` dão os MESMOS bits com `1`, `4` e `16` fios (`RAYON_NUM_THREADS`) E os da build sem a
feature (`4 096`: `eb8392c1a4123211` nos quatro); no produto, `1` e `16` fios os mesmos bits em todas as células
(`target/prova/onda5/paralelo_no_app.txt`); e o `ph2d_physics_c9` (o replay entre sistemas operativos da Física) dá o
MESMO hash com e sem a feature (`f72f20f9…`, binários diferentes). Os `1 600+` testes de `ph2d-physics`,
`ph2d-physics-ecs` e `ph2d-app-physics` verdes com ela (a feature unifica-se com a `ph2d-physics`). Custo do motor puro
(mediana · p95): `4 096` `2,0 · 4,8 → 0,95 · 2,1` ms; `16 384` `9,3 · 15 → 3,1 · 5,6`.

**(2) O pico do recomeço — CONSTRUÍDO: a taça é um colisor POR LADO.** O 1.º passo de um mundo novo de `16 384` peças
custava `207`–`213` ms (o 2.º `20`, o 3.º `2`) e não era a montagem (`3` ms) nem os lados (`32`/`128`/`512` iguais): a
polilinha tem a caixa do tamanho da taça e cada peça fazia par com ela. Sem a taça no mundo `10` ms; em segmentos `8`
ms, e o passo de regime `3,29 → 2,91`, a mesma pilha.

No produto (`prova_do_mundo_de_contacto`, os dois juntos; carga `5`–`10`): `4 096` por taça `4,30 · 7,51 → 2,32 ·
5,48` ms; `16 384` `14,3 · 24,4 → 6,96 · 17,1`. A sonda do tique do app a `16 384`: mediana `6,8` ms, máximo `27`, zero
picos. ⚠️ No app, com a máquina a carga `25`–`30` (outras linhas a correr testes a `1 585 %` de CPU), `16 384` leu
`4`–`59` fps — a medição do app fica para quando a carga deixar (abaixo).

**(3) O rolamento numa pilha de discos — MEDIDO, a trava passa a destrancar ao 2.º excesso seguido.** Três variantes
da regra de trancar/destrancar numa rodada (a tabela em `rolar.rs`, `EXCESSOS_PARA_SOLTAR`): o 2.º excesso é a melhor
no conjunto (caixas `0,1`/`0,25` `3,7°/4,4° → 3,2°/3,9°` com tremor `0`; discos `0,25` `44° → 0°`); os discos a `0,1`
continuam a agitar na QUEDA em todas (`104°` na 1.ª janela contra `10,7` sem o botão) e assentam como sem ele depois
(`3,1°` contra `2,4°`) — leitura (não prova): um monte de discos com rolamento fica inclinado e desaba enquanto
assenta. Gate novo `a_pile_of_discs_with_rolling_settles` (`420..480` `≤ 1°`; ao 1.º excesso `71°`).

**A régua das voltas do `Loop` saiu, e a lei que a substitui:** com a taça em segmentos a 2.ª volta afasta-se
`9,15e-2` da 1.ª (era `1,06e-3`) — o relógio `f32` amplificado pelo caos da pilha nas emendas; e a mutação «o mundo
velho continua» dava a MESMA diferença: o mundo já morria na PAUSA (a zona sem peças ⇒ sem colisor ⇒ sem mundo). ⇒
o gate passa a medir isso: `the_loop_pause_drops_the_world_and_the_next_fall_starts_a_new_one` (mutação `m15`
«o mundo fica sem colisor» sangra).

**Mutações** (o mesmo `mutacao_a_onda_5_2026-10-06.py`, `m14`/`m15` novas): `m3`, `m14`, `m15` sangram (as outras
`13` não mudaram de alvo).

**No app, com a carga a `4`–`8`** (`[motion-quadro]`, `release`, a pior janela de cada ciclo; `target/prova/onda6/app2/`):

| peças por taça | RTX | iGPU | o cozimento (mediana · máximo) |
|---|---|---|---|
| `4 096` | `59`–`60` fps em todas | `60` fps em todas | `2,5` · `12` ms |
| `16 384` | **`57`–`60` fps em todas** (antes `12`–`31`) | **`58`–`60` fps em todas** (antes `10`–`36`) | `7,7` · `26` ms (`≤ 2` tiques por quadro) |

⇒ **o item da placa (o prompt de 05/10) FECHA pela medição:** a CPU com o rapier em vários núcleos chega a `16 384`
por taça com o cozimento em metade do quadro; a cerca da placa fica (a cena com `Collide` coze na CPU). ⚠️ A primeira
leitura (carga `25`–`30`) dava `4`–`59` fps: com o quadro a atrasar o relógio recupera até `8` tiques por quadro e o
atraso alimenta-se — a medição do app não vale acima de `load ~5`.

### §9.22 — A QUEDA DURA O QUE A PILHA LEVA A ASSENTAR, e o que isso mostrou (2026-10-06, report do dono)

Report: *«a animação não dura o suficiente para ver todos os quadrados colidirem»*. A taça crescia com a pilha (§9.19)
e a queda ficava nos `3` s da cena do smoke: a `16 384` peças elas iam a `8` u/s quando a cena recomeçava. Medido
(`quanto_tempo_a_pilha_leva_a_assentar`, `release`, a taça da direita parada `0,5` s seguidos — média `< 0,05` u/s e
a mais rápida `< 0,3`; e «a olho», média `< 0,1`; `target/prova/onda6/assenta.txt`):

| lado | peças | a olho | parada |
|---:|---:|---:|---:|
| `5` | `25` | `1,37` s | `2,97` s |
| `16` | `256` | `2,03` | `7,12` |
| `32` | `1 024` | `6,95` | `14,23` |
| `48` · `64` · `96` · `128` | `2 304` … `16 384` | `4,7` · `8,2` · `8,1` · `9,0` | `13,1` · `11,2` · `13,6` · `14,8` |

⇒ a duração da zona passa a degraus que cobrem todas as medições (`duracao_de`): até `15` de lado os `3` s da cena do
smoke (ao bit), até `31` `8` s, daí para cima `15` s. Gate `the_pile_stops_before_the_fall_restarts` (a de `256` parada
antes do recomeço; com `3` s sangra — mutação `m16`).

⛔ **E a queda longa DESMENTIU o fecho do item da placa no §9.21:** as janelas do app de lá eram dos primeiros `3` s
(a queda); com a pilha FORMADA o tique cresce. Por fase (`custo_por_fase_da_queda`, sem vídeo, carga `8`–`10`):

| lado | `0–3` s | `3–6` s | `6–9` | `9–12` | `12–15` |
|---:|---:|---:|---:|---:|---:|
| `64` (`4 096`) | `2,3` (máx `8,7`) | `2,9` | `2,9` | `2,7` | `2,8` (máx `7,7`) |
| `128` (`16 384`) | `6,4` (máx `25`) | **`15,7`** (máx `51`) | `12,1` | `11,0` | `10,6` (máx `34`) |

No app a `16 384` com a pilha formada: `21`–`32` fps (fotos `target/prova/onda7/smoke/`). ⇒ **`4 096` por taça segura os
`60` fps na queda INTEIRA; `16 384` só na queda, não com a pilha formada.** As alavancas medidas no oráculo (o rapier
puro, `16 384` formada): dormir NÃO ajuda (a pilha é uma ilha só e nunca adormece: `16 384` acordados até aos `15` s);
`2`–`3` iterações em vez de `4` tiram `20`–`30 %`; o próprio rapier gasta `6`–`9` ms nessa pilha. ⇒ **o item da placa
REABRE**: `16 384` por taça a `60` fps com a pilha formada pede o contacto no dispositivo (um solver da família do
Box2D na placa: contactos persistentes e aquecimento), ou aceitar `~30` fps nessa escala.

### §9.23 — OS DISCOS COM `Rolling` BAIXO: avalanche ou a trava a alternar? (2026-10-06, escrito ANTES de construir)

O defeito (o item 2 do aberto de 06/10): a `=114` em discos com `Rolling 0,1` roda `104°` na janela `120..180` contra
`10,7°` sem o botão (`0,25`/`0,75`: `0°`), e depois assenta como sem ele. A leitura do §9.21 (3) — *«um monte inclinado
que desaba enquanto assenta»* — NÃO foi provada. Mecanismo antes da cura.

**As duas hipóteses e o que cada uma prevê** (a sonda `probe_avalanche_ou_trava`, discos da taça da direita, por disco
e por tique, na janela `120..180`):

| régua | AVALANCHE (física, fica) | A TRAVA A ALTERNAR (defeito) |
|---|---|---|
| deslocamento sobre a superfície contra `Σ|Δθ|·R` dos discos que mais rodam | `≈ 1` (rolar desce `Δθ·R`) | `≪ 1` (gira no lugar) |
| trocas trancar/destrancar por disco por segundo (`spin == 0` ⇔ trancado à saída do passo) | poucas, no início da descida | muitas, sem descida |
| a inclinação do topo do monte em `120 · 150 · 180` | maior com o botão, e CAI na janela | igual à de sem o botão |
| ⭐ a MESMA cena com o rolamento resolvido DENTRO do solver (abaixo) | o mesmo rodopio (a física é a mesma) | o rodopio cai para perto do de sem o botão |

⭐ **A última linha é o discriminador mais forte**, e por isso o candidato a cura entra na MESMA rodada (`CLAUDE.md`
§0.10): uma lei sem trancar nem destrancar é a referência que diz quanto do `104°` é física.

**O candidato — o rolamento DENTRO do solver pelo motor angular do próprio rapier:** uma junta genérica sem eixos
trancados entre cada peça com `Rolling` e um corpo FIXO do mundo, com um motor no eixo angular de velocidade-alvo `0`,
amortecimento rígido e força máxima `capacidade / dt` (o rapier converte em impulso `força · dt`: o tecto do impulso
por passo é a `capacidade` `Σ μr·λn·|braço|` do passo anterior, a mesma de hoje). O solver resolve-o nas iterações
dele junto com os contactos: parada enquanto o binário pedido cabe, binário constante quando passa — Coulomb nas duas
fases sem uma decisão fora do solver. Entra atrás de uma chave POR FIO só para a rodada (como a prova do §9.20).

**A rodada (uma compilação `release`, as duas leis no mesmo processo):** caixas e discos `Rolling 0 · 0,1 · 0,25 ·
0,75` (o perfil das três janelas, rodopio e tremor); as réguas do mecanismo acima; a bola da rampa da `=115` e a bola
que pára nos tempos da taça; o custo do tique a `4 096` e `16 384` por taça com `Rolling 0,1` (com o botão a `0` não
nasce junta nenhuma — o custo de hoje não muda).

**Kill-criteria (os do prompt, escritos antes):** discos `Rolling 0,1` em `120..180` `≤ 21°` (`2×` o de sem o
botão); e NENHUMA regressão — a tabela das caixas e dos discos a `0,25`/`0,75` do `rolar.rs`, a bola que pára nos
tempos da taça `±2 %`, a rampa da `=115` `< 0,035`, os gates da pilha, `4 096` por taça a `60` fps na queda inteira
(`[motion-quadro]`, `release`), duas corridas iguais ao bit, o recuo seguido de Play ao bit. ⚠️ Se a referência DENTRO
do solver der o mesmo rodopio, a leitura do §9.21 fica PROVADA e a lei de hoje fica (recusa medida do motor, com o
número).

**§9.23 — o resultado** (`release`, a `=114` com `duration` `12` s, `1` sub-passo; sondas em
[`motion_state_pilha_demo_discos_diag.rs`](../../../../crates/ph2d-app-motion/src/motion_state_pilha_demo_discos_diag.rs);
as rodadas com as leis atrás de uma chave POR FIO, retirada antes do fecho; `target/prova/onda8/`).

**1. Na árvore de hoje o defeito do prompt NÃO reproduzia — e a ablação disse porquê.** A realização central dava
discos `Rolling 0,1` `2,5°` contra `191°` sem o botão. A tabela do §9.21 (3) foi medida com a taça numa POLILINHA; ela
passou a um segmento por lado no MESMO commit (`12d3f220d`). Com a polilinha de volta (a mesma chave): `103,6°` e
`10,7°` — os números do prompt, ao bit. E nesse caso o mecanismo era um disco SOLTO a rolar `0,73` u monte abaixo
(nunca trancou; `0 %` de giro no lugar) — avalanche, a leitura do §9.21.

**2. Uma queda só não mede: com ONZE realizações (a grelha deslocada `±0,003` u) o defeito existe na taça de hoje.**
A lei de antes (destranca ao 2.º excesso): discos `0,1` queda mediana `1,8°` mas a PIOR `152°`; a `0,05` gira MAIS que
sem o botão (`225°` contra `131°` de mediana). Cinco realizações não o viram.

**3. ⛔ A régua «trancado = `spin == 0` à saída» estava CEGA:** a peça que solta, roda um passo e volta a trancar sai
com `spin` `0` em todos os tiques. A régua certa é «o passo não rodou a peça» (o `rot` igual ao bit). Com ela: na
pilha ASSENTE a trava trocava `20`–`39` vezes por peça por segundo (caixas e discos, `Rolling 0,05`–`0,25`), a pilha
rastejava, e uma em onze desabava tarde (`75°`, `0,18` u). Referência: discos com `Lock Rotation` (sem trava nenhuma a
ligar e desligar) — `0,000` u de deslize. ⇒ **é a trava a alternar (defeito), na pilha assente; na queda é avalanche.**

**4. ⛔ E um defeito que nenhum gate via: a bola numa rampa MAIS inclinada que o `Rolling` dela ficava presa.** A
`=115` (rampa de `12°`, `tg = 0,213`), `2` s, a distância contra a de `Rolling 0`: `0,1` · `0,15` · `0,2` deviam dar
`0,530` · `0,294` · `0,059` (teoria `(tg θ − μr)/tg θ`) e davam `0,020` em todos. Mecanismo (lido no código e medido):
depois de soltar, o 1.º passo dá sempre à peça um giro pequeno, que cabe na capacidade — e ela voltava a trancar.

**As leis medidas (a rodada inteira, as mesmas onze realizações; queda = rodopio `120..180` mediana · pior; assente =
depois do `240`, o pior rodopio, deslize e trocas/s):**

| trava | rampa `0,1`·`0,15`·`0,2`·`0,5` | discos `0,05` queda · assente | discos `0,1` queda · assente · trocas | caixas `0,1`·`0,25` assente (trocas) |
|---|---|---|---|---|
| (sem o botão) | `1` | `131`·`225` · `21°` | (o mesmo) | — |
| solta ao 2.º, tranca com `|L| ≤ cap` (a de antes) | `0,020` em todas | `225`·`295` · `117°` | `1,8`·`152` · `75°` · `21` | `1,4°`·`1,2°` (`35`·`21`) |
| solta ao 3.º | — | `189`·`271` · `71°` | `1,0`·`1,2` · `1,7°` · `5,4` | `2,5°`·`0,7°` |
| solta ao 8.º | — | `1,4`·`137` · `88°` | `0,4`·`0,4` · `0,5°` · `1,5` | `1,1°`·`0,8°` |
| solta ao 12.º | `0,019` em todas | `1,4`·`170` · `11°` | `0,3`·`0,3` · `0,4°` · `1,1` | `0,6°`·`0,5°` (`6,6`·`2,8`) |
| 12.º e só tranca quem NÃO acelerou | `0,556`·`0,241`·`0,103`·`0,019` | `40`·`53` · `293°` | `35`·`80` · `32°` · `4,0` | `1,7°`·`3,4°` |
| **12.º, e tranca quem não acelerou OU mal se mexeu (`|L| ≤ 25 %` da capacidade)** | **`0,556`·`0,241`·`0,103`·`0,018`** | **`116`·`199` · `210°`** | **`5,6`·`41` · `11,6°` · `3,8`** | **`0,9°`·`0,6°` (`6,6`·`3,5`)** |
| idem `10 %` · `50 %` | igual · a `0,15`/`0,2` volta a prender | `115`·`181` · `144`·`257°` | `6,3`·`16` · `47°` · `1,6`·`102` · `28°` | — |
| ⛔ folga no limiar `×1,25` · `×2` (5 realizações) | — | — | pior `153` · `138°` | — |
| ⛔ o passo que solta com o binário CONTRA o pedido | prende a `0,15`/`0,2` (`0,026`) | — | `36`·`62` · `13°` | — |
| ⛔ o motor angular do rapier (o rolamento DENTRO do solver: junta genérica a um corpo fixo, velocidade-alvo `0`, força `capacidade/dt`) | — | — | `58`·`110` · `28°` | tremor `0,013`–`0,016` (antes `0`) |

**ADOPTADA** (`ph2d-contact-world/src/rolar.rs`: `EXCESSOS_PARA_SOLTAR = 12`, `QUASE = 0,25`; `lib.rs`: o `ω` de antes
do passo): a única que acerta a rampa (a `0,1` `0,556` contra `0,530`; a `0,5` presa `0,018`) E deixa a pilha assente
calma (trocas `21 → 3,8` nos discos, `35 → 6,6` nas caixas; caixas assentes `≤ 1,2°`). As piores quedas que restam
(`41°` a `0,1`; as medianas de `84°`/`65°` a `0,15`/`0,2`) são, cada uma, UM disco a ROLAR `0,10`–`0,20` u por um monte
de `37°` (o caminho `1,24×` o giro vezes o raio, `0 %` de giro no lugar, `3` trocas) — avalanche: `Rolling 0,1` só
segura até `5,7°`. ⚠️ Por isso a mediana não é monótona no `Rolling` (`0,1` `5,6°`, `0,15` `84°`, `0,25` `0,3°`): o
rodopio é o MÁXIMO sobre as peças e conta se esse disco cai dentro da janela. O mesmo nos `210°` dos discos `0,05`
assentes: UM disco a rolar `0,41` u (caminho `1,11×`), os outros `≤ 1,2°`.

**O fecho** (HEAD `838384182`): `nextest-impacted` (`BASE=a46c4c200`) `18 773/18 773` · `check --workspace
--all-targets` com avisos negados ✓ · clippy das `3` crates `-D warnings` ✓ · `fmt` ✓ · censos `12/12` · standalone ·
workflow · `adr-index` · `doc-index` · `archive-index` · `machete` ✓ · `tests/it` da `ph2d-contact-world` `1/1`.

**O motor do rapier fica RECUSADO por medição** (pior que a lei de hoje na queda E assente, e a pilha de caixas parada
passa a tremer); o custo dele a `4 096`/`16 384` não foi medido (a sonda de relógio ficou escrita e não correu: a lei
caiu antes, pela física).

**Os kill-criteria do §9.23:** ✗ *discos `0,1` em `120..180` `≤ 21°`* pela PIOR realização (`41°`; a mediana `5,6°`) —
o critério foi escrito sobre UMA realização da taça antiga (`10,7°` sem o botão; com onze, sem o botão dá `131°` de
mediana) e a pior que sobra é avalanche (o próprio critério manda ficar); ✓ caixas `0,25`/`0,75` (`0,6°`/`0,0°`
assentes, antes `1,2°`/`0,1°`); ⚠️ discos `0,25`: de `0,0°` a `0,3°`·`0,6°` na queda e `1,0°` assente (o gate
`420..480 ≤ 1°` passa); ✓ a bola que pára nos tempos da taça (`±2 %`); ✓ a rampa a `0,5` (`0,0201 < 0,035`); ✓ duas
corridas iguais ao bit (a tabela final e a da rodada dão os mesmos números); o custo: a lei nova lê um `ω` por peça e
nada mais — com o `Rolling` a `0` (a `=114` de sempre) o passo é o mesmo.

**Gates novos:** `a_ball_rolls_down_a_ramp_steeper_than_its_rolling` (a rampa a `0,1`·`0,15`·`0,2`, `±0,06` da teoria
e monótona; a de antes `0,019`) · `the_rolling_lock_does_not_flicker_in_a_pile_of_discs` (onze realizações: trocas
`≤ 8`/s — antes `21` —, queda mediana `≤ 21°`). Mutações `m17` (solta ao 2.º), `m18` (tranca mesmo a acelerar), `m19`
(sem o `QUASE`).

