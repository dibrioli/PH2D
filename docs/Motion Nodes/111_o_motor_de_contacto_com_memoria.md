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
| **W3** | o **CACHE** | o `λ` acumulado por contacto persistente, na morada a decidir (§4.2) | ⭐ **é a obra**, e a W2 já a autorizou |
| **W4** | o **DISPOSITIVO** | o mesmo cache do lado do kernel | ⛔ sem ela a lei nova é CPU-only e custa o que vem curar |

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
