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

**Duas leituras, e a primeira é sobre a CENA e não sobre o motor:**

1. ⛔⛔ **Com `μ = 0` nem o oráculo assenta** — ele roda uma peça **`331°`**, quase uma volta
   inteira. A `=114` não escreve material nenhum nas peças, logo elas são **gelo** entre si
   (`Material::LISO`), e *uma pilha de quadrados sem atrito não assenta em motor nenhum: é Física.*
   ⇒ **toda medição desta obra corre com `μ ≥ 0,3`**, e a cena tem de ganhar atrito antes de
   qualquer veredito sobre o solver.
2. ⭐⭐⭐ **E com atrito o abismo é de CLASSE:** `0,0030 °/tique` contra os nossos `3,79` — **1 260×**.
   O giro fica na mesma banda (`24,82` contra `24,30..28,36`), ou seja o oráculo compra o silêncio
   **sem** pagar em rodopio, que é exactamente o par que as oito tentativas do doc 109 não
   alcançaram.

⇒ **A barra da obra:** `balanço ≤ 0,15 °/tique` **e** `giro ≤ 28°`, a `μ = 0,6`, na `=114`.

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
| **W0** | **o ATRITO na cena** | a `=114` (e o default do `source.shape`) deixam de ser gelo | ⛔ **sem isto nenhum número desta obra significa nada** — o §2 mostra que nem o oráculo assenta a `μ = 0`. É de PRODUTO, não de motor, e é a única wave que pode fechar sozinha. |
| **W1** | **o REPOUSO**, sozinho | um critério de adormecer por colunas + a medição contra a barra do §2 | é metade do resultado do oráculo e **não precisa de cache nenhum**. Se bastar, as W2–W4 não existem. |
| **W2** | a **CHAVE** do contacto | a identidade `(id, id, feição)` medida: quantos contactos sobrevivem ao tique seguinte num monte real | ⚠️ é o número que diz se o *warm starting* é sequer aplicável aqui — se as feições mudarem todos os tiques, não há o que aquecer. |
| **W3** | o **CACHE** | o `λ` acumulado, na morada que a W2 justificar | só depois de a W2 provar que há chave |
| **W4** | o **DISPOSITIVO** | o mesmo cache do lado do kernel | ⛔ sem ela a lei nova é CPU-only e custa o que vem curar |

⚠️ **A W1 é a fronteira da encomenda.** Ela é barata, mede-se contra uma barra que já existe, e pode
tornar as outras três desnecessárias — *medir se a composição já exprime o item antes de o construir*
(`CLAUDE.md` §5.0).

---

## §6 — ⛔ Recusas MEDIDAS que esta obra herda (não as reconstrua)

As oito do [doc 109 §8.14](109_o_colisor_na_forma.md), mais as três deste doc:

| # | ideia | veredito |
|---|---|---|
| 9 | **adoptar o `rapier2d` como motor da sim do Motion** | ⛔ tecto medido entre `1 600` e `6 400` peças contra `4,19 M` no dispositivo (§3) |
| 10 | medir a obra na cena como ela shipa (`μ = 0`) | ⛔ o oráculo roda `331°` ali: *a cena não tem resposta certa para medir contra* (§2) |
| 11 | o `angular_damping` como cura | ⛔ **inerte**: ele amortece a coluna `spin` e o contacto escreve o `rot` (doc 109 §8.12) |
