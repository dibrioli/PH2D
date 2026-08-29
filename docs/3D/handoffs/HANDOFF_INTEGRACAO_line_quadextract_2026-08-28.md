# HANDOFF DE INTEGRAÇÃO — `line/quadextract` (2026-08-28): **a régua estava errada, e o que faltava era o ACABAMENTO**

> **Leia primeiro:** [`ACHADO_o_acabamento_e_a_regua_da_densidade.md`](../quad-remesh/ACHADO_o_acabamento_e_a_regua_da_densidade.md)
> — é o documento de conteúdo, com as tabelas e as recusas medidas. Este traz o que o
> **integrador** precisa.

## §1 — O que esta jornada descobriu, em três frases

1. ⛔⛔⛔ **A barra do oráculo (`4,8°`–`7,1°` de enviesamento) estava a ser lida a 1/9 da
   densidade dele.** A nossa medição corria com `370`–`576` quads e a saída dele tem
   `3 352`–`4 696`. À densidade dele, a mesma cadeia **sem uma linha mudada** dá
   `3,8°`–`6,5°` — dentro da barra desde 2026-08-25. ⇒ *a semana das amarras dos arcos
   perseguiu um buraco da régua.*
2. ⭐⭐ **O que sobra é o passe de ACABAMENTO dele.** O oráculo grava duas saídas por peça
   (crua e `_smooth`); a nossa saída crua **bate a crua dele** em três peças, e o `_smooth`
   compra-lhe `−0,3°` a `−1,5°` de mediana e `−8°` a `−11°` de `p99`. O nosso acabamento
   eram `6` rondas de Laplaciano herdadas da montagem por patches, **nunca re-medidas** para
   a extracção.
3. ⭐⭐⭐ **A cadeia passa a ter um acabamento próprio, numa porta só**
   (`ph2d_quadfill::finish_extracted`): Laplaciano como **ronda zero**, depois **ajuste de
   quadrado alinhado ao relevo**, e a saída é a **melhor ronda**, não a última.

## §2 — O que mudou no produto

| onde | o quê |
|---|---|
| `crates/ph2d-quadfill/src/finish_extract.rs` | **NOVO** — a porta, as quatro constantes medidas e a comparação de Pareto |
| `crates/ph2d-quadfill/src/relax.rs` | `square_relax{,_capped,_aligned}` públicos · `steer` (o alinhamento) · cerca de viagem · raio de reprojecção que encolhe · saída por assentamento |
| `crates/ph2d-quadfill/src/quality.rs` | `Hint` + `surface_hint` — a direcção que a superfície prefere, por face da saída |
| `crates/ph2d-quadchain/src/lib.rs` | passa a **acabar** (entregava a malha crua) · `ChainTiming::finish` · `ChainReport::finish` |
| ⚠️ `shells/desktop/src/sculpt3d_history_retopo_extract.rs` | o botão chama a porta em vez do Laplaciano cru |
| `crates/ph2d-quadextract/examples/{chain_info,piece_report}.rs` | os instrumentos: `PH2D_RELAX_SCAN=1` varre **através da porta**; `PH2D_REF=<peça>.obj` mede relevo e fidelidade contra a escultura |

⚠️ **Tudo aditivo.** Nenhuma assinatura pública existente mudou de forma; `ChainTiming` e
`ChainReport` ganharam campos (os dois são `#[derive(Default)]`/construídos por nome aqui).

⛔ **O caminho do `ph2d_quadfill::fill` (a montagem por patches) fica INTACTO** — a tabela de
rejeição dele (`SQUARE_ROUNDS = 0`) foi medida noutra conectividade e continua a valer lá.

## §3 — As constantes, e de onde saiu cada número

| constante | valor | de onde |
|---|---|---|
| `EXTRACT_RELIEF_PULL` | **`1,0`** | ⭐ *o peso É a confiança* — a anisotropia crua, sem constante por cima. Numa esfera ela é `0` e a lei degenera **ao bit** no quadrado puro |
| `EXTRACT_SETTLE` | **`1e-3`** da aresta mediana | tabela medida **através da porta**; `3e-4` custa `1,5`–`3×` mais para `0,2`–`0,3°` |
| `EXTRACT_PATIENCE` | **`768`** rondas **sem aceitar nada** | `1,8×` a maior primeira aceitação medida (`418`) |
| `EXTRACT_MAX_ROUNDS` | `1 200` | a rede |
| `quality::HINT_SMOOTH_ROUNDS` | **`0`** | ⛔ construída, medida e **não adoptada** — ver §4.4 |

## §4 — ⚠️ As SETE coisas que uma leitura rápida do diff entende ao contrário

1. **A relaxação por ajuste de quadrado NÃO é nova** — existe desde 2026-08-22, com
   `SQUARE_ROUNDS = 0` **medida e rejeitada**. O que mudou foi a **conectividade** a que ela
   se aplica: a tabela da rejeição mediu a montagem por patches (`27°` de mediana, defeito na
   conectividade); a extracção entrega `1,10 / 3,8°`. *Uma recusa medida responde uma
   pergunta.*
2. **O Laplaciano NÃO saiu** — ele é a ronda zero, e é ele que mata a face extrema (`>60°` de
   `7` para `1` na `sculpt_hooked` fina). As duas leis atacam metades diferentes.
3. **A cerca de VIAGEM existe na API e nasce DESLIGADA** (`square_relax_capped`) — medida e
   rejeitada como cura: a `0,35 h` guarda o relevo e paga o `p99` (`52,8°` contra `34,5°`).
4. **A aceitação é contra a RONDA ZERO, e cobre CINCO colunas** (`>60`, enviesamento `p50` e
   `p99`, aspecto `p50` e `p99`). ⛔ A 1.ª redacção comparava com a **melhor até então** e era
   uma **catraca**: a relaxação mergulha antes de subir, e as quatro peças da densidade fina
   saíam intocadas. A escolha **entre** aceitáveis é a mediana, com o aspecto a desempatar.
5. ⛔ **A paciência conta rondas SEM ACEITAR NADA, não «desde a melhor».** Na
   `sculpt_hooked` fina a primeira aceitação é a `312` e a melhor é a `830`: com `128` rondas
   *desde a melhor* a peça saía **intocada** em vez de ir a `1,04 / 2,0° / p99 22,8`.
6. ⭐ **Há uma queda para a lei CEGA**, e ela só corre quando a alinhada **não se mexeu** — é
   isso que guarda o relevo onde ele estava em jogo (`5` das `8` células ficam com a
   alinhada). ⚠️ Nas outras três o preço é o relevo, e está medido.
7. **O raio de reprojecção encolher não é aproximação** — depois da 1.ª ronda o vértice está
   *sobre* a superfície, e uma esfera de `2×` o que ele andou contém o pé mais próximo. Vale
   `~12×` de relógio.

## §5 — ⭐⭐⭐ O RESULTADO, e o que fica ABERTO

À densidade do oráculo (`alvo 0,667`), contra a saída **`_smooth`** dele:

| peça | nós ANTES | ⭐ **nós DEPOIS** | oráculo `_smooth` |
|---|---|---|---|
| `sphere_uv` | `1,10 / 3,8° / 17,3°` | **`1,04 / 2,6° / 10,1°`** | `1,22 / 5,9° / 20,0°` |
| `sculpt_eared` | `1,10 / 6,3° / 27,2°` | **`1,04 / 3,3° / 11,0°`** | `1,08 / 5,7° / 20,2°` |
| `sculpt_hooked` | `1,11 / 6,5° / 33,0°` (`>60` 1) | **`1,04 / 2,0° / 22,8°`** (`>60` **0**) | `1,19 / 5,8° / 48,1°` (`>60` 4) |
| `sculpt_wrinkled` | `1,12 / 5,2° / 35,5°` | **`1,07 / 2,8° / 22,8°`** | `1,08 / 4,8° / **17,0°**` |

⇒ **batemos a saída alisada dele em TODAS as colunas de forma em três das quatro peças**, e
em duas de três na quarta (ele fica com a cauda da enrugada).

### ⛔ Aberto, com o número ao lado

- **O RELEVO** é a coluna em que ficamos atrás: `11,6°` contra `7,0°` na enrugada e `19,3°`
  contra `13,3°` no gancho (empatados na orelha). ⚠️ *Já estávamos atrás antes desta
  jornada* (`11,8°` e `17,7°`) — a queda para a lei cega paga mais um pouco em três células.
- ⛔ **A hipótese do «campo de direções ruidoso» está REFUTADA** como cura da recusa da lei
  alinhada: a suavização 4-RoSy foi construída, medida e não adoptada (§4.4 do ACHADO §10.4).
- **Preço:** `0,2`–`0,6 s` na densidade do botão, `3`–`12 s` na fina, e o botão corre a
  cadeia **duas** vezes. `PH2D_EXTRACT_FINISH=0` desliga.

## §6 — O que o Enio smoka

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract && env PH2D_SCULPT3D_SMOKE=35 cargo run -p ph2d-host-desktop --release
```

Depois: **`Quad Retopology`** no painel de escultura. `PH2D_EXTRACT_FINISH=0` volta ao
acabamento antigo (o Laplaciano cru), para comparar lado a lado.

## §7 — ⭐⭐ O veto pagava o acabamento para deitar a malha fora

⛔ Com o acabamento dentro de `quads_from_mesh`, uma peça **dura** passou a pagá-lo inteiro
para ser deitada fora: no cubo subdividido (o caso em que a cadeia perde **por medição**) a
saída abre arestas de bordo, o veto recusa, e o acabamento tinha corrido até ao tecto **duas
vezes**. ⚠️ **O sintoma foi um TESTE que passou de segundos a minutos**, não uma medição de
perf — *um teste que fica lento é uma medição de custo que ninguém pediu*.

⭐ A cura é a **ordem**: `quads_from_mesh` parte-se em `quads_from_mesh_raw` + o acabamento, e
o veto de **topologia** decide com a malha crua (uma relaxação move vértices e mais nada). A
propriedade tem gate (`the_finishing_cannot_change_the_edge_census`). Suite do `quadchain`
depois da cura: **10,06 s**.

⏳ **Aberto, deliberadamente:** dentro do `quads_or_keep_from` a superfície do acabamento é o
`feed` e não o `keep` — escolhido assim para a reordenação ser **provadamente neutra**.

## §8 — Gates novos (todos provados por mutação)

`crates/ph2d-quadfill/src/finish_extract_tests.rs` (**8**), `relax_tests.rs` (**+6**) e
`crates/ph2d-quadchain/tests/veto.rs` (**+1**).
**15 mutações, 15 mortas** — entre elas três que a 1.ª redacção dos gates deixava viver
(*a aceitação ignora o aspecto* · *a paciência conta do início* · *a lei cega entra sempre*).
⚠️ E um gate desta jornada era uma **tautologia** apanhada por mutação: ele media a rotação
com uma função que devolve `[0°, 45°]` **por construção**, logo não podia falhar.

## §8-bis — ⛔⛔⛔ «MUITO DEMORADO» (Enio, no smoke) — e o botão ficou **5,4×** mais rápido

O smoke validou a qualidade (*«melhor que o plugin padrão ouro do Blender»*) e reprovou o
relógio. ⭐ **A primeira coisa a fazer com um veredito de produto é transformá-lo num
número** — `cargo run --release -p ph2d-quadchain --example chain_time -- <peça>.obj`, que
corre a **mesma função que o botão**. Detalhe e tabelas: `ACHADO…` §14–§16.

| o desperdício | o que era | a cura | ganho |
|---|---|---|---|
| ⛔ `Mesh::rebuild()` **a cada uma das 726 rondas** do acabamento — reconstrói adjacência, curvatura e **octree** da saída, que uma relaxação não muda e não lê | `11,5 s` de `17,7 s` (`65 %`) | o laço corre sobre **buffers** e a `Mesh` é publicada **uma vez**, no fim (a porta única não é violada), e a ronda é **paralela e determinística** | **7,6×**, saída **idêntica** |
| ⛔ o G3 gastava **`8 000` rondas sem saída nenhuma**, e a partir da ~`2 750` o movimento é ruído de `f32` que não desce mais | `45 %`–`58 %` da cadeia | `WELD_PATIENCE = 256` rondas sem descer — *a mesma lei que o acabamento já tinha pago* | até **3,3×**, topologia **idêntica** |
| ⛔ as **duas tentativas** do botão em **série**, tratadas como uma «troca de qualidade por espera» | `2×` a cadeia inteira | `rayon::join` — elas são **independentes** | **1,87×**, saída **idêntica** |

⇒ **o botão na `sculpt_eared`: `~35 s` → `~6,5 s`.** ⭐⭐ E **nenhum** dos três trocou
qualidade por relógio: os três eram desperdício.

⚠️ **Duas armadilhas que a jornada pagou aqui:** (1) a 1.ª versão do laço paralelo somava
normais de Newell próprias e **o resultado mudou** (`726` rondas → `724`) — *uma relaxação que
muda de resultado ao ser optimizada não foi optimizada, foi substituída*; (2) a 1.ª mutação
do gate de determinismo **sobreviveu** por não ser observável (mexia no raio de reprojecção,
que é só um piso de busca) — *uma mutação que não muda o resultado não testa nada*.

## §8-ter — ⛔⛔⛔ «PIORA SEVERA, pontas com baixa resolução e furos» (Enio, 28/08, com foto)

⭐⭐⭐ **A primeira coisa que a medição disse é que NÃO foi a volta da performance.** As três
mudanças do §8-bis foram revertidas uma a uma pela porta (`PH2D_G3_PATIENCE=99999` ·
`PH2D_RETOPO_SERIAL=1` · `PH2D_EXTRACT_FINISH=0`, e as três juntas) sobre a peça que ele
mandou: **as cinco configurações devolvem a MESMA malha, ao vértice.** ⇒ *o defeito é
anterior a elas, e vinha a ser fotografado como se fosse novo.*

### O defeito: **o botão não é IDEMPOTENTE**

O alvo saía de `ph2d_quadflow::edge_for_detail_with`, cujo **piso** é
`0,75 × aresta_média(malha_da_cena)`. Isso é honesto na primeira passagem — *não se resolve
mais fino do que a entrada tesselou* — e é uma armadilha na segunda: **depois de uma
retopologia a malha da cena É a saída**, então o piso sobe com ela e o mesmo ponto do slider
passa a pedir quads cada vez maiores.

Medido na `sculpt_t002` do artista (`19 786` quads), `Detail` **parado** em `0,50`:

| aperto | alvo | quads | contra a entrada |
|---|---|---|---|
| entrada | — | `19 786` | — |
| 1.º | `0,0891` | `1 747` | `−91 %` |
| 2.º | `0,1614` | `520` | `−97 %` |
| 3.º | `0,2161` | ⛔ **`281`** | ⛔ **`−98,6 %`** |

⚠️ **A `281` uma ponta tem duas ou três faces** — é literalmente *«pontas com baixa
resolução»*. *Um slider parado que devolve três densidades é um slider que não tem
significado.*

### A cura: a faixa passa a ser **CONTADA**, e a âncora é a **ÁREA**

`ph2d_quadflow::edge_for_detail_by_count` — `MIN_QUADS` (`100`, que já era o teto grosso e já
era absoluto) até **`MAX_QUADS = 25 000`**, geométrico, e o lado do quad é `√(área/contagem)`.
⭐ **A área é da superfície, não da tesselação**, então re-perguntar sobre a própria saída pede
a mesma coisa. É também o que as três referências fazem (ZRemesher: *Target Polygons Count* ·
QuadriFlow: *Number of Faces* · Instant Meshes: alvo de vértices) — **nenhuma** deriva a faixa
da malha que tem na mão.

Medido, os mesmos três apertos depois da cura:

| aperto | alvo | quads | forma |
|---|---|---|---|
| 1.º | `0,0973` | `1 377` | — |
| 2.º | `0,0965` | `1 413` | — |
| 3.º | `0,0957` | **`1 494`** | aspecto `1,06` · enviesamento **`2,8°`** · `>60` **`0`** · `16` irregulares |

⭐ **A deriva que fica (`+8,5 %` em três apertos) é a ÁREA a crescer** — o acabamento alisa a
peça e a área sobe —, e o gate mede exactamente isso: a barra é `|√(área₁/área₀) − 1|`, ⛔ **não**
um número escolhido. *A 1.ª redacção usou `½·Δárea`, a aproximação de 1.ª ordem, e reprovou por
`0,02` ponto percentual — uma barra aproximada mede o erro da aproximação.*

⚠️ **O `MAX_QUADS` tem DOIS recursos, e o segundo não é o relógio:** a `24 190` quads a cadeia
sai limpa (`χ = 2`, zero bordo) em `35 s`; acima disso a `line/3DModeling` mediu a escada
inteira e o degrau `Max` saiu com `316` arestas de bordo e `6` não-manifold depois de
`27 min 29 s`. ⇒ *o tecto fica no ponto mais fino medido LIMPO, e quem o quiser subir mede a
topologia, não o relógio.*

⚠️ **O motor `Fast` recebe a mesma contagem e depois a CERCA dele** (`resolvable_edge_range`):
a extracção por retícula dele rasga com quads mais finos que o triângulo de entrada (a foto de
2026-08-19). *Quando a cerca morde, aquele motor deixa de ser idempotente — e é a cerca dele
que o diz, não o slider.*

### O segundo defeito: **«furo» só contava METADE**

O ficheiro que o artista exportou tem `19 786` quads impecáveis (valência máxima `5`, `21`
irregulares, área de face `p99/p50 = 1,8`) e **`2` arestas não-manifold num ponto só**, com três
vértices de valência `2`–`3`. Em qualquer visualizador isso é o mesmo entalhe escuro que um
buraco.

⛔ E a chave da frente de `worse` — a que escolhe entre as tentativas — contava **só as arestas
de bordo**. Uma aresta não-manifold passava invisível, e o campo alinhado produz exactamente
isso (medido antes: `sculpt_hooked`, `1` não-manifold contra `0` do liso, com o alinhado a
ganhar por `0,2°`). ⇒ `open_edges = bordo + não-manifold`, e é ele que decide **e** que arma a
3.ª tentativa.

⚠️ **Isto é preventivo na peça dele:** hoje as duas tentativas dão `0` não-manifold, então a
cura não é observável ali. O gate é que a torna afirmável — as duas fixturas **empatam** no
bordo e diferem só na aresta não-manifold, e a peça suja leva enviesamento **perfeito** contra
uma limpa horrível: *sob a lei antiga a asserção é falsa.*

### ⏳ O que fica ABERTO, nomeado

⛔ **`Follow Curvature` não tem consumidor no motor de omissão** (`let _ = adaptive;`): a
densidade é uniforme sobre a superfície, então uma ponta fina recebe quads do mesmo tamanho que
a barriga. O painel **declara** isso (`RetopoMode::uses_adaptive`), mas *densidade adaptativa é
a feature que responde a «pontas com baixa resolução» pelo lado da forma*, e ela não existe
nesta cadeia — o campo de escala teria de variar e o G3 de quantizar em cima disso.

⚠️ **O motor `Fast` do menu devolve, na peça dele, `437` quads + `150` faces que não são quads,
`188` de `514` vértices irregulares e valência até `9`** — contra `1 494` quads, `100 %` quads e
`16` irregulares do de omissão. *Ele está a um clique do botão, com o nome que um artista
alcança depois de ouvir que o bom é lento.* A decisão de o retirar do painel é do dono do
produto (ele já disse em 25/08 que *«o antigo não apresenta resultados úteis»*).

## §9 — Portão de fecho

| | |
|---|---|
| `cargo test -p ph2d-quadfill` | ⭐ verde (21 + 16 nas suites de integração, 0 falhas) |
| `cargo test -p ph2d-quadchain` | ⭐ verde (5) |
| `cargo test -p ph2d-gridmap` | ⭐ verde (60) |
| `cargo test -p ph2d-host-desktop --bins retopo` | ⭐ verde (5) |
| `cargo test -p ph2d-remesh-iso` | ⭐ verde (9) |
| `cargo clippy --all-targets` nas três + no shell | ⭐ limpo |
| `scripts/cleanroom-sweep.sh` sobre todo o diff | ⭐ limpo (vassoura de 56 entradas) |
| `scripts/doc-index.sh --check` | ⭐ 14 índices em dia (+ o de `docs/3D/handoffs/`, à mão, com a contagem **derivada do `ls`**) |

⭐ **E a re-corrida depois do §8-ter** (a idempotência e o `open_edges`):

| | |
|---|---|
| `cargo test -p ph2d-quadflow -p ph2d-quadchain -p ph2d-quadfill` | ⭐ verde (`32` + `5` + `23` + as suites de integração, `0` falhas) |
| `cargo test -p ph2d-host-desktop --bins retopo` | ⭐ verde (`6`) |
| `cargo clippy -p ph2d-quadflow -p ph2d-host-desktop --all-targets` | ⭐ limpo |
| `scripts/cleanroom-sweep.sh` sobre todo o diff | ⭐ limpo (vassoura de 56 entradas) |
| **prova de mutação** — `worse` volta a contar só o bordo | ⭐ MORREU |
| **prova de mutação** — o alvo volta a sair da aresta média da malha | ⭐ MORREU |
| **fim-a-fim** — três apertos na peça do artista | ⭐ `1 377 → 1 413 → 1 494` (era `1 747 → 520 → 281`) |
