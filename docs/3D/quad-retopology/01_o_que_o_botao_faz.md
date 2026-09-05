# 01 — O que o botão `Quad Retopology` faz, fase a fase

> **Este doc é o MAPA.** Ele diz *o que cada fase decide* e *qual porta a desliga* — não a
> história de como cada uma nasceu (essa está no
> [`PLANO_a_graduacao_da_ponta.md`](../quad-remesh/PLANO_a_graduacao_da_ponta.md), §1–§109) nem
> o estado vivo (esse é o `CLAUDE.md` §5).

## §1 — A cadeia, em uma tabela

| # | fase | crate | o que ela DECIDE | porta de bissecção |
|---|---|---|---|---|
| 0 | **o alvo** | `ph2d-quadflow` | o lado do quad, a partir de uma **contagem** ancorada na ÁREA da peça (`MIN_QUADS = 100` … `MAX_QUADS = 25 000`) | o slider `Detail` |
| 1 | **fase zero** (F1) | `ph2d-remesh-iso` | a malha de TRABALHO: triângulos isotrópicos a `ALPHA = 0,02 × diagonal`, **graduados** pela curvatura e com uma **calota** por espinho afiado | `PH2D_ISO_ADAPT=0` · `PH2D_TIP_CAP=0` |
| 2 | **campo cruzado** (F2) | `ph2d-crossfield` | para onde as linhas da grade apontam (`ALIGN_WEIGHT = 0,03` de alinhamento ao relevo) | `PH2D_TIP_ALIGN=<k>` (instrumento) |
| 3 | **traçado** (F3) | `ph2d-trace` | os patches, os lados e os arcos — a topologia grosseira da grade | — |
| 4 | **quantização** (F4) | `ph2d-quantize` | quantos quads cada arco leva (Bi-MDF, com ótimo demonstrado) | — |
| 5 | **mapa de grade inteira** (G3/G5) | `ph2d-gridmap` | as coordenadas `(u, v)` de cada vértice, com as **costuras por eliminação de variável** e o arredondamento misto-inteiro | `PH2D_GRIDMAP_WELD=0` |
| 6 | **extracção** | `ph2d-quadextract` | a malha de quads, das isolinhas inteiras do mapa | `PH2D_RETOPO_EXTRACT=0` (volta ao motor de patch) · `PH2D_EXTRACT_MIRROR=0` |
| 7 | **acabamento** | `ph2d-quadfill` | onde cada vértice pousa, e as quatro reparações | `PH2D_EXTRACT_FINISH=0` · `PH2D_TIP_SNAP=0` |
| 8 | **o selector** | shell | qual das `2`–`9` tentativas é entregue | `PH2D_RETOPO_TIPKEY=0` |

⚠️ **O motor de omissão é o `Even Grid` (global).** O `Fast` do dropdown é o motor **local**
(`ph2d-quadflow`, o porte do Instant Meshes) e devolve, na peça do dono, `437` quads com `150`
não-quads contra `1 494` e `100 %` — ⛔ ele está a um clique, com o nome que um artista alcança
depois de ouvir que o bom é lento.

## §2 — O que o acabamento faz, na ordem em que faz

O acabamento ([`ph2d_quadfill::finish_extracted_travel`]) tem **três saídas** (a lei alinhada, a
cega, e a queda), e **as quatro reparações correm nas três** — há gate a contá-las
(`as_tres_saidas_do_acabamento_desfazem_o_avesso`), porque *uma cura que vive em duas das três
saídas é uma cura que o produto às vezes não corre*.

1. **Ronda zero: o Laplaciano** — a malha extraída pousa na escultura.
2. **Ajuste de quadrado ALINHADO AO RELEVO** — o tamanho vem dos quatro pontos e a orientação
   roda para a direcção principal, com peso = a anisotropia crua (numa esfera ela é `0` e a lei
   degenera **ao bit** no quadrado puro). A saída é a **melhor ronda**, aceite contra a ronda
   zero nas três colunas da barra.
3. **Desembaraço** ([`untangle_bowties`]) — as faces do avesso: gravatas (todas) e dobras **em
   grupo de `≥ 2`**.
4. **Apagador de abas** ([`untangle::remove_flaps`]) — o grupo acusado cresce um anel, o disco
   sai, e o buraco fecha com um leque de `L/2` quads.
5. **Remate das pontas** ([`snap_tips`]) — o vértice mais próximo de cada bico afiado encosta no
   ápice da escultura, sob três cercas (ver [`03_as_curas.md`](03_as_curas.md) §7).

## §3 — A cascata, e por que ela existe

O botão não corre a cadeia uma vez: ele corre **tentativas** com knobs diferentes (peso de
alinhamento `0` ou `0,03`; linhas de feição ligadas ou não; densidade adaptativa; a cerca de
viagem do acabamento livre ou apertada) e **mede cada uma**. A escolha é do
[`super::decide::worse`], cujas chaves estão nesta ordem — e a ordem **é** a feature:

| # | chave | por quê está aqui |
|---|---|---|
| 1 | **furos** (bordo + não-manifold) | o dono nomeou três vezes; um furo não se compra com beleza |
| 2 | **ilhas** (componentes ligados) | a *almofada* — o mesmo quad emitido duas vezes — não move `χ`, nem bordo, nem não-manifold |
| 3 | **avesso** (gravatas + dobras em grupo, folga `INSIDE_OUT_SLACK = 20`) | a folga vive no vazio entre os dois vereditos do dono: `125` = *«destruiu a malha»*, `6`–`8` = *«melhor resultado até agora»* |
| 4 | **amputação** (`cut`, depois `over`, depois `p90`) | o bico a mais de meia célula da saída; *duas pontas partidas de raspão são pior que uma partida a fundo* |
| 5 | **grade na ponta** (`over`) | o report de 01/09 com foto: *«a área fica a meio caminho e a ponta fica menos densa»* |
| 6 | **faces `> 60°`** | a beleza que o oráculo mede |
| 7 | **`ENTREGA`** (razão ponta/corpo) | a chave contínua da densidade |
| 8 | **enviesamento mediano** | a última — e a única coluna que o dono nunca nomeou |

⚠️ **Reordenar as chaves está FORA** — a ordem foi medida em 30/08 sobre um report dele. Quando
uma candidata boa perde, a lei é **produzir** a candidata que tem as duas coisas, nunca
reordenar o desempate.

## §4 — O que o slider `Detail` faz, e o que ele NÃO faz

Ele escolhe uma **contagem de quads** entre `MIN_QUADS` e `MAX_QUADS`, ancorada na área da peça.
⛔ Ele **não** é um tamanho: ancorá-lo na aresta média da malha da cena tornava o botão
**não-idempotente** — carregar duas vezes dava `19 786 → 1 747 → 520 → 281` quads (`−98,6 %`),
que é literalmente *«pontas com baixa resolução»*.

⚠️ **O `MAX_QUADS` tem DOIS recursos**, e os dois estão medidos: o relógio (`35 s` a `24 190`
quads) e a **topologia**, que rebenta acima disso.

## §5 — O `Follow Curvature`, e por que ele nasce no MÁXIMO

Ele diz **quanto a densidade segue a curvatura**. Desde 2026-09-04 nasce em `1`: com `0`, dois
dos cinco espinhos da escultura do dono saem com **`3` a `5` faces dando a volta** (contra
`17`–`58`), e voltam `2` pontas amputadas e `4` com a grade grossa. A tabela dos três pontos do
curso, nas duas peças, está em [`03_as_curas.md`](03_as_curas.md) §8.
