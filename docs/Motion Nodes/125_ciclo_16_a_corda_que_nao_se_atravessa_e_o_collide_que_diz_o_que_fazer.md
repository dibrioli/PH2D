# 125 — CICLO 16: a corda que não se atravessa, e o `Collide` que diz o que fazer

> **Ordem do dono, 2026-10-08** (depois do smoke da `=129` aprovado): *«vamos corrigir tudo»* — os
> cinco abertos do [handoff de continuação](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-08_CICLO_16.md).
> Base: `line/motion-value` sobre o `main` `2a925859d` (os ciclos 13, 14 e 15 por integrar). Cada item
> foi auditado contra o CÓDIGO e medido antes de construir; o desenho e os kill-criteria (§0) foram
> escritos antes da construção de cada um.

## §0 — O estado medido, o desenho e o kill-criterion

### §0.1 — Item 1: o Bug #16 (a corda com colisor atravessa-se num chicote a `60 Hz`)

**Medido (doc 124 §2.3):** a fixtura `rope_thickness` (`24` nós, raio `0,5`, a cabeça a
`3,0·sin(2πt/0,6)`) pelo laço antigo `rope --pre--> motion.collide → rope.state` segura `0,811·D`
no fim e `0,0497` na 2.ª metade; com sub-passos do GRAFO, `0,52`/`0,81`/`0,80` a `2`/`4`/`8`. O
`motion.collide` está fora do catálogo: o artista não monta esse laço. A corda não lia o colisor de
peça nenhuma (`grep colisores` vazio na crate).

**Desenho (o padrão-ouro: o simulador LÊ o colisor das peças que simula, dentro do passo — o caminho
que o corpo mole ganhou no ciclo 15, doc 124 §6):**
- D1. A corda ganha a porta `shape` (a ÚLTIMA, `anchor_x`/`anchor_y`/`state` ficam em `0/1/2`):
  `Shape → Verlet Rope`. As peças distribuem-se pelos nós como o `Cycle` do Duplicator
  (`nó i ← peça i mod m`): uma forma só veste a corda inteira. A corda passa adiante a aparência delas.
- D2. Com o `Collide` do Shape ligado, o contacto do motor da casa (`ph2d_contact`) corre DENTRO do
  laço de restrições — e mede-se ONDE: (a) por iteração (o Vellum), (a') por sub-passo (o corpo
  mole), (b) o relógio do grafo, (c) o `maxSpeed` do FleX, e (d) um passo seguro (CFL: os sub-passos
  sobem até cada nó andar no máximo `k` alcances).
- D3. Os pares que o repouso sobrepõe (os vizinhos da corrente) são da corda (o filtro do ciclo 15).
- D4. Sem declaração (o `Collide` desligado, ou sem porta) a corda é a de sempre, ao bit.
- D5. O OUTRO APP corre-se: o oráculo Godot do ciclo 15 estende-se a uma CORRENTE de contas sobre a
  nossa fixtura (`ferramentas/godot_oracle_corrente/`), e a saída é fixtura do gate.

**Kill-criteria:** (1) pares não ligados `≥ 0,9·D` na 2.ª metade, pela porta do artista; (2) o
comprimento `≤ 1,10 ×` o da corda nua, na 2.ª metade inteira (não só no fim); (3) o controlo
alcançável — o `Collide` do cartão Shape, no catálogo; (4) desligado ⇒ byte-idêntico; (5) o custo no
tecto (`32 768` nós) medido; (6) a cena e o tutorial fotografados.

### §0.2 — Item 2: o `Collide` do Shape DEPOIS do corpo

**Medido (doc 124 §6):** em `Soft Body → Duplicator ← Shape → Output` os dois botões `Collide` visíveis
não fazem nada sozinhos (`0,1406` nas quatro combinações menos uma); juntos só arrumam o DESENHO. E
auditado no código: a declaração do Shape só tem leitores a jusante dele (o corpo lê a porta `shape`;
o passe do fim só corre com o `Collide` do Output); o diagnóstico não conhecia colisor nenhum.

**Desenho:** o padrão-ouro das referências é o simulador ler a SUA ENTRADA (Vellum: o `pscale` da
geometria de entrada; Cavalry: o Soft Body sobre a forma) — nenhum lê um nó a jusante, e fazê-lo
partiria o fluxo de dados (a saída de um nó dependeria dos params de quem o consome). ⇒ **o grafo DIZ
ao artista o que fazer**: um déficit novo (`ColisorMudo`) no diagnóstico, com a cura nomeada —
`"shape"` (a forma veste uma simulação: ligue-a à porta `shape` do corpo), `"output"` (nada lê a
declaração: ligue o `Collide` do Output), `"collide"` (o Output arma o passe e nada declara). O
registo ganha as duas chaves (quem DECLARA por interruptor, quem ARMA o passe), ao lado de quem LÊ.

**Kill-criteria:** (1) um censo pela bomba do produto que liga CADA `Collide` sozinho, em cada
montagem que o artista faz, e exige EFEITO no que se desenha OU o conselho no cartão; (2) nenhum
conselho onde o botão funciona; (3) cada conselho tem frase própria (o censo `Deficit::ALL`).

### §0.3 — Item 3: o `motion.scatter` com o `count` por fio

**Medido no código de hoje** (`measure_scatter_cost`, estendida a `3 000`/`10 000`/`32 768`):

| pontos | release | debug |
|---|---|---|
| 3 000 | `1,99 ms` | `8,0 ms` |
| 10 000 | `6,75 ms` | `27,1 ms` |
| 32 768 | `23,8 ms` | `94,7 ms` |
| 50 000 | `38,2 ms` | `149,8 ms` |

⭐ **`0,7 µs` por ponto a qualquer contagem — o nó é LINEAR**, e nas três caixas medidas (`800`,
`4`, `12`). A `NearGrid` de 2026-09-05 tirou-o de `O(count²)`; o tecto digitável de `3 000` e a nota
*«O(count²)»* ao lado dele mediam o algoritmo de ANTES, e ninguém os reconferiu quando a grelha
moveu o número (`CLAUDE.md` §0.0). E o *«congela o app por minutos»*: a `32 768` o `scatter` coze em
`95 ms` em debug — **a atribuição do censo do ciclo 15 era por crença**, e o que não acabou em `10`
minutos não era ele (§3.2). Não há rota de placa: o nó é `Pure` e coze quando um param muda.

**Desenho:** o tecto digitável passa a ser o da casa (`MAX_INSTANCIAS_POR_NO`) — o mesmo número em que
o `eval` corta, logo a caixa e o fio dizem a mesma verdade; um gate que CONTA as distâncias por ponto
(não o relógio) defende a linearidade. A família inteira mede-se: cada param que comanda a contagem
de cada criador, a `1e9` (o que um fio entrega cru ao `eval`).

**Kill-criteria:** (1) digitável = corte do `eval`; (2) o trabalho por ponto plano de `3 000` a
`32 768` (`≤ 1,5×`), em três caixas; (3) a sonda da família a `1e9` identifica quem não acaba, ou
mostra que ninguém.

### §0.4 — Item 4: o `ph2d-render` com o desenho do SIGSEGV

**Auditado:** o binário `it` do `ph2d-render` compila **13** `static OnceLock<Option<GpuContext>>`
(não `~20`; os `5` de `src/` vivem noutro binário). O CI corre-o por `nextest` (um processo por
teste); a família do ciclo 14 era o SIGSEGV ao SAIR de um processo com muitos contextos vivos.
**Desenho:** medir primeiro, os dois caminhos intercalados (o binário inteiro num processo, e o
`nextest`), a faixa normal e a de placa (`--ignored`), com o `rc` e a carga ao lado. Reproduz ⇒ a cura
do ciclo 14 (um contexto, um censo); não reproduz ⇒ o número e fecha.

### §0.5 — Item 5: a sonda `FIELD_DOC_VERSION`

**Confirmado no ficheiro:** `scripts/collision-surface.sh:91` diz *«o `FIELD_DOC_VERSION` saiu com a
`ph2d-field`, ADR-0179 — o número já não existe, a sonda também não»*, desde o `69b28daa5` (no `main`).
Os handoffs `13`/`14`/`15` davam-na por aberta: a referência fecha aqui (não se reconstrói nada).

## §1 — Item 1: a corda lê o colisor das peças

### §1.1 — O oráculo: uma corrente de contas no Godot (`4.7.2`, MIT)

`ferramentas/godot_oracle_corrente/` (`corre.sh` grava a `saida.txt` que o gate LÊ): `24` corpos
rígidos de raio `0,5`, ligados em fila por molas rígidas (`k = 1e6`, `4 800 Hz` — o Godot 2D não tem
junta de distância), a cabeça cinemática na âncora da fixtura, `4 s`:

| modo | pior não ligado (fim) | 2.ª metade | arco no fim | maior arco na 2.ª metade |
|---|---|---|---|---|
| nu | `1,0513` | `0,8631` | `6,169` | `6,329` |
| **lei** (os pares de repouso não colidem) | `1,0414` | **`0,9957`** | `6,136` | `6,665` |
| todos colidem | `1,0073` | `0,9570` | **`24,416`** | `24,499` |

⇒ com a lei a corrente de referência segura o diâmetro e não estica; colidindo os vizinhos, ela
INCHA para `4×` — a assinatura do Bug #15, agora numa corrente.

### §1.2 — A rodada: ONDE o contacto corre (`sonda_onde_o_contacto_corre`, release, mesmo processo)

| variante | 2.ª metade | arco máx (nua `6,420`) | sub-passos / `240` quadros |
|---|---|---|---|
| nua, `Substeps 1`…`8` | `0,0096`…`0,0055` | `6,42`…`6,01` | `239`…`1 912` |
| (a') por sub-passo, `1`/`2`/`4`/`8` | `0,15`/`0,33`/`0,61`/`0,73` | `8,45`/`6,83`/`6,30`/`6,27` | — |
| (a) por iteração, `1`/`2`/`4`/`8` | `0,915`/`0,910`/`0,962`/`0,982` | **`7,88`**/`6,73`/`6,24`/`6,20` | — |
| (c) `maxSpeed` `1` / `½` alcance | `0,90` / `0,93` | `8,55` / `7,09` | a cauda: `8,19` / **`3,67`** (o chicote morre) |
| (d) CFL `1` / **`½`** / `¼` alcance | `0,924` / **`0,966`** / `0,967` | `7,12` / **`6,50`** / `6,21` | `413` / **`691`** / `1 245` |

⭐ **O contacto tem de correr DENTRO das iterações** (por sub-passo não segura nem a `8`), e a
`Substeps 1` ele já segura (`0,915·D`) — mas a corda ESTICA `23 %` durante as dobras (as iterações
não convergem contra o contacto). O passo seguro a meio alcance segura `0,966·D` com o comprimento
`1,01×`, pagando `2,9` sub-passos por quadro só enquanto a corda anda depressa. O `maxSpeed` muda o
chicote (a cauda passa de `5,26` a `3,67`): recusado.

### §1.3 — A corda FINA, e a âncora que anda

A cena pediu contas de raio `0,12` (um colar: `2r ≈ 6/23`) e a corda deixou de segurar (`0,1451`
contra `0,24`). **Mecanismo:** a âncora saltava o quadro inteiro no 1.º sub-passo — a cabeça passava
ATRAVÉS das contas vizinhas, e o passo seguro, que lê o deslocamento, realimentava-se do salto
(pedia até `76` sub-passos):

| raio | âncora | 2.ª metade | pedido máx de sub-passos |
|---|---|---|---|
| `0,50` | salta / **anda** | `0,9659` / **`0,9745`** | `7` / `4` |
| `0,20` | salta / **anda** | `0,1851` / **`0,3862`** (`D 0,40`) | `76` / `9` |
| `0,12` | salta / **anda** | `0,1451` / **`0,2333`** (`D 0,24`) | `65` / `12` |

⇒ com o contacto, a âncora ANDA ao longo dos sub-passos (o alvo cinemático é contínuo). ⚠️ O
último sub-passo usa a âncora EXACTA: `c + (a − c)·1` não devolve `a` ao bit, e a primeira versão
mudava a corda a `Substeps 1` (`0,9154 → 0,8947`). Sem contacto a âncora continua a saltar — a corda
de sempre ao bit (os gates do `step` com sub-passos verdes sem uma letra mudada).

### §1.4 — A cena `=130` (e a `=131`)

O chicote com gravidade só cruza a corda nua por instantes (`33 %` dos quadros da 2.ª metade): uma
foto não o apanha. Medido (`sonda_as_receitas_da_corda`, pela bomba):

| receita | nua: quadros com contas atravessadas | com o `Collide`: pior par |
|---|---|---|
| chicote `0,6 s`, `g 9,8`, raio `0,12` | `33 %` | `0,2399` |
| roda (`x` e `y`, ¼ de volta) | `0 %` (a corda ESTICA para `7,3`–`8,3` e não se cruza) | — |
| sem gravidade, `1,0 s`, curso `3` AO LONGO da corda | `100 %` | `0,2398` |
| **sem gravidade, `1,3 s`, curso `3` DE LADO**, raio `0,12` | **`44 %`** | **`0,2369`** (`D 0,24`) |

⚠️ **A FOTO corrigiu a receita do meio:** sacudida ao longo do próprio comprimento a corda só encolhe
e estica EM FILA — a régua contava `100 %` de quadros com contas sobrepostas, e a foto mostrava uma
fileira quase recta (as fotos aos `8` e `12 s` saíram iguais: o mastro de `1 s` caía na mesma fase).
De LADO (a âncora `y`, perpendicular à corda) ela dobra-se sobre si mesma, e é isso que um artista vê.
⇒ a `=130` é a corda sem gravidade sacudida de lado, com UM só mastro (um cartão LFO que não mexe
nada seria um cartão mudo na cena). A `=131` é a mesma corda vestida DEPOIS
(`Rope → Duplicator ← Shape`) com o `Collide` ligado: o cartão do Shape aconselha, e o gesto que o
aviso pede (o fio do Shape para a porta `shape`) cala o aviso e segura (`0,2398`).

### §1.5 — O relógio do grafo (b), recusado pela medição que já existia

O ciclo 15 mediu-o: um `substeps` de grafo na corda fazia uma `sim.zone` VIZINHA mudar de resposta
(`19,011 → 19,298`) e custar `8×` (o comentário do manifesto). E as duas leis são a MESMA lei
(`1e-5` em `n = 1..16`): com o contacto dentro do `eval`, os sub-passos LOCAIS dão tudo o que os do
grafo davam, sem tocar no vizinho. A corda continua a não declarar o relógio.

### §1.6 — Construído e medido (os kill-criteria)

| kill-criterion | resultado |
|---|---|
| (1) `≥ 0,9·D` na 2.ª metade, pela porta do artista | `the_rope_reads_its_pieces_collider_and_does_not_cross`: `0,0138 → 0,9598` (o oráculo: `0,9957`) |
| (2) o comprimento | `6,421 → 6,668` (`1,04×`), na 2.ª metade inteira |
| (3) o controlo alcançável | `turning_on_the_left_shapes_collide_makes_the_rope_hold`: o `Collide` do cartão Shape → `0,2398` |
| (4) desligado ⇒ ao bit | por construção (sem `Contacto` o `step` corre a mesma aritmética: `sub`, âncora e laço) e as suítes da corda verdes sem uma letra mudada |
| (5) o custo no tecto | §1.7 |
| (6) a cena | `the_left_rope_crosses_and_the_right_one_holds`: a nua `44 %` dos quadros, a com `Collide` `0,2369`, arco `6,094 → 6,015` · fotos §8 |

### §1.7 — O custo no tecto

`sonda_o_custo_no_tecto` (release, o mínimo de cinco intercalados, carga `1,4`–`2,6`), UM tique a partir
da semente; «fixtura» é a geometria da `rope_thickness` esticada a `32 768` nós (segmento `6/23`, raio
`0,5`), «densa» é o comprimento `6,0` com todos os nós (cada nó sobrepõe centenas de vizinhos de
repouso — o pior caso da grelha):

| corda | nua | contacto, 1 sub-passo | a lei, âncora devagar (`2` sub-passos) | a lei, um puxão (`8`) |
|---|---|---|---|---|
| fixtura, `32 768` | `6,2 ms` | `16,4 ms` | `32,3 ms` | `128 ms` |
| densa, `1 024` | `0,20 ms` | `5,7 ms` | `12,4 ms` | `53 ms` |
| densa, `4 096` | `0,84 ms` | `70 ms` | `150 ms` | `742 ms` |

O recurso é o tempo de CPU por tique (a corda não tem rota de placa), e o tecto é o que já existia: o
`MAX_SUBSTEPS = 16` da corda limita o que o passo seguro pede (o pior caso no tecto da fixtura,
`~260 ms`, só enquanto a corda é arrancada). Nenhum tecto novo: o contacto é opcional (o `Collide`
do Shape), a `24` nós custa microssegundos, e a densa é uma corda que se sobrepõe a si mesma no
repouso centenas de vezes — o custo é o da grelha, não um defeito.

## §2 — Item 2: o `Collide` que diz o que fazer

### §2.1 — O censo (`every_collide_alone_has_an_effect_or_an_advice`, pela bomba, `120` quadros)

| montagem | `Collide` do Shape sozinho | `Collide` do Output sozinho |
|---|---|---|
| `Grid → Duplicator ← Shape → Output` | sem efeito · conselho `"output"` | sem efeito · conselho `"collide"` |
| corpo mole, peças DENTRO (`=129`) | **segura** | sem efeito · `"collide"` |
| corpo mole vestido DEPOIS | sem efeito · `"shape"` | sem efeito · `"collide"` |
| corda, Shape DENTRO (`=130`) | **segura** | sem efeito · `"collide"` |
| corda vestida DEPOIS (`=131`) | sem efeito · `"shape"` | sem efeito · `"collide"` |

O CONTROLO de cada montagem (as duas chaves certas mudam o desenho) passa — senão «sem efeito»
seria verdade por vácuo. E `a_working_collide_is_never_advised`: nenhum aviso onde o botão funciona.

⚠️ **O censo das cenas acendeu-se em seis**, e cada uma foi medida antes de decidir: a `=114`/`=115`
eram um FALSO alarme da régua (numa zona as peças chegam ao `sim.step` pela aresta de ESTADO, e o
fecho saltava as `delayed` — curado); a `=121`–`=123` nascem desarmadas DE PROPÓSITO (doc 115 W5: o
passo do dono é ligar o `Collide` do Output, e o conselho diz exactamente isso) e a `=131` é a lição
do item — as quatro ficam numa lista nomeada (`ENSINAM_A_SEGUNDA_CHAVE`) com a catraca ao contrário.

**O que o dono vê:** o selo ⚠ no cartão do Shape (`inert_reaching_output`, a porta que o painel
pinta) e, ao clicá-lo, a frase do `explain`. ⚠️ O selo só se desenha com os cartões em detalhe
(com o grafo afastado eles são cápsulas) — o smoke manda aproximar o grafo.
`a_collide_after_the_rope_gets_the_badge_and_the_rewire_clears_it`: o selo está no Shape da `=131`,
a frase nomeia a porta `'shape'`, e o fio do gesto tira-o.

### §2.2 — ⭐ O passe do fim inchava um corpo de peças (o Bug #15 no passe)

O censo achou-o: o passe do fim (`ph2d_contact::passe`) não lia o `rest`/`body` que o corpo publica,
e com as duas chaves ligadas empurrava os vizinhos de malha até ao diâmetro — no DESENHO. A cura é a
mesma lei do `motion.collide`: os pares que o repouso sobrepõe ficam com o corpo
(`Pecas::com_ligados`); sem as duas colunas, byte-idêntico.
`the_end_pass_does_not_swell_a_body_of_pieces` (raio `0,28`, maior que meio passo): extensão
`3,0281 → 3,0285`; sem a cura (a mutação M5) `3,0281 → 3,7561` — o desenho inchava `24 %`.

## §3 — Item 3: o `scatter` e o fio acima do digitável

### §3.1 — A cura

O `PARAM_HARD_MAX` do `count` passa a `MAX_INSTANCIAS_POR_NO` com a tabela do §0.3 ao lado, e
`the_grid_keeps_the_work_per_point_flat` conta as distâncias medidas por ponto: `58,5 → 59,6`
(`4×4` e `12×12`) e `52,5 → 57,8` (a caixa alongada `40×0,5`), de `3 000` a `32 768`. A régua de custo
do `measure_the_count_ceiling` deixa de chamar o nó *«o quadrático»* e varre `3 000`/`10 000`/`32 768`.

### §3.2 — A família: cada contagem a `1e9` por fio

`sonda_cada_contagem_a_1e9_por_fio` (debug, como o censo; `target/prova/c16/fio_1e9.log`): `34`
params que COMANDAM a contagem (dobrá-los sobe-a), nos dois arranjos de entradas, cada um sozinho a
`1e9` — o que um `value.*` ligado entrega cru ao `eval`:

| criador | a `1e9` por fio | linhas |
|---|---|---|
| `motion.voronoi` · `count` | **`59,7`–`66,2 s`** | `32 768` |
| `motion.scatter` · `count` | `0,10`–`0,11 s` | `32 768` |
| os outros `30` (`grid`, `wave`, `verlet_rope`, `soft_body`, `emitter`, …) | `≤ 0,01 s` | ao tecto ou ao lado |

⇒ **o que não acabava em `10` minutos era o `voronoi`**, cuja CPU é o caminho de REFERÊNCIA de um nó de
placa — já isento por medição no censo (`LENTOS_NO_EXTREMO`: *«quem manda no tecto dele é o
dispositivo»*, `CLAUDE.md` §0.0). Nenhum criador estoura nem passa o tecto por fio. A nota do censo
que acusava o `scatter` foi corrigida.

## §4 — Item 4: o `ph2d-render` — NÃO reproduz

O binário `it` (`13` contextos estáticos), `8` corridas de cada caminho, intercaladas, com o `rc` e a
carga de cada uma (`target/prova/c16/render_*`):

| faixa | caminho | corridas | SIGSEGV | o resto |
|---|---|---|---|---|
| normal (`114` testes) | o binário inteiro num processo | `8` | **`0`** | `1` reprovação fixa: o teste lê `renderer_draw.rs` por caminho relativo e a corrida era da raiz (artefacto do arnês) |
| normal | `nextest` (o CI) | `8` | **`0`** | `114/114` verdes `8/8` |
| placa (`--ignored`, `163`) | o binário inteiro num processo | `8` | **`0`** | `4` reprovações fixas: os goldens `w2`–`w5` do `smoke_fixture_renderable`, `unimplemented!()` por desenho (as notas deles dizem que os PNG não existem); e `1/8` do `the_pointwise_op_costs_much_less_than_a_blur` a carga `28` (uma razão de tempos — família dos flakes de carga) |
| placa | `nextest` | `8` | **`0`** | as mesmas `4` |

⇒ **`0` de `32`**: o desenho dos `13` `OnceLock` não reproduz o SIGSEGV do `ph2d-gpu-cook` nesta
máquina (carga `5`–`65` durante a série). Fechado com o número; a cura do ciclo 14 fica disponível
se um dia reproduzir. ⚠️ Os `4` goldens e o teste de tempo não são deste módulo — registados no
handoff para o integrador.

## §5 — Item 5: a sonda `FIELD_DOC_VERSION`

Fechada no `main` (`69b28daa5`); as referências dos handoffs de integração `13`/`14`/`15` apontam
para aqui.

## §6 — Provas de mutação — `12/12` sangram

Pré-voo das âncoras (cada uma `1×` no ficheiro), os alvos VERDES antes, restauro ao bit (sha256) com
`touch`, corrida limpa no fim (verde):

| M | a lei | a mutação | sangrou |
|---|---|---|---|
| M1 | o contacto dentro das iterações | sem a chamada por iteração | `the_rope_reads_its_pieces…` (`0,0051 < 0,9`) |
| M2 | a âncora anda | `anda = false` | a cena `=130` (`0,1835 < 0,216`), o gesto, a `=131` |
| M3 | o passo seguro | `cfl` devolve o `Substeps` do artista | a fixtura (estica) e as três da cena (`0,2159 < 0,2160`) |
| M4 | o filtro do repouso na corda | sem `com_ligados` | a fixtura: `11,562` contra `6,421` (a corda INCHA quase `2×`) |
| M5 | o filtro do repouso no passe | sem `com_ligados` | `the_end_pass_does_not_swell…`: `3,0281 → 3,7561` |
| M6 | o `ColisorMudo` | o diagnóstico nunca o diz | o censo dos botões, a `=131`, e a catraca ao contrário (`=121` saiu de `ENSINAM_A_SEGUNDA_CHAVE`) |
| M7 | a cura `"shape"` | o corpo a montante nunca visto | a `=131` (espera `"shape"`) e a catraca (`=131` saiu da lista) |
| M8 | as arestas de estado no fecho | o fecho salta as `delayed` | o censo das cenas: `=114`/`=115` acusadas |
| M9 | digitável = corte do `eval` | `max: 3_000` | `the_typed_ceiling_is_where_the_eval_clamps` e `the_scatter_ceiling_is_the_house…` (pelo painel) |
| M10 | a grelha linear | sem a paragem conservadora do anel | `the_grid_keeps_the_work_per_point_flat`: `17 994 → 196 602` distâncias por ponto |
| M11 | a corda regista-se como leitora | sem `register_declared_collider_reader` | o censo dos leitores, `a_working_collide_is_never_advised`, a `=131`, o censo das cenas |
| M12 | a ficha pinta o texto que mediu (§8.1) | o orçamento volta a ser o `label_budget` | `a_chip_paints_the_whole_text_it_measured`: a ficha de `63,5 px` pinta `«SEM Coll…»` |

⚠️ O M3 sangra a cena por `1e-4` (a corda fina a `Substeps 1` com contacto por iteração quase
segura); quem o apanha com folga é a fixtura (o comprimento, `1,23×` na rodada do §1.2).

## §7 — O que fica aberto

Nada dos cinco. Registado para o integrador (não é deste módulo): os goldens `w2`–`w5` do
`ph2d-render` (promessas `unimplemented!()` por desenho) e o teste de tempo
`the_pointwise_op_costs_much_less_than_a_blur` (`1/8` a carga `28`).

## §8 — Smoke (fotografado em `release`, `target/prova/c16/fotos/`)

As fotos da `=130` aos `9` e `12 s` (a laranja dobra e as contas ficam umas sobre as outras; a verde
dobra e as contas só encostam) e da `=131` aos `6 s` (a corda vestida depois, a atravessar-se). O
selo ⚠ da `=131` não se fotografa (a foto não aproxima o grafo: os eventos sintéticos não chegam ao
app) — o gate do §2.1 prova-o pela porta que o painel pinta.

### §8.1 — ⭐ A foto achou um defeito da FICHA da casa: toda ficha cortava as últimas letras

As legendas saíam `«Collide DESLIGADO: as contas atravessam…»`, `«SEM Collide: atravessam…»`,
`«SEM Col…»` — quanto mais curto o texto, MENOS se lia. **Mecanismo:** a ficha
(`ph2d_editor_core::readout::paint_chip`, partilhada pelas leituras do canvas, pelos rótulos de encaixe
do Vector e pelas legendas das cenas) mede-se com a folga dela (`CHIP_PAD_X_PX`, `6 px` de cada lado)
e pintava o texto com o orçamento de um RÓTULO (`label_budget`, `Spacing::Md` de cada lado) — a folga
contada duas vezes à mão divergia por uma constante, e a elisão comia sempre os mesmos píxeis (uma
fracção maior de um texto curto). **Cura:** a ficha pinta com o orçamento da MESMA folga com que se
mediu (`chip_text_budget`, pela porta `paint_text_centered_com_orcamento` que já existia para isto);
nenhuma ficha muda de tamanho, todas passam a mostrar o texto inteiro.
`a_chip_paints_the_whole_text_it_measured` (sete fichas reais) e a mutação M12 (§6).
