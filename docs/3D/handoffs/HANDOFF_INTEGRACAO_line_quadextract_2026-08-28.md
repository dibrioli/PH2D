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

### ⭐⭐⭐ E o varrimento de densidade achou a divergência entre as DUAS portas

Ao medir onde pôr o `MAX_QUADS`, a varredura correu pela porta da `ph2d-quadchain`
(`quads_from_mesh`), que remalha o **F1 para o alvo** (`phase_zero`, a correcção de
2026-08-25). O botão **não** faz isso: ele remalha com `ph2d_remesh_iso::ALPHA` **fixo** e
deixa a extracção resolver mais fino que a malha de trabalho. As duas portas divergem, e a
divergência é enorme:

| quads pedidos | pela `quadchain` (F1 **no alvo**) | pelo **botão** (F1 em `ALPHA`) |
|---|---|---|
| `~1 800` | `χ = 2` · `0` bordo · `11,7 s` | `χ = 2` · `0` bordo · `8,6 s` |
| `~4 600` | ⚠️ `χ = 1` · **`8`** bordo · `19,1 s` | — |
| `~9 000` | ⛔ `χ = 0` · **`12`** bordo · `1` não-manifold · `39,0 s` | — |
| `~13 600` | — | ⭐ `χ = 2` · `0` bordo · `22,3 s` |
| `~18 100` | ⛔⛔ `χ = −11` · **`64`** bordo · `4` não-manifold · **`289 s`** | — |
| `~24 200` | — | ⭐ `χ = 2` · **`0`** bordo · `35,1 s` |
| `~35 500` | ⛔⛔⛔ `χ = −27` · **`162`** bordo · `3` não-manifold · **`827 s`** | — |

⭐⭐ **Uma malha de trabalho mais FINA não é mais informação: é onde a topologia se perde**, e
paga `23×` o relógio para o fazer (`827 s` contra `35 s` por densidades comparáveis). O
`quads_or_keep` já tinha a tabela que diz isto para a *entrada*; esta diz o mesmo para a
**fase zero**.

⚠️ **Isto é um LEAD para outra linha, não trabalho desta:** a `line/3DModeling` tem uma recusa
medida — *«os níveis de exportação NÃO podem mandar na densidade dos quads: o degrau `Max`
custou `27 min 29 s` para sair com `316` arestas de bordo e `6` não-manifold»* — e o caminho
dela é o `quads_or_keep`, ou seja **exactamente esta porta**. *A parede de topologia que eles
mediram pode ser a fase zero a seguir o alvo, e não a extracção.*

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

## §8-quater — ⛔⛔⛔ «FACES SOLTAS, BURACOS, e as pontas perdem detalhe» (Enio, 28/08, 2 fotos)

O segundo report do mesmo dia, depois da cura da idempotência: *«melhorou. mas ainda temos
problemas em pontas finas. faces completamente soltas, buracos. As pontas finas que deveriam
ser relativamente mais densas que as áreas lisas têm menos densidade de faces e perdem
detalhes.»*

### ⭐⭐⭐ A face solta: **o mesmo quadrado emitido DUAS vezes, um virado ao contrário**

O ficheiro que ele exportou (`23 630` quads) mede-se assim:

| régua | valor |
|---|---|
| arestas de bordo · não-manifold | `0` · `0` |
| `χ` | ⚠️ **`3`** |
| **componentes ligados por aresta** | ⛔⛔ **`2` — de `23 628` e de `2`** |
| a ilha | `[68,69,70,71]` e `[71,70,69,68]`, em `(−1,03 · 1,00 · 0,86)`, arestas `3×` a mediana |

⭐⭐ **É uma ALMOFADA: duas faces coincidentes, costas com costas, a flutuar sobre uma ponta.**
⚠️ **Nenhuma régua desta linha a via** — `χ` conta os dois lados de uma almofada e dá `2`, o
bordo é zero, o não-manifold é zero, e a contagem de quads *sobe*. *Uma almofada é uma
superfície fechada legítima; o que ela não é, é parte desta.*

⭐ **A causa é uma DOBRA do mapa:** uma região coberta duas vezes com orientações opostas dá
dois percursos de célula sobre os mesmos nós. ⇒ a extracção passa a descartar **os dois**
(`CellStats::mirrored_cells`, e o log do botão diz *«N almofada(s) descartada(s)»*), com a
chave a ser o **ciclo sem sentido** — roda para o menor nó e fica com o menor entre o anel e
o seu inverso. ⚠️ **A cura é PREVENTIVA na peça dele:** a partir do ficheiro que ele mandou a
cadeia não reproduz a dobra (`PH2D_EXTRACT_MIRROR=0` e o default dão saída idêntica), e é o
gate que a torna afirmável.

### ⛔⛔⛔ A densidade nas pontas: CONSTRUÍDA, MEDIDA e **NÃO ADOPTADA**

⭐ **Ele tem razão, e há número:** na saída dele o expoente de `aresta ∼ curvatura^n` é
**`−0,003`** sobre uma faixa de curvatura de **`9,4×`** — a grade é rigorosamente uniforme.
*Nenhuma régua desta linha media isto:* todas olhavam a aresta **global** (mediana, máxima),
que não se mexe quando a grade ignora a forma.

⭐⭐ **O substrato foi construído e fica:** o passo do mapa deixou de ser um número e passou a
ser um campo — `ph2d_gridmap::Step { h, per_vertex }`, consumido no único sítio onde o passo
entra no sistema (o gradiente alvo de cada triângulo). A extracção é **agnóstica** a isto: ela
lê isolinhas **inteiras**, e um passo que varia deforma o mapa sem mexer no que é inteiro.

⚠️ **E a 1.ª medição não distinguia «o campo é constante» de «o campo não chegou»** — as três
posições do knob davam saída **byte-idêntica**. A instrumentação disse qual: o campo saía
`0,06728..0,06728`. ⭐ **O §0.0 outra vez:** `ScaleField::adaptive_with` tem o piso em
`0,75 × aresta_média(malha)`, que é a cerca do motor **local** (a extracção por retícula dele
rasga com quads mais finos que o triângulo de entrada); emprestada à cadeia global — que mede
`24 190` quads limpos sobre `4 110` triângulos — ela **não aperta a adaptação, apaga-a**. ⇒
`ScaleField::adaptive_graded`, a mesma lei com a cerca que este consumidor de facto tem (a
razão de gradação).

Com o campo a chegar de verdade:

| `Follow Curvature` | campo entregue | expoente da SAÍDA | apertada / chapada | quads | `>60°` |
|---|---|---|---|---|---|
| `0` | — | `+0,047` | `1,167` | `13 289` | `3` |
| `0,5` | `0,0243..0,0486` (`2×`) | `+0,024` | `1,133` | `11 963` | `3` |
| `1,0` | `0,0162..0,0648` (**`4×`**) | `+0,014` | `1,090` | ⚠️ `11 302` | ⛔ `6` |

⭐⭐⭐ **Pede-se `400 %` e a saída move-se `7 %`, pagando `15 %` da contagem e o dobro das faces
péssimas.**

⚠️ **O MECANISMO:** o G3 resolve um mapa **escalar por patch** cujo gradiente se aproxima de
`direcção / h`. Com `h` constante esse alvo é **integrável**; com `h` a variar deixa de o ser
(o rotacional deixa de ser nulo), e a projecção de mínimos quadrados fica com a parte
integrável — que é, quase exactamente, o campo uniforme. *A adaptação não é ignorada: é
projectada fora.*

⭐ **A cura publicada tem nome e é outra maquinaria:** o factor de escala tem de ser **conforme
por construção** — resolver `Δ log h` contra a curvatura de Gauss e usar `h = h₀·e^{−s}`, que é
integrável por definição (a família *«integer-grid maps with prescribed sizing»*). É uma wave
com espec própria.

⇒ **O `Follow Curvature` continua a nascer em `0` e o caminho de omissão é byte-idêntico.**

## §8-quinquies — ⛔⛔⛔ «O REMESH AMPUTOU PONTAS» (Enio, 29/08, 6 fotos) — e é um CICLO

O report mais claro da série: a foto do **antes** mostra uma bola com espinhos longos e
finos; a do **depois** mostra dois deles **rasgados e ocos**. E o diagnóstico dele:
*«o algoritmo não entende que em áreas mais finas é necessário mais faces»*.

### ⭐⭐⭐ A REPRODUÇÃO, e ela dá o endereço de um estouro que o repo perdeu

Correndo o **botão** sobre o `.obj` que ele exportou:

| fase | `χ` | bordo | não-manifold | componentes |
|---|---|---|---|---|
| entrada dele | `2` | `0` | `0` | `1` |
| ⛔ **depois da fase zero** | **`6`** | `0` | **`1`** | `1` |

⛔⛔ E a jusante: `panicked at crates/ph2d-gridmap/src/assembly.rs:193:34: index out of
bounds: the len is 28 but the index is 33`. ⭐ **É o estouro que o `CLAUDE.md` §5 diz estar
«SEM ENDEREÇO desde 26/08»** — a `line/3DModeling` procurava-o em `solve.rs:336`, que já não
existe. *Ele mora no `assembly.rs`, e o gatilho é uma malha de trabalho não-manifold.*

### ⭐⭐⭐ A causa é uma MORDIDA que se REALIMENTA

⚠️ **A peça dele já entra danificada, e o dano é nosso.** A saída que ele exportou tem
**`19` vértices de valência `2`**, todos em pontas finas, e **os `19` são doublets
clássicos**: um vértice preso entre **duas** faces que partilham três cantos.

⛔ **O ciclo:** a extracção emite doublets nas pontas → o artista exporta / continua a
esculpir → carrega outra vez → a **fase zero**, que só sabe remalhar superfície, não sabe o
que fazer com um vértice de duas arestas e **rasga a topologia** (`χ = 2 → 6`) → o solver
estoura ou a ponta sai amputada. *Cada volta piora a anterior.*

⚠️ **E as fixturas sintéticas NÃO reproduzem.** Varreu-se uma bola de espinhos com
`σ = 0,30 … 0,05` (raio de ponta a descer até bem abaixo da aresta alvo) e **todas** saem
`χ = 2`, zero não-manifold. ⇒ *não é a espessura sozinha que parte a fase zero: é a
espessura MAIS a mordida que já lá estava.* A fixtura ficou na sonda
(`spiked_ball`), e a lição é que **só a peça do artista continha o fenómeno**.

### As três curas, e o que cada uma vale medido

| | |
|---|---|
| ⭐ a extracção **não emite** doublets (`dissolve_doublets`, `CellStats::doublets`) | fecha o lado da produção |
| ⭐ o botão **repara** os que a peça já traz (`ph2d_quadextract::repair_doublets`) | fecha o lado do consumo — ⛔ *sem ele toda peça já gravada partiria o botão para sempre* |
| ⭐⭐ uma tentativa que **estoura** perde, em vez de derrubar tudo (`catch_unwind`) | o artista **não perde a escultura** porque a retopologia falhou |

⭐ **A dissolução é exacta e não move um vértice:** as duas faces partilham três cantos, logo
fundem-se numa — `V−1`, `E−2`, `F−1`, **`χ` invariante**. ⚠️ A **ordem** sai do percurso da
fronteira (`a → q → b → p`), e trocá-la daria um quad que se auto-intersecta — há gate, com
mutação. ⛔ E uma **almofada** (`p == q`) não é um doublet: ela descarta-se, noutro sítio.

Medido na peça dele, antes e depois das três:

| | `Detail 0,50` | `Detail 0,90` |
|---|---|---|
| ⛔ antes | **estoura**; `16` bordo; `non_quads` a estourar o `usize` | **estoura**; `16` bordo |
| ⭐ depois | sem estouro · **`4`** bordo · `0` não-quads | sem estouro · **`8`** bordo · `0` não-quads |

⚠️ **E fica ABERTO o que ele nomeou:** os espinhos ainda rasgam (`4`–`8` arestas de bordo, no
espinho a `r ≈ 1,15`). A cura de fundo é a **fase zero preservar a topologia que recebe** —
uma remalha isotrópica que não belisca uma agulha mais fina que a sua aresta alvo — e isso é
uma wave em `ph2d-remesh-iso`, com esta reprodução como gate de partida.

⚠️ **Um contador que descreve a fase errada:** a 1.ª versão da dissolução deixou o `st.quads`
a contar as células **antes** da fusão, e a jusante `faces − quads` deu `usize` negativo — o
log imprimiu `18446744073709551613`. *Corrigido na mesma wave, e nomeado aqui porque a
subtracção sem sinal é o modo de falha que não avisa.*

## §8-sexies — ⭐⭐⭐ AS TRÊS MALHAS DO ARTISTA, e a amputação tem UM dono

Em 2026-08-29 ele mandou, pela primeira vez, **a entrada**: `sculpt_antes.obj`, mais a nossa
saída (`sculpt_Depois.obj`) e a de uma ferramenta de terceiros do Blender
(`Sculpt_Blender.obj`, *QRemeshify*). ⭐ *Ler a SAÍDA de uma ferramenta é lícito e é o que
esta linha já faz com o oráculo; o código dela não se abre sem triagem de licença.*

| | faces | `χ` · bordo · n-manif. | irregulares | aresta mediana | ⭐ **alcance** |
|---|---|---|---|---|---|
| entrada | `13 824` | `2` · `0` · `0` | `2` | `0,0327` | **`2,355`** |
| ⭐ nossa | `15 426` | `2` · `0` · ⚠️ `1` | `48` | `0,0319` | ⚠️ **`1,963`** (`−16,6 %`) |
| QRemeshify | ⭐ `8 291` | `2` · `0` · `0` | `86` | `0,0462` | **`2,045`** (`−13,2 %`) |

⚠️ **O alcance quase empata — e não é aí que a diferença se vê.** Ela está na **secção** do
espinho, medida em bandas ao longo do eixo do mais longo:

| `t/L` | | raio local | faces na banda | aresta |
|---|---|---|---|---|
| `0,60` | entrada | `0,193` | `5` | `0,188` |
| | nossa | `0,123` | `43` | `0,033` |
| | ⭐ QRemeshify | `0,178` | `40` | `0,034` |
| `0,72` | entrada | `0,138` | `4` | `0,255` |
| | ⛔ **nossa** | ⛔ **`0,091`** (`−34 %`) | `9` | `0,060` |
| | ⭐ **QRemeshify** | ⭐ **`0,144`** | ⭐ **`46`** | `0,028` |
| `0,84` | ⛔ nossa | — (já não há espinho) | — | — |
| | ⭐ QRemeshify | `0,123` | `17` | `0,028` |

⭐⭐⭐ **Nós AFINAMOS a agulha; eles mantêm-na e gastam faces nela.** A `0,72` do comprimento
eles põem `46` faces contra as nossas `9`, com a aresta a `0,028` contra `0,060`. E a
densidade deles **segue a forma**: aresta na ponta / no corpo é **`0,64×`** neles e `0,90×`
em nós (expoente `aresta ∼ curvatura`: `−0,120` contra `−0,069`). *Com metade das faces no
total.*

### ⭐⭐⭐ E a amputação tem UM dono: a FASE ZERO

| peça | alcance entrada → fase zero |
|---|---|
| `espinhos σ = 0,30` | `2,022 → 1,989` (`−1,6 %`) |
| `espinhos σ = 0,10` | `2,021 → 1,903` (`−5,8 %`) |
| `espinhos σ = 0,07` | `2,021 → 1,760` (`−12,9 %`) |
| `espinhos σ = 0,05` | `2,020 → 1,701` (`−15,8 %`) |
| ⛔ **`sculpt_antes` do artista** | **`2,355 → 1,981`** (⛔ **`−15,9 %`**) |

⭐⭐ **A nossa saída final alcança `1,963`** — ou seja **a cadeia inteira a jusante perde mais
`0,018`, e a remalha isotrópica perde `0,374`.** *A amputação acontece antes de o campo
cruzado existir.*

⚠️ **E a fixtura sintética SEMPRE conteve o fenómeno** — o que faltava era a régua. As
mesmas peças que saíam `χ = 2` limpas (§8-quinquies) perdem `13`–`16 %` de alcance quando a
agulha afina. *Não foi a fixtura que falhou: foi medir topologia onde o defeito é métrico.*

### ⏳ A wave seguinte, já especificada

⛔ **A cura é em `ph2d-remesh-iso`, e a hipótese tem forma:** a relaxação tangencial com
reprojecção encolhe um tubo cujo **raio local** é comparável à aresta alvo — uma projecção
ao ponto mais próximo, numa agulha mais fina que o espaçamento, pode aterrar **do outro
lado**. ⇒ a régua da wave é o **alcance** e o **raio local por banda** (as duas acima), a
fixtura é `espinhos:6` com `σ = 0,05`–`0,07`, e a barra é o que a ferramenta de terceiros
mostra ser possível: raio local preservado a `0,72` do comprimento.

⭐ **E isto CORRIGE a nota do §8-quater:** lá escreveu-se que a densidade adaptativa foi
«medida e não adoptada» porque a projecção a lava. Isso continua verdade **do mecanismo que
se tentou** — e a saída do QRemeshify prova que o **objectivo** é alcançável (`0,64×` de
aresta na ponta, com metade das faces). *«Inalcançável» era uma afirmação sobre a nossa
tentativa, não sobre o problema* (`CLAUDE.md` §0.0).

## §8-septies — ⛔⛔⛔ A CURA DA AMPUTAÇÃO: achada, medida, e NÃO ADOPTADA

O §8-sexies deixou a wave especificada: *a reprojecção da fase zero encolhe um tubo cujo raio
local é comparável à aresta alvo — uma projecção ao ponto mais próximo, numa agulha, pode
aterrar do outro lado.* ⭐ **A hipótese estava certa e a cura já existia nesta árvore:**
`ph2d_remesh_iso::project_facing` recusa um pé cuja normal de face **discorda** da direcção
dada — e o `relax_and_project` chamava-a com `None`. *Uma capacidade construída e não ligada
é uma capacidade que não existe.*

### ⭐ Ela cura a fase que ataca

| peça | alcance perdido na fase zero, sem | com |
|---|---|---|
| `espinhos σ = 0,30` · `0,20` · `0,14` | `−1,6 %` … | **idêntico** (inerte sem agulha) |
| `espinhos σ = 0,10` | `−5,8 %` | `−6,3 %` |
| ⭐ `espinhos σ = 0,07` | `−12,9 %` | ⭐ **`−7,9 %`** |
| ⚠️ `espinhos σ = 0,05` | `−15,8 %` | ⚠️ `−18,0 %` |
| ⭐⭐ **`sculpt_antes` do artista** | ⛔ `−15,9 %` | ⭐⭐ **`−5,7 %`** |

⭐ Na peça dele passa a perder **menos que a ferramenta de terceiros** (`−5,7 %` contra os
`−13,2 %` do QRemeshify).

### ⛔⛔ E parte a fase seguinte

Medida de ponta a ponta **pelo botão**, mesma peça, `Detail 0,85`:

| | alcance final | `χ` | bordo | ilhas | dobras | `>60°` | relógio |
|---|---|---|---|---|---|---|---|
| ⭐ desligada (o que shipa) | `−12,4 %` | `1` | **`4`** | `1` | `76` | `2` | **`31 s`** |
| ⛔ ligada | ⛔ `−14,2 %` | ⛔ **`−16`** | ⛔ **`250`** | ⛔ **`5`** | ⛔ `798` | ⛔ `41` | ⛔ `79 s` |

⚠️ **O mecanismo do estrago:** guardar o vértice do seu lado guarda a agulha **e deixa lá uma
malha emaranhada** — a de trabalho passa de `3 982` para `9 458` faces com valência até `23`
(contra `8`). O campo cruzado e o traçado, que dependem de uma triangulação bem comportada,
perdem-se nela. ⛔ **E o alcance FINAL até piora:** a ponta guardada não sobrevive à cadeia.

⭐⭐⭐ **A lei que esta wave paga:** *uma fase medida sozinha pode melhorar e piorar o produto.*
A régua do §8-sexies (o alcance depois da fase zero) é honesta e **insuficiente** — ela mede a
fase, não a travessia. ⇒ toda cura de fase zero passa a ser medida **pelo botão**, não pela
fase.

⇒ A função fica, **desligada**, com a tabela ao lado e um gate sobre a decisão
(`a_reprojeccao_que_respeita_a_normal_nasce_desligada`, provado por mutação).
`PH2D_ISO_FACING=1` liga-a. **A cura verdadeira tem de tratar as duas fases ao mesmo tempo:
guardar a agulha e entregar ao campo uma malha que ele saiba ler.**

## §8-octies — ⭐⭐⭐ A AGULHA SOBREVIVE À FASE ZERO — e a cadeia não sobrevive à agulha

O §8-septies deixou a lei: *guardar a ponta e desenhar a grade são um trabalho só*. Esta wave
levou-a até ao fim e a resposta é a mesma, um nível mais fundo.

### A causa, escrita como número

A fase zero remalha para um alvo **uniforme** (`ALPHA × diagonal = 0,089` na peça do artista) e
o **raio local** de um espinho dele cai a `0,037`. ⛔ **O passe de COLAPSO come toda aresta
abaixo de `0,071`, e as arestas que dão a volta ao tubo são justamente essas** — a agulha
fecha-se sobre si antes de o campo cruzado existir. *Uma agulha mais fina que um triângulo não
tem onde ser representada.*

### A cura: o limiar deixa de ser um número e passa a ser um CAMPO

Duas portas irmãs em `ph2d-mesh`, **append-only** (`sizing = None` é byte-idêntico ao que
existia, e o pincel não muda de assinatura):

* `collapse_in_sphere_sized` · `refine_in_sphere_sized`, com `Sizing = Option<&dyn Fn([f32;3]) -> f32>`
* ⚠️ **por POSIÇÃO e não por índice de vértice**, porque estas portas **renumeram** (`Remap`) e
  correm várias passagens por chamada: *uma tabela que o próprio laço invalida é pior que
  nenhuma.*
* ⚠️ **o limiar é o do MEIO da aresta** — os dois extremos podem cair em bandas diferentes, e
  escolher um deles faria a decisão depender de qual canto a face propôs primeiro.

E em `ph2d-remesh-iso` a `SizingGrid`: alvo local `= alvo × clamp(mediana(|κ|)/|κ|, 1/4, 1)`,
numa grelha de célula `= alvo`, consultada pelo **mínimo dos 27 vizinhos** (⚠️ *um degrau no
limiar de colapso é uma fileira de arestas que morre de um lado e vive do outro*). ⭐ **O tecto
é `1`: ela nunca grosseira**, então não pode piorar região nenhuma que o laço já resolveu.

### ⭐⭐⭐ Ela CURA a fase, e o número é dramático

Alcance perdido pela fase zero, fixtura de espinhos:

| `σ` | sem | com |
|---|---|---|
| `0,30` | `+0,1 %` | `+0,3 %` |
| `0,20` | `−0,9 %` | ⭐ `+0,8 %` |
| `0,14` | `−1,6 %` | ⭐ `+0,8 %` |
| `0,10` | `−5,8 %` | ⭐ **`−0,8 %`** |
| `0,07` | `−12,9 %` | ⭐ **`−1,3 %`** |
| `0,05` | ⛔ `−15,8 %` | ⭐⭐⭐ **`−0,8 %`** |

⭐ E a malha de trabalho fica **perfeita**: `χ = 2`, zero bordo, zero não-manifold, valência
máxima `10`.

### ⛔⛔⛔ E PARTE A CADEIA — a segunda confirmação da mesma lei

Pelo **botão**, peça do artista, `Detail 0,85`:

| | alcance final | `χ` | bordo | não-manif. | relógio |
|---|---|---|---|---|---|
| ⭐ desligada (o que shipa) | `−12,4 %` | `1` | **`4`** | `0` | **`27,8 s`** |
| ⛔ ligada | ⛔ `−17,3 %` | ⛔ `−7` | ⛔ **`62`** | ⛔ `2` | ⛔ **`167 s`** |

⚠️ **O mesmo mecanismo das duas vezes:** a malha de trabalho passa de `3 982` para `33 156`
faces e **deixa de ser isotrópica**; o campo, o traçado e o mapa perdem-se nela, e o alcance
FINAL até piora.

⚠️ **E na peça dele a fase zero só recupera `0,9` ponto** (`−15,9 % → −15,0 %`) contra os `15`
pontos das agulhas sintéticas: *o que sobra ali é o ÁPICE* — um cone sculptado termina num
ponto, e nenhuma densidade finita o representa. **São dois defeitos, e só um é o colapso.**

### ⏳ O que fica escrito para a wave seguinte

⭐⭐⭐ **A lei, agora confirmada DUAS vezes** (`facing_on` e `adaptive_on`): *uma fase medida
sozinha pode melhorar e piorar o produto.* ⇒ **a cadeia inteira tem de ser consciente do
sizing** — é a mesma conclusão que o §8-quater tirou pelo outro lado (o passo do mapa), e as
duas apontam para a mesma obra: o **factor de escala conforme por construção**
(`Δ log h` contra a curvatura de Gauss, `h = h₀·e^{−s}`), que é integrável e portanto sobrevive
tanto à parametrização quanto à remalha.

⭐ **E o segundo defeito tem nome próprio:** preservar o **ápice** é *feature-preserving
remeshing* (pinar o vértice de canto), não densidade — outra wave, outra régua.

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

⭐ **E a re-corrida depois do §8-quater** (a almofada e o campo de passo):

| | |
|---|---|
| `cargo test -p ph2d-quadextract -p ph2d-quadflow -p ph2d-quadchain -p ph2d-gridmap` | ⭐ verde (`60` + `32` + as suites, `0` falhas) |
| `cargo test -p ph2d-host-desktop --bins retopo` · `--bins densidade` | ⭐ verde (`7` · `1`) |
| `cargo clippy` nas cinco crates `--all-targets` | ⭐ limpo (`0` avisos) |
| `scripts/cleanroom-sweep.sh` sobre todo o diff | ⭐ limpo (vassoura de 56 entradas) |
| **prova de mutação** — a chave do ciclo deixa de olhar o sentido inverso | ⭐ MORREU |
| **prova de mutação** — a renormalização da contagem sai | ⭐ MORREU |

⭐ **E a re-corrida depois do §8-quinquies** (a mordida, a reparação e a rede):

| | |
|---|---|
| `cargo test -p ph2d-quadextract -p ph2d-quadflow -p ph2d-quadchain` | ⭐ verde (`0` falhas) |
| `cargo test -p ph2d-host-desktop --bins retopo` | ⭐ verde (`7`) |
| `cargo clippy` nas cinco crates `--all-targets` | ⭐ limpo (`0`) |
| `scripts/cleanroom-sweep.sh` sobre todo o diff | ⭐ limpo (56 entradas) |
| **prova de mutação** — a ordem da fusão troca `p` com `q` | ⭐ MORREU |
| **prova de mutação** — a recusa da almofada sai | ⭐ MORREU |
| **prova de mutação** — a compactação dos órfãos sai | ⭐ MORREU |
| **fim-a-fim** — a peça do artista | ⭐ sem estouro; bordo `16 → 4` (`d 0,50`) e `16 → 8` (`d 0,90`) |

⚠️ **Dois gates da própria crate reprovaram primeiro, e o motivo é a lição:** `χ` saiu `14`
contra `2` e `13` contra `1` — **doze órfãos, doze unidades**. *A superfície estava certa e o
ARQUIVO não*, porque `V − E + F` conta todos os vértices e o vértice preso ficava lá.

## §8-nonies — ⛔⛔ O PORTÃO QUE ESTAS TRÊS WAVES NUNCA ALCANÇARAM, e o roteiro que mentia

### ⛔ O vermelho: **duas** crates acima do tecto de LOC, e nenhuma corrida desta linha o viu

`workspace_src_files_under_loc_cap` (em `ph2d-editor-core/tests/`) reprovava com

| ficheiro | LOC | tecto |
|---|---|---|
| `ph2d-remesh-iso/src/lib.rs` | `875` | `700` |
| `ph2d-quadextract/src/cells.rs` | `758` | `700` |

⚠️⚠️ **É a 5.ª vez que esta linha paga a MESMA forma** ([memória](../../../project-memory/feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate.md)):
todo portão de fecho destas waves correu **com filtro de nome** (`-p ph2d-quadextract`,
`--bins retopo`), e um gate que **VARRE A ÁRVORE** vive noutra crate — nenhum filtro por
pacote o alcança, por construção. *Um `-p` responde «a minha crate está verde?»; ele nunca
responde «a árvore está verde?».*

⚠️ E o `cargo fmt` da árvore expôs mais dois ficheiros meus por formatar
(`sculpt3d_history_retopo_extract{,_tests}.rs`), commitados assim numa wave anterior — pelo
mesmo motivo: `cargo fmt -p <crate> -- --check` nunca entrou em nenhum dos portões.

### ⭐ A cura: dois SPLITS por responsabilidade (⛔ nunca uma entrada no allowlist)

**`ph2d-remesh-iso`** (`875 → 573`) parte em dois irmãos, e a fronteira é uma pergunta cada:

| módulo | a pergunta que ele responde |
|---|---|
| `lib.rs` | *que aresta se divide, que aresta colapsa?* |
| `sizing.rs` | *qual é o alvo **AQUI**?* — a `SizingGrid` **e** as duas portas medidas-e-recusadas (`adaptive_on`, `facing_on`), cada uma com a sua tabela |
| `project.rs` | *onde é que este vértice **pousa**?* — `project_onto` / `project_facing` / o triângulo mais próximo |

⚠️ **O `dot` FICA no `lib.rs`** e o `project.rs` importa-o: ele tem um segundo consumidor
fora da projecção (`relax_and_project`), e movê-lo teria sido arrumação a fingir de desenho.

**`ph2d-quadextract`** (`cells.rs` `758 → 524`) ganha `doublets.rs`, e a fronteira também é
uma pergunta cada: `cells.rs` responde *«que células fecham?»*, `doublets.rs` responde *«que
vértice não devia existir?»*. ⭐ As duas cruzam-se **num sítio só** — o `build` chama
`dissolve_doublets` e `compact_verts` — e é essa a razão de o corte ser barato.

⚠️ **A porta pública muda de dono:** `pub use doublets::repair_doublets` (era `cells::`). A
assinatura não se mexe, então nenhum chamador nota.

### ⭐ E o ROTEIRO da cena `=35` descrevia o defeito ERRADO

O passo (4) de `announce_extract` dizia que a ponta *«chega a fechar-se num ponto»* por o
motor não ver o vinco. ⛔ **Isso era a leitura de 22/08 e o §8-septies/§8-octies refutaram-na:**
a ponta era **costurada fechada na fase zero**, e isso está curado; o que sobra é o **ápice**,
que é outro defeito (nenhuma malha finita representa um ponto) — *e o roteiro mandava o Enio
olhar para a causa que já não existe.*

Três mudanças, todas dentro do gate que já existia:

- **(0) `Ctrl+Shift+O`** para trazer a peça dele. ⚠️ O roteiro nunca nomeou o gesto, e
  **arrastar não funciona nesta máquina** (o Wayland não entrega `DroppedFile` — medido em
  `sculpt3d_import.rs`). *O smoke pedia uma peça e não dizia como a pôr lá dentro.*
- **(4)** os **dois** defeitos separados, com o curado marcado e o que sobra nomeado.
- **(8)** clicar **duas vezes seguidas** e ver a contagem ficar parada — a régua do §8-ter,
  que era a única das três curas sem passo de smoke.

## §9-bis — Portão de fecho da re-corrida (§8-nonies)

| | |
|---|---|
| `architecture_workspace_file_loc_cap` + `architecture_widget_loc_cap` | ⭐ verde (era **vermelho** com 2 ficheiros) |
| `cargo fmt` na árvore | ⭐ limpo (corrigiu 3 ficheiros desta linha) |
| `cargo test -p ph2d-remesh-iso -p ph2d-quadextract -p ph2d-mesh -p ph2d-quadfill` | ⭐ verde |
| `cargo test -p ph2d-host-desktop --bins retopo` · `--bins roteiro` | ⭐ verde |
| `cargo clippy --all-targets` nas duas crates + shell | ⭐ `0` avisos |
| `scripts/cleanroom-sweep.sh` sobre os 6 ficheiros | ⭐ limpo (56 entradas) |
| binário `--release` reconstruído | ⭐ pronto a correr |
