# O RESTO DO APP AINDA ACHAVA QUE A ARTE É PLANA — o censo, e as três ferramentas curadas

**Linha:** `line/Vector` · **Data:** 2026-09-15 · **Merge-base:** `1d43da737`
**Antecessor:** [`HANDOFF_A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14`](HANDOFF_A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14.md)

---

## §1 — UMA LINHA

A wave de 14/09 curou **o pincel**. O censo do dia seguinte mostrou que **o resto do app** continuava
a resolver o ponteiro pelo afim do **quad de repouso**: o conta-gotas apanhava a cor do texel errado,
o editor de curva punha as alças longe da tinta, e as três entradas de canvas da Remoção de fundo
faziam algo **pior que o afim** — uma caixa alinhada aos eixos, cega à rotação, ao pai e à malha.
As três estão curadas, cada uma por uma PORTA, e a tinta da máscara saiu do Vello para o passe de
sprites porque o caminho por recortes está **medido e refutado**.

---

## §2 — A LEI QUE ESTA WAVE PAGOU

> ⛔⛔ **Um controlo DESENHADO por um mapa e AGARRADO por outro é um controlo morto sob o dedo** — e
> a wave anterior fabricou três deles ao curar só metade do par.

Curar o ponteiro sem curar o chrome não é «metade do trabalho»: é uma **regressão**. Antes, as duas
metades concordavam **por acidente** (as duas liam o quad de repouso); depois, a que lê a malha
aponta para um sítio e a que lê o quad desenha noutro. ⇒ *as duas direcções viajam juntas ou nenhuma
viaja*, e é por isso que esta wave tem sempre um par: a porta **inversa** (quem aponta) e a porta
**directa** (quem desenha).

⚠️ **E a wave anterior gateou a porta que curou, não a PERGUNTA.** O
`architecture_the_canvas_pointer_asks_the_mesh` afirma que o Painter consulta a malha; nenhuma sonda
perguntava *«quem MAIS resolve um ponteiro de canvas?»*. O gate desta wave é de **FAMÍLIA**, com
piso de população e com a lei antiga **proibida pelo nome**.

---

## §3 — A PORTA QUE FALTAVA (substrato)

| porta | direcção | quem consome |
|---|---|---|
| [`ph2d_render::mesh_uv`] (14/09) | ecrã → texel | o pincel, o conta-gotas, o menu de alça, as 3 entradas da Remoção de fundo |
| **[`ph2d_render::drawn_mesh_of`] + [`DrawnMesh::world_at_uv`]** | **texel → ecrã** | o `CanvasMap` do chrome |
| **[`ph2d_render::drawn_instance_of`]** | a instância + a malha | a tinta da máscara |

⭐ As duas novas são **read-only** (`&World`) de propósito: quem desenha o quadro já tem o mundo de
apresentação emprestado, e um `query::<…>()` por chamada **aloca** as tabelas de arquétipo dele (a
nota do `PresentWorld::sweep` mediu `107` blocos / 10 quadros por isso). Atravessa-se o mundo uma
vez, guarda-se a malha **emprestada**, e cada ponto é uma pergunta sem alocação.

⭐ **Na direcção directa NÃO há ambiguidade de dobra, e é por construção:** a malha de repouso é uma
**partição** da imagem (dois triângulos não partilham texel). É a pergunta inversa que tem de
escolher o triângulo desenhado **por último**.

⚠️ **O custo por ponto é o número de TRIÂNGULOS** (varrimento linear, como a inversa). O consumidor
de hoje pergunta por dezenas de pontos enquanto uma curva está a ser editada. Se aparecer um
consumidor de MILHARES, a cura é um **índice por UV** construído dentro da porta — ⛔ nunca uma
segunda cópia desta álgebra.

---

## §4 — O QUE FOI CURADO, um a um

### 4.1 O conta-gotas (item 1)

`forwarding.rs::painter_eyedropper_sample` passa pela `mesh_uv` com pegada `[0, 0]` — *uma porta que
APONTA pergunta por um PONTO; a pegada só tem sentido para quem vai pousar um disco de tinta*. E a
**recusa** é tratada: um clique fora da arte desenhada cai para a leitura do ecrã, nunca para o quad
de repouso, que não está lá.

### 4.2 A curva e a linha (item 2) — as DUAS metades

- **O olho:** [`ph2d_app_painter::canvas_map::CanvasMap`] — imagem-px → ecrã, pela malha onde ela
  desenha e pelo afim onde não há malha (é ele que carrega a grelha da folha desdobrada). O ladrilho
  do *Repeat Image* é um `deslocado(dx, dy)` que entra nas **duas** metades (o afim *e* a projecção
  da malha) — metade dele desenharia as alças da malha por cima do ladrilho central.
- **O dedo:** o menu de alça (botão secundário) passa pela `malha_sob_o_cursor`, que é a porta que o
  botão **primário** já usava.
- Os **dois** desenhadores do gizmo de transformação recebem o mapa. ⚠️ A **escala** continua a sair
  do afim, de propósito: uma tolerância em px de imagem é global ao gesto, e o hit-test que ela
  espelha (`shape_grab_tol_from_affine`) lê o mesmo afim. Derivá-la da malha faria a alça mudar de
  raio ao passar por uma dobra.
- O `present` (**read-only**) atravessa `painter_bridge::dispatch → draw_overlays → os dois
  overlays`. Como a porta directa é read-only por desenho, **não há um `&mut` a atravessar o quadro**.

⛔ **O que o mapa NÃO faz: SUBDIVIDIR.** Um ponto atravessa exacto; um segmento é desenhado recto, e
sobre uma dobra o recto certo seria partido nas arestas dos triângulos. Para a **espinha** da curva
isso é inofensivo **por construção** — ela já chega achatada em muitos pontos, pela mesma
`flatten_spine` que a tinta percorre. Para a **caixa** do gizmo é aproximação **declarada**: quatro
cantos mapeados, arestas rectas.

### 4.3 A Remoção de fundo (item 3) — o ponteiro E a tinta

**O ponteiro.** As três entradas (conta-gotas, dab de protecção, *Add area*) montavam **cada uma** a
sua caixa `translation ± size/2` a partir da pose **LOCAL**. Isso ignora três coisas que o desenho
honra: a **rotação**, a pose do **PAI** (o defeito que o Painter pagou em 19/08) e a **MALHA**.
⇒ uma porta, [`input_dispatch::uv_sob_o_ponteiro`], com **três** estados:

| estado | o chamador faz |
|---|---|
| `Uv(u, v)` | a UV **não cortada** — quem quer saber se caiu dentro pergunta `(0.0..=1.0)` |
| `ForaDaArte` | consome o clique e não faz nada (o que a caixa já fazia com um ponto de fora) |
| `SemSujeito` | **recusa** o clique: ele cai para o pick / o arrasto da cena |

⛔ Colapsar os dois últimos num `Option` trocaria, **em silêncio**, quem fica com o botão do rato.

⚠️ **As dimensões em px CANCELAM-SE** e por isso não entram na assinatura (`img / iw` é a fracção).
A porta passa `1 × 1` e diz isso: *um número inventado ali leria-se como «a resolução importa»*.

**A tinta.** Ela era um `draw_image_rgba_transformed` com o afim. Hoje é **uma instância do passe de
sprites** com a **mesma malha** da arte (`upload_tint` + `tint_instances`, ranhura de GPU própria,
pré-multiplicada pela mesma razão da prévia). ⚠️ O `sub_order` é o que a põe **por cima** — a chave
de ordenação desempata por `texture_id`, e o da ranhura da tinta tanto pode ser maior como menor que
o da arte; empatar em tudo e confiar na ordem de inserção deixaria a tinta a desaparecer **por
baixo** da arte conforme a ordem em que as ranhuras foram pedidas.

---

## §5 — ⛔ RECUSA MEDIDA: a tinta por recortes do Vello

Sonda `ph2d-render/tests/it/skin_pieces_gpu_cost.rs :: measure_seams_against_clip_dilation`, corrida
em 2026-09-15 (px fora da barra de 8 de alfa, sobre um miolo de `484 880` px):

| arte | peças | dilatação | px fora | pior desvio |
|---|---:|---:|---:|---:|
| translúcida | 216 | `0,00` | `10 580` | `20` |
| translúcida | 216 | `0,50` | `40 050` | `111` |
| translúcida | 3 456 | `0,00` | `41 732` | `20` |
| translúcida | 3 456 | `1,00` | `210 814` | `124` |
| opaca | 216 | `0,50` | **`0`** | **`0`** |

⭐ **A cura barata das costuras — dilatar os recortes — funciona na arte OPACA e PIORA a
TRANSLÚCIDA**, porque a faixa sobreposta se compõe duas vezes. E a tinta é translúcida por natureza.
⛔ Acima disso, os buffers do Vello são de tamanho **FIXO** e o que os estoura degrada **em
silêncio** — que foi exactamente o *«Smooth bugado quebrando a forma»* que este módulo já pagou.
⇒ o passe de sprites, onde **não há costura por construção** (a regra de canto do rasterizador dá
cada centro de pixel a UM triângulo).

---

## §6 — SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **A prévia da Remoção de fundo já estava CERTA.** Ela vai pelo `PreviewOverride`, que só troca a
   textura da **mesma** instância ⇒ herda a malha. O que estava plano era a **tinta** que a anota (e
   o anel do pincel). *A minha própria nota ao dono dizia «a amostra e a pré-visualização» — metade
   disso estava errado, e só olhar para o `sim_extract` o mostrou.*
2. **A escala do gizmo NÃO passou a sair da malha, e isso é a decisão.** Uma tolerância de agarrar é
   global ao gesto e o hit-test dela lê o afim; derivá-la da malha faria a alça mudar de raio sobre
   uma dobra.
3. **O `CanvasMap` não substitui o afim — ele contém-no.** Sobre um quad a resposta é o afim **ao
   bit**, e há gate a afirmá-lo. É isso que mantém a grelha da folha e o *Repeat Image* intocados.
4. **O `1 × 1` da `uv_sob_o_ponteiro` não é um valor por defeito** — é a unidade em que a resposta é
   dada, e as dimensões cancelam-se na conta.
5. **O `present` que atravessa o `painter_bridge` é READ-ONLY** (`&World`). Não há um `&mut` do mundo
   de apresentação a viajar pelo quadro do Painter.
6. **O corte do `picking_tests.rs` não é arrumação:** ele estourou o tecto de LOC com os testes
   novos, e a cura foi corte **por responsabilidade** (`picking_doors_tests.rs` = as portas de
   canvas), nunca uma isenção.

---

## §7 — TRÊS premissas minhas que a medição derrubou

1. *«As guias são todas outra natureza — precisam de subdivisão»*. **Falso para os PONTOS.** Uma
   alça é um ponto e atravessa a malha exactamente; só os **segmentos** entre pontos pedem
   subdivisão, e a espinha da curva já chega achatada.
2. *«A tinta da máscara é cosmética, pode ficar para a wave das guias»*. **Falso:** ela é a metade
   que torna o ponteiro curado numa REGRESSÃO. Ou as duas, ou nenhuma.
3. *«O caminho do Vello por triângulo é aceitável para um HINT translúcido»*. **Refutado pela sonda
   que já existia** (§5) — e a cura barata das costuras piora exactamente o caso translúcido.

---

## §7-bis — O que o PORTÃO DE FECHO apanhou (e nenhum `check` via)

Quatro vermelhos, **todos** crescimento desta wave, e **nenhum** num ficheiro que ela editou por
assunto:

| gate | o que ele viu | cura |
|---|---|---|
| `the_app_does_not_grow_past_its_numbered_ceiling` | `App` a `189` campos contra `187` | ⭐ a Remoção de fundo ganha **CASA** |
| `shell_files_respect_hr18_loc_cap` | `app_state.rs` `1031` · `main.rs` `1120` | idem (a struct e o inicializador encolhem) |
| `shell_functions_respect_their_ceiling` | `main.rs::new` `239` · `bgremoval_preview.rs::dispatch` `206` | idem + **o que o quadro DESENHA NO CANVAS** sai para uma porta |
| `architecture_every_pointing_tool_asks_the_mesh` (**meu**) | a agulha pedia `pub(super) fn tint_instances(` | a agulha nomeia a **LEI** |

⭐⭐ **A catraca dos campos diz a cura por escrito** — *«um campo novo tem DONO: ponha-o no estado da
família do assunto dele»* — e aqui a família **já existia**: eram **cinco** campos soltos com o mesmo
prefixo (`bgremoval_*` + `last_bgremoval_pushed_entity`) e nenhuma casa. ⇒
[`bgremoval_shell::BgremovalShell`], `189 → 185` campos, e os **três** tectos descem com o corte
(`187 → 185`, `1019 → 997`, `237 → 235`). *Uma catraca que não desce vira licença.*

⚠️⚠️ **E o quarto é meu, e é a QUINTA vez que este repo paga a mesma forma:** a agulha do gate
nomeava a **VISIBILIDADE**. Cortar o `dispatch` por responsabilidade tornou a função privada — **sem
uma linha de comportamento mudar** — e o gate ficou vermelho no mesmo dia em que nasceu. *Um `pub`
não é uma propriedade do produto.*

---

## §7-ter — ⛔⛔⛔ O REPORT QUE DESMENTIU METADE DESTA WAVE

> *«O color picker não funciona de maneira nenhuma e em nenhum lugar, com a arte dobrada ou não.
> Sempre fica com a cor resultante `#00000000`.»* — o dono, no smoke do mesmo dia.

⭐⭐⭐ **Ele está certo, e o defeito NÃO é o que a §4.1 curou: eu curei *ONDE* o conta-gotas amostra
e nunca medi *SE* ele amostra.** São dois defeitos, e o censo da §4 só mediu o primeiro — porque a
pergunta que ele fazia era *«quem resolve o ponteiro pelo quad de repouso?»*, e um consumidor que
lê a **camada errada** responde a essa pergunta **correctamente**.

**O mecanismo.** O quadro deste app tem **duas** metades, em texturas diferentes:

| metade | textura | o que está lá |
|---|---|---|
| o MUNDO | a saída do `Tonemap` — **ou o `WorldRt`** num quadro intercalado / com o vidro do *Edit Prefab* | sprites, imagens, a prévia do Painter |
| o CHROME | a intermédia do `VelloPass` | os painéis **e a arte VECTORIAL do documento** |

A leitura de reserva do conta-gotas pedia **só a segunda**. Sobre o canvas ela é transparente **por
construção** — e transparente é literalmente `#00000000`. ⚠️ **O patch que existia cobria UM canto**
(o Painter activo sobre a sprite seleccionada, a amostrar a composição de camadas dele); em todo o
resto — outra ferramenta, outro painel de cor, o fundo do canvas — o artista recebia transparente.
*Um patch num canto lê-se como «funciona», e o report que o desmente chega meses depois.*

⇒ [`ph2d_render::screen_pick`]: lê UM texel de cada metade e **compõe as duas** com a lei do próprio
`compositor.wgsl`.

**Quatro coisas que só a construção revelou:**

1. ⭐⭐ **A lei da composição vivia DENTRO de `#[cfg(test)]`** (`composite_straight`), e por isso o
   único consumidor que precisava dela em produção não a podia chamar. *Uma lei que só existe para o
   teste é uma lei que o produto não tem.*
2. ⚠️ **A composição pode ser feita em BYTES** — o shader re-codifica o mundo para sRGB antes de
   misturar porque os bytes do Vello já são valores de designer (sRGB, alfa **directo**), e a saída
   do tonemap é `*Srgb`. ⇒ os dois lados já estão no mesmo espaço.
3. ⚠️ **A fase que faltava é desfazer `B G R A`** — metade desta porta, e a metade que uma leitura
   distraída do `read_pixel` vizinho (que é `RGBA`) não faz.
4. ⚠️⚠️ **A fonte do MUNDO tem DOIS modos.** Num quadro intercalado (ou com o vidro) o mundo que o
   ecrã mostra é o **acumulador**, não o tonemap — ler sempre o segundo devolveria a **última
   faixa**: certo no documento simples, errado em metade dos outros. ⇒ a escolha é UMA porta
   (`world_source`), a mesma pergunta que o compositor faz, e o `WorldRt` ganha `COPY_SRC` por isso.

⚠️ **A metade AUTORADA do Painter FICA, e não é redundância:** o ecrã passa pelo tonemap e pelo
dither da descida, logo escolher uma cor acabada de pintar e recebê-la com `±1` por canal não
fecharia o round-trip.

⭐ E o `VelloPass::read_pixel` ficou com **zero** chamadores e foi **apagado** — a nota de espaço de
cor dele viajou para a porta nova.

---

## §7-quater — E o report FECHOU pela MEDIÇÃO, não por mais uma leitura

O dono voltou com *«ainda não funciona para a arte dobrada pelo osso»*. Li o caminho inteiro **três
vezes** e concluí, de cada vez, que ele devia funcionar. ⇒ **é aí que se para de raciocinar.**

⛔ **Aquele caminho não é diagnosticável por leitura:** ele atravessa **duas** fontes (a composição
AUTORADA do Painter e a leitura do ECRÃ) e o ecrã tem **dois** modos (o mundo vem da saída do
tonemap, ou do **acumulador** num quadro intercalado). As quatro combinações falham de maneira
diferente e **em silêncio** — e a cena dos ossos cai justamente no modo intercalado (barras
vectoriais + a imagem presa), que é o único sítio onde a arte dobrada é a **única** coisa a viver na
camada do mundo. *Um report de «não funciona» sobre uma escolha com quatro combinações mudas custa
menos a medir do que a ler.*

⇒ `PH2D_PICK_LOG=1`, uma linha por escolha. A corrida do dono:

```
[pick] (1054, 318) sel=Some(4294967283) painel=false fonte-do-mundo=acumulador
[pick]   AUTORADA (Painter) = None
[pick]   DO ECRA            = Some([156, 99, 57, 255])
[pick]   texel mundo (BGRA) = Some([57, 99, 156, 255])   texel chrome (RGBA) = Some([0, 0, 0, 0])
```

⭐ **Tudo o que a porta decide, confirmado de uma vez:** a fonte do mundo é o **acumulador** (o modo
que a §7-ter previu), o chrome ali é `0,0,0,0` (não há nada por cima do canvas), os canais são
desfeitos correctamente (`B G R = 57, 99, 156` ⇒ `R G B = 156, 99, 57`) e a alfa sai opaca. O
caminho autorado **não disparou** — a resposta veio do ecrã, que é o que esta wave construiu.

⚠️ **E o log trouxe um vizinho por medir:** a linha `[hero] unhandled event: Click(NodeId(…))` que o
precede. Um `Click` que ninguém consome é a forma do **controlo morto** do `CLAUDE.md` §5.0 — não é
desta wave, e fica **nomeado** em vez de suposto benigno.

---

## §8 — O que fica ABERTO

- ⏳ **As guias que são CAMINHO** — a grelha, os contornos de selecção, os selos de operação, o véu
  de humidade e o gizmo de deformação. Elas precisam de **subdivisão** (um segmento recto sobre uma
  dobra não é uma recta), e a porta directa já existe para os **vértices** delas.
- ⏳ **O anel do pincel da Remoção de fundo** deriva o raio do afim do quad: sobre uma dobra a escala
  local é outra. Hoje é um círculo no cursor, e a cura pede a `warp` da malha ali.
- ⏳ **O custo da porta directa é linear nos triângulos** — nomeado no doc dela, com a cura (índice
  por UV) escrita e não construída, porque nenhum consumidor de hoje a alcança.
- ✅ **O `Click` sem consumidor FECHOU em 2026-09-15, e NÃO era um controlo morto** — ver a §10.
- ⏳ Os itens que já estavam abertos: **F8 — Bendy Bones** (⭐ **o 1.º passo, o ALCANCE, está MEDIDO
  — ver a célula F8 da [fila](../01_a_fila.md): a lei da pele já é de N ossos e não muda, o produtor
  é UM só sítio, e dos quatro consumidores só o desenho e o dedo precisam da curva**) · **F4 — *«undo tem poucos passos»*** (não
  reproduz) · a malha amostrada no EVENTO e não por dab · os traços de FORMA com uma dobra só · o
  `apply_jitter` com o raio inflado.

---

## §9 — O ITEM 4 (as guias chatas) foi MEDIDO, e ele parte-se em DUAS espécies

⭐⭐⭐ **O censo é a tabela dos consumidores do afim `imagem-px → ecrã`**, e ele parte a lista do dono
em duas metades cuja cura **não é a mesma**:

| espécie | quem | o que ela desenha | a cura |
|---|---|---|---|
| **A — GEOMETRIA** | a **grelha** · os **selos de operação** · o **gizmo de selecção** | linhas e contornos, ponto a ponto | a porta `CanvasMap`, mais **SUBDIVISÃO** |
| **B — IMAGEM** | a **selecção** (formigas + hachura) · a **humidade** | um blit de RGBA do tamanho do canvas, por UM afim | ⛔ a porta **não serve** — é preciso o passe de sprites |

### A espécie A está FECHADA (2026-09-15)

O `CanvasMap` ganhou [`segment`] e [`polyline`], e as três peças passaram por elas. A lei do número
de pedaços **não é nova** — é a do `ph2d_poly2d::refine` (`√(d/tol)` com **uma** conferência), e a
tolerância é o mesmo `0,5 px` daquele ficheiro.

⭐⭐ **O achado é que a lei do desvio se AUTO-LIMITA:** ela pediu `15`–`35` pedaços por linha em todas
as malhas medidas, e o tecto nunca mordeu. Quem cresce com a malha grossa é o número de pedaços;
quem cresce com a malha fina é o preço de cada um — *os dois puxam em sentidos opostos*.

| malha | pedaços | ms (40 linhas) | % de um quadro |
|---:|---:|---:|---:|
| 32 tris | 1 400 | 0,136 | 0,8 % |
| 128 tris | 1 040 | 0,321 | 1,9 % |
| 512 tris | 720 | 1,035 | 6,2 % |
| 1 152 tris | 600 | 2,070 | 12,4 % |

⏳ E a tabela nomeia a obra seguinte: o custo é **`pedaços × triângulos`** porque a travessia é uma
varredura LINEAR — um índice de UV na `DrawnMesh` tornaria cada pedaço `O(1)` e a tabela ficaria
plana. Não foi construído porque nenhum número dela o exige ainda.

### E a espécie A ganhou CENA (ordem do dono: *«monte uma cena»*)

`PH2D_VEC_BONE_PAINT_SMOKE=1` — um canvas do Painter **preso a ossos e dobrado**. Ela existe porque a
cura estava gateada e **invisível**: nenhuma cena punha o Painter a pintar sobre arte dobrada.

⭐⭐⭐ **E corrê-la achou DOIS defeitos que todos os portões verdes não viam** (DIRETIVA §1: *o verde
de compilação vale ZERO no audit*):

1. **O bind falhava em silêncio.** A leitura dos pixels foi copiada da cena irmã, que lê
   `SpritePixels` — e uma sprite que vive numa **célula do atlas** não carrega esse carimbo (ele é do
   caminho `Individual`, de textura própria). Os bytes vivem no `AssetDb` com o vínculo no
   `atlas_asset_map`. *Copiar a leitura de uma cena irmã leu o componente errado e devolveu `None`
   sem um erro* — e só a linha que a cena **imprime** o disse.
2. **A cena ficava acima do orçamento de peças do quadro.** Um canvas **opaco** é coberto de malha de
   ponta a ponta, logo o tamanho dele **É** a densidade da pele:

   | lado | peças | orçamento |
   |---:|---:|---:|
   | 1024 | 3 690 | 1 543 ⇒ acima |
   | 640 | 1 620 | 1 543 ⇒ acima |
   | **512** | **1 144** | dentro |

   ⛔ A alavanca é o TAMANHO, nunca a grelha: baixar o `GridOptions` armaria a cena com números que o
   produto não usa, e uma cena acima do orçamento ensina ao dono que o app engasga **sobre uma
   fixtura escolhida por mim**.

⚠️ **E a ORDEM da cena é load-bearing, com gate e prova de mutação:** ela prende **antes** de dobrar.
O repouso de uma pele é o instante do bind — dobrada primeiro, esta pose seria o repouso, o canvas
sairia recto, e a cena imprimiria a linha de sucesso sem provar nada. *Um smoke que monta e não
demonstra é pior que um ausente: ele é acreditado.*

### A espécie B está ABERTA, e a rota dela já está DECIDIDA por uma recusa medida

As duas chamam `draw_image_rgba_transformed(&rgba, w, h, afim, …)`: **uma imagem por um afim**. Há
exactamente dois caminhos para a pôr sobre uma malha, e **um deles já foi medido e recusado nesta
mesma jornada**:

1. ⛔ **Um `push_clip` + afim por triângulo (Vello).** RECUSADO com números — arte translúcida, `216`
   peças: `10 580 px` fora da barra de 8 de alfa (pior `20`); `3 456` peças: `41 732`. E a contagem
   de recortes do Vello degrada **em silêncio**, que é o *«Smooth bugado quebrando a forma»* que este
   módulo já pagou.
2. ⭐ **O passe de SPRITES, com a malha** — a rota que o tinte da Remoção de fundo já usa
   (`drawn_instance_of` → copiar a instância → trocar a textura → `LiftedInstances::push(inst, malha)`).
   São ~20 linhas de rota **mais** o ciclo de vida de uma textura de GPU por overlay.

⇒ **a obra é a 2, e o que ela custa não é a rota: é a TEXTURA.** Cada overlay precisa de subir o
RGBA quando ele muda e de a largar quando some — o que o `bgremoval_preview_gpu` faz em `upload_tint`
/ `release_preview_texture`, e que a família do Painter ainda não tem.

⚠️ **E ela muda o Z relativo:** hoje as duas vivem na camada de chrome (Vello) e passariam a viver no
passe do mundo. Medido no papel: elas já são desenhadas **antes** das alças, logo a ordem relativa
que o artista vê não muda — mas isso é uma leitura de código e **não um smoke**, e é a primeira coisa
que a próxima janela tem de confirmar com o dono a olhar.

⚠️ **E a espécie B não é igualmente urgente nas duas peças:** as **formigas** traçam uma FRONTEIRA (um
erro ali lê-se na hora, como o conta-gotas se lia); o **véu de humidade** é um campo borrado
desenhado em `ImageQuality::Low`, onde o mesmo erro é quase invisível. *Se a rota tiver de ser paga
uma vez, ela paga-se pela selecção.*

---

## §10 — ⭐⭐⭐ O `Click` SEM CONSUMIDOR: o id foi INVERTIDO, e a resposta inverteu o diagnóstico

A linha do log do dono, verbatim:

```
[hero] unhandled event: Click(NodeId(3001329642747827011))
[pick] (1054, 318) sel=Some(4294967283) painel=false fonte-do-mundo=acumulador
```

⚠️ **Ela vem LOGO ANTES de uma amostragem que FUNCIONOU**, e foi essa adjacência que a manteve
*«nomeada e por medir»* durante duas waves: parecia ruído de um sítio onde tudo dava certo.

### Como se soube qual era

O `NodeId` é um **FNV-1a de uma string**, e o repo declara-as todas por `hash_node_id("…")`. ⇒ o id
inverte-se **pela população**: `3 431` literais distintos varridos, um bate — **`blender_eyedropper`**,
o botão do próprio conta-gotas que ele estava a testar. *Um hash não se adivinha; conta-se o
domínio dele.*

### E o diagnóstico inverteu-se

O botão faz **todo** o trabalho no `Down` — e tem de ser ali, porque o `Down` **seguinte** é a
colheita. ⇒ o `Click` que o `Up` produz chega depois de tudo feito, ninguém o consome, e o detector
de costura morta da shell grita.

⭐⭐ **Não é um controlo morto: é o DETECTOR a gritar lobo.** E o preço está medido — *um detector
que grita lobo treina toda a gente, e a próxima LLM, a ignorá-lo*, que é exactamente o que
aconteceu.

### As três decisões da cura

| decisão | porquê |
|---|---|
| A isenção é da **FAMÍLIA**, não de um id | o conta-gotas foi só o que ele carregou; o `Close`, a barra de arrastar, o selector de paletas e as alças de tamanho produzem a MESMA linha com outro número |
| A **ROTA ganhou gate ANTES** da isenção | ⛔ *silenciar um diagnóstico sem prova por trás é armengo* — hoje há gate a medir que o `Down` arma, que o seguinte colhe, que o 2.º clique cancela e que o `Up` **não** desarma |
| A lei **mudou de casa** para a `ph2d-editor-core` | cada linha do `expected_unhandled` é um facto sobre o DESPACHO (quem lê por polling, quem age no `Down`); a shell não sabe nenhum deles — ela só decide imprimir |

⚠️⚠️ **E a mudança de casa deixou DUAS mutações a sobreviver**, apanhadas na prova: os gates ficaram
a medir a porta de BAIXO (`work_already_done_on_down`) e ninguém exercitava o `unhandled_is_expected`
com um `Click`, logo alargar a isenção ao TIPO inteiro — ou apagá-la — deixava a suíte verde.
*Mover uma lei sem mover a régua que a ATRAVESSA deixa um gate a medir a metade de baixo.*

---

## §11 — ⛔⛔⛔ A CENA FOI REPROVADA («ruim», com foto) E A CAUSA ERA A MINHA RÉGUA

O dono correu a cena do §9 e devolveu uma foto com duas setas: o traço preto que ele pintou saía
**RASGADO** — lascas destacadas do arco, e a própria silhueta branca do canvas com um estilhaço
triangular fora do sítio.

### O que estava errado

O canvas era **quadrado** (`512²`) com os três ossos deitados ao meio dele. O raio de um osso é o
**comprimento dele** vezes a `strength` ([`SkinBone::new`]), e um ponto fora do raio de TODO osso é
**órfão**: ele salta, em salto seco, para o osso mais próximo (o *point binding* do Moho, que o
`weights_at` documenta). **Um salto seco num mapa contínuo é um rasgo.**

⛔⛔⛔ **E a aritmética proíbe o quadrado — afinar o ângulo nunca ia curar.** Com `n` ossos deitados
ao longo da largura `W`, cada osso mede `W/n` e portanto alcança `W/n`; a arte sobe `W/2` acima do
eixo. `W/2 < W/n` só é verdade com **`n < 2`**. ⇒ *um quadrado com uma corrente de três ossos tem
banda órfã por construção, em qualquer ângulo e em qualquer tamanho.* Medido: **`33,85 %`** da arte.

### ⭐⭐⭐ O achado que vale mais que a cena: EU CITEI A COLUNA ERRADA

O doc-comment do `DOBRA_GRAUS` justificava os `25°` dizendo que estavam *«bem menos do que o ângulo
em que o mapa começa a dobrar sobre si mesmo (a régua da dobra vive na `ph2d_skeleton::fold`)»*.

**Era verdade, e era irrelevante.** A `fold::measure` devolve **quatro** colunas, e sobre a foto do
rasgo elas liam:

| coluna | leitura | o que ela mede |
|---|---:|---|
| `inverted` | **`0,00 %`** | arte DO AVESSO — o defeito que eu citei |
| `det_min` | `0,2622` | o pior ponto comprime, não inverte |
| **`orphan`** | **`33,85 %`** | arte **fora do alcance** — o defeito que estava no ecrã |

⚠️⚠️ **A coluna que gritava é aquela que o doc da própria régua chama de ANTI-VACUIDADE** — ela está
lá para impedir que uma «cura» que deixe a arte toda órfã se leia perfeita —, e eu li-a como
escrituração em vez de como diagnóstico. *Uma régua com N colunas responde a N perguntas, e citar a
errada devolve `0,00 %` sobre o defeito que está na foto.*

⇒ é a mesma família de
[`feedback_a_leak_ruler_masked_by_the_products_own_predicate_hides_the_leak`] e de *«uma medição
sobre o eixo que não dobra mede o caso que não existe»* (§9): **a leitura estava correcta e a
pergunta não era aquela.**

### A cura, e a cura ÓBVIA que foi medida e é PIOR

A altura do canvas passa a ser **DERIVADA** do alcance de um osso
(`ALTURA_PX = 2·LARGURA_PX·15 / (OSSOS·16)` ⇒ `512×320`), e não dois literais independentes: mexer
na largura ou no número de ossos leva a altura atrás, em vez de empurrar a arte para fora do alcance
**em silêncio**.

| canvas (3 ossos, `strength` do produto, `25°`) | meia-altura | alcance | `det_min` | invertida | **ÓRFÃ** |
|---|---:|---:|---:|---:|---:|
| `512×512` — a redacção reprovada | 256 | 170,67 | 0,2622 | 0,00 % | **33,85 %** |
| `512×384` | 192 | 170,67 | 0,2573 | 0,00 % | **12,31 %** |
| `512×336` | 168 | 170,67 | 0,2114 | 0,00 % | 0,00 % |
| **`512×320`** — a cena de hoje | **160** | **170,67** | **0,2497** | **0,00 %** | **0,00 %** |
| `512×352` | 176 | 170,67 | 0,1241 | 0,00 % | 4,12 % |

⛔⛔ **Alargar o ALCANCE é pior pelo meio, e isso é contra-intuitivo.** No quadrado de `512²`:

| `strength` | `det_min` | invertida | órfã |
|---:|---:|---:|---:|
| `1.0` | `0,2622` | 0,00 % | 33,85 % |
| **`1.5`** | **`−1,2829`** | **0,32 %** | 3,08 % |
| `2.0` | `0,4750` | 0,00 % | 0,00 % |

*Um alcance que cobre metade da banda órfã mistura um osso que CHEGA com um vizinho que SALTA, e a
mistura inverte-se — chegar a meio é pior que não chegar.* ⇒ a alavanca da cena é a **FORMA da
arte**, nunca um knob (a mesma conclusão do `LADO_PX` no §9, por outro mecanismo).

### O gate que não existia

`nenhum_pedaco_da_arte_fica_fora_do_alcance_dos_ossos` monta a corrente pela **mesma** porta que a
cena (`super::eixos`), prende pelo `bind_image`, resolve pelo `skin_of` e mede pela `fold::measure`
— ⛔ **sem uma segunda cinemática escrita no ficheiro de teste**, que seria uma segunda resposta à
mesma pergunta.

⚠️ E ele vem **com o controlo dentro**: `o_canvas_quadrado_que_o_dono_reprovou_e_acusado_por_esta_
mesma_regua` pede o quadrado e exige `orphan > 30 %`. Sem essa metade, uma régua que respondesse
`0,00 %` a tudo aprovaria qualquer canvas.

⚠️ O terceiro (`a_meia_altura_cabe_no_alcance_de_um_osso`) lê a `strength` do **produto**
(`Bone::default()`), não um `1.0` escrito no teste: se o default do alcance mudar, é o gate que fica
vermelho e não o smoke do dono.

**Prova de mutação:** repor `ALTURA_PX = LARGURA_PX` ⇒ **dois** gates RED, com a mensagem a nomear o
número (`meia-altura 256 nao cabe no alcance 170,67`).

**Corrida:** `512×320`, **780 peças** de um orçamento de quadro de `1 543` (a redacção anterior
gastava `1 144`), `orphan 0,00 %`, `inverted 0,00 %`.

---

## §12 — ⛔⛔⛔ A FOTO TINHA DOIS DEFEITOS, E O SEGUNDO É QUE O CENSO DO ITEM 4 ERA UMA LISTA ESCRITA À MÃO

Na mesma foto, além do rasgo, estava o **contorno amarelo da ELIPSE**: um círculo perfeito por cima
de arte dobrada. ⚠️ **E é exactamente a forma que o passo 2 do meu próprio roteiro de smoke manda
arrastar** — *«escolha uma forma (Rectangle/Ellipse) e arraste uma sobre o canvas: o CONTORNO e a
CAIXA dela têm de seguir a curva»*.

### O que falhou

O gate `the_curve_and_line_chrome_is_painted_where_the_art_draws_it` abre com esta frase, escrita
por mim:

> ⚠️ **Este gate é uma FAMÍLIA, não um sítio** — é essa a forma que a wave anterior não tinha: ela
> gateou a porta que curou e nenhuma sonda perguntou *«quem MAIS resolve um ponteiro de canvas?»*.

⛔⛔ **E ele era uma LISTA ESCRITA À MÃO de seis ficheiros** (curva · linha · grelha · selos · gizmo
de selecção · gizmo). *Uma lista escrita à mão não tem população para ter piso* — a armadilha da
§2.7 do HOWTO um nível acima: lá um censo por prefixo passa a varrer zero e fica verde; aqui ele
nunca varreu nada, **recitava**.

Varrendo de facto os `painter_bridge*.rs`, eram mais **cinco** desenhadores de geometria a mapear
pelo afim do quad de repouso:

| desenhador | o que ele pinta | curado |
|---|---|---|
| `draw_ellipse_overlay` | o contorno + alças da ELIPSE — **o da foto** | ✅ `polyline(.., true)` |
| `draw_polygon_overlay` | o N-gono | ✅ `polyline(.., true)` |
| `draw_stencil_overlay` | a caixa do stencil | ✅ `polyline(.., true)` |
| `draw_symmetry_overlay` | as guias tracejadas | ✅ `polyline` por segmento |
| `draw_deform_gizmo` | a caixa do *Deform Transform* | ✅ `polyline(.., true)` |

⚠️ **A guia de simetria não era «mapear dois pontos»:** ela atravessa o canvas inteiro, logo é UM
segmento, e `move_to(map(a)); line_to(map(b))` consulta o mapa e **desenha a recta na mesma**. E ali
a consequência é mais dura que estética — *a guia diz ONDE o motor replica os traços, e o motor
replica em espaço de IMAGEM*: uma guia recta sobre arte dobrada aponta para um sítio onde nada é
espelhado.

### ⭐ Dois que NÃO são gaps, e a razão de cada um está registada

- **`painter_bridge_brush_ring.rs`** — o anel do pincel é **ancorado no cursor** e usa só a parte
  LINEAR do afim; a FORMA dele já carrega a curvatura da arte pela pegada do motor
  (`FootprintDeform::curve`, a wave de 14/09). *Uma cura melhor que este mapa, por outro mecanismo.*
- **A célula do Grid Stamp** (`draw_grid_cell`, no mesmo ficheiro) fica no quad **de propósito**, e é
  um item ABERTO com o bloqueador nomeado: a metade do DEDO (`grid_cell_under`) **inverte o afim**,
  e a porta que inverte pela MALHA (`ph2d_render::mesh_uv`) exige `&mut World` enquanto todo este
  caminho de chrome tem `&World` — a `DrawnMesh` só oferece `world_at_uv`. ⛔ Curar só o desenho
  poria um rectângulo bem dobrado à volta da célula **ERRADA**: *meia lei aplicada é pior que
  nenhuma.*
- E as duas da **espécie B** (as formigas da selecção, o véu de humidade) continuam abertas com a
  rota decidida — ver o §9.

### O gate que substitui a lista

`the_canvas_chrome_census_is_derived_and_nobody_maps_an_authored_point_by_the_rest_quad` varre
**todos** os `painter_bridge*.rs`, proíbe a lei antiga (`affine * Point::new`) pelo nome, e tem **os
dois pisos**: quantos ficheiros existem (`≥ 16`, são 18) e quantos de facto **consultam** a porta
(`≥ 8`). Os isentos são uma tabela `(ficheiro, razão)` **com metade de obsolescência** — um isento
que já não estoura tem de sair da lista.

⚠️⚠️ **E o gate apanhou um erro meu na 1.ª redacção:** a agulha era `CanvasMap::new(` e leu **`7`**
onde havia `8` — o `painter_bridge_gizmo.rs` **recebe** o mapa por parâmetro em vez de o construir.
*Uma agulha que nomeia o CONSTRUTOR mede quem monta, e a lei é sobre quem CONSULTA.*

---

## §13 — ⛔⛔⛔ «A MALHA DEFORMA A CURVA»: HÁ **DOIS** PRECIPÍCIOS, E O SEGUNDO MORA LOGO DENTRO DO PRIMEIRO

3.º report do dono sobre a mesma cena, com três setas — a primeira na **borda de cima** do canvas.
O mapa estava **perfeito**: `0,00 %` órfã, `0,00 %` do avesso, **`0` triângulos invertidos** de `780`.
E a arte saía com quinas na mesma.

### O mecanismo

O peso de um osso é o *bump* `(1 − x²)²` com `x = d/raio`, e os pesos são **NORMALIZADOS**. Junto
da borda do suporte todos os pesos tendem a zero, e **a razão entre números que tendem a zero varia
depressa** ⇒ a curvatura do campo explode. O mapa continua contínuo e injectivo; a malha, que pinta
**um afim por triângulo**, é que deixa de o conseguir seguir.

⇒ **dois precipícios:**

| | onde | o que se vê | régua que o apanha |
|---|---|---|---|
| 1.º — **órfão** | FORA do raio de todo osso | a arte **RASGA** | `fold::measure().orphan` |
| 2.º — **normalização** | **logo DENTRO** da borda do raio | a arte **FACETA** | `poly2d::deviation` |

⚠️⚠️ **E o `FOLGA_16 = 15/16` do §11 desviou-se do primeiro e estacionou no segundo:** eu escolhi a
altura para a arte usar `93,75 %` do alcance — o máximo que a folga permitia — que é exactamente
onde a normalização é mais dura. *Uma folga calculada contra UM precipício encosta ao outro.*

### A medição (512×320, dobra `25°`, grelha de fábrica, **`780` triângulos em todas as linhas**)

| `strength` | meia-altura / raio | órfã | invertida | **desvio da faceta** | `det_min` |
|---:|---:|---:|---:|---:|---:|
| `1,0` — a redacção reprovada | `0,938` | `0,00 %` | `0,00 %` | **`14,24 px`** | `0,2518` |
| `1,5` | `0,625` | `0,00 %` | `0,00 %` | `1,89 px` | `0,5846` |
| **`2,0`** — esta cena | **`0,469`** | `0,00 %` | `0,00 %` | **`0,92 px`** | `0,7475` |
| `3,0` | `0,312` | `0,00 %` | `0,00 %` | `0,59 px` | `0,8279` |

⭐ **`15×` melhor sem um triângulo a mais.** A cura é a `strength` **DERIVADA da arte**
(`forca_do_osso()`), não um literal ao lado dos outros: *a arte é o sujeito e o rig serve-a*.

⛔ **Isto não é armar a cena por baixo da mesa.** A `strength` é propriedade AUTORADA do osso, no
painel, e significa *«até onde este osso alcança»* — um rig cuja arte vive a `94 %` do alcance é um
rig mal autorado. ⏳ **Que o app não guie o artista para longe dessa borda é ITEM ABERTO** (ele podia
derivar um alcance de omissão da arte presa, ou avisar); o número está aqui.

### ⭐⭐⭐ E a medição expôs DOIS defeitos do PRODUTO, com a foto do dono como prova

**(a) O `Smooth` está estruturalmente DESLIGADO em quase toda arte real — e o doc dele afirmava o
contrário.** O `k` do `refine_posed` é **GLOBAL** e o tecto é `max_split = ⌊√(orçamento / peças)⌋`.
Com `SKIN_FRAME_PIECES = 1 543`, **toda malha acima de `385` triângulos só admite `k = 1`** ⇒ o
`Smooth` é o `Fast` ao bit. A nota do orçamento dizia *«dentro dele o `Smooth` refina só o que a
dobra pedir»* — **falso**, e está corrigida com a aritmética dentro.

Nesta cena: `780` peças, a régua pede `k = 6` (`28 080` peças, `18×` o orçamento). ⭐ **O adaptativo
— `k` por TRIÂNGULO — custa `3 034`, `9×` menos que o global**, e numa cena bem autorada cai para
**`1 059`, dentro do orçamento**. ⏳ Não está construído (a armadilha é a aresta pendente entre
vizinhos de `k` diferente, que é o que fechou a *quadtree* na 2.ª mídia). ⚠️ O `PH2D_BONE_LOG=1`
passa a **dizer quando o refinamento está desligado por esta aritmética** — antes *«não precisou»* e
*«não pôde»* imprimiam a mesma linha.

**(b) A graduação da grelha está ANTI-CORRELACIONADA com o erro.** Ela grada cada eixo pela
proximidade à ARTICULAÇÃO, que é um **ponto sobre o eixo dos ossos**. Medido, os cortes em `y` que
ela produziu nesta cena (eixo dos ossos em `y = 160`):

```
0(26) 26(26) 52(26) 78(26) 104(26) 130(22) 152(8) 160(10) 170(14) 184(20) 204(26) …
```

⇒ células de **8–14 px em cima do eixo**, onde o desvio é `1,66 px`, e de **26 px nas bordas**, onde
ele é `14,24 px`. *A graduação põe os triângulos onde o erro não está.* Medido pelo par
(triângulos, pior desvio), ela é **pior que não graduar**:

| grelha | triângulos | pior desvio | px por 1000 tri |
|---|---:|---:|---:|
| de fábrica (`f10 c26 r40`) | `952` | **`14,24 px`** | `15,0` |
| **uniforme 20** (sem graduação) | `918` | **`11,96 px`** | `13,0` |
| raio = raio do OSSO (`170`) | `1 920` | `8,45 px` | `4,4` |
| uniforme 10 | `3 328` | `6,80 px` | `2,0` |

⚠️ **E a convergência é `O(h)`, não `O(h²)`** (`26 → 13` dá `1,72×`, não `4×`) — *o que confirma que
não é erro de discretização de um campo liso: é a curvatura da normalização*. ⇒ densidade sozinha
nunca lá chega, e a cura de fundo é a graduação passar a seguir **o gradiente do peso e o braço de
alavanca**, não a distância ao ponto da junta. ⏳ Wave própria, com o olho do dono — mexer na
graduação muda toda arte já presa.

### Os gates

| gate | o que afirma |
|---|---|
| `a_arte_nao_sai_facetada` | o desvio da malha ao campo `≤ 1,5 px` de ecrã (a cena dá `0,92`) |
| `nenhum_pedaco_da_arte_fica_fora_do_alcance_dos_ossos` | `orphan == 0` e `inverted == 0` |
| `as_duas_redaccoes_reprovadas_continuam_a_ser_acusadas` | o CONTROLO dos dois: o quadrado lê `> 30 %` de órfãs **e** a tira na borda lê `> 10 px` de faceta **com o mapa limpo** (`0/0`) — *é essa cláusula que guarda a lição: um mapa perfeito pode facetar* |
| `a_arte_vive_longe_da_borda_do_alcance` | a derivação aterra na fracção MEDIDA |

⚠️⚠️ **E a 1.ª redacção dos gates tinha uma mutação SOBREVIVENTE:** pôr `strength = 1.0` na `build`
deixava os seis verdes, porque o arnês **montava a corrente ele próprio e escrevia a força ele
próprio** — ele media uma *reconstrução* da cena. ⇒ a criação passou a ter uma porta única
(`corrente()`), por onde o produto **e** os gates passam; com ela a mesma mutação mata o gate da
faceta com o número na mensagem (`14,24 px`). *A lei escrita em dois sítios prova-se num e ship-a no
outro.*

---

## §14 — *«bem melhor. deformou um pouco»* — o que SOBRA é a lei, e tem número

4.º report, sobre a cena do §13: a faceta foi. O que o dono nota agora é a **circunferência sair
ligeiramente oval**, e a pergunta é se isso é defeito ou é a arte a dobrar.

**É a arte.** Medido no campo desta cena:

| | esticão local (razão dos valores singulares) |
|---|---:|
| no **centro** do canvas | **`1,017`** — praticamente rígido |
| nas juntas | `1,109`–`1,190` |
| nas pontas/bordas | `1,411` (o máximo) |

Uma circunferência de `r = 128 px` desenhada ao centro **amostra** essa variação (o topo dela está a
`y = 32`, a base a `y = 288`) e sai **`14,9 %`** fora de redondo. ⇒ *o esticão pontual é quase nulo
onde ela é desenhada; o que a deforma é ela ser GRANDE em relação à dobra.*

### A varredura do ângulo — a deformação é quase LINEAR nele, e a faceta acompanha

| graus por junta | faceta | esticão máx | **círculo fora de redondo** | `det_min` |
|---:|---:|---:|---:|---:|
| **`25`** — esta cena | `0,92 px` | `1,411` | **`14,9 %`** | `0,7475` |
| `20` | `0,75 px` | `1,321` | `11,6 %` | `0,8079` |
| `15` | `0,57 px` | `1,234` | `8,5 %` | `0,8644` |
| `10` | `0,39 px` | `1,151` | `5,5 %` | `0,9160` |
| `6` | `0,23 px` | `1,088` | `3,3 %` | `0,9529` |

⛔⛔ **E baixar o ângulo seria DISFARCE, não cura** — a cena existe para mostrar que a arte dobra e
que as guias a seguem; uma cena afinada para minimizar o fenómeno que demonstra é a *«cena que ensina
o contrário»* do `CLAUDE.md` §5.0. ⇒ o ângulo FICA, e o gate novo
`a_cena_deforma_a_arte_o_bastante_para_demonstrar_e_nao_mais` tem as **duas** metades: piso `10 %`
(*a dobra ainda se vê*) e tecto `20 %` (*a arte não está a ser maltratada*). **Prova de mutação:**
`6°` reprova o piso com `3,3 %`; `60°` reprova o tecto com `52,9 %`.

⛔ **A alternativa à lei já é recusa medida deste repo**: o movimento rígido (o *dual quaternion* do
2D) foi construído e **PIORA** a dobra (§5 do `CLAUDE.md`). Não reabrir sem número novo.

### ⚠️ E uma premissa minha que a varredura DERRUBOU no caminho

Ao ver o esticão maior nas PONTAS (`1,411`) do que nas juntas (`1,109`) concluí que dar alcance `2×`
tinha posto todos os ossos a influenciar a tela inteira e **piorado** a forma. Medido lado a lado,
é o contrário — a influência mais larga mistura mais suave:

| cena (3 ossos, `25°`) | tris | faceta | esticão máx | esticão no centro |
|---|---:|---:|---:|---:|
| `512×320` força `1,5` | `780` | `1,89 px` | `1,672` | `1,030` |
| **`512×320` força `2,0`** — esta | `780` | **`0,92 px`** | **`1,411`** | **`1,017`** |
| `512×320` força `2,5` | `780` | `0,81 px` | `1,487` | `1,011` |
| `512×160` força `1,0` (osso longo face à arte) | `468` | `1,88 px` | `1,531` | `1,070` |
| `1024×320` força `1,0` | `1 350` | `1,72 px` | `1,536` | `1,070` |

⭐ **A força `2,0` é o MÍNIMO do esticão máximo** (`1,672 → 1,411 → 1,487`) e o mínimo do esticão no
centro — *a configuração de hoje é o óptimo entre as nove medidas, e não um ponto de partida*.
⚠️ E as cenas «osso longo face à arte», que eu esperava serem melhores por usarem a força de fábrica,
são **piores nas três colunas**.

---

## §15 — ⭐⭐⭐ ORDEM DO DONO: PINTAR ACHATA A ARTE (e o item 4 fica DORMENTE)

*«Deixe como está por enquanto e Inative a possibilidade de pintar sobre malha deformada por ossos.
SE o usuário entrar no modo Painter em imagem deformada por ossos a imagem deixa a deformação para
ser pintada. Ao sair do modo painter, ela retorna a deformação.»* (2026-09-15)

### ⭐⭐ Por que isto custou UM PARÂMETRO e não uma wave

**Toda a app deriva de *«esta sprite tem malha posada?»***:

| quem pergunta | sem malha |
|---|---|
| `canvas_map::CanvasMap` (grelha, guias, gizmos, contornos) | degenera no afim do quad, **ao bit** |
| `ph2d_render::mesh_uv` (o dedo: conta-gotas, dab, alças) | `MeshUv::Quad` |
| o passe de sprites | desenha o quad |

⇒ **não atribuir a malha suspende as três metades de uma vez**, e nenhum consumidor precisa de saber
o que é um Painter. *Uma cerca em cada um seria a mesma lei em três sítios.*

⭐ **E o regresso é por CONSTRUÇÃO:** o `attach_skin_meshes` corre a cada quadro, logo deixar de
suspender devolve a deformação sozinho. ⛔ **Não há estado a repor** — não há como ficar preso
achatado depois de um crash, de um undo ou de trocar de selecção a meio.

⚠️ **A tinta sobrevive intacta:** a malha vive nos bytes opacos da `SkinBind` (traçada no bind) e
pintar escreve na TEXTURA. O que for pintado achatado dobra quando a pele volta.

### ⏳ A CONSEQUÊNCIA, nomeada e devolvida ao dono

**O chrome do Painter que passou a seguir a arte dobrada (§9 e §12 — grelha, selos, gizmos, os
contornos das formas) deixa de ter sujeito enquanto se pinta**, porque durante a pintura não há
dobra nenhuma. ⛔ Ele **não foi apagado e não custa nada** (sem malha o `CanvasMap` é o afim, ao
bit): fica **DORMENTE**, e volta a valer no dia em que pintar sobre a dobra for permitido.

⇒ o roteiro da cena mudou: ela passou a demonstrar o **ciclo** (dobrado → achata ao pegar no pincel
→ pinta → dobra outra vez, com a tinta a dobrar junto). *Um roteiro que continuasse a mandar
procurar a curva debaixo do pincel seria uma cena a ensinar o contrário do que acontece.*

### Os gates

| gate | onde | o que afirma |
|---|---|---|
| `a_sprite_suspensa_nao_recebe_malha_e_a_vizinha_recebe` | `ph2d-skeleton-live` | a suspensa fica sem malha, **a vizinha continua deformada** (senão é um interruptor geral com nome de excepção) e **o regresso** acontece ao deixar de suspender |
| `o_latch_fala_uma_vez_por_entrada_e_volta_a_armar` | `ph2d-app-painter` | o aviso fala à entrada, **cala-se** enquanto se pinta e **volta a armar** ao sair |
| `sem_o_painter_na_mao_nada_e_suspenso` | `ph2d-app-painter` | o controlo: a cura está presa ao MODO |
| `painting_flattens_the_art_and_the_frame_passes_it_through` | `ph2d-editor-core` | a costura: a resposta da porta **chega** ao produtor da malha |

### ⚠️ O que as catracas cobraram, e como foi pago

⛔ **`the_shell_only_shrinks` reprovou DUAS vezes** (`+25` e depois `+15` linhas). Pago **movendo**,
nunca subindo o número:

| o que saiu da shell | para onde | porquê |
|---|---|---|
| a decisão + o aviso | `ph2d_app_painter::skin_suspend::achata_e_avisa` | é vocabulário do Painter |
| a escolha `Fast`/`Smooth` | `ph2d_app_vec::pele_suave` | o knob é da família do vector |
| a escala de ecrã da cena | `ph2d_app_motion::field_gizmo::scene_px_per_world` | a JANELA já vivia lá, e duas contas divergiriam no primeiro split |
| o gate da costura | `ph2d-editor-core/tests/it/` (junto dos irmãos) | **um gate na árvore da shell paga o tecto dela como qualquer ficheiro** |

⛔ E o `skin_image_tests.rs` estourou o teto por ficheiro (`734` de `700`): partido por **assunto**
(`skin_suspend_tests.rs`, filho para herdar as fixturas do arnês).

⚠️⚠️ **E o meu próprio gate ficou VERMELHO no `cargo fmt`**: a 1.ª agulha casava a lista de
argumentos (`"px_por_metro, achatada,"`) e a formatação partiu-a sem uma linha de comportamento
mudar. Hoje ela mede a **propriedade** (a pergunta precede o produtor, e a resposta entra na
chamada). *É a 2.ª vez nesta jornada que uma agulha mede o layout em vez da lei.*

---

## §16 — A DECISÃO SOBRE O QUE FAZER AO CHROME DORMENTE: **fica, marcado** (dono, 2026-09-15)

Perguntado se dava para limpar *«aquela implementação de pintar na malha deformada»* sem risco, a
resposta foi medida antes de ser dada — e a fronteira é nítida.

### O que SAIRIA (só o Painter alcança, e ele nunca mais vê malha)

| peça | linhas |
|---|---:|
| `canvas_map.rs` — a porta | `287` |
| `the_canvas_map_lands_on_the_posed_art.rs` — os gates dela | `402` |
| as conversões nos 9 desenhadores | `61` linhas tocadas |
| o ponteiro do Painter + `architecture_the_canvas_pointer_asks_the_mesh.rs` | `~270` |
| ≈ | **`~1 100`** |

### ⛔ O que NÃO pode sair, e é o que decide a resposta

A maquinaria por baixo serve a **Remoção de fundo**, cujo `ToolId` é `"bgremoval"` — logo ela **NÃO
é achatada** pela suspensão do §15, que só testa `"painter"`. E ela usa tudo:

| consumidor vivo | o que usa |
|---|---|
| conta-gotas · pincel de proteção · *Add area* | `ph2d_sprite_screen::uv_sob_o_ponteiro` → `mesh_uv` |
| a tinta da máscara sobre arte dobrada | `ph2d_render::drawn_instance_of` + o passe de sprites |
| a 2.ª mídia inteira | `attach_skin_meshes` |

⭐ **E o anel do pincel dela não conta:** ele é ancorado no cursor e usa só a escala do afim — a
mesma espécie do anel do Painter. *Nenhuma ferramenta viva desenha geometria que precise do mapa.*

### ⭐ O facto que mudou a conta

**Sem malha, a porta é EXACTAMENTE o que restaria se fosse apagada** — medido e gateado
(`a_segment_over_a_flat_quad_is_still_one_straight_line`: sobre um afim o desvio é `0` e o `pedacos`
devolve `1`). ⇒ limpar compra tempo de compilação e menos código para ler; **não compra correção nem
velocidade**, e custa os gates que provam que o caminho recto é exacto.

### A decisão e o que ela obriga

**Fica, marcado.** A porta leva a declaração inteira no cabeçalho, e cada um dos **9** desenhadores
leva a marca `⏸️ DORMENTE` no topo — porque cada um deles *afirma por escrito* que segue a arte
dobrada, e hoje isso não acontece debaixo do pincel.

⛔⛔ **E a marca tem INSTRUMENTO**, senão ela era a próxima mentira:
`the_dormant_door_says_so_and_the_note_dies_with_the_flattening` ata a nota ao achatamento nas
**duas** direcções — com achatamento a marca tem de estar lá; **sem** achatamento ela tem de SAIR.
⚠️ O censo dos 9 é **DERIVADO** (varre a pasta, com piso de população), nunca uma lista escrita à
mão — é a lição que o §12 desta mesma jornada pagou. **Prova de mutação:** tirar a marca de um
desenhador acusa-o pelo nome; trocar a porta que achata pela irmã muda reprova a porta.

## §17 — ⭐⭐⭐ O PADRÃO-OURO ESTÁ LIGADO AO PRODUTO (ordem do dono: *«quero o estado da arte»*)

> *«eu não decido conforme o preço. Quero o estado da arte, o padrão ouro. Descubra qual é e
> implemente.»* — Enio, 2026-09-15

O que é: **Bounded Biharmonic Weights** (Jacobson/Baran/Popović/Sorkine, SIGGRAPH 2011), cujo caso
de demonstração no próprio artigo é uma personagem 2D. A crate `ph2d-skin-weights` (clean-room do
paper, commit `4d3a2d414`) era a lei **medida numa bancada**; esta jornada ligou-a.

### §17.1 — A tabela que decidiu tudo, e a coluna que não existia

Medida na cena do smoke (`512×320`, corrente de 3 ossos, `25°` por junta), pelo caminho do
**produto**, com `2 430` peças nas quatro linhas:

| lei | `strength` | faceta | esticão | círculo | **VAZAMENTO** |
|---|---:|---:|---:|---:|---:|
| euclidiana (local) | `1,0` | `6,78 px` | `2,234` | `1,2923` | `0,00 px` |
| euclidiana (a que shipou em 14/09) | `2,0` | `0,40 px` | `1,267` | `1,1491` | ⛔ **`26,51 px`** |
| **padrão-ouro** | `1,0` | **`1,27 px`** | `1,492` | `1,2913` | ⭐ **`0,78 px`** |
| **padrão-ouro** | `2,0` | **`1,27 px`** | `1,492` | `1,2913` | ⭐ **`0,78 px`** |

⛔⛔⛔ **A última coluna não existia quando a `strength = 2,0` shipou, e é ela que inverte o
veredito.** *Vazamento* é: rodar só a **ponta** da corrente e medir quanto a arte da **raiz** se
mexe. As outras três medem **suavidade** — e uma mistura larga de mais ganha nas três **por
construção**, porque fazer toda a arte responder a todos os ossos é suavíssimo. *Foi assim que uma
linha inteira de trabalho shipou um borrão global a chamar-se rig.*

⭐⭐ **A comparação honesta é a 1.ª linha contra a 3.ª** — as duas leis que não vazam, isto é, as
duas que são rigs. Ali o padrão-ouro leva a faceta de `6,78` para `1,27 px` (`5,3×`) **sem um
triângulo a mais**.

⭐⭐⭐ **E as duas últimas linhas são IDÊNTICAS coluna a coluna** ⇒ a `strength` ficou **inerte** para
uma imagem. A cena voltou ao valor de fábrica e a `FRACCAO_DO_ALCANCE` + `forca_do_osso()`
**morreram**. Isto fecha o item que estava no topo da fila desde 2026-09-09: *o alcance de um osso
não sabe nada da arte que carrega* — hoje ele não precisa de saber. Ele diz *«este pedaço é meu»* e
a energia sobre a arte decide o resto.

### §17.2 — O que se PERDE, dito em voz alta

O círculo desenhado sai de `1,15` para `1,29` de ovalização. ⛔ **Não é uma regressão do motor — é a
articulação a passar a existir:** uma circunferência por cima de uma junta que dobra `25°` *tem* de
deformar, e o que a mantinha redonda era o rig não estar de facto a articular.

### §17.3 — As quatro peças da ligação

1. **`ph2d_poly2d::refine_posed_attrs`** — atributos por vértice que viajam na subdivisão. Ela
   existe porque o padrão-ouro **não é função de uma posição**: um vértice que o `Smooth` inventa
   tem de **herdar** o peso do triângulo que o gerou. ⛔ Localizar o ponto seria `O(n)` por ponto
   para chegar à resposta que a proveniência já sabe de graça. Com `stride = 0` é o `refine_posed`
   **ao bit**.
2. **`ph2d_skeleton::SkinBone::tendon` + `Skin::point_with`** — a tabela guardada é por osso
   **autorado**; a pele resolvida tem `N` poses por osso (bendy) e **salta** os apagados. ⛔ Usar a
   posição na pele faria a arte saltar no instante em que alguém apagasse um osso, com zero gates a
   acusar. A repartição pelos sub-ossos é a **mesma** `bend::share` de sempre.
3. **`SkinnedMesh { mesh, pesos }`** nos bytes opacos do `SkinBind::source`. ⚠️ A lei deste repo era
   *«os pesos não se guardam»* — ela foi escrita para uma lei que é função de um **ponto**. ⭐ E eles
   não são um vector paralelo: são **o mesmo objecto**, com o comprimento de um função do do outro e
   uma porta que recusa o par que não fecha.
4. **`bind_image` resolve · `skinned_mesh_of` lê · `posed_sprite_mesh` usa** — e o onion pela mesma
   porta. ⚠️ `tendons_and_axes` produz os tendões **e** os eixos num percurso só: duas varreduras e
   um osso sem `StableId` numa delas poria a coluna `j` a descrever o osso `j+1`, com a soma a `1` e
   nenhum gate de geometria a acusar.

`PROJECT_SCHEMA` **129 → 130** (conte o DELTA). ⛔ **Limite NOMEADO:** uma forma **vectorial** fica
na lei euclidiana — o padrão-ouro precisa de uma malha do domínio e uma Bézier não tem uma. *Um rig
com as duas mídias tem hoje duas leis*, e essa é a obra seguinte.

### §17.4 — ⭐⭐⭐ A malha do bind virou um ORÇAMENTO (`GridOptions::target_tris`)

⛔ Um passo em pixels **não diz de que recurso é**: a contagem é `área / passo²`, então a MESMA
configuração entregava `180` triângulos numa arte de `256` e `2 586` numa de `1024` (**`14,4×`**) —
*o quadro paga por o desenho ser grande, não por ele ser difícil* (§0.0). Com a contagem:
`2 720 / 2 870 / 2 994`. É a mesma cura que o botão `Quad Retopology` pagou em 2026-08-28.

⭐ **O alvo `3 000` sai do TEMPO do quadro:** `0,44 µs` por peça **medido** (`1 152`/`4 608`/`10 368`
peças, o mínimo de 40 corridas) ⇒ `1,667 ms / 0,44 µs` = `3 788` para uma imagem sozinha.

### §17.5 — ⭐⭐ A GRADUAÇÃO da grelha TROCOU DE SINAL quando a lei mudou

Ela estava registada como **anti-correlacionada com o erro** — e estava, para o *bump* euclidiano:
fina no eixo do osso, grossa na borda, que é onde o bump normalizado explodia. Com o padrão-ouro os
pesos variam depressa junto das **restrições**, isto é, junto dos eixos. Medido a contagem
igualada (`~2 900` peças):

| grelha | peças | faceta |
|---|---:|---:|
| **graduada** | `2 430`–`2 958` | **`1,15`–`1,27 px`** |
| uniforme | `2 880`–`2 940` | `1,94`–`1,96 px` |

⇒ *trocar a lei curou a grelha.* **Um defeito registado dissolveu-se sem ninguém lhe tocar.**

### §17.6 — ⛔ TRÊS defeitos de RÉGUA que esta jornada apanhou em si mesma

1. **O vazamento escolhia a ponta pelo maior `to_bits`**, rodava a **raiz**, e leu `154 px` sobre a
   lei que não pode vazar. Hoje sai da porta do produto (`chain_ends`).
2. **A varredura da dobra escrevia `rotation = graus` numa cena JÁ dobrada** e leu `1,426` de
   esticão **em repouso**. *Repor um ângulo não é repor uma pose* — a pose de repouso de um osso
   filho é o que a `bone::create` escreveu. O controlo que ficou: uma cena que **nunca** dobra lê
   `4,97e-16 m` e as três poses saem `[1,0,0,1,0,0]` exactas.
3. **A inversão comparava sinais de área crus** entre a malha (`y` para baixo) e a pose (`y` para
   cima) ⇒ `100 %` invertido sobre uma cena perfeita.

⚠️ **E as réguas da cena mediam a lei ERRADA:** elas chamavam `pele.point`, a euclidiana derivada.
*Um arnês que não usa a lei do produto mede um programa que ninguém corre.*

⛔⛔ **A sonda do custo escrevia uma `Mesh2d` crua no `source`**, que o formato novo recusa — e ela é
`#[ignore]`, logo ficava **verde a medir zero peles**. *Uma sonda que o CI nunca corre é o sítio
onde um formato novo se esconde.*

### §17.7 — A prova de mutação que corrigiu a própria fixtura

⛔⛔ O gate da proveniência dos atributos corria a `0,5 px` e o estimador parava em **`k = 3`**, onde
o **único** nó de miolo de cada triângulo é o `(1,1)` ⇒ `u = v = ⅓`. Trocar `t[1]` por `t[2]` na
interpolação baricêntrica é, ali, a **identidade algébrica**: a mutação SOBREVIVEU com o gate verde.
*Uma grelha de `k = 3` não tem um único ponto interior onde as duas coordenadas baricêntricas
difiram, logo nenhuma fixtura nesse `k` pode distinguir os dois vértices.* A `0,12 px` o `k` sai `6`
e as duas mutações morrem.

### §17.8 — ⏳ O QUE FICA ABERTO

- ⛔ **A forma VECTORIAL na lei euclidiana** (§17.3). É a obra seguinte, e o mecanismo está escrito:
  o padrão-ouro precisa de uma malha do domínio.
- ⏳ **O refinamento do `Smooth` continua estruturalmente inerte** (`max_split = ⌊√(orçamento/peças)⌋`
  = `1` acima de `orçamento/4` peças). A cura medida é o **adaptativo por triângulo** (`3 034` peças
  contra `28 080` do `k = 6` global, `9×` mais barato) — §13.
- ⏳ **A faceta converge `O(h^1,2)`**, entre `O(h)` e `O(h²)`: o padrão-ouro é `C¹` mas não `C²` na
  fronteira do conjunto activo, que é **o preço das caixas** — e as caixas são o que compra a
  localidade. ⇒ a malha nunca o segue com a ordem cheia, e mais densidade tem retorno decrescente.
- ⛔⛔ **DÍVIDA NOMEADA: a escada do `PROJECT_SCHEMA` cresce a cada degrau e o tecto
  `the_shell_only_shrinks` é fixo.** Este degrau nasceu com `28` linhas, o tecto reprovou por `9`, e
  a cura foi apertar a prosa sem perder um facto. *O degrau que não puder ser apertado tem de MOVER
  a escada para fora da shell.*
- ⚠️ **VERMELHO PRÉ-EXISTENTE e não desta linha:** `clippy` acusa `PreviewDrive::len` sem `is_empty`
  na `ph2d-preview-drive` — confirmado numa árvore limpa.

## §18 — ⭐⭐⭐ A RE-MEDIÇÃO DA RECUSA APANHOU UMA REGRESSÃO, E A CAUSA ERA A CONDIÇÃO DE FRONTEIRA

O §0.0 manda: *quem move o número que tornava algo inalcançável tem de reconferir a nota.* A mesa do
oráculo ([`docs/Skeleton/oraculo/`](../oraculo/README.md), 2026-09-14) **recusou os pesos
harmónicos** com `11,3 %` de dobra contra `3,1 %` do que shipava — e o padrão-ouro é um vizinho
deles. Corrido naquela mesa, ele lia **`9,53 %` a `90°`**: dentro da família recusada.

E na cena do **produto**, um penhasco — não uma degradação:

| graus/junta | euclidiana (fábrica) | BBW (antes da cura) |
|---:|---:|---:|
| `45` | `0,43 %` | `0,00 %` |
| `60` | `0,75 %` | `0,29 %` |
| **`90`** | **`0,89 %`** | ⛔ **`9,80 %`** |

### §18.1 — A causa: eu sobre-restringia, e não era a energia

⭐⭐⭐ Dois ossos de uma corrente **partilham a junta**, então os eixos deles **tocam-se**. Prender o
eixo inteiro de cada um põe `w = 1` de um lado e `w = 0` do outro **em vértices vizinhos** — e a
energia nunca chega a ter voto ao longo do eixo, porque o eixo é **todo Dirichlet**.

⇒ [`Options::folga_da_junta`]: **ambíguo ⇒ LIVRE**. Um vértice que tem dois ossos igualmente perto
não é *«claramente de nenhum»*, e é exactamente ali que a mistura tem de acontecer. ⭐ A regra **não
precisa de saber o que é uma junta**.

### §18.2 — O `4` sai de um JOELHO, e os dois lados dele têm mecanismo

| folga | presos | arte rígida | **vazamento** | faceta | dobra `90°` |
|---:|---:|---:|---:|---:|---:|
| `0` | `49` | `10,1 %` | `0,13 px` | `1,96 px` | `9,44 %` |
| **`4`** | **`31`** | **`6,8 %`** | **`0,38`** | **`0,30`** | **`6,08 %`** |
| `8` | `15` | `2,4 %` | ⛔ **`5,24`** | `0,30` | `2,67 %` |
| `16` | `3` | `13,4 %` | ⛔ `10,93` | `0,20` | `2,74 %` |

⛔ **Acima do joelho o VAZAMENTO dispara `14×`** — alargar a mistura é o que faz um osso alcançar a
arte do vizinho, que é o defeito que esta lei entrou para curar. *Uma folga grande demais re-compra
o borrão global pela porta do lado.* E a `16` os pinos colapsam para `3` (um por osso, pela rede da
2.ª metade) e a **rigidez volta a subir**.

### §18.3 — O resultado na cena do produto

| lei | faceta | esticão | círculo | **vazamento** |
|---|---:|---:|---:|---:|
| euclidiana (fábrica) | `6,78 px` | `2,234` | `1,2923` | `0,00 px` |
| euclidiana `2,0` (shipou 14/09) | `0,40 px` | `1,267` | `1,1491` | ⛔ `26,51 px` |
| **padrão-ouro** | **`0,42 px`** | **`1,401`** | `1,2894` | ⭐ **`1,08 px`** |

⭐⭐⭐ **Ele passa a EMPATAR com a lei borrada na faceta** (`0,42` contra `0,40`) **sem o vazamento**,
e a dobra dura melhora em toda a escada: `45°` e `60°` vão a **`0,00 %`** (contra `0,43`/`0,75`) e
`90°` de `9,80` para `5,65 %`.

### §18.4 — ⛔⛔ TRÊS réguas minhas mentiram antes de uma delas dizer a verdade

1. ⭐⭐⭐ **O perfil do peso amostrava `y = 0`, que é a LINHA PRESA** — eu lia a **condição de
   fronteira** e chamava-lhe solução, e quase registei *«o padrão-ouro é uma função ESCADA»*. Fora do
   eixo ele é suave (`0,98 → 0,92 → 0,76 → 0,59 → 0,38 → 0,12` a `y = 0,3`), e quem é a escada é o
   **bump**: na borda da arte (`y = 2,3`, fora do raio dos dois ossos) ele lê
   `1,0000 · 1,0000 · 0,0000 · 0,0000`.
2. **A corrente que eu escrevi para a varredura compunha as poses ao contrário** — empurrava o
   `mundo` onde vai a matriz de pele `rest⁻¹ ∘ mundo` — e lia `21 %` de dobra a `25°` onde o produto
   lê `0,00 %`. ⇒ a varredura passou a usar **a porta que já existia**, parametrizada.
3. **O censo da rigidez usava «ALGUM peso é zero»** — com TRÊS ossos isso acusa `63 %` de uma malha
   perfeitamente misturada (um vértice `(0,5 · 0,5 · 0,0)` mistura dois ossos). *Um predicado quase
   sempre verdadeiro não é um censo, é ruído.*

### §18.5 — ⭐⭐⭐ E o controlo da RIGIDEZ explica a tabela do oráculo inteira

| lei | **arte rígida** (um osso leva tudo) | grad. vertical máx | dobra `90°` |
|---|---:|---:|---:|
| bump (o que shipava) | ⛔ **`37,6 %`** | `5,000` | `0,74 %` |
| **padrão-ouro** | **`8,1 %`** | **`1,542`** | `9,53 %` |

⇒ **a régua da dobra é ganha por RIGIDEZ**: arte presa a UM osso não pode inverter-se porque **não se
deforma**. É a **TERCEIRA** família de réguas desta jornada que uma resposta degenerada ganha por
construção — as de suavidade, pelo borrão global; esta, pela rigidez.

⚠️ **Isto não desculpa os `5,65 %` a `90°`**, e eles ficam nomeados. Mas explica por que a tabela de
2026-09-14 lia o que lia, e por que a linha *«pesos harmónicos: `11,3 %`»* nunca foi um veredito
sobre a lei — era um veredito sobre a mesa.

### §18.6 — ⏳ O QUE ISTO DESTRAVA: os CENTROS DE ROTAÇÃO voltam à mesa

A mesa de 2026-09-14 mediu os **centros de rotação optimizados** (Le & Hodgins, 2016) em `8,8 %` e
escreveu, ela própria, por que o número não valia: *«não se pode julgar uma mistura melhor por cima
de pesos degenerados — `48 %` da nossa arte não mistura nada»*. ⭐ **Essa premissa dissolveu-se:** a
arte rígida passou de `37,6 %` para `6,8 %`. ⇒ o candidato que ataca o colapso a `90°` **sem** tocar
nos pesos é agora avaliável, e é a obra seguinte. *A recusa não foi revogada — o chão dela mudou.*

## §19 — ⭐⭐⭐ A FORMA VECTORIAL ENTRA NO PADRÃO-OURO: as duas mídias voltam a ter UMA lei

O §17 ligou os BBW **só para imagens**, com o limite nomeado: *«o padrão-ouro precisa de uma malha
do domínio, e uma Bézier não tem uma»*. ⛔ O preço era um rig com **duas leis** — e a dívida era
minha.

⭐ **Ela tem domínio: o INTERIOR dos contornos fechados.**

```text
contornos FECHADOS ──▶ achatados em polígonos        (a fronteira, do documento)
                   ──▶ grelha graduada + orçamento   (ph2d_poly2d::grid_mesh_com)
                   ──▶ Bounded Biharmonic Weights
                   ──▶ um peso por PONTO DE CONTROLO (baricêntrico na malha)
```

### §19.1 — ⭐⭐ UMA grelha, DUAS coberturas

[`grid_mesh_com`] recebe a **cobertura injectada**: uma imagem responde pelo **alfa**, um caminho
por **estar dentro do contorno**. A graduação pelas articulações, a conformidade, o orçamento em
triângulos e a renormalização são do **GRID** e não da mídia.

⛔ **A malha do caminho NÃO é guardada nem desenhada** — ela é um **andaime do bind**. O que
sobrevive é um peso por ponto de controlo, e a forma continua a ser uma Bézier exacta e editável,
que é a razão de o vector usar LBS e não um envelope.

⛔ **Um caminho ABERTO fica na lei derivada, e isso é uma RESPOSTA.** Uma linha tem área zero.
*Inventar um domínio para uma linha seria inventar uma arte que o artista não desenhou.*

`PROJECT_SCHEMA` **130 → 131** (conte o DELTA).

### §19.2 — ⛔⛔⛔ Um gate apanhou um defeito REAL do pino, que a lei antiga escondia de graça

`binding_to_the_whole_scene_draws_the_same_as_binding_to_the_right_skeleton` reprovou: **um segundo
esqueleto a `400` unidades da arte mudava o desenho.** A rede *«todo osso leva pelo menos um
vértice»* fazia o osso distante roubar o vértice mais próximo e mandar nele com peso `1`.

⭐⭐ **A lei euclidiana estava protegida de graça — o suporte dela é FINITO. O do padrão-ouro é
global POR DESENHO, e é isso que compra a suavidade.** ⇒ a rede ganhou cerca: um osso cujo vértice
mais próximo está a mais de `2` arestas médias **não é handle desta arte**.

⚠️ *Toda propriedade que a lei antiga tinha «de graça» tem de ser reconferida quando a lei muda* —
e esta só apareceu porque o gate existia e media a cena que o artista produz **sem querer** (o
`Bind` sem semente apanha a cena toda).

### §19.3 — ⛔⛔ E uma mutação SOBREVIVEU aos três primeiros gates

Pôr `usa = false` no [`ph2d_vec_skin::aplica_com`] — o vector a **ignorar** os pesos guardados —
deixava os **29 verdes**. Eles provavam que a tabela **existe** e que ela **fecha**; nenhum provava
que ela **chega ao desenho**. *É o terceiro passo que um `grep` não vê: o painel escreve · alguém
lê · **o leitor DECIDE, ou entrega a quem descarta?***

⇒ `a_tabela_de_pesos_do_caminho_chega_ao_desenho`, com as DUAS metades: o quadro tem de dar o que a
lei **guardada** dá (`0,000`) **e** diferir do que a **derivada** dá (`10,47`).

⚠️⚠️ **E a fixtura teve de ser refeita DUAS vezes para produzir o fenómeno:**

| fixtura | separação entre as duas leis |
|---|---:|
| o `palco` (rectângulo `40 × 10`, pontos de controlo nos CANTOS) | `0,000` |
| seis vértices, **dois sobre a junta** | `0,064` |
| **e a arte ALTA (`40 × 30`)**, com a aresta de cima para lá do raio do *bump* | **`10,47`** |

⇒ *a diferença entre as duas leis vive junto da JUNTA e LONGE DO EIXO* — e um rectângulo baixo com
os vértices nos cantos não tem nenhum dos dois.

### §19.4 — ⛔⛔ O gate da ORDEM dos ossos mudou de PREMISSA, não de veredito

Enquanto os pesos eram DERIVADOS, a lista de tendões era a única coisa ordenada. Com a tabela
guardada, a coluna `j` **é** o tendão `j` ⇒ reverter os tendões sem reverter as colunas não permuta
nada: produz **dados incoerentes**, e o gate lia `18,5` a acusar um defeito que não existe. *É o
mesmo erro que reverter os triângulos de uma malha sem reverter os vértices.* Ele passa a permutar
**as duas metades**.

### §19.5 — A catraca do shell foi paga MOVENDO, como a dívida do §17.8 mandava

O degrau `131` não cabia por `21` linhas, e apertar a prosa não chegava — *era exactamente o caso
que o §17.8 nomeou*. ⇒ a faixa `v2`..`v82` da escada (`490` linhas de **prosa pura**) saiu para
[`docs/archive/project-schema/escada_v2_a_v82.md`](../../archive/project-schema/escada_v2_a_v82.md),
**verbatim**, com o `sha256` do original no fim. Os degraus **vivos** continuam colados à constante
— *quem conta o próximo degrau lê a escada, e o que ele precisa de ler é a PONTA.*

⭐ **A dívida ficou paga em vez de renomeada:** a escada tem agora `490` linhas de folga para crescer
antes de a colisão voltar.

## §20 — ✅ CHECKPOINT: `skeleton-padrao-ouro-checkpoint` (ordem do dono, 2026-09-16)

> *«resultado perfeito, superior a todos os anteriores. Crie um checkpoint nesta implementação»*

**Tag anotada** sobre `88297fbd3` (`119` commits desde o merge-base `1d43da737`):

```
git show skeleton-padrao-ouro-checkpoint          # a mensagem inteira, com a tabela
git switch -c <ramo-novo> skeleton-padrao-ouro-checkpoint
```

⚠️ **Ela é LOCAL** — nada foi enviado, e integrar/shipar continuam a exigir ordem explícita
(`CLAUDE.md` §0.7). ⭐ A mensagem da tag é **auto-suficiente**: ela carrega os números, os dois
smokes aprovados e o que fica aberto, para quem a encontrar daqui a meses não precisar deste ficheiro.

### O que este ponto foi CERTIFICADO a ter, antes de ser marcado

| | |
|---|---|
| `bash scripts/nextest-impacted.sh` | **15 281 verdes**, 0 vermelhos |
| `cargo fmt --all --check` | limpo |
| `cargo clippy --workspace --all-targets` | limpo (⚠️ menos o `PreviewDrive::len`, **pré-existente**) |
| `PROJECT_SCHEMA` | `131` |
| árvore | sem ficheiro por commitar |

⛔ **A certificação correu ANTES da tag e sobre o MESMO commit** — `git diff tag HEAD` é vazio. *Um
checkpoint que não diz o que foi verificado nele é um marcador, não um ponto de retorno.*

### Os dois smokes que o dono aprovou neste ponto

| cena | comando |
|---|---|
| a **imagem** presa (o canvas pintado) | `PH2D_VEC_BONE_PAINT_SMOKE=1` |
| a **forma vectorial** presa | `PH2D_VEC_BONE_SMOKE=1` |

Os dois a partir de `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector`, com
`cargo run -p ph2d-host-desktop --profile smoke`.

---

## §21 — ⛔⛔⛔⛔ O BOTÃO `Smooth` ERA UM CONTROLO MORTO (auditoria de 2026-09-16)

**A pergunta que o dono fez:** *«quais próximas etapas de implementação?»* ⇒ auditar a fila **contra
o código** (§5.0), não recitá-la. O primeiro achado não estava na fila: **o par `Fast`/`Smooth` do
painel entregava a mesma malha, ao bit.**

### §21.1 — A causa é ARITMÉTICA, e nenhuma sonda deste repo a vê

A lei de refinamento era o `k` **GLOBAL** (cada triângulo partido `k × k`), cujo tecto é
`max_split = ⌊√(orçamento / peças)⌋`. ⇒ **toda malha acima de `orçamento / 4` peças só admite
`k = 1`.** Medido:

| cena | peças | orçamento | `max_split` |
|---|---:|---:|---:|
| malha de bind (2026-09-15) | `2 430` | `1 543` | **`1`** |
| a mesma, com o orçamento corrigido | `2 430` | `3 787` | **`1`** |
| a do smoke do braço | `780` | `1 543` | **`1`** |

⛔ **A segunda linha é a que importa: subir o orçamento NÃO cura.** Para `k = 2` numa malha de
`2 430` peças ele teria de ser `9 720`, que a `1,08 µs`/peça são `10,5 ms` — dois terços de um quadro.

⛔⛔ **É a TERCEIRA espécie de controlo morto do `CLAUDE.md` §5.0**, e a mais cara de achar: o id
existe, é pintado, é registado, o clique chega à ferramenta e o valor chega ao consumidor — *e o
consumidor devolve a entrada*. O `architecture_panel_wiring_parity` mede focalizabilidade; os
`seam_*` provam que a escrita chega ao consumidor. **Nenhum instrumento deste repo pergunta se a
SAÍDA muda.**

### §21.2 — A lei nova, e a frase que ela refuta

A operação elementar deixa de ser *«partir um triângulo»* e passa a ser ***«partir uma ARESTA»***:
os dois donos partem-se no mesmo acto, logo a malha é conforme **depois de cada passo** e um ponto
no meio de uma aresta que deixou de existir não fica pendurado em nada.

⛔⛔ **Isto refuta por construção a frase que o cabeçalho do `refine.rs` carregava desde 10/09** —
*«um `k` por triângulo abriria nós pendurados, que é a fenda que a `grid.rs` recusa por escrito»*.
Ela estava certa sobre **uma** construção (a grelha baricêntrica por triângulo) e foi lida como se
fosse sobre a pergunta inteira. *Antes de dizer que uma pergunta é inexprimível, procure a outra
operação elementar.*

Qual aresta: a **mais longa** (Rivara/LEPP), que dá piso aos ângulos. Qual triângulo: o de **pior
desvio**. ⭐ O desvio de uma aresta é partilhado pelos dois donos e a conta que o mede **produz o
ponto que a bissecção insere** — medir e partir custam uma avaliação do campo, nunca duas. ⭐⭐ E o
desvio de um triângulo VIVO é **imutável** (partir qualquer aresta dele mata-o), logo a fila de
prioridade não precisa de actualizar entradas.

⚠️⚠️ **A ordem das arestas tem de ser TOTAL, e isso é load-bearing:** numa malha de grelha os
empates de comprimento existem **por construção**, e com empates a cadeia LEPP pode voltar a um
triângulo já visitado e girar para sempre. A chave é `(comprimento, vértice menor, vértice maior)`.

### §21.3 — ⚠️ Duas premissas minhas caíram na medição

1. **O ganho é função da ARTE, não uma constante.** Sobre um campo que curva por igual em todo o
   lado a lei uniforme está quase óptima (`1,9×`). O ganho grande é o caso do PRODUTO — a curvatura
   na articulação: **`11,3×`** a `0,50 px`, e ali a uniforme **nunca chega** (satura em `k = 8` e
   fica em `1,121 px`). *A vantagem dela não é «partir menos»; é partir **onde**.*
2. **Na cena do dono enquadrada inteira o `Fast` já entrega `0,34 px`** — abaixo da promessa de meio
   pixel, e não refinar é a resposta CERTA. A alavanca do produto é o **ZOOM** (a tabela está no
   gate `o_smooth_deixou_de_ser_um_controlo_morto_na_cena_do_produto`).

### §21.4 — ⛔⛔ E o ORÇAMENTO vinha de um custo que o produto nunca pagou

O `CUSTO_POR_PECA_NS = 1 080` foi medido em 13/09 sobre um `Smooth` que **refinava**; desde então o
que ele de facto pagava era o `Fast` mais o custo de decidir. Remedido com a lei que shipa
(`load 5,6`–`6,1`, mínimo de 40/60, **três** corridas entre `0,328` e `0,355`): `0,340 µs` de CPU +
`0,013` marginal de GPU ⇒ **`353 ns`**, e o orçamento vai de `1 543` para **`4 721`** peças.

*Um custo medido sobre um caminho que não corre é um orçamento que mente nos dois sentidos* — e este
mentia para BAIXO, o que fazia a malha de bind de `2 268` disparar o `avisa_malhas_acima_do_orcamento`
em **toda** execução, sobre um aviso correcto e um número errado.

⚠️ **A lei nova é `3,9×` mais cara POR PEÇA** (`0,340` contra `0,087`) — é o preço de ela decidir; a
antiga era barata porque não fazia nada. O livro de contas já foi cortado uma vez: dois mapas e um
`Vec` por aresta liam `0,52 µs`, e com **um** mapa e os donos num par fixo desceu a `0,34`.

⚠️⚠️ **E a sonda de custo media ZERO:** a tabela de pesos dela era **uniforme**, e com pesos iguais
em todo o lado a mistura das poses é a mesma em todo o ponto ⇒ o campo é um **AFIM**, que um
triângulo reproduz exactamente ⇒ desvio zero e nenhuma das leis refina (`saiu == pecas` nas cinco
linhas). *Uma fixtura no ponto neutro de um knob não testa esse knob.*

---

## §22 — ⭐⭐⭐ O OSSO QUE DOBRA FECHOU (os três abertos da F8, 2026-09-16)

### §22.1 — O esticão deixou de VARIAR, e a cura publicada não chegava

Medido com `8` sub-ossos: `12,63 %` de dispersão com as alças a `0,2 L`, `82,01 %` a `0,6 L`,
`1 051,95 %` com as alças cruzadas no eixo.

⛔⛔ **Equalizar o ARCO — a cura que a recusa nomeava — deixa um PISO que não desce com a tabela**
(`1,22 %` a `0,6 L`, igual de `16` a `32` amostras). *Um número que não se move quando se afina a
discretização não é erro de discretização.* A causa é geometria: o esticão é `corda / (L/n)`, e a
corda de um pedaço mais curvo é mais curta que o arco dele ⇒ **arcos iguais dão cordas desiguais**.

⇒ a lei parte da equalização por arco e **corrige-a para a CORDA**, em rondas de Gauss-Seidel sobre
a posição em arco de cada nó interior. As duas constantes saem da varredura, e a coluna que decide é
o caso duro:

| amostras × rondas | `0,2 L` | `0,6 L` | eixo cruzado |
|---|---:|---:|---:|
| **(sem equalizar)** | **`12,63 %`** | **`82,01 %`** | **`1 051,95 %`** |
| `8 × 0` (só o arco) | `0,068 %` | `1,258 %` | `93,65 %` |
| `8 × 2` | `0,000 %` | `0,000 %` | `0,922 %` |
| **`8 × 8`** | **`0,000 %`** | **`0,000 %`** | **`0,000 %`** |

⚠️⚠️ **A objecção registada na recusa era VERDADEIRA e não mordia:** *«um somatório de cordas não
devolve `L` ao bit»* — ele **nunca corre** no ponto neutro, porque a primeira linha é um `if` sobre
`Bend::is_straight`. *Uma recusa que nomeia um custo tem de dizer em que CAMINHO ele é pago.*

### §22.2 — As alças pegam-se no canvas, e a armadilha era o ponto NEUTRO

⛔⛔ No neutro a alça está **em cima do eixo do osso** (ela é o ponto de controlo no terço), e o
corpo é um alvo com outro verbo: a competição por proximidade que as outras alças usam tornava-a
**inalcançável no único estado em que todo osso nasce**. *Uma alça que só se agarra depois de já ter
sido movida não se agarra nunca.*

⇒ ela é a única que ignora o `d_osso`, e em troca paga um raio apertado (`BEND_HIT_PX = 8 px`), cujo
recurso tem nome: **o comprimento do osso que sobra para o verbo de girar**. As duas ficam nos
terços, logo o meio do osso continua a girar a partir de `48 px` de osso na tela — e há gate.

### §22.3 — As tangentes dos vizinhos, e por que o neutro é EXACTO

`Curve Handles: Manual | From Chain`. Em `From Chain` as alças saem das tangentes dos ossos
vizinhos. ⭐⭐ **As tangentes saem da transformação RELATIVA** (a pose de um filho em relação ao pai
**é** o `Transform` dele), nunca de uma volta pelo mundo: numa corrente recta o pai fica em
`(−L_pai, 0)` no local deste osso (o inverso de uma translação pura é exacto), as duas tangentes são
`(1, 0)` **ao bit**, e a conta *«um terço da tangente MENOS um terço do eixo»* dá zero exacto.
*Uma volta pelo mundo teria deixado `y ≈ 1e-17`, e ligar o modo num rig recto arquearia tudo.*

⛔ **Em `From Chain` não há alça para agarrar** (nem pintada, nem no dedo, nem escrevível) e os
quatro números do painel **somem** — elas são derivadas, e arrastar uma seria escrever num valor que
o quadro seguinte recalcula. O `curve` autorado fica **intocado**: *um modo que sobrescreve o valor
autorado é um modo que não se desliga.*

⚠️ **UMA porta resolve o modo** (`skin_live::effective_spec`), e os **quatro** leitores do
`Bone::spec()` passam por ela — o corpo que se pinta, a alça que se agarra, a pele que se deforma e
a malha do bind. *Um controlo DESENHADO por um mapa e DEFORMADO por outro é um controlo morto sob o
dedo.*

⚠️ `PROJECT_SCHEMA` **131 → 132** (o `Bone` ganhou `handles`) — conte o DELTA. O `#[serde(default)]`
**não** salva o postcard: ele serve formatos com nomes, e o postcard é posicional.

---

## §23 — ⚠️ TRÊS ENTRADAS DA FILA ESTAVAM OBSOLETAS, e uma delas eu relatei ao dono como aberta

A auditoria de 2026-09-16 varreu a `01_a_fila.md` contra o código e achou **três** entradas que
mandavam reconstruir trabalho já pago:

1. *«a malha é só o contorno, e um membro grosso dobra pela borda»* — a **própria linha** dizia, a
   seguir, *«o dono viu isso na primeira olhada e a F6-b curou-o»*, e mesmo assim tinha um ⏳ à
   frente.
2. *«o alcance de um osso não sabe nada da arte»*, com **três rotas** e um pedido de decisão do dono
   — ⛔ **MORTA**: o padrão-ouro apagou a pergunta (o alcance ficou inerte, e o hand-tuning dele foi
   apagado do produto na mesma jornada).
3. *«o conta-gotas do BgRemoval usa uma caixa alinhada aos eixos que ignora rotação e malha»* — 
   **curado pela F6-m, que está quatro secções abaixo NO MESMO FICHEIRO**.

⛔⛔ **E a terceira sobreviveu à minha própria auditoria: eu relatei-a ao dono como aberta.** Li a
linha, não li a secção que a fecha. *Uma nota obsoleta ao lado da secção que a cura é pior que uma
nota ausente — a ausente não é acreditada, e esta foi.*

---

## §24 — ⭐⭐⭐ O `Smooth` DESENHAVA OS VINCOS DOS PESOS (smoke do dono, foto, 2026-09-16)

*«Smooth parece ter resultado discretamente inferior, gerando micro irregularidades.»* Mecanismo,
tabela, gates e mutações: [Bug #33](../../Vector%20Module/BUGS_vector.md) · fila **F6-p**.

**O que o integrador precisa de saber:**

1. **Uma folha nova na `ph2d-poly2d`: `attr_law.rs`** (`AttrLaw`, `recover_gradients`,
   `hermite_attrs`, e o `midpoint` que as três réguas partilham). ⚠️ **As portas antigas mantêm a
   assinatura e a lei** (`refine_posed_attrs` e `deviation_attrs` = `AttrLaw::Linear`); as novas são
   `refine_posed_with` e `deviation_with`. A que MUDOU de assinatura é a
   `refine_posed_adaptive` (ganhou `law`) — ela é `pub`, e o único chamador de fora da folha é o
   despachante.
2. **Uma porta de produto nova: `ph2d_skeleton_live::skin_refine`** (`refine_skinned`,
   `skinned_deviation`, `weight_law`, `weight_attrs`). O `posed_sprite_mesh` chama-a; os gates e a
   bancada da `ph2d-app-vec` também — ⛔ *uma régua que refina por outra porta mede outro programa.*
3. **Env nova de bissecção: `PH2D_SKIN_WEIGHTS=linear`**, independente do `PH2D_SKIN_REFINE`.
4. **Zero schema, zero contrato, zero registo.** O formato guardado não muda: os gradientes
   recuperam-se por quadro, da malha e dos pesos que já viajavam.
5. **O orçamento não muda** (`CUSTO_POR_PECA_NS = 353`): a lei custa `−1 %` a `+3,8 %` por peça na
   cena do smoke, e `0,340 × 1,038 = 0,353`.
6. ⛔ **O gate `o_smooth_deixou_de_ser_um_controlo_morto_na_cena_do_produto` estava VERDE sobre o
   defeito** e continua lá: ele mede outra pergunta (o botão refina onde a dobra pede). A tabela dele
   mudou (`2 364`/`3 416`/`0,54` → `2 344`/`3 270`/`0,55`) porque o `Fast` e o adaptativo passaram a
   ser lidos pela régua da lei que o `Smooth` segue — e cada coluna agora diz qual régua a lê.
7. **O ficheiro de gates do `Smooth` partiu-se por responsabilidade:** a silhueta (a régua do olho,
   as sondas e o gate novo) vive em `smoke_bone_paint_silhueta_tests.rs`, filho do de antes.
8. ⚠️ **Achado PRÉ-EXISTENTE, não curado (é de outra folha e o CI não o vê):**
   `cargo clippy -p ph2d-preview-drive -- -D warnings` **reprova sozinho** no merge-base
   (`len_without_is_empty`): o `is_empty` só existe sob `cfg(any(test, feature = "test-support"))`,
   e na workspace a feature chega unificada por outra crate. *Uma crate que só passa o lint na
   companhia das outras é a família «a build da workspace esconde a crate que não compila
   sozinha».* A cura provável é o `is_empty` sem `cfg` (um método `pub` não dispara `dead_code`).

---

## §25 — ✅ O ANEL DO PINCEL DA REMOÇÃO DE FUNDO SEGUE A ARTE DOBRADA (2026-09-16)

Fila **F6-q**. O que o integrador precisa de saber:

1. **Porta nova na folha `ph2d-sprite-screen`: `anel_do_pincel`** (+ `LADOS_DO_ANEL`), ao lado da
   `uv_sob_o_ponteiro`, que é a metade inversa do mesmo par.
2. ⚠️ **A shell mudou de assinatura num sítio:** o `bgremoval_preview::dispatch` recebe o mundo de
   apresentação por **`&mut`** (a porta do ponteiro faz uma consulta ECS), e a fase passa
   `present.world_mut()`. O `brush_ring` leva o tamanho INTEIRO da origem (`(w, h)`), não só a
   largura. A shell **encolhe** (`−13` linhas).
3. **Dev-dependência nova:** a `ph2d-app-vec` ganha `ph2d-sprite-screen` só para a sonda da cena
   real.
4. **Zero schema, zero contrato, zero registo.**
5. ⚠️ **Aviso PRÉ-EXISTENTE visto na corrida impactada** (não é desta linha): dois exemplos
   chamados `probe_cost` (`ph2d-table` e `ph2d-node-source-lsystem`) colidem no `target/ci-test/examples`
   — o cargo avisa que isso *«pode vir a ser erro»*.
