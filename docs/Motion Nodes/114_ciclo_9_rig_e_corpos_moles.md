# 114 — CICLO 9 · RIG & CORPOS MOLES — «Coisas que se seguram»

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/114_ciclo_9_rig_e_corpos_moles.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| W1: escritor genérico de coluna (folha 16 §SUPERAR 6) | REFUTADA em 2026-09-17: ele já existe (a «caneta»); medir antes de construir poupou a wave inteira | vivo §4/§5 e [arquivo](../archive/motion-nodes-2026-10-12/114_ciclo_9_rig_e_corpos_moles.md) §3.1 |
| Kernel de GPU do `motion.wave` antes do tecto | A pergunta está mal posta enquanto o tecto era `60`: o `MAX_SIDE` sobe `60 → 512` primeiro | vivo §8 / [arquivo](../archive/motion-nodes-2026-10-12/114_ciclo_9_rig_e_corpos_moles.md) §9 |
| `motion.verlet_rope` fora de Gauss-Seidel | Fica em GS: a relaxação escreve `pos[i]`/`pos[i+1]` no sítio e a restrição seguinte lê o que a anterior escreveu | [arquivo](../archive/motion-nodes-2026-10-12/114_ciclo_9_rig_e_corpos_moles.md) §9 Decisão 2 / vivo §8 |
| «Os corpos moles ficam na CPU por guardarem estado» | Falsa: o canal `StateSelect` existe e o `sim.zone` usa-o (10 de 12 nós de simulação medidos no dispositivo); a razão de cada um está nomeada | vivo §8 |
| Cena `=120` aceite com a suíte verde | A imagem refutou a cena duas vezes; os gates da pele eram cúmplices (`d > 1e-3`) | vivo §10.2 |
| Passo impossível num PDF já impresso | O gate apanhou-o depois de impresso | vivo §10.3 |
| Sonda de sobreposição com `d < 2·raio` estrito | Em repouso os discos TOCAM-SE (`0,1399` contra `0,1400` em `f32`): «7 pares sobrepostos» ao lado de «100 % da barra» — duas colunas a contradizerem-se | vivo §11.5 |
| Divergência CPU/GPU do `Collide` pela forma | A cerca recusa a cadeia que declara colisor pela forma; com controlo (grelha sem forma) | vivo §12.3 |
| Medir a metade que AGE atrás do `rig.skeleton` | Não pode ser medida atrás dele, que é a cadeia que o artista de facto usa | [arquivo](../archive/motion-nodes-2026-10-12/114_ciclo_9_rig_e_corpos_moles.md) §2.1 |

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, nesta ordem, e **o tutorial É o
> smoke**. Este doc é o do ciclo: cada passo escreve a secção dele aqui.
>
> **Estado (2026-09-20): ✅ O CICLO FECHOU — os SETE passos.** O dono correu a cena `=120` com o
> tutorial 09 e aprovou (*«smoke OK»*); o passo 7 e os **oito** pedidos que ele devolveu pelo
> caminho — sete reports e uma ordem de varredura — estão na **§15**. Do passo 3/4 fecharam as waves **W0 · W1′ · W2 · W4 · W4-bis**;
> ⛔ a **W1 foi REFUTADA por medição** (§3.1) e a wave da força de constraint **dissolveu** na
> coluna que já existia (§7-W2). ⇒ o ciclo aberto passa a ser o **10**
> ([doc 116](116_ciclo_10_o_carimbo_no_dispositivo.md)).
>
> ⭐ **E a §16 fechou o 1.º item da §15.2 por ordem do dono (*«DEVE SIM»*):** uma peça de rig
> **veste o osso** — o `size` de cada elemento sai do `len` dele, e o que o artista escreve na
> forma passa a ser uma RAZÃO.

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

## §10 — ✅ W6: a CENA `=120` e o TUTORIAL (passo 6)

O entregável do ciclo, e **o tutorial É o smoke** (doc 103 §1). Três peças, nesta ordem: a cena, as
figuras que saem dela, e o PDF que as mostra.

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

## §14 — ▶️ REABERTO no mesmo dia: *o `collide` sai do app e tudo passa pela FORMA*

> ⚠️⚠️ **Esta secção nasceu «ADIADO» e a palavra durou horas.** O dono reabriu a ordem com uma
> metade NOVA — *«…e pelos outros objetos (como vector, Sprite, Flip) que serão criados com seus
> próprios colliders (sem usar o grafo)»* — e decidiu a pergunta que faltava: **o app separa
> sozinho**. O plano, com as quatro medições que o desenham, é o
> [**doc 115**](115_o_colisor_sai_do_grafo.md). *O que fica escrito aqui é o estado em que a ordem
> foi adiada, e serve de contraste: um «adiado» não é um estado estável.*

> **Ordem do dono, 2026-09-17, logo a seguir à §12:** *«Prefiro retirar o nó collide do app todo
> e deixar tudo para Shape. Prefiro que toda visualização passe pelo Duplicator e que nós como
> Grid, rope, etc, não passem de posições do espaço, sem nenhuma capacidade de gerar pixels na
> tela. MAs vamos adiar isso.»*

⛔ **Ele adiou explicitamente, e isto fica aqui para não se perder.** Não é trabalho pendente
desta linha; é a direcção arquitectural que a §12 tornou visível e que o dono escolheu não pagar
agora.

## §15 — ✅ W7: O SMOKE DO DONO (passo 7), e os OITO pedidos que ele devolveu

> **Ordem do dono, 2026-09-20:** *«smoke OK. Volte ao plano de implementação»* ⇒ **o ciclo 9
> FECHA**, com os sete passos, e o ciclo aberto passa a ser o **10**
> ([doc 103 §5](103_dinamica_dos_ciclos.md) · [doc 116](116_ciclo_10_o_carimbo_no_dispositivo.md)).

O passo 7 não foi uma corrida: foram **oito** pedidos — sete reports e uma ordem de varredura —, e
o que esta secção guarda não é a lista (ela está nos commits, com o mecanismo). É a **leitura de
conjunto**, porque ela é sobre o instrumento e não sobre o rig.

| # | o que ele escreveu | o que era | commit |
|---|---|---|---|
| 1–4 | *«a linha do Shape cruza os nós»* · *«nós estão se interpenetrando»* · *«se há 2 níveis, os níveis se alinham no centro»* · *«a arrumação deve considerar o tamanho vertical do nó»* | **uma** ausência com quatro caras: a lei em camadas espaçava por **duas constantes** e ordenava sem olhar a PORTA em que cada fio aterra | `8df5cc76d` |
| 5 | *«neste caso o preview deveria ser colocado para cima»* | o retrato saía sempre para baixo, e a recusa escrita no `extensao_de` temia um preço que **não se paga** (reserva-se UM lado) | `ecf3e917a` |
| 6 | *«a rot não acontece e o centro da rot não é a base do segmento»* | o `rig.bones::derive_frame` devolvia **RADIANOS** numa casa cujo `rot` é em **GRAUS** | `404c29575` |
| 7 | *«veja se erro similar acontece em outros locais do módulo»* (ordem) | a unidade está limpa nos treze sítios; o que a varredura achou foi **outra família com o mesmo sintoma** — o comprimento DESENHADO de uma peça é um número da FORMA e o verdadeiro é a coluna `len`, e nada os liga | `5d4fbb90a` |
| 8 | *«pisca do lado direito quando escondemos o inspector e aumentamos muito a área do grafo de nós»* | a régua da ocupação das colunas fechava um **ciclo de dois quadros** consigo mesma | `abdef008c` |

### §15.1 — ⛔⛔⛔ A leitura de conjunto: **nenhum dos oito foi apanhado por um gate, e cinco não podiam ser**

Os catorze gates da cena `=120` estavam verdes em todos os oito. Não por serem fracos — por
medirem **outra grandeza**:

- **três são de DISPOSIÇÃO** (1–5): eles medem `P`, `rot` e `size`, e a arrumação não muda nenhum
  dos três. *Um gate que lê o que uma cadeia PRODUZ é cego a onde ela é DESENHADA.*
- **o 6 é de UNIDADE** e o **7 é de ESCALA**: os dois vivem dentro de números que os gates lêem e
  aprovam — `rot` estava certo como número e errado como ângulo, `size` estava certo como forma e
  errado como comprimento. *Uma régua sem unidade aprova o dobro e a metade.*
- **o 8 é um CICLO ENTRE QUADROS**, e este é o mais duro: os gates que existiam alimentam a porta
  com rects escolhidos à mão e medem **um** quadro. *Uma régua de um quadro só não pode ver um
  ciclo de dois* ([memória](../../project-memory/feedback_a_ruler_that_reads_what_the_layout_produced_oscillates.md)).

⭐⭐ **E o que os achou aos oito foi a MESMA coisa: o dono a correr o tutorial do princípio ao
fim.** É a lei do doc 103 §1 — *o tutorial É o smoke* — a pagar-se: cinco destes oito só existem
porque um passo do PDF manda fazer uma coisa que nenhum teste faz (arrastar uma divisória, esconder
um painel, arrumar um grafo, ler uma corda de perto).

### §15.2 — ⏳ O que fica ABERTO do ciclo 9

Nada disto bloqueia o ciclo 10; está aqui para não se perder.

- ✅ **A peça VESTIR o osso — FECHADO, por ordem do dono (2026-09-20: *«DEVE SIM»*).** A §16
  tem o mecanismo, a tabela do antes/depois e o preço aceite.
- ⏳ **O `Shape: Rope Segment` não tem um único consumidor** (§10 e o commit `71b14b423`): ele é uma
  forma ORIENTADA e a `motion.verlet_rope` não publica `rot`. Quem o quiser precisa de um nó que
  escreva o ângulo da direcção **ao vizinho seguinte** — o `rig.bones` exige `parent`/`len`/`rot` e
  o `motion.look_at` aponta a um ALVO.
- ⏳ **A auto-colisão NATIVA na corda** (§11.3) e os dois tectos por medir do `motion.wave` e do
  `motion.boids` ([doc 91](91_os_tetos_que_ninguem_mediu.md)).
- ⏳ **A ALÇA no canvas** (P0 da folha 16) fica fora da lista por ser **UI e não param** — um gesto
  de canvas novo neste módulo compete com a selecção, e a decisão é de produto (§5).

---

## §16 — ⭐⭐⭐ A PEÇA VESTE O OSSO (ordem do dono, 2026-09-20: *«DEVE SIM»*)

Perguntado se *«uma peça de osso deve ajustar-se sozinha ao tamanho do osso»*, o dono respondeu
**DEVE SIM** — e com isso o item que a §15.2 deixou aberto fecha.

### §16.3 — ⚠️ O PREÇO, aceite e nomeado

O `size` deixa de ser o que o artista escreveu na forma — e o que ele escreve passa a ser uma
**RAZÃO**: o `amount` do `motion.scale` multiplica a base que a lei dá (`1` = justo · `0,9` =
folga · `0,5` = contas soltas).

⛔⛔⛔ **E o resto desta secção dizia, como «preço aceite», EXACTAMENTE o defeito que o dono
reportou uma hora depois** — *«a espessura da corda passa a seguir o `Count`: a `40` a peça mede
`1,90×` mais fina»*. A §16.5 é a correcção, e a frase fica aqui porque *um preço que eu declarei
aceite sem o ter posto à frente de quem paga não era um preço: era um defeito com um nome bonito*.

### §16.4 — ⛔ A cena `=125` tinha uma SEGUNDA CÓPIA da lei

Ela declarava `const TAMANHO: f32 = OSSO / 2.0` com o `½` do contrato explicado em prosa ao lado —
**a mesma conta**, escrita à mão. Ela acertava por os ossos daquela cadeia terem todos o mesmo
`length`, e ficava para trás no dia em que um deles não tivesse. Hoje a forma não escolhe tamanho
nenhum (o `size` de fábrica dela é `1`) e o `point_scale = 1` deixa a corrente decidir — a saída é
a MESMA (`2 × size = OSSO`, gateado), e é isso que torna a troca honesta.

⚠️ **E o cabeçalho dos gates daquela cena declarava uma LEI DO ARNÊS que não existe** (*«o que
passa por um `source.shape` coze `n = 0` headless»*): a cena irmã já tinha achado a porta, e ela é
**uma linha** — o `motion_shape_gen::publish`. Com ela o que a peça DESENHA é medível ali.

