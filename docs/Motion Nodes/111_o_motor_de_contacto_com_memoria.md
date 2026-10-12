# 111 — O MOTOR DE CONTACTO COM MEMÓRIA (a espec da obra encomendada)

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/111_o_motor_de_contacto_com_memoria.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Adoptar o `rapier2d` como motor da sim do Motion | Tecto medido entre `1 600` e `6 400` peças contra `4,19 M` no dispositivo | vivo §3 e §6 (#9) |
| Medir a obra na cena como ela shipa (`μ = 0`) | O oráculo roda `331°` ali: a cena não tem resposta certa para medir contra | vivo §2 e §6 (#10) |
| `angular_damping` como cura | Inerte: amortece a coluna `spin` e o contacto escreve o `rot` (doc 109 §8.12) | vivo §6 (#11) |
| Repouso (W1) como cura do zumbido | O zumbido de `3,79 °/tique` impede o sono: cerca pequena não o apanha, grande adormece o que se move | vivo §5.2 e §6 (#14) |
| W0: dar atrito à cena | Não existe: as peças já têm `μ = 0,5`, medido na coluna | vivo §2.1 e §6 (#13) |
| Repouso como cerca SEM ESTADO | Congela a pilha no ar no primeiro tique; as duas réguas do tremor leem `0,0000` | vivo §5.1 e §6 (#12) |
| A «lei certa» do passa-baixo (W3b) | Construída e a 4.ª régua refuta-a; a W3b ficou dissolvida (§11.1) | vivo §5.6 / [arquivo](../archive/motion-nodes-2026-10-12/111_o_motor_de_contacto_com_memoria.md) §5.5 |
| Manifesto de dois pontos como ordem a seguir | Foi morto por uma medição errada e depois revalidado: o encosto de dois pontos é a cura (§5.11) | [arquivo](../archive/motion-nodes-2026-10-12/111_o_motor_de_contacto_com_memoria.md) §5.10.2 / §5.11 |
| Duas formulações do encosto de dois pontos | Construídas, medidas e refutadas | [arquivo](../archive/motion-nodes-2026-10-12/111_o_motor_de_contacto_com_memoria.md) §5.11.3 |
| As duas leis de atrito (posicional + velocidade) em simultâneo | Duplicam a metade translacional; a cura é uma lei só, no nível da velocidade | [arquivo](../archive/motion-nodes-2026-10-12/111_o_motor_de_contacto_com_memoria.md) §6.3 / §7 |

> **Ordem do dono, 2026-09-15:** *«encomendas o motor de contacto novo»* — depois de oito cadeias de
> cura medidas e refutadas sobre o zumbido da pilha ([doc 109 §8](109_o_colisor_na_forma.md)).
>
> ⛔ **Nenhuma linha de motor foi escrita.** Este doc é a espec, e o que ele traz de novo são
> **quatro medições** que mudam o desenho antes de ele começar.

---

## §1 — ⭐⭐⭐ A TRIAGEM parou na primeira porta ABERTA (`CLAUDE.md` §0.9, passo 1)

O solver que o doc 109 §8.14 encomendou — **impulsos sequenciais com `λ` acumulado por contacto
persistente** — **já está nesta árvore**:

| | |
|---|---|
| pacote | `rapier2d` **0.35.3** |
| licença | **Apache-2.0** (permissiva) |
| já usado por | `ph2d-physics` e `ph2d-physics-ecs`, desde o [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) |

⇒ *não se escreve um solver para descobrir o que um solver faz: corre-se o que já lá está.* A
bancada é [`oraculo_da_pilha_do_motion.rs`](../../crates/ph2d-physics/tests/it/oraculo_da_pilha_do_motion.rs)
— a **mesma** cena `=114` (25 quadrados de `0,11`, grelha `5×5` de vão `0,32`, taça de raio `1,8`,
gravidade `4`), com as **mesmas duas réguas** do Motion.

---

## §2 — ⭐⭐⭐ O ALVO, medido

| atrito entre peças | balanço pior (°/tique) | giro líquido pior (°/0,9 s) |
|---|---:|---:|
| **`0,0`** — *a lei da nossa cena* | `1,51` | **`330,99`** |
| `0,3` | **`0,0030`** | `51,26` |
| `0,6` | `0,1223` | **`24,82`** |
| **o Motion, hoje** | **`3,79 .. 5,69`** | `24,30 .. 28,36` |

**Duas leituras:**

1. ⛔⛔ **Com `μ = 0` nem o oráculo assenta** — ele roda uma peça **`331°`**, quase uma volta inteira.
   *Uma pilha de quadrados sem atrito não assenta em motor nenhum: é Física*, e é por isso que
   **toda medição desta obra corre com `μ ≥ 0,3`**.
2. ⭐⭐⭐ **À MESMA fricção o abismo é de CLASSE:** o oráculo lê `0,1223 °/tique` a `μ = 0,6` e nós
   lemos `3,79 .. 5,69` a `μ = 0,5` — **≈ 31×** —, e o giro fica na MESMA banda (`24,82` contra
   `24,30 .. 28,36`). ⇒ *o oráculo compra o silêncio **sem** pagar em rodopio*, que é exactamente o
   par que as oito tentativas do doc 109 não alcançaram.

### §2.1 — ⛔⛔⛔ E a primeira redacção desta secção estava ERRADA sobre a nossa própria cena

Ela dizia: *«a `=114` não escreve material nenhum nas peças, logo elas são gelo»*, e daí tirava uma
wave inteira (a «W0 — o atrito na cena»). **Medido** (`probe_as_pecas_sao_gelo`, que lê a coluna do
stream COZIDO em vez de a supor):

```text
  friction   PRESENTE, 25 linhas, valor[0] = 0.500
  bounce     PRESENTE, 25 linhas, valor[0] = 0.000
  ph2d_contact::materiais devolve Some ⇒ μ do par = 0.500
```

⇒ **as peças já têm `μ = 0,5` — o default do Rapier, que o `source.shape` declara desde o doc 109
§7 e escreve SEMPRE que o `Collide` está ligado.** A W0 **não existe**, e a comparação honesta é a
do ponto 2 acima (`31×`, não `1 260×`).

⚠️⚠️ *É a terceira ausência que esta caça afirmou sem olhar para a coluna* — e a lei do repo
já a tinha escrito duas vezes: **uma ausência afirmada sem olhar a API é um palpite com cara de
medição**.

⇒ **A barra da obra:** `balanço ≤ 0,15 °/tique` **e** `giro ≤ 28°`, na `=114` como ela shipa.

---

## §3 — ⛔⛔⛔ E o ORÁCULO NÃO PODE VIRAR O MOTOR: o tecto está medido

`probe_o_tecto_do_oraculo`, em RELEASE, caixas assentes numa caixa:

| peças | ms por tique | quadro de 16,7 ms |
|---:|---:|---|
| 100 | `0,015` | cabe |
| 400 | `0,058` | cabe |
| 1 600 | `0,352` | cabe |
| **6 400** | **`54,451`** | ⛔ **ESTOURA** |

⇒ o tecto está **entre 1 600 e 6 400**. O dispositivo faz **4,19 M objectos em 3,85 ms**
([auditoria 98](98_auditoria_de_performance_2026-09-01.md)).

⛔⛔ **Adoptar o solver de referência como MOTOR poria o tecto do módulo ~2 000× abaixo do que a
máquina faz** — e é, à letra, o caso registado no `CLAUDE.md` §0.0 (*«o caminho mais lento definiu o
teto do mais rápido, no módulo cuja razão de existir é o mais rápido»*). ⇒ **ele fica ORÁCULO**, e a
lei nova nasce dentro do solver por COLUNAS que já corre no dispositivo.

---

## §4 — O que a obra É, então

**Dar MEMÓRIA ao contacto, sem sair do modelo de colunas.** Hoje cada tique recalcula cada encosto
do zero; o que falta é o `λ` (o impulso acumulado) de cada contacto **sobreviver ao tique**, para a
restrição sustentar a orientação sem a re-excitar — o *warm starting* que o §8.14 nomeia.

### As quatro perguntas que a espec tem de responder ANTES de haver código

1. ⭐ **A IDENTIDADE de um contacto.** Um `λ` por par precisa de uma chave estável entre tiques. As
   peças já têm identidade (a coluna `id`, que o `sim.spawn` atribui e o `sim.collide` lê), mas a
   chave é do PAR e de uma FEIÇÃO (que quina contra que face) — e num monte a feição muda.
   ⚠️ **Sem chave estável o `λ` guardado é pior que nenhum:** ele aquece o contacto errado.
2. ⭐ **ONDE o cache vive.** Uma tabela por-par não é uma coluna. As saídas conhecidas: um blob
   opaco no estado da zona (o molde do `TimelineDoc` dentro do `ProjectFile`), ou um memo do cook
   (o molde que o `motion.trail` usou e **abandonou** — ADR-0163, *«um ring contém o passado porque
   passado é o que um ring é»*). ⛔ A segunda tem precedente CONTRA: aquele memo foi substituído por
   re-cozinhar.
3. ⛔⛔ **O DISPOSITIVO.** O `sim.collide` tem kernel, e um documento que declara colisor já é
   **recusado para a CPU** por gate (doc 109 W2). Um cache de contactos no device é um `StreamOp`
   novo com contagem derivada — a mesma família de substrato que o doc 103 §5.1 põe no item `10`.
   ⚠️ **Se a lei nova só existir na CPU, ela repete o defeito que ela própria vem curar** (um nó
   CPU-only no meio da cadeia custa o dispositivo inteiro, `50,9×`).
4. ⭐ **O REPOUSO.** O oráculo adormece corpos; é metade de por que ele lê `0,003`. Um critério de
   repouso sobre colunas é barato e **não precisa do `λ`** — ⚠️ e por isso **mede-se primeiro,
   sozinho**: se ele já entregar a barra do §2, a obra encolhe para uma wave.

---

## §5 — A fila proposta (a decidir com o dono)

| # | wave | entrega | porquê nesta ordem |
|---|---|---|---|
| ~~**W0**~~ | ~~o ATRITO na cena~~ | — | ✅ **DISSOLVIDA pelo §2.1:** as peças já têm `μ = 0,5`. A wave saiu de uma ausência que nunca foi medida. |
| ~~**W1**~~ | ~~o REPOUSO~~ | — | ⛔ **REFUTADA no §5.2:** o zumbido IMPEDE o sono. |
| ~~**W2**~~ | ~~a CHAVE do contacto~~ | **`92,7 %` de sobrevivência** | ✅ **MEDIDA no §5.3: há o que aquecer.** Luz verde para a W3. |
| ~~**W3a**~~ | ~~a **MEMÓRIA**~~ | o `ph2d_contact::warm` — o cache, a chave e as cercas, com 9 gates e `3/3` mutações | ✅ FEITA (§5.4) e ⛔ **APAGADA no §11**: nunca teve consumidor. Vive no commit `76dfd1947`. |
| ~~**W3b**~~ | ~~a **LEI**~~ | — | ⛔ **DISSOLVIDA no §11:** o defeito que ela vinha curar foi curado pelos sub-passos (§5.8), e o que lhe sobra é custo, com tecto MEDIDO de `≈ 11 %`. |
| ~~**W4**~~ | ~~o DISPOSITIVO~~ | — | ⭐ **deixou de ser uma wave** (§5.4): o cache é uma COLUNA de largura fixa, logo atravessa a fronteira como o `age` |

⚠️ **A W1 é a fronteira da encomenda.** Ela é a mais barata das quatro e pode tornar as outras três
desnecessárias — *medir se a composição já exprime o item antes de o construir* (`CLAUDE.md` §5.0).

### §5.1 — ⛔⛔⛔ E a W1 já foi PROTOTIPADA: a versão SEM ESTADO congela a cena

O protótipo (uma cerca: a peça cujo passo linear **e** angular cabem nela volta ao sítio e perde a
velocidade) foi construído e medido em 5 realizações por célula, a `μ = 0,6`:

| cerca linear / lado | cerca angular | balanço pior | giro líquido pior | `y` mediano final |
|---|---|---|---|---|
| `0` (o que shipa) | `0°` | `3,83 .. 7,79` | `24,73 .. 38,15` | — |
| `0,02` | `0,5°` | **`0,0000`** | **`0,00`** | **`−0,35`** |
| `0,05` | `1,0°` | `0,0000` | `0,00` | `−0,35` |
| `0,20` | `4,0°` | `0,0000` | `0,00` | `−0,35` |

⛔⛔ **`y = −0,35` é exactamente o `ALTURA` da cena — a altura de NASCIMENTO. A pilha nunca cai:**
ela congela no primeiro tique, onde o passo ainda é menor que a cerca, e fica pendurada no ar.

⚠️⚠️ **E as DUAS réguas do tremor leem isso como uma vitória PERFEITA** (`0,0000` e `0,00`), porque
uma pilha imóvel é imóvel *seja qual for a razão*. Foi preciso uma terceira régua — a ALTURA — para
separar *«assentou»* de *«congelou»*. ⇒ **a espec ganha essa régua como condição de aceitação de toda
a obra**, e ela é a terceira vez nesta caça que uma cura melhora o que se mede e estraga o que não se
media.

⇒ **um repouso honesto precisa de um CONTADOR por peça** (*«quieta há `N` tiques»*), acordado pelo
contacto — o que o oráculo faz. Isso é estado, mas é estado **POR PEÇA**, logo cabe numa COLUNA do
`state`, ao lado do `age` e do `sim_t`.

### §5.2 — ⛔⛔⛔ E a W1 com CONTADOR também cai: **o zumbido IMPEDE o sono**

O contador foi prototipado e medido (5 realizações por célula, e agora com a **terceira** régua):

| cerca / lado | cerca ° | `N` | balanço pior | giro líquido pior | `y` final |
|---|---|---|---|---|---|
| `0` (o que shipa) | — | — | **`3,79 .. 5,69`** | `24,30 .. 28,36` | `−2,56` |
| `0,02` | `0,5°` | 8 | `4,17 .. 7,86` | `18,37 .. 34,05` | `−2,54` |
| `0,05` | `1,0°` | 8 | `6,16 .. 9,43` | `9,73 .. 23,58` | `−2,55` |
| `0,05` | `2,0°` | 15 | `4,67 .. 7,74` | `8,01 .. 20,95` | `−2,54` |
| `0,10` | `5,0°` | 15 | `5,76 .. 8,44` | **`5,74 .. 8,74`** | `−2,41` |
| `0,10` | `5,0°` | 30 | `4,11 .. 7,69` | `6,65 .. 22,05` | `−2,48` |

⭐ **O contador cura o defeito do §5.1** — o `y` final é `≈ −2,5` em todas as células, ou seja **a
pilha cai e assenta**; a queda livre reinicia o contador, como se previa.

⛔⛔ **Mas o balanço NUNCA melhora — nem uma célula fica abaixo da linha de partida.** E a razão
estava escrita antes de medir: *o zumbido vale `3,79 °/tique`, logo uma peça a zumbir não cabe em
cerca nenhuma pequena e nunca chega a adormecer.* A `5,0°` a cerca começa a apanhá-la — e aí ela é
**do tamanho do próprio defeito**, e adormece o que ainda se move.

⭐⭐⭐ **A leitura que fica é sobre o oráculo:** ele lê `0,12` **não porque adormece**, mas porque
**não zumbe**; o sono é CONSEQUÊNCIA do silêncio, nunca a causa dele. ⇒ *não há atalho: o cache do
`λ` não é uma das quatro waves, é a obra.*

⚠️ **O contador NÃO é inútil** — ele corta o giro líquido de `24,3 .. 28,4` para `5,7 .. 8,7`. Fica
como candidato a **acabamento**, depois de o silêncio existir, nunca antes.

## §6 — ⛔ Recusas MEDIDAS que esta obra herda (não as reconstrua)

As oito do [doc 109 §8.14](109_o_colisor_na_forma.md), mais as três deste doc:

| # | ideia | veredito |
|---|---|---|
| 9 | **adoptar o `rapier2d` como motor da sim do Motion** | ⛔ tecto medido entre `1 600` e `6 400` peças contra `4,19 M` no dispositivo (§3) |
| 10 | medir a obra na cena como ela shipa (`μ = 0`) | ⛔ o oráculo roda `331°` ali: *a cena não tem resposta certa para medir contra* (§2) |
| 11 | o `angular_damping` como cura | ⛔ **inerte**: ele amortece a coluna `spin` e o contacto escreve o `rot` (doc 109 §8.12) |
| 14 | o **REPOUSO** (a W1) como cura do zumbido | ⛔ o zumbido de `3,79 °/tique` **impede o sono**: nenhuma cerca pequena o apanha, e uma grande adormece o que se move (§5.2) |
| 13 | a **W0** (dar atrito à cena) | ⛔ **não existe**: as peças já têm `μ = 0,5`, medido na coluna (§2.1) |
| 12 | o repouso como **cerca SEM ESTADO** | ⛔ congela a pilha no ar, no primeiro tique (§5.1) — e as duas réguas do tremor leem `0,0000` |


### §5.6 — ⛔⛔⛔ A LEI CERTA foi construída — e a QUARTA régua refuta-a

Ordem do dono: *«siga direto para a lei certa»*. Ela é o **XPBD com complacência e `λ`
persistente**, e tem a propriedade que todas as oito anteriores não tinham:

```text
  Δλ = (penetração − α̃·λ) / (Σw + α̃)      λ += Δλ      aplica-se Δλ, nunca λ
```

⭐ **Em repouso o equilíbrio é `penetração = α̃·λ` ⇒ `Δλ → 0` e NADA é aplicado, com a restrição a
segurar na mesma.** É a diferença exacta entre esta lei e as que o §8.14 refutou: elas recalculavam
a correcção do zero e re-excitavam a peça a cada tique. ⭐⭐ E **`α̃ = 0` devolve
`Δλ = penetração / Σw`, a lei de hoje termo a termo** — *o interruptor é a própria constante
física*.

Construída nas DUAS metades (peça×peça e o contacto com o mundo), 5 realizações por célula:

| `α̃` | balanço pior | giro pior | `y` final | **VÃO típico** |
|---|---|---|---|---|
| `0` (o que shipa) | `3,79 .. 5,69` | `24,3 .. 28,4` | `−2,56` | **`0,3094`** |
| `1e-1` | `1,76 .. 2,89` | `22,6 .. 28,7` | `−2,71` | `0,1828` |
| `1e0` | `1,63 .. 11,94` | `30,1 .. 39,6` | `−2,88` | `0,0546` |
| **`3e0`** | **`0,56 .. 0,66`** | **`5,9 .. 12,3`** | `−2,90` | ⛔ **`0,0222`** |
| `1e1` | `5,68 .. 7,70` | `249 .. 414` | `−2,90` | `0,0026` |

⛔⛔ **A célula `3e0` bate a barra nas duas colunas que o dono viu — e o monte é uma POÇA.** O vão
típico cai de `0,31` para `0,02`: as peças estão **umas dentro das outras**, e o *«silêncio»* é o de
uma pilha que deixou de ser uma pilha. ⇒ *a complacência compra sossego com MOLEZA, e as duas
réguas do tremor não vêem moleza.*

⭐ **É a QUARTA régua que esta caça teve de inventar, e sempre pelo mesmo motivo:**

| régua | o que ela viu que as outras não viam |
|---|---|
| o balanço por tique | o tremor, que o VÃO da cena não via |
| o giro LÍQUIDO | o rodopio, que o balanço não via (§8.4) |
| o `y` final | a pilha CONGELADA NO AR, que ambos liam como perfeita (§5.1) |
| **o VÃO típico** | a pilha COLAPSADA, que os três liam como perfeita |

⇒ *cada cura desta obra melhorou a grandeza medida e estragou uma que ninguém media.* **A quarta
entra nas condições de aceitação.**

⛔ **A lei foi REVERTIDA** (o `git diff` do motor contra o início é `+pub mod warm;` e mais nada), e
fica a **décima sexta** recusa medida.

| # | ideia | veredito |
|---|---|---|
| 16 | **XPBD com complacência e `λ` persistente** (a «lei certa») | ⛔ todo ponto que compra silêncio **colapsa a pilha**: a `3e0` o balanço é `0,56` e o vão cai de `0,31` para `0,0222` |

## §5.9 — ⭐⭐⭐ O 5.º REPORT: o SALTO depois de assentar, e ele é MAIS VELHO que a cura

> *«Melhorou os tremores mas a física ficou imprecisa. Essas 4 caixas apontadas depois de se
> assentar corretamente uma sobre a outra, dão um salto e ficam nessa angulação irreal»*
> — o dono, 2026-09-15, com foto e **quatro setas no meio do monte**.

### §5.9.5 — ⛔ Porque NÃO há gate ainda

A barra teria de sair de um **vale medido**, e este corpus não tem nenhum: o pior de toda a
varredura é `30,32°` e um quadrado tem simetria de `90°`. *Inventar um número aqui é exactamente o
que o `CLAUDE.md` §0.0 proíbe* — o gate nasce **com a cura**, que é quem define o lado aprovado.
O que fica é a régua, as sondas e esta tabela.

⏳ **ABERTO — as duas hipóteses com endereço, nenhuma medida ainda:**
1. **A rotação de contacto é projecção de POSIÇÃO sem velocidade angular** (declarado no cabeçalho
   de [`contact.rs`](../../crates/ph2d-node-sim-step/src/contact.rs)) — nada a amortece, então uma
   peça encravada pode acumular correcção até se libertar de repente.
2. **Dois solvers independentes disputam a mesma peça:** o `sim.step` resolve peça×peça e o
   `sim.collide` resolve peça×mundo **a jusante**, sem nenhum saber do outro — uma peça entalada
   entre a parede da taça e as vizinhas é projectada por ambos, alternadamente.

## §6 — ⛔⛔⛔ AUDITORIA COMPLETA DO ATRITO (7.º report do dono)

> *«bounciness correto. Mas o atrito ficou estranhamente reduzido mesmo no máximo como se estivesse
> desligado. Auditoria completa»* — o dono, 2026-09-15.

⚠️ **O produto NÃO foi tocado nesta secção.** Ela é só medição: o diff são sondas e as portas que
elas precisam.

### §6.1 — Os elos, um a um

| # | elo | veredito | como se mediu |
|---|---|---|---|
| 1 | o **Friction** do cartão chega à coluna | ✅ | `probe_auditoria_do_atrito_na_cena`: a curva MOVE-SE (largura `2,21 → 1,08`) |
| 2 | `μ` do par = `√(μa·μb)` (Box2D) | ✅ | `atrito::mu`, gates da crate |
| 3 | caixa a deslizar sobre obstáculo fixo | ✅ | trava a `μ·g`: μ=1 percorre `0,1166` contra a teoria `0,125` |
| 4 | invariância aos SUB-PASSOS (do impulso) | ✅ | `0,1166` (sub 1) contra `0,1240` (sub 8) |
| 5 | numa TORRE, o peso propaga | ✅ | 1/2/4 andares ⇒ `0,1240` / `0,0467` / `0,0227` |
| 6 | **monotonia na pilha** | ⛔ | mínimo em `μ ≈ 0,25`; a `μ = 1` desliza `2,2×` MAIS |
| 7 | **o atrito POSICIONAL trava alguma coisa?** | ⛔⛔ | **NÃO.** `v` fica em `1,0000` a todo `μ` |
| 8 | um DISCO chega a rolar pelo `sim.step`? | ⛔ | `ω·R ≈ 0` a todo `μ`, antes e depois |

### §6.4 — ⛔ Porque o posicional NÃO SE REMOVE: ele é a única coisa que roda uma bola

Desligá-lo parte **cinco** gates da crate (`a_disc_that_slides_starts_to_roll_and_ice_does_not` ·
`the_rolling_split_gives_the_spin_twice_the_slide` · `a_spinning_disc_rubs_against_the_floor…` ·
`the_bounce_of_the_liveliest_pair…` · `the_piece_against_piece_contact_ignores_rolling_bit_for_bit`),
e desligar **só a metade translacional** dele ainda parte **dois** — os que afirmam a repartição
`⅓` translação / `⅔` rotação.

⇒ **as duas leis não são redundantes: são COMPLEMENTARES e sobrepostas.** A posicional é a única que
produz **ROTAÇÃO** (a alavanca `r·n`); a de velocidade é a única que produz **ADERÊNCIA
TRANSLACIONAL**. O que duplica é a metade translacional da primeira.

### §6.5 — ⏳ A CURA, nomeada e ainda não construída

**O atrito tem de viver num sítio só, e esse sítio é o da VELOCIDADE** (tecto linear em `dt`, logo
invariante aos sub-passos) — com a **metade ROTACIONAL trazida para lá**, usando a mesma alavanca
tangencial (`Contacto::braco_tangente`) e a mesma repartição por massa efectiva que a lei posicional
já usa. Com isso:

- o atrito passa a ser **monótono** (uma lei só, sem duas a disputar);
- o disco volta a **rolar** em vez de parar a seco;
- a lei posicional de atrito **sai inteira**, e com ela o tecto `μ·g·dt²`.

⚠️ **O preço nomeado:** o `impulsos` precisa de escrever em `Saida::giro`, que hoje não recebe — é
uma porta a mais na assinatura, não uma lei nova.
⛔ **E os dois gates da repartição `⅓/⅔` mudam de sujeito**, não de barra: eles afirmam a lei sobre
o `separate`, e ela passa a viver no impulso. *Um gate cujo sujeito se mudou tem de mudar de
endereço — nunca de exigência.*

### §6.6 — ⚠️ E duas coisas que esta auditoria diz sobre as JORNADAS de hoje

1. ⛔ **A cura do tremor (substeps `1 → 8`) dividiu o atrito por OITO**, e as quatro réguas de então
   (tremor · rodopio · altura · vão) eram todas cegas a isso. *Um knob que melhora quatro grandezas
   pode dividir por oito uma quinta que ninguém mede.*
2. ⛔ **O «atrito» que a cena mostrava antes de hoje era, em grande parte, uma ILUSÃO** produzida
   pela lei de velocidade antiga (a que matava a velocidade ABSOLUTA). Ela travava tudo,
   independentemente do `μ` — um amortecedor com nome de atrito. Trocá-la por física correcta tirou
   o disfarce, e é por isso que o dono só agora vê o atrito verdadeiro, que é fraco.

## §7 — ✅⭐⭐⭐ A CURA DO ATRITO: uma lei só, no nível da VELOCIDADE

A auditoria do §6 nomeou-a e esta secção constrói-a. **O atrito posicional saiu inteiro**; o impulso
de Coulomb do [`ph2d_contact::impulsos`](../../crates/ph2d-contact/src/impulso.rs) passa a carregar
as **duas** metades — travar **e** rodar.

### §7.5 — ⏳ O que FICA aberto, nomeado

⛔ **Um disco não converge para o rolamento puro: ele trava até parar.** `v = ω·R` nunca se atinge
porque **este modelo não tem coluna de velocidade angular** (doc 109 §6, declarado): o `giro` é uma
rotação por tique, não um estado que persista, logo o atrito vê sempre derrapagem plena. ⚠️ *Isto é
anterior a esta cura e ela não o piora* — antes o disco nem sequer rodava (`ω·R ≈ 0` a todo `μ`);
agora roda enquanto desliza. Curá-lo é dar uma **velocidade angular** ao contacto, que é obra com
espec própria.

⚠️ E o `Saida::salto` deixou de ter leitor no produto (o impulso lê o material directamente) — ele
continua a ser recolhido, e é **dívida nomeada**, não um descuido.

## §8 — O 8.º REPORT: o tecto do salto, e a ROTAÇÃO QUE NÃO PERSISTE

> *«Está melhor. Limite Bounciness para máximo de 1. Uma coisa estranha: enquanto umas caixas
> rotacionam correctamente com as colisões de umas com as outras, outras caixas parecem não
> rotacionar»* — o dono, 2026-09-15.

### §8.3 — ⛔⛔ A CURA FOI CONSTRUÍDA, MEDIDA E REVERTIDA (a 18.ª recusa medida desta linha)

O impulso passou a escrever **velocidade angular** na coluna `spin` (que o `sim.step` já integra com
o `angular_damping`), e a massa efectiva do impulso normal ganhou a alavanca (`kn = w + invI·(r×n)²`).
⭐ **Funcionou no que o report pede:** a caixa atingida passa a girar a `142°/s` e **continua a
girar** muito depois de se separarem.

⛔ **E parte a pilha.** Medido, `substeps = 8`:

| régua | antes | com velocidade angular |
|---|---|---|
| tremor | `0,014 .. 0,024` | **`1,000 .. 3,075`** |
| rodopio | `0,7 .. 2,4` | **`77,5 .. 153,2`** (barra `29`) |

⚠️ **E não é a duplicação com a rotação posicional:** desligá-la deixa `8,6 .. 79,0`, ainda muito
acima da barra. ⚠️ Nem é o atrito não ver o giro: medido isolado, uma caixa a `180 °/s` num chão
fixo trava para `−3 °/s` em `0,5 s`. *O que falha é a pilha, e a causa ainda não está nomeada.*

⇒ **Revertido.** A direcção está certa e medida; ela precisa da wave que faça um monte de peças
com velocidade angular ASSENTAR, e isso não é mais uma linha.

⭐⭐ **Três achados que ficam da tentativa, e que a próxima janela não tem de pagar:**
1. **O impulso do par age no CENTRÓIDE do manifesto, nunca num extremo.** Tomar `pontos().first()`
   dá uma alavanca que a geometria não tem, e duas caixas iguais deixam de partilhar o choque.
2. **A `vrel` tem de ser lida no PONTO DE CONTACTO** (`+ ω·(r×n)`), senão uma peça a girar não é
   vista a aproximar-se.
3. **A sonda tem de devolver o `spin` ao tique seguinte** — sem isso ela mede um programa em que a
   velocidade angular é deitada fora a cada quadro, que é precisamente o defeito a testar. *A
   primeira corrida leu `spin = 0,0000` sempre, e a leitura óbvia teria sido «a cura não funciona».*

✅ **FECHADO no dia seguinte — ver §9.** ⛔⛔ E **nenhuma das duas hipóteses que eu escrevi aqui era
a causa**: nem o impulso normal a realimentar-se pelo `ω·(r×n)`, nem a ausência de atrito de
rolamento. A causa eram **duas leis de rotação a correr ao mesmo tempo** — a posicional, que
acumula ângulo sem velocidade, a somar-se à angular. *Duas hipóteses com endereço podem estar as
duas erradas, e o que as desempata não é escolher entre elas: é a varredura que inclui a que
ninguém escreveu.*

---

## §9 — ✅⭐⭐⭐ A CURA: a rotação PERSISTE, e eram QUATRO leis (2026-09-16)

O §8.3 reverteu a velocidade angular com o mecanismo por nomear. Ele está nomeado, e a resposta não
era nenhuma das duas hipóteses que eu tinha escrito: **eram quatro peças, indivisíveis, e cada uma
sozinha lê-se como fracasso.** A lei que shipa é a [`ph2d_contact::Leis::EM_VIGOR`].

### §9.4 — ⛔⛔ O manifesto de UM ponto não é pior: é um SORTEIO

Com velocidade angular e um ponto de contacto só, a leitura do rodopio contra as iterações é:

```text
  iteracoes |  2  |   4   |   8  |  16  |  32
  um ponto  | 33..66 | 15..132 | 6,5..30 | 16..70 | 14..160
  dois pontos | — | — | 42..44 | 39..42 | 37..40
```

⭐⭐ **A dispersão é o diagnóstico.** Com um ponto o valor salta de `30` a `160` conforme a célula —
não é convergência, é caos: uma caixa apoiada por uma FACE é sustentada por um SÍTIO, o impulso
aplica-lhe binário, ela roda, o apoio desloca-se, e o resultado depende do sorteio. Com dois pontos
o intervalo é de `±2` e **desce monotonamente**.

⛔⛔ *A célula `8 iterações, um ponto` lê `6,5..30` e é a melhor da coluna — e é sorte.* Eu quase
concluí dela que as iterações eram a alavanca principal. **Uma célula só de um sistema caótico é
uma carta tirada do baralho**, e é a varredura ao lado dela que o diz.

### §9.5 — ⛔ Não é convergência: a ASSÍNTOTA está medida

`128` iterações lêem `37,3..39,7`, iguais às `32` (`37,5..40,4`) e às `16` (`38,7..41,6`), e custam
`1,26 ms` contra `0,62`. ⇒ *mais solver não cura; falta uma LEI* — e era a do §9.3.

⚠️ É por isso que o `iteracoes` shipa em **`8`**: `1` lê `72..77`, `2` lê `3,6..38,2` (não
convergiu), `4` já lê `3,0..4,1`, e de `8` para `64` a resposta não se move enquanto o relógio sobe
até `1,73 ms`. *O `8` é onde a curva assenta, não uma preferência.*

⛔ **E o `tecto_por_lambda` foi medido e REFUTADO** — o tecto de Coulomb pelo `λ` acumulado devolve
a dispersão (`rodopio 3,0..24,7`), porque o `λ` da 1.ª varredura ainda não conhece a carga. A
estimativa `max(vrel, pen/dt)` do doc 111 §7 é mais estável, e fica.

### §9.6 — ⛔⛔ O arrasto angular é PLANO, e isso é um achado

`angular_damping` de `1,00` a `0,30` (o knob que o `sim.step` já tem e a cena nunca usou):

```text
  arrasto | 1,00  | 0,95  | 0,90  | 0,80  | 0,60  | 0,30
  rodopio | 41,77 | 41,47 | 41,83 | 42,31 | 42,17 | 42,19
```

⇒ **quando nenhum ponto de um knob resolve, o que falta não é o knob** — a mesma lei que esta linha
já tinha pago com o `damping` linear no §5.12.3. E aqui ela tem mecanismo: a rotação que o rodopio
conta **não é energia por dissipar, é a caixa a ir onde tem de ir**. Amortecê-la fá-la chegar mais
devagar ao mesmo sítio, e o giro LÍQUIDO é o mesmo.

### §9.10 — O que fica ABERTO

- ✅ **O atrito de ROLAMENTO continua a não alcançar o contacto peça×peça** — **FECHADO no §10.**
- ✅ **(FECHADO no §11 — a memória foi apagada.)** **O `warm.rs` continua sem consumidor.** A fatia 2 encomendada pelo dono (a lei que consome a
  memória do `λ`) **não** foi o que curou isto, e o solver de `8` iterações arranca frio a cada
  sub-passo. Ele é o caminho para baixar as iterações, não para curar a rotação.
- ✅ O §7.5 fica de pé: um disco não converge para rolamento puro (`v = ω·R`) — **FECHADO pelo
  próprio §9, e quem o escondia era a SONDA** (ver §10.1).
- ✅ `Saida::salto` continua sem leitor de produção — **FECHADO no §11 (apagado).**
- ⚠️ **O `Leis` tem cinco campos e o produto usa UMA combinação.** Eles são as colunas de uma tabela
  medida, não configuração; quem lhes mexer sem re-medir a `=114` está a escolher uma célula ao
  acaso. Se a tabela não voltar a ser precisa, a struct colapsa na lei.

---

## §10 — ✅⭐⭐⭐ O BOTÃO `Rolling` DEIXA DE SER MORTO entre peças (2026-09-16)

Smoke do §9 aprovado pelo dono (*«smoke OK. Siga com o plano»*), e o passo seguinte era o que eu
lhe tinha nomeado: o atrito de rolamento. ⛔⛔ **Ele não faltava: o botão existia no cartão e era
MORTO numa pilha**, e o contrato da coluna dizia-o por escrito (`ROLLING_COLUMN`: *«no contacto
peça × peça … não há nada que este número possa travar»*). Era verdade até ao §9 e deixou de ser no
mesmo commit — *quem move o número que tornava algo inalcançável tem de reconferir a nota* (§0.0).

### §10.4 — ⛔⛔⛔ A 1.ª lei fazia o botão AGITAR o monte

A primeira redacção travava o giro **RELATIVO** do par (`ω_lo − ω_hi`, com a massa angular dos dois)
— a física certa para uma bola a rolar sobre outra, e o que a conservação do momento angular pede.
**O gate do disco ficou verde com ela**, porque contra um obstáculo (`invI = 0`) as duas formas são
a mesma conta. Na `=114`, rodopio janela a janela (`probe_o_rolamento_na_pilha`):

```text
  Rolling | forma do PAR (120..180 · 240..300 · 480..540) | forma de CADA peça (idem)
  --------|-----------------------------------------------|---------------------------
     0    |        2,66  ·  0,47  ·  0,51                 |   2,66  ·  0,47  ·  0,51
     0,25 |       40,98  ·  5,74  ·  0,49                 |   1,19  ·  0,02  ·  0,03
     0,75 |       28,90  · 22,03  ·  5,31                 |   1,22  ·  0,02  ·  0,02
     1,5  |       32,42  · 12,95  ·  6,34                 |   1,15  ·  0,05  ·  0,02
```

⇒ **o botão fazia o contrário do nome.** O mecanismo: numa pilha de CAIXAS o binário que trava a
caixa que tomba é entregue à vizinha PARADA, que roda e é desalojada — um acoplamento espúrio, porque
tombar por uma aresta não é rolar. A forma de cada peça é sempre dissipativa: só tira giro a quem o
tem. ⚠️ Preço declarado: uma bola sobre uma plataforma que GIRA é travada contra o mundo — um caso
que nenhuma cena do produto tem.

⭐⭐ **E com a forma certa o botão faz o que o nome promete:** a `0,25` o monte fica praticamente
imóvel (`0,02`) a partir dos `180` tiques, contra `0,5–0,8` sem ele. A `0,25` já compra quase tudo.

⚠️ **E meia lei é pior que nenhuma:** com o rolamento entre peças APAGADO (a prova de mutação), o
`Rolling = 0,75` lê `1,65` contra `0,47` sem o botão — a taça continua a travar as peças que lhe
tocam e as outras não.

### §10.6 — O que fica ABERTO

- ✅ **(FECHADO no §11.)** **O `warm.rs` continua sem consumidor** (a fatia 2 encomendada). O solver arranca frio a cada
  sub-passo com `8` iterações; a memória é o caminho para as baixar, e nenhuma régua desta cena a
  pede hoje.
- ✅ `Saida::salto` continua sem leitor de produção — **FECHADO no §11.**
- ⚠️ O `Leis` tem cinco campos e o produto usa uma combinação — ver §9.10.

---

## §11 — ⛔ A MEMÓRIA do contacto e o `Saida::salto` SAEM: duas portas sem chamador (2026-09-16)

Smoke do §10 aprovado (*«Smoke OK. Siga»*). Sobravam da obra dois itens internos, e os dois eram
**portas sem chamador** — *uma porta sem chamador e uma lei ausente produzem o mesmo app*, e a
primeira ainda paga gates, leitura e a ilusão de que a obra está a meio.

### §11.3 — O que a obra do contacto deixa

Nada aberto que mude o ecrã. ⚠️ O `Leis` fica com cinco campos e **uma** combinação em produto
(§9.10) — eles são as colunas das tabelas deste doc, e a varredura por mutação do `const` é o
instrumento que as lê.
