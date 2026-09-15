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

## §8 — O que fica ABERTO

- ⏳ **As guias que são CAMINHO** — a grelha, os contornos de selecção, os selos de operação, o véu
  de humidade e o gizmo de deformação. Elas precisam de **subdivisão** (um segmento recto sobre uma
  dobra não é uma recta), e a porta directa já existe para os **vértices** delas.
- ⏳ **O anel do pincel da Remoção de fundo** deriva o raio do afim do quad: sobre uma dobra a escala
  local é outra. Hoje é um círculo no cursor, e a cura pede a `warp` da malha ali.
- ⏳ **O custo da porta directa é linear nos triângulos** — nomeado no doc dela, com a cura (índice
  por UV) escrita e não construída, porque nenhum consumidor de hoje a alcança.
- ⏳ Os itens que já estavam abertos: **F8 — Bendy Bones** · **F4 — *«undo tem poucos passos»*** (não
  reproduz) · a malha amostrada no EVENTO e não por dab · os traços de FORMA com uma dobra só · o
  `apply_jitter` com o raio inflado.
