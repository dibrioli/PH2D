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

## §5.11 — ✅⭐⭐⭐ A CURA: o ENCOSTO DE DOIS PONTOS, e o trecho já estava a ser calculado

### §5.11.1 — ⭐⭐ A cura já era PRODUZIDA e a última linha deitava-a fora

O `ponto_de_contacto` de [`par.rs`](../../crates/ph2d-contact/src/par.rs) **já recortava** a face
incidente contra a de referência e obtinha os **dois** extremos do trecho — e devolvia
`(clo + chi) * 0,5`. O doc-comment de quem o escreveu já sabia porquê:

> *«A média, e não o vértice mais fundo: de chapa uma sobre a outra os dois vértices estão à mesma
> profundidade, e escolher um deles dá binário a uma pilha parada — ela tomba sozinha.»*

⇒ **escolheu-se o menos mau dos pontos ÚNICOS.** É a mesma forma que a `line/quadextract` já pagou
(*«a cura já era produzida e o desempate deitava-a fora»*).

⚠️⚠️ **E o que faltava não era o segundo ponto: era a PROFUNDIDADE PRÓPRIA de cada um.** Com
profundidades iguais os dois pontos entregam exactamente o binário do meio — *nada muda*. O que
endireita é a caixa **inclinada**, onde um canto está mais fundo que o outro: aí cada ponto pede a
sua correcção, e a diferença entre elas **é** o restaurador que faltava.

### §5.11.2 — O que shipa

- [`ph2d_contact::manifesto`](../../crates/ph2d-contact/src/par.rs) — o par entrega o **trecho**
  (1 ou 2 pontos, cada um com a sua profundidade). ⛔ Entre uma **quina e uma face** o contacto é um
  ponto **por geometria**, e isso não é limitação: os `45°` já são a configuração estável.
- O `contato` de sempre fica, **byte-idêntico**, como a leitura de UM ponto — o colisor de mundo e
  os censos usam-no. ⚠️ **Uma lei, dois leitores**: o eixo separador e a face incidente saíram para
  portas partilhadas, senão o ponto e o trecho passariam a falar de eixos diferentes.
- A acumulação por peça saiu para [`varredura.rs`](../../crates/ph2d-contact/src/varredura.rs) —
  o tecto de LOC reprovou a `720` e a cura foi **corte por responsabilidade**, ⛔ nunca uma isenção.

### §5.11.3 — ⛔⛔ DUAS formulações construídas, MEDIDAS e REFUTADAS

| formulação | o que faz | porque CAIU |
|---|---|---|
| **média + diferença** (a média translada, a diferença roda) | os dois pontos partilham UM `λ` | ⛔ **converge melhor e ERRA**: com `λ` partilhado o solver perde a capacidade de **redistribuir a pressão** ao longo do apoio, e é isso que deixa uma caixa apoiada pelo CENTRO ficar quieta. Medido: ela tomba `5,06°`. *Convergir e acertar são coisas diferentes.* |
| **somar** os dois pontos (sem média) | cada ponto aplica a correcção inteira | ⛔ sobrepassa: **cinco** gates de geometria caem de uma vez |

⇒ fica a lei do meio: **duas restrições independentes, cada uma com o seu `λ`, entrando pela MÉDIA**
— que é o esquema de Jacobi que o cabeçalho do módulo já declarava para os vizinhos.

### §5.11.4 — ⚠️ O preço: CONVERGÊNCIA, medida nas duas pontas

Cada ponto carrega o braço na massa efectiva dele (`k = w + invI·b²`, e `b` numa ponta é maior que
no meio), logo cada varredura corrige menos:

```text
                        8 varr. | 16 varr. | 32 varr. | 64 varr.
  caixas 0,5×2,0 (4:1)   −3,5 % |  −0,31 % | −0,002 % |  exacto
  quadrados da =114      −0,15 % | −0,003 % |  exacto  |  exacto
```

⛔ **Um resíduo que ENCOLHE com as varreduras é convergência, não viés** — e os gates medem as DUAS
pontas, em vez de afrouxar a barra até a de `8` passar. ⭐ O preço é do **ASPECTO** da caixa: uma
cena que empilhe formas esguias paga-o em sub-passos.

### §5.11.5 — ⭐⭐⭐ O resultado, medido

**O salto** (`probe_o_salto_contra_os_substeps`, 5 realizações por célula):

| substeps | UM ponto (antes) | DOIS pontos (agora) |
|---|---|---|
| 1 | `0,90 .. 8,93°` | **`0,69 .. 1,98°`** |
| 4 | `2,66 .. 17,45°` | **`1,25 .. 1,82°`** |
| **8** (shipa) | `3,15 .. 16,51°` | **`0,86 .. 0,96°`** |
| 16 | `1,60 .. 30,32°` | **`1,38 .. 1,49°`** |

⭐ Pior de toda a varredura: **`30,32° → 1,98°`**. E a **dispersão colapsou** (`3,15..16,51` ⇒
`0,86..0,96`): *o caos desapareceu, o que é a prova de que se removeu uma instabilidade em vez de a
baralhar.*

**As outras quatro réguas, a `substeps = 8`:**

| régua | UM ponto | DOIS pontos |
|---|---|---|
| tremor (°/tique) | `0,182 .. 0,219` | **`0,060 .. 0,168`** |
| rodopio (°/0,9 s) | `9,5 .. 31,7` | **`3,0 .. 3,1`** |
| altura `y` | `−2,53` | `−2,42` |
| vão típico | `0,2390` | **`0,2203`** |
| cozimento | `4,54 ms` | `6,73 ms` |

⭐⭐ **O vão bate `0,2203` contra o face-a-face EXACTO de `2 × LADO = 0,2200`** — a pilha passou a
encostar de **chapa** em vez de assentar em quinas. E `y = −2,42` está longe da altura de nascimento
(`−0,35`): ela não congela.

### §5.11.6 — O gate, e os DOIS que tinham o defeito escrito dentro deles

✅ **`a_settled_piece_does_not_jump`** — barra **`2,0°`**, o meio do vale medido `[0,96° .. 3,15°]`,
com prova red-first (colapsado a um ponto ele reprova nomeando a peça `19` a saltar `8,27°`).

⛔⛔ **E DOIS gates existentes reprovaram porque defendiam o defeito:**
`a_box_caught_off_centre_turns…` (nas duas crates) punha o centro da caixa **DENTRO** do apoio e
exigia que ela tombasse. A varredura que o apanhou:

```text
   centro |  apoiado? |    giro
     0,60 |       SIM |   0,000
     0,90 |       SIM |  −0,208   ← a fixtura velha, a exigir |giro| > 1
     1,05 |       NAO | −10,544
     1,10 |       NAO | −13,998
```

⇒ *um gate cuja fixtura encena o defeito passa a defendê-lo*, e a cura é a **fixtura**, nunca a
barra. Nasceu com eles o irmão que faltava — **`a_box_supported_under_its_centre_does_not_topple`**,
que é a metade que a cura compra.

## §5.12 — ✅⭐⭐⭐ O 6.º REPORT: o IMPULSO DO PAR (e a pergunta *«é a física do módulo Physics?»*)

> *«Bem melhor. Mas quando aumento bounciness as colisões não são realistas. Quando uma caixa bate
> na outra, não parecem ter a mesma massa. É como se uma fosse muito mais pesada que a outra? Vc
> está usando nossa própria física do módulo Physics ou criou outra?»* — o dono, 2026-09-15.

### §5.12.0 — A resposta à pergunta: **são DOIS motores**

| módulo | motor | dependência |
|---|---|---|
| **Physics** (`ph2d-physics`) | `rapier2d` 0.35, `enhanced-determinism` | ADR-0131 |
| **Motion** — `sim.step` / `sim.collide` | [`ph2d-contact`](../../crates/ph2d-contact/), PBD por projecção de posição | **zero** linhas de `rapier` |

*Não há uma linha em comum*, e a `=114` corre o segundo.

### §5.12.1 — ⛔⛔ O defeito, e ele é EXACTAMENTE o que o olho dele leu

A resposta de velocidade era **por PEÇA**, sobre a velocidade **ABSOLUTA** de cada uma, na direcção
em que a correcção de posição a tinha empurrado. Duas caixas iguais, a primeira a `1,0 u/s` contra
uma **parada**:

| bounciness | a que bate | a **PARADA** | momento (era `1,00`) |
|---|---|---|---|
| `0,00` | `0,0000` | **`0,0000`** | `0,00` |
| `0,50` | `−0,5000` | **`0,0000`** | `−0,50` |
| `1,00` | `−1,0000` | **`0,0000`** | **`−1,00`** |

⇒ **a peça atingida NUNCA recebia velocidade nenhuma.** Ela tinha `vn = 0`, o guarda *«só se
responde a quem se aproxima»* disparava, e o motor não lhe tocava. Não havia troca de momento — ele
não se perdia, **invertia-se**. *Uma peça parada era, literalmente, uma parede.*

⛔ E a direcção usada era `p − antes`, a correcção **TOTAL** da peça (a soma do que todos os vizinhos
lhe pediram): numa pilha apertada isso não é a normal de contacto nenhum.

### §5.12.2 — A lei: impulso normal + atrito de Coulomb, ambos por PAR

[`ph2d_contact::impulsos`](../../crates/ph2d-contact/src/impulso.rs), UMA passagem sobre os
contactos, nas posições em que eles de facto aconteceram:

```text
  vrel = (v_lo − v_hi) · n                    j = (1 + e) · vrel / (w_lo + w_hi)
  v_lo −= n · j · w_lo                        v_hi += n · j · w_hi
  jt   = clamp( vt / Σw ,  ±μ · vrel/Σw )     ← Coulomb, com o tecto no impulso NORMAL
```

⭐ **As três leis antigas continuam a valer por CONSTRUÇÃO**, não por promessa: quem nasce
sobreposto e parado não ganha velocidade (`vrel = 0`), só se responde a quem se aproxima
(`vrel > 0`), e um obstáculo devolve o salto inteiro (`w = 0` ⇒ `v' = −e·v`, exacto).

⭐⭐ **E o `dt` DESAPARECEU da assinatura do `resolve`.** A lei antiga precisava dele para pôr tecto
(`|Δp|/dt`) a uma velocidade que ela própria inventava; um impulso é limitado pela velocidade
**relativa que existe**. *Um parâmetro que deixa de ser preciso é a medida de quanto a lei nova sabe
a mais.*

Depois da cura, a mesma bancada:

| bounciness | a que bate | a parada | momento |
|---|---|---|---|
| `0,00` | **`0,5000`** | **`0,5000`** | `1,0000` |
| `0,50` | `0,2500` | `0,7500` | `1,0000` |
| `1,00` | **`0,0000`** | **`1,0000`** | `1,0000` |

Sem salto partilham a meias; com salto máximo **trocam**. ⚠️ **A barra do gate não é um número
escolhido: é a fórmula** (`v' = v(1∓e)/2`).

### §5.12.3 — ⭐⭐⭐ A metade que o impulso normal SOZINHO não tinha: o ATRITO na velocidade

⛔⛔ **Com só o impulso normal, a cena ficou PIOR do que com a lei errada:** o rodopio subiu de
`3,0..3,1°` para **`24,8..33,6°`**. A razão é que a lei velha, ao matar a velocidade ABSOLUTA, era um
**sorvedouro de energia** que também comia o deslize — e o atrito desta crate era **só posicional**:
ele desfaz o deslize já acontecido e não tira a velocidade que o vai repetir no tique seguinte.

⚠️ **E a primeira saída que tentei foi a ERRADA:** varri o `damping` do `sim.step` (que a cena nunca
usou), e **nenhum valor** passava as três réguas nas cinco realizações — `1,00` dava giro
`24,8..33,6`, `0,98` dava `19,3..41,2`. *Quando nenhum ponto de um knob resolve, o que falta não é o
knob.*

Com o **impulso tangencial de Coulomb**, e **sem tocar na cena**:

| damping | balanço | giro | salto |
|---|---|---|---|
| **`1,00`** (a cena como está) | `0,256..0,276` | **`2,5..2,7`** | `1,13..1,27°` |
| `0,97` | `0,234..0,276` | `2,4..2,7` | `1,00..1,15°` |

⭐ As quatro linhas são **iguais** ⇒ **a cena não precisa de arrasto nenhum**, e o rodopio ficou
**melhor** do que antes de toda esta jornada (`3,0..3,1`).

### §5.12.4 — ⚠️ A tabela dos sub-passos foi re-medida pela TERCEIRA vez no mesmo dia

| substeps | tremor | rodopio | altura | vão | cozimento |
|---|---|---|---|---|---|
| 1 | `42,5..48,0` | `15,5..63,4` | `−2,61` | `0,2005` | `0,99 ms` |
| 4 | `1,01..3,47` | `17,2..17,6` | `−2,46` | `0,2186` | `3,51 ms` |
| **8** (shipa) | **`0,246..0,327`** | **`2,6..2,8`** | `−2,43` | `0,2206` | `7,05 ms` |
| 16 | `0,091..0,236` | `3,2..3,5` | `−2,42` | `0,2202` | `14,05 ms` |

⛔⛔ **E o `4` DEIXOU de ser a opção viável que o §5.11 nomeou de manhã** (`1,01..3,47` de tremor
contra `0,150..0,217` então). *Uma alternativa medida sobre um substrato que mudou tem de ser
re-medida antes de ser oferecida outra vez* — com a física certa, a `8` é o **piso**.

### §5.12.5 — ⛔ Três sondas minhas nasceram partidas, e as três com a MESMA assinatura

| # | o furo | o que a sonda imprimia |
|---|---|---|
| 1 | `n` tirado do **primeiro** tique, onde a pilha ainda não nasceu (`rot` vazio ⇒ `n = 0`) | `0,000` em **todas** as células |
| 2 | `skip(120)` numa série que **não tem uma entrada por tique** (a `rot` só nasce com o 1.º contacto) ⇒ fatia **vazia** | `0,000` em todas |
| 3 | o salto corrido **sem** o arrasto que a linha varria | `1,65°` repetido nas cinco |

⇒ *um censo que mede NADA lê-se exactamente como «medi e está perfeito»*, e é a terceira vez nesta
jornada. As curas: **piso de população** (`assert n >= 20`), janela contada **a partir do fim**, e o
parâmetro varrido a chegar a **todas** as portas.

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

### §6.2 — ⭐⭐⭐ A CAUSA-RAIZ: o tecto de Coulomb posicional é **QUADRÁTICO no passo**

O cabeçalho do [`atrito`](../../crates/ph2d-contact/src/atrito.rs) já o escrevia sem tirar a
conclusão: *«`λn` numa pilha assente é a penetração que a gravidade fez naquele tique (`~g·dt²`),
então o atrito por tique é `μ·g·dt²` — pequeno de propósito»*.

⚠️⚠️ **`dt²` não é «pequeno»: é NÃO-INVARIANTE AOS SUB-PASSOS.** Medido, com só o atrito posicional
ligado, uma caixa a `1,0 u/s`:

```text
   μ   | sub | v final | travou | percorreu   (sem atrito nenhum: 0,5000)
  0,5  |  1  | 1,0000  | 0,0000 |  0,4833     ← 3,3 % de efeito
  0,5  |  8  | 1,0000  | 0,0000 |  0,4979     ← 0,4 % de efeito
  1,0  |  1  | 1,0000  | 0,0000 |  0,4663
```

⇒ **ele nunca remove velocidade** (a coluna `v final` é `1,0000` em todas as linhas) e o efeito
residual de posição **enfraquece `8×`** de `sub = 1` para `sub = 8`. *A cura do tremor desta manhã
dividiu o atrito por oito, e nenhuma régua desta cena o via.*

⭐⭐ **E a comparação que fecha o caso:** o impulso de velocidade tem tecto `μ·g·dt` — **linear**,
logo invariante aos sub-passos. É por isso que ele funciona e o outro não.

### §6.3 — ⛔⛔ O que o dono viu: as duas leis DUPLICAM a metade translacional

A/B por mutação, sobre a pilha (5 realizações por célula, deslize mediano por tique em fracção do
lado · largura do monte):

| μ | **ambos** (o que shipa) | só o POSICIONAL | só o de VELOCIDADE |
|---|---|---|---|
| `0,00` | `0,0929` · `2,21` | `0,0929` · `2,21` | `0,0929` · `2,21` |
| `0,25` | `0,0080` · `1,08` | `0,1084` · `2,23` | **`0,0010` · `0,92`** |
| `0,50` | `0,0105` · `1,18` | `0,0754` · `2,12` | `0,0043` · `0,98` |
| `1,00` | `0,0174` · `1,40` | `0,0523` · `2,44` | `0,0046` · `1,07` |

⭐⭐⭐ **Três leituras, e nenhuma era esperada:**
1. **O posicional sozinho é INERTE na pilha** — a largura fica em `~2,2` a TODO `μ`, exactamente
   como com atrito zero, e o deslize anda ao acaso sem tendência.
2. **O de velocidade sozinho é `4–8×` melhor que os dois juntos** (`0,0010` contra `0,0080`).
3. ⇒ **juntos são PIORES que um deles sozinho.** O posicional empurra a peça de volta pelo deslize
   que já aconteceu, o de velocidade tira a velocidade que o repetiria — a mesma coisa contada
   duas vezes, e a correcção de posição passa a **ultrapassar**, o que se lê como mais deslize.

⚠️ **A não-monotonia (`μ = 1` pior que `μ = 0,25`) sobrevive nos dois casos, mais suave com só o de
velocidade** (`0,0010 → 0,0046` contra `0,0080 → 0,0174`).
⚠️ E **a LARGURA a crescer com `μ` pode ser física correcta** — um monte com atrito tem ângulo de
repouso maior e não escorrega para uma pilha compacta. *Não a conto como defeito sem uma régua que
separe «não escorregou» de «não assentou».*

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

### §7.1 — As três peças que a fizeram funcionar

| # | a peça | porque ela era obrigatória |
|---|---|---|
| 1 | a **repartição** translação/rotação, `kt = w + invI·(r·n)²` | sem ela um disco **PÁRA A SECO** em vez de rolar — a energia era destruída, não convertida |
| 2 | o tecto lê **também a PENETRAÇÃO** (`max(vrel, pen/dt)`) | ⛔⛔ a 1.ª redacção prendia-o à velocidade de aproximação, e num contacto **assente ela é ZERO**: *o atrito ficava ligado no embate e desligado no repouso* |
| 3 | o `vt` inclui a **rotação PRÓPRIA** (`ω·(r·n)`) | sem ela uma bola a girar no sítio **não esfrega** — o centro está quieto e o atrito lia zero |

⚠️ **A nº 2 é a mesma degenerescência que a lei antiga tinha, um nível acima**: ali o tecto era a
penetração (`~g·dt²`, quadrática), aqui era a aproximação (zero em repouso). *A força normal de um
contacto em repouso tem de ser lida de onde ela de facto está*, e neste modelo ela está nas duas.

### §7.2 — ⭐⭐⭐ O resultado: monótono, plano, e MAIS BARATO

O `Friction` do cartão, 5 realizações por célula (deslize mediano por tique em fracção do lado · a
largura do monte):

| μ | ANTES (as duas leis) | DEPOIS (uma só) |
|---|---|---|
| `0,00` | `0,0929` · `2,21` | `0,0929` · `2,21` |
| `0,25` | `0,0080` · `1,08` | **`0,0039`** · `1,05` |
| `0,50` | `0,0105` · `1,18` | **`0,0036`** · `1,00` |
| `0,75` | `0,0146` · `1,33` | **`0,0035`** · `1,00` |
| `1,00` | `0,0174` · `1,40` | **`0,0036`** · `1,00` |

⭐ **A curva deixou de subir depois de `0,25`: ela desce e depois fica PLANA.** A `μ = 1` são
**`4,8×` menos deslize** e um monte **`29 %` mais apertado. *«No máximo parece desligado»* acabou.*

E as cinco réguas da cena, a `substeps = 8`:

| régua | antes | depois |
|---|---|---|
| tremor | `0,246 .. 0,327` | **`0,014 .. 0,024`** (`13×`) |
| rodopio | `2,6 .. 2,8` | **`0,7 .. 2,4`** |
| altura `y` | `−2,43` | `−2,43` |
| vão típico | `0,2206` | `0,2192` (face exacta `0,2200`) |
| **cozimento** | `7,05 ms` | **`6,17 ms`** |

⭐⭐ **Tudo melhora e ainda fica mais barato** — a matemática do atrito posicional saiu do laço.

E as bancadas analíticas continuam certas: uma caixa a deslizar trava a `μ·g` (`0,1241` contra a
teoria `0,125`), invariante aos sub-passos, e numa torre o peso propaga (`0,1241` / `0,0370` /
`0,0208`).

### §7.3 — ⛔ Os SEIS gates que mudaram de ENDEREÇO, nunca de exigência

O atrito mudou de **UNIDADE**: era uma correcção de POSIÇÃO e passou a ser um impulso de
VELOCIDADE. Seis gates mediam a metade translacional em `p − p0` — *o sítio certo da lei antiga, e o
sítio vazio da nova*:

- `the_rolling_split_gives_the_spin_twice_the_slide` — a razão `2` é a mesma, lida em `Δv`;
- `a_spinning_disc_rubs_against_the_floor_even_standing_still` — o empurrão passa a ser um travão;
- `the_step_tells_the_contact_how_much_the_spin_already_turned` — idem, na coluna `vel`;
- e o arnês `corre_com_atrito_girando` corre as **duas** portas, porque é isso que o produto faz.

⛔⛔ **E um gate da cena passava por COINCIDÊNCIA.** O `the_controls_the_announcement_names_do_what_it_says`
exigia que `Width 1,6` alargasse o monte `10 %` sobre `1,0`. Varrida, a grandeza **não é sequer
monótona** (`0,6 → 1,4150 · 1,0 → 1,2873 · 1,6 → 1,4108`): *a largura do monte é governada pela
TAÇA*, não pelo colisor. A régua certa é o **vão LATERAL** — a distância em `x` à vizinha à mesma
altura —, que dá `0,2596 → 0,3151 → 0,3520`, monótono e com margem. ⚠️ O vão ao vizinho **mais
próximo** também não serve: ele satura no espaçamento VERTICAL, que o `Width` não toca.

### §7.4 — ⛔⛔ E o VALE do gate do salto DISSOLVEU-SE, o que muda o que ele defende

A barra de `2,0°` do §5.11 saiu do vale `[0,96° .. 3,15°]` entre o manifesto ligado e desligado. Com
o atrito a funcionar, as duas configurações **deixaram de se separar** aos `substeps = 8`:

```text
  substeps | manifesto LIGADO             | manifesto DESLIGADO
         8 | 0,94 0,97 2,18 1,00 0,98     | 2,05 1,04 1,36 1,04 1,25
        16 | 0,74 0,82 0,76 0,77 0,82     | 1,38 1,39 4,67 2,50 7,89
```

⛔ **Subir a barra para cobrir o `2,18` tornaria o gate VAZIO** (o pior do lado reprovado a `8` é
`2,05`). ⇒ mudam-se duas coisas, **e nenhuma é a exigência**:

1. a grandeza passa a ser a **MEDIANA de cinco realizações** — a disciplina que esta cena exige em
   todo o resto; o `2,18` é um **sorteio**, e a mediana do mesmo conjunto é `0,98`;
2. o que ele defende passa a ser **o DEFEITO DO DONO** (`23,26°`, mediana `3,46°`), e não o
   manifesto — que tem gate próprio e directo na crate
   (`a_box_supported_under_its_centre_does_not_topple`).

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

### §8.1 — ✅ O tecto do salto volta a `1`

Ordem directa, feita: `BOUNCE_MAX` `2,0 → 1,0`, revertendo a ordem dele próprio de 2026-09-13.

⚠️ **A medição que abriu a faixa para `2` fica INTACTA no doc-comment da constante.** Ela respondia
*«o que acontece acima de `1`?»* e a resposta não mudou — *o que mudou foi o veredito de PRODUTO, e
ele não precisa de desmentir a medição para valer*. A tabela fica porque, no dia em que alguém a
quiser reabrir, ela é o que poupa a medição toda.

⛔ **Quatro gates afirmavam o `2` e mudaram de número, nunca de propriedade**, e um deles ensinou
uma lei: o `the_bounce_reaches_the_new_ceiling_in_the_scene` tinha a escada `[0, 1, BOUNCE_MAX]` —
ela media *«o tecto novo passa do antigo»* quando o tecto era `2`, e ao reverter **os dois degraus
de cima colapsaram no mesmo número**, deixando o gate a exigir que uma altura fosse maior que ela
própria. ⇒ *uma escada escrita com o valor de ontem mede o produto de ontem*; hoje ela DERIVA do
tecto. ⚠️ E a barra do topo passou a ser contra o `0`: medida, a resposta **satura** perto do
tecto (`0,9 → 0,572` contra `1,0 → 0,510`), e exigir margem ali mediria a saturação.

### §8.2 — ⭐⭐⭐ «Umas rodam, outras não»: a CAUSA está medida

Uma linha por caixa na `=114` (`probe_quem_roda_e_quem_nao`): **4 de 25 rodam menos de `0,1°`** em
toda a queda (`0,006` · `0,045` · `0,047` · `0,104`), contra `1,2°`–`4,8°` das outras — e a
`inv_inercia` é **idêntica** nas 25 (`123,97`), logo ⛔ **não é o `Lock Rotation`**.

A causa é a limitação que o doc 109 §6 declara: **não há velocidade angular.** Medida pela porta do
produto (`probe_a_caixa_atingida_continua_a_rodar`) — uma caixa atingida FORA DO CENTRO por outra:

```text
  tique |  rot do alvo | Δrot
      4 |      0.5341° | 0.5341    ← durante o contacto
      8 |      0.5341° | 0.0000
     …  |      0.5341° | 0.0000    ← 35 tiques, nunca mais
```

⇒ **ela leva um safanão de meio grau e CONGELA.** Quem fica encostado acumula rotação tique após
tique; quem só é tocado de passagem fica com um tremor e nada mais. *É exactamente o que o olho do
dono separou.*

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

### §9.1 — O instrumento: mutação do `const`, e a bancada que foi CONSTRUÍDA e DEITADA FORA

As leis são um argumento da porta (`impulsos(.., leis)`) e a escolha vive num `const` do `sim.step`.
Varrê-las a partir da cena obrigaria a uma **bandeira global** lida dentro do solver — *o defeito
que o `remesh_with` da `line/sculpt3d` pagou por escrito, e que alcança todo chamador*. ⇒ a
varredura é `backup → mutar o const → correr a sonda → restaurar`, que é a forma sancionada pelo
`CLAUDE.md` §2 para uma edição derivada de medição. A sonda é a
`motion_state_pilha_demo::obra::probe_a_linha_das_leis`, e cada célula custa uma recompilação.

⛔⛔ **A alternativa barata foi construída, medida e deitada fora no mesmo dia.** Uma bancada de
pilha DENTRO da `ph2d-contact` varre à vontade e não vale nada:

- a 1.ª fixtura era uma grelha `5 × 5` alinhada num caixote do tamanho dela. Ela cai a direito e
  pousa alinhada: `rodopio 0,00` com a lei de hoje e `0,46` com a velocidade angular, contra os
  `77,5..153,2` que a cena media. ⚠️ *Uma fixtura onde não acontece nada não distingue lei nenhuma*
  — ela teria elegido a primeira coluna da tabela;
- dando-lhe rampas e ângulos de chegada ela passou a ler `2,0×` onde a cena mede `~40×`. ⛔ **Afinar
  uma fixtura até ela concordar com a resposta que se quer é a pior espécie de medição.**

⇒ *o instrumento é a cena.*

### §9.2 — ⭐⭐⭐ A tabela

Cinco realizações do berço (`±0,003`), `substeps = 8`, janela `121..174`:

```text
  lei                                       |    tremor     |   rodopio   | salto | y     | ms
  ------------------------------------------|---------------|-------------|-------|-------|-----
  HOJE (o controlo, a lei de 15/09)         | 0,036..0,049  |  2,0..  2,8 |  1,58 | −2,42 | 0,62
  + angular                                 | 0,454..1,992  | 38,7..123,3 |  1,69 | −2,55 | 0,65
  + angular + por_ponto                     | 0,365..0,855  | 32,5.. 66,4 |  1,65 | −2,49 | 0,61
  + angular + 8 iteracoes                   | 0,122..0,345  |  6,5.. 30,4 |  1,57 | −2,54 | 0,65
  + angular + por_ponto + 8 iteracoes       | 0,840..0,905  | 42,1.. 43,6 |  0,87 | −2,47 | 0,62
  + … + tecto_por_lambda                    | 0,802..0,819  | 40,7.. 41,6 |  1,66 | −2,45 | 0,61
  por_ponto SÓ (sem angular)                | 0,018..0,022  |  1,0..  1,2 |  0,86 | −2,43 | 0,64
  8 iteracoes SÓ (sem angular)              | 0,139..0,200  | 10,4.. 12,5 |  1,24 | −2,42 | 0,68
  ⭐ EM_VIGOR (angular+ponto+8it+sem gposic) | 0,016..0,028  |  2,6..  3,1 |  2,42 | −2,43 | 0,70
```

⚠️ A coluna do SALTO está no tecto de «assentada» de `0,2°/tique`, que era o da cena quando a
varredura correu — ver §9.8: no tecto de hoje ela lê `1,57` (HOJE) contra **`0,74`** (EM_VIGOR).

⭐ **A EM_VIGOR bate o controlo em tudo**: tremor `2×` melhor, rodopio igual, salto melhor, altura e
vão iguais, e `0,08 ms` a mais por tique.

### §9.3 — ⭐⭐⭐ O MECANISMO: DUAS leis de rotação a correr ao mesmo tempo

A peça que faltava é a mais simples de escrever e a que eu não tinha medido: **desligar a rotação
POSICIONAL**. Com ela ligada a mesma lei lê `rodopio 42,1..43,6`; sem ela, **`2,6..3,1`**.

A razão é que elas não são a mesma grandeza. A separação de posição (`separate`) roda a peça para
resolver a sobreposição e **acumula esse ângulo no `rot` sem lhe dar velocidade nenhuma** — é
rotação que nenhum atrito pode travar, porque não existe no nível em que o atrito age. Enquanto o
contacto não tinha velocidade angular ela era a ÚNICA rotação que havia (e o controlo prova-o: a lei
de 15/09 sem rotação posicional lê `tremor 0,448` e `rodopio 25,1..26,6`, muito pior). A partir do
momento em que a rotação tem velocidade, ela passa a ser uma segunda rotação a somar-se à primeira.

⇒ **a lei: quando a rotação é uma VELOCIDADE, a passagem de POSIÇÃO não roda ninguém.**

⚠️ **Preço declarado:** uma caixa em penetração pura **com velocidade zero** deixa de rodar — a
despenetração é só translação. Sob gravidade isso é invisível (é o que a `=114` mede), e foi isso
que mudou o endereço de um gate (§9.9).

⚠️ **Dívida nomeada:** o `Leis::giro_posicional` é honrado pelo CHAMADOR, porque o dono dele é o
`separate`, que ainda não recebe as leis. Se ele alguma vez decidir mais alguma coisa, passa a ser
argumento daquela porta.

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

### §9.7 — ⭐⭐⭐ E a pilha CHEGA A PARAR — a régua é que media o transiente

A pergunta que nenhuma régua desta linha sabia fazer. Alongando a `duration` da zona e lendo as
réguas janela a janela (`probe_a_pilha_chega_a_parar`):

```text
  janela (tiques) | rodopio HOJE | rodopio ANGULAR | tremor HOJE | tremor ANGULAR
  ----------------|--------------|-----------------|-------------|----------------
      120..180    |     2,10     |      41,53      |    0,036    |     0,763
      180..240    |     1,74     |       8,62      |    0,031    |     0,118
      240..300    |     2,39     |       4,04      |    0,162    |     0,068
      300..360    |     2,74     |       2,70      |    0,018    |     0,046
      360..420    |     2,95     |       2,22      |    0,016    |     0,038
      480..540    |     1,16     |       1,22      |    0,032    |     0,020
```

⭐ A partir dos `300` tiques as duas leis são **indistinguíveis**, e em duas janelas a angular lê
MELHOR. ⇒ *o `41,53` da primeira janela era o transiente de assentamento, e a `=114` reinicia aos
`3,0 s` — mesmo a meio dele.* ⚠️ Isto **não** justificava sozinho ignorar o número: o artista via a
pilha a rearranjar-se e nunca assente, que é a queixa original. Quem o resolveu foi o §9.3.

### §9.8 — ⭐⭐ A régua do SALTO mudou de tecto, e o critério APERTOU

Com a lei nova o `a_settled_piece_does_not_jump` lia `2,42°` contra a barra de `2,0`. O dossiê diz o
que ela estava a apanhar: `18,43° → 20,81° → 18,14°`, **um arco suave que VOLTA ao sítio**, com
`|Δrot|` de `0,11..0,19°/tique`. O tecto de «assentada» era `0,2°/tique` — e a `0,2` uma peça anda
`2,4°` nos `12` tiques da janela, logo **conta como parada**.

```text
  quieto | lei de 15/09 | lei EM_VIGOR | eventos (realizacao central)
  -------|--------------|--------------|-----------------------------
   0,200 |         1,58 |         2,42 | 3 844 / 2 659
   0,100 |         1,57 |         0,74 | 3 506 / 2 365   ← o tecto de hoje
   0,050 |         0,36 |         0,40 | 2 757 / 2 122
   0,010 |         0,08 |         0,08 |   304 /   271
```

⚠️⚠️ **Toda a diferença entre as duas leis vive na banda `0,1 < |Δrot| ≤ 0,2`** — abaixo dela elas
lêem o mesmo, e a nova chega a ler melhor. E o evento que o tecto velho elegia é a peça `19` no
tique **`61`** — *o instante em que a pilha ATERRA* —, com `antes 0,184` · `depois 0,200` (colada ao
tecto) e **`desloca 0,15`**: a viajar `15 %` da própria largura durante o «salto».

⭐ **O tecto apertou de `0,2` para `0,1`; a BARRA (`2,0°`) não se mexeu.** Um critério que exclui uma
peça a viajar `15 %` da largura dela não é uma barra mais baixa: é a barra a voltar a medir o que o
nome dela diz. O gate não fica vacuoso (`2 365` eventos concorrem), e o salto que o dono fotografou
(parada · `23°` · parada) continua a passar o filtro por construção.

⭐⭐ **E o `Salto` ganhou a DERIVA** — o giro LÍQUIDO das janelas dos dois lados — porque o
`antes`/`depois` mede o pior `|Δrot|` **por tique**, e isso deixou de separar *parada* de *a rodar
devagar* no dia em que a rotação passou a persistir. Ela aperta o critério nos dois lados; ⛔ e não
entra no gate com barra própria, porque a varredura dela (`inf`/`1,0`/`0,5`/`0,25`) não tem vale —
*sem vale medido não se escreve uma barra*.

⚠️ **E a sonda e o gate estavam a imprimir `2,42` e `0,74` para a MESMA grandeza**, porque cada um
tinha a sua cópia do tecto. Hoje é uma porta só (`salto::QUIETO`, com quatro leitores) — *quando uma
página imprime duas medidas da mesma grandeza e elas discordam, isso É o achado.*

### §9.9 — Os gates que mudaram de ENDEREÇO, nunca de exigência

Dois gates do `sim.step` caíram, e os dois pela MESMA razão: **a rotação mudou de coluna**, de `rot`
(um ângulo do tique) para `spin` (uma velocidade), e eles liam `rot` depois de UM passo.

- `the_step_hands_the_contact_the_slide_and_the_material` — passa a ler o `spin`. É a tradução
  directa, e é a mesma lição que o `delta_vel` da `ph2d-contact` já tinha pago quando o atrito saiu
  da posição (§7.3).
- `a_box_caught_off_centre_turns_unless_the_column_locks_it` — a fixtura tinha a caixa **parada** em
  penetração pura, e com o §9.3 isso já não roda ninguém. Ela passa a PRESSIONAR a caixa contra a
  beira (uma velocidade para baixo, que é o que a gravidade faz) e a marchar `12` passos. ⚠️ E a
  metade travada deixou de exigir que a coluna `rot` **não nasça**: a fixtura agora autora-a, logo
  ela sai sempre — o que a lei tem de provar é que fica em ZERO. *Exigir a ausência mediria a
  fixtura, não a lei.*

⚠️ **E o `contact_tests.rs` estourou o tecto de LOC (`709` de `700`) por acumulação** — curado por
corte de responsabilidade: os gates AFIRMAM (`contact_tests.rs`, `454`) e as sondas IMPRIMEM
(`contact_probes.rs`, `265`), com o arnês a viver no irmão.

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

### §10.1 — ⭐ Antes de construir: o §7.5 já estava curado, e a sonda escondia-o

A `probe_auditoria_do_rolamento` lia `ω·R = 0,0000` em todo `μ` — *«a bola derrapa para sempre»*.
Ela carregava o `spin` de volta mas **lia o giro como `Δrot` no último sub-passo**, e nunca
realimentava o `rot`: media `(spin − spin_anterior)·dt ≈ 0`. Lida no `spin`:

```text
   μ   |   v    |  ω·R   | rola?
  0,00 | 1,0000 | 0,0000 | nao   ← gelo: desliza, como deve
  0,25 | 0,6666 | 0,6667 | SIM   ← o ⅓/⅔ de manual, num passo
  1,00 | 0,6666 | 0,6667 | SIM
```

⇒ **o §9 curou o §7.5, e a régua lia o produto de ontem** — a mesma mudança de endereço
(`rot` → `spin`) que partiu dois gates no §9.9, agora numa sonda e com o comentário dela a afirmar
*«o contacto NÃO tem velocidade angular»*.

### §10.2 — A lei: a da TAÇA, pela porta da taça

`rolamento_um` corre em cada restrição **depois** do atrito (lê o `ω` já corrigido por ele, a ordem
da taça) e **mesmo com `μ = 0`** (achatar-se não depende de esfregar). ⭐ **Cada peça é travada
contra o PRÓPRIO giro, com o PRÓPRIO `Rolling`** — `atrito::rolamento(ω/invI, μr, λn, r·n)`, termo
a termo o que a `sim.collide` faz. O tecto lê o MESMO normal que o Coulomb (`normal()`, uma porta
para os dois tectos). Sem velocidade angular não corre (a lei de 15/09 fica ao bit), e com
`Rolling = 0` — o default do cartão — o tecto é zero: **a `=114` aprovada não se move**.

### §10.3 — ⭐⭐⭐ As duas metades do app dão a MESMA resposta

Um disco `R = 0,2` já a rolar a `1 u/s`, `g = 4`, `μ = 1`, segundos até parar — pelo `sim.step`
(peça × obstáculo no solver de pares) contra a tabela que a taça mediu (`ROLLING_MAX`):

```text
  rolamento | 1 passo | 8 sub-passos | a taça
       0,00 |     inf |          inf |    inf
       0,02 |   18,58 |        18,56 |  18,57
       0,05 |    7,43 |         7,43 |   7,43
       0,10 |    3,72 |         3,71 |   3,72
       0,25 |    1,48 |         1,48 |   1,50
       0,50 |    0,73 |         0,74 |   0,78
       1,00 |    0,37 |         0,37 |   0,43
       1,50 |    0,23 |         0,25 |   0,33
       2,00 |    0,23 |         0,25 |   0,33
       4,00 |    0,23 |         0,25 |   0,33
```

⭐ Até `0,25` concordam a `≤ 1,5 %` e são **invariantes aos sub-passos**. Acima separam-se por
construção: ali quem trava é o Coulomb (teórico `v/(μ·g) = 0,25 s`), a taça lê `0,33` porque o
contacto dela não acontece em todos os tiques, e este lê `0,23–0,25`. ⭐ **E o tecto `1,5` vale
para este consumidor também** — a coluna satura no mesmo sítio.

⛔ **A 1.ª fixtura mentia:** a `0,02` lia *«não pára em 30 s»*. A série no tempo mostrou o disco a
desacelerar **exactamente** ao ritmo da taça até aos `5 s` e depois a **cair pela ponta** da prancha
(meia-largura `4`; `y = −1098` aos 28 s). Um rolamento fraco anda `~9` unidades antes de parar, e a
taça mede contra um plano infinito. *Uma fixtura mais curta que o fenómeno lê o fim dela como um
defeito do produto* — a prancha tem hoje meia-largura `40`.

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

### §10.5 — Os gates

- `a_piece_rolling_on_a_piece_stops_as_it_does_on_the_bowl` (`sim.step`) — a barra é a TAÇA
  (`0,02`/`0,05`/`0,10`, folga `2 %`, `7×` o maior desvio), com `1` e `8` sub-passos, e *sem
  rolamento rola para sempre*. Mutação (apagar a lei): **RED** — *«a bola nunca parou»*.
- ⭐ `the_rolling_on_the_card_calms_the_pile` (`=114`) — **o gate que a forma do par não teria
  passado**: a razão do rodopio COM/SEM o botão (janela `240..300`) tem de ficar abaixo de `0,5`, com
  o vale `0,04` (aprovado) · `47` (forma do par). Mutações: apagar a lei → **RED** (`1,65`/`0,47`);
  inverter o sinal → **RED** (`26 311`). ⚠️ Piso: sem o botão a pilha tem de girar `> 0,05`, senão
  uma razão sobre zero não mede nada.

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

### §11.1 — A W3b está DISSOLVIDA, com o tecto do que ela ainda podia comprar

A memória (`ph2d_contact::warm`, W3a) foi encomendada como o chão da W3b — *«o `λ` acumulado a
entrar no solver»* — para curar o **zumbido** da pilha. O zumbido foi curado por outro lado: os
**sub-passos** (§5.8), e depois o solver de velocidade do §9 passou a acumular o `λ` DENTRO do
sub-passo. O que o aquecimento ainda podia comprar é **custo**, e esse tecto já estava medido nas
tabelas do §9 (`=114`, lei em vigor):

```text
  iteracoes |  1   |  2   |  4   |  8   |  16  |  32  |  64
  ms/tique  | 0,59 | 0,62 | 0,65 | 0,66 | 0,73 | 0,82 | 1,73
```

⇒ aquecer o `λ` até uma varredura só dar o que dão oito poupa, **no melhor caso**, `0,07 ms` de
`0,66` — **`≈ 11 %`** de um cozimento que já é barato. E ela traz uma cerca de CORRECÇÃO (§5.4):
sem a coluna `id` — que a `=114` não tem — a chave é o índice, e um nascimento ou uma morte aquece o
**contacto errado**, que é pior que não aquecer. ⇒ *11 % não compra uma cerca que parte cenas com
`sim.spawn`.*

⛔ **Apagada**, com os `9` gates dela (`ph2d-contact` passa de `39` para `30`, exactamente). Ela vive
no commit **`76dfd1947`**. ⭐ **O gatilho que a traria de volta tem nome:** um solver de contacto no
DISPOSITIVO — um Jacobi em GPU converge devagar e é onde o aquecimento compra ordens de grandeza, e
a morada dela já foi medida para isso (uma coluna de largura fixa, `24 B` por elemento).

### §11.2 — O `Saida::salto`: um gate a defender uma saída sem leitor

A varredura de posição recolhia o *«salto efectivo de cada peça»* (o maior dos pares que ela tocou)
e o devolvia na `Saida`. **Ninguém o lia** desde o §5.12, quando o ressalto passou a viver no
impulso do par (`atrito::salto` dentro do `monta`). E o gate
`the_bounce_of_the_liveliest_pair_reaches_the_piece_that_touched` **defendia essa saída** — *um gate
verde sobre uma saída sem consumidor é um controlo morto com certificado.*

Hoje o gate mede onde o salto ACONTECE: um disco a `1 u/s` contra um chão fixo, `salto = 0,8`, volta
a `+0,8` (`Δv = 1,8`). Mutação (o ressalto a zero no impulso): **RED**, `Δv = 1,0`.

⚠️ **E o gate da fronteira do rolamento mudou de afirmação.** O
`the_piece_against_piece_contact_ignores_rolling_bit_for_bit` declarava *«o contacto peça×peça
ignora o rolamento»* e prometia reprovar *«se um dia alguém der velocidade angular a este
solver»* — **e não reprovou quando o §9 o fez**, porque o arnês dele corre o `Leis::HOJE` e o produto
passou a correr o `EM_VIGOR`. *Um gate cujo sujeito não é a lei que o artista vê não afirma nada
sobre ela.* Hoje ele defende o que ainda é verdade: **sem velocidade angular o rolamento é inerte ao
bit** (o controlo de toda medição da família não se move).

### §11.3 — O que a obra do contacto deixa

Nada aberto que mude o ecrã. ⚠️ O `Leis` fica com cinco campos e **uma** combinação em produto
(§9.10) — eles são as colunas das tabelas deste doc, e a varredura por mutação do `const` é o
instrumento que as lê.
