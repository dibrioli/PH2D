# 124 — CICLO 15: o tecto por nó em TODO criador, e o colisor que respeita o corpo mole

> **Ordens do dono, 2026-10-08** (depois do ciclo 14): *«o nó de corpo mole deve ter limite de
> 32 768»* e *«o defeito pode consertar conforme padrão ouro»* (o Bug #15). A lista é a do
> [handoff de continuação](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-08_CICLO_15.md).
> Base: `line/motion-value` sobre o `main` `2a925859d` (os ciclos 13 e 14 por integrar). Cada item foi
> auditado contra o CÓDIGO antes de construir; o desenho e os kill-criteria (§0) foram escritos ANTES.

## §0 — O estado medido, o desenho e o kill-criterion (escritos ANTES)

### §0.1 — Item 1: quem PRODUZ mais que o tecto (o censo)

O censo leu cada `crates/ph2d-node-*` e separou três espécies — e só a primeira é a ordem do dono
(*«nenhum nó que cria instâncias pode gerar mais de»* o tecto, `node.rs:MAX_INSTANCIAS_POR_NO`):

| espécie | nós | hoje | decisão |
|---|---|---|---|
| **A — o nó PRODUZ acima do tecto a partir dos SEUS params/dados** | `motion.soft_body` | lado `512` (literal) ⇒ `262 144` | **cortar**: cada lado em `LADO_MAX_DE_GRELHA` (o molde do `motion.grid`, `d60a8398c`) ⇒ `181² = 32 761` |
| | `motion.emitter` | `MAX_ALIVE = 4·1024²` (um orçamento de MEMÓRIA da placa, de julho — antes da ordem de 21/09) | **cortar**: `MAX_ALIVE = MAX_INSTANCIAS_POR_NO`; a lei de contagem é UMA para as duas rotas (ADR-0130), logo o corte vale na placa sem segunda cópia |
| | `motion.path` | `RECOMMENDED_MAX_ELEMENTS = 2^24` (cerca de SEGURANÇA, não de produto) nos dois modos (`count` e `spacing`); o `ParamHardMax` de `count` é um literal `500` | **cortar** nos dois modos; `count` digitável = o tecto, derivado (como o `distribute_curve`) |
| | `motion.distribute_poisson` | `MAX_CELLS = 2^18` (memória da grelha; um ponto por célula ⇒ até `262 144`) | **cortar** pelo RAIO: o piso do raio deriva do tecto pela desigualdade de Oler (pontos a distância `≥ r` num convexo: `N ≤ (2/√3)·A/r² + P/(2r) + 1`) — o tecto é PROVADO, sem aparar a frente do Bridson (aparar deixaria um buraco na distribuição) |
| | `source.table` | sem tecto (*«um milhão cabe num quadro»*, medido antes da ordem); o tutorial 08 promete o milhão | **cortar**: as primeiras `MAX_INSTANCIAS_POR_NO` linhas (`Index`/`Count` do que sai); o `value.table` (cada linha é um MOMENTO, não um objecto) não muda |
| **B — MULTIPLICA o que chega** (`factor × entrada`) | `motion.trail` · `motion.mirror` · `motion.combine` · `fx.rgb_split` · `fx.drop_shadow` | tectos MEDIDOS no dispositivo por recurso (`instance_ceiling_agrees.rs`) e a lista `NOMEADOS` do gate `nenhum_no_emite_mais_do_que_o_tecto` — uma decisão já escrita: cortá-los apaga a razão de ser do nó (a sombra some com a entrada no tecto) | **não se corta por analogia** — é outro recurso (linhas a jusante), medido e com catraca |
| **C — passa a contagem de montante** | `motion.soft_body` pela porta `shape`, todo modificador | `n` de saída = `n` de entrada | não é criador |

**Barra (gate de censo):** `nenhum_criador_passa_o_tecto_com_a_contagem_no_extremo` — a população é
COMPORTAMENTAL (quem emite mais do que recebe), e para cada criador os params que COMANDAM a contagem
(variar um muda o número) são levados ao EXTREMO todos juntos (`1e9` se a contagem sobe com ele, `0`
se desce), num ponto fixo (o `max` do emissor só comanda depois de o `rate` subir), num cozimento a
`t = 10 s`. Piso de população, controlo positivo (o `motion.grid` no extremo dá EXACTAMENTE
`CELULAS_DE_UMA_GRELHA_CHEIA`). Os dois que leem o mundo (`motion.path` · `source.table`) não
aparecem sem um canal publicado ⇒ gate próprio na crate de cada um.

**Kill-criterion:** o gate VERMELHO sobre o código de hoje (o corpo mole, o emissor e o Poisson
acima do tecto), verde depois; cada corte com a mutação «tecto antigo» a sangrar; a cascata (o tutorial
09 «Soft Body 512×512, 2,26 ms» re-medido, a cena `=5` de `1,2 M` partículas, o tutorial 08 e o milhão
da tabela, as figuras e os PDFs).

### §0.2 — Item 2: Bug #15 (o colisor no laço de um corpo mole)

**Estado medido ANTES** (`sonda_o_colisor_no_laco_do_corpo_mole`, a fixtura `soft_body_radius`: `5 × 5`,
passo `0,30`, raio `0,28` ⇒ diâmetro `0,56`; «ligado» = distância de REPOUSO `< 2·raio`, os quatro
vizinhos e as diagonais):

| régua | antes |
|---|---|
| pior par NÃO ligado à saída do COLISOR (2.ª metade · fim) | **`0,2237`** · `0,3978` |
| pior par NÃO ligado à saída do CORPO (o que se desenha; 2.ª metade · fim) | **`0,0236`** · `0,5434` |
| par ligado mais próximo (fim) | `0,2890` (passo `0,30`) |
| extensão: corpo nu → com colisor | `1,9221 → 2,7571` |

**O que a composição já exprime (auditoria, §5.0):** o `motion.collide` não sabe quem é vizinho — o
`soft_body` publica só `P`/`sb_vel`/`sim_t`. A corda (`rope_thickness`) «funciona» porque o Verlet lê o
deslocamento do colisor como VELOCIDADE, e o corpo mole não: o empurrão entra na posição e a velocidade
do passo seguinte é a de ANTES do contacto — o corpo volta a entrar no vizinho a cada tique. O passe do
fim do cozimento (`ph2d_contact::passe`, o `Collide` do cartão) só age sobre colisores DECLARADOS e o
corpo não declara nenhum; o `sim.collide` não é alcançável por uma malha. ⇒ a família é o
`motion.collide`, nas duas rotas.

**O ORÁCULO, corrido sobre a NOSSA fixtura** (`ferramentas/godot_oracle_corpo_mole/corpo_mole.gd`,
Godot `4.7.2`, **MIT** por `pacman -Qi godot`; círculos de raio `0,28` ligados por molas aos pares
sobrepostos no repouso, a linha de topo presa à âncora que varre `±2,0` com período `0,7 s`, `3 s` a
`960 Hz`; o contacto é o solver do Godot):

| modo | pior não-ligado (fim · 2.ª metade) | par ligado mín. | extensão |
|---|---|---|---|
| `nu` (sem colisão) | `0,5162` · `0,4877` | `0,2459` | `1,8094` |
| ⭐ `lei` — os pares ligados NÃO colidem (`add_collision_exception_with`) | **`0,5603` · `0,5563`** | `0,2673` | **`1,8005`** |
| `todos` — todos os pares colidem | `0,6000` · `0,6000` | `0,3000` | **`2,7601`** |

⭐⭐ **O «inchar» era a ASSINATURA do defeito, não a lei.** O modo `todos` do oráculo reproduz o
`2,76` que o ciclo 14 tomou por *«o que um particle radius faz»*; com a lei, o corpo NÃO incha
(`1,81 → 1,80`) e os não-ligados ficam no diâmetro menos `0,0037` (a folga do próprio solver).
⇒ o kill-criterion do prompt *«a extensão continua a inchar»* é **refutado pelo oráculo** e sai.

**Desenho (o padrão-ouro):**
- (a) a lei de exclusão: **um par do MESMO corpo cuja distância de REPOUSO é menor que `r_i + r_j` não
  colide** — o `eNvFlexPhaseSelfCollideFilter` do FleX (Macklin et al. 2014, a MESMA família que o
  `push_apart` já cita para o Jacobi médio, e cujos corpos moles SÃO casamento de forma), e o
  `collideConnected = false` do Box2D/Godot para juntas explícitas. Num casamento de forma não há
  arestas: a restrição liga o corpo inteiro, e o repouso é o que diz quem está junto.
- (b) o corpo DECLARA o repouso: `motion.soft_body` e `motion.verlet_rope` publicam `rest` (`Vec2`, a
  pose de repouso; o nome do Houdini) e `body` (`Scalar`, `node_key + 1`; `0` = sem corpo, que é o que o
  zero-fill do `motion.combine` dá a uma linha de outra fonte).
- (c) o `motion.collide` lê as duas colunas na CPU (`push_apart`) e no WGSL; ausentes ⇒ nenhum filtro ⇒
  byte-idêntico em toda cena que existe.
- (d) o contacto entra na VELOCIDADE do corpo mole (PBD: `v = (x_final − x_antes)/dt`): o corpo publica
  a posição que EMITIU (`sb_pos`) e o passo (`sb_dt`), e no tique seguinte soma
  `Δ·keep/dt_anterior` à velocidade, com `Δ = P_que_volta − sb_pos`. `Δ = 0` é exacto ao bit.
- ⛔ **Recusado:** o corpo escrever `inv_mass = 0` nos seus pinos para o colisor os tratar como
  obstáculo — o estado é a saída dele, então desligar o `pin` deixava um tique pinado, e o
  `motion.pin_constraint` escreve a MESMA coluna no laço.

**Kill-criterion (antes de construir):**
1. à saída do COLISOR, o pior não-ligado na 2.ª metade `≥ 0,5563` (o número do oráculo: `2·r − ε`,
   `ε = 0,0037`, a folga medida do solver de referência);
2. à saída do CORPO (o desenhado), o pior não-ligado na 2.ª metade `≥ 0,9·2r = 0,504` — o passo
   seguinte volta a mover o corpo, e a régua diz quanto;
3. a estrutura: par ligado mínimo `≥ 0,9 · passo = 0,27`;
4. ⭐ a extensão NÃO incha: `|extensão com colisor − extensão nua| ≤ 5 %` (o oráculo: `0,5 %`);
5. o corpo continua a simular (o gate que existe) e a corda continua a ter espessura (`rope_thickness`);
6. paridade CPU × placa do `motion.collide` com `rest`/`body` (gate `placa:`);
7. o custo: o colisor da CPU é `O(n²·iter)` — medido no tecto do item 1 (`181² = 32 761`), com o recurso
   dito; a rota da placa é a grelha espacial.

⚠️ **Revisto com a medição (§2), antes de construir o gate:** o critério 3 (`≥ 0,27`, absoluto) foi
refutado pelo próprio corpo NU (`0,2511` a 4 sub-passos) e pelo oráculo (`0,267` com a lei) — o
colisor não comprime a malha, é o corpo que a comprime ao balançar. ⇒ o critério passa a ser
RELATIVO ao corpo nu (`≥ 0,95 ×`). E os critérios 1–2 só se cumprem com sub-passos (§2.2).

## §1 — Item 1: o tecto em todo criador

| nó | antes | depois | gate |
|---|---|---|---|
| `motion.soft_body` | lado `512` (`262 144`) | lado `LADO_MAX_DE_GRELHA` (`32 761`); `clusters` digitável `MAX_SIDE/2` | `the_mesh_never_passes_the_per_node_ceiling` (`1e9 × 1e9` ⇒ `CELULAS_DE_UMA_GRELHA_CHEIA`) |
| `motion.emitter` | `MAX_ALIVE = 4·1024²` | `MAX_ALIVE = MAX_INSTANCIAS_POR_NO` (as duas rotas: a lei de contagem é uma) | o censo (`4 083 858 → 32 764` no extremo) · `the_emitter_fountain_demo_is_actually_dense` |
| `motion.path` | `2^24` nos dois modos; `count` digitável `500` | `MAX_INSTANCIAS_POR_NO` nos dois; digitável derivado | `neither_mode_passes_the_per_node_ceiling` (curva de `10 000` no piso do espaçamento; `count = 1e9`) |
| `motion.distribute_poisson` | um ponto por célula, `MAX_CELLS = 2^18` | piso do raio pela desigualdade de Oler — PROVADO | `the_radius_floor_keeps_every_rectangle_under_the_ceiling`: no piso o Bridson entrega **`17 056`–`17 426`** pontos (`52–53 %` do tecto) em `4×4`, `20×10`, `100×100`, `300×2` |
| `source.table` | sem tecto | as primeiras `MAX_INSTANCIAS_POR_NO` linhas; `Index`/`Count` do que sai | `a_table_above_the_ceiling_delivers_its_first_rows` |

**O gate de censo** `nenhum_criador_passa_o_tecto_com_a_contagem_no_extremo` (`ph2d-app-motion`,
`0,9 s` em debug): cada nó em DOIS arranjos (sem entradas · todas as portas — alimentar a porta
`shape` tirava o corpo mole do censo, e era ele o caso da ordem), os params que comandam a contagem
levados ao tecto DIGITÁVEL numa subida MONÓTONA, controlos (a grelha dá exactamente
`CELULAS_DE_UMA_GRELHA_CHEIA`; o corpo mole tem de ser medido; piso de `20` criadores), e uma
catraca `LENTOS_NO_EXTREMO` com censo de obsolescência para o `motion.voronoi` (verificado pelo que
DECLARA). No extremo, `13` criadores chegam ao tecto exacto (`32 761`/`32 764`/`32 766`/`32 768`).

**⛔ Recusas MEDIDAS na construção do censo:**

| tentativa | o que a máquina disse |
|---|---|
| params que comandam a contagem a `1e9` | o `motion.scatter` honra no `eval` o tecto da casa mas é `O(count²)` (o digitável dele é `3 000`, de recurso): a corrida não acabou em `10` min |
| os params que NÃO comandam no PISO (barato) | o `source.lsystem` caía de `4 095` para `1` e o `motion.trail` de `6` para `3` — um piso apaga a saída e a régua leria «abaixo do tecto»; o cheque «o extremo encolheu» apanhou-o |
| cada param ao seu extremo por conta própria | o `motion.distribute_curve` caía de `12` para `1` (a contagem nasce do comprimento da curva) ⇒ a subida monótona |
| o `motion.voronoi` cozido no extremo | Lloyd `O(pontos × amostras)` na CPU de referência a `32 768`: minutos ⇒ catraca nomeada |

**Os multiplicadores NÃO se cortam** (`motion.trail` · `motion.mirror` · `motion.combine` ·
`fx.rgb_split` · `fx.drop_shadow`): são `factor × entrada`, têm tectos MEDIDOS por recurso no
dispositivo (`instance_ceiling_agrees.rs`) e a lista `NOMEADOS` do gate
`nenhum_no_emite_mais_do_que_o_tecto` — outro recurso, já decidido.

**A cascata:** a cena `=5` (a fonte de partículas) corria `1,2 M` — passa ao tecto
(`12 000/s × 3 s > 32 768`, a janela continua a deslizar; grão `0,04`); as sondas do emissor, do
empacotamento na placa, da costura das fontes e do grupo do corpo mole tinham escadas acima do
tecto (mediriam `32 768` com o rótulo de outro número) — param no tecto, derivado. O que se mediu
acima do tecto continua verdadeiro e fica aqui: o corpo mole a `512²` custava `1,43 ms` (uma
moldura) e `5,3–5,9 ms` com `clusters`; o emissor a `4,19 M` simulava em `3,6 ms` na RTX
(`~370 MB` de residência); uma tabela de um milhão de linhas LIA-SE em `620 ms` (uma vez) e
copiava-se em `6,90 ms` por quadro.

## §2 — Item 2: o Bug #15

### §2.1 — A lei (o filtro do repouso) e o contacto na velocidade

`motion.soft_body` e `motion.verlet_rope` publicam `rest` + `body`; o `motion.collide` (CPU
`Estrutura::segura` e o WGSL, ao verbatim) não separa um par do mesmo corpo cujo repouso já o põe a
menos de `r_i + r_j`; sem as colunas, byte-idêntico (`without_the_body_columns_the_packing_is_bit_identical`).
O corpo mole guarda o que EMITIU (`sb_pos`, `sb_dt`) e soma `Δ·keep/dt` à velocidade do passo
seguinte. Gates: `a_pair_the_rest_pose_overlaps_is_held_by_the_body_not_the_collider` (o par
segurado; o mesmo par longe no repouso, de corpos diferentes e sem corpo colidem) ·
`the_body_declares_its_rest_and_its_identity` · `the_body_filter_packs_the_same_on_both_routes`
(placa, dois corpos dobrados um sobre o outro e linhas sem corpo; controlo: sem a identidade a pose
muda).

### §2.2 — Sem sub-passos não chega: o tempo

A fixtura do ciclo 14 (`5 × 5`, a âncora a varrer `±2,0` em `0,7 s`) move a linha de topo um
espaçamento INTEIRO por quadro. Medido pela porta do pump (`substep_islands` → `Cook::substep`),
`stiffness 0,6`:

| sub-passos | nu: corpo (pior solto) | colisor (2.ª metade) | corpo desenhado | ligado mín. (nu → com) | extensão (nu → com) |
|---|---|---|---|---|---|
| 1 | `0,0285` | `0,4378` | `0,0470` | `0,0588 → 0,0343` | `1,92 → 1,84` |
| 2 | `0,4332` | `0,5599` | `0,5022` | `0,1732 → 0,2037` | `1,911 → 1,917` |
| **4** | `0,5504` | **`0,5600`** | **`0,5549`** | `0,2511 → 0,2539` | **`1,7477 → 1,7476`** |
| 8 | `0,5832` | `0,5833` | `0,5832` | — | (o corpo nu já não se dobra) |

⇒ a `1` sub-passo o próprio corpo NU se atravessa (`0,0285`): o casamento de forma a `60 Hz` com a
linha de topo a teletransportar-se não segura a malha, com ou sem colisor. A `4` o resultado é o do
oráculo (`0,5563`, sem inchar). ⇒ o `motion.soft_body` ganhou o param **`substeps`**, lido pelo
SEQUENCIADOR (a convenção `SUBSTEPS_PARAM` do `motion.integrate` e da `sim.zone`; tecto digitável
`64`, o mesmo — o ritmo é do grafo), default `1` = o corpo de sempre ao bit.

O gate (`the_collider_holds_the_diameter_and_the_body_does_not_swell`, `stiffness 0,3`,
`substeps 4` — a fixtura em que o corpo nu SE ATRAVESSA, `0,5022 < 0,56`): colisor **`0,5600`**
(`≥ 0,5563`, o oráculo) · desenhado **`0,5414`** (`≥ 0,504`) · ligado `0,2242` contra `0,2079` no
nu · extensão `1,8059` contra `1,8026` (`+0,2 %`).

### §2.3 — A corda (o gémeo) tinha o MESMO defeito, escondido pelo gate

| `rope_thickness` (`24` nós, raio `0,5`, chicote) | par distante (fim) | 2.ª metade | comprimento (repouso `6,0`) |
|---|---|---|---|
| nua | `0,3398` | `0,0138` | `6,311` |
| colisor SEM filtro (o de antes) | `1,1800` | `0,4808` | **`8,199`** |
| colisor COM filtro (hoje) | `0,8110` | `0,0497` | `6,589` |

⭐ **O «verde» antigo era a corda ESTICADA `37 %`:** o colisor empurrava os vizinhos de corrente (a
`0,26` no repouso) até ao diâmetro, contra a restrição de distância — o mesmo inchar do corpo mole.
Com a lei a corda guarda o comprimento; o gate passa a exigi-lo (`≤ 1,10 ×` a nua) e a pose final a
`≥ 0,75·D` (medido `0,811`). ⚠️ **Resíduo NOMEADO:** durante o chicote a `60 Hz` a corda
atravessa-se (`0,0497`); com sub-passos do grafo segura `0,52`/`0,81`/`0,80` a `2`/`4`/`8` — mas a
corda não declara o ritmo do grafo (o `solver_substeps` dela é LOCAL por uma decisão medida: um
`substeps` de grafo fazia uma `sim.zone` vizinha custar `8×`), e o `motion.collide` está fora do
catálogo (o artista não monta este laço). Registado no BUGS como Bug #16.

## §3 — A cena `=129` (o smoke)

Duas gelatinas `7 × 7` desenhadas como discos do raio do colisor (`source.shape` círculo +
`motion.duplicator`), um mastro a varrer e um `force.attractor` dentro de cada uma (o campo que as
dobra); a da esquerda sem colisor, a da direita com. A receita saiu de QUATRO varreduras pela bomba
(`sonda_as_receitas`) e de três rodadas de fotos: `stiffness 0,03`, **`clusters 3`**, íman `400` a
`y = 0,5`, curso `2,0`, **`substeps 8`** ⇒ sem colisor a MEDIANA do pior par solto na 2.ª metade é
**`0,14`** (os discos encavalitam-se o tempo todo), com colisor o MÍNIMO é **`0,53`**.

**O que as fotos ensinaram (e os gates não viam):**

| receita | o gate dizia | a foto mostrava |
|---|---|---|
| `0,08`, íman `150`, `4` sub-passos, `1` moldura | verde (mínimo `0,34` contra `0,54`) | as duas gelatinas IGUAIS: o mínimo era um INSTANTE, e com UMA moldura o casamento de forma não dobra — só balança rígido |
| `0,03`, íman `400`, `8` sub-passos, `1` moldura | verde (mínimo `0,19`) | igual: a mediana era `0,29`, ao lado do passo da malha (`0,30`) — indistinguível dos vizinhos que se sobrepõem por desenho |
| **`0,03`, íman `400`, `8` sub-passos, `3` regiões** | verde (mediana `0,14`) | a laranja um amontoado de discos encavalitados; a verde dobrada com cada disco à parte |

⭐ **Os sub-passos decidem e as iterações do colisor não**: `8`/`32`/`64` varreduras dão o mesmo, e a
`4` sub-passos o mesmo íman vence o colisor (`0,47`) — o contacto precisa de TEMPO. ⇒ o gate da
cena mede a MEDIANA (o que uma foto num instante qualquer apanha), não o mínimo:
`the_left_body_crosses_and_the_right_one_holds` (pela bomba do produto, com as membranas
publicadas: `49` discos em cada saída). O «não incha» não se mede aqui (a laranja colapsa, comparar
tamanhos com ela não diz nada) — mede-se no gate do Bug #15. Fotos: `target/prova/c15/fotos/`
(`5`, `7`, `9`, `12 s` de relógio, `release`; a rota da CPU dá a mesma imagem).

## §4 — A rodada de medição (uma, em RELEASE, com a placa)

Carga `19 → 56` durante a rodada (outras linhas a compilar): cita-se o MÍNIMO de cada escada (o que
a carga não infla — `MEDIR_VELOCIDADE.md`); as RAZÕES e as paridades não dependem da carga.

| o quê | sonda | resultado |
|---|---|---|
| o corpo mole no tecto por nó (`181²`) | `measure_the_soft_body_group` (mínimo de `31` quadros) | **`0,254 ms`** (`1,5 %` de um quadro), `7,8 ns`/linha — a `512` a coluna `linhas` diz `32 761` (o corte); o tutorial 09 passa de «`512×512`, `2,26 ms`» a «`181×181`, `0,25 ms`» |
| a tabela no tecto | `measure_the_source_seam` | `32 768` linhas: **`0,04 ms`** por quadro; um FICHEIRO de `1 000 000` linhas também `0,04 ms` (entrega `32 768`) — o tutorial 08 reescrito |
| o colisor da CPU no tecto do corpo mole | `sonda_o_custo_do_colisor_da_cpu_no_tecto` | `32 761` discos, `8` varreduras: **`10 701 ms`** — `O(n²)`; o recurso é o tempo da CPU de REFERÊNCIA |
| o colisor na placa no tecto | `how_far_does_the_packing_scale` (escada parada no tecto) | `2 025` → `1,29 ms` · `8 100` → `1,32 ms` · **`32 761` → `1,56 ms`** (`9 %` de um quadro) — a grelha espacial |
| paridade CPU × placa do filtro do corpo | `the_body_filter_packs_the_same_on_both_routes` | `1` varredura `5,96e-8` · `8` varreduras `1,19e-7`; controlo: sem a identidade dos corpos a pose muda |

⚠️ **O preço que fica dito:** um corpo mole de `181²` com o `motion.collide` no laço, numa rota que
caia para a CPU, custaria `~10,7 s` por cozimento (`× substeps`). O `motion.collide` está fora do
catálogo (o artista não o põe no grafo), e a rota da placa faz o mesmo em `1,56 ms`.

## §5 — Provas de mutação (pré-voo das âncoras, alvo VERDE antes, corrida limpa no fim, `touch`)

| # | mutação | sangrou em |
|---|---|---|
| M1 | `MAX_SIDE = 512` no corpo mole | a cerca de compilação (`E0080`); sem ela, `the_mesh_never_passes…` (`262 144` contra `32 761`) e o censo |
| M2 | `MAX_ALIVE = 4·1024²` | o censo (`motion.emitter: 4 084 124`) |
| M3 | sem o piso de Oler no Poisson | `a_pathological_radius…` (`80 684` pontos) e `the_radius_floor…` (`80 996`) |
| M4 | o caminho a `2^24` | `neither_mode…` (`1 000 000`) |
| M5 | a tabela sem corte | `a_table_above_the_ceiling…` (`33 768`) |
| M6 | o filtro do corpo desligado (`segura` → `false`) | o gate unitário, a corda (`8,199` contra `6,311`) e o corpo mole (extensão `2,34` contra `1,80`) |
| M7 | o contacto FORA da velocidade (`fold_correction` devolve `vel`) | a cena `=129` (`0,5434 → 0,4092`); ⚠️ no gate do Bug #15 o efeito é `0,5414 → 0,5060`, dentro da margem — a cena é a fixtura que o mede |
| M8 | sem o param `substeps` | o gate do Bug #15 (colisor `0,4300` contra o oráculo `0,5563`) |
| M9 | a corda sem `body` | `rope_thickness` (a corda estica `8,199`) |
| M10 | o censo só com todas as portas | o controlo `vistos_corpo_mole` |

## §6 — O smoke do dono e a decisão: o `Collide` do Shape segura o corpo mole

**O report (08/10):** *«o colisor estava desligado nas duas. ao ligar não vi diferença»*. Medido
(`sonda_os_botoes_collide_que_o_dono_ve`, pela bomba, o que se DESENHA, mediana da 2.ª metade):

| `Collide` do Shape | `Collide` do Output | laranja | verde |
|---|---|---|---|
| não | não | `0,1406` | `0,5403` |
| sim | não | `0,1406` | `0,5403` |
| não | sim | `0,1406` | `0,5403` |
| sim | sim | `0,2194` | `0,5267` |

⇒ os dois botões que o artista VÊ não faziam nada sozinhos, e juntos só afastavam um pouco o
DESENHO (o passe do fim é acabamento: não realimenta a simulação — doc 115 §15.3). O que segurava a
verde era o cartão `Collide` fora do catálogo (o dono tirou-o do grafo em 17/09). ⛔ **A cena
ensinava uma coisa que o artista não consegue fazer** (`CLAUDE.md` §5.0).

**Decisão do dono (08/10):** *o botão `Collide` do Shape segura também as bolinhas de um corpo mole.*

**Desenho (o padrão-ouro da casa: o simulador LÊ o colisor das peças que simula, DENTRO do passo —
o que o `sim.step` faz; Cavalry põe o Soft Body numa FORMA):**
- D1. O `motion.soft_body` passa a ser um leitor do colisor declarado: as peças entram pela porta
  `shape` (`Grid → Duplicator(Shape) → Soft Body`), e com o `Collide` do Shape ligado a separação do
  motor da casa (`ph2d_contact`) corre em cada passo, depois do casamento de forma e antes de a
  velocidade ser lida — o contacto fica na velocidade sem coluna nenhuma, e sub-tica com o corpo.
- D2. Os pares que o REPOUSO sobrepõe não colidem: o `ph2d_contact` ganha o filtro opcional
  (`Pecas::com_ligados`), ausente ⇒ idêntico.
- D3. O corpo passa adiante a APARÊNCIA das peças (as colunas da porta `shape`), logo elas
  desenham-se sem um segundo Duplicator.
- D4. Sem declaração (o `Collide` do Shape desligado) o corpo é o de hoje ao bit.
- D5. A cena `=129` refeita assim: o mesmo par, e a diferença entre as duas é SÓ o `Collide` do
  Shape — o gesto do dono é ligar o da esquerda e vê-la segurar.

**Kill-criteria:** (1) na cena, a esquerda (desligado) atravessa-se (mediana `< 0,5·D`) e a direita
(ligado) segura (`≥ 0,9·D`), pela bomba; (2) ligar o `Collide` do Shape da ESQUERDA fá-la segurar
(o gesto do dono, medido); (3) desligado ⇒ byte-idêntico; (4) com a lei, o corpo não incha
(`≤ 5 %` contra o nu); (5) `49` peças desenhadas como o Shape; (6) o custo no tecto (`32 761`
peças) medido.

### §6.1 — Construído e medido

| kill-criterion | resultado |
|---|---|
| (1) a cena: desligado atravessa-se, ligado segura | `the_left_body_crosses_and_the_right_one_holds`: mediana `0,1043` (desligado) · mínimo `0,5535` (ligado) |
| (2) o GESTO do dono: ligar o `Collide` do Shape da esquerda | `turning_on_the_left_shapes_collide_makes_it_hold`: a esquerda passa a `0,5535` — IGUAL à direita |
| (3) desligado ⇒ o corpo de sempre | por construção (o contacto só corre com declaração; a mesma aritmética na mesma ordem) e os gates do corpo (`port_tests`, `tests`) verdes sem uma letra mudada |
| (4) a lei: não incha | `a_body_of_pieces_holds_the_diameter_and_does_not_swell`: a fixtura do Bug #15 feita de PEÇAS dá `0,5022 → 0,5600` e extensão `1,8026 → 1,8063` — **os MESMOS números da rota do `motion.collide`** (as duas portas confirmam-se) |
| (5) peças desenhadas | `49` peças em cada saída, sem segundo Duplicator |

**Um achado no caminho:** a 1.ª forma do filtro perguntava pela SOMA DOS ALCANCES, e o `Collider Shape`
do Shape nasce `Box` — uma caixa `0,28` tem alcance `0,40`, a soma `0,79` tirava do contacto pares a
`0,6` e `0,67` no repouso, e o corpo atravessava-se com o botão ligado (`0,21`). ⇒ o filtro pergunta
ao PRÓPRIO motor se os dois colisores já se SOBREPÕEM no repouso (`par::contato`), para qualquer
forma; e a cena põe o `Collider Shape` em `Circle`, o mesmo disco que se desenha.

**Mutações (pré-voo, alvo verde antes, restauro com `touch`):** M11 o filtro nunca segura → o gate do
motor (`a_pair_the_body_holds_is_left_alone`) e o do corpo de peças (incha para `2,49`); M12 o corpo
não chama o contacto → a cena (`0,0129` com o botão ligado) e o gesto do dono.

⚠️ **O que continua a ser do passe do fim e não do corpo:** `Soft Body → Duplicator(Shape) → Output`
(as formas DEPOIS do corpo) — aí o corpo não tem peças, e o `Collide` do Shape só arruma o
DESENHO com o `Collide` do Output ligado. O tutorial 09 ensina a ordem que segura: as peças ENTRAM
no corpo.

**A cena, refeita pela FOTO outra vez:** com o raio `0,28` (maior que meio passo) as vizinhas
sobrepunham-se por desenho, e a foto da verde parecia um monte de anéis (os contornos do colisor por
cima). ⇒ peças de raio `0,15` (metade do passo: TOCAM-SE no repouso) e o contorno do colisor
escondido. Medido pela bomba: desligado a mediana do pior par é **`0,075`** (contas umas sobre as
outras), ligado o mínimo é **`0,283`** contra o diâmetro `0,30`. A lei do repouso SOBREPOSTO
continua gateada no raio `0,28` (`a_body_of_pieces…`, `soft_body_radius`). Fotos `5`/`9`/`12 s`,
`release`, `target/prova/c15/fotos/`.

