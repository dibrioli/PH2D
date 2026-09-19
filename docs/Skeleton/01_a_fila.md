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
| **F21** | ✅ **A cena dedicada do ENVELOPE** (*«melhor montar uma cena específica para me mostrar isso»*, 2026-09-18) | ✅ **FECHADA em 2026-09-19 — e ela REFUTOU a lei da F20**: o envelope é inerte em toda forma FECHADA (amplitude `0,000000` numa faixa de `80 ×`), porque uma forma fechada também usa o padrão-ouro desde 15/09. A lei passou a perguntar ao **BIND** e não à mídia. Cena **`PH2D_VEC_BONE_SMOKE=2`** — ver F21 abaixo |
| **F22** | ⭐⭐⭐ **A ESCOLHA da lei de pele, POR DESENHO** (ordem do dono, 2026-09-19: *«construa. por desenho»*) | ✅ **FECHADA no mesmo dia** — fileira **`Deform By`** (`Artwork` \| `Bone Reach`) no painel Bones, por DESENHO e para as duas mídias. ⭐ A escolha diz se o quadro **LÊ** a tabela do padrão-ouro, nunca se a calcula ⇒ a volta é **exacta ao bit** e não re-resolve nada. `PROJECT_SCHEMA` **+1** — ver F22 abaixo |
| **F30** | ⭐⭐⭐ **A arte segue o peso ENTRE os nós** (a 2.ª saída da F26) | ✅ **CONSTRUÍDA, e a MALHA não foi precisa.** A `ph2d-vec-envelope` já deforma Bézier por um mapa não-afim, e o cabeçalho dela descreve o defeito que a pele tem hoje. Sonda: peso entre dois nós move a arte `0,000000 → 0,242375`, o fit converge, `0,163 ms` em release. ⭐⭐⭐ E ela **dissolveu a compensação da F28** — ver F30 abaixo |
| **F29** | ⏳ **Os DOIS modos de atribuir peso** (ordem do dono, 2026-09-19) | ⏳ **ABERTO, na fila.** *Absoluto* (o valor entra e o resto reparte-se pelos outros ossos na proporção deles; Add/Subtract inactivos) e *Cumulativo* (o de hoje). ⚠️ **Não é UI: a correcção é uma mancha que SOMA**, e uma absoluta não é um campo somável — ver F29 abaixo |
| **F28** | ⭐⭐⭐ **UM PONTO NOVO NUMA FORMA PRESA** (a 1.ª das duas saídas da F26, escolhida pelo dono: *«primeiro 1 e depois o 2»*) | ✅ **FECHADA**, e o smoke dela REPROVOU a 1.ª versão. O ponto sobrevive ao quadro, já nasce com peso, **o desenho não salta** (`18,89 % → 0,000000 %`) e a caneta MOSTRA onde o clique poria o nó. ⛔⛔ Duas conclusões minhas caíram: *«custo zero de arquitectura»* (medido: o ponto evaporava-se) e *«o salto é refinamento»* (o dono recusou — ver F28-b) |
| **F27** | ⭐⭐⭐ **O CENSO DOS VERBOS DO OSSO** (o aberto que a F16 deixou por escrito) | ✅ **FECHADO no mesmo dia — ZERO verbos mortos.** Os catorze botões chegam a um efeito, medidos pela captura que o undo tira. ⛔⛔ E uma **mutação sobreviveu**: apagado o corpo do braço do *Add Smart Bone* na fase do quadro, **23 testes da shell ficaram verdes** — o terceiro elo do §5.0 não tinha instrumento nenhum. Zero schema, zero registo — ver F27 abaixo |
| **F26** | ⭐⭐⭐ **CORRIGIR UM PESO À MÃO** (auditoria, 2026-09-19) | ✅ **FECHADA no mesmo dia** — o 3.º verbo do osso (**`Weight`**) pinta a influência sobre a arte presa, com os pesos **à vista** por baixo do pincel. A correcção é uma **MANCHA no espaço** (nunca uma tabela por vértice) e é ancorada no **REPOUSO** do ponto que o dedo aponta. `PROJECT_SCHEMA` **+1** — ver F26 abaixo |

---

---

### F20 — ✅ **O GIZMO DO ENVELOPE SÓ EXISTE ONDE ELE MANDA, e agora POR OSSO** (report do dono, 2026-09-18)

*«O gizmo do envelope fica sempre visível mesmo quando não é usado?»* — **sim, ficava.** A F17 curou
o CAMPO do painel e deixou a **mancha** e a **alça** no canvas.

⭐⭐ **A lei entra na [`influence_region`] e não em quem desenha, porque essa porta tem DOIS
consumidores** — o desenho da mancha e o **hit-test da alça**. *Curar só o pintor deixaria o artista
a arrastar uma alça invisível, que é pior do que a mancha a mais.*

⛔⛔⛔ **E a pergunta passou de CENA para OSSO, porque uma premissa MINHA caiu.** Eu escrevi que *«o
`SkinBind` guarda a malha e os pesos, **não** a que ossos ficou preso»* — e ele guarda
(`Tendon::bone`, um `StableId`). Com a pergunta larga, numa cena **mista** a mancha acendia em
**todos** os ossos — e foi **a cena que o dono pediu** que expôs isso, antes de ela existir.

⭐⭐⭐ **E a medição que explica o resto do report** (*«não vi em nenhum dos casos o envelope fazer
diferença na deformação»*): com **UM** osso o envelope é **INERTE** — os pesos renormalizam e o
único osso leva sempre a fatia inteira (`[1.0]` a `0,3` **e** a `4,0`). Com **três**, ele manda
(`[1, 0, 0]` → `[0,42, 0,58, 0]`). *O alcance só decide quando DOIS ossos disputam o mesmo ponto.*
⚠️ ⇒ a porta é **necessária e não suficiente**: ela esconde o caso claro e mostra o resto, que é o
lado conservador, e o limite está escrito nela.

⛔ **A lei por CENA foi APAGADA, não guardada** — ficou sem chamador no instante em que a por osso
nasceu, e *uma lei viva que nenhum gesto consulta é uma lei órfã*.

⛔⛔ **E um gate de OUTRO assunto reprovou, com razão:** o `when_two_handles_overlap_the_nearer_one_wins`
construía a sobreposição a partir da região, e a fixtura dele **deixou de conter o fenómeno** quando
a lei mudou. *A cura é da fixtura, nunca da lei* — ela ganhou uma forma vectorial presa ao osso.

⛔ Tecto de LOC (`707` contra `700`) curado por **CORTE**: a mancha mudou de **ficheiro e não de
endereço** (`pub use`), a mesma lei que o `bend_live` já aplica no mesmo sítio.

Mutação **4 de 4** a sangrar; portão `15 101` verdes.

✅ **A CENA DEDICADA EXISTE — e ela REFUTOU a lei que esta secção acabara de shipar.** Ver **F21**.

### F21 — ⭐⭐⭐ **O ENVELOPE MANDA ONDE O PADRÃO-OURO NÃO RESOLVEU — a MÍDIA nunca foi a pergunta**
(a cena que o dono pediu, 2026-09-19)

⛔⛔⛔ **A minha resposta ao dono estava ERRADA, e a F20 shipou a lei errada por cima dela.** Eu
disse-lhe que *«numa forma vectorial o envelope manda como sempre»*, com o argumento — escrito no
doc da porta — de que *«o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma»*.
⚠️ **Essa premissa expirou em 2026-09-15**, quando o `ph2d_vec_skin::pesos::pesos_do_caminho` passou
a construir a malha do **INTERIOR** de um contorno fechado e a resolver os mesmos BBW. *Quem move o
número que tornava algo inalcançável tem de reconferir a nota* (§0.0) — e ninguém reconferiu.

**Medido pela porta do produto** (`sonda_do_envelope_no_vector_tests`, `ph2d-skeleton-live`),
variando o `strength` do osso do meio de `0,1` a `8,0` — uma faixa de **`80 ×`**:

| forma | fechada? | amplitude da deformação |
|---|---|---:|
| `Rectangle` · `Ellipse` · `Star` · `Polygon` · `Segment` · `Pie` | fechada | **`0,000000`** |
| `Line` | ABERTA | `2,03` |
| `Arc` | ABERTA | `4,25` |
| `Spiral` | ABERTA | `2,05` |

⇒ **o dono tinha mais razão do que a minha resposta lhe deu:** o envelope é inerte em **toda** forma
preenchida e em **toda** imagem que resolve. Ele manda num sítio só — onde a tabela de pesos do bind
está **VAZIA**, porque um caminho **ABERTO** não tem interior, logo não tem domínio para a energia.

⭐⭐ **A lei passou a perguntar ao BIND** (`caiu_na_lei_derivada`): *este bind guarda a tabela do
padrão-ouro?* As duas mídias respondem pela mesma porta — uma [`SkinnedMesh`] e um [`SkinnedPath`]
guardam a MESMA coisa —, e a mídia entra só para **escolher o descodificador**. ⛔ Uma `source` que
nem descodifica responde **não**: ali o quadro pula a pele, e acender a mancha seria prometer um
efeito que não existe.

⭐⭐⭐ **E a CENA é `PH2D_VEC_BONE_SMOKE=2`** — três fileiras, a mesma corrente de três ossos, a mesma
dobra; só muda o alcance do osso do meio:

| fileira | o que é | o envelope |
|---|---|---|
| `Corda (alcance 1)` | um traço ABERTO, alcance de fábrica | VIVO — mancha e alça |
| `Corda (alcance 4)` | o MESMO traço, alcance `4` no osso do meio | VIVO — e a corda acaba **noutro sítio** |
| `Barra preenchida` | o MESMO arco, fechado pela corda | INERTE — sem mancha, sem alça |

⚠️ **A env `PH2D_VEC_BONE_SMOKE` era de PRESENÇA e passou a ter níveis** (`NIVEIS = 2`): ilegível ou
ausente ⇒ `1`, a cena que o dono já aprovou.

⛔⛔ **A FOTO apanhou QUATRO defeitos que os gates não podiam ver** (`fotografa_cena.sh`, com um
`HOME` temporário sobre uma CÓPIA do `~/.ph2d` do dono): a terceira fileira **cortada** pela borda
de baixo (a arrumação dele abre a timeline, que come um terço da altura ⇒ a cena passou a
**fechá-la e só depois pedir o *Frame All***, nesta ordem) · as duas cordas desenhadas como um **fio
fino** (um caminho aberto não se vê pelo preenchimento ⇒ traço grosso) · o controlo como uma **barra
recta com um vinco** (a deformação vectorial corre nos PONTOS DE CONTROLO, e um `RoundRect` tem
oito ⇒ ele passou a ser o **mesmo arco fechado pela corda**, que tem os mesmos pontos) · e o
preenchimento de um arco aberto a desenhar **a corda da corda**.

⛔⛔ **E o `when_two_handles_overlap_the_nearer_one_wins` reprovou PELA SEGUNDA VEZ, pela mesma
forma:** a fixtura dele perde o fenómeno sempre que esta lei muda. ⇒ ela passou a vir de uma PORTA
que nomeia a condição (`test_support::pele_na_lei_derivada`), em vez de uma `SkinBind` montada à
mão. *Uma fixtura montada à mão fica abaixo da lei que se está a medir.*

⛔ **Uma mutação SOBREVIVEU e mudou o desenho:** trocar o guarda da ponte (`if nivel() == 2`) por
`if false` deixava **tudo verde** — *um gate de texto afirma que o código EXISTE, nunca que ele
CORRE*. ⇒ o guarda saiu: a ponte passou a ter **uma chamada incondicional** (`prologo_do_nivel(n)`)
e a inércia do `=1` virou uma lei PURA, medida pelos dois lados. O que sobra por medir — a ponte
CORRER — fica **dívida nomeada**, com um gate que reprova se alguém repuser o guarda.

⚠️ **E o número da dobra é MEDIDO:** a `40°` por junta o alcance `1 → 4` move a corda **`18,4 %`** do
comprimento dela (a `15°` são `8,9 %`; a `60°`, `27,7 %` — e aí o bloco deixa de caber no ecrã).

Mutação **10 de 10** a sangrar; portão `15 110`, com o único ✗ a ser o
`an_abandoned_march_returns_nothing_and_returns_fast` — membro **confirmado** da família de flakes de
fan-out (3 de 3 verde sozinho a `load 18,36`, zero linhas do diff naquela crate).

⏳ **ABERTO, e é decisão do dono:** com o envelope inerte em toda arte preenchida, o `Strength`
serve **um** caso — um traço aberto preso a ossos (uma corda, um cabelo, um cabo). *Manter o
controlo escondido por osso é o que shipa; tirá-lo do produto é a outra saída, e é dele.*

### F22 — ⭐⭐⭐ **A ESCOLHA: por que lei CADA DESENHO se deforma** (ordem do dono, 2026-09-19)

Ele perguntou, depois de aprovar a cena da F21: *«como se usa os dois modos? como se escolhe se os
envelopes vão ou não influenciar?»* — e a resposta honesta era **não se escolhe**.

⛔⛔⛔ **O app decidia, e decidia pelo DESENHO:** uma forma com interior (ou uma imagem que resolve)
ia para o padrão-ouro e o alcance ficava inerte; um traço ABERTO caía na lei euclidiana. *Qual lei
deforma o personagem é uma decisão de RIG, e ela estava escondida dentro de uma decisão de DESENHO.*
⚠️ Não havia interruptor nenhum — varrido: o único candidato (`PH2D_SKIN_WEIGHTS=linear`) troca como
os pesos guardados são **interpolados** ao refinar, não qual lei os **produz**.

⭐⭐ **E a capacidade já existia inteira** — medido ANTES de escrever uma linha: com a tabela de pesos
apagada, uma forma FECHADA corre na lei do envelope e move **exactamente** o mesmo que o traço
aberto (`6,4368` contra `6,4368` sobre a mesma curva; um rectângulo move `6,9835`). *O que faltava
não era motor, era o botão.* ⇒ ordem dele: **«construa. por desenho»**.

⭐⭐⭐ **A decisão de desenho que é a wave inteira: a escolha diz se o quadro LÊ a tabela, nunca se
ele a CALCULA.** O padrão-ouro custa dezenas de milissegundos a resolver e fica guardado no bind; se
a escolha mandasse no cálculo, voltar atrás obrigaria a re-resolver e o artista veria a ferramenta
engasgar ao alternar. Assim ela é **viva** — troca-se no quadro seguinte, nos dois sentidos, e a
volta é **exacta ao bit** porque a tabela nunca é tocada.

⇒ `SkinBind::law: SkinLaw` (`Auto` | `Envelope`) e **uma porta** (`SkinBind::pesos_do_quadro`) com
**TRÊS leitores**: o recook de uma forma, o desenho de uma imagem presa, e a pergunta *«o envelope
manda neste osso?»* que acende a mancha e a alça. ⛔ Escrita em três sítios, a mancha apareceria onde
o alcance não governa nada — que é, à letra, o report de 2026-09-18.

⚠️ **`PROJECT_SCHEMA` +1 — conte o DELTA** (`144 → 145`). Campo novo numa struct já gravada ⇒ regra
dos degraus 109/110; ⚠️ **a tripla NÃO vê este degrau** (a SÉTIMA vez). ⛔ Os três registos de
componente **não se mexem**: não há tipo novo.

**Na tela:** a fileira **`Deform By`** (`Artwork` | `Bone Reach`), ao lado do *Release* — ⚠️ pintada
para as **duas mídias** (ao contrário do *Expand*) e **só quando há algo preso escolhido**: *sem pele
não há lei de pele, e um selector sem sujeito é a classe de controlo morto do §5.0*.

⛔⛔⛔ **E a FOTO apanhou DOIS defeitos que os gates não podiam ver.** (1) O painel **Bones nasce
fechado** e só se abre sozinho quando um OSSO é escolhido — e o passo do roteiro manda escolher um
**DESENHO**: *um passo que nomeia uma linha de painel afirma que ela está lá, e o dono aprova o smoke
com o passo impossível dentro* ⇒ a cena passou a abri-lo no prólogo, **antes** do *Frame All* (ele é
uma coluna lateral, e abri-lo depois mudaria a área que o enquadramento mediu).

⭐⭐⭐ **(2) E o segundo era a LENTE DO PAINEL mais ESTREITA que o sujeito — a MESMA forma do report de
2026-09-18, e o defeito era MUDO.** Clicar numa linha da **Hierarquia** escreve na selecção do
**GIZMO** (`hero.gizmo.replace_selection`), nunca na lista de caminhos do pen — e o `Skinned` do
painel lia `vector` só do pen e `imagem` só do gizmo. ⇒ uma **forma vectorial** escolhida na
Hierarquia lia `{vector: false, imagem: false}`, e o *Expand*, o *Release* **e** a fileira nova **nem
chegavam a ser pintados**. *O artista não vê um botão morto: vê a ausência de um botão.* ⇒ as duas
metades passam pela porta da família (`skin_law::escolhidas`), a mesma que o dreno do chip usa.

⚠️ **A recusa é PRÓPRIA e não a do vizinho:** `RecusaDoOsso::NadaAQuemMudarALei` (a quarta) — ⛔
reaproveitar a `NadaASoltar` diria *«nada a soltar»* a quem carregou noutro botão. ⚠️ E ela **não**
cobre *«já estava nessa lei»*: escrever a lei que já lá está não é um acontecimento, e queixar-se
disso é o ruído que o artista aprende a ignorar.

Cena **`PH2D_VEC_BONE_SMOKE=2`**, passos (3) a (5). Mutação **8 de 8** a sangrar.

⏳ **ABERTO:** a escolha não tem gesto de canvas (só o painel) · e com N desenhos escolhidos em leis
diferentes o chip acende por `any` — a escolha está **declarada** no doc da porta, e mostrar a
divergência é mais honesto do que mostrar a maioria, mas ela é decisão de produto.

### F23 — ⭐⭐⭐ **A POSE DE REPOUSO, e a cura do *Reset Transform*** (auditoria contra o oráculo, 2026-09-19)

O dono mandou auditar o sistema de ossos contra o Godot 4.7.2 (MIT, corrido sem interface) e fazer o
que faltasse, menos o movimento secundário. A auditoria devolveu **um defeito vivo** antes das
ausências, e este é ele.

⛔⛔⛔ **MEDIDO no caminho do produto** (`ph2d_skeleton_live::sonda_do_reset_na_hierarquia_tests`): a
tabela do menu de contexto da Hierarquia é **PLANA** — ela não sabe o que a linha é —, e sobre um
osso `*t = Transform::IDENTITY` movia a arte presa **26,484841 unidades num desenho de 60 (44 %)**,
com uma mensagem **VERDE** a dizer que correra bem. *Num osso a direcção mora na `rotation` e a
posição na `translation`: «repor a transformação» de um osso não é a identidade, é o REPOUSO dele* —
que não existia.

| lei | quanto a arte salta | fracção da forma |
|---|---|---|
| a identidade (o que o app tinha) | **26,484841** | **44 %** |
| o repouso (esta wave) | **0,000000** | nada |

⭐⭐ **`BoneRest` é um COMPONENTE e não um campo do `Bone`**, a forma do `BoneLimit`/`IkGoal` — e a
razão é que **a ausência é uma resposta**: um `Option` dentro do osso obrigaria todo `Bone::default()`
a escolher um valor, e o valor neutro de uma pose é exactamente a **identidade**, *o mesmo byte que é
o defeito*. Com um componente, um osso sem repouso **não o tem**, e o verbo recusa em voz alta.
⭐ De graça: blob-key própria ⇒ o `PROJECT_SCHEMA` **não se mexe** (precedente da `PhysicsJoint`/W3);
registo do esqueleto **6 → 7** e catálogo **6 → 7**.

⚠️ **Ele guarda os SEIS números em `f32` e não um `Transform`:** aquele viaja num invólucro
**versionado** (`TransformVersioned`), e aninhá-lo aqui poria os bytes **fora** dele — um campo novo
lá leria todo repouso gravado errado, em silêncio.

⇒ uma porta (`pose_de_repouso`) com `guardar` · `repor` · e o veredito
`repor_transformacao -> Reposicao { NaoEOsso | SemRepouso | Reposta { ossos } }`. **Três** respostas
e não duas: um osso **sem** repouso guardado não cai de volta na identidade — era por aí que o
defeito voltaria para todo rig anterior a esta wave.

⚠️ **O sujeito é o osso ESCOLHIDO e a descendência dele**, e é estritamente mais expressivo: escolher
a raiz repõe o boneco todo, escolher o antebraço repõe o antebraço e a mão. ⛔ Uma lei que subisse à
raiz sozinha tornaria *«repor só este braço»* inexprimível.

**Na tela:** *Rest Pose* e *Set Rest Pose*, **antes** dos números do osso (o gesto mais frequente do
rig não fica no fim de uma lista que rola), e o *Reset Transform* da Hierarquia passa a perguntar à
mesma porta.

⚠️ **O que a construção refutou:** a `esqueletos::ossos_desde` devolve um **conjunto** determinístico
e **não** uma ordem hierárquica (ela ordena por `to_bits`, que aqui sai ao contrário da criação) — a
1.ª redacção dos gates presumiu-a e o gate da sub-árvore reprovou a acusar a LEI de mexer no pai
quando quem estava trocado era a fixtura. E a régua textual do gate da shell leu o **doc-comment que
EXPLICA a cura** (a armadilha que a Fase B da física já pagou por escrito) ⇒ ela passa a deitar fora
a prosa antes de medir.

Mutação **9 de 9** a sangrar. Tecto de função curado por **CORTE** (`hierarchy_reset`, irmão do
`hierarchy_delete`), nunca por uma entrada nova no `FN_OVERAGE_OK`.

### F24 — ⭐⭐⭐ **UM OSSO QUE APONTA PARA UM ALVO (*Look At*) — e o motor já existia** (2026-09-19)

A mesma auditoria nomeou o `SkeletonModification2DLookAt` como ausência nossa. ⭐⭐⭐ **A §5.0 correu
antes da primeira linha e disse NÃO:** a lei do alcance tem, escrito no corpo dela, um braço para uma
corrente de **um** osso (*«um osso só: aponta, e o comprimento manda»*), e medido pelo caminho do
produto (`goal::add` + `solve`) ele aponta com erro **`0,000000°`** em cinco direcções. *O que
faltava era o NOME, o desvio, e esconder o que ali não faz nada.*

⚠️ **O controlo que impede a conclusão de ser fabricada:** num esqueleto de um osso só a corrente
resolvida é sempre `1`, logo tudo aponta — a metade negativa corre num **braço**, onde a corrente de
`2` resolve o par pela lei dos cossenos e o ombro vai parar a outro sítio.

⭐⭐ **E a lente é MEDIDA, não escolhida.** Com a corrente resolvida em UM:

| knob | move o osso |
|---|---|
| *IK Mix* | **sim** (é o único que continua a mandar) |
| *IK Softness* | **zero** — a lei de um osso põe a ponta a `comprimento` na direcção |
| *IK Bend* | **zero** — não há cotovelo, logo não há lado |

⇒ o painel **esconde** os dois inertes e **pinta** o desvio. ⛔ Pintá-los ali seria a classe de
controlo morto que o `CLAUDE.md` §5.0 nomeia.

⇒ `IkGoal::offset` (o `additional_rotation` da referência) + o verbo **`Look At`**, que é o `add` com
a corrente em `1` — ⛔ ele **delega** no irmão e não repete o nascimento (o alvo, a marca `IkTarget`,
a semente do `StableId` e a captura do lado vivem lá).

⛔⛔ **O desvio só é LIDO quando a âncora APONTA**, e a cerca é a wave inteira: somá-lo a uma corrente
que **alcança** quebraria o alcance que ela acabou de resolver — a mão deixaria de tocar aquilo que a
restrição existe para tocar. A porta é `goal::aponta` (a corrente **resolvida**, nunca o número
escrito no campo), com **três** leitores: o solver, o espelho do painel e o verbo.

⚠️ **E ele entra ANTES da mistura e do limite**, a ordem que os dois já declaram: somá-lo depois faria
o `Mix = 0` deixar de devolver a pose autorada — o artista desligaria a restrição e o osso ficaria
rodado, sem nada que o explicasse.

⚠️ **`PROJECT_SCHEMA` +1 — conte o DELTA** (`145 → 146`). Campo novo numa struct já gravada ⇒ regra
dos degraus 109/110; ⚠️ **a tripla NÃO vê este degrau** (a OITAVA vez). ⛔ Os três registos de
componente **não se mexem**: não há tipo novo.

**Na tela:** o botão **`Look At`** ao lado do *Add IK* (as duas portas de entrada, e a diferença é o
que a restrição FAZ), e o campo **`Aim Offset`** em graus, depois do `IK Chain`.

Mutação **13 de 13** a sangrar.

⏳ **ABERTO:** o apontar não tem gesto de canvas (só o painel) · e o desvio é um número, não uma alça
— arrastar o olhar no canvas seria outro gesto, e é decisão de produto.

### F25 — ⭐⭐⭐ **ESPELHAR UM RAMO — o lado esquerdo construído a partir do direito** (2026-09-19)

O terceiro item da auditoria. ⭐⭐ **A lei é uma CONJUGAÇÃO e calcula-se à mão, sem uma única
constante escolhida.** Seja `M` a reflexão do mundo na vertical `x = c` e `G` a que troca o sinal do
`y` **dentro do referencial de um osso**. O referencial que leva a cabeça a `M(cabeça)`, a ponta a
`M(ponta)` **e** repõe o sinal do determinante é `W' = M ∘ W ∘ G` — e daí sai tudo:

| o que | como espelha | porquê |
|---|---|---|
| filho (pose local) | `translação.y := −y` · `rotação := −r` · skews `:= −` | `T' = G ∘ T ∘ G` |
| raiz do ramo | derivada da cabeça e da ponta **reflectidas**, no referencial do pai | `T' = P⁻¹MP ∘ T ∘ G` |
| `length` | **igual** | o `G` fixa o eixo `+X`, e o comprimento vive nele |
| `curve` `y` | **negado** | o arco é um desvio em `y` |
| `curve` `x` | **igual** | ele mede-se **ao longo** do eixo |
| `BoneLimit` | **`{ −max, −min }`** | sob `r ↦ −r` a faixa inverte **e troca de ponta** |

⛔ **Só negar o limite deixaria `min > max`, e a lei trava a junta no CENTRO do que estiver escrito**
— o cotovelo espelhado ficaria preso a meio caminho, sem nada na tela que o explicasse.

⚠️ **O EIXO é DERIVADO** (§0.0): a vertical que passa pela origem do osso **RAIZ** do esqueleto —
num personagem, o quadril; é o mesmo `X = 0` da armadura que o Blender espelha. ⛔ Um campo com um
número seria uma terceira coisa a manter coerente com a pose.

⭐⭐ **A cópia passa pela CÓPIA PROFUNDA da casa**, e é isso que faz o ramo novo carregar o que esta
shell não conhece (o limite, a curvatura, o repouso, e o que vier): ela copia o que o **registo**
descreve. ⛔ Uma cópia campo a campo esqueceria o primeiro componente novo, em silêncio.

⛔⛔⛔ **E a cópia profunda NÃO REMAPEIA REFERÊNCIA NENHUMA — o doc dela di-lo por escrito.** O
`Bone::curve_tip` nomeia um **filho por identidade**, logo a cópia ficaria a apontar para o filho do
**ORIGINAL**: o ramo espelhado arquearia a seguir a um osso do outro lado do corpo, e a referência
**resolve**, logo nada acusaria. ⇒ ele é remapeado pelo mapa `StableId → StableId` que a cópia
devolve. ⚠️ **Um id de FORA do ramo fica intocado** — ali a referência do original continua a ser a
resposta certa.

⛔ **O que NÃO viaja, e é decisão declarada:** a **âncora de IK** e o **osso inteligente**. Os dois
nomeiam OUTROS objectos da cena por identidade, e copiá-los daria duas correntes a puxar o **mesmo
losango** — o braço espelhado seguiria a mão do original. *Mirrorar uma referência a um objecto é
uma segunda decisão que este verbo não pode tomar sozinho.* O artista carrega em *Add IK* / *Look At*
no ramo novo.

⚠️ **O NOME troca de lado por uma TABELA e não por um `replace` cego** — trocar todo `L` por `R`
renomearia `"Leg"` para `"Reg"`. O que se troca é um **marcador**: um sufixo (`.L`, `_Right`) ou uma
**palavra inteira** (`Left Arm`). ⚠️ E o resultado passa **sempre** pela porta da unicidade: a
referência durável entre objectos nesta casa é o NOME, e dois ossos com o mesmo seriam o mesmo
sujeito para a timeline. ⭐ Um nome sem lado (`"Bone 7"`) devolve `None` — *inventar-lhe um lado seria
escrever uma decisão do artista*.

⭐⭐⭐ **A prova mais dura é a INVOLUÇÃO:** espelhar duas vezes devolve a geometria original (barra
`1e-4`, derivada do `f32` da pose). Mais: cada osso da cópia vai de `M(cabeça)` a `M(ponta)` — medido
em **MUNDO** e não nos campos locais, *senão a régua mediria a implementação e ficaria verde sobre
uma cópia que aponta ao contrário* — e o **original não se mexe**.

**Na tela:** o botão **`Mirror Branch`**, terceiro do trio que age sobre *este osso e a descendência
dele* (os outros dois são o par do repouso). Zero schema, zero registo novo.

Mutação **12 de 12** a sangrar.

⏳ **ABERTO:** o espelho não tem gesto de canvas (só o painel) · e a arte presa não é espelhada com
os ossos — o ramo novo nasce sem pele, e prendê-la é o gesto que já existe (*Bind*).

### F31 — ⭐⭐⭐ **O PINCEL DE PESO ALCANÇA O MEIO DE UMA ARESTA** (report do dono, 2026-09-19: *«o que vc mandou fazer não funcionou»*)

⛔⛔⛔ **A F30 shipou uma LEI SEM GESTO, e o report tinha DUAS causas — a minha e a do produto.**

**(a) A minha.** O smoke que mandei mandava escolher **«Bone 14»** e pintar na **barra laranja**. A
cena tem **dois** esqueletos de três ossos — a barra vectorial (`Bone 1..3`) e o braço **PINTADO**
(`Bone 13..15`) — e o `Bone 14` é o do MEIO do segundo. *Seguir o passo à letra não podia funcionar.*
O dono disse-o melhor do que qualquer sonda: *«Bone 14 está ligado à imagem e não ao vetor. Bones 1,
2 e 3 estão ligados na barra laranja.»* ⇒ o roteiro passa a NOMEAR quem governa a barra, **derivado
do mundo** (os nomes são o índice da entidade: acrescentar uma peça à cena renumera tudo o que vem
depois), e a listar a cadeia **em ordem** — a [`esqueletos::ossos_desde`] ordena por `to_bits`, que
no bevy é a criação INVERTIDA, e a 1.ª frase saía *«Bone 3, Bone 2, Bone 1»*.

**(b) A do produto, e é a que importa.** O gate da F30 constrói a
[`CorreccaoDePeso`](../../crates/ph2d-skeleton-ecs/src/skin_bind.rs) **à mão**, com `centro` no meio
de uma aresta — e **nada no repo perguntava se o PINCEL consegue produzir esse centro**. Ele não
conseguia: a mancha era ancorada no **NÓ mais perto** e o gesto recusava (`ForaDaArte`) quando o
dedo estava mais longe do que o raio do pincel.

| o dedo, na barra do smoke | nó mais perto | raio de fábrica | veredito |
|---|---|---|---|
| no MEIO da barra | **`3,041`** | `0,40` | **`ForaDaArte`** |
| idem, com o raio a `400 px` | `3,041` | `4,00` | `Pintada`, **com o centro na QUINA** `(−8,0 · 2,0)` |

⇒ *é o terceiro elo do `CLAUDE.md` §5.0 outra vez — o censo prova que a PORTA faz efeito, a costura
prova que o clique chega ao BARRAMENTO, e nada juntava as duas pontas.* ⚠️ **Os gates que existiam
não podiam apanhá-lo:** eles pintam **em cima de um vértice**, que é o caso em que as duas leis
concordam. *Uma fixtura que aponta sempre para um nó não testa o que acontece entre eles.*

⭐⭐ **A lei que fica** ([`ancora_da_mancha`](../../crates/ph2d-skeleton-live/src/ancora_da_mancha.rs)):
a mancha pousa no **ponto do CONTORNO** sob o dedo, e o repouso dele sai do **MESMO parâmetro** da
curva. ⚠️ O achatamento é o mesmo nos dois lados, e é isso que torna a tradução honesta — *dois
achatamentos diferentes dariam um repouso plausível e errado*. ⭐ E **estar DENTRO da forma conta**:
a barra tem meia unidade de meia-altura contra um pincel de `0,40`, logo uma régua que só olhasse o
contorno recusaria exactamente a linha por onde o artista arrasta.

⚠️ **A diferença entre as mídias é DECLARADA:** numa IMAGEM continua a ser o vértice da malha mais
perto — ali a deformação entre dois vértices é a interpolação linear deles, não há «entre» a que
pousar, e os vértices são densos. *Uma lei só para as duas teria de escolher entre recusar o meio de
uma barra e mover um mapa que o dono já aprovou em smoke.*

**Medido, de ponta a ponta pela porta do produto** (a barra da cena, dobrada `0,8` rad na ponta, seis
pinceladas com o raio e a magnitude de FÁBRICA ao longo do meio dela):

| | |
|---|---|
| a arte move-se, pela lei da curva | **`0,141983`** |
| as MESMAS manchas, pela lei dos pontos de controlo | **`0,000000`** |

⭐ O controlo é o A/B das duas leis sobre as mesmas manchas — *é a metade que prova que quem move a
arte é a F30 e não o recook a mexer-se sozinho*.

⛔⛔ **DUAS premissas morreram e foram reescritas com a morte à vista no diff:** *«a mancha é
ancorada no NÓ»* (hoje: **sobre a ARTE** — a cerca que o report original pedia fica, e mais forte,
porque uma alça de quina vive FORA da curva) e *«o meio da aresta não é um ponto que o desenho
tem»*.

⚠️ E uma mutação SOBREVIVEU: prender a fracção da projecção a `[0,1]`. Sem isso o cursor projecta-se
**para lá do fim** da corda e essa distância ganha da verdadeira — a mancha pousaria **fora da
peça**. *Um `clamp` não é defensivo: é a diferença entre projectar numa CORDA e projectar na RECTA
que a contém.*

Mutação **8 de 8** a sangrar. Zero schema, zero registo novo.

⏳ **ABERTO e NOMEADO: o INDICADOR ainda mostra só os NÓS.** A barra tem oito pontos coloridos e a
mancha entre eles não move nenhum deles (ela está a `3` unidades de qualquer um, com raio `0,40`)
⇒ *o artista vê a arte dobrar e as cores paradas*. A cura é a mesma lei da F30 — o peso ao longo da
curva é `lerp(ra, rb, t)` mais as manchas —, e ela vive **dentro** da
[`curva::SegmentoDaPele::ponto`](../../crates/ph2d-vec-skin/src/curva.rs): expô-la como porta com
dois consumidores (o refit e o olho) é a wave. ⛔ **Não a reescreva no indicador** — uma segunda
resposta à mesma pergunta divergiria, e o sintoma seria o olho a pintar um peso que a arte não tem.

### F30 — ⭐⭐⭐ **A ARTE SEGUE O PESO ENTRE OS NÓS** (ordem do dono, 2026-09-19, *«construa e veremos se fica bom»*)

**MEDIDA antes de escrita uma linha de produto**
([`sonda_da_pele_como_warp`](../../crates/ph2d-vec-skin/src/sonda_da_pele_como_warp_tests.rs)), e a
medição **mudou o desenho**.

⛔⛔⛔ **A nota que descrevia esta saída estava errada em DUAS coisas.** Ela dizia *«deformar a forma
por uma MALHA … o preço NÃO é “a `ph2d-poly2d` já existe”: ela parte de uma **grelha de ALFA**, logo
a forma teria de ser RASTERIZADA»*.

1. **A rasterização não é precisa:** a [`ph2d_poly2d::triangulate`] recebe um **anel de pontos** e a
   [`ph2d_poly2d::grid_mesh_of`] também — a grelha de alfa é **uma** das entradas
   ([`ph2d_poly2d::mesh_of`]), não a única. *É a terceira nota minha que a medição derruba nesta
   jornada.*
2. ⭐⭐⭐ **E a MALHA não é precisa para a GEOMETRIA.** A [`ph2d_vec_envelope`] já deforma geometria
   **Bézier** por um mapa **não-afim**, e o cabeçalho dela descreve, por escrito, o defeito que a
   pele tem hoje: *«só transformações afins comutam com a avaliação de Bézier … a curva resultante
   não é a imagem da curva original … ela acerta em `t=0` e `t=1` exactamente, e no interior
   nunca»*. ⇒ **é a razão de a arte não responder a peso pintado entre os nós**, e é a mesma raiz do
   salto que o dono recusou na F28.

**O que a sonda mediu** (dois ossos, rectângulo de `40 × 10`, uma mancha no meio da aresta de baixo —
o sítio exacto da pergunta do dono):

| pergunta | resposta |
|---|---|
| peso pintado ENTRE dois nós move a arte? | **hoje `0,000000`** · pela rota do warp **`0,242375`** |
| o `fit_to_bezpath` converge? | **sim**, mesmo com jacobiana por diferença finita (⇒ não morre aqui) |
| custo | **`0,163 ms`** em `--release` (`0,881` em debug), `accuracy 0,05` |
| nós da saída | `4` na fonte ⇒ **`16`** no desenho |

⛔⛔ **O QUE FALTA, e é o que torna este item pegável:**

1. ⚠️⚠️ **A sonda usou a lei DERIVADA (`weights_corrected(p, None, …)`), e o produto usa a do
   PADRÃO-OURO** — uma linha guardada **por ponto de controlo**, que **não tem forma contínua**. ⇒ a
   malha volta, mas **só como portadora do campo de pesos**, nunca da geometria: guardar a malha do
   domínio do bind e amostrá-la, ou interpolar as linhas dos dois nós ao longo do `t` (a lei que a
   F28 já escreve para o ponto novo). *É esta a decisão que abre a wave.*
2. **A jacobiana tem de ser FECHADA.** O contrato do [`ph2d_vec_envelope::Warp`] exige a derivada
   real, e o doc dele mede que uma inconsistente faz o fit **não convergir** — ela falha **alto**.
   `∂W/∂p = Σ_j [ A_j(p) ⊗ ∇w_j(p) + w_j(p) · L_j ]` ⇒ é preciso `∇w_j`, que um campo baricêntrico
   dá **descontínuo** e o campo de Hermite da [`ph2d_poly2d::hermite_attrs`] dá **suave**.
3. **O `recook` corre por quadro.** `0,163 ms` por forma é `~1 %` de um quadro; dez formas presas são
   `10 %`. ⇒ ou memo por pose, ou a `accuracy` deixa de ser `0,05` — e **nenhum dos dois números foi
   escolhido por ninguém**.
4. ⏳ **Decisão de PRODUTO, e é do dono:** o desenho cozido passa a ter **mais nós** que a fonte
   (`4 → 16`). Ninguém os edita (a fonte é que se edita), mas o ***Expand*** assa a geometria de agora
   no desenho — ali o artista fica com a forma refitada. *É o único sítio onde o número sai do
   quadro e entra no documento.*


---

#### ✅ CONSTRUÍDA no mesmo dia, e ela DISSOLVEU a F28

A lei vive em [`ph2d_vec_skin::curva`](../../crates/ph2d-vec-skin/src/curva.rs) e é o caminho de
**OMISSÃO** (`PH2D_SKIN_CURVE=0` bissecta). Num segmento de `a` para `b`, o peso do ponto `C(t)` é a
**mistura** das linhas dos dois nós — `lerp(ra, rb, t)`, com as manchas somadas **no ponto** —, a
curva é amostrada e **refitada** (`kurbo::fit_to_bezpath`), e a remontagem do contorno é a **porta**
que a [`ph2d_vec_envelope`] já tinha (⛔ duplicá-la poria a convenção `(⅓, ⅔)` da elevação de recta
em dois sítios).

| o quê | medido |
|---|---|
| mancha ENTRE dois nós move a arte | **`0,000000` → `0,836850`** |
| os NÓS mexem-se? | **`0,000000000`** — em `t = 0` e `t = 1` a mistura é a linha do próprio nó |
| em REPOUSO | **`0`** ao bit |
| custo | `0,877 ms` em debug · `0,163 ms` em `--release` |
| nós do desenho | `4` na fonte ⇒ `6` desenhados |

⛔⛔⛔ **E o REFIT só corre onde o mapa NÃO é afim — isto não é optimização, é a cura de um defeito
medido.** A 1.ª redacção refitava **sempre**, e o gate `binding_a_shape_moves_nothing` acusou
`13,333…` = **`40/3`** em REPOUSO: a elevação `(⅓, ⅔)` de uma recta desenha a **mesma** curva com
outros pontos de controlo. *O desenho estava certo e a representação é que mudava*, e **oito** gates
da casa mediam a representação. ⇒ a lei de hoje corre **sempre e primeiro** (ela preserva o `kind` e
o `corner_radius`, que um refit não pode preservar), e só os contornos que se afastam mais do que a
tolerância são refitados.

⭐⭐⭐ **E a F30 DISSOLVEU a compensação da F28.** Com o desenho a ser a imagem verdadeira da fonte,
partir a fonte **não o move** (`0,000002 %`) — e compensar **estraga** (`11,11 %` da peça). ⇒ a
compensação passa a ser da lei dos pontos de controlo, e a decisão sai da mesma porta que o `recook`
lê. *Uma cura fica errada no dia em que o defeito que ela curava deixa de existir.*

⚠️⚠️ **E a lei viaja como PARÂMETRO, nunca num estado global.** A 1.ª redacção pôs um átomo com uma
porta `forcar_lei` para os gates medirem o outro lado, e o doc dela dizia *«o nextest corre um
processo por teste»* — verdade para o `nextest`, **falsa** para o `cargo test`, que corre os testes
em THREADS do mesmo processo. A suíte **reprovava em conjunto e passava sozinha**, que é a assinatura
mais cara que há. ⇒ `recook_com` / `insere_ponto_com`, e quem lê o ambiente é a porta de cima.

⚠️ **Cinco gates da casa tiveram a premissa mudada, e a morte de cada uma está no diff:** o
instantâneo do hit-test tinha *«um ponto por ponto DESENHADO»* e passa a ter **um por ponto da
FONTE** (⭐ e ter menos é a resposta certa: o peso vive nos nós, e um ponto do indicador onde não há
peso para corrigir seria um controlo morto) · a expectativa da tabela guardada constrói-se com a
**mesma** lei que o quadro corre · e a barra da excursão do envelope desceu de `1,0` para `0,5`, com
o número medido ao lado.

⛔ **E uma MUTAÇÃO SOBREVIVEU duas vezes, nas duas crates:** apagar a lei de hoje do início da porta
não partia nada, porque **toda** fixtura dobrava um osso e o refit escrevia por cima — *o caminho
onde a lei de hoje é a única a trabalhar não tinha fixtura nenhuma*. ⇒ o gate novo é **UM** osso,
onde a deformação é afim por teoria: ali a arte tem de se mover **e** o desenho tem de ficar
byte-idêntico ao de sempre.

Mutação **7 de 7** a sangrar; `nextest-impacted` **15 686** verdes.

### F29 — ⏳ **ABERTO: os DOIS modos de atribuir peso** (ordem do dono, 2026-09-19, *«coloque na fila»*)

*«Precisamos de 2 modos de atribuir peso aos pontos.»*

1. **Valor ABSOLUTO** — o valor de *Brush Strength* é posto **imediatamente** no osso em mãos, e o
   que sobra (`1 − v`) reparte-se pelos **outros** ossos que já têm peso naquele ponto, **mantendo a
   proporção entre eles**. ⇒ neste modo os botões *Add* e *Subtract* ficam **inactivos**.
2. **Valor CUMULATIVO** — a cada pincelada o nó ganha ou perde o valor de *Brush Strength*, conforme
   o botão marcado. **É o que existe hoje.**

⚠️⚠️ **A DIFERENÇA NÃO É DE UI — É DO MODELO DE DADOS, e é por aí que se começa a medir.** A
correcção é hoje uma [`ph2d_skeleton::Correccao`] — uma **MANCHA no espaço** que **SOMA**
(`w += delta · bump · quota`, e a normalização vem depois). Duas manchas sobrepostas **acumulam-se
por construção**, que é exactamente o modo 2.

⛔ **O modo 1 não é exprimível como uma mancha de soma**, e a pergunta que o decide é uma medição:
*duas manchas ABSOLUTAS sobrepostas — o que recebe um ponto que está debaixo das duas?* Se a resposta
é *«a última que o artista pintou»*, então uma correcção absoluta **não é um campo somável** e o
`Correccao` precisa de espécie (`Soma` / `Alvo`), com a ordem da lista a passar a ter significado —
⚠️ e ela **viaja em bytes opacos dentro do `SkinBind`**, logo é degrau de `PROJECT_SCHEMA`.

⚠️ **E a repartição do modo 1 não é a normalização que já existe.** Hoje o `corrige` soma e depois
divide pela soma — o que *diminui* proporcionalmente **todos**, incluindo o osso em mãos. O modo 1
pede outra coisa: **prender** `w[alvo] = v` e escalar **só os outros** por `(1 − v) / Σoutros`.
⛔ E ele tem um caso degenerado nomeado: *e quando os outros somam ZERO?* (um ponto que só o osso em
mãos governa). Ali não há por onde repartir, e a resposta tem de ser escrita antes de o código a
escolher sozinho.

⭐ **O que já está pronto:** o `WeightDirection` (os botões *Add*/*Subtract*) tem porta própria e
**três** consumidores — esconder/inactivar os dois no modo absoluto é a lente do painel, que já
existe para o `Pose` e o `Density`. E o censo dos knobs mede se um controlo chega ao barro, logo um
botão inactivo que continue a escrever seria apanhado.

### F28 — ⭐⭐⭐ **UM PONTO NOVO NUMA FORMA PRESA SOBREVIVE, E JÁ NASCE COM PESO** (ordem do dono, 2026-09-19)

A 1.ª das duas saídas que a F26 deixou ao dono para *«pintar peso entre os vértices de uma forma
vectorial»*, e ele escolheu-as **em ordem**: *«primeiro 1 e depois o 2»*.

⛔⛔⛔ **E a nota que descrevia esta saída estava ERRADA no ponto que decidia o preço.** Ela dizia
*«acrescentar vértices com a caneta … o gesto já existe nesta casa. **Custo: zero de
arquitectura**»*. **Medido pelo caminho do produto antes de escrever uma linha**
([`sonda_do_ponto_novo_tests`](../../crates/ph2d-app-skeleton/src/sonda_do_ponto_novo_tests.rs)): a
caneta escreve no documento **VIVO**, e o `recook` reconstrói esse documento a partir da geometria
**autorada** que o bind guardou — uma vez por quadro. *O ponto aparece sob o dedo e desaparece
sozinho*, sem erro, sem aviso e sem recusa. ⇒ *uma PRESENÇA afirmada sem olhar o caminho do produto
é um palpite com cara de medição* — a mesma família que este repo já pagou no sentido oposto.

⛔ **E o contorno óbvio — «acrescente o ponto e carregue em *Bind* outra vez» — custa o trabalho do
artista:** o `SkinBind::new` nasce com `correcoes: vazio` e `law: Auto`, logo um re-bind deita fora
**todas as correcções pintadas à mão** (a feature da F26) e a escolha de lei daquele desenho.

⭐⭐ **A lei: a FONTE é que ganha o ponto, e a tabela cresce com ele**
([`ph2d_skeleton_live::ponto_novo`](../../crates/ph2d-skeleton-live/src/ponto_novo.rs)). O ponto
entra na geometria autorada, no mesmo segmento e no mesmo parâmetro em que a mão o pediu, pelo mesmo
`split_segment` de sempre; o quadro seguinte re-deriva o desenho dali. ⛔ *Escrever também no
documento vivo seria a segunda resposta à mesma pergunta.*

⚠️ **A linha de pesos do nó novo é a MISTURA das dos dois vizinhos, no mesmo `t`** — e não a lei
automática. A tabela guardada vem do padrão-ouro (uma resolução **global** sobre a malha do domínio):
pedir a lei derivada só para este nó poria **um ponto a obedecer a outra lei** no meio de uma forma,
e re-resolver o global mudaria o peso de **todos** os outros nós, apagando a linha de base que o
artista corrigiu. ⭐ A mistura é uma combinação **convexa** de duas partições da unidade, logo não há
normalização a fazer — e há gate a afirmá-lo.

⭐⭐⭐ **E o desenho move-se um pouco ao acrescentar o ponto — o que parecia um defeito é REFINAMENTO,
e a escada prova-o.** O desenho cozido é a Bézier dos pontos de controlo **deformados**, e não a
imagem verdadeira da curva de repouso pela pele: *ele já é uma aproximação*. Cortando o mesmo
segmento `1 → 2 → 4 → 8` vezes, o desvio entre degraus cai **`18,89 % → 3,13 % → 1,00 %`** da peça —
uma sequência que converge geometricamente não corrompe nada.

| fixtura | salto ao acrescentar um ponto |
|---|---|
| esqueleto em **REPOUSO** | **`0` ao bit** |
| aresta **CRUA** (um segmento a atravessar os dois ossos) | **`18,89 %`** da peça |
| aresta **DESENHADA** em 8, pior segmento (o da junta) | **`0,91 %`** da peça |

⚠️ **Os `18,89 %` não são o custo de acrescentar um ponto — são o tamanho do erro que aquele único
segmento já tinha, e o corte mostra-o.** A barra do gate é uma **catraca MEDIDA** (`1 %`) com censo
de obsolescência nos dois sentidos, ⛔ nunca um *«acima de X o artista vê»*, que seria um palpite.

⚠️⚠️ **DUAS armadilhas de FIXTURA, as duas apanhadas pelos controlos e nenhuma pelo olho:**
1. A 1.ª régua da forma desenhada leu **`0,0000 %`** — os dois extremos do segmento `0` estão ambos
   dentro do primeiro osso, logo a lei preserva a forma **ao bit por construção** e a barra passava
   por **vácuo**. Quem a apanhou foi o controlo `gap`. O sítio onde o peso varia é a **junta**, e o
   gate passa a medir o **pior** segmento.
2. O construtor da fixtura «desenhada» subdividia sempre o **primeiro** pedaço, deixando o **último**
   a atravessar a junta inteira — ela chamava-se desenhada e media o mesmo segmento grosseiro do
   outro palco. *Uma fixtura com o nome errado responde à pergunta do vizinho.*

⭐ **TRÊS peças de substrato que a wave obrigou, e as três são melhores do que o que substituem:**
o formato guardado ganhou **porta** (`skinned_mesh::le`/`grava` — ele era descodificado **à mão em
sete sítios**, cada um com a sua cerca); a caneta passa a **reportar onde inseriu**
(`PenTool::take_insercao`, porque o `t` é do dedo e reconstruí-lo do outro lado faria o ponto nascer
noutro sítio da mesma curva); e o `SkinnedPath` ganhou `linha_do_no`.

⛔⛔ **E uma cerca SAIU por uma mutação que sobreviveu:** o `if !fonte.valida()` depois do splice é
inalcançável por construção, e o `recook` já o faz a jusante, onde ele defende do caso real (uma
fonte gravada por outra versão). *Uma linha que a mutação não consegue matar não é lei, é comentário
com sintaxe de código.*

⛔⛔⛔ **E o FIO teve DUAS mutações sobreviventes, uma em cada ponta:** apagar o registo na caneta
deixava `10` testes da shell verdes (o gate de costura de lá lê o TEXTO do despacho — ele afirma que
a shell *drena*, nunca que a caneta *grava*), e cravar `t = 0,5` no registo passava o gate novo,
porque o dedo dele estava **no meio do segmento**, onde o `t` verdadeiro *é* `0,5`. *As duas pontas
de um fio precisam cada uma do seu gate, e um corpus no ponto neutro de um valor não testa esse
valor.*

**Na tela:** nada de novo — é a CANETA de sempre, e o roteiro da cena `PH2D_VEC_BONE_SMOKE=1`
ensina-a **onde a limitação aparece**: no aviso de que a barra laranja só tem oito nós (gate a exigir
que a cura fique a menos de 400 bytes do aviso que a motiva — *duas linhas separadas por vinte lêem-se
como dois assuntos*).

⛔⛔⛔ **E o PORTÃO DE FECHO apanhou um gate MEU vermelho, com uma causa que vale para toda régua de
curva desta casa: o `t` é o PARÂMETRO DA CURVA, não a fracção ao longo da CORDA.** A minha régua
esperava `0,3` (onde o dedo estava) e leu **`0,375`** — numa quina os dois pontos de controlo
interiores colapsam nas âncoras, logo a cúbica é `P0,P0,P1,P1` e a posição avança com `3t² − 2t³`, o
*smoothstep*; resolvendo, dá exactamente `0,375`. *A régua estava errada e o código certo.* ⇒ ela
passa a medir o **PRODUTO** — onde o ponto NASCEU —, que é a pergunta do artista, não depende da
parameterização, e mata na mesma o `t` cravado.

⛔⛔⛔ **E isso expôs um furo no ARNÊS DE MUTAÇÃO que invalidava as provas daquele gate: ele não
perguntava se o teste estava VERDE antes de mutar.** Com o gate já vermelho, **todas** as mutações
sobre ele liam *«SANGRA»* — *um teste já vermelho certifica qualquer mutação*. O arnês ganhou o
controlo (`exit 5`, com a razão), e as duas provas daquela ponta foram **refeitas** com ele.

⚠️ **Promoção pedida à lista de flakes de carga do `CLAUDE.md` §5.0:**
`a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget`
([`ph2d-app-flip`](../../crates/ph2d-app-flip/)) — reprovou no meio de um fan-out de **15 506** testes
e passa **3 de 3 sozinho a `load 33–38`**, com **zero** linhas do diff desta wave naquela crate. Na
mesma corrida reprovou o `no_expression_allocates_no_link_frame`, que **já é membro nomeado**.

Portão: `fmt` limpo · clippy `-D warnings` a zero · `nextest-impacted` **15 506 verdes** (eram
`15 297`) · mutação **12 de 12** a sangrar (três sobreviveram primeiro — duas viraram gate e uma
matou uma cerca —, e duas foram refeitas depois do arnês ganhar o controlo de verde).
Zero contador partilhado, zero contrato, zero ADR. Tecto de LOC curado por **CORTE** (os gates do
roteiro saíram para `smoke_bone_roteiro_tests.rs`), nunca por isenção.

⏳ **ABERTO:** a 2.ª saída que o dono pediu a seguir — **deformar a forma por uma MALHA**.

#### ⛔⛔⛔ F28-b — O SMOKE REPROVOU-A, com DOIS reports, e o primeiro derrubou uma conclusão MINHA

*«não ficou bom. O ponto criado na malha já conectada aos ossos deforma a malha»* · *«não tem
indicação visual que você está em cima da linha para criar um ponto»* (2026-09-19).

**(1) O SALTO DA FORMA ERA UM DEFEITO, e eu tinha-lhe chamado refinamento.** A F28 mediu o salto,
mostrou que a escada da subdivisão converge, concluiu *«é a aproximação a ser refinada»* e **disse-o
ao dono como se fosse normal**. ⛔ *A régua dele é a que manda: o desenho é o que o artista vê.* A
conclusão não era falsa — era uma explicação a fazer de veredito.

⭐⭐⭐ **A cura é uma INVERSÃO, e ela sai da própria estrutura da lei:** o `recook` lê **uma** linha de
pesos — a da âncora — e aplica-a às três metades do vértice; com essa linha fixa, `x ↦ blend(x, w)` é
um **AFIM**. ⇒ o corte faz-se no **DESENHO** (de Casteljau sobre os pontos de controlo já deformados)
e o ponto de repouso que desenha em `X` é `L⁻¹(X − c)`, com `L` e `c` lidos por **três** avaliações da
própria porta — sem uma segunda cópia da lei.

| fixtura | ANTES | AGORA |
|---|---|---|
| aresta CRUA (um segmento sobre os dois ossos) | `18,89 %` da peça | **`0,000000 %`** |
| aresta DESENHADA em 8, pior segmento (o da junta) | `0,91 %` | **`0,000023 %`** |
| escada `1 → 2 → 4 → 8` | `18,89 → 3,13 → 1,00 %` | **`~1e-14`** nos três |

⭐⭐ **Os dois vizinhos não entram na conta, e é por isso que a lei é barata:** o `out` do anterior e o
`in` do seguinte já saem certos **ao bit** — eles são combinações afins de pontos que usam o MESMO
peso, logo o corte comuta com a deformação ali. *Só o vértice do meio mistura os pesos das duas
pontas.*

⭐ **Em REPOUSO a compensação é a IDENTIDADE ao bit** (toda pose é a identidade ⇒ `L = I`, `c = 0`), e
há gate a afirmá-lo. *Ela só existe onde há deformação para preservar.*

⚠️ **O preço, declarado:** o que absorve a diferença é a geometria de REPOUSO. Acrescentar um ponto
com o rig POSADO deixa o repouso deslocado do corte ingénuo pela mesma grandeza que o desenho
deixaria de saltar. ⛔ **As duas coisas não podem ser preservadas ao mesmo tempo** (só o seriam se as
duas pontas do segmento tivessem o mesmo peso), e a escolha é a do dono: *o desenho é o que ele vê*.

⚠️⚠️ **O número de PASSAGENS ficou observável por uma MUTAÇÃO SOBREVIVENTE, e a fixtura mordeu DUAS
vezes antes de conter o fenómeno.** A linha de pesos depende da POSIÇÃO (as manchas do pincel, o
`quota` de um osso que dobra), e mover a âncora muda-a ⇒ a compensação repete. Mas no corpus de então
**nada** dependia da posição, e `1` passava. A fixtura nova é uma forma com mancha pintada, e ela
falhou duas vezes: a 1.ª punha o ponto no **cume** da bolha, onde o `clamp(0,1)` **satura** e o peso
volta a ser constante (*uma mancha saturada não é uma mancha, é um planalto*); a 2.ª escrevia o braço
de «uma passagem» à mão e **não fazia crescer a tabela de pesos** (*um controlo que não percorre a
MESMA porta compara dois programas*). Com ela: `0,0772 → 0,0014 → 2,6e-5 → 4,7e-7 → 0` — cada
passagem divide por **~55**, e **`6`** é onde a escada acaba. ⛔ E uma saída antecipada por
convergência **SAIU** por outra mutação sobrevivente: ela não muda um bit, só poupa passagens de custo
nulo.

**(2) A PRÉVIA DE INSERÇÃO — o gesto existia e era INVISÍVEL.** O artista tinha de adivinhar a que
distância da curva o clique deixa de acrescentar um ponto e passa a **começar uma forma nova** —
*duas coisas muito diferentes, sem nada na tela a separá-las*. ⇒ um anel VERDE e OCO com uma CRUZ,
no ponto onde o clique poria o nó.

⭐⭐ **A posição e o RAIO vêm da porta do clique** (`PenTool::previa_de_insercao` chama o `insert_hit`
e decide o raio com a mesma linha do press) — *um realce calculado por uma segunda conta acende num
sítio e insere noutro*, o defeito que os realces do Trim e do Balde já nomeiam por escrito. ⚠️ E ele
é **derivado por quadro** e **LIMPO fora do modo Pen**: um realce deixado a arder depois de trocar de
ferramenta promete um ponto que nenhum clique põe.

⚠️ **As três metades têm gates separados porque são três defeitos:** ninguém calcula (nunca acende) ·
ninguém pinta (o report volta inteiro com o trabalho feito por baixo) · e acende onde o clique **não**
insere (pior do que não acender). ⛔ **O que NÃO se pôde fotografar:** o realce precisa do cursor
sobre a linha, e o XTest da sessão virtual é ignorado — *a foto prova o que abre, não o que passa o
rato*.

Portão: `fmt` limpo · clippy `-D warnings` a zero · `nextest-impacted` **15 630 verdes** · mutação **9 de 9** a sangrar (duas sobreviveram primeiro: uma virou fixtura e a outra matou a
linha). Zero contador partilhado, zero contrato, zero ADR.

### F27 — ⭐⭐⭐ **O CENSO DOS VERBOS DO OSSO: o clique chega a um EFEITO?** (2026-09-19)

**O item que a F16 deixou aberto por escrito, fechado — e o veredito é bom: ZERO verbos mortos.** Os
**catorze** botões da secção chegam a um efeito: **treze** mexem no mundo e **um** declara que o
consumidor dele é o clique seguinte.

⛔⛔ **É a segunda metade da pergunta que o `§5.0` nomeia sobre o repo inteiro** (*«nenhum instrumento
pergunta se o VALOR chega a um consumidor»*). A F16 fechou-a para os **números** em 18/09 e escreveu
na própria célula que os verbos ficavam com censo de *chegam ao barramento* e nenhum de *chegam a um
efeito* — que é a família de metade dos reports do dono nesta linha: *o botão pinta, acende sob o
rato, o clique atravessa o painel, e o mundo não se mexe.*

⭐⭐ **A régua é o PRODUTO e a fotografia é a do UNDO.** Cada verbo corre pela **porta que a shell
chama**, sobre um palco montado para ele, e o que se mede é a captura
[`world_to_snapshot`](../../crates/ph2d-ecs/src/scene/save.rs) — a mesma que a fila do undo tira.
⚠️ **A cena vectorial entra ao lado dela**, e não por gosto: o *Expand* escreve a geometria deformada
no **documento do vector**, que não é uma entidade — sem essa metade, uma mutação que fizesse o
*Expand* chamar o *Release* ficava invisível, porque os dois tiram o `SkinBind`.

⛔⛔⛔ **E UMA MUTAÇÃO SOBREVIVEU, e é o achado da wave: NADA no repo liga as duas pontas.** Apagado o
corpo do braço do *Add Smart Bone* na fase do quadro, **`23` testes da shell ficaram verdes**. O censo
da família prova que a **PORTA** faz efeito; a costura do painel prova que o clique chega ao
**BARRAMENTO**; e o terceiro elo — *o braço que recebe chama alguma porta?* — não tinha instrumento
nenhum. É a quarta vez que esta rota morre nesta linha. ⇒ `VerboDoOsso::rastos_na_shell` mais o censo
[`todo_verbo_do_osso_deixa_rasto_na_shell`](../../shells/desktop/tests/it/os_verbos_do_osso_chegam_do_botao_ate_a_lei.rs),
que vive **ao lado dos três gates que já faziam isto à mão** para o *Look At*, o desvio e o espelho —
*uma segunda superfície para a mesma pergunta seria a lista que envelhece*.

⚠️⚠️ **Ele mede TEXTO e não uma chamada, e a limitação é DECLARADA:** as fases são métodos de `App`,
que segura uma surface de janela real, logo nenhum teste as corre. *Ele apanha o braço que deixou de
chamar a porta; o braço que a chama com o argumento errado é apanhado do outro lado* — pelo censo da
família, que corre as duas portas e exige que elas **difiram**.

⭐⭐⭐ **E a `smart::add` NASCEU por causa do censo.** Das catorze rotas, o *Add Smart Bone* era a única
cujo efeito estava escrito **dentro da fase do quadro** (um `insert` de uma linha), logo a única que o
censo não conseguia correr sem re-escrever a lei — *e uma régua que re-escreve a lei mede outro
programa*. A shell decide a ORDEM; **o que** um verbo faz é conhecimento de quem possui o componente,
que é a lei que o [`knobs`](../../crates/ph2d-app-skeleton/src/knobs.rs) já escreve.

⚠️ **A tradução `id → verbo` resolve pela POSIÇÃO na tabela** (a mesma lei do lado da dobra e do
sentido do pincel de peso), e é isso que impede uma segunda lista de catorze braços. ⛔ **O preço está
pago com gate:** trocar dois itens da `VECTOR_BONE_VERBS` faria o botão que diz *Bind* mandar
*Release* — *um botão que faz o contrário do que diz é pior do que um morto* —, e por isso o censo
pina **cada id ao verbo pelo NOME**, um a um.

⛔ **A ÚNICA isenção é NOMEADA e tem gate próprio:** o *Pick Object* arma um **MODO** e o consumidor
dele é o clique seguinte. *Uma célula sem proveniência e uma com proveniência têm o mesmo aspecto numa
tabela* — é a mesma forma do `Strength` no censo dos números, e o número de isentos está gateado em
`1`.

⚠️⚠️ **DUAS armadilhas de FIXTURA, as duas apanhadas pela primeira corrida:**
1. **Prender e assar no mesmo instante devolve a FONTE** — o `bind` guarda a pose de AGORA como
   repouso, logo *o Expand não tem nada para assar num corpo que não saiu do repouso*, e os dois
   verbos liam-se idênticos. A ordem do palco é **prender · dobrar · re-cozinhar**.
2. **O piso do censo textual era a SOMA e tinha de ser POR VERBO.** Quatro verbos declaram dois
   rastos (a porta partilhada mais o discriminador), logo esvaziar um verbo inteiro deixava
   `16 >= 14` e o censo verde. *Uma lista vazia lê-se exactamente como aprovada* — a catraca sem
   censo de obsolescência, um nível abaixo.

Mutação **16 de 16** a sangrar (duas sobreviveram primeiro e as duas viraram gate). Zero contador
partilhado, zero contrato, zero ADR, zero linha de produto mudada — a única troca no caminho do
artista é o `insert` do *Add Smart Bone* passar a ir pela porta.

### F26 — ⭐⭐⭐ **CORRIGIR UM PESO À MÃO — o pincel, a mancha e o olho** (2026-09-19)

O quarto e último item da auditoria: *«quando a conta automática erra num sítio, não há como
acertar aquele ponto»*. Hoje há — um **terceiro verbo** na fileira do osso (**`Weight`**), e
arrastar sobre a arte presa empurra a influência do osso em foco para cima (ou, com o valor
NEGATIVO, para baixo).

#### A lei: uma MANCHA no espaço, nunca uma tabela por vértice

⛔⛔ **A tabela por ordem de varredura é o *vector paralelo* que o `VecVertex::corner_radius` proíbe
por escrito**, e este módulo já a recusou uma vez (os pesos *derivam-se*, não se guardam): dezenas
de operações inserem, apagam, invertem e soldam vértices, e cada uma teria de se lembrar de a
mexer. ⇒ a correcção é **ancorada na geometria** ([`CorreccaoDePeso`]): ela diz *«aqui»*, e
continua a dizer «aqui» depois de o artista mexer no desenho.

A bossa é `(1 − x²)²`, a **mesma** da lei euclidiana — `C¹` na borda por construção, logo a
correcção não põe um degrau no campo. ⭐ E por ser somada DEPOIS da lei, ela vale nas **duas**
(`Auto` do padrão-ouro · `Envelope`): *o artista corrige aquele ponto, e de que lei veio o peso que
ele está a corrigir não é pergunta dele.*

#### ⭐⭐⭐ O que a torna correcta: a mancha é pintada na POSE e guardada no REPOUSO

O artista vê a arte **deformada** — é lá que ele vê o defeito — e a correcção tem de viver na
geometria de repouso, senão ela andaria com a pose e corrigiria o sítio errado no quadro seguinte.
⇒ o dedo escolhe o ponto **POSADO** mais perto e o que se guarda é o **repouso desse mesmo ponto**
([`peso_a_mao`]). ⚠️ E o **centro nunca é o cursor cru**: ancorá-lo ali poria a mancha no vazio
quando o dedo passa ao lado da arte, e ela deixaria de corrigir exactamente quando o artista pensa
que a pôs.

#### O olho: o pincel deixou de ser cego

⛔ Corrigir um peso sem o ver é apontar para um número que não está na tela. Com o verbo armado,
cada ponto da arte presa é um **ponto colorido** pela influência do osso em foco — a rampa
`Info → Danger` que toda ferramenta de rig usa —, mais o **anel** do pincel (raio em MUNDO, porque
o raio *é* uma distância do desenho). ⭐ **Um peso de `0` é pintado, e é a metade que importa:** sem
ele o artista vê onde o osso já manda e **não vê onde ele devia mandar e não manda**.

⚠️ **O olho lê a MESMA porta que o quadro** (`weights_corrected`, com as manchas já dentro) — *uma
pré-visualização que ignora o trabalho feito faria o artista pintar duas vezes o que já pintou*.

#### As duas constantes, e o que cada uma é

| const | valor | o recurso |
|---|---|---|
| `MANCHAS_MAX` | `128` | o **relógio do quadro**: `205 µs` sobre `2 000` pontos com o tecto cheio (`--release`), `1,2 %` de um quadro de 60 Hz — `8 ×` abaixo do décimo que o gate exige |
| `FUSAO` | `0,5` | a **distância**: duas pinceladas a menos de meio raio uma da outra são a mesma mancha, e é isso que faz um arrasto custar o que ele percorre |

⚠️⚠️ **E a `FUSAO` quase ficou sem régua:** o gate óbvio (*«pintar duas vezes no mesmo sítio dá uma
mancha»*) fica **verde com ela a zero**, porque o centro é snapado ao ponto da pele e duas
pinceladas no mesmo sítio fundem por igualdade **exacta**. Quem a mede é a irmã, com dois pontos
**vizinhos** e o raio DERIVADO da distância entre eles. *Uma mutação que sobrevive é a régua a
dizer onde ela não olha* — e o tecto pagou a mesma lição (a estrela nunca o alcança; ele é medido
na LEI).

**Na tela:** a fileira do osso passa a ter **três** segmentos, e com o `Weight` armado aparecem
**`Brush Radius`** e **`Brush Strength`** (com sinal — negativo TIRA; ⛔ não há um segundo verbo
«apagar» a lembrar nem um modificador a adivinhar). `PROJECT_SCHEMA` **+1** — conte o DELTA.

⚠️ **O traço pertence à arte em que começou** (o alvo congela no press): com duas formas presas a
encostar-se, re-perguntar a cada evento poria metade da correcção no desenho errado.

#### ⛔⛔ E a FOTO mostrou que o verbo era inalcançável na própria cena dele

O painel dos ossos é a **única** porta dos três verbos (`Create` · `Transform` · `Weight`), e o
prólogo da cena `PH2D_VEC_BONE_SMOKE` só o abria no nível `=2` — logo o `=1`, que é o que **tem
arte presa**, mostrava um esqueleto cujas ferramentas o artista não conseguia alcançar. ⭐ *A cena
estava certa como DADOS e era impossível como GESTO* — a mesma forma que o `#15` da `line/components`
pagou, e que nenhum dos gates dela via, porque todos liam a cena como dados.
⇒ `painel_do_osso` passa a ser **incondicional** (a timeline e o enquadramento continuam do `=2`,
e é isso que mantém intacta a cena que o dono aprovou), com a morte da premissa **visível no diff**
do gate que a prendia. ⚠️ E o texto que a cena imprime deixou de mandar *«abra o painel Skeleton»*:
*uma instrução que descreve o app de ontem é mais cara que instrução nenhuma.*

#### ⚠️ A escolha do alvo é LEI, e vivia no laço de desenho

O tecto de LOC da fase do overlay obrigou o corte, e ele achou o defeito: *«de quem se mostram os
pesos»* — o alvo congelado do traço, senão o que está sob o dedo — estava escrita dentro do laço de
desenho da shell, **onde teste nenhum lhe chega**. ⇒ [`peso_a_mao::pontos_do_indicador`], com gate
de **quatro** braços e a fixtura de **duas** peles que é o que o torna discriminante (com uma só,
«o congelado ganha» e «o dedo escolhe» devolvem o mesmo bloco).

#### ⛔⛔⛔ O SMOKE DO DONO REPROVOU-O, e os DOIS reports eram o mesmo defeito com duas caras

*«a barra laranja não é subdividida o bastante (só tem pontos nas extremidades)»* · *«os pontos não
ficam coloridos (não há indicativo de peso)»*. Medido na peça REAL da cena (o `RoundRect` do braço,
3 ossos, `ppm 100`):

| osso | peso `0` | peso `1` | **entre** |
|---|---|---|---|
| `Bone 1` | 12 | 12 | **0** |
| **`Bone 2`** | **24** | 0 | **0** |
| `Bone 3` | 12 | 12 | **0** |

⭐ **O peso vive por VÉRTICE, e a barra tem `14` posições distintas — todas em quatro cachos nos
cantos, nenhuma ao longo do comprimento.** O report 1 descreve o modelo com exactidão: *o esqueleto
só pode mover os pontos que o desenho tem*. E o report 2 tem três causas, **as três minhas**:

1. ⛔⛔ **O passo do smoke que eu escrevi mandava clicar no `Bone 2`** — o único osso da cadeia que
   não possui **nada** naquela arte (o miolo de uma cadeia de 3 sobre 4 cantos). Com ele em foco
   todos os pontos leem `0`, e a tela fica de uma cor só. *Um passo que nomeia uma coisa AFIRMA que
   ela serve para o que o passo diz* — a família do §0.8, outra vez.
2. ⛔⛔⛔ **O raio de fábrica era `20` unidades de MUNDO = `2 000` px**, contra `702` px da peça
   inteira e `27,6` px entre dois pontos vizinhos: **`2,85 ×` a peça**. Um clique agarrava todos os
   pontos de uma vez e o anel era maior que a janela. ⇒ o raio passa a ser de **ECRÃ** (`40` px,
   com a tabela derivada: `4 ×` o raio de pick da casa, `1,45 ×` a distância entre vizinhos, `5,7 %`
   da peça), convertido a mundo nos **dois** sítios do despacho — com gate, porque converter num só
   deixaria o pincel a escolher a peça com um raio e a pintar com outro.
   ⚠️ **E o doc do anel ARGUMENTAVA a favor da unidade errada** (*«o raio do pincel É uma distância
   do desenho»*): verdade sobre o que a **mancha guarda** e falso sobre o que o **artista escolhe**
   — e um número de mundo **não pode ter valor de fábrica**, porque teria de saber a escala da cena.
3. ⛔⛔ **As três recusas do pincel eram `eprintln!`** — *uma recusa que só o terminal vê é um botão
   mudo*, e a porta para a tela tinha sido construída por esta mesma linha **um dia antes**. Elas
   entram agora na população `RecusaDoOsso`, que é a que o censo deriva.
   ⚠️ **E o censo da família reprovou ao recebê-las, duas vezes, cada uma por uma cegueira própria:**
   ele contava **chamadas num ficheiro só** (as do pincel saem do despacho, não da fase) e uma mesma
   recusa pode ter **dois** sítios que a levantam ⇒ passou a perguntar *«alguém consegue EMITIR
   isto?»*; e a 2.ª redacção, que procurava o NOME nas superfícies, acusou de muda a
   `VariosEsqueletos` — que viaja como **valor** vindo da porta e nunca é nomeada ⇒ o universo passa
   a ser as superfícies **mais os produtores da crate**.

✅ **RESPONDIDA pelo dono em 2026-09-19: *«primeiro 1 e depois o 2»***. A primeira FECHOU no mesmo dia
(ver **F28**) e ⛔ **a nota abaixo sobre ela estava errada — *«custo: zero de arquitectura»* foi
medido e é falso**: a caneta escreve no documento vivo e o `recook` deita-o fora todo quadro. A
segunda está ABERTA, e é a próxima.

⏳ **A pergunta, como ela foi posta:** pintar peso **entre** os vértices de
uma forma vectorial era impossível, porque não há lá peso nenhum para corrigir. Duas saídas:

- **acrescentar vértices com a caneta** onde se quer controlo — o modelo do Moho/Spine, e o gesto já
  existe nesta casa. Custo: zero de arquitectura.
- **deformar a forma por uma MALHA**, como a imagem já é. ⚠️ **E o preço NÃO é «a `ph2d-poly2d` já
  existe»**: ela parte de uma **grelha de ALFA** (*«geometria pura sobre uma grelha de alfa»*, diz o
  cabeçalho dela), logo a forma teria de ser **rasterizada** para se tirar a cobertura, e o desenho
  passaria a ser deformado por uma amostragem dele em vez de pelos próprios pontos — o que muda o
  que o traço é. ⛔ Arquitectura, e não é minha para decidir.

#### ⛔⛔⛔ E O SMOKE SEGUINTE REPROVOU OUTRA VEZ — *«nada fica vermelho e nada fica azul»*

⭐⭐⭐ **A causa foi a minha PRÓPRIA cura anterior a expor um segundo defeito que ela escondia.** A
porta que responde *«que arte está sob o cursor»* media a **distância ao PONTO posado mais perto** —
e os pontos de uma forma vivem nos CANTOS dela. Medido na barra da cena, com o cursor a meio do
comprimento:

| cursor em `x` | achou a arte? |
|---|---|
| `−8,4` (canto) | sim |
| `−8,0` … `−2,0` (todo o miolo) | **NÃO** |
| `−1,6` (canto) | sim |

Sem arte encontrada não há ponto nenhum para desenhar ⇒ a tela fica **vazia**, que é o report à
letra. ⚠️⚠️ **E enquanto o raio valia `20` unidades de mundo (`2 000` px) isto era invisível**, porque
a arte era sempre encontrada — *por acidente*. ⇒ **um número a fazer dois trabalhos esconde o defeito
do segundo enquanto estiver errado no primeiro**: aqui eram *«até onde o pincel alcança»* (um
tamanho de pincel) e *«que arte está debaixo do dedo»* (um teste de acerto), e o segundo **nunca foi
uma distância**.

⇒ a porta passa a perguntar pela **SILHUETA POSADA**: dentro do contorno achatado (par/ímpar) para um
caminho, dentro de um triângulo posado para uma imagem, com a proximidade a um SEGMENTO — e nunca a
um vértice — como rede para o caminho ABERTO, que não tem interior. ⛔ Tudo em geometria pura: a
crate é folha e não traz a `kurbo` para responder a um teste de ponto. ⭐ Quem CONTÉM ganha de quem
está perto, senão com duas artes sobrepostas o dedo pintaria a que só passa por ali.

#### ⭐ E a segunda observação do dono estava CERTA, com a cura ao contrário do que parece

*«parece que os pesos não são aplicados apenas nos nós, mas também nos handles (alças)»* — **é
verdade, e tem de continuar a ser**: o esqueleto transforma a âncora **e** as duas alças, e uma alça
parada com a âncora a andar quebrava a curva. O que estava errado era o **DESENHO**: numa forma de
cantos arredondados as alças ficam em posições distintas, logo `8` nós apareciam como **`24`
pontinhos**, e a leitura era ruído. ⇒ o indicador mostra **um ponto por NÓ**; a lei continua a
devolver os três. ⚠️ **E as duas metades são gateadas juntas**, porque as curas seriam opostas:
esconder o desenho e apagar o efeito leem-se igual numa tabela.

⭐ **Medido: âncora e alças nunca divergem nesta arte** (`divergem 0` nos três ossos), o que torna o
ponto do nó uma descrição fiel e não um resumo. ⛔ Elas **podem** divergir em geral (o peso sai da
POSIÇÃO, e uma alça longa alcança território de outro osso), e é por isso que a mancha continua a
apanhá-las pelo espaço.

#### O ponto era pequeno demais para a cor ser legível

| grandeza | píxeis |
|---|---|
| dois NÓS vizinhos da barra | `70,7` |
| o ponto de então (raio) | `2,5` — **`3,5 %`** do vão |
| o ponto de hoje (raio) | `5,0` — `14 %` do vão |

⚠️ O diâmetro de `10` px não é escolhido: é a família das alças de gradiente desta casa (`~9` px).
⛔ E a rampa foi **ilibada com número** antes de se lhe tocar: os dois extremos são tokens de hue
`25` e `235` — vermelho e azul de verdade.

**Resultado, pelo caminho do produto, com o cursor no meio da barra:** `8` pontos, **`4` vermelhos e
`4` azuis** no `Bone 1` e no `Bone 3`; uma cor só no `Bone 2`, que é a resposta CERTA (o osso do
meio de uma cadeia de três sobre oito nós não possui nada) — *e foi ele que o meu passo de smoke
mandou clicar*.

#### ⛔⛔⛔ E A FOTOGRAFIA ACHOU UM TERCEIRO DEFEITO QUE GATE NENHUM PODIA VER

Com os dois curados acima, a tela continuava vazia — e o que o mostrou foi **ver**, não medir. A
sonda `PH2D_VEC_WEIGHT_PROBE=1` arma o verbo, escolhe o osso pelo NOME e pousa o cursor no meio da
barra; com ela, o diagnóstico do caminho do produto diz `pontos=8` e **a tela não os desenha**.

⭐⭐⭐ **A causa: para escolher entre CAMINHO e IMAGEM eu perguntei `tem Sprite?`** — e no app (ao
contrário da fixtura de unidade) **uma forma vectorial também carrega um `Sprite`**. Toda arte
vectorial ia pelo ramo da imagem, onde o `SkinnedMesh` não parseia, a porta respondia *«não achei»*
e não havia ponto nenhum. ⚠️ **A porta certa já existia no mesmo ficheiro** (a `e_caminho`, escrita
para o indicador uma hora antes): *duas respostas à mesma pergunta divergem, e estas divergiram em
duas horas.*

⚠️⚠️ **E a fixtura de unidade estava VERDE sobre o defeito** porque a entidade dela não tinha
`Sprite` — *uma fixtura que não contém o fenómeno não prova nada sobre ele*. Ela passou a ter um, e
a mutação que repõe a pergunta pelo `Sprite` sangra.

#### ⚠️ E o que a foto mostrou a seguir mudou o SMOKE, não o código

Com tudo certo, na barra laranja o artista **continua a ver uma cor só** — e a razão é geométrica:
os `8` nós dela estão nos dois extremos, e **a câmara da cena corta a ponta esquerda**, que é a que
o `Bone 1` possui. Com o `Bone 1` vê-se o extremo direito (todo a `0`, azul); com o `Bone 3`, o
contrário.

⭐⭐⭐ **Na IMAGEM pintada da mesma cena o indicador é o que devia ser:** `925` pontos numa nuvem
densa de **azul → vermelho**. ⇒ *o pincel é demonstrável na mídia que tem malha, e quase inútil numa
forma de oito nós* — que é a pergunta de produto abaixo, agora com foto dos dois lados.

Mutação **27 de 27** a sangrar, **cinco** delas sobreviventes à primeira e curadas com gates novos.

⏳ **ABERTO e nomeado:** o caminho de **GPU** não conhece as manchas — dívida **com gate**
(`a_pele_da_placa_nao_conhece_as_correccoes_e_isso_esta_nomeado`), inofensiva só enquanto ele não
tiver consumidor de produto · não há botão de *limpar as correcções* (o `Ctrl+Z` e o valor negativo
cobrem-no) · e a mancha não é espelhada pelo `Mirror Branch`.

#### ⭐⭐⭐ F26-b — O TERCEIRO REPORT: *«as cores não ficam tão boas como no blender · o algoritmo continua considerando pesos em alças · as cores só aparecem se o mouse estiver sobre a forma»*

Três queixas, **três mecanismos diferentes**, e nenhuma delas era a mesma coisa que as duas rondas
anteriores tinham curado.

**(1) A rampa era uma CONFUSÃO DE CATEGORIA.** Ela era `ColorToken::Info → ColorToken::Danger`, e um
token semântico é escolhido para ser **CALMO** dentro do chrome (`Info` é `oklch(0,720 0,110 235)`,
um azul de baixa croma); uma leitura de VALOR é escolhida para ser **DISTINGUÍVEL**. São requisitos
opostos. Medida em OKLab com `21` amostras:

| rampa | caminho total | **pior passo** | uniformidade |
|---|---|---|---|
| `Info → Danger` (a de ontem) | `0,305` | `0,0131` | `0,739` |
| as 5 paradas da indústria, espaçadas em `t` (o porte INGÉNUO) | `1,435` | `0,0082` | **`0,057`** |
| as 5 paradas em OKLCH, por arco | `1,397` | `0,0383` | `0,443` |
| **as 5 paradas em OKLab, por ARCO** (a que shipa) | `1,393` | **`0,0497`** | `0,651` |

⭐⭐ **O número que decide é o PIOR passo e não o caminho** — é ele que diz se dois pesos vizinhos se
distinguem —, e por ele o **porte ingénuo da rampa da indústria seria PIOR que o que havia**
(`0,63 ×`): o verde puro é um **planalto**. ⇒ a cura não é mudar as cores, é **parametrizar por
comprimento de arco perceptual**, o que põe as paradas em `0,000 · 0,380 · 0,546 · 0,680 · 1,000`.
O verde — a única referência que um artista lê («metade») — desloca-se `+0,046`; o ciano, que
ninguém lê como número, paga os `+0,130`. ⚠️ **Ela não muda com o tema, e isso é lei:** *uma rampa
que muda com o tema deixa de ser a leitura de um número.*

**(2) «Pesos em alças» — a ordem do dono, agora na LEI e não só no desenho.** A ronda anterior curou
o **DESENHO** (um ponto por nó) e deixou a lei como estava, com a objecção escrita: *«uma alça parada
com a âncora a andar quebrava a curva»* (o `CubicWeight` do Rive). ⛔ **Ele repetiu, e a objecção
fica registada e NÃO VENCIDA** — mas a medição deu-lhe razão por um mecanismo que a objecção não
via: a mancha do pincel é um **bump radial**, vale `1` no centro (a âncora) e menos nas alças, que
estão ao lado. Medido na arte dele (a barra de `PH2D_VEC_BONE_SMOKE=1`, um dab de `amount = 1`):

| ponto | lei de hoje (peso do NÓ) | lei de ontem (peso da posição) |
|---|---|---|
| âncora | `0,5000` | `0,5000` |
| alça de entrada | `0,5000` | **`0,2150`** |
| alça de saída | `0,5000` | `0,5000` |

⭐⭐ **A assimetria é o mais duro:** das duas alças do MESMO nó, uma seguia e a outra não — a
tangente partia-se exactamente no ponto que o artista acabara de pintar. *Um peso que ele não
consegue entregar ao nó inteiro num gesto não é um peso que ele controla.*

⭐ **E a cura tem TRÊS metades, porque «pesos em alças» aparecia em três sítios:** a deformação
([`ph2d_vec_skin::aplica_corrigido`] faz **uma** conta por vértice, na âncora), o instantâneo posado
que o hit-test usa, e — a que ninguém tinha visto — **onde a mancha é ANCORADA**: o
[`ponto_sob_o_cursor`] escolhia entre todos os pontos, logo o centro de uma correcção podia cair
numa alça. Os três lêem a MESMA porta ([`ph2d_vec_skin::dono_do_peso`]).

⚠️ **Sem dab as duas leis CONCORDAM nesta arte** (as alças de uma quina ficam a `0,28` da âncora) —
é a pincelada que as separa, e é por isso que a régua que só olhava o repouso não via nada.

**(3) «As cores só aparecem se o mouse estiver sobre a forma» — DUAS perguntas lidas como uma.** O
indicador perguntava *«que arte está debaixo do dedo?»* — a mesma pergunta do pen-down, com a
justificação escrita no código —, e devolvia **vazio** no vão entre as formas. ⛔ *Onde o traço vai
pintar* é do DEDO; *o que este osso governa* é do OSSO, e não tem cursor nenhum dentro. ⇒ o
indicador mostra toda a pele cujos tendões contêm o osso em foco, sempre. ⭐ **A população é exacta
e não uma escolha:** uma pele que não o tenha é a que o `pinta` recusa com `OssoDeFora` — *pintá-la
de azul prometeria um pincel que a porta ao lado recusa*. Preço medido: **`2,16 ×` o `recook` da
mesma arte** (`27,75 µs` contra `12,83 µs` na estrela), com a razão gateada contra `4 ×` — *deixou
de haver um quadro barato (o dedo no vão) e um caro; todos passaram a custar o caro.*

**E a auditoria achou um QUARTO, da família do controlo morto:** o aviso do bind
(*«N pontos de controlo caem FORA do interior da forma»*) contava as **alças**, cuja linha da tabela
deixou de ser lida. ⇒ ele conta só os **NÓS** — *queixar-se de uma condição que já não tem consumidor
é ensinar o artista a ignorar a queixa*. ⚠️ As linhas das alças continuam a ser **gravadas** (a
tabela viaja em bytes opacos dentro do `SkinBind::source`) e ficam **dívida NOMEADA**: elas são
AMOSTRAS e não incógnitas — o sistema resolve-se na malha do domínio —, logo tirá-las não mexeria
num único peso de âncora. *É dívida de tamanho, nunca de resultado.*

⛔ **Três premissas MORRERAM e as três morrem à vista no diff:** o cabeçalho do
[`ph2d-vec-skin`] (*«cada metade com os pesos da posição dela»*), o gate
`the_three_halves_of_a_vertex_answer_to_their_own_position` (substituído por
`uma_alca_move_se_pelo_peso_da_ancora_dela`, **na mesma fixtura**, com o veredito invertido e o
controlo positivo dentro) e o `o_indicador_segue_a_arte_do_traco_e_nao_o_dedo`.

**E o ROTEIRO da cena passou a ENSINAR o pincel** — ele nunca o mencionava, em **três** reports do
dono sobre ele. ⛔⛔ **E o osso que ele nomeia é o do MEIO, nunca a PONTA, medido por FOTOGRAFIA:**
a 1.ª redacção reaproveitava o nome que a lição do *Onion* já tinha à mão (o da ponta) e dali a
parte visível do braço lê-se **quase toda azul** — a zona que a ponta governa sozinha cai atrás do
painel *Bones*. ⚠️ *É a mesma armadilha do «Bone 2» do 1.º report, e ela voltou porque o nome mais
fácil de alcançar no código não era o nome certo para o gesto.* ⭐ A derivação virou **porta**
(`osso_do_meio`) por causa de mais uma mutação sobrevivente: com ela inline, o gate tinha de
**copiar** a conta — e uma cópia julga a cópia.

#### ⭐⭐⭐ F26-c — *«no lugar de valores negativos em Brush Strength prefiro botões Add e Subtract»* (smoke APROVADO + ordem, 2026-09-19)

⛔⛔⛔ **A objecção estava escrita no painel e fica REGISTADA E NÃO VENCIDA:** *«o `Amount` é COM
SINAL, e é isso que faz o gesto ser um só: negativo TIRA peso. ⛔ Um segundo chip «apagar» seria a
segunda maneira de dizer a mesma coisa.»* ⭐ **O que ela não via** está agora escrito na
[`ph2d_tool_vector::WeightDirection`]: enquanto o sinal vivia dentro do número, *«tirar peso»* era um
**estado invisível** — o artista tinha de **ler um menos** para saber o que o próximo arrasto ia
fazer. ⇒ *uma pergunta, um controlo*: o número responde **QUANTO** (uma magnitude, que não tem sinal
que faça sentido) e os dois botões respondem **PARA QUE LADO**.

⭐⭐ **A composição é uma PORTA** ([`WeightDirection::delta`]) e não um `if` no despacho da shell:
*uma lei que só existe num laço de input é uma lei que ninguém pode contradizer* — foi exactamente
assim que a escolha do alvo do pincel viveu até esta manhã, e foi preciso um report do dono para a
descobrir. Gate na shell a exigir a porta **e** a proibir o sinal escrito ao lado dela.

⚠️ **Um negativo escrito à mão entra em valor ABSOLUTO, nunca cortado a zero:** cortá-lo deixaria o
pincel **inerte e calado**, que é a família de reports que esta casa já pagou três vezes. O campo é
re-semeado do estado da ferramenta a cada quadro ⇒ *o ecrã corrige-se à vista*.

⛔ **E escolher um lado NÃO arma o verbo `Weight`**, ao contrário dos três chips acima: a secção
destes dois só é pintada com ele já na mão, logo já se está lá — a mesma regra que os chips da
largura do lápis já escrevem, com gate nas duas metades.

⭐⭐⭐ **DUAS mutações sobreviveram e as duas eram achados de DESENHO:**
- **a lista de ids podia trocar de ordem sem nada acusar** — e com `[Sub, Add]` o segmento rotulado
  *Add* passava a mandar `Subtract`. ⚠️ *«Índice-alinhadas» era uma afirmação que nada verificava.*
  ⇒ o gate ata a posição ao SIGNIFICADO (`IDS[Add.indice()] == …_ADD`);
- **o id e o RÓTULO viviam em duas listas paralelas** ⇒ passam a viajar **emparelhados** numa porta
  só. *Um controlo que faz o contrário do que o rótulo dele diz é pior do que um controlo morto: o
  morto não engana.*

⚠️ **E o `populate` — a SÉTIMA vez que esta casa paga a lição:** os dois chips entram nele, com o
gate de costura a carregar-lhes com o **ponteiro REAL** (um `Click` sintético passa com o chip
morto). Mutação **10 de 10** a sangrar.

**Mutação `11 + 2` a sangrar, TRÊS sobreviveram à primeira e as três eram achados** (a barra da
rampa cega ao espaçamento uniforme em OKLab · o filtro por tendão que era a segunda resposta à
mesma pergunta · o instantâneo posado que podia divergir do desenho sem nada acusar). Portão:
**15 280** testes verdes, censos da árvore `90/90`, clippy `-D warnings` a zero. ⚠️ A `15 281.ª`
corrida acusou o `the_cost_of_depth_is_linear_not_explosive` — **membro nomeado** da família de
flakes de fan-out do `CLAUDE.md` §5.0, com **zero linhas** do diff naquela crate e a `line/UIUX` a
correr a suíte dela na mesma máquina (`load 30`); `2` de `3` verde isolado.

### F19 — ✅ **O CHIP `Auto` DIZ QUE LADO DERIVA** (report do dono, 2026-09-18)

*«IK Bend não funcionou com Auto IK e trocando CCw por CW no painel lateral»* — ⭐ **reproduzido, e
a lei estava CERTA.**

⛔⛔⛔ **A causa, MEDIDA** (`sonda_do_lado_do_joelho_tests`, no braço em **S** da cena dele): com o
`IK Chain` de **fábrica (`2`)** o `Auto` e o `Cw` dão a **MESMA pose, ao bit** (`0,0000`), e a cena
do osso **captura `Cw`** (lido do log dela). ⇒ o artista clica em **dois** dos quatro chips e não vê
nada mudar — indistinguível de um controlo partido. *Só o `Ccw` move (`3,0000`).* ⚠️ Com
`Chain = 3` o `Auto` **deixa** de coincidir (`2,9991`), e é essa metade que impede a leitura errada
*«o Auto é sempre o Cw»* — que levaria alguém a esconder um chip vivo.

⚠️ **A cura NÃO é esconder nem mexer no solver:** o `Auto` significa *«deriva o lado da pose que
chega»* e coincide **nesta** pose, não sempre. ⇒ ele **diz**: o rótulo passa a ser `Auto (CW)`, e o
artista vê, sem clicar, que pedir esse lado é um no-op.

⭐ **A porta do lado derivado é a MESMA que o solver usa** (`goal::side_for_chain`) — uma segunda
resposta a *«de que lado a pose está?»* divergiria da que governa a corrente, e o chip mentiria.

⛔⛔ **E TRÊS hipóteses caíram por medição antes desta**, cada uma com o número: a fiação do chip
está completa (`fase_bus_clicks` → `g.bend = lado`) · o painel **já** marca o chip activo · e os
chips **só** são pintados no osso que tem a âncora (a lei do controlo morto já lá estava).

⛔⛔ **E a minha régua mediu a grandeza errada, a QUARTA vez nesta sequência:** a 1.ª redacção lia a
**translação** do cotovelo e devolvia `[10, 0]` nos três lados — *o solver escreve ÂNGULOS*, e a
translação de um osso filho é fixa. *Uma régua que mede o campo que a lei não escreve dá sempre o
mesmo número*, e ela acusava o produto pelo report do dono.

⛔⛔⛔ **E uma MUTAÇÃO expôs uma cegueira do gate do elo:** ele lê o pintor por
`include_str!("section.rs")` e **a agulha que procura estava escrita nele próprio** ⇒ apagar a
chamada deixava-o VERDE. ⇒ os gates do rótulo mudaram para um ficheiro irmão. *Um gate
`include_str!` que procura uma string escrita nele mesmo não afirma nada.*

⛔ Tecto de LOC da fase (`208` contra `200`) curado por **CORTE** (`publica_a_ancora`), nunca por
uma entrada nova no `FN_OVERAGE_OK`.

Mutação **4 de 4** a sangrar; portão `15 099` verdes.

⏳ **ABERTO e nomeado:** o **pixel** do rótulo não é alcançável de um teste — o testkit desta casa
não tem leitor de texto pintado, e o que liga a lei ao pintor é um gate `include_str!`.

### F18 — ✅ **O LADO DA DOBRA ANIMA** (pedido do dono, 2026-09-18)

Ele escolheu *«o lado da dobra é escolhido e é consistente, mas **não é animável**»*. ⇒
`PropKind::IkBendSide` (id de fio **17**, append-only): uma track como qualquer outra, na lista do
*+ Track*, com rótulo, chave de i18n e alias de expressão (`Nome.bend_side`).

⛔⛔⛔ **E NÃO é o *Pole Target* — a recusa MEDIDA fica de pé, e está no doc do
[`ph2d_skeleton::BendSide`]:** em 3D o triângulo raiz–cotovelo–ponta roda em torno do eixo
raiz→ponta (um grau de liberdade **contínuo**, que um objecto no espaço fixa) e no plano isso **não
existe**: sobra **um bit**. Um alvo arrastável que codifica um bit dá a ilusão de um controlo
contínuo e **salta** ao cruzar a recta — Godot (`flip_bend_direction`) e Spine (`bendDirection`),
independentes, escolheram o interruptor. ⭐ *O que faltava não era o alvo: era a ANIMABILIDADE do
bit*, que o Spine tem e nós não tínhamos.

⚠️ **A convenção do valor é a do Spine** (`>= 0` ⇒ anti-horário) e o empate tem **vencedor
declarado**: sem isso o instante do salto dependeria do último bit de um `f32`. ⛔ O `Keep` e o
`Mixed` **não são alcançáveis** pelo canal, e a ausência é a decisão: os dois significam *«deriva o
lado da pose que chega»*, e um canal que os animasse estaria a keyar a AUSÊNCIA de uma escolha.

⛔ **Fora do auto-key**, e a razão está escrita: o valor muda por um clique num chip, e o auto-key
desta casa grava o que a MÃO move no canvas. *Pô-lo lá faria toda troca de chip virar uma chave.*

⛔⛔⛔ **E o achado da wave foi um `_ => None`:** os **quatro** `match` exaustivos da crate
obrigaram-me a responder pelo canal novo; o `from_target` — que traduz o id **opaco que o documento
grava** — tem wildcard, e a variante caiu nele **em silêncio**. Sem aquela linha uma track gravada
**não se resolve ao carregar**, e o gate genérico de ida-e-volta **saltava o canal por vacuidade**.
*Um `match` com wildcard é onde uma variante nova desaparece, e o preço ali é a PERSISTÊNCIA e não a
compilação.* Quem o mostrou foi uma **mutação** (a sonda podia escrever qualquer número e passava).

⚠️ **Duas cegueiras de régua, as duas registadas no ficheiro:**
1. o piso `checked >= 7` do gate genérico **segurava o número enquanto a população encolhia** ⇒ hoje
   é derivado (`checked == resolviveis`);
2. ⛔ e o derivado **também não apanha** a remoção da linha do `from_target`, porque `checked` e
   `resolviveis` descem juntos — *a régua partilha a lei do produto, e um espelho não acusa*. Quem a
   apanha é o gate do id, que afirma o `17` pelos **dois** lados.

⚠️ **Os gates vivem atrás da feature `skeleton`:** um `cargo test -p ph2d-timeline` sozinho imprime
`0 passed` — *um teste que não corre lê-se como verde*. O portão do workspace corre-os (a shell liga
a feature; conferido por `nextest list --workspace`).

⭐ De graça: a `ph2d-skeleton-ecs` passou a re-exportar o `BendSide` — o **segundo** caso que a nota
dela previa por escrito (*«um campo público cujo tipo não é alcançável pelo mesmo caminho é meio
campo»*).

Mutação **5 de 5** a sangrar; portão `15 094` verdes.

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

✅ **FECHADO na F27 (2026-09-19):** este item dizia que o censo cobria os **números** e que os
**verbos** tinham censo de *chegam ao barramento* e nenhum de *chegam a um efeito*. Os catorze têm-no
agora, e a construção devolveu o terceiro elo que faltava — *nada no repo perguntava se o braço que
recebe o pedido chama alguma porta*.

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
