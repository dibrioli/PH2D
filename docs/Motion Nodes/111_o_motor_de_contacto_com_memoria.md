# 111 — O MOTOR DE CONTACTO COM MEMÓRIA (a espec da obra encomendada)

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
| **W3a** ✅ | a **MEMÓRIA** | [`ph2d_contact::warm`](../../crates/ph2d-contact/src/warm.rs) — o cache, a chave e as cercas, com 9 gates e `3/3` mutações | **FEITA** (§5.4). ⛔ Nada a consome ainda ⇒ o produto é byte-idêntico. |
| **W3b** | a **LEI** | o `λ` acumulado a entrar no [`separate`](../../crates/ph2d-contact/src/lib.rs), atrás de interruptor | é a obra; a W3a é o chão dela |
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

### §5.3 — ⭐⭐⭐ A W2 está MEDIDA, e ela AUTORIZA a obra

A pergunta: *o `λ` acumulado precisa de uma chave estável entre tiques — ela existe?* Medido na
janela assente da `=114` (`probe_os_contactos_sobrevivem`), com a chave mais grosseira possível (o
PAR de `id`s, sem feição):

| | |
|---|---|
| contactos que SOBREVIVEM ao tique anterior | **`1 440`** |
| contactos NOVOS | `113` |
| **taxa de sobrevivência** | **`92,7 %`** |

⇒ **há o que aquecer.** ⚠️ A medição é do PAR; uma chave com a FEIÇÃO dentro sobrevive **menos**, e
esse número é o primeiro passo da W3 — mas a `92,7 %` diz que a família não está fechada à partida,
que era o risco real.

---

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


### §5.4 — ✅ A W3a FEITA: a memória existe, e a morada dela foi MEDIDA

⭐⭐⭐ **A pergunta do §4.2 (*«onde vive o cache?»*) tinha resposta MEDIDA, e ela muda a fila.** Um `λ`
por PAR seria uma tabela lateral, e uma tabela lateral não atravessa a fronteira do dispositivo — o
defeito que a obra vem curar, um nível acima. Medido na `=114`
(`probe_quantos_encostos_por_peca`), sobre `1 375` peças-tique da janela assente:

| encostos numa peça | peças-tique | acumulado |
|---:|---:|---:|
| 0 | 19 | `1,38 %` |
| 1 | 370 | `28,29 %` |
| 2 | 475 | `62,84 %` |
| 3 | 262 | `81,89 %` |
| 4 | 191 | `95,78 %` |
| **5** | 58 | **`100,00 %`** |

⇒ o pior caso é **5**, e `K = 6`: a memória é **por ELEMENTO e de largura fixa**, logo **É uma
coluna** (`K/2 = 3` colunas `Vec4`, `24 B` por elemento). Ela viaja no laço do estado como o `age` e
o `sim_t`, e um kernel lê-a sem substrato novo — **a W4 deixa de ser uma wave e passa a ser uma
consequência do desenho.**

⛔⛔ **E a CHAVE trouxe uma cerca que a lei consumidora tem de honrar.** Medido
(`probe_a_pilha_tem_identidade`): a `=114` **não traz a coluna `id`** — as colunas dela são
`Count · Index · P · bounce · collider_box · friction · geometry_id · rolling · size`, porque aquela
cena nasce de uma grelha com carimbo e quem cunha `id` é o `sim.spawn`. ⇒ [`warm::chaves`] cai no
**índice**, e daí:

> ⛔ **Sem `id`, a memória só é válida enquanto a POPULAÇÃO não muda.** Um nascimento ou uma morte
> reindexa a corrente, e um `λ` guardado num índice que se deslocou aquece o **contacto errado** —
> pior que não aquecer.

**O que a W3a entrega, com gate:** o que se guarda volta para o parceiro certo e para mais nenhum ·
guardar duas vezes ACTUALIZA (senão seis tiques enchiam a memória com um só parceiro) · o
**transbordo esquece o apoio mais FRACO** (esquecer o forte tiraria a memória de onde ela sustenta a
pilha) · um `λ` a zero **APAGA** a ranhura · a ida e volta pela corrente é a **identidade**, com uma
identidade de `2²⁰` dentro dela (⚠️ sem essa fixtura um empacotamento em `u16` passava o gate e
truncava toda cena com mais de `65 536` peças, **em silêncio**) · colunas ausentes ⇒ **tudo frio**,
a lei de hoje ao bit.

**`3 de 3` mutações mortas**, e a terceira — o `u16` — só morre por causa da identidade grande.
### §5.5 — ⭐⭐ A W3b PROTOTIPADA: o PASSA-BAIXO — real, medido, e ainda `4×` curto

⭐⭐⭐ **A W3a desbloqueou uma coisa que o doc 109 §8.14 dava por impossível:** ele fechava com *«o
`sim.collide` é `Effect::Pure`, logo não tem memória»*. **A premissa dissolveu-se** — o cache é uma
COLUNA, e uma coluna ENTRA e SAI de um nó puro; o laço do estado devolve-a no tique seguinte. *Um nó
sem estado pode ter memória, desde que a memória viaje nos dados.*

**A forma da lei saiu do SINAL, não de uma escolha.** O zumbido alterna de sinal em **17 de 18**
tiques (§8.2 nº 6) — **Nyquist puro** — e o que SEGURA o ângulo é a componente **DC**. ⇒ um
**passa-baixo** passa o DC inteiro e anula o Nyquist, ⛔ ao contrário de baixar o ganho, que corta os
dois (é por isso que aquela família fechou, §8.9).

**Só a metade do MUNDO** (5 realizações por célula):

| mistura | balanço pior | giro líquido pior | `y` final |
|---|---|---|---|
| `0` (o que shipa) | `3,79 .. 5,69` | `24,30 .. 28,36` | `−2,56` |
| **`0,3`** | **`2,06 .. 3,05`** | **`12,88 .. 27,74`** | `−2,54` |
| `0,5` | `1,36 .. 1,91` | `15,60 .. 71,96` | `−2,56` |
| `0,9` | `0,79 .. 1,00` | `28,71 .. 63,22` | `−2,49` |

⭐ **A `0,3` é a PRIMEIRA coisa desta caça inteira que melhora as DUAS colunas** — zumbido `1,9×`
melhor **e** o pior giro abaixo do de hoje — com a pilha a cair na mesma.

**As DUAS metades juntas** (mundo + peça×peça, a mesma mistura):

| mistura | balanço pior | giro líquido pior | `y` final |
|---|---|---|---|
| `0,3` | `2,18 .. 3,34` | `17,15 .. 88,97` | `−2,57` |
| `0,5` | `1,64 .. 2,20` | `25,11 .. 34,58` | `−2,50` |
| **`0,7`** | **`0,65 .. 1,32`** | `21,50 .. 39,91` | `−2,50` |
| `0,9` | `1,09 .. 1,34` | `66,65 .. 97,85` | `−2,47` |

⇒ **o melhor ponto medido é `0,65`**, contra a barra de **`0,15`** — **`4,3×` curto** —, e a partir
de `0,5` o giro volta a sair da banda. *A troca regressa, com câmbio melhor.*

⛔ **O protótipo foi REVERTIDO** (o `git diff` do motor contra o início da jornada é `+pub mod
warm;` e mais nada): ele vivia atrás de uma porta atómica, e uma porta de instrumento não é produto.
O que fica é esta tabela — e a **décima quinta** recusa medida.

| # | ideia | veredito |
|---|---|---|
| 15 | o **passa-baixo** sobre a correcção angular | ⚠️ **parcial**: `5,8×` no zumbido no melhor ponto, mas `4,3×` curto da barra, e acima de `0,5` o giro sai da banda |

**O que isto ensina à obra que falta:** o filtro ataca o SINTOMA (a alternância) e não a causa (a
correcção ser recalculada do zero). ⇒ a lei tem de usar a memória para **o `λ` não ser recalculado**,
e não para suavizar o que foi recalculado. É a diferença entre filtrar um ruído e não o produzir.

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

### §5.7 — ⇒ O que sobra, e agora está cercado por medição

O silêncio **a rigidez plena** é o que falta, e as três saídas conhecidas estão agora todas
medidas ou cercadas:

1. ⛔ **baixar o ganho** — refutada (§8.9): corta o DC junto com o Nyquist;
2. ⚠️ **filtrar** — parcial (§5.5): `5,8×`, e `4,3×` curta;
3. ⛔ **amolecer** (complacência) — refutada aqui: colapsa;
4. ⏳ **SUB-PASSOS** — a que o oráculo usa e que esta obra nunca tocou: em vez de resolver o tique
   inteiro de uma vez, parti-lo em `N` passos pequenos, cada um com a sua integração **e** o seu
   contacto. É por isso que um solver de impulsos assenta a rigidez plena, **e** é o que o
   `substeps` da `sim.zone` já exprime (`CLAUDE.md` §5.1: *«o RELÓGIO do grafo, e a palavra tem
   dono»*, tecto `64`).

⭐⭐⭐ **A 4 é a próxima medição, e é BARATA: o knob já existe no cartão.** ⚠️ E ela vem com um preço
óbvio a medir junto — `N` sub-passos custam `N` vezes o solver, o que é exactamente o tipo de troca
que o `CLAUDE.md` §0.0 manda medir antes de escrever qualquer tecto.

---

## §5.8 — ✅⭐⭐⭐ A CURA: os SUB-PASSOS, e ela já estava num botão do cartão

Ordem do dono: *«pode medir»*. Medido — e é a última saída de pé do §5.7 que resolve, sem uma linha
de motor.

### §5.8.1 — ⛔⛔ Mas a primeira medição leu CINCO CÉLULAS IDÊNTICAS, e o relógio não subia

| substeps | balanço | giro | `y` | vão | cozimento |
|---|---|---|---|---|---|
| 1 · 2 · 4 · 8 · 16 | `3,79 .. 5,69` **(todas)** | `24,3 .. 28,4` | `−2,56` | `0,2583` | `0,65 ms` |

⇒ *se `16` sub-passos custassem `16` vezes e o relógio não sobe, o param não está a chegar.* **Não
estava:** o motor do substep é a **marcha do PUMP** (`advance_or_scrub_*`), e a sonda chamava o
`Cook::cook` à mão — um `cook` por tique, sem substepping nenhum. ⇒ ⛔ **a sonda media um programa
que não tem sub-passos**, que é a mesma forma de erro que o `CLAUDE.md` já regista sobre sondas que
armam um módulo por outra porta.

⚠️⚠️ **E não era só a sonda: TODO gate desta cena marchava assim.** É por isso que a cura passou
despercebida a uma tabela inteira de tentativas — *as réguas mediam uma variante da `=114` que o
artista nunca vê*. As duas do tremor passaram a marchar pelo pump.

### §5.8.2 — A tabela, pela porta certa (5 realizações por célula, as QUATRO réguas)

| substeps | balanço pior | giro pior | `y` final | **vão típico** | cozimento |
|---|---|---|---|---|---|
| **1** (o que shipava) | `3,79 .. 5,69` | `24,3 .. 28,4` | `−2,56` | `0,2583` | `0,67 ms` |
| 2 | `0,67 .. 1,40` | `17,9 .. 35,0` | `−2,48` | `0,2630` | `1,44 ms` |
| 4 | `0,17 .. 0,61` | `19,9 .. 33,1` | `−2,43` | `0,2493` | `2,63 ms` |
| **8** | **`0,18 .. 0,22`** | `9,5 .. 31,7` | `−2,53` | `0,2390` | **`4,67 ms`** |
| 16 | `0,087 .. 0,171` | `8,4 .. 16,5` | `−2,43` | `0,2358` | `9,94 ms` |

⭐⭐⭐ **As QUATRO réguas passam juntas, pela primeira vez em dezasseis tentativas:** o tremor cai
`20×`, o rodopio pela metade, e as duas de FORMA ficam intactas — a pilha **não congela** (`y` na
mesma) e **não colapsa** (vão `−9 %`, contra os `−93 %` da complacência do §5.6).

**A cena shipa `8`**, e o recurso que escolhe o número é o **QUADRO**: `4,67 ms` são `28 %` de
`16,7`, e o `16` custa `9,94` (`60 %`) para comprar `2×` num tremor que a `8` já é `20×` menor que
o de ontem. ⛔ Acima daqui o preço cresce linear (`0,67 → 1,44 → 2,63 → 4,67 → 9,94`) e o ganho não.

### §5.8.3 — ⭐⭐ O que esta obra inteira ensinou

**O defeito nunca esteve no motor: esteve num número da CENA** — e foram precisas **dezasseis**
recusas medidas para lá chegar, porque *o knob que já existia nunca foi medido*. A lei do repo
(`CLAUDE.md` §5.0) diz exactamente isto: **antes de construir um item de lista aberta, MEÇA se a
composição já o exprime.**

⚠️ E o que tornava a medição impossível era o furo do §5.8.1: **a régua e o produto entravam por
portas diferentes.** Enquanto isso durou, nenhuma varredura de `substeps` podia dizer nada.

O gate `the_pile_settles_instead_of_buzzing` deixa de ser um vermelho declarado e passa a **VERDE**,
com prova red-first: com `SUBSTEPS = 1` ele reprova nomeando as **cinco** peças do fundo que o dono
fotografou.

## §5.9 — ⭐⭐⭐ O 5.º REPORT: o SALTO depois de assentar, e ele é MAIS VELHO que a cura

> *«Melhorou os tremores mas a física ficou imprecisa. Essas 4 caixas apontadas depois de se
> assentar corretamente uma sobre a outra, dão um salto e ficam nessa angulação irreal»*
> — o dono, 2026-09-15, com foto e **quatro setas no meio do monte**.

### §5.9.1 — ⛔⛔ A régua nova nasceu ERRADA TRÊS vezes, e cada erro tem nome

A medição está em [`motion_state_pilha_demo_salto.rs`](../../crates/ph2d-app-motion/src/motion_state_pilha_demo_salto.rs).
⚠️ **Antes de ela ver o defeito, foi preciso corrigi-la três vezes** — e as três correcções são a
lição, não o preâmbulo:

| # | o que estava errado | o que ela lia | porquê |
|---|---|---|---|
| 1 | a **UNIDADE DE TEMPO**: salto = `1` tique | `0,89°`, invisível | *o olho não chama salto a um tique* — chama a tudo o que passa mais depressa do que ele acompanha (~`0,1 s`) |
| 2 | a **JANELA**: parava em `174` (`2,9 s`, o 1.º ciclo) | nada | o salto cai no assentar da **SEGUNDA** queda, e a margem `JANELA + salto` comia ainda mais cauda |
| 3 | o **SUJEITO**: `com_substeps` escrevia numa zona só | as células `1/2/4/8` **idênticas** | ver §5.9.2 |

⇒ *a grandeza deste report é um EXTREMO num intervalo curto*, e as quatro réguas que a cena tinha
são **médias, somas ou fotografias** — cegas a ele por construção, cada uma por um motivo diferente.
É a **quinta** vez que esta linha paga esta forma exacta.

### §5.9.2 — ⛔⛔⛔ A CURA PARTIU A RÉGUA QUE A MEDIU, e ela lia `max(8, pedido)`

O `substeps` é o **RELÓGIO DO GRAFO**: [`graph_substeps`](../../crates/ph2d-nodegraph/src/cook_substep.rs)
toma o **MÁXIMO** sobre todo nó que declara o param. A sonda escrevia só na zona da **DIREITA**
(`.last()`), e a cena tem **duas**. No dia em que a cura assou `SUBSTEPS = 8` no `build()`, a zona da
**ESQUERDA** passou a fixar um CHÃO de `8`:

```text
  ANTES da correcção          DEPOIS (todas as zonas)
  1 | 0.182..0.219  4.57 ms   1 | 3.787..5.691  0.73 ms
  2 | 0.182..0.219  4.59 ms   2 | 0.670..1.400  1.43 ms
  4 | 0.182..0.219  4.54 ms   4 | 0.172..0.610  2.57 ms
  8 | 0.182..0.219  4.54 ms   8 | 0.182..0.219  4.54 ms
 16 | 0.087..0.171  9.77 ms  16 | 0.087..0.171  9.78 ms
```

⚠️⚠️ **Quatro células idênticas leem-se como uma varredura.** É a mesma forma do censo que passa a
varrer por um prefixo que já não existe: *a cura mudou o SUJEITO da medição, e a medição não soube.*
⭐ Com a correcção, **a tabela do §5.8.2 reproduz exactamente** — o `8` que shipou foi medido certo.

### §5.9.3 — ⭐⭐⭐ O defeito, medido

`substeps = 8`, peça `12`, tique `157`: quieta a **`0,147 °/tique`**, roda **`23,26°` em 8 tiques**
deslocando **`0,52` do lado dela**, e fica quieta outra vez a **`0,337°`**. A peça `17` salta
`16,5°` no **mesmo instante** — *duas peças de uma vez, que é o que as quatro setas mostram.*

O perfil no tempo mostra-o sem ambiguidade (a pilha assente, o salto, a pilha assente):

```text
  indice |  pior |Δrot| do tique
     150 |   0.27°   ← parada
     163 |   7.81°
     164 |  18.20°   ← o salto
     171 |   1.08°   ← parada outra vez
```

### §5.9.4 — ⛔⛔ E os SUB-PASSOS **não o causam** — eles REVELAM-no

Cinco realizações por célula (`probe_o_salto_contra_os_substeps`), perturbando o berço em `±0,003`:

| substeps | pior salto legítimo (5 realizações) | mediana |
|---|---|---|
| **1** (o que shipava ontem) | `8,93°` `8,41°` `1,73°` `0,90°` `8,45°` | `8,41°` |
| 4 | `17,45°` `17,29°` `2,66°` `17,32°` `17,36°` | `17,32°` |
| **8** (o que shipa) | `7,78°` `3,40°` `16,51°` `3,15°` `3,46°` | `3,46°` |
| 16 | `2,46°` `30,32°` `1,60°` `21,97°` `20,13°` | `20,13°` |

⭐⭐⭐ **O salto já existia a `substeps = 1`**, e as medianas **NÃO ORDENAM**: o ruído entre
realizações (`0,90°` a `30,32°`) engole qualquer tendência. ⇒ *a cura do tremor não trouxe este
defeito; ela tirou o tremor que o CAMUFLAVA.* Antes nada assentava — e **um salto não tem contraste
contra ruído**.

⚠️ **Consequência de produto: não há motivo para reverter os sub-passos.** As duas queixas são
defeitos diferentes, e a segunda é mais velha que a primeira cura.

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

## §5.10 — ⭐⭐⭐ A CAUSA do salto: **uma caixa de face sobre outra é um equilíbrio INSTÁVEL**

### §5.10.1 — O dossiê que a nomeia

`probe_o_dossie_do_salto`, peça `17`, `substeps = 8` — os tiques à volta do salto:

```text
  tique |    rot | |Δrot| | dist a' taca |    vao min | viz. | desalinho
    137 |   1.53° |  0.01° |     0.9967   |     0.2232 |   22 |    1.7°   ← FACE-A-FACE, parada
    150 |   1.71° |  0.01° |     1.0015   |     0.2236 |   22 |    1.9°
    157 |   1.93° |  0.06° |     1.0003   |     0.2240 |   22 |    2.1°
    159 |   2.18° |  0.15° |     0.9979   |     0.2245 |   22 |    2.4°
    161 |   2.73° |  0.33° |     0.9931   |     0.2258 |   22 |    3.1°
    163 |   4.52° |  1.23° |     0.9814   |     0.2303 |   22 |    5.8°
    164 |  10.26° |  5.74° |     0.9610   |     0.2459 |   22 |   16.5°
    165 |  18.43° |  8.17° |     0.9305   |     0.2683 |   12 |   44.6°   ← quina contra face
    174 |  18.88° |  0.07° |     0.9299   |     0.2671 |   12 |   44.9°   ← e fica la', imovel
```

**Três factos, e cada um mata uma explicação:**

1. ⛔ **A peça NÃO toca a taça** (`dist 1,00` contra a parede em `1,69`) ⇒ a hipótese dos **dois
   solvers a disputá-la** (§5.9.5 nº 2) está **REFUTADA** para este evento. O salto é peça×peça.
2. ⭐ **O apoio é FACE-A-FACE**: desalinho `1,7°` e vão `0,2232` contra o exacto `2 × LADO = 0,2200`.
   Duas caixas pousadas de face, imóveis durante 20 tiques.
3. ⭐⭐⭐ **O `|Δrot|` cresce `×1,4` por tique durante 8 tiques** (`0,06 → 0,10 → 0,15 → 0,23 → 0,33
   → 0,56 → 1,23 → 5,74 → 8,17`) e **pára exactamente nos `45°`**. *Isto não é um empurrão: é
   realimentação positiva a divergir de um equilíbrio instável, e o `45°` é onde ela encontra um
   ponto fixo estável.*

⇒ **A hipótese nº 1 do §5.9.5 confirma-se, e com o mecanismo NOMEADO:** a rotação de contacto é uma
projecção de posição sem nada que a trave, e sobre um apoio de **UM ponto** o equilíbrio de uma face
sobre outra tem ganho de laço `> 1`. *Um contacto pontual não resiste a binário nenhum* — a caixa
tomba até a quina encravar.

### §5.10.2 — ⛔⛔⛔ E a cura é a ORDEM DO DONO que EU matei com uma medição errada

> *«atacar o encosto de dois pontos agora»* — o dono, 2026-09-15.

Eu refutei-a no mesmo dia (doc 109 §8.7) com *«`0 %` dos contactos são face-com-face»*. **A refutação
tem dois defeitos, e qualquer um a invalida:**

| # | o defeito da medição | o que ela lia | o que lê corrigida |
|---|---|---|---|
| 1 | chamava `pump.cook.cook(..)` **directamente** ⇒ cena **sem sub-passos** | `0 %` face-a-face, p10 `15,8°` | **`9 %`**, p10 **`6,0°`** |
| 2 | conta o monte **ASSENTE** ⇒ conta **SOBREVIVENTES** | `0` no fim | **pico `4`** já a `substeps = 1` |

⭐⭐⭐ **O segundo é o que importa, e é uma lei nova para o repo:** *um censo tirado DEPOIS do evento
mede o resultado do defeito e lê-se como a ausência da precondição dele.* O `0 %` não dizia «esta
cena não tem apoios de face» — dizia **«esta cena DESTRÓI os apoios de face»**, e quem os destrói é
exactamente o defeito que o manifesto de dois pontos curaria. *A recusa media a própria consequência
daquilo que recusava.*

⇒ **a ordem do dono estava certa desde o início**, e a wave que ela pede volta à fila com o
mecanismo, o dossiê e a régua já construídos.

### §5.10.3 — O que a wave tem de entregar, e a régua que a julga

- **Onde:** [`ph2d-contact`](../../crates/ph2d-contact/) — o contacto entre duas caixas passa a
  devolver **o TRECHO** (dois pontos) onde as faces se sobrepõem, e não o ponto médio dele.
- **A cerca:** entre uma **quina e uma face** o contacto é um ponto **por geometria** e nada o
  desdobra — o manifesto só age onde há trecho, e `45°` continua a ser um ponto só. ⚠️ *Isso não é
  uma limitação: é o caso que já é estável.*
- **A régua:** [`probe_os_saltos`] / [`pior_salto`] — o salto de `23,26°` da peça `12` e o de
  `16,51°` da `17` têm de desaparecer, e as **quatro** réguas do §5.8.2 (tremor · rodopio · altura ·
  vão) têm de continuar dentro da banda. ⛔ *Uma cura que mate o salto e colapse o monte é a 17.ª
  recusa desta linha, não a primeira vitória.*
- ⛔ **A barra do gate nasce COM a cura** (§5.9.5): o lado aprovado ainda não existe.
