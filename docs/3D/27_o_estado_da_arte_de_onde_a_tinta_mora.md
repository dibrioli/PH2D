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
