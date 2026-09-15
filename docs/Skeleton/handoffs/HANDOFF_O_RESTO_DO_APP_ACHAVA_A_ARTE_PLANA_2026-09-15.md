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
