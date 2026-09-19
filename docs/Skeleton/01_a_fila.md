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
| ~~Uma cena **muito acima** do orçamento fica com o `Smooth` igual ao `Fast`~~ — ⛔ **a premissa MORREU em 2026-09-17**: não há orçamento por quadro nem duas leis, a densidade é decisão do BIND | `3 015` |
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
| **F9** | ⏸️ **A PELE DEFORMADA NA GPU** (pedido do dono, 2026-09-16) | ⏸️ **PARADA em 2026-09-17, com o gatilho escrito** — a premissa dela (*«o `Smooth` a alisar em qualquer cena»*) foi **refutada por medição** e o botão foi apagado por ordem do dono; o que sobrava é um ganho de RELÓGIO (`~11 %` de um quadro a 8 imagens) e **zero pixels**. Ver F9 abaixo |
| **F10** | ✅ **O AutoKey com a corrente de ossos** (decisão do dono, 2026-09-16) | ✅ **JÁ ESTAVA FEITO — a nota envelheceu, e auditá-la contra o CÓDIGO custou dez minutos** (2026-09-18). O passe grava **a corrente INTEIRA que a mão moveu** (não só o osso seleccionado) desde 2026-09-14, e também **o ALVO de uma restrição de IK** — porque com uma restrição viva a rotação dos ossos é DERIVADA e o que o artista autora é a âncora. ⚠️ Quem filtra é o **DIFF**: um osso cuja pose é a da curva não cunha nada. Seis gates em [`autokey_bone_tests.rs`](../../shells/desktop/src/render_loop/autokey_bone_tests.rs), entre eles `autokey_records_every_bone_the_hand_moved_not_only_the_selected_one`, `dragging_the_ik_anchor_records_the_anchor` e o controlo `a_bone_the_hand_holds_but_did_not_move_keys_nothing`. ⛔ **O que FALTAVA não era a lei, era o SMOKE:** nenhuma cena do app armava o AutoKey, logo o dono nunca lhe chegou ⇒ cena **`PH2D_VEC_BONE_MEDIA_SMOKE=3`** |
| **F11** | ✅ **Imagens em 9 fatias e folhas de quadros DEFORMAM com os ossos** (ordem do dono, 2026-09-17) | ✅ **FECHADO** — ver F11 abaixo |

---

---

### F17 — ✅ **O ENVELOPE SÓ É PINTADO ONDE AINDA MANDA** (ordem do dono, 2026-09-18)

Ele perguntou *«Por que o envelope já não influencia na deformação?»* e a resposta expôs um
**controlo morto**: com os pesos do **padrão-ouro** uma imagem deforma **igual** a `1` e a `2` —
medido, coluna a coluna, na tabela que vive no doc da cena do pincel. Num rig só de imagens aquele
campo aceitava teclas, gravava no documento e **não mudava um pixel**.

⭐ **O envelope não morreu — MUDOU DE DONO:** uma forma **vectorial** presa continua na lei
euclidiana (o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma), e ali ele
manda como sempre. ⇒ o campo **volta** assim que houver uma forma vectorial presa.

⚠️ **A pergunta é da CENA e não do osso, e isso é uma limitação NOMEADA:** o `SkinBind` guarda a
malha e os pesos e **não os ossos**, logo *«este esqueleto tem forma vectorial?»* não é derivável.
A pergunta mais larga erra sempre para o lado **conservador** — *esconder um controlo vivo é pior do
que mostrar um inerte*.

⚠️ **O default publicado é `true`**, e a `limpa()` do arnês repõe-no: sem isso o teste que o desliga
contamina os seguintes, e eles ficam verdes sobre um painel sem aquele campo.

⛔ **E o meu censo de ontem (F16) não o apanhou:** ele mediu o alcance pela lei **euclidiana** (os
pesos por raio), onde ele é vivo — e é falso para uma imagem. *Uma régua que mede a lei antiga não
vê o que a lei nova apagou.*

Mutação **3 de 3** a sangrar, mais uma **inerte de controlo que sobrevive** (o arnês não é
hipersensível).

⚠️ **Promoção pedida à lista de flakes de fan-out do `§5.0`:**
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (`ph2d-tool-painter`) — gate de
RAZÃO, irmã de ficheiro de um membro já listado, reprovou no meio de um fan-out de `15 090` e passa
**3 de 3 a `load 64–74`**, que é carga MAIOR do que aquela em que reprovou, com zero linhas do diff
naquela crate.

### F16 — ✅ **O CENSO DOS NÚMEROS DO OSSO: o valor chega a um CONSUMIDOR?** (2026-09-18)

**O instrumento que faltava, e o veredito é bom: ZERO knobs mortos.** Os sete números do osso chegam
a um consumidor — **seis** à pose que a lei deriva ([`bend::frames`]) e o **alcance** aos pesos da
pele, com prova própria.

⛔⛔ **É a pergunta que o `§5.0` nomeia sobre o repo inteiro** (*«nenhum instrumento pergunta se o
VALOR chega a um consumidor»*) e a família dos dois reports do dono desta semana. Os censos que já
existiam neste painel provam que o clique e o valor **chegam ao barramento**; nenhum provava que
alguma coisa acontece a seguir.

⛔⛔⛔ **E uma varredura por NOME não serve — medido, e teria produzido 12 falsos positivos:** a shell
despacha estes ids **por tabela** (`VECTOR_BONE_BEND_IDS.iter().position(…)`, `BoneKnob::of_id`),
logo um `grep` pelo nome de cada id acusa **12 controlos VIVOS**. *Um id que a régua não vê e um id
morto leem-se igual* — a mesma forma que o `hit_indexed_ids_are_registered` já pagou noutro painel.

⚠️⚠️ **E a régua mentiu DUAS vezes antes de dizer a verdade, as duas por FIXTURA:**
1. Num osso **RECTO** o afim de flexão é a identidade qualquer que seja o comprimento ⇒ o `Length`
   lia-se **morto** sobre produto certo. *Uma régua medida no ponto neutro de OUTRO knob acusa
   este* — o arranjo do censo passa a ter curvatura, e há mutação a prová-lo load-bearing.
2. Com **um** osso a normalização dá-lhe sempre a fatia inteira (`1` contra `1`), e com um segundo
   **fora** do alcance responde o caminho de recurso *«o mais próximo leva tudo»* — o vizinho tem de
   estar **dentro** do alcance para que a razão seja o que se mede.

⭐⭐ **E uma MUTAÇÃO expôs uma cegueira do censo:** trocar um item de `TODOS` por uma cópia de outro
**compila**, mantém o comprimento em `7`, e tira uma variante da população sem ninguém ver. ⇒ o gate
passa a exigir **distintos**, não só a contagem. *Uma lista guardada só pelo tamanho não é uma
população.*

⛔ **O `Strength` é excepção NOMEADA e não uma folga:** o consumidor dele são os pesos, não a pose, e
ele tem gate próprio — *uma célula sem proveniência e uma com proveniência têm o mesmo aspecto numa
tabela*.

Mutação **5 de 5** a sangrar; portão `15 088` verdes.

⏳ **ABERTO:** o censo cobre os **números**; os **verbos** (Bind · Expand · Release · Add/Remove IK ·
Add/Remove Limit · Smart) têm censo de *chegam ao barramento* e **não** de *chegam a um efeito*.

### F15 — ✅ **AS TRÊS RECUSAS DOS VERBOS DO OSSO SOBEM À TELA** (2026-09-18)

**A dívida que a F13 abriu e a F14 herdou, fechada.** As três recusas do botão de osso saíam só no
terminal — *uma recusa que só o terminal vê é um botão mudo* —, e o dono aprovou **dois** smokes em
que foi preciso dizer-lhe *«olhe na janela preta»*.

⭐ **Nenhuma superfície nova:** a [`ph2d_editor_core::ToastQueue`] já servia a irmã desta mesma
família (o *solta-se-sozinho* de uma ferramenta que muda a moldura, com chave de i18n própria). O
`FrameGfx` já a carregava — *a composição já o exprimia, e ninguém tinha medido* (§5.0).

⚠️ **As três juntas, e não só a nova:** curar uma deixaria duas maneiras de responder à mesma
pergunta. ⇒ um enum só (`RecusaDoOsso`, três variantes) e **uma** porta na shell (`avisa`) — com
três `push` espalhados, a quarta recusa nasce muda, que é como estas viveram até aqui.

⭐⭐ **Toda recusa tem chave de i18n, e o `match` da `chave()` é EXAUSTIVO** ⇒ uma variante nova **não
compila** até alguém lhe dar uma. *É a diferença entre uma lista que alguém tem de se lembrar de
estender e uma que não fica verde sem a extensão.*

⚠️ **O terminal FICA ao lado do aviso, e não é duplicação:** um smoke headless não tem tela, e é ali
que a sonda lê. *A tela é para o artista; o terminal é para quem mede* — o mesmo par que o
`PH2D_BONE_LOG` já é.

⛔ **E a agulha de um gate contou `1` de `3` sobre produto CERTO**, pela segunda vez nesta jornada: o
`cargo fmt` parte as chamadas longas em várias linhas. *Um literal lê-se do ficheiro já formatado.*

Mutação **5 de 5** a sangrar; portão `15 084` verdes.

### F14 — ✅ **DESCONECTAR A MALHA DO OSSO numa IMAGEM** (report do dono, 2026-09-18)

*«Acho que ainda não temos a opção de desconectar a malha do osso. Deveríamos ter.»* — **ele tinha
razão, e o defeito eram DUAS metades, ambas mudas.**

⛔⛔ **(a) O painel não sabia.** O facto publicado era um `bool` que só olhava
`self.vec.pen.selected_paths()`, que para uma imagem dá **zero** ⇒ com uma imagem presa escolhida
ele lia `false` e os botões *Expand* e *Release* **nem eram pintados**. *O artista não via um botão
morto — via a ausência de um botão*, que é exactamente o que ele escreveu.
⚠️⚠️ **E o cabeçalho da própria fase já prometia a lei por escrito** (*«se a selecção tem forma
PRESA ou imagem com pele»*): *um doc que declara a lei que o código não implementa lê-se como
auditado.*

⛔⛔ **(b) O verbo não alcançava.** O `release` percorre `paths`; a lei da imagem
([`skin_image::release_image`]) **existia** e o **único** chamador de produto dela era **automático**
(uma ferramenta que muda a moldura solta o osso sozinha). *Uma lei sem gesto é uma lei que o artista
não tem* — a irmã do `dock_columns::close`, que este doc já nomeia.

⇒ o facto publicado passa a ser um **TIPO** (`Skinned { vector, imagem }`, pela lei que o `state.rs`
já escreve para o `BoneSpec`: os campos viajam juntos), o *Release* solta as duas mídias, e o
**`Expand` fica de fora por LEI da mídia** — ele troca o desenho autorado pela geometria deformada
de agora, e uma imagem **não tem geometria autorada** (a malha é derivada da tinta, por quadro).
Assar a deformação nos pixels é **outra** operação, que não existe. ⇒ escondido, não pintado-e-morto.

⚠️ **Duas cercas, não uma:** o painel esconde o *Expand* e o verbo cerca-se a `Keep::Source` — para o
caso de o comando chegar por outra porta.

⚠️ **E uma medição minha falhou por um `head -5`:** li *«`set_current_skinned` só tem chamadores de
teste»* porque a janela cortou a linha da shell. *Um `head` é uma janela, não um veredito* — a lição
já estava na memória do repo, e paguei-a na mesma.

Mutação **5 de 5** a sangrar; portão `15 082` verdes.

⏳ **ABERTO e nomeado:** assar a deformação de uma imagem nos pixels (o *Expand* da 2.ª mídia) não
existe — é wave própria, e só faz sentido com quem a peça.

### F13 — ✅ **VÁRIAS IMAGENS NUM ESQUELETO SÓ (o PERSONAGEM), e o *Bind* deixou de prender ao amálgama** (2026-09-18)

**A capacidade existe, está MEDIDA e é alcançável pelo gesto.** Duas sprites presas ao mesmo osso
semente deformam as duas, com excursões **diferentes** (`1,123 m` / `1,195 m`) ⇒ cada uma tem pele
própria, não é cópia. Cena **`=4`** (`PH2D_VEC_BONE_MEDIA_SMOKE=4`): três desenhos separados, um
esqueleto em **árvore** (tronco + dois membros) — ✅ smoke do dono aprovado.

⛔⛔ **A 1.ª sonda não media partilha nenhuma, e foi uma MUTAÇÃO que o mostrou:** num mundo com um
esqueleto só, `skeleton_of(sim, None)` devolve *«todos os ossos»*, que são os mesmos ⇒ prender ao
seed e prender a `None` dão o mesmo. ⇒ corrente **ISCA** + a grandeza que separa, que é a
**CONTAGEM de ossos do bind** (`3` contra `6`). ⚠️ A distância da isca **não** é load-bearing (a
mutação que a aproxima sobrevive, e está escrito no ficheiro).

⛔⛔⛔ **E isso expôs um defeito de PRODUTO, medido:** o botão *Bind* passa `semente =
osso_selecionado`, que é `None` quando nenhum osso está aceso. Com **dois** esqueletos na cena a
forma ficava presa aos **seis** ossos das duas cadeias, em silêncio, com o log a dizer *«1 imagem
presa»*. ⇒ porta pura [`ph2d_skeleton_live::recusa_do_bind`] — ela **recusa em voz alta** e diz o
gesto que cura (*escolher também um osso na Hierarquia*). ⚠️ **A cerca é o que a torna aceitável:**
com **um** esqueleto o caminho é byte-idêntico ao de sempre; *exigir sempre o osso partiria o fluxo
que o artista já aprendeu, para curar um caso que só existe quando há ambiguidade*.

⭐ A subida à raiz virou porta ([`esqueletos::raiz_do_osso`]) com **dois** leitores — e mudá-la de
sítio **tirou** linhas do `skin_live.rs`, que estava a `697` de um tecto de `700`.

⏳ **DÍVIDA NOMEADA:** esta recusa sai no **terminal**, como as duas que o mesmo botão já tinha.
*Uma recusa que só o terminal vê é um botão mudo* — e curar só a nova deixaria duas superfícies para
a mesma pergunta. **As três sobem à tela juntas**, numa wave com superfície própria.

⚠️ **Três leituras que o diff inverte:**
1. *«zero chamadores de produto de `bind_image`»* — **falso**, era a **fachada** da shell que o grep
   não resolve (`pub(crate) use ph2d_skeleton_live::skin_live::*`). A régua das fachadas erra nos
   **dois** sentidos, e aqui fez ler *«não existe»* sobre algo que existe.
2. *«a subida é comum às três peças»* — **refutado**: os pesos dependem da distância ao osso, e a
   própria sonda já media excursões diferentes.
3. A régua da disposição comparou com o **vão** (centro a centro) quando o que cruza é a **folga**
   (borda a borda) — a foto mostrou três peças sobrepostas **com o gate verde**.

⛔ **E um gate reprovou sobre produto CERTO:** o `the_bind_verb_reaches_both_media` ancorava em
`if pending_bone_bind {`, e a recusa exigiu um bloco rotulado (`'bind: { … break 'bind }`) para não
levar com ela o **soltar** e os **knobs** do mesmo quadro. *Um gate ancorado no idioma reprova no
dia em que o idioma muda* — a afirmação ficou, só a âncora foi curada.

Mutação **11 de 11** a sangrar (6 na cena + 5 na recusa); portão `15 079` verdes.

### F12 — ⏳ **ABERTO e NOMEADO: o *Frame All* enquadra a JANELA, e os painéis tapam-lhe as bordas** (2026-09-18)

⛔⛔ **Não é da pele nem do esqueleto — é do verbo da CÂMERA, e vale para toda a casa.** O
[`drain_view_focus`](../../shells/desktop/src/hero_intents/view.rs) do `ViewFocusKind::All` calcula

```rust
let aspect = window_size.width / window_size.height;      // a JANELA, não o canvas
let need_h = span_y.max(span_x / aspect);
camera.height_world = need_h * 1.1;
```

e o mundo é desenhado na janela inteira com os painéis **por cima**. ⇒ ele enche `110 %` da janela
com o conteúdo e **tudo o que um dock tapa fica fora**.

**Medido** (foto de 2026-09-18, janela `1930 × 1040`): as colunas laterais tapam `~37 %` da largura
e a timeline aberta `~33 %` da altura. Com a timeline aberta a cena `=3` pedia `Frame All` e ficava
a mostrar `±80 px` de mundo sobre um braço de `±120` — **cortado nas duas pontas**.

⚠️⚠️ **E NENHUM tamanho de cena o resolve:** o ajuste é derivado do próprio conteúdo, logo encolher
a cena encolhe o enquadramento junto. *Uma cena larga «não caber» não é propriedade da cena — é
propriedade do verbo.* (A nota da `=1` dizia *«cenas largas nunca cabem»* e tratava-o como lei da
cena; ele é do verbo.)

⭐ **A cura tem endereço:** ajustar ao rectângulo **LIVRE** (a janela menos os docks — os rects já
existem em [`panel_ops::panel_rects`](../../crates/ph2d-editor-core/src/interaction/state/panel_ops.rs))
em vez do da janela. ⛔ **Não foi feita aqui de propósito:** ela muda o enquadramento inicial de
**todas** as cenas de **todos** os módulos, e isso é decisão do dono e da linha da UI, não de uma
linha a meio de uma wave. *Contornar por dentro da minha cena e não dizer nada seria esconder um
defeito que todo artista atinge ao carregar em «Frame All» com a timeline aberta.*

⚠️ **O que a `=3` faz enquanto isso:** não pede `Frame All` (`Prologo::enquadrar = false`) e
dimensiona-se para a **câmera de omissão** (`height_world = 10 m`), com o número derivado dela.

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

### F11 — ✅ **AS 9 FATIAS E AS FOLHAS DE QUADROS DEFORMAM** (ordem do dono, 2026-09-17)

> *«vamos lá: imagens em 9 fatias e folhas de quadros»*

**A medição veio antes da primeira linha** (sonda `sonda_as_tres_formas`, `--ignored`, fica no repo),
com a mesma arte presa a uma corrente dobrada:

| forma | instâncias | com malha | o que se via |
|---|---:|---:|---|
| sprite simples | 1 | 1 | certo |
| folha `4×1` | 1 | **1** | **ERRADO, e calado** |
| 9-slice | 9 | **0** | sem deformar (avisava no terminal) |

⛔⛔ **A folha era o caso PIOR, e não era o que o aviso descrevia.** Ela passa o guarda (a instância
É o quad da sprite) e o defeito estava no BIND: a malha era traçada sobre a folha INTEIRA e o
`pixel_to_local` espremia-a no quad de UMA célula — `1 277` peças recortadas dos quatro quadros
dentro do sítio de um, com a UV de um só esticada por cima. *O 9-slice pelo menos avisava.*

**As duas curas**

- **A folha:** a malha nasce sobre a CÉLULA, e a tinta é a **UNIÃO de todos os quadros** — uma malha
  traçada só sobre o quadro vivo RECORTA todos os outros (prende-se no `0`, dá-se play, e os braços
  do `3` somem). ⭐ Com uma célula só é a identidade byte-a-byte. A porta é a
  [`ph2d_render::SourceCells`], a lei que a shell já tinha **duas** vezes e que desceu ao motor com
  o terceiro leitor.
- **O 9-slice:** a malha é cortada nas linhas das fatias **em pixels da imagem**
  ([`ph2d_poly2d::submesh_in_rect`]) e cada pedaço é esticado no quad DELE. ⛔ Recortar o QUAD está
  refutado por construção: o pedaço do meio mostra a faixa central ESTICADA. A costura é o extract a
  **publicar** a fracção (`SlicePatchSource`) — re-derivar a cadeia região → célula → fatia numa
  segunda casa divergiria no dia em que uma das duas ganhasse uma cerca.

⚠️ **Divergência declarada:** um quad que LADRILHA não repete a silhueta (o `uv_xform` faz a tinta
repetir e a malha é o pedaço único esticado). Numa arte opaca — toda moldura — é invisível.

⚠️ **E o `pixel_to_local` passou a ser o CASO PARTICULAR da régua geral** (`rect_to_quad`), com os 18
gates que já existiam verdes: é isso que prova que a generalização é exacta.

⏳ **Aberto:** a **pré-visualização de uma folha aberta** (o quad desdobrado sob uma ferramenta de
pixels) continua sem deformar, com o aviso — e é desenho: aquele quad não é um pedaço da arte desta
sprite. *Ela já nasce suspensa quando uma ferramenta a está a editar (regra F6-s), então o caso que
sobra é estreito.*

**Smoke:** `PH2D_VEC_BONE_MEDIA_SMOKE=1` — três imagens presas ao mesmo gesto, com o CONTROLO ao
lado. ⚠️ **Sete fotos antes de ir ao dono**, e cinco defeitos que nenhum gate via: ver a mensagem do
commit `02462ca8f` (o enquadramento que nunca cabia · a arte ao contrário · a união com gargalos ·
**configurar depois de prender** · e o toast da cena irmã a nomear a fileira apagada no dia anterior).

---

### F9 — ⏸️ **PARADA POR DECISÃO (2026-09-17): a pele deformada na GPU** (pedido do dono, 2026-09-16)

> ⛔⛔⛔ **LEIA ISTO ANTES DE TUDO O QUE VEM ABAIXO (2026-09-17): A PREMISSA DESTA FILA ESTÁ
> REFUTADA POR MEDIÇÃO.** Report do dono, depois de a porta abrir: ***«Como eu já havia dito muitas
> vezes: Fast e Smooth estão sempre idênticos. Nada mudou»***.
>
> Ele tem razão, e o número é este: medida a distância **em pixels de ECRÃ** entre o sítio onde o
> `Fast` põe cada texto da arte e o sítio onde o `Smooth` o põe, na dobra que a cena ship (`25°`) e
> no zoom `1`, ela é **`0,04 px` na mediana e `0,34 px` no pior ponto**. E o controlo diz o resto:
> o `Fast` está a **`0,33 px`** do campo VERDADEIRO (uma malha `64×` mais fina). *Nenhum olho
> distingue um terço de pixel* — as duas desenham o mesmo.
>
> | dobra/junta | `Fast × Smooth` pior | mediana | `Fast × campo` pior |
> |---:|---:|---:|---:|
> | `25°` (a da cena) | `0,335` | `0,044` | `0,334` |
> | `60°` | `0,775` | `0,100` | `0,771` |
> | `90°` | `1,095` | `0,144` | `1,090` |
> | `150°` | `1,496` | `0,205` | `1,489` |
>
> ⛔⛔ **A premissa do botão MORREU e ninguém reconferiu.** Ele nasceu do report de 2026-09-10
> (*«arestas retas ao dobrar»*), quando a malha do bind era uma **grelha uniforme** e os pesos eram
> **euclidianos**. As duas waves seguintes — a **grelha graduada pelas articulações** (10/09) e os
> pesos do **padrão-ouro com a lei de Hermite** (16/09) — curaram a faceta **na própria malha do
> bind**. ⇒ o `Fast` passou a estar certo e o `Smooth` ficou sem nada para corrigir. *§0.0: quem
> move o número que tornava algo inalcançável tem de reconferir a nota — aqui o número moveu-se por
> baixo de uma feature inteira, e a F9 foi construída em cima dela.*
>
> ⚠️ **O que a F9 construiu continua CERTO e continua a não ser visível:** a malha assada erra
> `2,3×` menos que a do bind, e as duas erram menos de meio pixel. *Uma cura de uma grandeza que já
> estava abaixo do limiar do olho não muda nada no ecrã.*
>
> ⚠️⚠️ **E TODAS as réguas desta linha mediam a grandeza errada** — o desvio ao campo em pixels da
> ARTE, que é uma propriedade da aproximação. O dono vê **pixels de ECRÃ**. O gate que fixa isto é
> `o_fast_ja_desenha_o_campo_a_menos_de_meio_pixel` (`ph2d-app-vec`), com as duas metades: o `Fast`
> está certo **e** o `Smooth` separa-se num regime real (`150°` com zoom `8`), que é o que impede
> alguém de ler isto como *«apague o botão»* — essa é decisão do dono.
>
> ✅ **O DONO DECIDIU no mesmo dia: *«1- Pode apagar a seção deform. 2- Escolha o melhor a fazer»*.**
>
> **(1)** A fileira foi **APAGADA**, e com ela o `SkinDeform` inteiro — enum, campo, as duas rotas de
> clique, os dois ids, os dois espelhos da shell, as três chaves de texto e os dois gates de costura.
> ⚠️ *Retirar o gesto retira a CAPACIDADE:* deixar a lei viva e inalcançável é o defeito que este
> repo já pagou, e por isso ela não ficou a dormir. A fileira do painel dá lugar a um bloco que diz
> **porque** ela saiu, com a medição ao lado.
>
> **(2)** A lei que fica é **sempre a malha ASSADA no bind** (`ph2d_skeleton_live::skin_bake_cache`),
> e a escolha é medida nas duas colunas: ela erra o campo **menos** que a malha crua **e** custa
> menos (`2,7 ×` mais peças por `5,6 ×` menos relógio — refinar `~0,32 µs`/peça contra desenhar uma
> peça já fina, `~0,017 µs`). ⇒ morreram com ela o repartir do orçamento e o aviso de malha acima
> dele: os dois existiam para governar um refinamento **por quadro** que já não acontece.
>
> ⏸️ **E a F9 PÁRA aqui, com o gatilho escrito.** O que sobrava dela era a metade 2 (a deformação no
> *vertex shader*), e ela **não compra um pixel**: o ganho medido é de RELÓGIO, `~11 %` de um quadro
> a 8 imagens presas. *Não se gasta uma wave a comprar 11 % de um quadro que hoje sobra.*
>
> ⏳ **O gatilho para a reabrir** (qualquer um dos três, e todos são MEDIÇÕES, não palpites):
> uma cena do dono onde a pele passe do orçamento de peças e o log (`PH2D_BONE_LOG=1`) o mostre ·
> um report de engasgo cuja sonda aponte para o `attach_skin_meshes` · ou a arte presa passar de
> `~8` imagens por cena. O desenho está escrito abaixo e continua válido — ⚠️ com **uma** correcção
> já medida: os `@location` 0..15 do *vertex* estão CHEIOS, logo os pesos têm de chegar por
> *storage buffer* indexado pelo `@builtin(vertex_index)`, nunca por um atributo novo.


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
- ✅ **W1 — FECHADA (2026-09-17). ⚠️ Ela fechou com a porta DESLIGADA e o número que dizia que ela
  tinha de ficar assim; a medição do custo por quadro REFUTOU esse número no mesmo dia e a porta
  ABRIU — ver a W2c abaixo.**
  - **A porta**: [`ph2d_poly2d::refine_rest_by_attrs`] refina a malha de **repouso** onde o campo de
    atributos curva (`Σ_j |w_j(meio) − w̄_j|`), sem pose nenhuma; e
    [`ph2d_skeleton_live::skin_bake::assar`] liga-a aos pesos BBW do bind. `PH2D_SKIN_BAKE=1` abre.
  - ⭐⭐⭐ **A conta que sustenta a F9 está escrita e CORRIDA:** com ossos afins,
    `P(meio) − corda = Σ_j Δw_j · T_j(meio)` ⇒ *a única coisa não-linear numa aresta é o PESO*, e
    assar com tolerância `τ` garante `|erro| ≤ τ · dispersão` em **toda** pose. É o mecanismo por
    trás do que a W0 mediu.
  - ⛔⛔ **A tolerância que eu tinha escrito era INERTE, e foi a arte REAL que o disse:** `0,02`
    saía de uma fixtura sintética, e na cena do dono o pior desvio de peso de toda aresta já é
    **`0,0154`** — a malha do bind **já é graduada pelas articulações** (wave de 10/09), logo a
    densidade já está onde o campo vira. ⇒ a tolerância passa a ser **DERIVADA**
    (`0,5 px / diagonal da arte`), que é a mesma barra que o `Smooth` do quadro promete.
  - **Medido na arte do dono** (`512 × 320`, `2 430` peças, BBW por 3 ossos), com `τ = 8,3e-4`:

    | desenho | peças | desvio ao CAMPO |
    |---|---:|---:|
    | `Fast` (o bind de hoje) | `2 430` | `0,4143 px` |
    | `Smooth` (do quadro, zoom `8×`) | — | `0,0881 px` |
    | **assada** | **`13 996`** (`5,76×`) | **`0,1781 px`** |

    ⇒ a assada erra `2,3×` menos que o `Fast` e fica dentro da barra de `0,5 px`. ⚠️ Ela erra `2×`
    mais que o `Smooth` **por desenho**: aquele refina para ESTA pose e este zoom, e a assada é
    independente da pose — *uma aproximação que serve todas nunca bate, peça a peça, uma feita para
    uma só*.
  - ⛔⛔ **E a frase que estava aqui — *«é o `5,76×` que PROVA que a porta fica fechada até à W2»* —
    foi REFUTADA no mesmo dia (W2c):** o `SKIN_FRAME_PIECES` é um tecto de **REFINAMENTO**, e uma
    malha já assada **não refina**. *Comparar uma contagem de peças com um orçamento cuja unidade é
    «peças que a lei pode PARTIR» é somar duas grandezas diferentes* — e o resultado dessa soma
    mandava fechar a porta que a medição mandou abrir. Na placa, `13 000` triângulos custam
    `~0,15 %` de um quadro (a tabela da W0-b). **A assadura não é cara; caro é deformá-la na CPU —
    e mesmo isso cabe (`1,4 %` numa imagem).**
  - ⚠️⚠️ **A régua da silhueta NÃO serve para comparar densidades diferentes** (a fila pedia-a, e a
    medição refutou o pedido): o «vai-e-volta» soma a viragem absoluta da polilinha, logo **cresce
    com o número de nós por construção** (`Fast` `26,60°` com 46 nós · `Smooth` `26,71°` com 64 ·
    assada `29,44°` com 85). A régua com unidade e barra declarada é o **desvio ao campo**.
- ⏳ **W2 — o *vertex shader* de pele**, atrás da mesma escolha `Fast`/`Smooth` do painel, com o gate
  de paridade CPU×GPU e o caminho da CPU vivo para bissecar.
  - ✅ **A metade da CPU FECHOU (2026-09-17)** — [`ph2d_skeleton_live::skin_gpu`]: o empacotamento e
    a **lei de referência** (`posa_como_a_placa`), que é o que **define** o shader e contra o que a
    paridade se vai medir. Ela reproduz a lei do produto a **`1,4e-5`** contra uma barra derivada de
    `4 ULP` de `f32` na magnitude em jogo (`7,6e-5`), com o controlo dentro (poses erradas violam-na
    `100×`).
  - ⭐⭐⭐ **E o achado que torna o shader TRIVIAL:** a quota que reparte o peso de um tendão pelos
    sub-ossos de um osso que dobra depende de `u = projecção do ponto no eixo de REPOUSO` ⇒ ela é
    uma grandeza do **BIND**. Logo a tabela de pesos **por osso, já normalizada**, só muda quando a
    TOPOLOGIA do rig muda — nunca quando o artista posa. ⇒ *o shader não precisa de saber o que é
    um osso que dobra*: ele lê `N` pesos por vértice e `N` afins por quadro, e a mistura é a linear
    clássica. Gate `mover_um_osso_nao_muda_a_tabela_de_pesos`.
  - ⚠️⚠️ **E a 1.ª fixtura destes gates tinha a corrente toda RECTA — MEDIDO, ela deixa a mutação
    que apaga a quota passar em TODOS os três gates.** Com um osso que dobra, ela sangra. *Uma
    fixtura no ponto neutro de uma lei não testa essa lei.*
  - ✅ **A W2b FECHOU: a malha assada é DERIVADA, e o painel continua a escolher** (2026-09-17) —
    [`ph2d_skeleton_live::skin_bake_cache`]. A W1b assava **dentro do `bind_image`**, substituindo a
    malha guardada, e isso é de PRODUTO e não de relógio: ⛔ o `Fast` deixava de ser barato (passava
    a desenhar a malha `5,76×` maior), a escolha `Fast`/`Smooth` **colapsava** (as duas desenham a
    mesma malha) e a densidade ficava **congelada no ficheiro**. ⇒ a assadura sai do documento e
    passa a viver num **memo por bind**; o `Smooth` consulta-o, o `Fast` não passa por lá.
    ⭐⭐ **É o mesmo memo que a placa vai querer** — quando o *vertex shader* posar, o que sobe uma
    vez por bind é exactamente esta malha (repouso + tabela de pesos). *A casa é a mesma; muda quem
    a lê.*
    - ⚠️ **A chave é a ENTIDADE e a prova é o CONTEÚDO:** `Entity::to_bits()` é só o ENDEREÇO da
      gaveta (o degrau 122 da escada já escreveu porque ele não serve como identidade durável), e
      quem diz se o conteúdo serve é a **igualdade byte a byte** da fonte. ⛔ Uma função de dispersão
      criptográfica seria **dez vezes mais cara** que a prova exacta (`~100 KiB` de bind: memcmp
      `~10 µs` contra SipHash `~100 µs`) — *uma chave derivada só compensa quando comparar o
      original é caro.*
    - ⚠️ **O `None` também é guardado** — com a porta fechada ele é a resposta de toda a arte, e sem
      o guardar o caminho de omissão pagaria uma tentativa por imagem por QUADRO.
    - ⚠️⚠️ **O aviso de orçamento partiu-se em DOIS, porque a mesma condição passou a ter
      significados OPOSTOS:** sem assadura ela é um AVISO (*o botão que o painel diz ligado desenha
      o que o `Fast` desenha*); com assadura ela é a wave a **funcionar** (a densidade veio do bind,
      e não haver refinamento por quadro é o que a torna independente do tamanho da cena).
    - ⛔⛔ **Duas mutações SOBREVIVERAM primeiro, as duas a acusar código meu:** o `filter` que
      protegia a gaveta recém-assada do despejo era **inerte** (com `visto = agora` ela nunca pode
      ser o mínimo) — *uma linha que a mutação não consegue matar não é lei, é comentário com
      sintaxe de código* —, e o refresco do relógio no ACERTO não tinha régua nenhuma, logo o memo
      era **um FIFO com o nome de cache**. ⚠️ E a mutação que morde a primeira só é observável num
      gate cuja ordem de ENTRADA discorda da ordem dos BITS, que é o caso normal.
    - ⚠️ **A premissa de um gate MORREU e ele foi reescrito com a morte visível no diff:**
      `o_bind_da_imagem_chama_o_assador` afirmava o CONTRÁRIO do que hoje é verdade ⇒
      `o_assador_tem_um_chamador_e_ele_nao_e_o_bind`, com as duas metades.
    - **7 gates · 7 mutações, todas sangram.**
  - ✅⭐⭐⭐ **A W2c FECHOU, e é ela que responde ao pedido do dono: A PORTA ABRIU** (2026-09-17) —
    `PH2D_SKIN_BAKE=0` passa a ser a porta de **bissecar**, e o caminho de omissão do `Smooth` é a
    malha ASSADA. **Medido na arte do dono** (zoom `8×`, `N` cópias, o MÍNIMO de 30, `load 4,6`):

    | imagens | lei | porta | peças entregues | ms | % de um quadro |
    |---:|---|---|---:|---:|---:|
    | 1 | `Fast` | — | `2 430` | `0,056` | `0,3 %` |
    | 1 | `Smooth` | **fechada** | `5 143` | `1,244` | `7,5 %` |
    | 1 | `Smooth` | **aberta** | **`13 996`** | **`0,226`** | **`1,4 %`** |
    | 4 | `Smooth` | **fechada** | `9 720` ⇐ **é o `Fast`** | `0,228` | `1,4 %` |
    | 4 | `Smooth` | **aberta** | `55 984` | `0,903` | `5,4 %` |
    | 8 | `Smooth` | **fechada** | `19 440` ⇐ **é o `Fast`** | `0,459` | `2,8 %` |
    | 8 | `Smooth` | **aberta** | `111 968` | `1,824` | `10,9 %` |

    ⭐⭐⭐ **Numa imagem a assadura é `5,5×` MAIS BARATA e entrega `2,7×` MAIS peças** — *refinar* uma
    peça custa `~0,32 µs` e *desenhar* uma peça já fina custa `~0,017 µs` (números da F6-t, que
    ninguém tinha composto). ⛔⛔ **E as linhas de `4` e `8` com a porta fechada são o report do dono
    reproduzido ao número:** `peças(Smooth) == peças(Fast)`.
    - **O gate que é a F9 numa asserção:** `o_smooth_alisa_em_qualquer_cena` — *a malha assada é um
      CHÃO que o tamanho da cena não consegue erodir*, com as três metades (a cena **contém** o
      fenómeno · o `Smooth` entrega **estritamente** mais que o `Fast` · e entrega pelo menos o
      chão, senão ele degrada em vez de sumir). ⚠️ O chão sai da **lei do produto**, nunca de um
      número escrito no gate — a tolerância é derivada da diagonal da arte. **3 mutações, todas
      sangram** (a porta fechada · o `Smooth` sem consultar o memo · a tolerância de volta ao `0,02`
      que a arte real já tinha refutado).
    - ⚠️ **Assar custa `3,9 ms`, UMA vez por bind**, ao lado do solver BBW que o mesmo `bind_image`
      já paga, e **fora** do quadro.
    - ⚠️ **DUAS premissas morreram com a morte visível no diff:** a porta nascer desligada, e o
      *«sem espaço no orçamento o `Smooth` desenha o `Fast` AO BIT»* — hoje ele desenha a **assada**,
      que é o ponto.
  - ✅⭐⭐ **E A CENA PARA SE VER ISSO EXISTE** (2026-09-17): `PH2D_VEC_BONE_PAINT_SMOKE` deixou de
    ser um interruptor e o nível dele é uma **CONTAGEM de canvas** — `=1` é a cena de 8 passos que o
    dono já aprovou, **byte-idêntica**; `=3` ou mais põe a cena acima do orçamento do quadro, que é
    o regime do report. ⚠️ *Sem ela a cura estava gateada e invisível, e uma cura que ninguém pode
    ver é uma cura que ninguém julga.*
    - ⛔⛔ **A FOTO (`fotografa_cena.sh`) apanhou DOIS defeitos de cena que gate nenhum via:** a
      1.ª disposição era uma COLUNA e a arte dobrada **varre para cima** muito além da caixa de
      repouso, logo o canvas de cima ficava sempre cortado — *nenhum valor do espaçamento serve,
      logo o que estava errado era a disposição* (hoje é uma FILEIRA: dobrar **encurta** a pegada
      horizontal); e o enquadramento `All`, que eu tinha posto para caber a cena inteira, ajusta-se
      às CAIXAS das sprites e cortava as pontas de qualquer maneira. ⇒ volta ao `Selected`, e a
      leitura muda com ele: *os outros canvas existem para ENCHER o orçamento, não para serem vistos
      ao mesmo tempo.*
    - ⚠️ E o roteiro passou a ser **outro** conforme a contagem: *«um roteiro que tenta ensinar as
      duas coisas manda o dono fazer oito passos para chegar ao que ele foi ver».*
  - ⏳ **O que falta da W2 (a placa), e o que já está medido sobre isso:**
    - o **formato de vértice**: o [`ph2d_render::QuadVertex`] é **partilhado com o quad simples**
      (`pos` + `uv`, 16 bytes), logo acrescentar-lhe pesos paga em toda sprite do app ⇒ ou um
      segundo *layout*/pipeline, ou um buffer à parte indexado pelo vértice. ⚠️ Medido na arte do
      dono: `3` tendões e **nenhum vértice esparso** (`139` vértices usam 1 osso, `662` usam 2,
      `487` usam 3) — *num rig pequeno não há esparsidade a explorar, e um `K = 4` fixo do formato
      da indústria seria um TECTO a justificar, não um ganho*;
    - o **buffer por-BIND com invalidação DO LADO DA PLACA** (hoje o `MeshFrame` é reconstruído do
      zero a cada quadro) — é ele que troca o upload de `19 MiB/quadro` por `N × 6` números. ⭐ A
      metade da CPU já existe (a W2b); o que falta é o `MeshFrame` deixar de ser por-chamada;
    - ⛔ **e o formato NÃO pode crescer por atributo de vértice:** o `pipeline.rs` declara por
      escrito que *«o limite de 16 atributos do dispositivo (`@location` 0..15) está cheio»* — a
      `InstanceInput` ocupa `2..15` e o `QuadVertex` o `0..1`. ⇒ os pesos por vértice entram por
      **storage buffer** indexado pelo `@builtin(vertex_index)` (que numa chamada não-indexada é o
      índice ABSOLUTO no buffer, logo um vector paralelo ao dos vértices costurados resolve sem
      offset nenhum), e não por um atributo novo;
    - as **10 costuras** do censo da W0, cada uma com a espécie de resposta já escrita.
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
| **Apertar os pesos** para curar a dobra da pele (F6-j, 2026-09-14) — ⚠️ **a recusa vale; o NÚMERO dela é da lei ANTIGA** (ver a linha seguinte) | **Piora, e é a resposta intuitiva:** `0,25 ×` do osso dá `0,64 %` de arte invertida contra `0,17 %` do alcance de hoje. O que dobra a arte é o **gradiente** dos pesos — apertá-los torna-o mais íngreme. Quem cura é ALARGAR: a `2,08 ×` a meia-altura da arte são **zero** pontos invertidos até `150°`. |
| **SUBDIVIDIR o osso (o mecanismo do B-Bone) como cura da dobra** | Com a população de amostras constante ele **piora**: `2,61 % → 4,94 %` a `24` sub-ossos. Com o alcance já certo não cura nada — compra **margem** (`det_min` `0,013 → 0,367`). ⇒ a ordem é o alcance primeiro. ⛔ E a variante «raio encolhe com o sub-osso» lê `0 %` invertido a **`94,5 %` de amostras órfãs**: é a régua a não medir nada. |
| **Ler os lados do modo MISTO da pose VIVA** | Estável enquanto o alvo está ao alcance (o modo é ponto fixo, e há gate) e **apagado para sempre** no primeiro arrasto que o leve para fora dele: fora do alcance a resposta certa é a RECTA, e uma recta não tem lado nenhum para ler. «Inicial» tem de ser o DOCUMENTO. |
| ⭐⭐⭐ **A DOBRA SOB A LEI DE PESO DE HOJE, RE-MEDIDA** (2026-09-18) — *não é uma recusa, é a reconferência que o §0.0 exige* | ⛔⛔ A recusa acima diz *«zero pontos invertidos até `150°`»* e mede a lei **derivada por distância**; o bind passou ao **padrão-ouro (BBW)** em **15/09** e **ninguém reconferiu**. Re-medida pela porta do produto sobre a arte do braço da cena `=2` (sonda `sonda_da_dobra`, `ph2d-skeleton-live`): a lei **continua de pé** — `0` triângulos do avesso a `0/13/25/50/75°` por junta, e a **primeira** inversão a `90°` (`19` de `3 593`), com a corrente dobrada por completo sobre si. Pior factor de área: `1,44 · 1,22 · 0,99 · 0,49 · 0,03 · −0,18`. ⇒ **gate** `a_pele_nao_vira_um_triangulo_ate_setenta_e_cinco_graus`, com o controlo positivo a `90°` dentro dele. ⛔⛔ **E uma nota de PRODUTO caiu junto:** a cena `=2` dobrava `13°` por um doc meu que dizia que a `25°` *«a malha dobra sobre si mesma e a arte lê-se RASGADA»* — **falso**; o rasgo da foto eram os gargalos da união dos quadros e o configurar-depois-de-prender, os dois curados na mesma jornada. *Baixar o ângulo fez o sintoma encolher, e por isso pareceu uma cura.* A cena volta a `25°`. |
| **Fazer a malha SEGUIR a silhueta** em vez de a cobrir (F6-b) | Traz de volta as células deformadas da borda, que são o defeito que a wave cura. O recorte fino é do **alfa da própria arte**, de graça e ao sub-pixel — o *Expansion* do *Puppet* do AE. |


| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
