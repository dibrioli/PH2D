# AVALIAÇÃO — o Painter a pintar na malha 3D, com todos os seus modos

> **Pergunta do dono (2026-09-20):** *«avalie a possibilidade de usar o módulo
> painter em todo seu poder para pintar sobre a malha 3d. Ou seja: hoje o Módulo
> painter pinta sobre um sprite. Como fazê-lo pintar sobre a malha 3d exatamente
> como pinta sobre a sprite? Com todos os seus modos.»*
>
> ⛔ **Isto é uma AVALIAÇÃO, não uma wave.** Nada foi implementado. Os números
> abaixo são medidos nesta árvore em 2026-09-20; os que faltam estão **nomeados
> como faltando** em vez de estimados.

---

## §0 — A resposta em três frases

1. **Dá, e a peça mais cara já está construída** — a casa tem um parametrizador
   UV global (`ph2d-gridmap`), medido, gateado e no produto.
2. **«Exactamente como pinta sobre a sprite» obriga a malha a TER uma imagem.**
   Não há atalho: o Painter é **denso em `(x, y)`** de ponta a ponta — 13 modos,
   4 meios e 4 buffers por camada, todos indexados por pixel. Dar-lhe outra
   coisa é dar-lhe outro produto.
3. **Os modos NÃO atravessam todos pelo mesmo preço.** Os que escrevem por-pixel
   atravessam de graça; os que leem a **vizinhança** (Smear, Blur, Deform, Wet
   Paint, Knife, Fill, Inpaint — **7 dos 13**) param nas **costuras** do atlas, e
   esse é o defeito estrutural que toda pintura em textura tem.

---

## §1 — As duas pontas, medidas

### O Painter, do lado de dentro

| peça | o que é | medido |
|---|---|---|
| a porta de entrada | `CanvasPaintTool::on_canvas_pointer(CanvasPointer)` | **UM** método, 🔒 congelado (ADR-0040 Am. 3) |
| o que o ponto é | `pos: [f32; 2]` em **pixels de imagem** + pressão + inclinação + fase | o doc di-lo: *«the shell has already removed the canvas pan/zoom»* |
| onde ele escreve | `images` · `heights` · `covers` · `mats`, **um `Vec` por camada** | `BTreeMap<RtLayerId, Arc<Vec<…>>>` |
| o tamanho | `source_size: (u32, u32)` | uma imagem, e só |
| os modos | `PaintMode` | **13** (`Paint` · `Smear` · `Blur` · `Clone` · `Mask` · `Inpaint` · `Fill` · `Selection` · `Deform` · `Sculpt` · `Knife` · `WetPaint`) |
| os meios | `PaintMedia` | **4** (`Digital` · `Watercolor` · `Impasto` · `WetPaint`) |

⭐ **A conclusão que isto força:** o Painter **nunca pergunta o que a superfície
é**. Ele pede um ponto em pixels e escreve num `Vec` indexado por pixel. *Toda a
potência dele — a sim de fluido, o relevo, o liquify, a selecção — vive nessa
indexação.*

### A malha, do lado de dentro

| canal da `Mesh` | tipo | classificação |
|---|---|---|
| `positions` · `normals` | `[f32; 3]` | — |
| `curvatures` · `curv_world` | `f32` | **derivados** |
| `colors` · `masks` | `[f32; 3]` · `f32` | **autorados** (lazy) |
| `ao` · `thickness` | `f32` | **medidos da forma** |
| **`uv`** | — | ⛔ **NÃO EXISTE** |

⇒ **a malha não tem UVs, e tudo o que ela carrega é por-vértice.**

### O fio que JÁ liga os dois — e o sentido dele

`ph2d-form-donation::donated_form`: a malha rasteriza `[nx, ny, nz, cobertura]`
e a oclusão **por texel do canvas do Painter**, e o passe de luz do Painter
compõe isso com o gradiente da tinta. O doc declara a cerca:

> *«quem rasteriza a malha é o `ph2d-mesh-render`, uma vez, e o resultado chega
> aqui como números. É essa cerca que mantém o Painter sem saber o que é um
> triângulo.»*

⚠️⚠️ **E ele é SCREEN-SPACE**, não UV — o próprio doc o diz ao listar quando o
plano muda: *«a forma só muda quando a malha, a **câmera** ou o canvas mudam»*.

⇒ *hoje a malha DOA forma ao Painter para a tinta 2D acender; o dono pede o
sentido inverso, e o fio existente não serve para ele.*

---

## §2 — A fronteira exacta

Para o Painter pintar na malha **com todos os modos**, a corrente tem de fechar
nos dois sentidos:

```
cursor no ecrã ──► ponto na malha ──► (u, v) ──► pixel da textura ──► o Painter pinta
                                                        │
                            o shader amostra ◄───────────┘
```

**O elo que não existe é o `(u, v)`.** Tudo o resto ou já existe, ou é barato:

| elo | estado | onde |
|---|---|---|
| cursor → ponto na malha | ⭐ **existe** | o `pick` devolve **face + baricêntricas** (a wave do pincel afiado já as usa) |
| ponto → `(u, v)` | ⛔ **falta o canal** | a interpolação baricêntrica é trivial **assim que a malha tiver UVs** |
| `(u, v)` → pixel | ⭐ trivial | `source_size` |
| o Painter pinta | ⭐ **existe INTEIRO** | os 13 modos, os 4 meios |
| a textura volta ao ecrã | ⛔ **falta** | o shader amostra `vcolor` hoje, não uma textura |
| quem POSSUI a textura | ⭐ **quase de graça** | `PaintedDoc` e `Sculpt3dPieceRef` são **ambos** componentes de entidade: a peça ganha um `PaintedDoc` e herda undo, persistência e instâncias |

---

## §3 — O que JÁ está construído (e é a parte cara)

### ⭐⭐⭐ O parametrizador UV existe, está no produto e é gateado

`ph2d-gridmap` declara-se, no próprio cabeçalho, *«**o MAPA DE GRADE INTEIRA** —
uma parametrização para a peça inteira, em vez de um achatamento por patch»*, e
o tipo que ele devolve é:

```rust
pub struct GridMap {
    /// Por patch, por vértice local, o `(u, v)` em unidades de grade.
    pub uv: Vec<Vec<[f32; 2]>>,
}
```

Com ele vêm, já medidos e gateados:

| peça | estado |
|---|---|
| **G1** — a malha cortada em discos + a tabela de **costuras** (`CutMesh` · `Seam` · `SeamSide`) | feito |
| **G3** — o solver contínuo, com a **costura por ELIMINAÇÃO de variável** (não por penalização) | feito, e a obra A de 24/08 fechou a casca: resíduo `1,00 → 0,000`, bordo `30`–`78 → 0`, `χ → +2` |
| a qualidade | **enviesamento `3,8°`–`6,5°`**, dentro da barra do oráculo de produção (`4,8`–`7,1°`) |

⚠️ **Ele foi construído para OUTRO fim** (extrair quads das isolinhas inteiras), e
é por isso que ninguém reparou que a casa tem um *unwrapper*. ⭐ *A pergunta do
dono é, em boa medida, sobre uma capacidade que já foi paga.*

### E a `Mesh` já sabe que um quinto canal vem aí

O `mesh_shrink.rs` escreve, ao lado do bloco que faz a média dos canais num
colapso:

> *«o dia em que entrar um **quinto plano por-vértice** quem esquecer dele é esta
> função»*

⇒ o sítio onde um canal `uv` teria de ser tratado **está nomeado por escrito**,
nas duas portas (`splice_topology` no refino, `shrink_topology` no colapso).

---

## §4 — O que falta, peça a peça

| # | peça | dificuldade | porquê |
|---|---|---|---|
| 1 | o canal `uvs: Option<Vec<[f32; 2]>>` na `Mesh` | **baixa** | é o 5.º plano, e as duas portas já o prevêem por escrito |
| 2 | **empacotar as ilhas** em `[0, 1]²` | **média** | ⛔ **nada disto existe no repo** — o `GridMap` devolve `(u, v)` em unidades de grade, por patch, **sem normalização e sem arrumação**. É um *bin-packing* de rectângulos com margem |
| 3 | a textura na peça + o shader a amostrá-la | **média** | o shader tem hoje `vcolor` por-vértice; passa a precisar de `uv` + `texture_2d` + sampler. Toca o `ph2d-mesh-render`, partilhado com o modelador |
| 4 | o `PaintedDoc` na entidade da peça + a ponte de posse | **baixa** | os dois são componentes; o precedente é o sprite |
| 5 | o cursor → `(u, v)` → `CanvasPointer` | **baixa** | o pick já dá face + baricêntricas |
| 6 | a **dilatação das costuras** (*bleed*) | **média, e é onde mora o defeito** | ver §5 |

⚠️ **E a peça 2 tem um preço que não é código:** empacotar bem decide quanto da
textura é desperdiçado. Um empacotamento ingénuo gasta metade dos texels.

---

## §5 — ⛔ Os 13 modos NÃO atravessam pelo mesmo preço

Um atlas UV corta a superfície em ilhas. Dentro de uma ilha, o vizinho no espaço
da textura **é** o vizinho na superfície. **Na fronteira, não é.**

| modo | o que ele lê | atravessa? |
|---|---|---|
| `Paint` · `Clone` · `Mask` · `Sculpt` | escreve **por pixel** | ⭐ **de graça** |
| `Selection` | uma máscara por pixel | ⭐ de graça (⚠️ e hoje *«paints nothing on canvas»* — o motor dela ainda não existe) |
| `Smear` · `Knife` | arrasta o conteúdo **ao longo do traço** | ⛔ **pára na costura** |
| `Blur` | a média da **vizinhança** | ⛔ pára na costura |
| `Deform` (liquify) | `out[dst] = sample(dst − D(dst))` — um **warp de imagem** | ⛔ pára na costura |
| `WetPaint` | uma **simulação de fluido** densa (o líquido escorre entre pixels) | ⛔⛔ pára na costura, e é o pior caso: a água atravessaria para a ilha vizinha errada |
| `Fill` | *flood fill* por região conexa | ⛔ pára na costura |
| `Inpaint` | PatchMatch multiescala sobre a vizinhança | ⛔ pára na costura |

⇒ **7 dos 13 leem vizinhança.** Nenhum deles quebra — eles funcionam, e produzem
uma **descontinuidade visível na costura**, exactamente como no alvo (o report
público *«seams in texture painting»* é o defeito mais conhecido do Blender
Texture Paint).

### As curas conhecidas, e o que cada uma custa

| cura | o que faz | preço |
|---|---|---|
| **bleed / dilate** | escreve alguns texels **para fora** de cada ilha | barato, e **não cura** — só esconde o fio até ao raio dilatado |
| **menos costuras** | ilhas maiores ⇒ menos fronteira | o nosso G1 corta por *patch*, e o número de patches é do traçado |
| **atlas sem costura** (PTex / mapas por-face) | cada face tem a sua mini-textura | ⛔ obriga a reescrever a indexação inteira do Painter — é o oposto de *«exactamente como pinta sobre a sprite»* |
| **leitura pela SUPERFÍCIE, não pela textura** | quem lê vizinhança atravessa a costura pelo mapa de adjacência | ⭐ correcto, e é **lei nova em 7 modos** — a antítese de composição |

⚠️ **Nenhuma destas está medida nesta casa.** Qualquer número que eu desse aqui
seria um palpite.

---

## §6 — ⚠️⚠️ O conflito que decide a ORDEM do trabalho

**Topologia dinâmica e pintura em textura são hostis uma à outra**, e não por
implementação:

* o refino cria vértices **no meio de arestas** — o UV interpolado está certo, e a
  densidade de texels por área da superfície deixa de ser uniforme;
* o colapso **funde** dois vértices, e o UV médio de dois vértices em lados
  opostos de uma costura **não está em ilha nenhuma**;
* e a lei do §27 desta linha aplica-se igual: *a renumeração de um colapso é uma
  **CADEIA**.*

⭐ **É por isso que as três referências separam as fases**, e a casa já está
alinhada com elas sem o ter planeado:

| fase | a ferramenta | estado nesta casa |
|---|---|---|
| esculpir com topologia dinâmica | **cor por-vértice** (*polypaint*) | ⭐ **feito ontem** — `Paint` · `Blur` · `Smear Color`, com a caixa de cor de hoje |
| arrumar a malha | retopologia | ⭐ **feito** — o botão `Quad Retopology` |
| parametrizar | *unwrap* | ⭐ **feito e não exposto** — o `ph2d-gridmap` |
| **pintar em textura** | o Painter | ⛔ **é esta a pergunta** |

⇒ *a wave de ontem não é um substituto desta: é o degrau anterior dela, e as duas
coexistem nas três referências.*

---

## §7 — As três saídas, com o preço de cada uma

### (A) Textura com UV — *«exactamente como pinta sobre a sprite»*

O que o dono pediu, à letra. Os 13 modos e os 4 meios chegam **sem uma linha
nova no Painter**, porque a textura **é** a imagem que ele já sabe pintar.

* **compra:** o poder inteiro do Painter, e um formato que qualquer motor lê.
* **paga:** as 6 peças do §4, e as costuras do §5 em 7 modos.
* **memória, medida:** `RGBA8 (4) + altura f32 (4) + cobertura u8 (1) + material [u8;7] (7) = **16 B por pixel por camada**`
  ⇒ a `2048²` = **67 MB/camada** · a `4096²` = **268 MB/camada** (mais o composto).
  ⚠️ *Uma peça com cinco camadas de impasto a `4096²` passa de 1 GB.*

### (B) Projecção do ecrã (*project paint*)

O artista pinta no ecrã e o resultado é projectado de volta.

⛔⛔ **ESTE PARÁGRAFO ESTAVA ERRADO PELA METADE, e a §11 é a correcção.** Ele
dizia *«não evita nada: ele também precisa da textura e das UVs para ter onde
gravar»* — **certo sobre o DESTINO e falso sobre a VIZINHANÇA**, que é o que de
facto limita 7 dos 13 modos (§5). Ver a §11.

### (C) Ficar no canal por-vértice e enriquecê-lo

Dar à malha `height`/`cover`/`material` **por vértice** em vez de por pixel.

* **compra:** zero costuras, zero UVs, convive com topologia dinâmica.
* **paga:** a resolução é a da malha (a lição que a cena `=51` ensina), e ⛔ **os 7
  modos de vizinhança continuam a não existir** — um *liquify* ou uma sim de
  fluido por-vértice é **outro motor**, não o Painter.
* ⇒ *isto não é «o Painter na malha»; é mais polypaint.*

---

## §8 — Recomendação

**(A), em três waves, e a primeira não toca no Painter.**

| wave | o que entrega | porque esta ordem |
|---|---|---|
| **W1 — a malha ganha UV** | o canal `uvs` + as duas portas + o empacotamento de ilhas + um botão *Unwrap* que expõe o `gridmap` que já existe | ⭐ ela **tem valor sozinha**: exportar um `.obj` com UV serve qualquer motor, e o dono pode julgá-la sem que o Painter exista |
| **W2 — a peça ganha uma textura** | o `PaintedDoc` na entidade + o shader a amostrar + o `Flat`/matcap a compor | ⭐ julgável sozinha: importar uma imagem e vê-la na peça |
| **W3 — o ponteiro chega ao Painter** | cursor → face+baricêntricas → `(u,v)` → `CanvasPointer` | ⭐ **aqui os 13 modos acendem de uma vez**, e as costuras aparecem — com o dono a julgar quais o incomodam |

⚠️ **A W3 é a barata**, e é contra-intuitivo: o trabalho está em dar à malha uma
imagem, não em ligar o pincel.

⛔ **E a W3 é onde as costuras se medem, não antes.** Escrever hoje um plano de
cura para elas seria escolher entre quatro saídas sem uma tabela — exactamente o
que o §0.0 proíbe.

---

## §9 — O que falta MEDIR antes da primeira linha

> ✅ **MEDIDO em 2026-09-20 — quatro das cinco linhas desta tabela estão fechadas, e duas
> delas ao CONTRÁRIO do que ela supunha:** a malha do artista parametriza-se **melhor**
> que a remalhada, e um patch **não** é uma ilha (são `4`–`13`, não `88`–`116`). A quinta
> continua a ser acto de uma janela **E**. ⇒ [`26_a_parametrizacao_como_atlas.md`](26_a_parametrizacao_como_atlas.md),
> com o instrumento versionado em `ph2d-quadchain/examples/atlas_probe.rs`.
>
> ⛔ **E a 1.ª linha da tabela abaixo está ERRADA como estava escrita:** a casa TEM o
> número por fase desde que a `ChainTiming` existe, e o `chain_time` imprime-o pela porta
> do produto; os `123 s` que ela cita são o **BOTÃO** (quatro tentativas em cascata sobre
> dois campos), não a cadeia, que na peça do dono custa `7 884 ms`.

| pergunta | porque não está respondida | como se mede |
|---|---|---|
| quanto custa a **parametrização sozinha** (G1–G3) | o único número que a casa tem é a cadeia INTEIRA de retopologia: **`123 s`** na peça do dono, e ela inclui F1–F5, a extracção, o acabamento e **4 tentativas em cascata** | correr o `gridmap` isolado sobre o corpus, com o relógio |
| quantas **ilhas** e quanta **fronteira** uma peça típica produz | decide se a costura é uma nota de rodapé ou o assunto | contar `Seam`s e comprimento de fronteira sobre o corpus |
| que **resolução** a peça do dono pede | decide a memória (§7) e se `2048²` chega | o `tip_deviation` já mede em unidades de quad; a régua equivalente em texels não existe |
| o que o **alvo** faz na costura | é o único lado APROVADO que existe | ⚠️ **acto de um agente E** — o Blender é GPL, parede obrigatória, e a porta (`blender -b -P`) já está medida |
| o desperdício de um **empacotamento** | decide se `2048²` rende `2048²` ou metade | medir a área útil depois de arrumar |

⛔ **E uma decisão de produto que é do dono, não minha:** com topologia dinâmica
ligada, a textura fica esticada onde a malha adensou. As três referências
resolvem isso **proibindo** — pinta-se em textura *depois* de retopologizar. Se
o dono quiser as duas ao mesmo tempo, isso é uma quarta saída que nenhuma
referência tem, e teria de ser medida antes de prometida.

---

## §10 — O que esta avaliação NÃO afirma

* Não mediu o relógio da parametrização sozinha (§9, linha 1).
* Não mediu nenhuma cura de costura — as quatro do §5 estão **nomeadas**, não
  medidas.
* Não correu o alvo. A triagem foi feita (Blender = GPL ⇒ **parede**; a porta
  `blender -b -P` está medida na tabela do arsenal), e correr o oráculo é acto de
  uma janela **E**, nunca desta.
* Não olhou o fonte de alvo nenhum.

---

## §11 — ⭐⭐⭐⭐ «Ter de trabalhar com UV é necessário?» (pergunta do dono, 2026-09-20)

> *«ter que trabalhar com UV é necessário? Já há tecnologia suficiente para o
> usuário nem pensar em UV?»*

A pergunta obrigou a re-medir, e ela **derruba parte do §7(B)**.

### A resposta tem duas metades, e elas são diferentes

| a pergunta | resposta |
|---|---|
| **o ARTISTA tem de pensar em UV?** | ⭐ **NÃO — e isso é a norma da indústria há uma década.** Ninguém num Substance Painter ou num 3D-Coat desdobra à mão |
| **o SISTEMA precisa de UV?** | **não obrigatoriamente** — há três vias, e uma delas tem porta **aberta e instalada** |

⚠️ **E a W1 do §8 estava exposta ao contrário:** eu escrevi *«um botão Unwrap»*,
e um botão com esse nome **é** fazer o artista pensar em UV. O desdobramento não
tem de ser um gesto: ele acontece **quando a peça ganha uma textura**, e o UV é
um detalhe que ninguém vê — exactamente como ninguém vê o octree.

### ⛔⛔ O erro do §7(B): «UV ou não UV» é a pergunta errada

A grandeza que decide não é a existência do UV — **é onde a vizinhança é
correcta**, porque é isso que os 7 modos de vizinhança leem:

| onde o traço acontece | o vizinho de um pixel é o vizinho na superfície? |
|---|---|
| atlas UV | ⚠️ **dentro da ilha sim, na costura não** |
| **Ptex** (uma textura por face) | ⛔ **dentro da face sim** — *mais* fronteiras, não menos |
| cor por-vértice | ⚠️ é o anel da malha, e o Painter não sabe lê-lo |
| ⭐ **o ECRÃ** | ⭐⭐⭐ **SEMPRE. É uma imagem verdadeira.** |

⇒ **a via que dá ao Painter o poder TOTAL é pintar no ECRÃ e gravar o resultado
onde quer que seja.** Durante o traço, o Smear, o Blur, o liquify, o PatchMatch
e a simulação de fluido correm sobre uma imagem 2D **perfeita** — que é
literalmente o caso para que foram escritos.

E aí o **destino deixa de precisar de suportar vizinhança nenhuma**: ele só
guarda *«que cor/altura/cobertura/material há neste ponto da superfície»*.

### ⭐⭐ E a metade CARA desta via já está construída, medida e com cache

`MeshRenderer::form_plane` rasteriza a malha **no tamanho do canvas do Painter**,
com a câmera do escultor, e o `FormStamp` decide quando re-rasterizar — *«uma
forma parada custa zero»*. A tabela de custo já está no repo (`measure_a_donation`,
RTX, release):

| canvas | uma rasterização | lidos |
|---|---|---|
| 512² | 1,54 ms | 4 MB |
| **1024²** | **5,94 ms** | 16 MB |
| 2048² | 27,72 ms | 64 MB |
| 4096² | 123,49 ms | 256 MB |

⛔ **O que falta nesse G-buffer é UM plano:** ele devolve `[nx, ny, nz, cobertura]`
e a oclusão, e **não** a posição em mundo nem `(face, baricêntricas)`. Sem isso
não há como projectar de volta. ⭐ É o passe que já corre, com um alvo a mais —
o SSAO dele já usa profundidade internamente.

### As três vias para o DESTINO, sem o artista ver UV em nenhuma

| via | o artista vê UV? | resolução da tinta | licença do estado da arte |
|---|---|---|---|
| **atlas automático** | ⭐ não (nasce com a textura) | a da textura (`2048²` = **42,7×** as amostras do default do módulo) | — (o `ph2d-gridmap` é nosso) |
| **Ptex** — uma textura por face | ⭐ não (não há UV nenhum) | por face, adaptativa | ⭐⭐ **BSD-3-Clause, da Disney, e `ptex 2.5.4` está INSTALADO nesta máquina** (`/usr/include/Ptexture.h`, `/usr/lib/libPtex.so`) |
| **por-vértice** (o de hoje) | ⭐ não | a da malha — e o número mede-se: o default do módulo equivale a uma textura de **`313²`**; só depois de `K` duas vezes (`1,57 M` vértices) ele chega a `1254²` | — |

⚠️⚠️ **A triagem parou na primeira porta ABERTA (§0.9), e ela estava instalada
sem ninguém saber.** O Ptex é a tecnologia de produção para *«nem pensar em
UV»*, é **BSD-3**, e está nesta máquina com headers e `.so` ⇒ **lê-se, porta-se e
liga-se, com atribuição** — sem parede, sem clean-room, sem subagentes.
⛔ Ele **não** consta do arsenal (`docs/_ComoInvestigarApps/01`), e a razão é a
lição do MyPaint: *a porta sem interface pode ser a BIBLIOTECA e não o binário*.

### ⛔ O que o Ptex NÃO resolve, e é preciso dizê-lo

Ele elimina o UV e as ilhas — **e não elimina a fronteira**: passa a haver uma por
face. O que ele traz é a **adjacência explícita** para a atravessar. Ou seja:

* como **DESTINO** de uma projecção de ecrã: ⭐ excelente, e sem costuras visíveis;
* como **SUPERFÍCIE onde o Painter pinta directamente**: ⛔ pior que o atlas — os
  7 modos de vizinhança passariam a cruzar uma fronteira a cada punhado de
  texels.

⇒ *o Ptex é uma resposta para onde GRAVAR, nunca para onde PINTAR.*

### A recomendação, corrigida

**A §8 fica de pé na ordem e muda no conteúdo da W1 e da W3:**

| wave | antes (§8) | agora |
|---|---|---|
| **W1** | *«um botão Unwrap»* | ⛔ **sem botão**: a peça ganha a parametrização **quando ganha a textura**, e o artista nunca lê a palavra UV |
| **W2** | a textura + o shader | igual — ⭐ e é aqui que se escolhe o destino (atlas · Ptex · por-vértice), com a §9 medida primeiro |
| **W3** | cursor → `(u,v)` → `CanvasPointer` | ⭐⭐ **cursor → o ECRÃ**: o Painter pinta na imagem que já é rasterizada da malha, e o resultado é projectado ao largar. **Os 13 modos ficam perfeitos durante o traço** |

⚠️ **O preço honesto da projecção**, e ele é real:

1. **só se pinta o que se vê** — o que está de costas fica por pintar até rodar;
2. **dois traços de ângulos diferentes encontram-se no destino**, e é *ali* que a
   costura (do atlas) ou a fronteira (do Ptex) aparece — mas **entre** traços, não
   **dentro** de um;
3. o G-buffer precisa do plano de posição (acima);
4. ⛔ **e a projecção de um dab é `N` texels**: fazê-la por raio custa `0,436 µs`
   cada (medido na wave do `Scene Project`) ⇒ um dab de `100 px` de raio seriam
   `~13,7 ms`. *Por isso ela tem de sair do G-buffer, não de raios* — e isso
   ainda não foi medido.

### O que esta secção acrescenta ao §9 (o que falta MEDIR)

| pergunta | como se mede |
|---|---|
| o plano de posição custa quanto no `form_plane`? | acrescentar o alvo e re-correr o `measure_a_donation` |
| a projecção de um dab, a partir do G-buffer, custa quanto? | não existe ainda; mede-se com o plano acima |
| o Ptex serve de destino a **quatro** canais (cor · altura · cobertura · material)? | ele é multi-canal por desenho; ⭐ confirma-se **correndo-o**, que é legal (BSD) |
| quantos texels por face o Ptex pede para igualar `2048²` numa peça típica? | contar faces × resolução por face |
