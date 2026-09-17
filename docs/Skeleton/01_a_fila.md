# 01 — A FILA do módulo do esqueleto

> **O que está ABERTO, na ordem em que faz sentido pegar.** Cada item traz o **mecanismo** (ou o
> instrumento que o nomeia numa corrida) e o que as referências fazem — não uma promessa.
>
> ⚠️ **Uma nota de diferido não é uma spec.** O que torna um item desta fila pegável não é ele estar
> escrito: é ele dizer *por onde começar a medir*. Um item sem isso é trabalho a redescobrir.
>
> ⚠️ **A ordem é uma recomendação, não uma decisão.** Quem escolhe é o dono.

---

## ⚠️ ESTE DOC É UM ROTEADOR — a história está ARQUIVADA, verbatim

Ele chegou a **264 KB** (3 608 linhas) por append, uma wave de cada vez, e o joelho medido deste
repo está entre **80 e 110 KB**: acima disso um `Read` deixa de o alcançar e o acesso vira raspagem
por shell — *uma regra na linha 3 000 não é «difícil de achar», ela não é lida por ninguém*
(`CLAUDE.md` §5.0). Em 2026-09-16 ele foi cortado com prova (`scripts/doc-split.py`, remontagem
`sha256` idêntica): o que ficou aqui é **o que está ABERTO** mais o índice das **recusas medidas**.

📚 **As 47 waves fechadas (F1..F6-v) vivem verbatim em**
[`docs/archive/skeleton-fila-2026-09-16/01_a_fila.md`](../archive/skeleton-fila-2026-09-16/01_a_fila.md)
— com o mecanismo, as tabelas e as provas de mutação de cada uma. ⚠️ **Elas são o sítio onde vive
*«medido e REJEITADO»*:** consulte-as (e a tabela de recusas no fim deste ficheiro) **antes** de
propor qualquer mudança de desenho neste módulo.

### ⏳ O que está ABERTO dentro das waves FECHADAS — o endereço de cada um

*Um item aberto dentro de uma wave fechada não deixa de existir por ela fechar.* A lista sai do
arquivo (`grep -n '⏳ \*\*ABERTO' docs/archive/skeleton-fila-2026-09-16/01_a_fila.md`), e cada linha
diz onde ler o mecanismo:

| o quê | onde (linha do arquivo) |
|---|---|
| Com **escala NÃO-UNIFORME** na cadeia o arco do limite distorce-se (é geometricamente correcto; ⛔ não é o que o dono viu) | `497` |
| O **Arrange com pilha**: o `clip_time` responde `None` quando o clip toca zero ou duas vezes — nenhum fantasma, e **não foi smokado** | `2 137` |
| A cerca de *«quem autora no canvas»* vive num FIO e o gate dela é **textual** — a cura de fundo é o `Tool` declarar-se, e ele é contrato **congelado** (§6) | `2 184` |
| Uma cena **muito acima** do orçamento fica com o `Smooth` igual ao `Fast` (aviso único no terminal) ⇒ é o que a **F9** fecha | `3 015` |
| O campo de Hermite amostrado denso ainda **vai e volta `39,67°`** no lado de cima (ondulação abaixo da tolerância) | `3 196` |
| A `ph2d-poly2d` guarda as **duas** leis de refinamento (`PH2D_SKIN_REFINE=uniforme` bissecta) | `3 324` |

---

## Aberto de waves anteriores (as opções que o dono ainda não escolheu)

| # | O quê | Estado |
|---|---|---|
| F3 | **Smart Bones** (Moho) | ✅ **FECHADO** (2026-09-08) — ver abaixo |
| F4 | **Limites de ângulo por junta** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F5 | ~~**Pole target**~~ → **O LADO DA DOBRA** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F6 | **A segunda mídia** (raster/Flip) | ✅ **FECHADA para o RASTER** (2026-09-09) — ver F6 abaixo. ⛔ A nota antiga dizia *«bloqueado: precisa de uma malha sobre a imagem, que não existe»*: estava certa sobre o facto e errada sobre o preço — **duas das quatro peças já existiam**, e o doc de uma delas dizia-o por escrito. O **Flip** continua por fazer |
| **F8** | ✅ **BENDY BONES (B-Bones) — FECHADO em 2026-09-15**, da lei ao painel ([handoff](handoffs/HANDOFF_O_OSSO_QUE_DOBRA_2026-09-15.md)) | Um osso ganha `segments` + duas alças e **arqueia**: ele parte-se em `N` sub-ossos ao longo de uma Bézier, o desenho e o dedo seguem a curva, e o painel oferece os dois controlos. ⭐⭐⭐ **A LEI DA PELE NÃO MUDOU UMA LINHA** — o `Skin` já misturava `N` poses RÍGIDAS por peso, que é exactamente o que um B-Bone é; o que mudou foi **quem produz**, e era **um** sítio (`resolve_with`). ⛔⛔ **E esta célula dizia que o B-Bone «ataca na ORIGEM» a queixa das *«arestas retas ao dobrar»* — REFUTADO** pela recusa medida um bloco abaixo (subdividir com a população de amostras constante **piora**: `2,61 % → 4,94 %` a `24` sub-ossos): *o B-Bone é uma feature de AUTORIA — um rabo em S, um membro flexível —, não a cura da dobra.* ⭐⭐ **O ponto neutro é exacto POR CONSTRUÇÃO** (a fábrica colapsa num osso só quando a curva é recta, e mesmo sem colapsar o frame seria a identidade ao bit) ⇒ todo rig já autorado desenha-se e deforma-se **ao bit** como antes. ⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. **Tecto MEDIDO: `MAX_SEGMENTS = 32`** (`17,9 %` de um quadro com um osso curvo sobre 20 000 pontos; a `64` um par come o quadro) — a tabela vive no doc da const. ✅ **OS TRÊS ABERTOS FECHARAM EM 2026-09-16.** **(1)** O esticão deixou de VARIAR ao longo do osso — os nós saem agora da **CORDA** e não do parâmetro (`12,63 % → 0,000 %` com as alças a `0,2 L`; `82,01 % → 0,000 %` a `0,6 L`; `1 051,95 % → 0,000 %` com as alças cruzadas no eixo). ⛔⛔ **E a cura publicada — equalizar o ARCO — NÃO chegava**, o que só a varredura da densidade disse: ela deixa um piso que **não desce com a tabela** (`1,22 %` a `0,6 L`, igual de `16` a `32` amostras), porque *arcos iguais dão cordas desiguais* e a grandeza que o artista vê é a corda. ⚠️ **E a objecção registada na recusa era verdadeira e não mordia** (*«um somatório de cordas não devolve `L` ao bit»*): o somatório **nunca corre** no ponto neutro — *uma recusa que nomeia um custo tem de dizer em que CAMINHO ele é pago*. **(2)** As alças **pegam-se no canvas** (duas alças de Bézier, com as hastes até à raiz e à ponta) — ⛔ e a armadilha foi que no ponto NEUTRO a alça está **em cima do eixo**, logo a competição por proximidade de sempre torná-la-ia inalcançável no único estado em que todo osso nasce: ela é a única que ignora o corpo, e paga um raio apertado cujo recurso é o comprimento que sobra para o verbo de girar. **(3)** As **tangentes dos vizinhos** existem (`Curve Handles: Manual | From Chain`), e o ponto neutro é **exacto** porque elas saem da transformação RELATIVA e não de uma volta pelo mundo. ⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. Cena **`PH2D_VEC_BONE_SMOKE=1`**. |
| F7 | **O painel próprio do módulo** | ✅ **FECHADO** (2026-09-09, por escolha do dono) — ver F3-m abaixo. A nota antiga: ⏸️ **a condição CAIU e a medição era falsa por ~3×** — ela dizia *«adiado até F3–F5 lhe darem conteúdo (hoje são 3 botões e 5 campos)»*, e as três estão ✅ nesta mesma tabela enquanto a secção tem **10 verbos** e **9 campos** (`VECTOR_BONE_VERBS`/`_FIELDS`, comprimento verificado pelo compilador), mais uma fileira segmentada e dois selectores. ⇒ decisão do dono, não mais um adiamento medido |
| **F9** | ⏳ **A PELE DEFORMADA NA GPU — o `Smooth` a alisar em QUALQUER cena** (pedido do dono, 2026-09-16) | ⏳ **NA FILA** — ver F9 abaixo |
| **F10** | ⏳ **O AutoKey com a corrente de ossos, como o Blender de hoje** (decisão do dono, 2026-09-16) | ⏳ **NA FILA** — medir primeiro, no Blender instalado e corrido por script (§0.9), que ossos recebem chave quando a corrente é movida pela ponta (IK por restrição e *Auto IK*); depois fazer igual |
| **F11** | ⏳ **Imagens em 9 fatias e folhas de quadros DEFORMAM com os ossos** (decisão do dono, 2026-09-16) | ⏳ **NA FILA** — hoje desenham-se sem deformar (aviso no terminal). A malha do bind conhece só o quad da sprite: numa folha ela tem de ser a de UMA célula (a mesma para todas, com a UV da célula viva), e no 9-slice o mapa de UV por pedaços tem de entrar na malha (vértices nas linhas dos cortes) |

---

---

### F6-g — ⏳ **O que sobra do orçamento, MEDIDO: o tecto é um buffer do QUADRO, e as costuras existem em todo modo** (2026-09-13)

**1. O tecto duro.** O Vello guarda a informação de todo desenho num buffer FIXO
(`bin_data = 1 << 18`, *«hand picked»* no `vello_encoding::BufferSizes::new`), e `binning_size =
bin_data − layout.bin_data_start` dá a volta a um `u32` quando passa: **pânico em debug, quadro em
branco em release — painéis incluídos**. Uma peça custa **11 palavras**, linear (gate
`a_skin_piece_costs_eleven_vello_bin_info_words_and_the_cost_is_linear`; sonda do produto
`VectorScene::probe_bin_info_words`).

**2. A GPU**, sonda `ph2d-render::skin_pieces_gpu_cost` (arte opaca `320×96`, alvo limpo por
grelha; zoom 8, carga `2,2`→`9,5`, as últimas linhas de relógio valem pouco):

| peças | quadro | BURACOS | px de costura | alfa mín |
|---:|---:|---:|---:|---:|
| sem recorte | `1,25 ms` | – | – | – |
| `216` (`Fast`) | `1,58 ms` | `0` | `33 476` | `182` |
| `3 456` | `2,46 ms` | `0` | `132 164` | `182` |
| `7 776` | `5,02 ms` | `0` | `259 231` | `171` |
| `17 496` | `10,2 ms`* | `0` | `408 886` | `170` |
| `21 600` | — | **todo o miolo** | — | `0` |
| `≥ 31 104` | pânico no Vello (debug) | | | |

⇒ numa cena **só com a pele** o quadro fica em branco entre `17 496` e `21 600` peças (as 11 palavras
mais a distribuição por bins, que usa o resto do mesmo buffer).

**3. ⛔⛔ As COSTURAS existem em todo modo, o `Fast` incluído:** dois recortes vizinhos com AA
analítico compõem `1 − a·b` na aresta partilhada, e o fundo espreita até **~29 %** (alfa `182`) numa
linha por aresta — `16 580` px a zoom 4 com as `216` peças do smoke.

**4. ✅ A guarda por QUADRO** (o tecto de `1 024` por IMAGEM passava por cima da tolerância — a
`k = 2` o `Smooth` entregava `3,5 px` numa dobra forte contra `0,5 px` pedidos — e não protegia o
quadro: N imagens presas multiplicavam-no). Medido o outro consumidor do buffer, o chrome do editor
pintado sem ecrã pelo registo real (sonda `ph2d-editor-core::vello_bin_budget_of_an_editor_frame`,
texto incluído): **`109`** palavras com os painéis de omissão, **`~1 190`** com todos abertos. ⇒
`SKIN_FRAME_PIECES = (1 << 18) ÷ 2 ÷ (11 + 4) = 8 738`, repartido **proporcionalmente** pelas
imagens presas (o mesmo `k` para todas), e dentro dele **a tolerância decide**. ⚠️ A metade que
sobra é da arte do canvas, que **não foi medida**. Gate
`the_smooth_pieces_of_all_skinned_images_share_one_frame_budget` (visto RED com o tecto por imagem:
`18 t` contra `9 t`). Malhas `Fast` que sozinhas passam do orçamento não têm o que cortar: aviso
único no stderr.

⏳ **ABERTO, e nomeado:**
- ✅ **As costuras — CURADAS pela F6-i.** Sobrepor cada recorte `~0,5 px` foi medido e recusado
  (dobra a composição em arte translúcida ao longo da costura); ficou o **pipeline de triângulos
  texturados** que a F6 nomeou *«com razão medida»* — e ele não pediu pipeline nova: é a malha
  dentro do passe de sprites.
- ✅ **Medido por leitura, e é pior do que a pergunta:** ver a F6-h.

### F9 — ⏳ **A PELE DEFORMADA NA GPU: o `Smooth` a alisar em QUALQUER cena** (pedido do dono, 2026-09-16)

> Perguntado *«para ele alisar em qualquer cena a deformação teria de passar para a placa de vídeo —
> quer que isso entre na fila?»*, o dono respondeu: ***«Quero que isso entre na fila!»***

**O problema, medido (F6-t):** hoje a CPU deforma cada vértice de cada imagem presa a cada quadro, e
o `Smooth` refina a malha na CPU dentro de um orçamento de `5 144` peças por quadro (`1/10` de um
quadro de 60 fps). Uma cena com mais arte presa que isso — um personagem de muitas partes — fica
com o `Smooth` **igual ao `Fast`** (agora de graça, mas sem alisar). Os números: avaliar uma peça
`0,156 µs`, cada peça nova `~0,32 µs`, o `Fast` `0,024 µs` por peça; a GPU desenha centenas de
milhares de triângulos num quadro sem esforço.

**A direcção (a confirmar pela medição da W0, nada disto está decidido em código):**

1. **A densidade sai do quadro e vai para o BIND.** A malha fina é assada uma vez, em repouso,
   onde o CAMPO DE PESOS curva (a mesma lei de Hermite do `Smooth`, medida contra os pesos e não
   contra uma pose) — e fica guardada. ⚠️ A pergunta a medir primeiro: uma malha fixa assada em
   repouso alisa a dobra FORTE como o refinamento por quadro alisa? (a régua existe: a silhueta e a
   faceta de `smoke_bone_paint_silhueta_tests.rs`, com as mesmas barras).
2. **A deformação vai para o *vertex shader*:** por vértice, os índices e pesos dos ossos (enviados
   quando a malha muda); por quadro, só as poses dos ossos (`N × 6` números por esqueleto). ⚠️ O
   `Skin` mistura poses RÍGIDAS por peso, e um B-Bone é `N` sub-ossos — as duas coisas cabem num
   *uniform/storage buffer* de poses, sem lei nova.
3. **A lei da CPU fica como REFERÊNCIA**, e a paridade CPU×GPU é um gate com a barra derivada do
   formato (o molde é o do Flip: `rgba16float` ⇒ `2⁻¹¹`; aqui, posições `f32` em pixels de ecrã).

**As costuras que a W0 tem de mapear antes de qualquer código** (quem lê a malha DESENHADA na CPU,
2026-09-16): o ponteiro (`ph2d_render::mesh_uv`), o `drawn_mesh_of`/`drawn_instance_of` (o anel do
Liquify e da Remoção de fundo, o `CanvasMap`, a caixa do gizmo, a tinta da protecção), os fantasmas
do onion e o `sprite_collect` (a tira do passe de sprites). ⛔ **Nenhum deles pode passar a ler a
malha GROSSA enquanto a GPU desenha a FINA** — seria o *«controlo desenhado por um mapa e agarrado
por outro»* que esta fila já pagou (F6-m). Cada um precisa de uma resposta: CPU da mesma malha fina
só onde se pergunta (um ponto, não a malha inteira), ou leitura da GPU.

**Ondas propostas:**
- **W0 — medir:** o custo e a qualidade da malha fina ASSADA contra o refinamento por quadro (na
  dobra de `25°`/`60°`/`150°`, zoom `1`–`16`); o custo GPU real de `10⁴`–`10⁶` triângulos
  deformados no *vertex shader* nesta máquina; e o censo das costuras acima.
- ✅ **W0 — FECHADA (2026-09-17). As três metades, e uma delas reescreveu a pergunta.**
  1. **A topologia assada serve todas as poses** — medido com CONTROLO (o bind é idêntico nas 5
     dobras, `< 1e-12`), assando no pior caso e re-posando em `5 × 4` células
     (`ph2d-app-vec/src/smoke_bone_paint_assada_tests.rs`, `18aca6a75`). *A direcção da F9 aguenta.*
  2. ⭐⭐⭐ **O CENSO DAS COSTURAS achou o facto que reescreve a W0-b: a malha JÁ vai para a placa
     todos os quadros.** Desde que a pele entrou no passe de sprites, o `renderer_draw` copia o
     `SpriteMesh` para um buffer e desenha — a CPU posa **e faz upload** de `N` vértices por quadro.
     ⇒ a F9 **não acrescenta** um desenho de `N` triângulos: ela TIRA da CPU a deformação por
     vértice e o upload, trocando-o por `N_ossos × 6` números. ⛔ A prosa desta fila listava os
     leitores e a lista estava **incompleta** (faltava a grelha da folha de quadros) — hoje são
     **10**, cada um com a espécie de resposta que vai precisar (**UM PONTO** `O(1)` na CPU · a
     **MALHA** inteira), derivados por
     [`architecture_who_reads_the_posed_skin_mesh`](../../crates/ph2d-editor-core/tests/it/architecture_who_reads_the_posed_skin_mesh.rs)
     — *um leitor novo reprova ali, e não no dia do smoke*.
  3. **O tecto do passe REAL** (`ph2d-render/tests/it/skin_mesh_gpu_ceiling.rs`, `--release`,
     offscreen, mínimo de 5, ⚠️ **`load 7,53`** ⇒ a coluna do relógio pede re-leitura abaixo de `5`):

     | triângulos | upload/quadro | quadro | de `16,67 ms` |
     |---:|---:|---:|---:|
     | `10 082` | `199 KiB` | `0,11 ms` | `0,7 %` |
     | `100 352` | `1,92 MiB` | `0,73 ms` | `4,4 %` |
     | `999 698` | `19,1 MiB` | `10,25 ms` | `61,5 %` |
     | `3 998 792` | `76,3 MiB` | `52,51 ms` | `315 %` |

     ⇒ **o desenho NÃO é o tecto.** O orçamento de hoje (`SKIN_FRAME_PIECES = 8 738` ⇒ `~17 k`
     triângulos) custa à placa `~0,2 %` de um quadro, e `100 k` custam `4,4 %` — **6×** o orçamento
     actual com folga. Quem tem o tecto é a CPU (F6-t: `0,156 µs` para avaliar uma peça, `~0,32 µs`
     por peça nova ⇒ `50 k` peças ≈ `7,8 ms`), que é exactamente o que a F9 remove.
- **W1 — a malha fina no bind**, com a régua da silhueta a mesma de hoje (sem mudar o que se vê).
- **W2 — o *vertex shader* de pele**, atrás da mesma escolha `Fast`/`Smooth` do painel, com o gate
  de paridade CPU×GPU e o caminho da CPU vivo para bissecar.
- **W3 — as costuras** (ponteiro, chrome, onion) contra a malha que a GPU desenha.
- **W4 — o orçamento**: ele deixa de ser um tecto de peças da CPU; o que sobra de CPU por quadro é
  enviar poses, e o recurso passa a ser memória de GPU (com o número medido ao lado).

**⛔ Não é:** subir o `SKIN_FRAME_PIECES` (o recurso dele é o tempo da CPU, e está medido) nem
refinar em *compute shader* por quadro sem primeiro medir a malha assada.

---

## ⛔ Recusas MEDIDAS deste módulo — não as reconstrua

> ⚠️ **As seis de 2026-09-07/08 entraram aqui na auditoria de 08/09** — elas viviam só em prosa e em
> doc-comments, e o §5.0 é explícito: *arquivar sem indexar as recusas seria apagá-las.*

| recusa | o mecanismo MEDIDO |
|---|---|
| **Pôr o `VecDrivenStyle` no ledger de pré-visualização** (o «quinto de cinco» da auditoria de 08/09) | Ele é **desregistado, e não por esquecimento** — não deriva `Serialize`, e o `register_default` exige-o, logo *uma linha de registo não compila*: ele nunca entra no snapshot, no save, nem num passo de undo, que é exactamente o que o ledger compra para os outros quatro. E ele **volta ao autorado TODO QUADRO** (`settle_to_authored`), contra o `release_to_authored`, que só corre quando um motor é desligado. ⇒ acrescentá-lo poria no memo um facto que nunca esteve na fotografia. |
| **Avisar em vez de COAGIR** o nome de um clip a ser único | Os dois nomes são igualmente válidos, então o artista fica com um documento que ele não consegue reparar renomeando — a forma de uma recusa com passos extra. A lei já estava escrita no `doc.rs` e honrada nas duas portas que INVENTAM um nome; faltavam as duas que o RECEBEM. |
| **Roubar `Body`/`Joint` do gizmo de sprite** ao alargar as alças de osso a todo modo de vector | Os dois verbos (girar · deslocar) **já existem** na seta, e agarrá-los aqui trocaria a lei do arrasto dela **em silêncio**: o artista escolhe um osso com a seta e o arrasto passa a fazer outra coisa. A linha é o VERBO — entram só os quatro que nenhuma outra ferramenta sabe exprimir. |
| **Reordenar as secções do painel** para a SKELETON subir quando tem sujeito | Cura o mesmo report que o *revelar-ao-focar* (o cabeçalho a `1316 px` sobre uma faixa de `900`) e muda a ordem do painel para **toda** ferramenta e todo objecto — é decisão de produto, não de correcção, e a revelação é a metade pequena e já precedentada pela timeline. |
| **O *pole target*** para escolher o lado do joelho | Em 3D o triângulo raiz–cotovelo–ponta roda em torno do eixo raiz→ponta — um **grau de liberdade contínuo**, que um objecto no espaço fixa. No plano sobra **UM BIT**. Godot (`flip_bend_direction: bool`) e Spine (`bendDirection ±1`) escolheram o interruptor, cada um por si. ⇒ o alvo de pólo resolveria com um objecto o que um booleano resolve. |
| **Priorizar a ordem no hit-test** para resolver a colisão alça↔ponta | *Não cura: só troca a vítima.* Medido: com o osso na parede a distância ponta→alça é `0,000000`, logo quem quer que ganhe a ordem, o outro fica inalcançável. A cura foi **afastar** a alça (folga derivada do dedo da casa). |
| **Adoptar o clip ABERTO** no *Add Smart Bone* | `TimelineDoc::new()` tem **um** clip, `"Main"` ⇒ todo controlo casava com a animação principal da cena, em silêncio. |
| **Criar uma acção com o nome do osso** no *Add Smart Bone* | Veredito do dono (*«porque criar Bone Action no inspector e na timeline? Melhor não criar nada»*): duas coisas fabricadas por um clique, nenhuma pedida. |
| **Herdar o encaminhamento** pendurando os ids da fileira do lado da dobra na `VECTOR_BONE_VERBS` | Reprovado pelo `table_driven_chips_are_registered_too`: ele exige que o `populate` itere a MESMA tabela que o `paint`, e sem esse laço a fileira seguinte nasce **morta sob o dedo**. |
| **Registar o chip do selector como `Button`** | Mutação medida: o clique **acende e nunca abre lista nenhuma** (`the_action_picker_lists_the_document…` fica vermelho em *«com a lista ABERTA a acção tem de ser pintada»*). É a cicatriz da swatch dos tokens e dos dois números do Input Map. |
| **A *quadtree* graduada** como malha da imagem (F6-b) | Ela deixa **nós pendurados** na transição entre níveis, e um nó pendurado abre **FENDA** numa deformação: ele move-se pelos pesos dele enquanto a aresta do vizinho grosso se move linearmente entre as pontas. Curá-los pede a tabela de moldes de transição (5 casos a menos de rotação). A **grelha-produto** entrega o mesmo adensamento e **CONFORMA por construção** — dois vizinhos partilham a aresta inteira, sempre. |
| **Guardar quadriláteros** em vez de dois triângulos (F6-b) | Um afim não leva um quadrilátero qualquer a outro qualquer: quatro pontos são **oito equações para seis incógnitas**. «Quadmesh» aqui é a DISPOSIÇÃO dos vértices, nunca o primitivo guardado. |
| **Escolher a diagonal da célula pela forma DEFORMADA** (a mais curta das duas — a resposta clássica) | A malha trocaria de diagonal a meio de um gesto ⇒ *o desenho pisca exactamente enquanto o artista dobra.* A diagonal `a–c` fixa-se no **repouso**. |
| **Dilatar cada recorte** para fechar as costuras da pele de imagem (F6-g, 2026-09-13) | Dilatar `0,5 px` (homotetia pelo incentro) fecha a costura em arte OPACA (`16 580 → 0` px com `216` peças) e, em arte TRANSLÚCIDA (alfa `128`), compõe a faixa sobreposta DUAS vezes: `10 580 → 40 050` px com alfa errado e o pior erro `20 → 111` (`3 456` peças: `41 732 → 158 046`). Sombras suaves, bordas anti-aliased e brilhos são translúcidos ⇒ a cura estraga mais do que conserta. Sonda `ph2d-render::skin_pieces_gpu_cost::measure_seams_against_clip_dilation`. A cura que resta é rasterizar a malha SEM AA nas arestas internas. |
| **O sinal de cada junta como restrição DENTRO das varreduras do FABRIK** (a forma clássica; F5-c, 2026-09-14) | **Oscila.** A ida prega a ponta no alvo e re-resolve a corrente inteira sem olhar aos sinais; a correcção desfaz isso. Medido: o erro da ponta **cresce** passagem a passagem em 2 dos 12 alvos do zig-zag (`0,52 → 1,58` num alcance de `3`; `1,87 → 2,00` num de `5`) — e os dois **têm pose exacta**, achada por busca cega sobre os ângulos com os sinais como restrição. A cura é outro solver (descida junta a junta), não outra projecção. |
| **Deitar a junta violada na FRONTEIRA** (a projecção de norma mínima) | Ela move aquela junta o mínimo e custa à CORRENTE o máximo: desfaz a **dobra**, e uma ponta que só se alcança dobrando deixa de se alcançar — `2,43` de erro num alvo a `1,52` de uma corrente de alcance `4`, que tem pose exacta com aqueles sinais. |
| **Amortecer entre a fronteira e o espelho** (`λ · ângulo`, varrido em `0,0 · 0,2 · 0,4 · 0,5 · 0,6 · 0,8 · 1,0`) | Nenhum valor resolve os dois alvos teimosos, e os intermédios são **piores que qualquer um dos extremos** (a `λ = 0,5` três alvos que o `λ = 1` resolve ao bit passam a errar `0,60`–`1,34`). *Não é afinação — é o laço.* |
| **`livre ± 2π` entre os candidatos** do passo do misto | Código defensivo **sem consumidor**: nunca venceu em `900` fixturas, e não pode vencer — a pose de partida é feita dos sinais que dela se leram, logo cada ângulo já está dentro da sua parede e o intervalo vive inteiro dentro de `(−π, π)`, onde o candidato do interior também vive. |
| **Traçar a linha do `IK Chain` pela POLILINHA das juntas** (F5-d, 2026-09-14) | Numa corrente quase esticada ela cai **exactamente** sobre os corpos dos ossos e lê-se como parte deles; numa dobrada, serpenteia. O que o controlo tem de dizer é uma EXTENSÃO, e uma extensão desenha-se como cota: recta e deslocada. |
| **DESLOCAR a recta da corrente para o lado livre** (F5-e, veredito do dono) | A folga contra os ossos em toda pose custa as PONTAS: ela deixa de tocar a junta onde o `Chain` pára e o losango do alvo, e um indicador de extensão que não encosta nas pontas não diz qual extensão é. A corda passa por fora do arco sozinha; em pose esticada a folga vem de a linha ser **fina**. |
| **Deslocar a recta da corrente por uma CONSTANTE** | Não limpa uma corrente que se enrola mais de meia volta: ela tem bojo dos DOIS lados e vem por trás da recta (medido: `18,39 px` de um osso que ocupa `18,75`). O afastamento tem de passar por fora da **excursão** do lado escolhido. |
| **Portar os pesos do Godot** (F6-k, 2026-09-14) | Não há nada para portar: ele **não os calcula**. Só `get/set_bone_weights` e uma acção de painel — *Paint Bone Weights*. |
| **Adoptar os pesos automáticos do Blender** (difusão de calor) | Na nossa fixtura eles dobram `5`–`9 %` da arte acima de `60°`, contra `0,65`–`2,3 %` dos nossos e `0 %` do alcance curado. No nosso meio (folha plana, ossos no plano dela) eles degeneram numa **partição dura** — o método é de outro meio. |
| ⭐⭐⭐ **Os CENTROS DE ROTAÇÃO optimizados** (Le & Hodgins 2016), **re-medidos em 2026-09-15 sobre pesos NÃO degenerados** | ⚠️ **A recusa de 14/09 era *«não dá para julgar por cima de pesos degenerados»* — e essa premissa DISSOLVEU-SE** (arte rígida `37,6 % → 6,6 %`). Re-medido, ele perde na mesma: `11,70 %` de dobra a `90°` contra `8,27 %` do LBS. ⭐ **A causa é OUTRA e é do MEIO:** a nossa arte é uma **folha plana com os ossos a correr pelo meio**, logo o campo de pesos é **simétrico em `y`** — medido, `1 176` pares espelhados com diferença de peso `6,7e-4`. A semelhança do artigo é função **dos pesos e de mais nada** ⇒ não distingue os dois lados, e o centro de ambos cai **no eixo** (`\|y\|` médio `0,0003` numa arte que vai de `−2,4` a `+2,4`; `2 074` de `2 243` vértices a mais de `0,5` do próprio centro). É a limitação que o próprio artigo nomeia — aqui ela **é a forma normal da nossa arte**. ⛔ **Nenhum `σ` cura**: dois pontos com o mesmo vector de peso são indistinguíveis para qualquer função deles. Bancada: `ph2d-skin-weights/src/bancada_centros.rs`. |
| ⭐⭐ **A LEI DO MEIO que as duas recusas acima partilham** | ⛔ *Duas técnicas de topo do campo (difusão de calor · centros de rotação) foram recusadas pela MESMA propriedade da nossa geometria:* uma folha plana com os ossos no plano dela. Qualquer candidata que dependa de **distinguir pontos pelos PESOS** falha aqui, porque os dois lados da folha têm o mesmo vector de peso. ⇒ *sabe-se antes de construir*, e é isso que esta linha vale. |
| **Apertar os pesos** para curar a dobra da pele (F6-j, 2026-09-14) | **Piora, e é a resposta intuitiva:** `0,25 ×` do osso dá `0,64 %` de arte invertida contra `0,17 %` do alcance de hoje. O que dobra a arte é o **gradiente** dos pesos — apertá-los torna-o mais íngreme. Quem cura é ALARGAR: a `2,08 ×` a meia-altura da arte são **zero** pontos invertidos até `150°`. |
| **SUBDIVIDIR o osso (o mecanismo do B-Bone) como cura da dobra** | Com a população de amostras constante ele **piora**: `2,61 % → 4,94 %` a `24` sub-ossos. Com o alcance já certo não cura nada — compra **margem** (`det_min` `0,013 → 0,367`). ⇒ a ordem é o alcance primeiro. ⛔ E a variante «raio encolhe com o sub-osso» lê `0 %` invertido a **`94,5 %` de amostras órfãs**: é a régua a não medir nada. |
| **Ler os lados do modo MISTO da pose VIVA** | Estável enquanto o alvo está ao alcance (o modo é ponto fixo, e há gate) e **apagado para sempre** no primeiro arrasto que o leve para fora dele: fora do alcance a resposta certa é a RECTA, e uma recta não tem lado nenhum para ler. «Inicial» tem de ser o DOCUMENTO. |
| **Fazer a malha SEGUIR a silhueta** em vez de a cobrir (F6-b) | Traz de volta as células deformadas da borda, que são o defeito que a wave cura. O recorte fino é do **alfa da própria arte**, de graça e ao sub-pixel — o *Expansion* do *Puppet* do AE. |


| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
