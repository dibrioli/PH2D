# 114 — CICLO 9 · RIG & CORPOS MOLES — «Coisas que se seguram»

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, nesta ordem, e **o tutorial É o
> smoke**. Este doc é o do ciclo: cada passo escreve a secção dele aqui.
>
> **Estado (2026-09-17):** passos **1** (grupo), **2** (auditoria), **5** (a medição, §7) e **6**
> (a cena `=120` + o tutorial, §10) FECHADOS. Do passo 3/4 fecharam as waves
> **W0 · W1′ · W2 · W4 · W4-bis**; ⛔ a **W1 foi REFUTADA por medição** (§3.1) e a wave da força de
> constraint **dissolveu** na coluna que já existia (§7-W2). ⏳ **Falta o passo 7 — o smoke do
> DONO**, que não é da linha: ela fecha, entrega o handoff e espera (§0.7).

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
| **W4** ✅ | A **razão nomeada** de cada corpo mole estar na CPU (§8) — FECHADA; o **preço** do recuo veio com a W5: `13,770 ms` a 3 600 agentes | Lei 1 do doc 103 §2 |
| **W4-bis** ✅ | O **tecto `MAX_SIDE`** do `motion.wave` — `60 → 512`, FECHADA em 2026-09-17 (§9) | ⛔ **Vem ANTES do kernel** (§8): não se decide uma placa para um campo que não pode crescer |
| **W5** ✅ | A **MEDIÇÃO** do grupo (passo 5) — FECHADA em 2026-09-17 (§7): 21 células em RELEASE com o `loadavg` impresso pela sonda | §0.0 |
| **W6** ✅ | O **TUTORIAL em PDF** (passo 6) + a cena `=120` — FECHADA em 2026-09-17 (§10) | O tutorial É o smoke |
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

## §7 — ✅ W5: A MEDIÇÃO do grupo (passo 5)

Corrida em **RELEASE**, `load 22,88` (`/proc/loadavg` impresso pela própria sonda, §0.0), pela ponte
do produto: `<nó> → motion.output` com a aresta de estado ATRASADA sobre si mesmo, mediana de **9**
quadros depois de **60** tiques de aquecimento. Um quadro de 60 fps tem **16,67 ms**.

⚠️⚠️ **A carga LÊ-SE NA TABELA e não se corrige:** cada número aqui é um **tecto**, nunca um valor
nominal — a máquina estava a `22,88` com outra linha a cozinhar. Isso é conclusivo **numa direcção
só**: o que já cabe no quadro AQUI cabe sempre; o que não cabe, não se pode absolver com esta
corrida. (O `motion.wave` a `512` lê `1,322` aqui e leu `1,001` a `load 16,17` na W4-bis — `+32 %`
de deriva de carga sobre o MESMO binário, que é a escala do ruído que esta coluna carrega.)

| nó | lado | linhas | quadro/ms | ns/linha | % de um quadro |
|---|---:|---:|---:|---:|---:|
| `motion.wave` | 16 | 256 | 0,001 | 5,6 | 0,01 % |
| `motion.wave` | 32 | 1 024 | 0,004 | 3,6 | 0,02 % |
| `motion.wave` | 60 | 3 600 | 0,010 | 2,9 | 0,06 % |
| `motion.wave` | 128 | 16 384 | 0,046 | 2,8 | 0,28 % |
| `motion.wave` | 256 | 65 536 | 0,207 | 3,2 | 1,24 % |
| **`motion.wave`** | **512** | **262 144** | **1,322** | **5,0** | **7,93 %** |
| `motion.soft_body` | 16 | 256 | 0,002 | 8,3 | 0,01 % |
| `motion.soft_body` | 32 | 1 024 | 0,006 | 6,1 | 0,04 % |
| `motion.soft_body` | 60 | 3 600 | 0,020 | 5,5 | 0,12 % |
| `motion.soft_body` | 128 | 16 384 | 0,094 | 5,7 | 0,56 % |
| `motion.soft_body` | 256 | 65 536 | 0,373 | 5,7 | 2,24 % |
| **`motion.soft_body`** | **512** | **262 144** | **2,259** | **8,6** | **13,55 %** |
| `motion.verlet_rope` | 16 | 256 | 0,070 | 272,7 | 0,42 % |
| `motion.verlet_rope` | 32 | 1 024 | 0,281 | 274,0 | 1,69 % |
| `motion.verlet_rope` | 60 | 3 600 | 0,992 | 275,5 | 5,95 % |
| `motion.boids` | 16 | 256 | 0,044 | 172,3 | 0,26 % |
| `motion.boids` | 32 | 1 024 | 1,099 | 1 073,4 | 6,59 % |
| **`motion.boids`** | **60** | **3 600** | **13,770** | **3 825,0** | **82,6 %** |

⚠️ **As duas escadas não são a mesma escada, e a coluna `linhas` é quem o diz:** um campo 2D cresce
com a ÁREA (`lado²` células) e uma corda é 1D — a sonda dá aos nós de CONTAGEM o **lado ao quadrado**
como contagem, justamente para as duas colunas serem comparáveis. ⛔ O `ns/linha` **só** se compara
entre linhas da mesma contagem.

---

### §7.1 — O que a tabela decide, uma leitura por nó

⭐⭐⭐ **(1) O IRMÃO MAIS CARO É QUEM TINHA O TECTO MAIS LARGO — e agora está medido na escada
inteira, não num ponto.** O `motion.soft_body` custa `1,71×` o `motion.wave` a `512²` (`2,259`
contra `1,322`) e `1,80×` a `256²`, e **já shipava com `MAX_SIDE = 512`** enquanto o mais barato
estava preso em `60`. *A W4-bis não escolheu um número: ela copiou o do irmão, e esta tabela mostra
que o irmão o suporta com folga maior do que o novo dono precisa.*

⭐⭐ **(2) O `ns/linha` do campo tem VALE, e o vale é a memória.** `5,6 → 3,6 → 2,9 → 2,8` e depois
`3,2 → 5,0`: o custo por célula **desce** enquanto a grelha cabe em cache (o estêncil de 5 pontos lê
os quatro vizinhos) e **volta a subir** quando ela deixa de caber — a `512²` são `262 144` células ×
(estado + saída), e o passeio deixa de ser servido pelo L2. ⇒ *o recurso do tecto do campo é a
LARGURA DE BANDA de memória, e não a aritmética* — o que torna `512` um sítio honesto para parar
**hoje**, e o kernel de GPU a resposta certa para o degrau seguinte (lá a banda é outra ordem de
grandeza).

⭐⭐⭐ **(3) A CORDA É PLANA, e é isso que fecha a decisão de a deixar em Gauss-Seidel.** `272,7 ·
274,0 · 275,5` ns/ponto sobre uma faixa de **14×** na contagem — variação de `1,0 %`. ⇒ o custo é
**estritamente linear** nos pontos, logo uma corda do tamanho que alguém de facto faz (100–200
pontos) custa **`0,027`–`0,055 ms`**, `0,2 %`–`0,3 %` de um quadro. *Paralelizar a relaxação
compraria um número que já é ruído e pagaria com a CONVERGÊNCIA, que é o resultado.* ⛔ O gatilho de
reabertura fica nomeado e é o mesmo: cordas de dezenas de milhares de pontos — e a tabela diz quando
isso morde (`3 600` pontos já são `5,95 %` de um quadro).

⛔⛔⛔ **(4) O PREÇO DO RECUO ESTÁ MEDIDO, e é o número mais violento desta tabela: `13,770 ms`.**
O `motion.boids` é o **único** dos dez nós do grupo que chega ao dispositivo (§2), e este arnês coze
com o `Cook` da **CPU** — logo o que esta linha mede é a **rota lenta de quem tem placa**, que é
exactamente o que a lei 1 do doc 103 §2 manda medir. A `3 600` agentes ele come **82,6 % de um
quadro** sozinho, e a escada diz porquê:

| de → para | vezes mais agentes | vezes mais caro | expoente |
|---|---:|---:|---:|
| 256 → 1 024 | 4,00× | 25,0× | **2,32** |
| 1 024 → 3 600 | 3,52× | 12,5× | **2,01** |

⇒ **`O(N²)` confirmado pela porta do produto** (cada agente consulta todos os outros), com o
expoente a assentar em `2,0` assim que a contagem sai do regime em que o custo fixo ainda pesa.
⭐ *É por isso que a escada dos nós de contagem pára em `60` e não é preguiça: `512² = 262 144`
agentes na rota da CPU custariam `~5,3 × 10⁶ ms` pela mesma lei — a corrida penduraria, e uma escada
que ninguém sobe mede um programa que ninguém corre.*

⚠️ **E isto é a MELHOR notícia da tabela, não a pior:** o número grande é o preço de **não** estar na
placa, e ele mede-se **num nó que tem para onde ir**. Os outros nove não têm — é isso que a §8 nomeia
um a um.

---

### §7.2 — O que a tabela NÃO diz

⛔ **Ela não mede a cadeia.** Cada linha é **um** nó entre a fonte e o `motion.output`; uma cena real
empilha `field.*`, `motion.scale` e o resto por cima, e o custo do grupo numa cena cheia é outra
medição (a do doc 98, que já existe e mede o MÓDULO).

⛔ **Ela não mede o dispositivo.** A coluna `quadro/ms` é sempre CPU, **incluindo** a do
`motion.boids` — *um controlo que não percorre o mesmo caminho não controla nada*, e a sonda diz
isso de si mesma no doc-comment da tabela `MOLES`.

⚠️ **E o `ns/linha` da corda e do bando não se compara com o do campo:** a contagem de um nó 1D é o
lado ao quadrado **por construção da escada**, para as duas caberem no mesmo eixo. Um ponto de corda
custa `~273 ns` e uma célula de campo `~3 ns` porque são trabalhos diferentes (a corda faz
`solver_substeps` passagens de relaxação sobre as restrições; o campo faz **um** estêncil).


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
---

## §10 — ✅ W6: a CENA `=120` e o TUTORIAL (passo 6)

O entregável do ciclo, e **o tutorial É o smoke** (doc 103 §1). Três peças, nesta ordem: a cena, as
figuras que saem dela, e o PDF que as mostra.

### §10.1 — A cena, e por que ela é esta

`PH2D_GPU_COOK_DEMO=120` — seis panos em três fileiras, cada fileira um par *«o base | o base mais
uma coisa»*:

| fileira | a pergunta | esquerda | direita |
|---|---|---|---|
| **cima** | e se a coisa **se segurar sozinha**? | `Verlet Rope` | `Wave` |
| **meio** | e se **eu** quiser segurá-la? | `FK` (eu digo o ângulo) | `IK 2-Bone` (eu digo a mão) |
| **baixo** | quanto é que **cada osso** segura? | a pele com todos iguais | a pele com **quinhão** |

⭐ **A partição em duas categorias É o grupo** (§1) e não arrumação: a fileira de cima são nós
**Source**, as outras duas são **Transform**. É essa a premissa do tutorial.

⭐⭐ **Cada wave deste ciclo tem um passo que o dono EXECUTA**, e é isso que a torna entregue:

| wave | o passo | a alavanca |
|---|---|---|
| **W4-bis** (§9) | escrever `512`/`512`/`0,004` no campo | `Rows` · `Cols` · `Spacing` |
| **W2** (§7-W2) | apertar o raio do cartão `Strength` | `Radius`, `20 → 1` |
| **W1′** (§6) | mexer no fim da banda do quinhão | `End` do `Range: que ossos puxam` |

⛔ **QUATRO dos dez nós não estão na cena, com motivo:** `motion.soft_body` e `motion.boids`
produzem como a corda e o campo e já têm cena própria; `rig.fabrik` e `rig.rubber_hose` são **a
mesma lei do `ik_2bone`** com outra contagem de juntas — pô-los lado a lado ensinaria *«há três
nomes»*, que é o oposto de *«a mão vai ao alvo»*. ⭐ E a força dos três é a **mesma** coluna, logo o
passo da W2 vale para todos.

### §10.2 — ⛔⛔⛔ A IMAGEM REFUTOU A CENA DUAS VEZES, com a suíte VERDE

*Nenhum gate desta linha olha para uma figura*, e as duas vezes o defeito só apareceu ao render os
SVG e olhar.

**(1) A pele lia-se como RUÍDO.** Ela era uma grelha `5 × 5` de vão `0,4` sobre uma corrente de
`1,8` de comprimento ⇒ **mais larga do que a corrente é comprida**, com metade das peças longe de
qualquer osso e cada uma a seguir o osso mais próximo por si. ⇒ a pele passa a ser uma **MANGA**
(`3 × 7`, `0,44 × 1,68`) que embrulha a corrente, e a figura desenha a **malha** (fio à direita, fio
abaixo) em vez de pontos soltos. ⚠️ *A ligação não é decoração: é a informação que a nuvem de pontos
perdeu* — numa corrente ela é o `parent`, numa pele é a vizinhança da grelha.

**(2) E depois disso os dois panos de baixo ainda eram INDISTINGUÍVEIS.** O envelope era uma
**rampa** (`0` na raiz, `1` na ponta) — e a ponta é justamente onde a corrente mais se dobra, logo o
quinhão pequeno caía sobre a parte da pele **que já não se mexia**. Medido nas próprias figuras:

| envelope | desvio máximo entre os dois panos | em lados de peça |
|---|---:|---:|
| rampa (`value.instance_field`) | `0,035` de mundo | **`0,39`** |
| **banda** (`field.index_range`) | `0,63` de mundo | **`6,97`** |

⇒ o envelope passa a ser uma **banda** com `soft = 0`: as duas últimas juntas ficam com quinhão
**zero** e a metade de cima da manga fica direita, *como uma manga larga que não acompanha o
cotovelo*. **`18×`** mais contraste.

⭐⭐ **E a cura trouxe uma lição melhor que a que substituiu:** a caneta do envelope passou a ser o
**mesmo trio** do cartão `Strength` do pano do IK — um **campo** decide o *quanto*, o
`value.attribute` lê esse número da coluna `falloff`, e o `motion.drive(Custom…)` escreve-o na
coluna que se quiser. *Dois panos, uma lição*, e é exactamente a composição por que a W1 foi
refutada (§3.1).

⛔⛔ **E os gates da pele eram CÚMPLICES.** Eles pediam `d > 1e-3` — *«os dois panos diferem»* — e a
versão invisível passava-os com folga. ⚠️ *Uma régua que só vê o SINAL não vê a MAGNITUDE*, a mesma
família do `edge_max` cego ao quad fino. ⇒ barra **`VISIVEL = 0,3`** de mundo (quase três peças da
pele), com a medição e a folga de `2×` escritas no doc-comment: o que ela proíbe não é o defeito de
hoje, é a **regressão ao invisível**.

### §10.3 — ⛔⛔ E o GATE apanhou um passo IMPOSSÍVEL num PDF já impresso

O passo 3 do tutorial mandava mudar a linha **`Height Channel`** do cartão `Wave`, e o cartão
mostra **`Height Drives`** — o param chama-se `height_channel` e o **rótulo que o artista lê é
outro**. ⚠️ *Um passo que manda clicar numa linha AFIRMA que ela está no cartão*, e o modo de falha
é o pior de todos: o dono procura, não encontra, e conclui que o programa está partido. Curado nos
**três** sítios (o HTML, o anúncio do terminal e o gate).

⚠️ E a cena ganhou um rótulo por causa disto: ela tem **quatro** `motion.drive`, e o passo dizia
*«o cartão Drive»*. *Um passo que diz «o Drive» num grafo com quatro é um passo que o dono não
consegue executar.*

### §10.4 — O que fica GATEADO

| gate | o que ele afirma |
|---|---|
| `a_cena_monta_seis_panos_e_nenhum_vem_vazio` | os seis panos, com piso de população |
| `a_corda_balanca_e_o_campo_ondula` | a fileira de cima mexe-se — pela **assinatura** (posição **e** tamanho) |
| `a_cinematica_directa_e_a_inversa_dao_panos_diferentes` | o par do meio não é o mesmo nó duas vezes |
| `a_mao_segue_o_alvo_e_o_pano_do_fk_nao_se_mexe` | a mão segue — com o FK `Pure` como CONTROLO |
| `apertar_a_forca_da_restricao_muda_a_pose_do_ik` | a W2, pelo BARRO e não pelo param |
| `o_quinhao_por_osso_muda_a_pele` · `o_quinhao_a_zero_devolve_a_pele_ao_repouso` | a W1′, com a barra do que se **vê** |
| `o_passo_do_tecto_entrega_meio_milhao_de_celulas` | a W4-bis, pela porta da cena |
| `every_row_the_rig_tutorial_names_is_on_the_card` | cada linha que o PDF nomeia está no cartão |
| `every_figure_the_rig_tutorial_shows_exists` | as seis figuras existem, contadas |
| `the_rig_tutorial_opens_the_scene_this_cycle_built` | ele abre a `=120` (metade em COMPILAÇÃO) |
| `the_cost_table_the_rig_tutorial_prints_is_the_one_that_was_measured` | os números da §7 do PDF são os da §7 deste doc |

⚠️ **O último NÃO re-mede:** medir num gate fá-lo-ia membro da família de flakes de carga
(`CLAUDE.md` §5.0), e o que se defende ali é a **honestidade do texto**, não o relógio. *Uma tabela
de custo num PDF é a afirmação mais fácil de deixar apodrecer do repo — ela não compila, não corre
e ninguém a relê.*

⚠️ **E um gate do ciclo 8 teve a premissa MORTA**, que é o gate a funcionar: ele afirmava
`!is_cycle_scene("120")` — *«a tabela não pode responder por uma cena que não existe»* — e este
ciclo construiu-a. ⛔ O defeito não era o número, era a **forma**: um gate escrito sobre o SUCESSOR
de hoje reprova no dia em que alguém escrever o ciclo seguinte, **sobre produto correcto**. ⇒
reescrito DERIVADO do tecto (`MAX_DEMO_LEVEL + 1`), com a morte visível no diff.

---

## §11 — ⛔⛔⛔ A COLISÃO NO GRUPO: o report do dono, MEDIDO

> Report (2026-09-17): *«Verlet rope com Shape e Collision ON não reconhece colisões entre as
> próprias células. Talvez todos do grupo não aceitem colisão.»*

⭐ **Ele tem razão, e a segunda frase é a mais importante das duas.** A sonda é
[`motion_rig_colisao_probe.rs`](../../crates/ph2d-app-motion/src/motion_rig_colisao_probe.rs)
(`#[ignore]`, corre-se à mão).

### §11.1 — São TRÊS perguntas, e respondê-las juntas dá a resposta errada a uma

| # | a pergunta | a resposta |
|---|---|---|
| 1 | a corda colide **consigo mesma**? | **não**, e é estrutural |
| 2 | o `Collide` do `source.shape` **chega** a ela? | **não pode**, e é estrutural de outra maneira |
| 3 | a **composição** já exprime isto? | ⭐ **SIM, por inteiro** — e estava a um cartão de distância |

### §11.2 — (1) A corda não colide consigo mesma, e vê-se no solver

A relaxação do `motion.verlet_rope` tem **exactamente duas** restrições: `i↔i+1` (a distância de
repouso) e `i↔i+2` (a flexão, só com `bend > 0`). **Nenhuma `i↔j`** para pares afastados — uma volta
do laço atravessa a outra porque *nada no solver olha para esse par*.

Medido pela porta do produto (25 pontos, âncora chicoteada, 240 tiques, discos de raio `0,07` ⇒ a
barra é `0,140`):

| arranjo | menor vão entre não-vizinhos | pares sobrepostos |
|---|---:|---:|
| **a corda NUA** | **`0,0079`** | **13** |
| `+ motion.collide` depois | `0,1051` | 9 |
| `+ motion.collide` no laço de estado | `0,1063` | 7 |

⇒ `0,0079` sobre uma barra de `0,140` é **um ponto praticamente em cima do outro**.

⚠️ **O chicote é parte da régua, não cenografia:** uma corda pendurada em repouso é uma catenária e
**nunca se toca**. Medi-la em paz responderia *«não há sobreposição»* sobre um arranjo que não a
pode ter — *uma régua que não vê o fenómeno acontecer não prova que ele não aconteceu.*

### §11.3 — (2) O `Collide` da forma não tem por onde chegar

O `source.shape` **declara** o colisor em COLUNAS (`COLLIDER_COLUMN` · `COLLIDER_BOX_COLUMN` ·
`COLLIDER_OFFSET_COLUMN`), e o censo de quem as **lê** dá três crates, todas da família da
simulação: `ph2d-node-sim-collide` · `ph2d-node-sim-step` · `ph2d-contact`.

⛔ **E o `source.shape` é uma FONTE — `inputs: &[]`.** Ela não pode estar a jusante da corda; o
caminho por que um colisor declarado chega a um solver é `source.shape → sim.spawn(template) →
sim.zone/sim.step → sim.collide`, que é a pilha do **ciclo 5**.

As portas de entrada de cada nó, contadas do manifesto:

| nó | entradas | as portas |
|---|---:|---|
| `motion.verlet_rope` | 3 | `anchor_x` · `anchor_y` · `state` |
| `motion.wave` | 3 | `drive` · `state` · `inject` |
| `motion.soft_body` | 4 | `anchor_x` · `anchor_y` · `state` · **`shape`** |
| `motion.boids` | 4 | `target_x` · `target_y` · `state` · **`obstacle`** |
| `rig.skeleton` | 0 | — |
| `rig.skin_deformer` | 3 | `in` · `rest` · `posed` |
| `sim.collide` | 1 | `in` |

⚠️⚠️ **DUAS portas parecem ser o que não são, e é por isso que esta tabela está aqui:**

- O **`shape`** do `motion.soft_body` é a **forma de REPOUSO** (a wave da folha 03), não um colisor.
- O **`obstacle`** do `motion.boids` é uma nuvem de **pontos a evitar**, e o que ele aplica é uma
  **força de direcção** (`avoid_accel`, com `avoid_radius` e `lookahead`) — ⛔ *evitar não é
  colidir*: dois agentes continuam a poder ocupar o mesmo sítio, e nada ali lê as colunas do
  colisor.

⇒ **a segunda frase do report está CERTA:** nenhum dos dez lê o colisor que a forma declara.

### §11.4 — ⭐⭐⭐ (3) E a composição entrega-o POR INTEIRO, a um cartão de distância

O `motion.collide` é um separador de não-penetração a sério (PBD, Müller et al. 2007), é
`Effect::Pure` e aceita **qualquer** nuvem — inclusive a da corda. Pondo-o **dentro do laço de
estado** (`rope → collide → (atrasada) → rope.state`) e varrendo as iterações dele:

| iterações do `collide` | menor vão | pares sobrepostos | % da barra |
|---:|---:|---:|---:|
| 8 (o valor de fábrica) | `0,1063` | 7 | 76 % |
| 16 | `0,1284` | 3 | 92 % |
| **32** | `0,1383` | **0** | **99 %** |
| 64 | `0,1399` | 0 | 100 % |
| 128 | `0,1399` | 0 | 100 % |

⇒ **a `32` iterações a corda deixa de se atravessar, medido: ZERO pares sobrepostos.**

E o preço (RELEASE, mediana de 9 quadros em regime; um quadro tem `16,67 ms`):

| pontos da corda | a corda só | `+ collide` a 32 | % de um quadro |
|---:|---:|---:|---:|
| 25 | `0,006 ms` | `0,017 ms` | `0,1 %` |
| 50 | `0,013 ms` | `0,060 ms` | `0,4 %` |
| 100 | `0,027 ms` | `0,211 ms` | `1,3 %` |
| 200 | `0,053 ms` | `0,785 ms` | `4,7 %` |

⇒ uma corda do tamanho que alguém faz (100–200 pontos) com auto-colisão **completa** custa
`1,3 %`–`4,7 %` de um quadro. ⚠️ O `motion.collide` é `O(n²·iterações)` na CPU — a `200` pontos e
`32` iterações são `1,28 M` testes de par por quadro, e é daí que vem o `4,7 %`.

### §11.5 — ⛔ A minha régua acusou a própria CONVERGÊNCIA

A 1.ª redacção desta sonda contava um par como sobreposto com `d < 2·raio` **estrito** — e o
repouso de um separador de não-penetração é *os discos a TOCAR*, que em `f32` pousa em `0,1399`
sobre uma barra de `0,1400`. ⇒ ela imprimia **«7 pares sobrepostos»** ao lado de **«100 % da
barra»**: *duas colunas da mesma medição a contradizerem-se, e a leitura errada — «não funciona» —
é a que se acredita.*

⇒ `TOLERANCIA = 2 %`, e ela **não é um epsilon de vírgula flutuante**: é *«a penetração é
visível?»*. Sobre um disco de `0,07` são `0,0028` de mundo, contra os `0,066` que a corda nua
penetra.

### §11.6 — O que isto deixa em aberto, e de quem é

⭐ **A capacidade EXISTE; o que falta é ERGONOMIA**, e é uma decisão de produto:

1. **O `Collide` da forma lê-se como um controlo que não faz nada** quando o grafo é uma corda —
   ele está vivo (a família `sim.*` lê-o) e é **inalcançável a partir daqui**. ⚠️ *Um controlo morto
   e um que não se aplica a este caminho dão o MESMO report* (`CLAUDE.md` §5.0), e a diferença só
   se vê medindo.
2. **O `motion.collide` de fábrica vem a `8` iterações**, e este uso precisa de `32`. O default está
   certo para o uso dele (separar uma nuvem de clones); aqui ele é o número errado e nada o diz.
3. ⏳ **A auto-colisão NATIVA na corda** (uma restrição `i↔j` dentro do solver dela) não foi
   construída — e a medição diz que ela **não é necessária para a capacidade**, só para o conforto:
   a composição já entrega o resultado. *Construí-la sem essa medição teria sido reconstruir o que a
   composição exprime*, que é a §5.0 e a mesma lei por que a W1 deste ciclo caiu.

---

## §12 — ✅ O BOTÃO `Collide` DA FORMA PASSA A VALER FORA DA SIMULAÇÃO

> Ordem do dono (2026-09-17): *«O botão Collide da Shape deve funcionar para todo e qualquer
> duplicador.»*

### §12.1 — O censo DERIVADO: quem é duplicador, e o que ele faz ao colisor

⛔ **A população sai do REGISTO, nunca de uma lista escrita à mão** — *«todo e qualquer»* não se
adivinha. Sonda
[`motion_colisor_duplicador_probe.rs`](../../crates/ph2d-app-motion/src/motion_colisor_duplicador_probe.rs):
cada nó recebe `source.shape(Collide)` e conta-se quem devolve **mais peças do que recebeu**.

**14 fontes saltadas** (não têm porta de entrada — não são duplicadores) · **121 varridos** ·
**13 multiplicam**:

| o colisor declarado | quantos | quem |
|---|---:|---|
| ✅ **chega a TODAS as peças** | **5** | `motion.clone` · `motion.mirror` · `motion.kaleidoscope` · `fx.drop_shadow` · `fx.rgb_split` |
| — geram nuvem NOVA (nunca receberam a forma) | 8 | `motion.boids` · `motion.distribute_curve` · `motion.distribute_radial` · `motion.lattice` · `motion.soft_body` · `motion.verlet_rope` · `motion.voronoi` · `motion.wave` |
| ⛔ deitam fora um colisor que receberam | **0** | — |

⭐⭐⭐ **O achado: a declaração JÁ chegava intacta a todas as peças dos cinco duplicadores
verdadeiros. O que não existia era um LEITOR.** As três crates que consumiam aquelas colunas eram
todas da família `sim.*` ⇒ o botão só fazia alguma coisa dentro do laço da simulação — *um controlo
vivo e inalcançável a partir de todo o resto do catálogo*, que dá exactamente o mesmo report que um
controlo morto.

⚠️⚠️ **E o discriminador que separa as duas últimas linhas é o `geometry_id`.** Sem ele as duas
leem-se iguais numa tabela de *«o colisor não chegou»* e **as curas são opostas**: uma é um defeito
de passagem, a outra é a pergunta de produto *«uma peça que este nó INVENTA deve herdar o colisor da
forma que entrou?»*. ⛔ A minha primeira leitura acusou o `motion.scatter` de deitar o colisor fora,
e o discriminador **refutou-a**: ele é uma FONTE e nunca recebeu nada.

### §12.2 — A cura: o `motion.collide` honra o colisor DECLARADO

[`declarado.rs`](../../crates/ph2d-node-motion-collide/src/declarado.rs) — o nó passa a perguntar à
[`ph2d_contact`] (o motor de peça-contra-peça da casa: grelha espacial, caixas orientadas, rotação)
em vez de separar discos de raio uniforme. Medido pela rota do produto
(`source.shape(Collide) → motion.clone(distance 0) → motion.collide`):

| o botão | como as cinco cópias ficam |
|---|---|
| **desligado** | espalhadas na DIAGONAL — `(−0,64,−0,64) … (0,64,0,64)`. *Um disco não tem orientação.* |
| **ligado** | numa COLUNA, todas no mesmo `x` — `(0,00,−1,39) … (0,00,1,39)`. *Caixas arrumam-se como caixas.* |

**Três decisões, cada uma com o porquê no código:**

1. **A declaração GANHA do `Radius` do cartão** — *só quem desenha sabe o tamanho do que desenha*.
2. **Uma peça SEM declaração cai no `Radius`, como disco** — sem isto uma corrente MISTA deixaria
   metade das peças inertes e caladas, e um nó que separa umas e não outras é pior que um que não
   separa nenhuma.
3. ⛔⛔ **O `Strength` e o `falloff` entram por MISTURA no fim, e NUNCA nos pesos.** Multiplicar o
   `inv_mass` das duas peças de um par pelo `strength` fá-lo **CANCELAR** (`λ = pen/(k_a+k_b)`,
   `Δp = n·λ·w`) e o knob ficaria **inerte** — é a armadilha que o `push_apart` do mesmo ficheiro já
   documenta um nível acima, a morder outra vez noutra aritmética.

⭐ **Sem declaração a porta nem abre** (`colisores()` devolve `None`) ⇒ **todo o catálogo que já
existe sai AO BIT**. As 31 do `motion.collide` e a varredura impactada (**4 441 corridos, 4 441
verdes**) afirmam-no.

### §12.3 — ⛔⛔ A divergência CPU/GPU que isto teria aberto — e a cerca que já existia

O kernel de WGSL do `motion.collide` separa **discos de raio uniforme** e tem `applicable: None`.
Com a cura, o MESMO grafo daria **uma pilha de caixas na CPU e um borrão de discos na placa, sem
erro nenhum**.

⛔ **E a `applicable` não podia resolvê-lo:** ela recebe `fn(&dyn Fn(&str) -> f32) -> bool` — só os
**params** do nó —, e a declaração é uma propriedade da **CORRENTE** que chega.

⭐⭐ **A cerca já estava construída, e são DUAS, cobrindo as duas rotas:**

| a rota | quem a recusa |
|---|---|
| o `source.shape` declara pelo cartão | `graph_has_live_vector_source` (ADR-0154) |
| alguém escreve a coluna **pelo NOME** (`motion.drive(Custom…)`) | `graph_declares_collider` (doc 109 W2) |

⇒ um documento que carrega colisor declarado **já era planeado para a CPU**, e a divergência é
impossível por construção. ⚠️ *Isto foi MEDIDO e não lido:* o gate
`a_cadeia_que_declara_colisor_pela_forma_e_recusada_do_dispositivo` corre a cadeia inteira, **com o
CONTROLO** (uma grelha sem forma, que não pode ser recusada — senão a cerca seria incondicional e o
gate passaria por ela).

### §12.4 — Três vezes o ARNÊS se leu como um defeito de produto

⚠️⚠️ **As três foram apanhadas pelo CONTROLO, nunca pela leitura do código** — e é o registo mais
útil desta wave:

1. **`source.shape` coze ZERO peças sem as membranas** — ele lê um external que a shell publica.
2. **E as membranas vivem no cozedor do PUMP**: um `Cook::new()` nasce sem nenhuma, e o nó devolve
   *stream vazio, sem erro* (está escrito no `eval` dele). A sonda lia `0 peças, 0 colunas`, que é
   exactamente *«o duplicador deitou o colisor fora»*.
3. **Ligar um fio a uma porta que NÃO EXISTE não falha** — o grafo aceita a aresta, o cozedor
   ignora-a, e o nó emite a nuvem dele. A 1.ª redacção do censo contava assim **14 fontes** como
   duplicadores.

⭐ E uma quarta, no gate: **um `set_param` com um nome que o nó não tem também não falha** — ele
fica guardado e ninguém o lê. O `step_x`/`step_y` que escrevi no `motion.clone` (cujo param é
`distance`) deixou o CONTROLO com o arranjo de fábrica, e o gate teria medido outra coisa.

### §12.5 — O que fica ABERTO, e é decisão do dono

⏳ **O botão ainda precisa de um cartão `Collide` na cadeia.** Ele agora decide **por que colisor**
as peças se arrumam — em todo duplicador que carregue a forma —, e não **se** elas colidem. Fazer o
botão sozinho separar exigiria um passe implícito sem cartão: um solver escondido, com custo
escondido e sem controlo de iterações. ⇒ **medido e não construído**, à espera do veredito.

⏳ **E os 8 nós que geram nuvem NOVA** (a corda, o campo, o corpo mole, o bando, as distribuições)
não recebem a forma de ninguém: as peças deles não são cópias de nada. Dar-lhes o colisor da forma
que entra é uma pergunta de produto, não um defeito — e para a corda a §11 já mostra que o caminho
que existe hoje entrega auto-colisão completa.

---

## §13 — ✅ O PAINEL LATERAL DE PARAMS SAIU DO APP (pedido da `line/UIUX`)

> **Ordem do dono (2026-09-17), depois de eu lhe devolver a pergunta com as três leituras
> possíveis do pedido:** *«Só o painel de parâmetros do Motion»*. A coluna da direita e o
> Inspector **ficam**; o que sai é a crate `ph2d-panel-motion-params`.

O pedido vive em [`104_pedido_retirar_o_painel_lateral.md`](104_pedido_retirar_o_painel_lateral.md)
e a §3 dele prescreve cinco passos. Os cinco estão feitos. O que este §13 regista é o que a
execução mediu **além** deles — que foi mais do que o pedido supunha, em duas direcções opostas.

### §13.1 — O tamanho REAL, contra o que o pedido estimava

| | O pedido dizia | Medido |
|---|---|---|
| mover para fora da crate | «a caixa de correio» (~85 linhas) | **592** linhas em 3 ficheiros — o `ParamRow` inteiro (522) mais a caixa (70) |
| gates de outras crates a partir | «0» | **4** censos textuais (o de rótulos que embrulham, o de hashes nomeados, o registo tipado, o catálogo da paleta) |
| apagar | 7 473 linhas | **7 551** linhas em 35 ficheiros, com **66** testes |

⭐⭐ **A correcção que mudou a wave é a primeira linha:** o que o painel guardava não era só um
canal — era o **vocabulário de rows** de que o CARTÃO se serve. *Ele PINTAVA o `ParamRow`; não o
definia.* Foi por isso que a mudança de casa foi limpa (zero imports no ficheiro movido) e é por
isso que apagar a crate sem o mover teria parado o cartão inteiro, que é exactamente o que o
`CLAUDE.md` §5 avisava por escrito desde 2026-09-07.

### §13.2 — ⛔⛔⛔ O achado que vale mais que a remoção: **TRÊS TECTOS MEDIDOS FICARAM ÓRFÃOS**

O painel não levou só a pintura. Ele levou o **consumidor** de números que gates verdes continuavam
a afirmar:

| tecto | quem o fazia valer | leitores de produto hoje |
|---|---|---|
| `MAX_PARAM_ROWS = 16` | o `.take()` do `paint_rows` **do painel** | **0** |
| `MAX_ENUM_OPTIONS = 48` | o `.min()` do `rows_paint_kinds` **do painel** | **0** |
| `INSPECTOR_MAX_H` (foundational) | o painel media contra ele quantas linhas cabiam | **0** |

⚠️⚠️ **E o cartão NÃO herda a pergunta** — isto foi medido antes de cortar, não presumido: ele não
desenha uma fileira de slots com `.take()`; ele pinta as rows que o snapshot traz, e um selector
dele **cicla** (`ClickDoes::Cycle(labels.len())`) em vez de expor uma opção por botão. *Não há onde
truncar, logo não há param nem opção que caia em silêncio.*

⇒ **Um tecto cujo consumidor saiu não protege nada: ele fica a ser um número que gates verdes
continuam a afirmar sobre o vazio.** É a forma mais cara de cobertura falsa, porque ela cresce —
cada wave nova soma gates àquele número. Os dois primeiros saíram com os gates que os mediam; o
terceiro é foundational e fica **NOMEADO com o mecanismo** no `layout.rs`, porque apagá-lo é
decisão de quem possui aquela fundação.

### §13.3 — ⛔⛔ E o mesmo aconteceu com DOZE IDS, mas eu só o vi à terceira medição

O ficheiro `motion_param_row_ids.rs` (397 linhas) tinha doze ids e os dois tectos. A primeira
leitura resgatou **três** — as amostras de cor — apoiada numa frase do doc da
[`ph2d_param_editors::EditorKey`]: *«as AMOSTRAS de cor têm prefixo próprio, e não é arrumação: a
shell também as deriva»*.

⛔⛔⛔ **A frase era verdadeira quando foi escrita e falsa no dia em que a li.** Ela descrevia os
dois hospedeiros de então — a row do painel e o cartão. Medido nos SÍTIOS DE CHAMADA, o cartão usa
`motion-card/swatch/{nó}/{canal}` e `card/editor_swatch/{nó}/{param}`: prefixos **com o nó lá
dentro**, que é a razão de ele nem precisar de o nó estar seleccionado. *Nenhum `motion_param/*`
atravessa fronteira nenhuma.*

⇒ o censo por porta deu `produto = 0` para os **doze**, e o ficheiro foi apagado inteiro.

⚠️ **A lei que isto paga, e que esta casa já tem escrita:** *uma ausência — ou uma presença —
afirmada pela PROSA é um palpite com cara de medição.* Eu li o doc-comment de uma chave em vez de
`git grep` nos chamadores dela, e construí uma justificação inteira, com gate novo, sobre o palpite.
O gate teria ficado verde para sempre a medir três ids que ninguém regista.

### §13.4 — O que a remoção de facto encostou, gate a gate

- **`motion_bridge_dock_height_tests.rs`** (555 linhas, 6 testes) — **apagado**: cada teste pintava
  o painel por `MockPanelHost::with_panel::<MotionParamsPanel>` e media rects. *Um teste que se
  apaga com o sujeito é o caso certo.*
- **`motion_bridge_enumcap_tests.rs`** (137 linhas, 3 testes) — **apagado** com o tecto de opções.
- **`motion_bridge_rowcap_tests.rs`** — cortado a meio: saem os dois gates do tecto e a sonda;
  **ficam** a ordenação por secções e a integridade da tabela de grupos, que são leis do
  **snapshot** e nunca foram sobre quem pinta.
- **`the_census_measures_the_fattest_panel_a_gesture_can_reach`** — perde a metade que comparava
  contra o censo do tecto; **fica** a que mede que um gesto revela linhas que o estado de fábrica
  não tem.
- **`the_channel_picker_fits_the_panels_ceiling`** → renomeado
  **`the_channel_picker_offers_the_weight_the_fields_write`**. ⚠️ *Um gate cujo nome promete uma
  propriedade que já não existe é pior que a ausência dele.*
- **`with_the_side_panel_out_the_card_still_writes`** — **fica inteiro**, e é o gate mais
  load-bearing da wave: ele é quem prova que truncar o `publish` não parou o cartão. A metade
  `current_params().is_none()` deixou de ser uma asserção e passou a ser o **compilador** — *uma
  asserção que se torna trivialmente verdadeira lê-se como cobertura e não é nenhuma*.
- **`two_cards_of_the_same_type_never_ask_for_the_same_colour_picker`** — a metade *«mesmo com A
  seleccionado»* virou **estrutural**: o `picker_target_of` deixou de receber o nó seleccionado.

### §13.5 — A ponte de cor perdeu METADE, e o código já a tinha separada

O `motion_bridge_color.rs` tinha, escrito nos próprios comentários, **dois ramos rotulados**: `(a) a
row do painel` e `(b) uma amostra de cartão`. Saiu o (a), e com ele `seed_color_swatches`,
`gradient_picker_stop`, `palette_picker_index`, `gradient_params`, `palette_params` e quatro
argumentos de duas portas (`sel`, `groups`, `grad_params`, `pal_params`).

⭐ **O cartão semeia as amostras dele sozinho** (`register_card_swatches` regista no índice de hits
**e** escreve a cor, por cartão, a cada pintura) — medido antes de cortar a semeadura do painel.

### §13.6 — Os quatro vermelhos que só a varredura IMPACTADA viu

Nenhum deles vive numa crate que a wave editou, e os quatro são **censos de obsolescência a
funcionar**:

| gate | era | é |
|---|---|---|
| `…never_gives_a_row_label_a_wrap_budget` | `ELIDED_TODAY = 30` | **16** (o painel levava 14 rótulos que cortam) |
| `every_non_literal_hash_is_named` | uma entrada nomeando `motion_param_row_ids.rs :: fnv_id` | apagada |
| `build_typed_registry_matches_enabled_features` | 28 painéis | **27** |
| `the_two_ugly_derived_titles_are_named_here` | *"Motion Params"* na paleta | fora |

⚠️ **O terceiro expôs um defeito anterior:** o contador do painel de params era um `{ n += 1; }`
**nu**, sem `cfg` — ele tinha perdido a feature dele numa wave anterior e ninguém reparou. *Um
censo derivado que não consegue explicar uma das suas próprias linhas já não é derivado.*

### §13.7 — E TRÊS vermelhos de clippy severo que eram MEUS, deste ciclo

O `-D warnings` só corre no portão de fecho, e apanhou três coisas que os `cargo check` desta linha
nunca podiam ver:

1. `#[expect(clippy::cast_sign_loss)]` **incumprido** no `ph2d-node-motion-wave` (W4-bis) — a lente
   prova o literal não-negativo e a isenção nunca dispara. *Uma isenção que não é usada lê-se como
   uma cerca a proteger alguma coisa*; virou `usize::try_from`.
2. `items after a test module` em **três** `pose.rs` das crates de rig (W2) — o módulo de teste
   vivia a meio, com `const` e funções por baixo.
3. `empty line after doc comment` no ficheiro movido — um doc-comment que ficou **sem item por
   baixo** quando as declarações de módulo do hub saíram. *O clippy disse à letra o que a nota
   dizia: um hub que sobrevive ao seu edifício documenta o vazio.*

### §13.8 — A porta de escape foi APAGADA, e essa é a diferença entre 09-07 e hoje

Em 2026-09-07 o painel foi **desligado** e o `PH2D_MOTION_PANEL=1` trazia-o de volta: um
interruptor de bissecção honesto, porque a crate existia. Com ela apagada, ele passaria a pôr
**VISÍVEL** um painel que nenhum pintor conhece.

⇒ `painel_lateral()`, a env e a escrita `panel_visibility["motion_params"]` saíram juntas. *Uma
porta de escape que já não pode cumprir o que promete é pior que nenhuma*, porque quem a usar lê o
ecrã inalterado como prova de que o defeito não estava ali.

### §13.9 — Prova de fecho

- `cargo check --workspace --all-targets` — verde, **zero avisos**.
- `scripts/nextest-impacted.sh` — **15 030** testes, 15 030 verdes (a 1.ª corrida acusou os 4 da §13.6).
- `cargo clippy --workspace --all-targets -- -D warnings` — verde (a 1.ª corrida acusou os 3 da §13.7).
- `scripts/censos-da-arvore-combinada.sh` — **87/87**, com o controlo do filtro a `8 de 8`.
- Contadores partilhados: `PROJECT_SCHEMA`, os três registos de componente e os schemas de documento
  **intocados** — esta wave não escreve um único número que some entre linhas.
- Diff: **97 ficheiros, +508 −8 539**.

⏳ **ABERTO e NOMEADO** (nada disto é desta linha):
- o `INSPECTOR_MAX_H` do `ph2d-editor-core` e o `MOTION_PARAMS_SCROLLBAR_ID` ficam **órfãos com a
  nota ao lado** — o segundo permanece no livro-razão de propósito, porque *um id retirado ocupa uma
  linha e um id reusado ocupa duas barras ao mesmo tempo*;
- o ramo do `dispatch/scroll.rs` que despacha aquele id fica **inerte por construção** (ninguém o
  regista, logo nenhum `id ==` casa).

---

## §14 — ⏸️ ADIADO POR ORDEM DO DONO: *o `collide` sai do app e tudo passa pela FORMA*

> **Ordem do dono, 2026-09-17, logo a seguir à §12:** *«Prefiro retirar o nó collide do app todo
> e deixar tudo para Shape. Prefiro que toda visualização passe pelo Duplicator e que nós como
> Grid, rope, etc, não passem de posições do espaço, sem nenhuma capacidade de gerar pixels na
> tela. MAs vamos adiar isso.»*

⛔ **Ele adiou explicitamente, e isto fica aqui para não se perder.** Não é trabalho pendente
desta linha; é a direcção arquitectural que a §12 tornou visível e que o dono escolheu não pagar
agora.

### §14.1 — As três metades da ordem

1. **O nó `motion.collide` sai do catálogo.** A colisão passa a ser sempre o botão `Collide` da
   forma — que é a decisão de 2026-09-10 (*«vou preferir colocar na shape»*) levada até ao fim.
   A §12 deixou o substrato: o leitor do colisor declarado já existe e já honra a declaração em
   todos os duplicadores.
2. **Toda visualização passa pelo Duplicador.** Hoje vários nós desenham por si.
3. **`Grid`, `rope` e os irmãos passam a ser POSIÇÕES e mais nada** — *«sem nenhuma capacidade de
   gerar pixels na tela»*.

### §14.2 — O que a §11 e a §12 já mediram, e que quem pegar nisto herda

- A composição `rope → motion.collide → rope.state` entrega **auto-colisão completa** a 32
  iterações, por `1,3 %`–`4,7 %` de um quadro (§11). Retirar o nó **tem de** manter essa
  capacidade por outro caminho, ou o grupo perde-a.
- Os **8 nós que geram nuvem NOVA** (corda, campo, corpo mole, bando, distribuições) não recebem
  forma de ninguém: as peças deles não são cópias de nada. *A ordem 3 é exactamente a pergunta de
  quem lhes dá o colisor* — e a §12 já a nomeia como pergunta de produto, não defeito.
- O `sim.collide` (peça contra MUNDO) é outro nó e outra pergunta; a ordem fala do irmão.

⚠️ **Nada disto está construído.** Quem abrir esta wave começa por medir o que a composição já
exprime (§5.0), como a §12 fez — foi isso que mostrou que faltava um LEITOR e não um motor.
