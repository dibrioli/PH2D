# 27 — O ESTADO DA ARTE DE ONDE A TINTA MORA, e o que ele diz das cinco waves de atlas

> **Ordem do dono (2026-09-20):** *«vc deve buscar o estado da arte, se for necessário pesquisar
> como fazem os que fazem o melhor»* — depois de eu lhe ter proposto, como wave seguinte, aproveitar
> a grade que a retopologia já produz ([26 §13.4](26_a_parametrizacao_como_atlas.md)).
>
> ⚠️ **Este documento é PESQUISA, não implementação. Nenhuma linha de produto muda por causa dele.**
> O molde é o [24](24_pesquisa_o_estado_da_arte_do_alinhamento.md): triagem de licença primeiro,
> famílias da literatura, medição no nosso corpus, proposta em waves, e o que fica para o dono.

---

## §1 — A pergunta tem DUAS metades, e eu só tinha respondido bem a primeira

| a pergunta | onde ela foi respondida | veredito |
|---|---|---|
| **onde o PINCEL CORRE** | [25 §11](25_avaliacao_o_painter_na_malha.md) | ⭐ respondida: no ECRÃ, porque ali a vizinhança é sempre uma imagem verdadeira |
| **onde a TINTA MORA** | [25 §11](25_avaliacao_o_painter_na_malha.md), tabela das três vias | ⛔ **incompleta — faltavam DUAS famílias inteiras**, e uma delas é a continuação do que já shipámos |

A tabela daquele §11 oferecia **atlas · Ptex · por-vértice**. A literatura de 2010 em diante tem mais
duas, e a segunda muda a recomendação:

* **Htex** (per-halfedge, 2022) — o Ptex corrigido para topologia arbitrária;
* ⭐⭐⭐ **Mesh Colors** (Yuksel, Keyser, House — TOG 2010; *Mesh Color Textures*, HPG 2017) — que é,
  à letra, **cor por-vértice com amostras também nas ARESTAS e no INTERIOR das faces**.

---

## §2 — Triagem de licença (§0.9, passo 1) — e ela não encontra parede nenhuma

⭐⭐ **Cinco portas ABERTAS, e uma já está instalada nesta máquina.**

| artefacto | licença | verificada como | uso |
|---|---|---|---|
| **Ptex 2.5.4** | **BSD-3-Clause** | `pacman -Qo /usr/include/Ptexture.h` | ⭐ **instalado** — lê-se, porta-se e liga-se, com atribuição |
| **Htex** (`wbrbr/htex`) | **MIT** | ficheiro de licença do repositório | ⭐ lê-se e porta-se |
| **xatlas** (`jpcy/xatlas`) | **MIT** | idem | ⭐ o atlas-padrão da indústria aberta (D-Charts + LSCM + arrumação) |
| **Boundary First Flattening** | **MIT** | página do projecto | ⭐ tem porta de consola (`bff-command-line`) ⇒ serve de **oráculo que se corre** |
| **cyCodeBase** (Yuksel) | **MIT** | página de código | ⚠️ **não traz mesh colors** — só malha, cor, BVH, GL |
| Mesh Colors / Mesh Color Textures | **papers** | — | ⭐ clean-room de paper é o método desta casa ([ADR-0167](../architecture/decisions/0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)) |
| Ministry of Flat · RizomUV · Substance Painter · 3D-Coat | **comercial fechado** | — | ⛔ nem oráculo: não há entrada nossa que se lhes dê por consola |
| Blender Texture Paint | GPL | arsenal §2 | **parede** — oráculo que se **corre**, nunca fonte que se lê |

⛔ **Nenhuma linha de fonte restrito foi lida para esta pesquisa.** O que foi lido: o cabeçalho
BSD-3 instalado (`/usr/include/Ptexture.h`), papers públicos, e a nossa própria árvore. A única
excepção a declarar está na [§12](#12--registo-de-contaminação-e-é-meu).

---

## §3 — As famílias, e o eixo que as separa

O relatório de referência é o **STAR de Eurographics 2019, *Rethinking Texture Mapping*** (Yuksel,
Lefebvre, Tarini) — o levantamento que existe exactamente por causa desta pergunta.

| família | onde a tinta mora | costuras | vizinhança para um pincel |
|---|---|---|---|
| **A. Atlas UV** | um quadrado | **sim**, nas ilhas | ⚠️ certa dentro da ilha, ERRADA na costura |
| **B. Per-face (Ptex)** | uma mini-textura por face | fronteira por face, dados **duplicados** | precisa de adjacência + **5** buscas |
| **B'. Per-halfedge (Htex)** | uma por aresta | idem, **3** buscas | idem, e **aceita triângulos e n-gons** |
| ⭐ **C. Mesh Colors** | amostras em **vértices + arestas + interiores**, as de fronteira **PARTILHADAS** | ⭐ **nenhuma** | ⭐ uma **retícula de superfície**, regular menos nos vértices extraordinários |
| **D. Volumétrica** (octree, brickmap, hash) | em 3D | nenhuma | certa, mas o custo é de volume |
| **E. Por-vértice** (o que shipa hoje) | vértices | nenhuma | correcta, e a resolução é a da malha |

⭐⭐⭐ **E a linha que reordena tudo: `E` é o caso `R = 1` de `C`.** As amostras de mesh colors ficam
em posições baricêntricas `i/R, j/R, k/R`; com `R = 1` sobram exactamente os três cantos — *cor
por-vértice*. **Não há salto de família entre o que shipámos a 19/09 e o estado da arte: há um
número.**

### ⛔ O que o cabeçalho BSD instalado diz, e nenhuma página web dizia

Lido em `/usr/include/Ptexture.h` (Ptex 2.5.4, nesta máquina):

* `struct Res { int8_t ulog2, vlog2 }` ⇒ **a resolução é POR FACE e por eixo, em potências de dois**;
* `DataType` × `numChannels` ⇒ ⭐ **multi-canal por desenho** — responde à pergunta aberta do
  [25 §11](25_avaliacao_o_painter_na_malha.md) (*«serve de destino a quatro canais?»*): **serve**;
* ⛔⛔ `enum EdgeId { ... }` leva escrito **`Edge ID usage for triangle meshes is TBD`** —
  *a adjacência de Ptex para malhas de TRIÂNGULOS não está especificada*, e a nossa malha de
  escultura é de triângulos. É precisamente o buraco que o Htex existe para tapar (o paper dele
  abre com *«existing implementations are restricted to quad-only meshes»*);
* `PtexWriter::writeFace(faceid, info, data)` ⇒ ⚠️ **o escritor produz um FICHEIRO, face inteira de
  cada vez.** Ptex é um formato de armazenamento, não uma estrutura de pintura interactiva.

---

## §4 — ⭐⭐⭐ A MEDIÇÃO que decide, no nosso corpus

⭐ **O instrumento é versionado** — [`docs/3D/ferramentas/onde_a_tinta_mora.py`](ferramentas/onde_a_tinta_mora.py),
e corre-se sobre qualquer `.obj`/`.obj.gz` do corpus:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && \
  python3 docs/3D/ferramentas/onde_a_tinta_mora.py \
  crates/ph2d-quadfill/tests/fixtures/pontas/Sculpt_Blender.obj.gz
```

**A régua:** amostras por unidade de MUNDO — e o que conta é **o pior sítio**, porque foi o pior
sítio que a [W5](26_a_parametrizacao_como_atlas.md#12) mediu e é dele que o artista se queixa
(*«o pincel mudou de tamanho»*). Peça: a saída do **BOTÃO** sobre o `_base_sculpt` (`1 425` quads,
`2 850` arestas, área `16,90`). Memória a `16 B` por amostra (cor + altura + cobertura + material,
a conta do [25 §7](25_avaliacao_o_painter_na_malha.md)).

| esquema | amostras | MB | dens. p50 | **dens. PIOR** | costuras |
|---|---|---|---|---|---|
| por-vértice, na malha desta peça | `0,001 M` | `0,0` | `9` | `9` | nenhuma |
| por-vértice, no default do módulo | `0,10 M` | `1,6` | `76` | `76` | nenhuma |
| ⭐ **por-vértice, depois de `K,K`** | `1,57 M` | **`25,2`** | `305` | **`305`** | **nenhuma** |
| **atlas `2048²` — O NOSSO, medido** | `4,19 M` | `67,1` | `304` | ⛔ **`113`** | ilhas |
| atlas `2048²` — SOTA, ⚠️ **estimado** | `4,19 M` | `67,1` | `431` | `302` | ilhas |
| per-face (Ptex/Htex), mesmo orçamento | `5,02 M` | `80,3` | `545` | `385` | fronteira por face |
| ⭐ **mesh colors**, mesmo orçamento | `4,86 M` | `77,7` | `536` | **`379`** | ⭐ **NENHUMA** |

> ⚠️ **A linha «SOTA, estimado» NÃO é uma medição desta casa** — ela supõe `75 %` de tinta no
> quadrado e um pior sítio a `0,70×` da mediana, que é o que a literatura reporta para um bom
> empacotador com energia isométrica. As nossas duas (`37,3 %` e `0,37×`) são **medidas**
> ([26 §12.5](26_a_parametrizacao_como_atlas.md), [§13.3](26_a_parametrizacao_como_atlas.md)).
> *Ela está na tabela para não deixar a conclusão depender do nosso atlas ser fraco.*

### ⛔⛔⛔ E a leitura mais desconfortável é a terceira linha

**A pintura por-vértice que já shipa, na malha que o dono já tem depois de carregar `K` duas vezes,
é `2,7×` mais fina no pior sítio do que o atlas `2048²` que estas cinco waves construíram** — e
custa `2,7×` menos memória, e não tem uma única costura.

⚠️ Ela é **pior na mediana** contra um atlas SOTA (`305` contra `431`), e **igual** contra o nosso.
*O que a tabela diz não é «o atlas é mau»: é que a coluna em que o atlas perde é exactamente a que
o dono julga.*

---

## §5 — ⭐⭐⭐⭐ O que NENHUM parametrizador remove, e é um teorema

O espalhamento de densidade de um atlas não é um defeito do nosso `G3`. É o **Teorema Egregium de
Gauss**: *uma superfície com curvatura não-nula não tem achatamento isométrico.* Achatar um pedaço
curvo obriga a distorcer ângulo ou área, e a área **é** a densidade de texels.

| esquema | o que ele achata | distorção de área |
|---|---|---|
| atlas | um pedaço **CURVO** da superfície | ⛔ **inevitável** — só o valor muda com o solver |
| per-face / mesh colors | **uma face plana de cada vez** | ⭐ **zero** — um mapa afim num triângulo preserva a razão de áreas |

⇒ a dispersão passa de *«um teorema, e medimos `3,2×`»* para *«um arredondamento»*: com `R` por face
escolhido pela área e quantizado a potências de dois, o pior caso é **`√2 = 1,41×`**, e quem escolhe
o arredondamento é quem escreve o código.

⛔⛔ **E isto re-lê a [W5](26_a_parametrizacao_como_atlas.md#12) desta sessão.** O
`TECTO_DA_IGUALACAO = 3,0` foi varrido em duas peças, o *«sem tecto»* saiu **dominado**, e o
resíduo ficou nomeado como aberto. Com o teorema ao lado, a leitura completa é: *aquela wave estava
a limitar uma grandeza que nenhum parametrizador leva a `1`.* A lei que ela shipou continua certa e
continua a ser a melhor coisa a fazer **dentro da família do atlas**.

---

## §6 — ⭐⭐⭐ O achado: o estado da arte já está na árvore, DUAS vezes

Como no [doc 24](24_pesquisa_o_estado_da_arte_do_alinhamento.md), a pesquisa acabou dentro de casa.

### (a) O LSCM está na árvore, e foi recusado para a OUTRA pergunta

[`ph2d_quadfill::lscm`](../../crates/ph2d-quadfill/src/lscm.rs) é clean-room de **Lévy, Petitjean,
Ray, Maillot — *Least Squares Conformal Maps for Automatic Texture Atlas Generation*, SIGGRAPH
2002**, com a energia deduzida de raiz, passo de Gauss–Seidel em forma fechada, e a tabela de
convergência no cabeçalho. É **o mesmo algoritmo que o xatlas usa**.

⛔ Ele shipa **`LSCM_MAP = false`**, e a recusa está medida — *«um mapa praticamente conforme dá o
PIOR resultado dos três»*. ⚠️ **Mas a pergunta a que ela responde é a QUADRATURA DOS QUADS**, e o
mecanismo escrito ao lado dela di-lo: o que o mapa conforme piora é a **subdivisão do arco** do
preenchimento por leque. *Uma recusa medida responde UMA pergunta* (§0.0) — e o título do paper que
ele implementa é, à letra, a outra.

⇒ **se a família do atlas for a escolhida, o parametrizador de cartas já existe nesta árvore** e o
que falta é a segmentação em cartas e a ligação, não a energia.

### (b) A cor por-vértice de 19/09 é o primeiro degrau do mesh colors

`Paint` · `Blur` · `Smear Color` ([§99–§102](handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md))
escrevem num canal por vértice, o refino **interpola** a cor dos pais
(`the_new_vertices_carry_colour_and_mask`), e o `Flat` mostra a tinta sem sombra.
**Isso é mesh colors com `R = 1`.**

⚠️ **E há uma dívida já nomeada que é exactamente a mesma peça:** o report de 20/09 (*«manchas
pretas ao pintar com topologia dinâmica»*) aponta para o **colapso não permutar o plano de cor**. A
resposta que mesh colors dá a isso está no paper e é a mesma que o §27 desta linha já pagou para as
posições: *cada amostra tem uma posição 3D bem definida, logo re-amostrar é trivial.*

---

## §7 — ⚠️ O que isto CORRIGE nos docs 25 e 26

| onde | o que estava escrito | a correcção |
|---|---|---|
| [25 §11](25_avaliacao_o_painter_na_malha.md), tabela das vias | três destinos | ⛔ **faltavam Htex e Mesh Colors**, e o segundo é a continuação do que shipámos |
| [25 §11](25_avaliacao_o_painter_na_malha.md) | *«⭐ o ECRÃ: SEMPRE. É uma imagem verdadeira»* como a **única** vizinhança correcta | ⚠️ certo sobre uma **imagem**; falso como *única* — mesh colors dá uma **retícula de superfície** correcta em todo lado menos nos vértices extraordinários, onde é **irregular e não errada** |
| [25 §5](25_avaliacao_o_painter_na_malha.md) | *«PTex / mapas por-face ⛔ obriga a reescrever a indexação inteira do Painter»* | ⚠️ verdade para Ptex/Htex (que **duplicam** a fronteira e precisam de rotação de adjacência); **falso** para mesh colors, que a **partilha** |
| [25 §5](25_avaliacao_o_painter_na_malha.md) | *«7 dos 13 param na costura»* | ⇒ numa retícula de superfície **param 2 ou 3**, e a análise está na [§8](#8--e-quantos-dos-13-modos-precisam-mesmo-de-um-array-plano) |
| [25 §11](25_avaliacao_o_painter_na_malha.md) | *«o Ptex serve de destino a quatro canais?»* — em aberto | ✅ **serve** (`DataType` × `numChannels`, lido no cabeçalho instalado) |
| [25 §11](25_avaliacao_o_painter_na_malha.md) | *«quantos texels por face o Ptex pede para igualar `2048²`?»* — em aberto | ✅ **medido**: lado por face `p05 38 · p50 53 · p95 71` texels; o total arredondado à potência de dois mais perto é `1,20×` o quadrado |

---

## §8 — E quantos dos 13 modos precisam MESMO de um array plano?

⚠️ **Isto é ANÁLISE do mecanismo, não medição** — nenhuma linha destas foi corrida.

| modo | o que ele lê | numa retícula de superfície |
|---|---|---|
| `Paint` · `Clone` · `Mask` · `Sculpt` · `Selection` | por amostra | ⭐ de graça |
| `Smear` · `Knife` · `Blur` | a vizinhança | ⭐ **funciona** — é um passeio na retícula, e a fronteira é partilhada |
| `Fill` | região conexa | ⭐ **funciona** — é uma inundação num grafo |
| `Deform` (liquify) | `out[dst] = sample(dst − D(dst))` | ⚠️ **lei nova** — amostrar em qualquer ponto é trivial; o campo `D` deixa de ser 2D |
| `WetPaint` | simulação de fluido densa, passes por LINHA | ⛔ **precisa de array regular** |
| `Inpaint` | PatchMatch — janelas quadradas | ⛔ **precisa de array regular** |

⇒ **2 modos exigem um array plano; 1 pede lei nova; 10 atravessam.** É a razão pela qual a
recomendação abaixo **não escolhe** entre o ecrã e a superfície: ela usa os dois, cada um onde ele é
a resposta certa.

---

## §9 — A proposta, em waves, com o preço de cada uma

**Recomendação: família C (mesh colors), subindo o `R` que já temos em `1`.**

| wave | o que entrega | porque esta ordem | julgável pelo dono? |
|---|---|---|---|
| **P1 — o `R` sobe** | amostras nas ARESTAS e no interior (`R = 3`) | ⭐ `9×` as amostras e **`3×` a densidade linear** sem tocar na malha — e a lei do pincel não muda. ⛔ **O que muda é o SHADER:** hoje ele interpola entre três vértices, e passa a ter de achar as amostras à volta de `(face, baricêntricas)`. *É esse o preço real da P1, e o mip é a parte difícil* ([Mesh Color Textures, HPG 2017](https://www.cemyuksel.com/research/meshcolors/mesh_color_textures.pdf) existe por causa dele) | ⭐ sim: pinta-se e vê-se |
| **P2 — `R` por FACE** | resolução escolhida pela área da face | ⭐ é aqui que a densidade fica uniforme **por construção** (§5) | sim: a marca deixa de mudar de tamanho |
| **P3 — a retícula atravessa** | `Blur` · `Smear` · `Fill` a passear pela fronteira partilhada | ⭐ os três já existem por-vértice; o que muda é o **vizinho** | sim |
| **P4 — o ECRÃ para os dois que faltam** | `WetPaint` e `Inpaint` correm num buffer 2D e aterram na retícula | ⛔ **só depois de P1–P3**: sem destino, um buffer de ecrã não tem onde pousar | sim |
| **P5 — a SAÍDA** | assar mesh colors numa textura UV ao exportar | ⭐ **é aqui que as cinco waves de atlas se pagam** — e o consumidor é qualquer motor | sim: exporta e abre noutro app |

⭐⭐ **A W1–W5 desta sessão NÃO é trabalho perdido — ela muda de papel.** O atlas deixa de ser *onde
a tinta mora* e passa a ser *como a tinta sai daqui para outro programa*, que é o que o
[25 §8](25_avaliacao_o_painter_na_malha.md) já dizia ter valor sozinho (*«exportar um `.obj` com UV
serve qualquer motor»*). ⚠️ E nesse papel as duas colunas fracas pesam menos: um assado tolera
costuras (há dilatação) e tolera dispersão (a peça já está pintada).

---

## §10 — ⛔ O que NÃO fazer, com o motivo

1. ⛔ **Não portar o Ptex como destino.** Ele é BSD e está instalado, e mesmo assim: a adjacência
   dele para **triângulos** está marcada `TBD` no cabeçalho que shipa, o escritor é de **ficheiro**
   e não de pintura, e ele **duplica** a fronteira que mesh colors partilha. *Ele é uma resposta
   para onde GRAVAR, nunca para onde PINTAR* — o que o [25 §11](25_avaliacao_o_painter_na_malha.md)
   já tinha escrito e eu quase esqueci ao ver que estava instalado.
2. ⛔ **Não trocar o `G3` por um solver isométrico (SLIM / simétrico-Dirichlet) para curar a
   dispersão.** Compra a coluna errada: o teorema da §5 diz que o chão não é `1`, e o custo é um
   solver esparso novo. *Se a família do atlas ficar, a cura barata continua a ser a da W5.*
3. ⛔ **Não construir o desdobramento por partes / neural** (PartUV SIGGRAPH Asia 2025, ArtUV,
   DreamUV, Nuvo). Eles resolvem *«dar UV bonito a uma malha que caiu do céu»*; nós temos a malha, a
   topologia e o campo. E os dois de optimização que a bancada do PartUV mede **custam minutos por
   peça**.
4. ⛔ **Não ir buscar geração neural de textura** (Paint3D, MV2UV, TexPainter, FlexPainter). É outro
   produto: elas **inventam** a textura; o Painter é para a **mão** do artista.
5. ⚠️ **Não prometer que mesh colors resolve o report das manchas pretas.** Ele aponta para o
   colapso não permutar o plano de cor — isso é um defeito de HOJE e cura-se hoje, com ou sem `R`.

---

## §11 — O que fica para o DONO decidir

1. ⭐ **Seguir a família C (recomendada)** — P1..P5 acima. A tinta deixa de ter costuras, o artista
   nunca lê a palavra UV, e o atlas vira o **exportador**.
2. **Ficar na família A** — acabar o atlas (a wave que eu tinha proposto: levar a grade que a
   extracção já produz, `0,76×..1,34×` contra `0,37×..1,18×`). Mais curta, e paga costuras para
   sempre em 7 dos 13 modos.
3. **Parar aqui** — a pintura por-vértice que já shipa é, medida, `2,7×` mais fina no pior sítio do
   que o nosso atlas depois de dois `K`. *Não fazer nada é uma saída defensável, e é a primeira vez
   nesta linha que isso é verdade com número.*

⚠️ **As saídas 1 e 2 não são exclusivas** — a P5 **é** a 2, no papel certo. *A pergunta não é qual
das duas, é qual delas vem primeiro.*

---

## §14 — ⭐⭐⭐ A VIA 1 ESCOLHIDA: a lei existe e o dab já escreve nela

> **Ordem do dono (2026-09-20):** *«1»* — seguir pelos pontos da malha.

⚠️ **O que esta secção NÃO diz:** que o dono já pode ver. A **lei** está
construída e gateada e o **pincel** já a escreve; o que falta é a fiação que a
torna visível (armar pelo painel, subir à placa, a cena). Isso está na
[§15](#15--o-que-falta-para-o-dono-ver).

### §14.1 — As duas medições que escolheram a fiação

| medição | número | o que ela decidiu |
|---|---|---|
| construir o plano | **`40 ms`** na malha de fábrica (`99 225` V / `197 192` F) | ⇒ **não cabe num dab** — o plano arma-se uma vez, não se refaz por carimbo |
| `PRIMITIVE_INDEX` | **`true`** nas três rotas desta máquina (RTX 5060 Ti/Vulkan · RADV iGPU · RTX/GL) | ⇒ um shader de fragmento **pode saber em que face está**, que é o que a leitura por amostra pede |

Instrumentos versionados: `ph2d-mesh-colors --example mede_o_plano` ·
`ph2d-gpu --example o_que_a_placa_anuncia`.

### §14.2 — ⭐⭐⭐ O que muda no pincel: o SÍTIO onde a distância é medida, e só

A lei do pincel **não muda**. O peso de uma amostra sai das mesmas portas do dab
por-vértice, e a composição é a mesma do `GripLaw` verbo a verbo. O que muda é
onde a distância é medida.

⛔⛔ **E para que isso fosse verdade e não uma promessa, duas portas tiveram de
ser EXTRAÍDAS** ([`peso_do_ponto`](../../crates/ph2d-sculpt3d/src/peso_do_ponto.rs)):
a **curva de queda de um ponto** e a **ordem do produto** (`fall × intensity ×
keep`, cuja re-associação diverge em **30,4 %** dos triplos). A extracção é
verbatim e o caminho por-vértice fica byte-idêntico — quem o afirma são os
**618 + 248** testes das duas crates, que correram verdes sem uma barra mexer.
*Uma lei escrita em dois sítios ainda não é uma lei; só uma PORTA é.*

### §14.3 — ⭐⭐⭐ A escada, medida

Um traço de pintura na mesma peça, lido nos mesmos pontos da superfície pela
porta que o shader vai usar, comparado com o limite (nível `4`):

| nível | lado | amostras | erro contra o limite |
|---|---|---|---|
| `0` | `1` | `362` | `0,01753` |
| `1` | `2` | `1 442` | `0,00547` |
| `2` | `4` | `5 762` | `0,00146` |
| `3` | `8` | `23 042` | `0,00030` |
| `4` | `16` | `92 162` | — |

⇒ **o erro cai `~3,6×` por nível** e as amostras sobem `4×`, que é a assinatura
de um interpolante linear sobre um perfil suave. *A malha não mexeu uma vez.*

### §14.4 — ⛔ O que os gates apanharam EM MIM, e nenhum passou

1. ⛔⛔ **Um CANTO está em DOIS lados ao mesmo tempo**, e a minha régua devolvia
   `Option<usize>` com o primeiro. A `lado = 1` isso fazia **três dos seis**
   pares de um tetraedro saírem a dobrar — e o sintoma é *um fio mais escuro ao
   longo de metade das arestas da peça*.
2. ⛔⛔ **A `uv_sphere` desta casa é quase toda de QUADS**, e o meu laço só
   tratava triângulos ⇒ o pincel não pintava nada. O gate leu *«o nível zero
   divergiu»*, que é a frase certa sobre a causa errada.
3. ⛔⛔ **Armar a tinta fina sobre uma peça JÁ PINTADA apagava-a** — a
   `Tinta::nova` nasce branca. Desvio medido: **`1,0` num canal**, que é uma cor
   inteira e nunca um arredondamento. ⇒ a porta `Tinta::semeada`, que é também a
   lei de **subir e descer** o nível sem perder tinta.
4. ⛔⛔ **Faltavam-me TRÊS metades da lei do anel**: a própria amostra entra com
   peso `1` (sem ela um dab a peso cheio apaga a cor de uma vez), a direcção do
   esfregão é do **MODO** e não do caminho, e é preciso **DIVIDIR** em vez de
   multiplicar pelo recíproco. Desvio contra o caminho por-vértice:
   **`3,8e-2` → `5,96e-8`**, que é um ULP.
5. ⛔ A minha régua de convergência **lia onde a lei não age**: ela pintava no
   equador (tudo quads) e lia nos pólos (os únicos triângulos), e os quatro
   níveis deram `6e-7` — ruído de `f32`. *Uma régua que lê onde a lei não age
   mede o nada e chama-lhe empate.*

### §14.5 — ⛔ Uma constante foi escrita, MEDIDA e RETIRADA

A `MARGEM_DO_ANEL` existia para o anel não ficar truncado na borda da pegada.
Varrida contra o caminho por-vértice (que lê a adjacência inteira da malha), ela
lê **`5,96e-8` em `1,00`, `1,25`, `1,50` e `2,00`** — *o mesmo ULP nas quatro*,
porque a consulta do octree já é conservadora. ⇒ ela saiu, e quem garante a
propriedade é o gate. *Uma constante que a medição não consegue mover é um
comentário com sintaxe de código.*

### §14.6 — ⚠️ E um achado que NÃO é desta wave: de quem é o último bit

O gate irmão do caminho por-vértice afirma que o `Blur` é **inerte ao bit** numa
peça de cor uniforme. Medido com uma cor qualquer (`0,3 · 0,7 · 0,45`), **as
DUAS rotas mexem**: `4` vértices no caminho por-vértice e `103` amostras na
tinta fina (mais porque são mais). A causa é a forma do aplicador que as duas
partilham — `b·(1−a) + t·a`, a que o `stroke_apply` mede com **53 315**
divergências na coluna `t = b`. ⇒ *a inércia é uma promessa sobre a cor de
FÁBRICA*, e o gate passou a dizer qual é em vez de a herdar.

---

### §14.7 — ⛔⛔ Uma MUTAÇÃO SOBREVIVENTE achou um ramo SEM CHAMADOR — e, por baixo dele, um gate CITADO que nunca existiu

A `M1` troca o discriminante tri/quad do `topo::cantos`
(`4 if face[3] == TRI => 3` por `=> 4`) e **sobreviveu aos dezoito gates que a
crate então tinha**.

⭐ **A causa não é uma fixtura em falta, é um ramo que ninguém percorre.**
Todas as fixturas passam fatias de `3`, e no produto a
[`ph2d_mesh::Face::verts`](../../crates/ph2d-mesh/src/face.rs) **corta o
sentinela antes de sair** — ela devolve `&self.0[..self.vert_count()]`. ⇒ um
triângulo marcado **nunca chega** a esta crate pelo caminho do produto, e o
ramo que o lê é *defensivo*.

⚠️ **E ele FICA em vez de ser apagado, com o mecanismo:** a porta desta crate
aceita `&[u32]` cru, e o array de uma `Face` desta casa é um `[u32; 4]` com
`u32::MAX` no 4.º slot. Um chamador que passe `&face.0[..]` em vez de
`face.verts()` é o erro mais natural que aqui existe — e sem o ramo ele lê um
triângulo como quad com um canto `u32::MAX`, que é um `index out of bounds` na
`Tinta::semeada` ou endereços trocados **em silêncio**. *Um ramo defensivo sem
gate é indistinguível de um ramo morto, e os dois leem-se igual numa mutação.*
⇒ `o_sentinela_do_triangulo_nao_muda_uma_amostra`, cuja régua é a **igualdade
das duas `Tinta` inteiras** e não uma contagem (uma contagem igual com
endereços trocados é exactamente o defeito que ele existe para impedir).

⛔⛔ **E a puxar esse fio apareceu a família de 13/09, reintroduzida por mim:**
o doc da `TRI` desta crate dizia *«é GATEADO do lado de lá»* e apontava para
`ph2d-mesh :: o_sentinela_do_triangulo_e_o_mesmo` — **que o `git log -S` não
encontra**. Ele não podia existir ali: a `ph2d-mesh-colors` declara **zero
dependências** e a `ph2d-mesh` não a conhece, logo a primeira crate que vê as
duas constantes é a `ph2d-sculpt3d`, e é lá que o gate vive hoje
(`o_sentinela_do_triangulo_e_o_mesmo_nas_duas_crates`).

### ⛔⛔ E a SEGUNDA sobrevivente é a mesma forma noutro sítio: a fronteira de dois QUADS

A `M6` troca o `t: lado - j` do lado **`d→a`** do `sitio_quad` por `t: j` — e
**sobreviveu aos dezanove gates**, com a fronteira partilhada a ser a claim
central desta família inteira ([§3](#3--as-famílias-e-o-eixo-que-as-separa)).

⭐ **A causa é outra vez a fixtura, e a razão pela qual a bijecção não a vê é
exacta:** inverter o `t` de uma aresta é uma **PERMUTAÇÃO** do bloco dela, logo
a contagem de índices distintos fica *igual ao bit*. O que a apanha é a
igualdade por ponto **FÍSICO**, e o gate que a faz
(`as_duas_faces_leem_a_mesma_amostra_na_aresta_comum`) tinha como fixtura dois
**TRIÂNGULOS** — o único quad do corpus estava **sozinho**, onde não há
vizinho com quem discordar.

⭐⭐ **E o tamanho da fixtura nova é DERIVADO da pergunta, não escolhido:** os
quatro lados de um quad têm de ser partilhados pelo menos uma vez, senão o ramo
que não é cruzado fica sem régua. Numa **fita de dois** só os lados `1`/`3` se
tocam e o gémeo `c→d` ficava de fora — a `M10`, escrita depois, prova-o. ⇒ a
fixtura é uma **grelha `2×2`**, e o controlo positivo é uma contagem fechada:
`4(L+1)² − (2L+1)²` pontos vistos mais de uma vez.

### As dez mutações

⭐ **O arnês é versionado** — [`docs/3D/ferramentas/muta_a_lei_da_reticula.sh`](ferramentas/muta_a_lei_da_reticula.sh),
e corre-se de uma vez:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && \
  bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_lei_da_reticula.sh
```

⚠️ Ele **controla-se a si mesmo em três pontos**, porque cada um deles já
mentiu nesta casa: a âncora tem de casar **exactamente uma vez** (contada em
Python, não por `grep -cF`), a mutação tem de **compilar**, e a corrida tem de
correr **`N > 0`** testes — contados de `test result:`, porque o
`running N tests` conta os `#[ignore]`.


| # | lei | a troca | veredito |
|---|---|---|---|
| `M1` | `topo::cantos` | o sentinela lê-se como quad | ⭐ **sobreviveu à 1.ª corrida** |
| `M2` | `interior_por_face` | o triângulo conta o dobro | sangra |
| `M3` | `indice` | a aresta virada não vira o `t` | sangra |
| `M4` | `indice` | o canto deixa de ser o índice do vértice | sangra |
| `M5` | `total` | o bloco das arestas conta uma a mais | sangra |
| `M6` | `sitio_quad` | o lado `d→a` anda para a frente | ⭐ **sobreviveu à 1.ª corrida** |
| `M7` | `semeada` (tri) | `i` e `j` trocados | sangra |
| `M8` | `semeada` (quad) | a bilinear vira o primeiro canto | sangra |
| `M9` | `mistura` | os pesos são ignorados | sangra |
| `M10` | `sitio_quad` | o lado `c→d` anda para a frente | sangra *(o gémeo da `M6`, escrito por causa dela)* |

⚠️⚠️ **E o ARNÊS mentiu antes de dizer a verdade, na forma que esta casa já tem
escrita:** `grep -cF` conta **LINHAS**, logo uma âncora de duas linhas casa
«duas vezes» numa ocorrência só — as `M6` e `M8` **abortaram alto** por isso, e
a `M6` só se revelou depois de a contagem passar a ser feita em Python. *Um
aborto do arnês e uma sobrevivência leem-se igual num placar que não os
separe.*

⚠️⚠️ **O censo que cura essa família não a podia ver:** o
`named_gates_census_tests` lê a lista `FAMILIA`, que tinha **cinco** crates e
não a que nasceu esta semana. ⇒ ela entrou (`5 → 6`), e o nome morto entrou nas
`MEMORIAS` **com o mecanismo**, que é a porta que aquele censo já tinha para
prosa que não é endereço. *Uma crate-folha nova de uma família é população
nova, e o censo dela não a vê até alguém a escrever na lista.*

---

## §15 — ⏳ O que falta para o DONO ver

| peça | estado |
|---|---|
| a lei da retícula (`ph2d-mesh-colors`) | ✅ **20 gates, 10 de 10 mutações sangram** (§14.7) |
| o dab por amostra (as três leis de cor) | ✅ **7 gates** |
| armar o plano pelo painel (*Paint Detail*) | ✅ §16 |
| subir as amostras à placa e lê-las no shader | ✅ §16 |
| uma cena de smoke | ✅ a `=52` (§16) |
| o desfazer ligado à janela de amostras | ⚠️ a janela existe (`tocadas`/`base`); falta o consumidor |
| o passe de topologia com o plano armado | ⛔ **decisão de produto** — ver abaixo |

⛔⛔ **E a decisão que o dono tem de tomar quando a vir:** construir o plano
custa `40 ms`, logo ele **não pode ser refeito a cada carimbo que mude a
topologia**. A saída conservadora é *com a tinta fina armada, os pincéis de cor
deixam de adensar a malha* — e ⭐ **a razão pela qual eles a adensavam
DISSOLVEU-SE**: a ordem de 14/09 (*«dynamic topology para os 3 pincéis»*) vinha
de *«a cor por vértice é uma IMAGEM e a resolução dela É a da malha»*, e é
exactamente isso que esta wave deixa de ser verdade (§0.0 — *quem move o número
que tornava algo inalcançável tem de reconferir a nota*). A manutenção
incremental do plano é wave própria.

---

## §16 — ⭐⭐⭐⭐ A METADE VISÍVEL: o botão, a placa e a cena

*Ordem do dono, 2026-09-20: **«sim. siga»** — construir a parte visível.*

### §16.1 — O que o artista faz agora

Fileira **`Paint Detail`** no painel, colada à caixa de cor, com quatro chips —
**`Mesh`** (o caminho de sempre, ao bit) · `2x` · `4x` · `8x`. Com um deles
armado a tinta deixa de ter a resolução da malha, **e a malha não muda**. Cena
**`=52`**, que abre com uma peça GROSSA e o arame LIGADO de propósito: sem os
dois, o degrau que ela ensina é invisível.

### §16.2 — ⛔⛔ Onde o plano VIVE, e porque não é onde parecia

Ele vive na **PEÇA** (`SceneObject::tinta`) e não na cena: o endereço de uma
amostra é `(face, sítio)` **daquela** malha, logo um plano partilhado leria a
tinta de uma peça na geometria de outra. O knob do painel é a ESCOLHA
(`Sculpt3dScene::tinta_nivel`); o plano é o EFEITO.

⚠️ **E só a peça ACTIVA ganha um plano novo** — as outras mantêm o que já têm.
Um plano de nível `3` custa `64` amostras por vértice (medido, §16.5), e armar
um knob não pode multiplicar isso por toda a cena.

### §16.3 — ⭐⭐⭐ O plano é EMPRESTADO ao traço, e as três consequências

O `pen-down` faz um `take` do `Option` da peça e põe-no no `SculptStroke`; o
`close_stroke` devolve-o. É a forma que o cabeçalho da `TintaDoTraco` escolheu
para **não mexer na assinatura do `dab`**, que é a porta de todos os corpora de
oráculo desta casa. Três coisas caem disso, e as três são código:

1. **A reconciliação não corre durante o traço.** Ali o `Option` da peça está
   VAZIO — reconciliar construiria um plano BRANCO novo em cada quadro, que o
   `close_stroke` depois sobrescreveria. *Um alocador de dezenas de MB a 60 Hz,
   invisível a toda régua de cor.*
2. **O upload lê o plano de ONDE ELE ESTÁ.** Durante o gesto quem o segura é o
   traço; ler o `Option` da peça subiria `armado = 0` e *o artista veria a tinta
   fina desaparecer no instante em que começasse a pintar*.
3. **Devolver REESCREVE o canal por vértice** com as `V` primeiras amostras.
   Tudo o que não lê o plano (o assado, a doação, o `.ph2dproj`, o próprio
   caminho `Mesh`) lê `Mesh::colors`, e sem essa linha a peça voltava à tinta de
   antes assim que o artista desarmasse.

### §16.4 — ⚠️ A armadilha MUDA: o plano é paramétrico nas FACES

Ele sobrevive a qualquer pincel que só **mova** vértices e não sobrevive a um
que **parta ou funda** uma face. Um plano da topologia anterior lido sobre a
malha de agora **não estoura e não desenha lixo óbvio** — ele põe a tinta de uma
face na face vizinha.

⇒ `concorda_com` é lida em todo quadro (vértices **E** faces: *uma régua que
conta uma grandeza só aprova metade das mudanças de topologia*), e a
discordância **reconstrói o plano semeado da cor por vértice**, que é a única
resposta que a malha sabe dar.

⭐⭐ **E é por isso que o pen-down FALA** — a quarta entrada da família do
`recusa.rs`, e a **primeira que não é uma ausência**: as três de antes dizem
*«falta-te uma coisa»*, esta diz *«o que vais fazer vai CUSTAR uma que tu
tens»*. Ela não bloqueia; põe o preço à vista antes de ele ser pago.

⚠️ **A lente é a do CONSUMIDOR** (`refina_no_dyntopo` ∪ `colapsa_no_dyntopo`) e
não a do interruptor: um verbo que não mexe na topologia deixa-a em paz mesmo
com o passe ligado, e *um aviso que soa sempre é ruído que o artista aprende a
ignorar — exactamente quando ele passar a ser verdade*.

⛔ **Suprimir o passe de topologia enquanto o plano está armado é DECISÃO DO
DONO**, e fica aberta com as duas frases que a põem.

### §16.5 — ⛔⛔ O tecto, e a minha conta errada por 2×

A 1.ª redacção do doc do `NIVEL_MAX` dizia *«~3,1 M amostras, 37,7 MB»* para a
peça de fábrica. **Errado por 2×:** eu contei os interiores de cada quad e
esqueci que as ARESTAS também levam `L−1` amostras cada.

Medido (gate `o_custo_do_tecto_por_vertice_e_o_que_a_constante_diz`, num toro de
quads): `1 + 2(L−1) + (L−1)² = L²` ⇒ **`64` amostras por vértice** no tecto, e a
peça de fábrica (`98 306` vértices) custa **`6,29 M` amostras = `75,5 MB`**
contra `1,2 MB` do canal por vértice. O nível `4` seria `302 MB` **por peça**.

⚠️ *Um número escrito de cabeça ao lado de um tecto é o palpite que o §0.0
proíbe*, e a cura não é corrigir o número — é o gate medi-lo. A fixtura teve de
trocar de esfera para **toro**: os pólos de uma esfera UV são leques de
TRIÂNGULOS, e o multiplicador só descreve a família dos quads.

### §16.6 — ⚠️ O que o CLIPPY apanhou e o `check` não

Apagar o alias `pipeline::MESH_WGSL` (morto desde que a fonte passou a ser
COMPOSTA) partiu **cinco** sítios em `lighting_tests.rs` e três em
`shade_tests.rs`, com `cargo check -p <crate> --all-targets` **verde**: ele não
compila os testes de uma DEPENDÊNCIA. É a lei que o §5 do roteador já escreve, e
aqui ela mordeu na direcção barata (falha alto).

### §16.7 — ⛔⛔⛔ Os DOIS defeitos que eu introduzi, achados a RELER

Nenhum dos dois foi apanhado por um teste. Os dois vivem no mesmo sítio: **a
rota que o produto de facto toma não tinha régua nenhuma** — a bancada de
paridade tem arnês de compute próprio e nunca chama o `upload_tinta_at`, e a
suíte de desenho nunca arma um plano.

**(a) `cap_idx` era DERIVADO de `cap_tri`, e mentia na PRIMEIRA subida.**

```
poe(.., &mut g.cap_tri, origem)   // realoca: cap_tri = tris*4 bytes
let mut cap_idx = g.cap_tri * 3;  // = tris*12  (um LOCAL, perdido)
poe(.., &mut cap_idx, idx)        // n <= cap  ⇒  ESCREVE
```

O `origem` realoca e actualiza o `cap_tri`; `cap_tri * 3` já descreve o tamanho
NOVO enquanto o `idx` ainda é o buffer-dummy de `16` bytes ⇒ `write_buffer` de
milhares de bytes lá dentro = **erro de validação do `wgpu`**. ⭐ *Uma
capacidade derivada da de outro buffer é uma segunda resposta à pergunta «quanto
cabe AQUI?»* ⇒ `cap_idx` é campo.

⚠️ **E o que escondia a classe era um ACIDENTE:** as capacidades iniciais diziam
`4` sobre buffers de `16` bytes — conservador por engano. *Uma capacidade que
mente para baixo só desperdiça; uma que mente para cima escreve fora do buffer*,
e o `4` fazia a primeira parecer inofensiva. Hoje elas são o tamanho real.

⇒ gate novo [`tinta_no_device.rs`], com **a sequência que estoura**: subir, e
subir outra vez MAIOR. Uma subida só não chega — o defeito nasce da realocação.
⚠️ E o veredito vem do `ErrorScopeGuard` do `wgpu` 29: uma escrita fora do
buffer é um erro de VALIDAÇÃO, entregue por callback e não por `Result` — sem o
escopo o teste passaria com o device a acumular erros em silêncio.

**(b) O PLANO não contava no peso da peça.**

O braço `StrokeUndo::RemovedObject` guarda um `SceneObject` **inteiro**, logo
apagar uma peça com tinta fina armada punha até `75 MB` na fila de desfazer
**invisíveis ao tecto em bytes que existe para os impedir**. ⛔ E o doc do
`footprint_bytes` dizia, por escrito, *«a pilha inteira, que é tudo o que tem
tamanho aqui»* — **uma enumeração fechada num doc é uma afirmação que o campo
seguinte contradiz em silêncio**. ⇒ `Tinta::footprint_bytes` (na crate que
POSSUI os vectores) e a soma, com o CONTROLO no gate: sem plano, o número não
muda.

### §16.8 — ⛔⛔ E o ARNÊS da mutação mentiu CINCO vezes, com o CONTROLO dentro

A 1.ª corrida deu `9 de 15` com **cinco abortos** *«zero testes correram»* — e
um deles era o **CONTROLO** (uma linha em branco, que não pode abortar).
Corridas à mão provaram que as cinco de facto **SANGRAVAM**: a `M6`, medida
isolada, dá `rc = 101` com `257` testes contados e o teste certo vermelho.

⚠️ *Um aborto MUDO lê-se exactamente como uma mutação que não entrou*, e nas
duas leituras o número final é o mesmo. ⇒ o aborto passa a **IMPRIMIR as últimas
linhas da corrida**: *um instrumento que se declara inconclusivo sem dizer
porquê não é mais honesto que um que mente.*

⭐⭐⭐ **E a prova que ele passou a imprimir acusou o BINÁRIO DE TESTE:** as cinco
mensagens são a mesma — `signal: 11, SIGSEGV` no processo do `--lib` da
`ph2d-app-sculpt3d`, que é o defeito **já NOMEADO** no `CLAUDE.md` §5 desde
19/09 e que nenhuma medição soube atribuir. Ele mata o processo inteiro e leva o
`test result:` com ele ⇒ *o arnês reportava o próprio acidente*.

⭐⭐ **A cura estava escrita na mesma nota:** o `nextest` corre **um processo por
teste** e lê `229/229` em 3 de 3 na mesma árvore. Hoje o `corrida()` é
`cargo nextest run`, e a população é o **`N tests run`** do `Summary` —
⚠️ **nunca o `running N tests` do libtest, que CONTA os `#[ignore]`**. Medido
depois da troca: `296 tests run: 296 passed`, **zero abortos**.

### §16.8-bis — ⛔⛔⛔ E com o arnês honesto sobrou UMA sobrevivente REAL: uma barra ESCOLHIDA

A `M13` leva as `LATITUDES` da cena `=52` de `24` para `96` — a peça passa de
**`738` para `3 042`** vértices, **quatro vezes mais fina** — e o gate da peça
grossa **passava**. Ele pedia *«pelo menos `4×` mais grossa que a irmã»*
(`3 042 × 4 = 12 168 < 13 682` ✅) e *«entre `400` e `4 000` vértices»* (`3 042`
✅), e **nenhuma das duas nomeava recurso nenhum** (§0.0).

⭐⭐⭐ **A barra nova DERIVA-SE, e os dois lados dela já existem no produto:** a
fileira tem quatro chips e a peça da `=51` é a densidade em que a marca por
vértice **já sai limpa** — a cena que o dono aprovou. ⇒

> **o chip que o roteiro manda carregar tem de ser o PRIMEIRO da fileira que
> alcança essa densidade.**

Ela aperta pelos **dois** lados sem uma constante escolhida: com a peça fina de
mais o `4x` já lá chega (a mutação), com a peça grossa de mais **nem o `8x`
chega** e a cena promete um detalhe que o produto não entrega.

⚠️ **A contagem sai da `Tinta` e nunca da fórmula `L²`** — os pólos de uma
esfera UV são leques de TRIÂNGULOS, e o multiplicador só descreve quads (a mesma
armadilha do §16.5). ⭐ E o **CONTROLO do gate é a própria mutação**: ele
constrói a peça `4×` mais fina e exige que ela alcance a densidade limpa **antes**
do topo — *sem isso a régua podia responder «é o último» por vácuo*.

**Placar final: `15 de 16 sangram`**, com a `16.ª` a ser o CONTROLO, que não
pode. ⚠️ *Foram precisas TRÊS corridas para o número querer dizer alguma coisa —
as duas primeiras mediam o instrumento, não o produto.*

### §16.9 — ⏳ O que fica ABERTO

- ~~**A cor não viaja no `.ph2dproj`**~~ — ⛔ **MEDIDO e FALSO** (21/09): o
  `MeshData` tem `colors`, o `to_data`/`from_data` carregam-no, e uma sonda pelo
  caminho real (`encode_doc` → `decode`) lê **`SIM, AO BIT`**. *A dívida era
  herdada de uma nota, não do código.* E o **plano** passou a viajar no mesmo
  dia (handoff §18).
- **O upload é INTEIRO e por quadro durante um traço** (`O(V + F)`): a escrita
  da tinta fina é por AMOSTRA e não passa pelo `dirty`, que é uma janela de
  VÉRTICES — não há upload parcial a que recorrer. O custo por quadro **não foi
  varrido** (a máquina esteve entre `load 10` e `27` a jornada inteira, e
  nenhuma leitura de relógio vale nada acima de `~5`).
- ~~**Voltar a `Mesh` perde o detalhe fino**~~ — ⭐ **CURADO em 23/09 por ordem
  do dono** (handoff §32): o plano que sai fica numa **ranhura por peça** e
  volta inteiro quando o artista re-arma o mesmo degrau. ⚠️ A ranhura é **uma**,
  logo o que o mata é passar por **DOIS** degraus diferentes pelo meio, pintar
  no nível da malha, ou esculpir enquanto ele dorme — e o roteiro da `=52`
  di-lo no passo (5).
- **O passe de topologia com o plano armado** — decisão do dono (§16.4).

---

## §12 — Registo de contaminação, e é meu

⚠️ Ao pesquisar como a referência escreve os píxeis, uma busca devolveu-me **a descrição em prosa do
algoritmo de *projection paint* do Blender**, tirada do rastreador público de defeitos dele (o
esquema de baldes por ecrã, a rasterização por triângulo, a lista de píxeis em cache e a dilatação
de costura). Eu **não** abri fonte nenhum, e o material é documentação pública sobre o programa, não
código — mas o método desta casa manda que o alvo com parede entre só como **oráculo que se corre**.

⇒ **Consequência, e ela é executável:** nada nesta pesquisa deriva daquela leitura — a §8 é análise
do nosso próprio catálogo de modos, e a recomendação é a família que o alvo **não** usa. Se alguma
wave vier a implementar projecção de ecrã, ela passa pelo protocolo normal (janela **E** para correr
o alvo, **R** para atestar), e **quem escrever o produto não sou eu**.

---

## §13 — Fontes

- [Rethinking Texture Mapping — Yuksel, Lefebvre, Tarini, STAR Eurographics 2019](https://onlinelibrary.wiley.com/doi/10.1111/cgf.13656) · [curso SIGGRAPH 2017 com as notas em PDF](http://www.cemyuksel.com/courses/conferences/siggraph2017-rethinking_texture_mapping/)
- [Mesh Colors — Yuksel, Keyser, House (TOG 2010)](https://www.cemyuksel.com/research/meshcolors/meshcolors_tog.pdf) · [Mesh Color Textures — Yuksel (HPG 2017)](https://www.cemyuksel.com/research/meshcolors/mesh_color_textures.pdf) · [página do projecto](https://www.cemyuksel.com/research/meshcolors/)
- [Htex: Per-Halfedge Texturing for Arbitrary Mesh Topologies — Barbier, Dupuy (HPG 2022)](https://arxiv.org/abs/2207.05618) · [repositório MIT](https://github.com/wbrbr/htex)
- [Ptex — Burley, Lacewell (Walt Disney Animation Studios)](https://www.disneyanimation.com/open-source/ptex/) — e o cabeçalho **instalado** em `/usr/include/Ptexture.h`
- [xatlas — parametrização e empacotamento, MIT](https://github.com/jpcy/xatlas)
- [Boundary First Flattening — Sawhney, Crane, MIT](https://geometrycollective.github.io/boundary-first-flattening/)
- [Least Squares Conformal Maps for Automatic Texture Atlas Generation — Lévy et al., SIGGRAPH 2002](https://dl.acm.org/doi/10.1145/3596711.3596734)
- [Scalable Locally Injective Mappings (SLIM) — Rabinovich et al.](https://igl.ethz.ch/projects/slim/)
- [PartUV: Part-Based UV Unwrapping of 3D Meshes — SIGGRAPH Asia 2025](https://arxiv.org/html/2511.16659v2) · [ArtUV](https://arxiv.org/pdf/2509.20710) · [DreamUV](https://arxiv.org/pdf/2606.22445)
- [Advances in Neural 3D Mesh Texturing: A Survey (2026)](https://arxiv.org/html/2606.00137v1)
- [3D-Coat — os três modos de pintura (per-pixel · microvertex · Ptex)](https://3dcoat.com/documentation/manual/workspaces-rooms/paint/texture-painting-and-modes/)
- [Maxon ZBrush — PolyPaint](https://www.maxon.net/en/zbrush/features/polypaint)

---

## ⛔ Recusas MEDIDAS

| recusa | onde | porquê |
|---|---|---|
| portar Ptex como destino de pintura | [§10.1](#10---o-que-não-fazer-com-o-motivo) | adjacência de triângulos `TBD` no cabeçalho instalado; escritor de ficheiro; fronteira duplicada |
| trocar o `G3` por solver isométrico | [§10.2](#10---o-que-não-fazer-com-o-motivo) | o chão da dispersão é um teorema, não o solver |
| desdobramento por partes / neural | [§10.3](#10---o-que-não-fazer-com-o-motivo) | resolve o problema de quem não tem a topologia; minutos por peça |
| geração neural de textura | [§10.4](#10---o-que-não-fazer-com-o-motivo) | inventa a textura; é outro produto |
