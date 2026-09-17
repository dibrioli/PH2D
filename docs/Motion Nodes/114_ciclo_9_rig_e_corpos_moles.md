# 114 — CICLO 9 · RIG & CORPOS MOLES — «Coisas que se seguram»

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, nesta ordem, e **o tutorial É o
> smoke**. Este doc é o do ciclo: cada passo escreve a secção dele aqui.
>
> **Estado:** passos **1** (grupo) e **2** (auditoria) FECHADOS em 2026-09-17. Os passos 3–7 têm
> plano na §5 e ainda não começaram.

---

## §1 — O GRUPO (passo 1), **contado** e não copiado

⚠️ A linha do doc 103 §5 (`rig.*` · `soft_body` · `verlet_rope` · `wave` · `boids`) é uma
**verificação**, nunca a fonte: o grupo sai do que o artista VÊ na paleta. Contado pelo censo
(`the_source_palette_census`, [`motion_fontes_probe.rs`](../../crates/ph2d-app-motion/src/motion_fontes_probe.rs)),
em 2026-09-17, são **DEZ** nós — e eles vivem em **duas categorias diferentes**:

| categoria na paleta | nó | cartão | o que ele é |
|---|---|---|---|
| **Source** | `rig.skeleton` | Skeleton | EMITE a corrente de juntas (`parent`, `len`, `rot`) |
| **Source** | `motion.soft_body` | Soft Body | emite a nuvem de um corpo mole |
| **Source** | `motion.verlet_rope` | Verlet Rope | emite a corda |
| **Source** | `motion.wave` | Wave | emite a superfície que ondula |
| **Source** | `motion.boids` | Boids | emite o bando |
| **Transform** | `rig.fk` | FK | resolve a pose pela árvore de pais |
| **Transform** | `rig.ik_2bone` | IK 2-Bone | cinemática inversa fechada e EXACTA |
| **Transform** | `rig.fabrik` | FABRIK | a iterativa, para N juntas |
| **Transform** | `rig.rubber_hose` | Rubber Hose | o membro de borracha |
| **Transform** | `rig.skin_deformer` | Skin | a pele que segue os ossos |

⭐ **A partição em duas categorias é o grupo, não um acidente de arrumação:** cinco nós **produzem**
uma coisa que se segura e cinco **agem** sobre ela. É essa a premissa do tutorial — *«coisas que se
seguram»* — e é ela que dá a forma da cena do passo 6.

⚠️ **A folha de conferência [16_rig.md](89_conferencia/16_rig.md) abre com um aviso que EXPIROU:**
*«esta família está DEFERIDA por decisão do Enio … nenhum item abaixo vira wave agora»* (2026-08-09).
O doc 103 §5 (**05/09**, ordem do dono) põe a família como ciclo 9, e é a ordem mais nova que manda.
⇒ a folha volta a ser tabela de trabalho; o cabeçalho dela é corrigido na wave que a tocar.

---

## §2 — A RESIDÊNCIA (passo 2, primeira metade): **1 de 10 chega ao dispositivo**

A lei 1 de todo ciclo (doc 103 §2) manda cada nó dizer ONDE corre. Censo por crate, contra o canal
de kernel do registry (`register_gpu_kernel`, `ph2d-node-registry/src/gpu_channels.rs`), em
2026-09-17:

**O RETRATO do grupo** (`audit_the_rig_group`, 2026-09-17) — as colunas saem do **registry**, que é
quem responde ao planeador:

| nó | params | no cartão | **device** | portas | efeito |
|---|---|---|---|---|---|
| `motion.boids` | 16 | 14 | ✅ **sim** | 4→1 | Temporal |
| `motion.soft_body` | 10 | 10 | ⛔ não | 4→1 | Temporal |
| `motion.verlet_rope` | 11 | 11 | ⛔ não | 3→1 | Temporal |
| `motion.wave` | 10 | 10 | ⛔ não | 3→1 | Temporal |
| `rig.skeleton` | 4 | **5** | ⛔ não | 0→1 | Pure |
| `rig.fk` | **0** | 0 | ⛔ não | 1→1 | Pure |
| `rig.ik_2bone` | 2 | 2 | ⛔ não | 2→1 | Pure |
| `rig.fabrik` | 1 | 1 | ⛔ não | 2→1 | Pure |
| `rig.rubber_hose` | 1 | 1 | ⛔ não | 2→1 | Pure |
| `rig.skin_deformer` | 1 | 1 | ⛔ não | 3→1 | Pure |

⇒ **1 de 10 no dispositivo.**

⚠️ **DOIS instrumentos independentes dizem o mesmo**, e é por isso que a linha acima se pode
escrever: o retrato pergunta ao **registry** (coluna `device`), e o censo por crate pergunta a quem
chama `register_gpu_kernel` — ali o `motion.boids` aparece (`lib.rs:648`), as outras nove não, e o
**controlo positivo** é que o instrumento vê as **100** crates que registam kernel (o
`motion.oscillator` entre elas). *Um censo que mede zero e um que mede «nenhum» lêem-se igual; dois
que concordam por caminhos diferentes, não.*

⚠️ **E a contagem de params NÃO se conta por `grep ParamSpec`:** a primeira leitura desta auditoria
fez isso e leu **11 · 12 · 11 · 17** onde o registry lê **10 · 11 · 10 · 16** — um a mais em cada, do
próprio tipo na assinatura. ⭐ E o erro tinha o sinal contrário no `rig.skeleton`, que declara **4**
params e pinta **5** no cartão: o quinto é o **text param `branches`** (a ramificação, fechada em
2026-08-12), e um text param não é um `ParamSpec`. *A fonte é o registry.*

⛔⛔ **E isto é PIOR do que o mesmo número noutro grupo, por causa da POSIÇÃO dos nós na cadeia.**
O ciclo 7 mediu *«seis dos dez levavam a cadeia inteira para a CPU»* porque um nó de aparência é o
**último** de um grafo. Aqui a partição da §1 diz que cinco são **fontes** — o **primeiro** nó — e
cinco são **transformes** no meio: ⇒ *todo grafo que segure seja o que for corre inteiro na CPU*,
com a única excepção de um bando de boids que ninguém deforme a seguir.

### §2.1 — E agora quem o diz é o PLANEADOR (W0, fechada)

⭐ As duas leituras acima são do **registo**. A que vale para o artista é a do **planeador** sobre a
cadeia montada, e ela existe: `the_rig_group_route_only_improves`
([`motion_rig_probe.rs`](../../crates/ph2d-app-motion/src/motion_rig_probe.rs)), com a lista
`NA_CPU` a nascer com **nove** nomes e a razão de cada um. **Ela passa** ⇒ o planeador confirma, nó
a nó, o que o registo dizia.

⚠️⚠️ **A forma da cadeia é diferente para cada metade, e a escolha é o MÉTODO:**

| metade | cadeia medida | porquê |
|---|---|---|
| **PRODUZ** | `X → scale → output` | ela é o primeiro nó; um `X` sem kernel derruba tudo |
| **AGE** | `motion.grid → scale → X → output` | mede-se atrás de uma fonte que **já está no dispositivo** |

⛔ **A metade que AGE não pode ser medida atrás do `rig.skeleton`**, que é a cadeia que o artista de
facto escreve: ele próprio não tem kernel, logo a cadeia cai para a CPU **por causa da fonte** e o
nó medido nunca é a causa — *uma régua em que o sujeito não pode falhar sozinho não mede o sujeito*.

⭐ **E a catraca tem CONTROLO POSITIVO dentro:** sem ele, um `cadeia_no_dispositivo` que devolvesse
sempre `false` deixaria as nove asserções verdes e a décima nunca correria — *a catraca ler-se-ia
como a funcionar sobre um instrumento morto*. **Prova de mutação 2 de 2**: a régua cravada em
`false` mata o controlo, e tirar **um** nome da `NA_CPU` faz o laço acusá-lo pelo nome.

---

## §3 — O QUE FALTA (passo 2, segunda metade): **duas metades com doenças OPOSTAS**

A auditoria contra o estado da arte está feita e é boa: a folha
[16_rig.md](89_conferencia/16_rig.md) mede os seis `rig.*` contra **Rive**, **Spine**, **Blender** e
**RubberHose/DUIK**, com fonte por afirmação, 23 linhas, cercas grepadas e uma §SUPERAR. O que este
ciclo acrescenta é a outra metade do grupo e a leitura de conjunto.

### §3.1 — Os seis `rig.*`: MUITO poder, quase nenhum dial

**9 params em 6 nós** (4 · 0 · 2 · 1 · 1 · 1) — ⭐ **reconferido contra o registry em 2026-09-17: a folha de 2026-08-09 continua exacta.** O IK do Spine sozinho tem 6 propriedades mais 3
referências de osso; o Rive põe `Strength` em **cada um** dos 7 constraints dele.

⭐⭐⭐ **E a folha já nomeou a causa mecânica, numa linha (§0 dela):** o catálogo sabe **LER**
qualquer coluna por nome e sabe **ESCREVER** exactamente cinco (`X`, `Y`, `Rotation`, `Size`,
`Opacity`, via `motion.drive`) ⇒ **`parent` e `len` — as duas colunas que FAZEM de uma corrente um
esqueleto — não têm ESCRITOR nenhum.** Isso explica **seis** «inexprimíveis» de uma vez
(comprimento por osso · peso por osso · limite de junta · rigidez por junta · stretch · compress).

> *A família não é magra por natureza: é magra por não ter uma caneta.*

### ⛔⛔ A CANETA EXISTE — e a §0 da folha está REFUTADA por medição (2026-09-17)

A frase acima era verdade **em 2026-08-09** e já não é. O `motion.drive` ganhou desde então o canal
**`Custom…`** (`CH_CUSTOM = 9`) mais o text param `column`, e com eles escreve **qualquer** coluna
com os **oito** modos dele (`Add · Set · Multiply · Subtract · Divide · Min · Max · Remap`). A folha
mede-o como *«cobre X/Y/Rotation/Size/Opacity e nada mais»* — eram cinco, hoje são cinco **mais
todas**.

⭐ **MEDIDO pelo caminho do produto**
(`a_caneta_que_a_folha_diz_nao_existir_ja_escreve_o_comprimento_do_osso`), na cadeia
`rig.skeleton(joints=4, length=1) → motion.drive(Custom, "len") ← value.instance_field(Ramp)`, com o
`rig.fk` a resolver a pose:

| | comprimentos que o `rig.fk` resolve |
|---|---|
| **sem a caneta** (o CONTROLO) | `1,000 · 1,000 · 1,000` |
| **com** `drive(Custom, "len")` | **`0,333 · 0,667 · 1,000`** |

⚠️ **Mede-se a GEOMETRIA e não a coluna:** ler `len` de volta provaria que o `motion.drive`
escreveu, e a pergunta é se **o solver obedece**. São duas afirmações e só a segunda fecha a célula.
⚠️ E o **CONTROLO é metade do valor** — sem a cadeia sem-escritor a devolver comprimentos iguais,
*«eles variam»* não distinguiria a caneta de um esqueleto que já nascia irregular.

⛔⛔ **MAS a caneta é NECESSÁRIA e não SUFICIENTE, e é aqui que o diagnóstico fica mais preciso que
o da folha.** Das seis células que ela atribui à caneta ausente:

| célula | o que falta HOJE |
|---|---|
| comprimento por osso | ✅ **nada** — medido acima |
| ramificação | ✅ nada — o text param `branches` fechou-a em 2026-08-12 (a própria folha o diz) |
| **peso por osso** (P0) | ⛔ o **LEITOR**: o `rig.skin_deformer` lê `P`·`parent`·`len`·`rot`·`wrot` e **nenhuma coluna de peso** |
| limite de junta | ⛔ o LEITOR, no `rig.fk`/`rig.fabrik` |
| rigidez por junta | ⛔ o LEITOR, no `rig.fabrik` |
| stretch / compress | ⛔ o LEITOR, no `rig.ik_2bone` |

⇒ ⭐⭐⭐ **o que bloqueia quatro das seis não é escrever a coluna: é o SOLVER não a ler.** *Uma
ausência afirmada sem olhar a API é um palpite com cara de medição* — e desta vez a nota errada era
sobre um nó que a própria casa tinha alargado três semanas depois de ela ser escrita.

**Os três buracos de maior alcance, na ordem em que a folha os mede:**

1. **`Strength` / `Mix`** — Rive tem em **7 de 7** constraints, Spine em **4 de 4**, nós em **0 de
   6**. *Um item, sete lugares.* ⭐ E a §SUPERAR da folha mostra que aqui ele nasce **melhor que a
   referência**: nas três, o `Mix` é um número keyado; aqui um esqueleto é uma corrente ordinária,
   logo o strength é uma **COLUNA** — e a família `field.*` inteira (com gizmo de canvas) já a
   produz. *«Um IK cuja influência desvanece com a distância de uma caixa que o artista arrasta»*
   não existe em nenhuma das três.
2. **O peso por osso do `rig.skin_deformer`** (P0 na folha) — *«o item que todo rigger encontra no
   primeiro dia»*. Hoje há **um** expoente global (`falloff`, inteiro 1..8) onde Spine e Rive
   **pintam** peso por vértice e por osso.
3. **A ALÇA no canvas** (P0) — hoje o alvo do IK é *«o primeiro elemento de uma corrente»*, uma
   convenção **invisível** onde as três referências têm uma alça que se arrasta. ⚠️ É UI e não
   param, e é o que torna a família **usável**.

⚠️ **E um item da folha é um DEFEITO, não um pedido:** o `motion.look_at` escreve o heading de
**MUNDO** numa coluna que o `rig.fk` lê como **LOCAL** ⇒ acerta a raiz e **rasga** tudo abaixo dela
(cerca 2 da folha: `rot` é LOCAL, `P` é DERIVADO). Ele está derivado do código e **pendente de gate
red-first**.

### §3.2 — Os quatro corpos moles: MUITOS dials, nenhuma placa

Params pelo **registry** (§2), 2026-09-17: `soft_body` **10** · `verlet_rope` **11** · `wave`
**10** · `boids` **16**, e os quatro são `Temporal`. ⇒ **a doença é a oposta** da dos `rig.*`: aqui
os dials existem — e quase todos chegam ao cartão — e o que falta é a **rota**.

| nó | o que está aberto hoje | fonte |
|---|---|---|
| `motion.wave` | ⏳ **[Bug #7](BUGS_motion_nodes.md)**, report do dono: a fileira de mar de **4 ondas não mostra cristas diferentes** — a BOIA é um passa-baixo e apaga as camadas finas (excursão `0,228` contra `0,377` da de 1 onda). A alavanca medida é o **calado**; ⛔ **não** a densidade, que reabre o Bug #6 | CLAUDE.md §5 |
| `motion.wave` · `motion.boids` | ⏳ **tectos por medir** — o bloco Z do [doc 91](91_os_tetos_que_ninguem_mediu.md) fechou 7 células e deixou estas duas de fora | doc 91 |
| `motion.verlet_rope` | a composição **sub-passos × `damping`**, medida e **não curada de propósito**; ⚠️ o sub-passo local dele chama-se `solver_substeps` e **não** `substeps` — enquanto usava a mesma chave, o app corria as duas leis e a corda caía **4,8× menos** que os gates dela medem | CLAUDE.md §5 |
| `motion.soft_body` | ✅ o P1 dele FECHOU (a porta `shape`: a nuvem que chega é a forma de repouso) — a folha 03 está a **zero P1** | CLAUDE.md §5 |

---

## §4 — A LEITURA DE CONJUNTO (o que decide este ciclo)

⭐⭐⭐ **O grupo tem duas metades com doenças opostas, e uma cura comum.**

- Os `rig.*` têm **o solver e não a interface** (a folha di-lo por escrito: *«temos o SOLVER — falta
  a INTERFACE»*). O que lhes falta escreve-se em **colunas**, e a coluna não tem caneta.
- Os corpos moles têm **a interface e não a placa**. O que lhes falta é a **rota**.

⛔⛔ **E a §SUPERAR item 6 da folha — *«o primeiro item é um escritor genérico de coluna»* — foi
MEDIDA e está FECHADA: ele já existe** (§3.1). O que a medição pôs no lugar dela é mais estreito e
mais accionável: **o que falta é o LEITOR no solver**, quatro vezes. ⇒ a W1 deixa de ser *«construir
a caneta»* e passa a ser *«dar ao `rig.skin_deformer` o peso por osso»*, que é o **P0** da folha e o
item que ela chama *«o que todo rigger encontra no primeiro dia»*.

⭐ *E isto poupou a wave inteira que eu ia escrever*: o `CLAUDE.md` §5.0 manda medir se a composição
já exprime o item antes de o construir, e aqui ela exprimia — *o que se perde ao não reconferir não
é tempo, é construir o que já existe*.

---

## §5 — O PLANO (passos 3 a 7), na ordem em que serão atacados

| wave | o que é | porquê primeiro |
|---|---|---|
| **W0** ✅ | A **catraca da rota do grupo** (§2.1) — FECHADA em 2026-09-17, 2 de 2 mutações a sangrar | *Sem a régua, toda a §2 era uma leitura de registo em vez de uma medição do planeador.* |
| ~~**W1**~~ ⛔ | ~~O escritor genérico de coluna~~ — **REFUTADA em 2026-09-17: ele já existe** (§3.1) | *A composição já o exprimia; medir antes de construir poupou a wave inteira* |
| **W1′** ✅ | O **peso por osso** do `rig.skin_deformer` — o LEITOR que faltava (P0 da folha) — FECHADA em 2026-09-17 (§6) | Era o que a medição pôs no lugar da W1, e é o item *«que todo rigger encontra no primeiro dia»* |
| **W2** ✅ | **`Strength`/`Mix` como COLUNA** nos constraints — FECHADA em 2026-09-17 (§7) | Um item, três lugares — e nasce melhor que as três referências |
| **W4** ⏳ | A **razão nomeada** de cada corpo mole estar na CPU (§8) — ✅ feita; o **preço** espera máquina calma, e a sonda está comitada | Lei 1 do doc 103 §2 |
| **W4-bis** | O **tecto `MAX_SIDE = 60`** do `motion.wave`, com o recurso nomeado e medido | ⛔ **Vem ANTES do kernel** (§8): não se decide uma placa para um campo que não pode crescer |
| **W5** | A **MEDIÇÃO** do grupo (passo 5): tabela CPU · dispositivo · passes · objectos/ms, com `loadavg` ao lado | §0.0 |
| **W6** | O **TUTORIAL em PDF** (passo 6) + a cena de smoke | O tutorial É o smoke |
| **W7** | O **smoke do dono** (passo 7) | **Enio** |

⛔ **A ALÇA no canvas (P0 da folha) fica NOMEADA e fora desta lista até o dono decidir:** ela é
**UI e não param**, e um gesto de canvas novo neste módulo compete com a selecção — a decisão é de
produto, com o preço na folha.

⚠️ **A cena de smoke deste ciclo é a `=120`** — ⛔ **conte-a no
[roteador](../../crates/ph2d-app-motion/src/motion_state_demo_router.rs)** antes de a escrever
(`MAX_DEMO_LEVEL` era `119` em 2026-09-17), nunca nesta linha: o gate
`no_two_smoke_scenes_claim_the_same_level` mede o **piso**, não o tecto.

---

## §6 — CERCAS que este ciclo herda (grepadas, não lembradas)

As dez da folha [16_rig.md](89_conferencia/16_rig.md) `CERCAS:` valem inteiras. As três que mais
provavelmente mordem uma wave deste ciclo:

1. **Os leaves `fk.rs`/`pose.rs`/`trig.rs` são BYTE-IDÊNTICOS nas 6 crates** — *«a cópia não pode
   divergir, é o contrato dela»* (doc 42 §4). ⚠️ **Toda mudança de leaf é × 6**, e uma wave que
   edite `fk.rs` numa crate só **nasce quebrada**.
2. **Um solver escreve uma POSE, nunca posições** (doc 41 §3) — mutante provado: escrever `P`
   directo deixa **16 de 17** testes verdes. É isso que mata o `motion.mixer` como rota de
   `Strength`, e é por isso que a cadeia da W2 mistura **ângulos**.
3. **`break_collinearity`** (`fabrik/lib.rs:104-129`) — a degenerescência é o **default** aqui, não
   um caso exótico. ⛔ Qualquer `strength`/`pre` novo corre **depois** dela.

---

## §7 — A MEDIÇÃO (passo 5)

⏳ Por correr — é a W5. A tabela vem para aqui com o `loadavg` ao lado de cada leitura (§0.0), pela
sonda que a W0 deixar escrita.


---

## §6 — ✅ W1′: o ENVELOPE POR OSSO (o P0 da folha 16)

A lei do `rig.skin_deformer` passa de `w_j ∝ 1/d_j^falloff` para **`w_j ∝ envelope_j / d_j^falloff`**,
com o `envelope_j` a sair da coluna opcional **`bone_weight`** do `rest`. É o *envelope weight* por
osso do Blender, os *Tendons* do Rive e o *dropoff* por influência do Maya — onde nós tínhamos **um
expoente global**.

⭐ **Ausente ⇒ `1,0` ⇒ a lei de ontem AO BIT**, e o gate afirma-o com `assert_eq!` sobre os bits, não
com uma barra de tolerância: `x * 1,0` é exacto em IEEE-754, logo **nenhum documento já autorado
muda de pixel**.

⛔ **O nome NÃO é `weight`, e isso foi medido:** `weight` já é coluna deste repo — a espessura da
fonte, escrita pelo `source.text`. *Uma colisão de nome de coluna passa MUDA* (§5.0), e ali ela
juntaria a pele de um esqueleto ao peso de um glifo.

⚠️ **O envelope viaja no OSSO e não num índice**, porque o construtor de ossos **FILTRA**: a junta
sem pai não produz osso, logo **o osso `k` não é a junta `k`**. Quem indexasse a coluna pelo índice
do osso daria a cada um o envelope do vizinho — em silêncio, com a pele a deformar-se com ar de
certa. Há gate cuja fixtura separa as duas leituras (o envelope da RAIZ não pode mudar nada).

⛔⛔ **A FIXTURA era metade do gate, e a primeira redacção estava degenerada:** os três gates usavam
uma rotação **RÍGIDA** do esqueleto, e numa rotação rígida todo osso sofre a MESMA mudança de
referencial ⇒ *a pele sai no mesmo sítio seja qual for o peso*. O teste vizinho, escrito há meses,
**afirma isso por escrito**. Com ela, o gate do envelope a zero reprovava sobre produto **correcto**
e o gate da raiz passava por **vácuo**. ⇒ a pose passou a DOBRAR numa junta do meio.

⭐⭐ **E a costura foi medida pelo caminho do produto** — o §5.0 diz que *nenhum instrumento deste
repo pergunta se o VALOR chega a um consumidor*:

```text
motion.grid ──────────────────────────────────────────> skin.in
rig.skeleton ─[drive(Custom,"bone_weight") ← rampa]─> fk ──> skin.rest
             └─[drive(Custom,"rot")        ← rampa]─> fk ──> skin.posed
```

A caneta escreve, o valor atravessa um `rig.fk` e o solver obedece. **Prova de mutação 3 de 3**: com
o leitor a ignorar a coluna o gate de costura lê **`0e0` de desvio** — exactamente zero, que é o
sinal mais forte que uma régua destas pode dar.

⏳ **ABERTO e nomeado:**

- **Todos os envelopes a zero** caem no ramo que já existia (*repartir por igual em vez de dividir
  por zero*), escrito para a inalcançabilidade GEOMÉTRICA. Dizer *«o ponto não se move»* é o que um
  rig zerado quer, e a função de pesos não o sabe exprimir — **decisão de produto**, com o preço no
  cabeçalho do nó.
- **Descoberta:** a coluna escreve-se digitando `bone_weight` no campo *Column* do `motion.drive`,
  como toda coluna deste catálogo. Não há fileira de painel — e isso é a cerca 3 da folha (*um dial
  aqui seria uma 2.ª fonte de verdade*), não um esquecimento.

---

## §7 — ✅ W2: a FORÇA de uma restrição, e ela é a coluna que já existia

O `Strength` que o **Rive** põe em **7 de 7** constraints e o `Mix` que o **Spine** põe em **4 de 4**
— nós tínhamos **0 de 6**. Agora os três solvers (`rig.ik_2bone` · `rig.fabrik` ·
`rig.rubber_hose`) misturam a pose resolvida com a que entrou, junta a junta.

⭐⭐⭐ **E o canal NÃO é uma coluna nova: é o `falloff`.** Foi a segunda vez neste ciclo que medir
antes de construir poupou trabalho — a folha pedia um `strength`, e `falloff` é exactamente *«quanto
este elemento participa»* no vocabulário desta casa: é o que o `motion.falloff` escreve, o que a
família `field.*` inteira produz **com gizmo de canvas**, e o que o `motion.scale` já lê com a mesma
convenção (*ausente vale `1,0`, efeito cheio*).

⇒ ⭐⭐ **o item `SUPERAR:` nº 1 da folha cai de graça.** Nas três referências o `Strength` é **um
número keyado**; aqui *«um IK cuja influência desvanece com a distância de uma caixa que o artista
arrasta na tela»* é **um fio**, e não existe em nenhuma delas.

**A escada, gateada nos três nós:**

| força | o que sai |
|---|---|
| `0` | a pose que ENTROU, **ao bit** — a restrição não faz nada |
| `½` | *estritamente* entre as duas (senão «mistura» seria um interruptor) |
| `1` **ou ausente** | o solve de sempre, **ao bit** |

⚠️ **A mistura é de ÂNGULOS e nunca de posições** — a cerca 4 da folha, que a própria célula do
`Strength` já invocava: um `P` misturado discorda do `rot` e a cadeia sai **rasgada**. É também por
isso que o `motion.mixer` não servia como rota.

⛔⛔ **E uma mutação REFUTOU um comentário meu, escrito minutos antes.** Eu justifiquei o atalho de
força-cheia como *«o que mantém a saída byte-exacta»*; apagando-o, **os três gates da escada
continuam verdes** — a exactidão vem da FORMA (`a·(1−t) + b·t` em `t = 1` é `a·0 + b·1 = b`,
exacto), e não daquela linha. *Uma linha que a mutação não consegue matar não é lei, é comentário
com sintaxe de código* — o atalho fica com o nome que merece (poupa uma alocação e uma passagem no
caso comum) e o comentário foi corrigido no mesmo commit.

⚠️ **A forma importa e a alternativa é a armadilha:** o `a + (b−a)·t` dos manuais dá, em `t = 1`,
`a + (b−a)` — que **arredonda** quando `a` e `b` estão longe. *Uma das duas formas mexe em silêncio
com todo rig do repositório.*

⚠️ O `t` é preso a `0..1`: uma força é uma **mistura**, e um campo que entregue `2,0` não pode fazer
uma restrição ultrapassar o próprio solve.

**Prova de mutação 2 de 2** (a segunda documentada como não-sangrante de propósito, acima), e a
folha `pose.rs` continua **byte-idêntica** nas três crates, que é a cerca 10.

---

## §8 — W4: por que cada corpo mole está na CPU — **a razão NOMEADA, nó a nó**

A lei 1 do doc 103 §2 manda que *um nó do grupo que caia para a CPU saia do ciclo com a razão
nomeada e o preço medido*. A §2 diz **que** eles caem; esta diz **porquê**, e as três respostas são
**diferentes** — o que importa, porque duas delas são trabalho e uma é uma decisão.

### ⛔ Primeiro, o que NÃO é a razão: o estado

A leitura fácil é *«eles guardam estado entre tiques, e o dispositivo não faz isso»*. **É falsa, e
o próprio repo a desmente:** o canal [`StateSelect`] existe no substrato — *«a porta que traz a
população semente»* mais *«a porta que traz o estado evoluído do tique anterior»* — e o `sim.zone`
usa-o hoje. O ciclo 5 mediu **10 de 12** nós de simulação no dispositivo. ⇒ *estado no dispositivo
é um problema resolvido nesta casa*, e os três corpos moles têm exactamente a forma de portas que
aquele canal descreve (um `state` ao lado das portas de entrada).

### A razão de cada um

| nó | o que o prende | natureza |
|---|---|---|
| **`motion.wave`** | ⭐ **NADA de estrutural.** O `step` é um estêncil **explícito**: cada célula lê `h` e `h_prev` (as matrizes do tique anterior) e escreve uma matriz NOVA — nenhuma célula lê o que esta passagem escreveu. É o caso embaraçosamente paralelo, o trabalho clássico de uma placa | **trabalho por fazer** (o kernel), não um bloqueador |
| **`motion.soft_body`** | **Reduções de stream inteiro** — o centroide ponderado e o `A_pq` do *shape matching* são somas sobre TODOS os pontos antes de qualquer ponto poder mover-se. ⭐ E o canal existe: o `ReduceSpec` (o canal DEFORMER, ADR-0126) é exactamente *«as reduções de stream inteiro que o kernel deste nó lê»* | **trabalho por fazer**, maior — duas passagens em vez de uma |
| **`motion.verlet_rope`** | ⛔⛔ **Gauss-Seidel**, e está escrito no código: a relaxação faz `pos[i]` e `pos[i+1]` **no sítio**, e a restrição seguinte lê o que a anterior acabou de escrever (o comentário do nó nomeia-o, e compara-se ao Vellum). *Não existe forma de o correr em paralelo que dê o mesmo resultado* | **BLOQUEADOR real** |

### ⚠️ E o bloqueador da corda é uma DECISÃO, não uma dificuldade

As duas saídas paralelas conhecidas — **Jacobi** (todas as restrições a partir da mesma fotografia)
e a **coloração** (ímpares, depois pares) — são leis **diferentes**: para o mesmo número de
iterações a corda fica **mais mole**, porque uma correcção deixa de ver a anterior. ⇒ pôr a corda na
placa **move todos os gates daquela crate**, incluindo os que medem quanto ela cai — e o `CLAUDE.md`
§5 já regista que a composição *sub-passos × `damping`* dela foi **medida e não curada de
propósito**.

⇒ **Não é um kernel que falta: é um veredito sobre se a corda pode mudar de lei.** Fica como
**decisão do dono**, com as duas alternativas nomeadas e o preço escrito.

### ⛔⛔ E a pergunta do kernel do `motion.wave` está MAL POSTA enquanto o tecto dele for `60`

A escada de preço bateu num muro que não era o esperado: pedir lado `64` ao `motion.wave` devolve
**`3 600` células e não `4 096`**, porque ele prende o lado em **`MAX_SIDE = 60`**. E a justificação
escrita ao lado da constante é *«field cost is O(rows·cols)»* — **uma lei de crescimento, não um
recurso**, que é precisamente o que o §0.0 proíbe.

⇒ ⭐⭐⭐ **A ordem do trabalho inverte-se.** *«Vale a pena um kernel de GPU para o campo?»* é uma
pergunta sobre o que acontece quando ele fica GRANDE — e hoje ele **não pode ficar grande**, porque
o tecto o para primeiro. Escrever WGSL para um campo preso a `60×60` é o caso canónico do §0.0: **o
caminho lento a definir o tecto do rápido**, no nó cuja física é a mais adequada a uma placa de todo
o grupo.

⇒ **primeiro o tecto (com a medição que diz de que recurso ele é), só depois o kernel.**

⚠️⚠️ E o tecto estava escondido de uma maneira que vale registar: o
[doc 91](91_os_tetos_que_ninguem_mediu.md) — *a auditoria dos tectos que ninguém mediu* — **nomeia o
`motion.wave`**, porque auditou o `MAX_DT` dele. A palavra `MAX_SIDE` não aparece lá uma única vez.
*Uma auditoria de tectos responde pelos tectos que olhou, e um nó que aparece numa lista de dívida
PAGA lê-se como um nó sem dívida.*

### ⏳ O preço, e por que ele ainda não está aqui

O relógio da CPU destes três **não foi medido** — a máquina esteve acima de `load 5` (`§5.0`) desde
que esta secção foi escrita, e uma leitura ali não vale nada. A sonda corre pelo mesmo vigia que
fechou a tabela do ciclo 8 (`ferramentas/medir_quando_calmo.sh`). ⚠️ **Sem esse número, «pôr o
`motion.wave` na placa» é uma aposta e não uma decisão** — é ele que diz se vale um kernel ou se o
estêncil já cabe num quadro na malha que o artista usa.

---

## §9 — ✅ W4-bis: o TECTO DO CAMPO, medido — e as três decisões do grupo

Ordem do dono (*«decida pelo padrão ouro buscando o melhor resultado e performance»*). As três
decisões abaixo saem todas da mesma tabela, medida pelo caminho do produto em **RELEASE**.

### A medição

| lado | células | quadro/ms | ns/célula | % de um quadro de 60 fps |
|---|---|---|---|---|
| 16 | 256 | 0,001 | 5,7 | 0,01 % |
| 32 | 1 024 | 0,004 | 3,6 | 0,02 % |
| **60** | **3 600** | **0,010** | 2,8 | **0,06 %** ← o tecto ANTIGO |
| 128 | 16 384 | 0,046 | 2,8 | 0,28 % |
| 256 | 65 536 | 0,240 | 3,7 | 1,44 % |
| **512** | **262 144** | **1,001** | 3,8 | **6,00 %** |

⭐⭐ **A leitura correu a `load 16,17` e é conclusiva pelo lado SEGURO.** Sob carga um relógio só
pode ler **pior**, logo um número que cabe ali cabe na máquina calma — é a lei do `CLAUDE.md` §5.0
(*um green sob carga é conclusivo; um red sob carga não prova nada*) usada na direcção em que ela
funciona. ⭐ E o `ns/célula` é **plano** (`2,8`–`3,8`): o estêncil é `O(células)` sem joelho nenhum.

### Decisão 1 — `motion.wave::MAX_SIDE` sobe de `60` para **`512`**

⛔ A `60` este nó usava **`0,06 %`** de um quadro: o tecto estava **~100× abaixo de qualquer
recurso**, e a justificação escrita ao lado dele (*«field cost is O(rows·cols)»*) era uma lei de
crescimento e não um recurso — §0.0 exactamente.

⭐⭐⭐ **E o número não foi escolhido: é o do IRMÃO.** O `motion.soft_body` é a outra simulação de
grelha 2D desta casa, o tecto dele é **`512`**, e ele custa **`2,32 ms`** a `512²` contra os
**`1,00`** deste. ⇒ *o nó mais BARATO tinha o tecto mais apertado*. Alinhá-los faz a família
responder a mesma coisa à mesma pergunta, e o recurso passa a estar nomeado: **o orçamento do
quadro** (`6 %` para o campo sozinho).

⚠️ **E o tecto DIGITÁVEL passou a existir.** Antes, `clamp` e slider valiam ambos `60`: *a
capacidade do motor acabava onde o dedo acabava*. Agora são o par que o irmão já shipa e que o
[doc 91](91_os_tetos_que_ninguem_mediu.md) pôs em 25 params — clamp `512`, slider `64` (a faixa de
autoria, estritamente abaixo).

⛔⛔ **E uma MUTAÇÃO SOBREVIVEU, e ela mudou o desenho dos gates.** O gate que escrevi dentro da
crate do nó monta os `Params` **à mão** e chama o `simulate` — ele nunca atravessa o
`clamp(2, MAX_SIDE)`, que vive no `eval`. Encolhi esse clamp para `60` e o gate ficou **VERDE**:
*um arnês que monta o estado à mão mede a LEI e não a PORTA*, a forma que esta casa já pagou quatro
vezes noutros módulos. ⇒ o gate do produto vive agora onde um grafo se coze de verdade
(`o_campo_chega_ao_tecto_pela_porta_do_produto`), e com a mutação ele diz *«o artista pediu 512×512
e recebeu 3 600 células»*. **3 de 3 mutações sangram.**

### Decisão 2 — a `motion.verlet_rope` FICA em Gauss-Seidel

Medido: **`278` ns por ponto, e a coluna é PLANA** (`276,9` · `277,8` · `278,1`). Uma corda com os
`100`–`200` pontos que alguém de facto autora custa **`0,03`–`0,06 ms`** — `0,3 %` de um quadro.

⇒ ⭐⭐ **trocar a lei por Jacobi ou por coloração compraria paralelismo numa contagem que ninguém
alcança, e pagaria com o RESULTADO**: para o mesmo número de iterações a corda fica **mais mole**,
porque uma correcção deixa de ver a anterior. O padrão-ouro aqui é *não mexer* — a convergência
exacta é a feature, e os gates que medem quanto ela cai ficam todos de pé.

⚠️ **É uma decisão e não uma omissão**, e fica registada com o número ao lado. O gatilho que a
reabriria está nomeado: alguém querer cordas de dezenas de milhares de pontos (tecido, cabelo em
massa), que é outro produto.

### Decisão 3 — o kernel de GPU do `motion.wave` é o PRÓXIMO tecto, não este

O §0.0 diz que *quem manda no tecto é o dispositivo*. Hoje não há kernel, logo `512` é o que o
**caminho de referência** sustenta — e está nomeado como tal. ⭐ O que a medição mudou é o **preço da
espera**: com `512²` a custar `1 ms` na CPU, o artista ganha **`73×` mais células hoje**, sem uma
linha de WGSL e sem nenhum risco de divergência CPU/GPU.

⇒ **o kernel deixa de ser o que destrava o nó e passa a ser o que o leva de `512` a milhões** — com
o `ns/célula` plano a dizer exactamente quanto ele compraria. *Uma optimização com o número ao lado
é uma decisão; sem ele era uma aposta.*