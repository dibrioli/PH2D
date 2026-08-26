# Plano — a **máquina de estados do Morph**, autorada no canvas

> **Fila F1** ([doc 29](29_fila_morph_state_machine_e_texture_pattern.md)) · pesquisa em
> [doc 31](31_pesquisa_maquinas_de_estado.md) · o estado medido na reabertura está no §
> *RECONFERÊNCIA* do doc 29.
>
> Enio, 2026-08-24: *"um tipo de state machine específico para o tool Morph (…) entre múltiplas
> formas, de forma não destrutiva e funcional no runtime do game"*, com as setas *"no próprio canvas
> 2D (…) e nas setas colocaremos condições"*, e a configuração *"à seção states do módulo vector
> utilizando um modo preview"*. Em 2026-08-25, com as duas opções na mão: **"as setas devem ser
> desenhadas no canvas onde as formas foram desenhadas"**.

## §0 — O modelo, numa frase

⭐ **Um ESTADO é uma FORMA DESENHADA.** O artista desenha A, B e C no canvas e liga-as com setas; o
estado da máquina é *em qual delas ela está*, e a seta é *como se vai de uma para a outra*.

É a decisão que **apaga o caso especial**: nada de novo a nomear, nada de novo a gravar, e o que ele
vê **é** o modelo. Foi ela que a frase do Enio escolheu — *"setas de uma forma para outra"*.

### ⚠️ Isto NÃO é o `ph2d-ui-state`, e a correcção fica escrita

Na reabertura eu afirmei ao Enio que *"a máquina que contém a outra é a `ph2d-ui-state`"*. **Estava
errado**, e a medição que o mostra é sobre o produto, não sobre o código: aquela máquina interpola
**poses de N objectos** (translação, tinta, traço, geometria) entre papéis **fixos** de UI — hover,
pressed. Um estado dela é uma **gravação**, não uma forma no canvas. Aqui o assunto é **duas formas
e um `t`**.

> *Quem decide qual subsistema serve é o que o ARTISTA desenha, não o código que está por perto.*

O que de facto se reusa: o **motor** (`ph2d-spring`, `ph2d-anim::Easing` — os dois já folhas), o
**renderizador do par** (`VecMorph`, intocado), a **separação de undo** (`Driver::MorphT`, que já
existe) e o **vocabulário das condições** (as acções do Input Map, de ontem).

## §1 — A porta ÚNICA de cada pergunta

| Pergunta | A porta | ⚠️ |
|---|---|---|
| *que formas, que setas, que condições?* | `MorphGraph` (autorado, no documento) | as formas são **derivadas** das setas (`shapes()`) — uma lista à parte teria um estado que nenhuma seta alcança |
| *em que forma estou?* | `MorphMachine::current` | salta **no lançamento**, nunca na chegada nem na fila (§2) |
| *que par a cena mostra?* | `MorphMachine::pair` → `VecMorph::sources` | fica o par do ÚLTIMO voo com `t=1`; ⛔ **não** `(current, current)` |
| *onde no caminho?* | `MorphMachine::t` → `VecMorph::t` | e o undo não o vê — `Driver::MorphT` já existe |
| *a acção X aconteceu?* | `MorphMachine::fire(&str)` | ⛔ a crate **nunca** resolve o nome |
| *o que fazer daqui?* | `MorphMachine::live_actions` | a cura do medo do Animator (doc 31) |
| *ver a seta sem lhe dar nome* | `MorphMachine::travel(ix)` | a porta da pré-visualização |

## §2 — As leis, e por que cada uma é assim

1. ⭐ **Só as setas do estado CORRENTE disparam** (`MorphGraph::from`) — a correcção nº 1 da
   pesquisa (o *State Tree* do Unreal). Um varrimento global é a teia do Animator, construída por
   acidente.
2. ⛔ **Uma seta sem condição nunca dispara sozinha.** Ela nasce sem nome quando o artista a
   desenha; sem esta guarda, toda seta recém-desenhada responderia a uma acção de nome vazio.
3. ⭐ **O `current` salta no LANÇAMENTO** — a meio de `A→B` as setas oferecidas são as de `B`.
4. ⛔⛔ **Mas ele NÃO salta num pedido em fila**, e o gate nasceu vermelho a provar porquê: com o
   salto na fila, o segundo pedido era lido a partir de um estado onde a máquina ainda não está —
   ou não casava com seta nenhuma (*o input do jogador desaparecia em silêncio*), ou casava com uma
   seta cujo `from` não é onde ela aterra, e o par **saltava um estado inteiro**.
5. **Um pedido a meio do voo ESPERA a chegada** (*input buffer* de UM). ⛔ As duas alternativas
   estão fechadas: **ignorar** perde o input; **saltar** não é exprimível, porque o `VecMorph`
   guarda **um par** e sair do meio de `(A,B)` para `(B,C)` precisaria de uma mistura de três.
6. **O mais novo ganha e a fila não cresce** — uma fila funda reproduziria teclas que o jogador já
   esqueceu. Seguro **por construção** graças à lei 4: todo candidato parte do mesmo sítio.
7. **A fila é RECONFERIDA na chegada** — o artista pode ter apagado a seta durante o voo.
8. ⭐ **Chegar não troca o par.** O cache de `Plan` do `morph_live` é chaveado pela geometria em
   **mundo** das duas fontes, e a busca de fase custa os **5,9 ms** que o `Plan` foi inventado para
   matar; `t=1` em `(A,B)` já **é** a forma B.

## §3 — Onde encosta

* **Contrato congelado (§6):** nenhum. A crate é nova e o `VecMorph` **não se mexe**.
* ⛔ **Schema: NÃO sobe — esta linha do plano estava errada e a W2 mediu-a.** O `ComponentBlob` é
  chaveado por `blake3(nome canónico)`, **não** por posição: um ficheiro gravado antes desta wave
  simplesmente **não tem** o blob, e a entidade volta **sem máquina** — que é a leitura correcta de
  *"ninguém desenhou seta nenhuma"*. É a mesma lição que a `line/3DModeling` registou na W35 (*"a
  peça atravessa o arquivo e o `PROJECT_SCHEMA` não se mexe; a nota que dizia o contrário era
  velha"*), e há gate: `a_morph_without_arrows_comes_back_without_them`.
* ⚠️ **O que SOBE é a contagem do REGISTO, nos TRÊS sítios** — `ph2d-ecs` **70 → 71** e os dois
  espelhos (`ph2d-render`, `ph2d-script`) **71 → 72**. Número que **soma entre linhas**; ⛔ conte-o
  contra o `main` do dia, nunca o copie daqui.

## §3-bis — As duas leis de LEGIBILIDADE da seta (W3a), e as três tentativas que morreram

1. ⭐ **`A→B` e `B→A` não se sobrepõem.** Toda máquina útil tem pares de ida e volta, e duas rectas
   entre os mesmos dois centros são **uma** recta na tela. As setas curvam, e o lado sai de
   `perp(u)` — a normal da **direcção de viagem**, que inverte com ela **por construção**.
   ⛔⛔ **Duas tentativas anteriores morreram, e a segunda passou no gate pelo motivo errado:**
   a 1ª fazia `perp(u) · sign(ids)` e os dois fatores **cancelavam-se** (em `B→A` o versor inverte
   *e* o sinal inverte) ⇒ as duas setas dobravam para o mesmo lado; a 2ª construiu um "versor
   canónico" `u·sign` e tirou-lhe a normal com o mesmo `sign` — que é **algebricamente `perp(u)`**.
   O gate ficou verde, mas a mutação `sign = 1.0` **não o matava**, porque o `sign` não decidia
   nada. *Um gate verde sobre código morto continua verde — quem o apanhou foi a prova de mutação.*
2. **A ponta encosta na BORDA da forma, não no centro.** Uma seta que morre no meio de um
   rectângulo grande fica escondida por baixo dele.
   ⚠️ **E ela aponta na tangente de CHEGADA, não na recta centro-a-centro** — numa seta curva as
   duas divergem. ⛔ **A régua desta lei também teve de se corrigir:** o teste do **sinal** do
   produto interno **sobreviveu à mutação** (com 22 px de curvatura sobre um segmento de centenas,
   as duas direcções passam num teste de sinal). A régua é o **ÂNGULO**: o bissector das duas abas
   é *exactamente* o oposto da tangente de chegada.
3. ⚠️ **A curvatura é de ECRÃ.** Uma curvatura em mundo desapareceria ao afastar o zoom — e é
   exactamente com a máquina inteira à vista que a ida e a volta precisam de se distinguir.

⚠️ **A seta é CHROME, não desenho:** não se selecciona como forma, não exporta, não imprime. É por
isso que ela vive no overlay e **não** como um `VecPath` derivado, que é o que o **conector** faz.
*O conector é uma linha que o artista quer no produto final; a seta é a explicação de uma regra.*

## §3-ter — O gesto (W3b): o mesmo movimento do conector, outro produto

⭐ **`DrawMode::MorphLink`** — pressiona numa forma, arrasta, solta noutra. É **um modo próprio** e
não uma variante do `Connect`, e a razão é o **produto**: o conector faz uma **linha no documento**
(que exporta, imprime e se selecciona); esta faz uma **aresta num grafo**, que é chrome. *Dois
produtos atrás do mesmo movimento da mão precisam de dois modos, senão o artista não tem como dizer
qual deles quer.*

⚠️ **O hit-test é o MESMO** (`App::shape_under_cursor`), não uma cópia — a pergunta é literalmente
a mesma, e duas respostas divergiriam no dia em que uma anotação nova nascesse.

* ⛔ **Sem um Morph SELECIONADO o gesto é inerte**, e é a resposta honesta: uma seta é uma aresta no
  grafo de alguém. Criar um `VecMorph` do nada a meio de um arrasto poria no documento um objecto
  que o artista não pediu.
* ⭐ **A PRIMEIRA seta faz nascer a máquina**, com `start` na forma de onde ela parte — é por isso
  que o `VecMorphMachine` não tem `Default`: o `start` é um facto do **gesto**.
* ⛔ **Uma forma não se liga a si própria.** ⚠️ E isto **não** é a decisão do conector, que aceita o
  laço de propósito: lá o laço é um **desenho** legítimo; aqui seria uma regra vazia.
* ⚠️ **Uma seta repetida não se duplica** — duas arestas iguais seriam duas linhas idênticas no
  painel, uma impossível de distinguir da outra ao apagar (a lei que a ligação de tecla do Input
  Map já paga).
* A seta **em voo** é **recta**: a curvatura existe para separar a ida da volta, e uma seta sem
  destino não tem par de quem se separar.

⚠️ **A lei mora FORA do `impl App` (`link_shapes`), e é isso que a torna gateável:** o gesto precisa
de um `AppGfx` — janela real e superfície de GPU —, que um teste não alcança (a mesma parede que o
undo do filtro do sculpt3d registou). ⇒ a **lei** tem gate de comportamento; a **costura** tem gate
de texto, e sem essa metade os outros quatro ficariam verdes sobre uma feature que **gesto nenhum
alcança**.

⛔⛔ **DEZ sítios para um modo novo, e o décimo só apareceu porque um gate o disse.** Eu editei
nove — o `enum` · o `NodeId` · o censo de ids · o re-export do painel · o `populate` (sem ele o
pill nasce **morto sob o ponteiro**) · a fileira pintada · o clique→modo · o gate de costura
id→modo · o rótulo i18n — e o portão reprovou com **`Ignored` em vez de `Consumed`**: havia uma
**décima** lista, uma allowlist de `VECTOR_MODE_*` em `event_clicks.rs`, e sem ela o clique era
**engolido** e o modo nunca trocava.

⚠️ **O pill teria ficado pintado, registado, e inerte** — o terceiro membro daquela família nesta
linha, e o único que nenhuma das minhas nove edições apanharia. *Uma feature espalhada por dez
listas escritas à mão só é alcançável se um gate percorrer as dez* — e a mensagem daquele gate
**nomeia o ficheiro que falta**, que é o que o torna útil em vez de só vermelho.

## §3-quater — A secção (W4a)

⭐ **Ela vive na MESMA seção *States*** que as poses de UI — Enio, 24/08 — e a lei da casa concorda:
**o Inspector mostra o que o objecto TEM** (ADR-0166). Um objecto raramente é as duas coisas, então
não é preciso aba nenhuma.

* **Duas linhas por seta**, o corte do `paint_signals`: `de -> para` + a lixeira em cima; a
  **condição** em baixo. Espremer as duas numa daria um chip de meia dúzia de caracteres.
  ⛔ **`->` em ASCII**: a fonte da casa não cobre o bloco de setas do Unicode e o glifo sairia caixa
  vazia (gate `no_tofu_glyphs`, que já mordeu três vezes neste repo).
* ⚠️ **A condição é um MENU das acções do Input Map, nunca um campo de texto.** Um nome digitado
  pode não existir, e uma seta que espera uma acção inexistente **nunca dispara** — sem uma palavra
  na tela. *Um modelo que aceita o que o painel não mostra produz estado inalcançável.*
* ⚠️ **A opção `0` é o «—»**: tirar a condição tem de ser um gesto, senão o artista só poderia
  apagar a seta inteira para se arrepender.
* ⭐⭐ **Um Morph SEM máquina publica a face VAZIA, e nunca `None`.** `None` = *"a seleção não é um
  Morph"* (a seção nem fala de setas); vazio = *"é um Morph e ainda não tem setas"* — e é essa face
  que **diz o gesto** (nomeia o pill e o movimento da mão). Sem ela o artista vê um cabeçalho e nada
  por baixo, e *"não há setas"* e *"isto está partido"* leem-se igual.
* ⚠️ **O Morph é achado na seleção INTEIRA**, nunca em `sel.first()`: tocar num morph traz o grupo,
  e a seção mostraria as setas de um objecto enquanto o clique escrevia noutro (a lição do
  `host_of_selection`).
* ⚠️ **As acções são PUBLICADAS pela shell**, e o índice escolhido resolve-se contra **essa mesma**
  lista — uma segunda leitura poria o nome escolhido a apontar para outro no quadro em que o artista
  criasse uma acção.
* ⛔ **A seta NÃO tem botão de «percorrer» nesta wave, e a ausência é deliberada:** o que ele faria é
  pôr a máquina VIVA a andar, e ela nasce na W5. Um botão pintado antes disso é um clique que não
  faz nada — *é assim que o artista aprende a não confiar nos botões desta seção*, e é a lei que a
  própria seção de poses já escreve (*"Show e Clear só existem depois do Rec"*).

## §3-quinquies — A máquina a correr (W5)

⭐ **O «modo preview» que o Enio pediu é o RELÓGIO A ANDAR** — e ele já existia. Neste editor *"o
jogo a correr"* é o playhead, a mesma porta pela qual o dedo do jogador alcança a física; uma
terceira noção de runtime seria uma terceira coisa para o artista aprender.

⛔⛔ **E a guarda não é conservadorismo.** A condição de uma seta é uma **acção do Input Map**, isto
é, uma **tecla**. A escutar durante a edição, carregar em `Z` faria a forma mudar **e** o que quer
que o `Z` faça no editor — os dois, sem que nada na tela explicasse. É o argumento do `ui_preview`
(*"um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável"*) com
outro dispositivo de entrada, e a resposta é a mesma: **um modo**.

* ⭐⭐ **PARAR o relógio devolve a forma autorada, e isso não custa código:** é o que o ledger faz
  por construção. *Sair restaura o MUNDO, nunca «vá para o estado inicial»* — que moveria o desenho.
* ⛔⛔ **O `Driver::MorphT` sozinho NÃO bastava, e o plano já o tinha medido:** ele cobre o `t` e
  **só** o `t`. Sem o **`Driver::MorphPair`** novo, trocar de par durante a reprodução entrava no
  undo como se o artista tivesse re-ligado as fontes à mão. ⚠️ Dois motores sobre o mesmo
  componente é seguro **porque os campos são disjuntos** — a curva da timeline escreve o `t`, a
  máquina escreve o par; é a forma do par `SpriteAnim`/`SpriteAlpha` sobre a `Sprite`, e ali o doc
  avisa que *duas granularidades sobre o mesmo componente escreveriam por cima uma da outra*.
* ⚠️ **O ledger PRIMEIRO, a escrita depois** — ele precisa do valor ANTES para saber o que repor.
* ⚠️ **`just_pressed`, nunca `pressed`:** uma tecla segurada re-disparava a cada quadro e a máquina
  saltava a cadeia inteira num piscar de olhos.
* ⚠️ **Uma máquina cuja entidade morreu some junto** — senão sobreviveria ao objecto e o mapa
  cresceria para sempre (a varredura das `UiMachines`).

## §3-sexies — A cena (W6)

`PH2D_BUILD_SMOKE=75`. Ela arma o **material** — três formas bem diferentes (larga · alta · fina) e
um Morph entre as duas primeiras — e ⛔ **nada nasce ligado**: nenhuma seta, nenhuma condição.

⚠️ **É deliberado, e é a disciplina das cenas irmãs desta linha:** quem desenha as setas e escolhe o
que as dispara é o artista, e é **exactamente essa** a costura que a wave existe para provar. *Um
smoke que arma o gesto por baixo do pano pula a costura que devia testar.*

⚠️ **As acções são as de FÁBRICA** (`jump`, `dash`) — nenhuma nova é criada. A lista que o menu da
condição mostra é a do **projecto**, e usar a que já lá está prova que o vínculo com o Input Map é
real em vez de o simular. ⚠️ **E o roteiro nomeia a TECLA lida do mapa VIVO**, nunca de memória: se
o artista já remapeou, o texto acompanha.

⚠️ **A diferença entre as três formas é o instrumento**: um morph entre dois rectângulos parecidos é
indistinguível de *nada a acontecer*.

⭐ **Os passos 6 e 7 são os CONTROLOS**, e são o que separa esta feature de uma que parece funcionar:
parar o transporte tem de **devolver a forma desenhada** e deixar o Ctrl+Z **sem nada** para
desfazer; e com o transporte parado a tecla **não pode mover nada**.

## §4 — As waves

| | | estado |
|---|---|---|
| **W1** | **A LEI**, folha (`ph2d-morph-machine`): grafo · setas · condições · fila · mola/curva | ✅ **2026-08-25** — 13 gates, **8 mutações, 8 sangraram** |
| **W2** | O componente + a persistência | ✅ **2026-08-25** — 2 gates, 1 mutação, e ⛔ o `PROJECT_SCHEMA` **não** se mexeu (§3) |
| **W3a** | **O CANVAS, metade de VER**: as setas desenhadas entre as formas | ✅ **2026-08-25** — 8 gates, **6 mutações, 6 sangraram** |
| **W3b** | **O CANVAS, metade de AUTORAR**: `DrawMode::MorphLink` — arrastar de uma forma para outra cria a seta | ✅ **2026-08-25** — 5 gates, **4 mutações, 4 sangraram** |
| **W4a** | A secção **States**: a lista de setas + a **condição** (menu das acções do Input Map) + apagar | ✅ **2026-08-25** — 7 gates, **6 mutações, 6 sangraram** |
| **W4b** | O **ritmo** por seta (duração · curva · mola) — e o botão de **percorrer**, que precisa da máquina viva (W5) | ⏳ |
| **W5** | A **máquina VIVA** + o ledger de undo | ✅ **2026-08-25** — 5 gates, **5 mutações, 5 sangraram** |
| **W6** | A cena de smoke (`PH2D_BUILD_SMOKE=75`) | ✅ **2026-08-25** |

⚠️ **O que a W5 vai encontrar, e está medido de antemão:** o ledger de pré-visualização
(`preview_drive.rs`) já tem `Driver::MorphT` / `Driven::MorphT(f32)` — construído em 23/08 para a
curva de `Morph` da timeline. Ele guarda **o `t` e só o `t`**. A máquina escreve **também o
`sources`**, e esse facto **não tem dono no ledger** ⇒ sem o acrescentar, mudar de par durante a
pré-visualização entra no undo.

> ⚠️ **Meça cada linha deste plano antes de a honrar.** Escrito em 2026-08-25.

---

## §5 — ⭐⭐ O PIVÔ de 2026-08-25: um botão, o grafo completo, e as setas deixam de se ver

> Enio, depois do smoke:
>
> > *"É impressão minha ou vc contaminou ou até mesmo estragou a feature states previamente
> > implementada? Os states de morph deveriam ter sessão exclusiva. Outra coisa: melhor criar um
> > modo automático de atribuição: 1) o usuário seleciona todas as peças que estarão envolvidas na
> > máquina de estados do morph. 2) Com o clique de um único botão um objeto novo surge na
> > hierarquia tendo como filhos as shapes escolhidas. Todas as setas são atribuídas automaticamente
> > cobrindo todas as morphs possíveis entre todas as formas (tanto de ida como de volta). As setas
> > são virtuais e ninguém jamais vê. No canvas uma única shape aparece (a shape do estado atual) e
> > as demais ficam ocultas. Na seção exclusiva dos estados no painel aparece a opção de atribuição
> > de inputs para cada uma das morphs possíveis. Restaure ao original o painel e funcionamento da
> > seção states para criação de animações."*

### §5.1 — A contaminação: o que a W4 fez, e por que o argumento dela era irrelevante

A W4 pôs as transições do Morph **dentro** da seção `States` — a das poses de UI e do Smart Animate.
O argumento escrito no código era o [ADR-0166](../architecture/decisions/) (*o Inspector mostra o que
o objecto **TEM***) mais *"um objecto raramente é as duas coisas"*.

⚠️ **Os dois são verdadeiros, e nenhum deles era a pergunta.** O efeito prático foi:

| o que a seção `States` fazia antes | o que passou a fazer |
|---|---|
| aparece com uma forma única na seleção | aparece **também** por causa de um Morph |
| o corpo é sempre poses | o corpo pode ser transições de outra feature |

⇒ *A lei do ADR-0166 diz **o que mostrar**, nunca **onde**.* Duas features com donos diferentes,
histórias diferentes e gates diferentes debaixo de um cabeçalho só é **uma porta a mais na seção de
quem chegou primeiro** — e quem chegou primeiro é quem paga a regressão.

⛔⛔ **A causa de fundo é a mesma da auditoria do Input Map, e é o achado que importa:** dos doze
gates da W4, **nenhum olhava para o que era PINTADO**. Todos mediam o mapa e o estado publicado.
É por isso que doze verdes conviveram com um cabeçalho alheio a aparecer. O gate novo
([`seam_morph_states.rs`](../../crates/ph2d-panel-vector/tests/seam_morph_states.rs)) mede a
**ausência nos dois sentidos** — um morph não pinta o cabeçalho `States`, e poses não pintam o
`Morph States` —, e a mutação que repõe a forma exacta da W4 sangra com essa mensagem.

**A restauração é literal:** `paint_states.rs` voltou por `git checkout main --` e o
`git diff main` desse ficheiro é **vazio**.

### §5.2 — O modo automático: `n` formas, `n(n-1)` transições, um passo de undo

O clique em **Make Morph States** faz quatro coisas **no mesmo quadro**:

1. nasce o objecto (um `VecPath` vazio + `VecMorph`, que é o que a cena **desenha**);
2. as formas escolhidas viram **filhos** dele (`ChildOf`), na ordem de z;
3. cada uma ganha `Visibility::hidden()` — *no canvas aparece uma forma só*;
4. o `VecMorphMachine` recebe o **grafo completo dirigido** sobre elas.

⚠️ **Um passo de undo, não quatro.** As quatro escritas caem no mesmo quadro e o `post_frame_undo`
regista por DIFF. Reparentar num quadro e esconder no seguinte daria dois passos, e o primeiro
deixaria o artista com `n` formas empilhadas.

⚠️ **`sources = [start, start]` e `t = 0.0`**, e **não** `VecMorph::new` (que nasce a `t = 0,5` de
propósito, para um morph autorado *se anunciar*). Aqui é o contrário: o conjunto tem de mostrar
**exactamente** o estado inicial, senão a primeira coisa na tela é uma forma que ninguém desenhou.

⚠️ **A ordem das arestas é determinística** (`from` externo, `to` interno, ambos na ordem dos
membros). A lista do painel indexa por **posição**: uma ordem que dependesse de iteração de mapa
faria o menu de uma linha escrever a condição noutra depois de um undo.

⚠️ **Toda transição nasce SEM condição.** É a metade que torna o grafo completo seguro — se cada
aresta nascesse com uma acção, um conjunto de 9 formas nasceria com 72 regras a disparar todas na
primeira tecla.

### §5.3 — O TETO, medido: 9 formas

O recurso é o **relógio de pintura do painel**, e o número que manda é o de **formas** (o artista
escolhe formas; as setas são derivadas). Medido com o `MockPanelHost` a pintar o painel Vector
inteiro, release, 2026-08-25 — o painel **sem** esta seção custa `0,746 ms`:

| formas | setas `n(n-1)` | painel | delta da seção | % de um quadro de 16,7 ms |
|---:|---:|---:|---:|---:|
| 7 | 42 | `1,181 ms` | `0,435 ms` | 7,07 % |
| 8 | 56 | `1,330 ms` | `0,584 ms` | 7,97 % |
| **9** | **72** | **`1,497 ms`** | **`0,752 ms`** | **8,97 %** |
| 10 | 90 | `1,699 ms` | `0,954 ms` | 10,18 % |
| 11 | 110 | `1,899 ms` | `1,154 ms` | 11,37 % |

Slope linear: **`0,0104 ms` por linha**. ⇒ **9 é o último `n` em que esta seção sozinha não custa
mais do que TODO o resto do painel junto** (`0,752` contra `0,746`). Em 10 ela passa a custar mais
que todas as outras seções somadas, e o painel existe para responder a mais perguntas do que esta.

⛔ **O pool de ids NÃO é o recurso** — foi a primeira hipótese e a medição refutou-a: registar 132
linhas × 25 widgets custa `0,293 ms` **uma vez**, no `populate`, nunca por quadro.

⛔ **A régua não ficou como gate.** Ela divide dois relógios, que é exactamente a família de flakes
sob fan-out do `CLAUDE.md` §5.0 — a tabela acima é o registo, e re-medir é rodar a sonda.

### §5.4 — ⛔ RECUSAS MEDIDAS desta wave (não reconstrua sem ler)

| o que foi retirado | por quê |
|---|---|
| **as setas desenhadas no canvas** (W3a, `morph_arrow_overlay.rs` + gates) | *"as setas são virtuais e ninguém jamais vê"* — decisão directa do dono. E desenhar `n(n-1)` setas entre formas que **já estão escondidas** é ruído sobre uma resposta que ninguém precisa de ler no canvas. |
| **o modo de arrasto forma→forma** (W3b, `DrawMode::MorphLink` + `morph_link_gesture.rs`, dez sítios) | o grafo passou a ser **completo por construção** ⇒ o arrasto criaria uma aresta **que já existe**. Um gesto cujo produto já está lá é um gesto que não faz nada, e o pill dele competiria com treze irmãos pela fileira. |
| **a lixeira por linha** (`ArrowCmd::Delete`) | o conjunto de arestas é uma **função pura** das formas. Apagar uma linha seria apagar uma passagem que a próxima derivação repõe. *Desligar uma transição é tirar-lhe a condição* — o «—» do menu —, e uma seta sem condição existe e nunca acontece. |

⚠️ **A razão de fundo é uma só:** *duas portas para a mesma pergunta divergem em silêncio*. Com o
botão a gerar `n(n-1)` e um arrasto a acrescentar à mão, a lista deixaria de ser derivável e a
próxima derivação apagaria o trabalho do arrasto.

### §5.5 — As provas de mutação (11, todas sangraram)

Arnês: verde-antes · a mutação **compila** · o gate **correu** (`running 1 test`) · restore por
escrita.

| mutação | gate que sangrou |
|---|---|
| `from != to` passa a aceitar o laço | `the_graph_covers_every_ordered_pair_in_both_directions` |
| `start` = a **última** forma | `the_first_shape_chosen_is_the_start` |
| os membros **não** se escondem | `the_set_owns_the_shapes_hides_them_and_shows_the_start` |
| o conjunto nasce a `t = 0,5` | `the_set_owns_the_shapes_hides_them_and_shows_the_start` |
| um Morph vira estado de outro conjunto | `a_morph_is_never_a_state_of_another_set` |
| o teto de formas deixa de valer | `one_shape_or_too_many_refuses_without_littering_the_scene` |
| ⭐ **a forma EXACTA da W4 de volta** (`ui_states_section` consulta o morph) | `a_morph_machine_never_makes_the_ui_states_section_appear` |
| a seção do Morph pintada **sempre** | `ui_poses_never_make_the_morph_states_section_appear` |
| o botão aparece com **uma** forma | `one_shape_offers_no_button_at_all` |
| o botão sai da allowlist do painel | `the_make_button_is_alive_and_reaches_the_bus` |
| o botão sai do `populate` (morto sob o ponteiro) | `the_make_button_is_alive_and_reaches_the_bus` |

---

## §6 — ⭐⭐ W9 (2026-08-25, o 2º smoke): o MODO, o alinhamento e a pose

> Enio, depois de o conjunto funcionar:
>
> > *"Funciona corretamente mas precisamos de um modo preview (com botão) como o de states de
> > animação pois senão temos conflitos de atalhos (como setas do teclado movendo as formas).
> > Outras coisa: aqui diferente da tool morph, todas as peças participantes são alinhadas numa
> > mesma posição e o morph states faz o morph numa mesma posição, não desloca a peça de lugar.
> > O objeto criado como pai (o morph states da hierarchy) tem que ser arrastável no canvas como um
> > objeto qualquer, embora com a forma da shape ativa no momento, e deve arrastar os filhos junto."*

### §6.1 — ⛔⛔ O playhead era a porta e deixou de ser

A W5 escreveu, com todas as letras:

> *"E o modo já existe: neste editor, «o jogo a correr» é o **playhead a andar** — a mesma porta
> pela qual o dedo do jogador alcança a física. Uma terceira noção de runtime seria uma terceira
> coisa para o artista aprender."*

⚠️ **É um argumento bom sobre a coisa errada.** O playhead **não tranca o teclado do editor**: com
ele a andar, as teclas continuam a chegar a todos os atalhos. ⇒ a mesma tecla morfa a forma **e**
faz o que ela faz no editor — que é exactamente o que aquela nota dizia estar a evitar. O smoke
mostrou-o na forma mais visível possível: *as setas do teclado a mover as formas*.

⇒ **A porta é o interruptor `Preview`** da seção, e ele **toma o teclado**:

| onde | o quê |
|---|---|
| `input_dispatch/keyboard.rs` | a guarda, **depois** do `input.apply_event` e **antes** de todo atalho |
| depois | a acção continua a ser alimentada ⇒ a máquina lê; nenhum atalho do editor vê a tecla |
| a excepção | **Esc** (pede a saída) e os acordes com `Ctrl`/`Super` (`Ctrl+S`, `Ctrl+Z`) |

⚠️ **A posição da guarda é o desenho inteiro.** Barrá-la *antes* do retrato dos dispositivos
mataria a própria acção que a máquina lê: o modo ficaria inerte **com o teclado tomado** — o pior
dos dois mundos. Há gate sobre a ordem (`kb.find(apply_event) < kb.find(guarda)`).

⚠️ **A porta de saída é anunciada, e aqui é obrigatório:** este modo come exactamente as teclas com
que o artista tentaria escapar dele. *Um modo que consome a própria tentativa de sair lê-se como
travado.*

⛔ **Uma porta, não duas.** Deixar o playhead a dirigir também manteria o conflito vivo na porta que
não tranca nada. *Um modo cuja entrada não exclui os outros consumidores não é um modo — é mais um
produtor.* O gate afirma a ausência: o `playhead.is_playing()` não pode voltar àquela chamada.

### §6.2 — O alinhamento: a diferença de PRODUTO entre o Morph e o conjunto

| | o que faz | por quê |
|---|---|---|
| **Morph** (a tool) | as fontes ficam **onde foram desenhadas**; a forma viaja entre elas | é um efeito de **transição** — a viagem é o produto |
| **Morph States** | os estados são **alinhados num ponto só**; a forma muda **em lugar** | é **um objecto** que muda de aparência: a personagem que agacha não salta dois metros por isso |

A conta (`morph_set::align`): para o membro `i` com pose de mundo `translate(t) · M` e centro de
mundo `c_i`, a pose **local** nova é `translate(t − c_i) · M`; o conjunto nasce em `C` = centro da
caixa que continha todos. Compondo: `C + c_i − c_i = C`, **o mesmo ponto para todos**.

⚠️ **Só a translação muda** — rotação, escala e cisalhamento sobrevivem. Endireitar um estado ao
alinhá-lo seria destruir o desenho para o pôr no sítio, e *a diferença entre as formas é o que a
máquina existe para mostrar*.

⚠️ **A caixa é a da CURVA** (`path_curve_bbox`), com os **quatro** cantos transformados: sob rotação
a caixa alinhada aos eixos do mundo não é a imagem da caixa local, e dois cantos poriam uma forma
girada fora do centro.

⚠️ **Mede tudo antes de escrever qualquer coisa** — medir e escrever intercalados fariam o segundo
membro ser medido contra um mundo que o primeiro já mexeu.

### §6.3 — ⭐⭐⭐ A pose: por que um Morph não se arrasta e um conjunto sim

O mecanismo que torna um Morph comum **não-arrastável** nunca foi uma proibição — é o `recook`:

1. ele coze as fontes em **MUNDO** e escreve isso como geometria do morph;
2. logo, uma pose por cima levaria o afim **outra vez** ⇒ a forma andaria o dobro;
3. por isso ele **zera o `Transform`** todo quadro ⇒ arrastar volta ao sítio no quadro seguinte.

⇒ **o conjunto escapa pelo passo 1, não pelo 3.** Ele coze as fontes nas poses **LOCAIS** dos
filhos — que são filhos DELE —, então o que fica guardado é geometria do referencial do conjunto, e
o `Transform` aplica-se **uma** vez, como em qualquer forma do documento. O `!is_set` do passo 3 é
consequência, não causa.

⭐ **E o plano não se refaz ao arrastar:** as poses locais dos filhos não mudam quando o pai anda,
então `a` e `b` são os mesmos bytes e o cache do `plan_for` acerta. *Arrastar um conjunto de nove
estados custa o que custa arrastar um rectângulo.* (Há gate: a geometria guardada é byte-idêntica
antes e depois do arrasto.)

⚠️ **Os filhos vão junto de graça** — eles são `ChildOf(host)`, e a travessia de mundo já compõe.

### §6.4 — As provas de mutação da W9 (7, todas sangraram)

| mutação | gate |
|---|---|
| o `align` não corre (as formas ficam lado a lado) | `every_state_is_centred_on_the_set_so_the_morph_never_travels` |
| alinhar **endireita** o estado (perde rotação/escala/skew) | `aligning_moves_the_position_and_nothing_else` |
| a pose do conjunto é **zerada** pelo `recook` | `dragging_the_set_carries_the_states_and_the_drawing` |
| o conjunto volta a cozer em **MUNDO** (afim entra 2×) | `dragging_the_set_carries_the_states_and_the_drawing` |
| o interruptor sai do `populate` (morto sob o ponteiro) | `the_preview_toggle_is_alive_and_reaches_the_bus` |
| o interruptor deixa de registar hit-rect **com o modo ligado** | `the_way_out_stays_clickable_while_the_mode_runs` |
| o interruptor é pintado **sem máquina** | `a_selection_without_a_machine_offers_no_preview_toggle` |
| a porta modal deixa de ser chamada (a tecla faz duas coisas) | `the_arrow_click_reaches_the_world` |
| o conjunto novo **não fica seleccionado** | `the_arrow_click_reaches_the_world` |
| uma forma **com dono** entra num segundo conjunto | `a_shape_that_already_belongs_to_a_set_is_never_offered_to_another` |
| converter em curvas deixa a máquina **órfã** | `converting_the_set_to_curves_takes_the_machine_with_it` |

### §6.5 — Três buracos que a W9 fechou, e os dois primeiros são a MESMA porta

⛔⛔ **Depois de criar o conjunto, a selecção ficava nos MEMBROS.** Eles acabam de ficar ocultos e
com dono — e `morph_of_selection` não acha morph neles, então a seção voltava a oferecer
*"Make Morph States"* **sobre as mesmas formas**, prometendo um segundo conjunto por cima do
primeiro. Duas curas, porque são duas portas para o mesmo defeito:

1. o objecto novo **fica seleccionado** (`vec_pen.select_many`, a mesma escolha do botão Morph ao
   lado) — fecha a porta do fluxo normal;
2. o `eligible` **exclui uma forma cujo pai já tem `VecMorphMachine`** — fecha a porta da
   Hierarquia, onde um membro oculto continua a ser clicável.

⚠️ ***A primeira mutação SOBREVIVEU***, e foi ela que ensinou o resto: eu tinha escrito o
`select_many` e **nenhum gate o cobria**. Tirá-lo deixava a suíte inteira verde. *Uma afirmação que
mutação nenhuma mata é uma afirmação sobre nada* — a agulha entrou no gate de costura depois.

⛔ **E converter um conjunto em curvas deixava a máquina ÓRFÃ:** o `drop_relation_hosts` tirava o
`VecMorph` e deixava o `VecMorphMachine`. A seção continuava a listar as `n(n-1)` transições e o
interruptor `Preview` a acender — sobre um objecto que já não tem morph nenhum a dirigir. *Um painel
que oferece o que o mundo não pode fazer é pior que um painel vazio.*

⚠️ **E um gate que a W8 não tinha, e a lei que ele nomeia:** `the_new_set_actually_draws_the_start_shape`.
Os outros mediam **componentes**; nenhum media o que sai no canvas — e o par do conjunto é
**degenerado** por construção (`[start, start]`). Um plano de correspondência que recusasse um par
igual deixaria o path **vazio**: o artista carregava no botão, as formas escolhidas desapareciam
(ficam ocultas) e **nada** aparecia no lugar delas. *Contar o trabalho FEITO não é contar o trabalho
ENTREGUE.*

---

## §7 — ⭐⭐⭐ W10: a tecla pertence à FORMA, e a lista encolhe de `n(n-1)` para `n`

> Enio, 2026-08-25:
>
> > *"em vez de um evento para cada transição, melhor seria um evento por shape. Ou seja: se a seta
> > para cima leva ao retângulo azul, independente de que forma estiver ativa no momento, a seta
> > para cima vai levar ao retângulo azul, ou seja: todas as transições que levam ao retângulo azul
> > tem o mesmo evento gatilho na máquina de estados. assim reduzimos o número de transições no
> > painel para o número de formas envolvidas."*

### §7.1 — O que mudou, e por que é MODELO e não painel

| | W8/W9 | **W10** |
|---|---|---|
| o que se guarda | `n(n-1)` **arestas** (`from`, `to`, `when`, ritmo) | `n` **estados** (`shape`, `when`, ritmo) |
| a quem pertence a condição | à **passagem** | ao **destino** |
| linhas no painel com 9 formas | **72** | **9** |
| a mesma tecla, escrita quantas vezes | `n-1` | **1** |

⚠️ **As passagens não desapareceram — deixaram de ser GUARDADAS.** De qualquer forma continua a dar
para ir a qualquer outra: isso é consequência de haver `n` formas, e guardá-lo obrigava o artista a
escrever a mesma tecla `n-1` vezes **sem nada a impedir que as `n-1` discordassem**.

⭐ **O `start` também deixou de ser um campo** — ele é `states.first()`. Um `start: ShapeId` guardado
podia apontar para uma forma que a lista não tem, e essa discordância passa **muda** por uma fusão
(`CLAUDE.md` §5.0: *o git não sabe o que o número significa*).

⛔ **O que isto apaga, e é deliberado:** a possibilidade de a mesma tecla levar a sítios diferentes
conforme o estado. É literalmente o pedido (*"independente de que forma estiver ativa"*), e é
também a teia que a [pesquisa 31](31_pesquisa_maquinas_de_estado.md) descreve como o medo do
Animator do Unity.

⛔ **Uma lei NOVA que o modelo obriga:** *chegar onde já se está não é chegar.* A tecla da forma
corrente **não faz nada** — ela seria uma transição de uma forma para ela própria, que nem sequer é
exprimível (o `VecMorph` guardaria `(X, X)` e o `t` andaria sobre um caminho de comprimento zero) e
que o artista leria como um estremecimento sem causa.

### §7.2 — ⚠️ O formato quebra, e **só agora** isso é de graça

`VecMorphMachine` viaja como `ComponentBlob`, e um blob que não descodifica **aborta o carregamento
inteiro** (`snapshot_to_world` propaga com `?`). ⇒ mudar a forma do componente parte todo
`.ph2dproj` que contenha um.

⭐ **Medido antes de mexer:** o componente nasceu **nesta linha**, hoje, nunca foi integrado e não
existe ficheiro nenhum que o contenha (`git log main -- vec_morph_machine.rs` é vazio; não há
`.ph2dproj` no repo). *A janela em que uma mudança de modelo custa zero é exactamente esta, e ela
fecha na integração.*

### §7.3 — ⚠️ O TECTO era 9 e passou a 118, pela MESMA regra

O `9` da W8 não era sobre formas — era sobre o **relógio de pintura** de `n(n-1)` linhas a
`0,0104 ms` cada, sob a regra *«esta seção sozinha nunca custa mais do que TODO o resto do painel
junto»*. Com `n` linhas, a mesma sonda e a mesma regra (painel sem a seção: `0,726 ms`):

| formas = linhas | painel | delta da seção | % de 16,7 ms |
|---:|---:|---:|---:|
| 9 | `0,824 ms` | `0,085 ms` | 4,93 % |
| 64 | `1,128 ms` | `0,389 ms` | 6,75 % |
| **118** | **`1,457 ms`** | **`0,731 ms`** | **8,73 %** |
| 122 | `1,492 ms` | `0,765 ms` | 8,93 % |

⇒ **118**, e o antigo `9` custa hoje **um nono** do que custava. *Quem move o número que tornava
algo inalcançável tem de reconferir a nota* — e aqui o que se moveu não foi o tecto, foi o
**expoente**.

⛔ A constante `MAX_MORPH_ARROWS` **morreu**: *quantas formas* e *quantas linhas* passaram a ser o
mesmo número, e duas constantes para uma pergunta divergem.

### §7.4 — ⭐ Um gate que a mudança de modelo teria deixado verde sobre nada

`a_held_key_fires_once_and_not_every_frame` media uma **cadeia** `A --jump--> B --jump--> C`: com
`pressed` em vez de `just_pressed`, a máquina saltava a cadeia inteira num quadro.

**Essa cadeia deixou de ser exprimível** (uma tecla nomeia UMA forma) ⇒ a mutação
`just_pressed → pressed` passaria a **SOBREVIVER**: o segundo disparo é recusado por já se estar em
`B`, e nada observável muda.

⇒ *o dano mudou de forma, e a régua tem de o seguir.* Com `pressed`, uma tecla segurada **PINA** a
máquina naquela forma — toda outra transição é desfeita no quadro seguinte. O gate passou a
`a_held_key_fires_once_and_never_pins_the_machine` e mede isso: segura `jump`, carrega `dash`, e a
máquina **fica** em C.

⚠️ **É o mesmo mecanismo do achado da W9** (o `select_many` sem gate), do outro lado: ali uma
afirmação sem gate; aqui um gate cujo alvo o modelo dissolveu. *Uma mudança de modelo tem de
re-perguntar o que cada gate ainda mede.*

### §7.5 — As provas de mutação da W10 (6, todas sangraram)

| mutação | gate |
|---|---|
| a acção deixa de valer de qualquer estado | `the_same_action_reaches_the_same_shape_from_anywhere` |
| a tecla da forma corrente volta a re-disparar | `the_key_of_the_shape_you_are_already_on_does_nothing` |
| o `start` vira a **última** forma | `the_shapes_are_the_list_and_the_start_is_the_first` |
| `live_actions` promete a tecla de onde já se está | `live_actions_names_only_what_does_something_from_here` |
| a lista deixa de ter uma entrada por forma | `the_list_has_one_entry_per_shape_and_nothing_more` |
| a tecla segurada **pina** a máquina | `a_held_key_fires_once_and_never_pins_the_machine` |

### §7.6 — ⚠️ E os NOMES foram renomeados, porque passaram a mentir

O modelo deixou de ter arestas e os símbolos continuavam a dizer `arrow`. A casa tem lei sobre isto
([`feedback_stale_comment_and_dead_code_lie`](../../project-memory/feedback_stale_comment_and_dead_code_lie.md)):
*um nome que mente é um defeito* — e o próximo agente grepa por `arrow` e acha uma lista de FORMAS.

| era | é | ×  |
|---|---|---:|
| `ArrowCmd` | `MorphCmd` | 19 |
| `MorphArrowRow` | `MorphShapeRow` | 10 |
| `morph_arrow_when_option_id` | `morph_shape_key_option_id` | 11 |
| `morph_arrow_when_id` | `morph_shape_key_id` | 7 |
| `arrow_cmd_for_id` | `morph_cmd_for_id` | 8 |
| `arrow_head_row` / `arrow_when_row` | `shape_name_row` / `shape_key_row` | 4 |
| `VECTOR_MORPH_ARROWS_LABEL` | `VECTOR_MORPH_SHAPES_LABEL` | 1 |

⚠️ **Renomear os ids muda o `NodeId`** (eles são hash da string) — o `node_id_collisions` re-confere,
e nada externo os alcança. ⚠️ E cada troca correu com **`assert` de contagem**: um `replace` que não
casa é no-op **silencioso**, e o script imprime sucesso na mesma
([memória](../../project-memory/feedback_python_replace_silent_noop_after_fmt.md)).

---

## §8 — ⏳ A FILA (Enio, 2026-08-26 — **pedido para ficar em fila, NÃO implementado**)

> *"Precisamos de um botão para desfazer tudo em morph states, e precisamos de botões Show e clear
> como na seção states, para cada uma das formas envolvidas. e precisamos que sendo uma forma que
> previamente não participava do Morph states, se for arrastada na hierarquia e se tornar filha de
> um objeto Morph State, automaticamente passa a fazer parte do sistema.
> Mas coloque tudo isso na fila de implementações. Pois hoje não implementaremos"*

> **⚠️ ESTADO em 2026-08-26, depois do «Siga»:** o Enio **revisou** os três controlos (Play no
> lugar de Show · Desconectar no lugar de Clear · um **botão que abre a lista** no lugar do
> dropdown), acrescentou a **compatibilidade com o sistema States**, e mandou seguir.
>
> | | estado |
> |---|---|
> | **F1** desfazer tudo | ✅ **FEITO** (W11b) |
> | **F2** Play · Desconectar · botão-que-abre-a-lista | ✅ **FEITO** (W11b) |
> | **F3** arrastar na hierarquia faz entrar | ✅ **FEITO** (W11a) — e foi ele que decidiu o modelo |
> | **F4** compatibilidade com o sistema **States** | ✅ **FEITO** (W11c) — §8.4 tem o desenho, §9 o que a implementação achou |

### §8.1 — Os três itens

| # | O quê | Estado do substrato |
|---|---|---|
| **F1** | **Desfazer tudo** — um botão que dissolve o conjunto: o objecto pai some, as formas voltam **soltas e visíveis**, onde estavam | o inverso exacto do `morph_set::upkeep`; o `vec_entities::ungroup_entities` é o vizinho a ler (⚠️ ele recusa um *pai com geometria*, e o conjunto **tem** `VecPathRef` — não serve como está) |
| **F2** | **Show / Clear por forma**, como a seção *States* | ⭐ **O `Show` já tem motor:** `MorphMachine::travel(graph, ix)` existe desde a W1, com gate, e **não tem consumidor nenhum** (medido 26/08). Falta o botão. ⚠️ O `Clear` é o gesto que eu próprio nomeei como **ausente** num doc-comment da W10: *"tirar uma forma do conjunto, que é outro gesto e ainda não existe"* |
| **F3** | **Arrastar na Hierarquia para dentro de um Morph States ⇒ a forma entra no sistema** | a porta é `hero_intents::drain_reparent` (`render_loop/hierarchy.rs`), que já corre; falta quem reaja a ela |

### §8.2 — ⭐⭐⭐ O ACHADO: os três são UM, e o F3 decide o modelo dos outros dois

Hoje há **duas representações de «que formas estão neste conjunto»**:

1. `VecMorphMachine.graph.states` — a lista **autorada**;
2. `Children(host)` — o facto da **hierarquia**, escrito pelo `morph_set::upkeep`.

⚠️ **Elas já podem discordar** (apagar um filho deixa a lista a nomear uma forma que não existe, e o
painel mostra `#id`). O F3 torna a discordância **um gesto do artista**, e portanto obrigatória de
resolver — não dá para ter «arrastar para dentro faz entrar» com a lista a ser a fonte.

⇒ **A cura provável é a lei que o módulo 3D Modeling já paga** (`CLAUDE.md` §5.1: *«a hierarquia da
cena É o documento — o `FieldDoc` é cozido dela a cada quadro»*):

> **os FILHOS são a lista de estados; a tecla é side-metadata indexada por `ShapeId`.**

O que isso arruma de uma vez:

- **F3** passa a ser de graça — reparentar **é** entrar, sem código de reacção;
- **F2 `Clear`** vira *reparentar para fora* (e o botão é um atalho para o gesto que já existe);
- **F1** vira *reparentar todos para fora* + apagar o pai — e deixa de precisar do `ungroup_entities`;
- a discordância deixa de ser exprimível, em vez de ser reconciliada.

⚠️ **O que é preciso decidir antes de escrever uma linha:** a tecla e o ritmo passam a viver numa
**tabela por `ShapeId`** (uma forma sem entrada usa os valores de partida). Isso é uma **segunda
mudança de formato** do `VecMorphMachine`.

⛔⛔ **E ela só é de graça enquanto a linha não integrar.** Um blob que não descodifica **aborta o
carregamento inteiro** (`snapshot_to_world` propaga com `?`, ver §7.2) — hoje não existe ficheiro
nenhum com este componente, e depois da integração passa a existir. *A janela é esta.*

### §8.3 — Perguntas abertas que a implementação tem de responder

1. **O `start`.** Se os filhos são a lista, o estado inicial é o **primeiro filho** — e a ordem de
   irmãos é dado (`SiblingOrder`). Arrastar para reordenar passa a mudar onde a máquina nasce: é
   isso que se quer, ou o `start` é uma marca explícita?
2. **O `Clear` da última forma.** Um conjunto com **uma** forma ainda é um conjunto? (O `create`
   recusa abaixo de 2.) Ou o `Clear` da penúltima dissolve-o, como o `ungroup` faz?
3. **Uma forma arrastada para dentro fica OCULTA?** O `upkeep` esconde os membros; um filho novo tem
   de receber o mesmo tratamento — e sair pelo `Clear` tem de o **des**esconder, senão a forma
   volta a ser solta e invisível, que é a pior das saídas.
4. **O `Show` durante a edição.** O `travel` mexe no `VecMorph` (par + `t`), que é **pré-visualização**
   e passa pelo ledger — ⚠️ mas fora do modo `Preview` não há quem faça o tempo andar (o
   `morph_machine_drive::tick` só corre no modo). ⇒ o `Show` ou liga o modo, ou salta instantâneo.

### §8.4 — ⏳ F4: a compatibilidade com o sistema **States** (o que falta, já medido)

> Enio, 2026-08-26: *"Assegure-se que esse sistema de states em morph seja integrado e
> completamente compatível com o sistema de States previamente existente, ou seja, que eu possa
> usar o state morph nas animações criadas em States."*

⭐ **O padrão já foi construído uma vez neste repo, e tem nome:** `BoolMorph`. Em 23/08 a mesma
crate aprendeu a fazer uma transição de UI **carregar um morfo de booleana por objecto**
(`Transition::bool_morphs(t)`), e o desenho é literalmente o molde:

| peça | onde | o que fazer |
|---|---|---|
| a pose grava | `ph2d_ui_state::ObjectPose` | **um campo novo**: em que forma o conjunto está |
| a transição emite | `Transition::bool_morphs` → um irmão | *"este conjunto vai de X para Y, a `t`"* |
| a shell coze | `render_loop::ui_state_bridge` | escreve `VecMorph::sources`/`t` |

⭐ **O `ObjectPose` já tem a FAMÍLIA certa, e ela está documentada lá dentro:** `width`, `filters`,
`bool_op` — *canais que não vivem no `VecPath`, então a pose carrega-os por si*. O estado do Morph
é o **quarto membro exacto** dessa família.

⛔⛔ **O PREÇO, medido antes de começar:** o `StateSets` viaja **directamente no `ProjectFile`**
(`project_migrate.rs`), não como `ComponentBlob`. ⇒ um campo novo no `ObjectPose` é
**`PROJECT_SCHEMA` 97 → 98 + um degrau de migração**, em **três sítios** (a constante, a escada, a
tripla do gate) — e é um número que **soma entre linhas** e colide **mudo** (`CLAUDE.md` §5.0).

⚠️ Diferente do `VecMorphMachine` (§7.2), aqui **não há janela de graça**: o `ph2d-ui-state` já
shipou e já existem projectos com poses. A migração é obrigatória, não opcional.

⚠️ **A pergunta de produto que a implementação tem de responder primeiro:** gravar uma pose de UI
sobre um conjunto captura *que forma está a mostrar* — ⇒ pôr o rato num botão pode **morfá-lo**.
É isso que se quer (é o pedido), mas quem grava o `Default` tem de gravá-lo na forma certa, senão
todo hover volta à forma errada.


---

## §9 — ⭐⭐⭐ W11c: o conjunto de Morph States DENTRO de uma animação de States

> Enio, 2026-08-26: *"Assegure-se que esse sistema de states em morph seja integrado e completamente
> compatível com o sistema de States previamente existente, ou seja, que eu possa usar o state morph
> nas animações criadas em States."*
>
> E, quando eu nomeei o preço: *"não há projetos salvos. esse app está em fase inicial de
> desenvolvimento, podemos fazer o que quisermos."*

### §9.1 — A costura, e por que ela é pequena

O padrão **já existia com nome** (`BoolMorph`, 23/08). O trabalho foi segui-lo:

| peça | onde | o quê |
|---|---|---|
| a pose grava | `ObjectPose::morph_shape: Option<VecPathId>` | em que forma o conjunto está |
| a transição emite | `Transition::morph_steps(t) -> Vec<MorphStep>` | *de que forma, para que forma, a que altura* |
| a máquina publica | `Machine::morph_steps()` | o mesmo `if was` do irmão |
| a shell coze | `morph_machine_drive::apply_ui_steps` | escreve `VecMorph::sources`/`t` **pelo ledger** |
| a autoria grava | `vec_ui_state_edit::capture` | a forma que a cena **mostra** (`sources[1]`) |
| a chegada repõe | `vec_ui_state_edit::install` | põe o par em `(shape, shape)` |

⭐ **O `ObjectPose` já tinha a FAMÍLIA certa, documentada lá dentro:** `width`, `filters`,
`bool_op` — *canais que não vivem no `VecPath`, então a pose carrega-os por si*. O estado do Morph
é o **quarto membro exacto**, e a única coisa que faltava era alguém escrever a linha.

### §9.2 — As três leis que o campo herdou, e uma que ele não podia herdar

1. **É a FORMA (`VecPathId`), nunca o índice na lista.** A lista é derivada dos filhos (W11a) e muda
   quando o artista arrasta um para dentro — um índice guardado passaria a apontar para outra forma
   **sem que nada mudasse na pose**.
2. **A pose SEGURA na forma de partida até chegar** — a lei do `bool_op`: não há meio caminho entre
   duas formas *nesta lista*, e um id interpolado seria o de uma **terceira**, ou de nenhuma. Quem
   desenha o meio é o motor, pelo passo.
3. **As pontas `t = 0` / `t = 1` publicam VAZIO** — ali o desenho já é uma das duas formas.
4. ⛔ **E a que ele NÃO herdou:** no `bool_op`, `None` é *«volta à herança»* (uma decisão). Aqui é
   *«não me pronuncio»*, e o `install` **não escreve nada** — uma pose gravada antes de o objecto
   ser um conjunto não pode passar a mandá-lo para a primeira forma no dia em que ele virar um.

### §9.3 — ⚠️ O PREÇO, e a decisão do Enio que o dissolveu

As poses viajam **dentro** do `ProjectFile` (o `StateSets`), não como `ComponentBlob` ⇒ um campo
novo move o **`PROJECT_SCHEMA`**. Medido antes de começar, e nomeado a ele.

Ele respondeu que não há projectos salvos. ⇒ **`97 → 98` sem degrau de migração** — mas o bump
**fica**, e a razão é o oposto de cerimónia: postcard é posicional e não-auto-descritivo, então
**sem ele um ficheiro v97 seria lido ERRADO em silêncio**. Com ele, o `project_load` recusa em voz
alta. *O bump é o que transforma um mal-entendido silencioso numa recusa legível.*

### §9.4 — ⛔⛔ DUAS mutações SOBREVIVERAM, e as duas eram buracos reais

Eu escrevi as duas guardas e **nenhuma tinha gate**:

| mutação que sobreviveu | o dano | gate que a fecha |
|---|---|---|
| apagar `self.morph_steps = f.tr.morph_steps(t)` do `advance` | **a compatibilidade inteira ficava MORTA** e nada dizia: os gates do `Transition` provam que a crate *sabe* calcular o passo, e nenhum provava que a máquina o **entrega** | `a_running_machine_publishes_the_morph_steps` |
| apagar a guarda do `VecMorphMachine` no `install` | uma pose com forma instalada sobre um **morph autorado à mão** prendê-lo-ia num par degenerado, **matando a curva** que a timeline conduz | a 2.ª metade de `a_hand_authored_morph_records_no_shape` |

*Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada* — e a segunda é a mesma classe
do `select_many` da W9. **Três vezes na mesma linha**: escrevo a guarda certa e não a gateio.

### §9.5 — As provas (5 mutações, todas sangram)

| mutação | gate |
|---|---|
| `None` vira o id **zero** (a primeira forma de toda cena) | `a_side_without_a_shape_never_becomes_a_step` |
| a pose **salta** para o destino a meio | `the_pose_holds_the_start_shape_until_it_arrives` |
| a pose grava a forma de **onde a máquina veio** (`sources[0]`) | `a_ui_pose_records_which_shape_the_set_is_showing` |
| a máquina nunca publica passo nenhum | `a_running_machine_publishes_the_morph_steps` |
| o `install` prende um morph autorado à mão | `a_hand_authored_morph_records_no_shape` |

---

## §10 — ⭐⭐⭐ W11d: a AUDITORIA do report de 2026-08-26 (o ▶ não segurava a forma no Rec)

> Enio, 2026-08-26: *"na animação de States, o morph não consegue segurar os estados atribuidos no
> momento do Rec. Auditoria Completa. Lembrando que para animações de states eventos atribuidos para
> Morph states não devem ser necessários, pois os estados morph são mudados com play"*

A auditoria seguiu o caminho inteiro — `▶ Play` → mundo → `capture` → `Transition` → `install` /
`apply_ui_steps` → `recook` — e achou **cinco defeitos, duas afirmações falsas e dois gates que não afirmavam nada**.
Os dois primeiros defeitos, compostos, **são** o report.

### §10.1 — O que estava errado

| # | Defeito | Mecanismo | Cura |
|---|---|---|---|
| **A** | ⛔⛔ **o `▶ Play` NUNCA viajava vindo de fora do modo** | o mapa `morph_machines` é **propriedade do `tick`**, e o `tick` **esvazia-o** em todo quadro com a pré-visualização desligada. O braço do `Play` corre **depois** do `tick` no mesmo quadro ⇒ o `get_mut` encontrava o mapa **vazio**, ligava o modo e voltava. A forma só mudava ao **segundo** clique | `morph_machine_drive::play`, uma porta que **abre** a máquina (`open`) em vez de a procurar |
| **B** | ⛔⛔ **uma máquina nova discordava do canvas** | ela nascia em `graph.start()`. Mas sair do modo **não** repõe a forma (§10.2), então a cena já estava noutra ⇒ o `travel` para a forma que o artista queria era **recusado** pela regra *«chegar onde já se está não é chegar»* — sobre um «onde» que só a máquina acreditava | `MorphMachine::seeded(graph, showing)`, semeada por `VecMorph::sources[1]` |
| **C** | ⚠️ **dois escritores no MESMO campo por quadro** | numa transição, o `install` da pose escrevia `[from, from]` **e** o `apply_ui_steps` escrevia `[from, to], t`. Pior: o `write_driven` lia o valor do `install` como o **autorado** (`PreviewDrive::driven`, o ramo da *outra mão*) e perdia o de verdade | `Transition::at` publica `morph_shape: None` **exactamente** onde o `morph_steps` publica um passo. *Um campo, um escritor por instante* |
| **E** | ⚠️ **o ⊘ Desconectar deixava as poses da forma na tabela** | um estado grava a **sub-árvore** com a pose **LOCAL** de cada filho. O ⊘ tira a forma do conjunto e devolve-lhe o mundo, mas a pose antiga ficava — e o `install` do Show seguinte **reescreve-lhe o `Transform`**: a forma solta **salta para a origem do conjunto**. ⚠️ Família pré-existente (reparentar faz o mesmo), mas aqui a um clique | `morph_set::disconnect_row`, uma porta com as **duas** metades — e a 2.ª é `forget_object_in_all_states` |
| **D** | ⚠️ **a pose de um conjunto gravava geometria DERIVADA** | o `morph_live::recook` reescreve a forma do conjunto em **todo** quadro. O `install` escrevia-a para o `recook` a apagar no mesmo quadro, e o `Transition::new` pagava um `Plan::new` (**13 079×** um passo) para animar um canal que ninguém lê | `capture` não grava `geometry` quando o objecto tem `VecMorphMachine` — **a tinta fica** (o `recook` não escreve `fill`/`stroke`) |

### §10.2 — ⛔ Duas afirmações que estavam ERRADAS, e a segunda mentia ao Enio

1. O doc-módulo do `morph_machine_drive` dizia *"ao largar as máquinas a cena volta ao que o artista
   desenhou"*. **Não volta.** A lei que manda é a da `PreviewDrive::settle`: o ledger repõe o
   autorado **dentro da fotografia** enquanto o motor conduz, e no primeiro quadro em que ele **para**
   a entrada morre ⇒ a captura seguinte vê o vivo e regista **UM** passo. É o *«desfaz a corrida»*, e
   aqui significa: **sair do modo COMPROMETE a forma em que se ficou.**
   ⭐ O comportamento é o certo (o objecto fica onde o artista o pôs, desfazível num Ctrl+Z) — o que
   estava errado era a nota. E é a nota que fazia o defeito **B** parecer impossível.
2. O passo **9** do smoke repetia a mesma falsidade ao Enio (*"a forma volta a ser a primeira"*), e o
   passo **13** afirmava que o `▶` viaja com o modo desligado — que é **exactamente** o defeito **A**.
   *Um smoke que descreve o que devia acontecer, e não o que acontece, aprova o defeito.*

### §10.3 — ⛔ E um gate meu era VÁCUO

`the_same_shape_on_both_sides_is_not_a_step` passava duas poses **idênticas**, que o
`Transition::new` descarta antes de existir um `Step` — o balde ficava vazio e o `is_empty()` lia
como *«a lei funciona»*. *Um zero de «não medido» e um de «correcto» são o mesmo byte.*
Hoje as pontas diferem na posição e o gate afirma `tr.len() == 1` antes de medir.

### §10.4 — Sobre *"eventos não devem ser necessários"*

Conferido, e era verdade em toda parte **menos** no `▶`: o `graph_of` deriva um estado por filho
independentemente de tecla, o `travel` é a porta **sem condição**, e nem o `capture`, nem o
`install`, nem o `morph_steps` olham para uma acção. O único sítio que lia teclas era o `live_actions`
do `tick`. ⇒ com o defeito **A** curado, um conjunto **sem uma única tecla atribuída** é
integralmente conduzível — que é o que uma animação de *States* precisa. As duas fixturas novas
não atribuem tecla nenhuma, de propósito.

### §10.5 — Os gates e as mutações

| Gate | Onde |
|---|---|
| `the_play_button_travels_on_the_very_frame_the_mode_turns_on` | `morph_machine_drive_tests.rs` |
| `a_machine_born_after_the_mode_reopens_agrees_with_the_canvas` | `morph_machine_drive_tests.rs` |
| `the_pose_and_the_step_never_speak_at_the_same_instant` | `ph2d-ui-state/morph_step_tests.rs` |
| `play_records_and_the_ui_transition_morphs_the_set` | `morph_set_ui_state_tests.rs` — **o quadro inteiro** |
| `a_set_pose_carries_no_geometry_and_costs_no_plan` | `morph_set_ui_state_tests.rs` |
| `disconnecting_a_shape_takes_it_out_of_the_recorded_states` | `morph_set_ui_state_tests.rs` |

⚠️ **O quarto é o que importa metodologicamente.** Os três gates da W11c mediam `capture` e
`install` — as duas metades **certas** — e o defeito vivia na **composição**, no braço do despacho.
*Um gate de unidade é cego à fiação da shell*, e esta linha já tinha pago a mesma lição na W4 (doze
gates verdes sobre uma secção que ninguém pintava).

**Seis mutações, todas sangram**, cada uma no gate que a nomeia:

| Mutação | Sangra em |
|---|---|
| o `play` volta ao `get_mut` | os **três** gates de composição |
| o `open` semeia com `MorphMachine::new` | `a_machine_born_after_the_mode_reopens_agrees_with_the_canvas` |
| a pose volta a segurar `from` no meio | `the_pose_and_the_step_never_speak_at_the_same_instant` |
| o `capture` grava a geometria derivada | `a_set_pose_carries_no_geometry_and_costs_no_plan` |
| o `disconnect_row` não esquece as poses | `disconnecting_a_shape_takes_it_out_of_the_recorded_states` |
| o `forget_object_in_all_states` varre a lista toda | idem (o controle das outras três poses) |

⚠️ **E a 1.ª redacção do gate do ⊘ chamava as duas metades À MÃO** — provava que elas funcionam, e
**não** que o botão as chama, que é o defeito. Foi o que forçou a porta `disconnect_row`: *duas
linhas num braço de `match` do laço de render não são alcançáveis de um teste*, e foi assim que a 2.ª
metade ficou por escrever uma wave inteira. Quarta ocorrência do padrão nesta linha
(`project-memory/feedback_i_write_the_right_guard_and_do_not_gate_it.md`).

---

## §11 — ⭐⭐⭐ W11e: o 2.º report (26/08) — **dois motores, e a ordem por quadro não bastava**

> Enio, 2026-08-26: *"em states Default gravei Morph States em wide, em hover gravei Morph states em
> tall. Ao ligar o preview Default não segurou wide e está em tall. No hover há uma transição
> tall - wide - tall. Ao sair de hover o mesmo acontece: tall - wide - tall."*

### §11.1 — O mecanismo, e por que a W11c já o tinha «resolvido»

A W11c pôs o `apply_ui_steps` **depois** do `tick` no quadro, e escreveu que *"se as duas coisas
escrevem o mesmo objecto, quem manda é a transição de UI"*. Isso é verdade **só nos instantes em que
a transição fala** — e o `morph_steps` **cala-se nas pontas**, de propósito (§9). Logo:

| O que o Enio viu | Quem escreveu |
|---|---|
| *"Default não segurou wide e está em tall"* | o `ui_preview::enter` **instala** o `Default` (wide); o `tick` do quadro seguinte repõe **tall**, onde o ▶ deixou a máquina |
| *"no hover: tall - wide - tall"* | o hover morfa `wide -> tall` correctamente — mas o quadro **anterior** mostrava tall, então vê-se um salto para wide antes do morfo |
| *"ao sair de hover: tall - wide - tall"* | a saída morfa `tall -> wide`, **chega**, o `apply_ui_steps` cala-se, e o `tick` repõe **tall** |

⚠️ **Os três são o mesmo facto**: nos instantes de repouso e de chegada, quem escrevia o `VecMorph`
era a máquina de teclas.

### §11.2 — A lei

⇒ **o sistema de States tem PRECEDÊNCIA, e a máquina de teclas LARGA enquanto ele age**
(`morph_machine_drive::drives(morph_preview, ui_state_live)`). O `ui_state_live` é verdade com o modo
de pré-visualização ligado **ou** com alguma transição no ar — as duas situações em que a forma é
função do estado de UI, não da tecla.

⭐ **Largar (e não «não escrever») é o que faz a volta ser suave:** o `!active` do `tick` **apaga** as
máquinas, e a seguinte nasce **semeada pelo mundo** — ou seja onde os States a deixaram. *A cura da
W11d (`open`/`seeded`) é o que torna esta possível*; sem ela, sair da pré-visualização daria um salto.

⚠️ **Uma função, e não um `&&` no braço do despacho** — quinta vez nesta linha que uma lei escrita
dentro do laço de render fica fora do alcance de todo gate.

### §11.3 — ⛔⛔ E a W11d foi REVERTIDA num ponto: a pose **não** se cala a meio do voo

A W11d fez `Transition::at` devolver `morph_shape: None` no meio, para o `install` e o
`apply_ui_steps` não escreverem o mesmo campo. **Duas coisas estavam erradas nisso:**

1. **`None` já tem dono como significado** — *«esta pose não se pronuncia»* (a pose gravada antes de
   o objecto ser um conjunto). Um segundo sentido (*«estou a meio»*) põe dois factos no mesmo valor.
2. **A escrita dupla não é um defeito, é uma CAMADA:** a pose escreve a forma de **base** e o passo
   **refina-a** com o `t`. Calada a base, o mundo fica com o valor do quadro ANTERIOR sempre que o
   passo não fala — e ele não fala nas pontas, nem quando uma delas é `None`, que é exactamente o
   estado em que uma **interrupção** deixa a máquina (`Machine::go_to` faz
   `Transition::new(&self.live, ..)` — *a pose viva é o `from` da próxima transição*).

⛔ E o custo que a W11d queria evitar é **inócuo, medido**: o ledger não é lido durante uma transição
porque o `ui_state_live` **suprime a fotografia**.

⚠️ **Consequência NOMEADA (não curada):** interromper um voo a meio faz a forma **saltar** para a de
partida em vez de desmorfar — o par vivo `(A, B, t)` não cabe numa pose, que carrega **uma** forma.
Curá-lo é modelo novo, não um ajuste no `at`.

### §11.4 — Gates e mutações

| Gate | Onde |
|---|---|
| `the_key_machine_lets_go_while_the_ui_states_act` | `morph_set_ui_state_tests.rs` — **o report, reproduzido** |
| `the_pose_names_a_shape_at_every_instant_of_a_flight` | `ph2d-ui-state/morph_step_tests.rs` |
| `the_arrow_click_reaches_the_world` (censo da costura) | passa a exigir o `drives` na chamada do `tick` |

**Duas mutações, ambas sangram:** o `drives` devolver `morph_preview` (ignorando o `ui_state_live`),
e o `at` voltar a calar-se no meio.

⚠️ **O censo da costura apanhou a mudança sozinho** — ele exigia a agulha antiga na chamada do
`tick`, e ficou vermelho no momento em que a fiação mudou. *É o único gate desta linha que olha para
a shell, e foi o único que reagiu.*

---

## §12 — ⭐⭐⭐ W11f: a PERTENÇA — o que a W11 entregou pela metade

> Enio, 2026-08-26, depois do smoke OK: *"Veja o que falta na fila de implementações"*.

A conferência da fila **mediu** o F3 (*arrastar na hierarquia faz entrar*) e o F2 (*Desconectar*), e
achou **três** coisas que o §8.3 tinha deixado como pergunta aberta e ninguém respondeu.

### §12.1 — ⛔⛔ A ocultação não acompanhava o arrasto — nos DOIS sentidos

A W11 fez a **lista** de estados ser os filhos, mas a **ocultação** continuou a ser uma escrita do
`upkeep` (um `Visibility::hidden()` guardado no momento da criação). Medido, com a sonda a reproduzir
o gesto que a Hierarquia faz (`ChildOf` e mais nada):

| gesto | a lista | o canvas (antes) |
|---|---|---|
| arrastar para **DENTRO** | entra (3 → 4 estados) | ⛔ **continua visível**, desenhada por cima do conjunto |
| arrastar para **FORA** | sai (4 → 3 estados) | ⛔ **continua escondida** — a forma **desaparece** |

⚠️ A segunda é a pior, e o doc do `disconnect` **já a nomeava** como *"a pior saída possível"* — para
o botão ⊘. O gesto de arrasto chegava lá pela porta que ninguém tinha olhado.
⚠️ E o **passo 14 do smoke afirmava** que a de dentro sumia do canvas. *Um passo de smoke que
descreve o que devia acontecer aprova o defeito.*

**A cura:** `morph_set::is_set_member` — *o meu pai tem máquina, logo eu sou um estado, logo eu não
me desenho* —, consultada pelo `vec_entities::visible_chain`, que é a **porta única** que o canvas lê
(um chamador, medido). A ocultação passa a ser **derivada**, como a lista já era.

⭐ Com isso os dois gestos ficam **de graça e simétricos**, e não há estado guardado que possa
discordar da árvore. ⛔ **O olho do artista sobrevive:** a derivação só ACRESCENTA uma razão para
esconder, e o `disconnect` deixou de fazer `remove::<Visibility>()` — que **destruía** a escolha de
quem tivesse escondido a forma antes de ela entrar.

### §12.2 — ⛔ Um conjunto esvaziado desenhava um FANTASMA

Medido: com o ⊘ a tirar as formas uma a uma, o conjunto ficava com **zero** estados e mantinha o
`VecMorph` que o `upkeep` lhe deu — o `sources` continuava a nomear a **primeira forma**, que já
tinha saído. ⇒ o artista desconecta as três e fica com um objecto que **desenha uma cópia da
primeira**, sem saber o que é nem como o apagar.

⇒ **a fronteira é a do `create`** (`MIN_STATES = 2`, agora com **dois leitores** e uma constante só):
sair dela **dissolve**, exactamente como o `ungroup` faz com o último filho. *Um objecto deixa de ser
uma relação quando fica com um lado só.* É a resposta à pergunta 2 do §8.3.

⚠️ **A contagem lê-se ANTES de desconectar** — depois a lista já encolheu e a fronteira leria-se ao
contrário (há mutação a prová-lo).
⭐ E os dois verbos que apagam o conjunto (`Dissolve` sempre; `Disconnect` na fronteira) passam a
sair pela **mesma porta** no despacho: cada um a remover o path por si seriam duas respostas a *"o
que é apagar um conjunto"*.

### §12.3 — ⚠️ E TRÊS gates mediam o componente, não o que o canvas vê

Eles liam `Visibility` **directamente**, e por isso nenhum via o defeito. A sonda mudou-se para
`view_state().hidden` — a resposta que o canvas de facto consome. *Contar o trabalho FEITO não é
contar o trabalho ENTREGUE.*

### §12.4 — Gates e mutações

| Gate | Onde |
|---|---|
| `dragging_into_the_set_hides_and_dragging_out_shows` | `morph_set_membership_tests.rs` |
| `the_artists_own_eye_survives_joining_and_leaving_the_set` | idem |
| `disconnecting_the_last_but_one_dissolves_the_set` | idem |

**Cinco mutações, todas sangram:** `is_set_member` sempre `false` · `is_set_member` a ignorar o
`ChildOf` · o `disconnect` a voltar a remover o `Visibility` · o `disconnect_row` a ignorar a
fronteira · a fronteira lida **depois** de desconectar.

---

## §13 — ⭐⭐⭐ W11g: o RESQUÍCIO do ⊘ (3.º report) — *a lista é derivada, o par desenhado é guardado*

> Enio, 2026-08-26: *"desconectar muda correctamente na hierarquia e painel, mas deixa a imagem de
> resquício no canvas e o nome de resquício no painel"*

### §13.1 — Um mecanismo, dois resquícios

MEDIDO com sonda: um conjunto a mostrar a forma `0`, o artista carrega no ⊘ **dessa** forma, e o
`VecMorph::sources` fica em **`[0, 0]`** com a lista já em `[1, 2, 3]`.

| o que o Enio viu | quem o produz |
|---|---|
| *"imagem de resquício no canvas"* | o `morph_live::recook` continua a cozer a forma que saiu ⇒ ela aparece **duas vezes**: solta no sítio dela, e **clonada** dentro do conjunto |
| *"nome de resquício no painel"* | o `vec_morph_edit::publish` lê `sources[1]` para o readout ⇒ o painel **nomeia** a forma que já não é estado |

⚠️ **É a MESMA família da W11f, um valor depois.** A W11 tornou a **lista** derivada e deixou dois
valores **guardados** sem quem os acompanhasse: a **visibilidade** (curada ontem) e o **par
desenhado** (hoje). ⛔ Um terceiro candidato fica coberto pela mesma varredura: uma forma **apagada**
também sai dos `Children`.

### §13.2 — A lei

⇒ `morph_machine_drive::reconcile`, **todo quadro e FORA do modo** (o ⊘ corre com a
pré-visualização desligada): se um lado do par não é membro, o par **colapsa** num que seja; se
nenhum, no primeiro estado.

⭐ **E a máquina viva é LARGADA junto**, em vez de corrigida — ela renasce **semeada pelo mundo**
que a varredura acabou de arrumar (a cura da W11d). ⚠️ Escrita **directa**, não pelo ledger: é a
consequência documental de um gesto do artista, e o `post_frame_undo` regista-a **junto** com ele.

### §13.3 — ⛔⛔ Duas mutações SOBREVIVERAM, e são de espécies opostas

1. **Um buraco real:** apagar o `machines.remove(&bits)` deixava a suíte inteira verde — **nenhum
   gate corria o `tick` DEPOIS da varredura**, e sem ele o resquício **volta no quadro seguinte**,
   só dentro do modo de pré-visualização (que é onde o ▶ acabou de pôr o artista). ⇒
   `the_ghost_does_not_come_back_on_the_next_tick`.
2. **Uma afirmação sobre NADA:** eu documentei que *"o destino tem precedência sobre a origem"*.
   A guarda anterior já sai cedo quando os dois são membros ⇒ **no máximo um** passa o `find`, e
   trocar a ordem **não muda uma única resposta**. A cura foi **apagar a afirmação**, não inventar
   um gate para ela.

⚠️ **É a 5.ª ocorrência do padrão nesta linha**, e a medição refinou-o: o dano vive **um passo à
frente** do que o gate da feature olha — noutro subsistema, ou **no quadro seguinte**. Registado em
`project-memory/feedback_i_write_the_right_guard_and_do_not_gate_it.md`.

### §13.4 — Gates

| Gate | O que afirma |
|---|---|
| `disconnecting_the_shown_shape_leaves_no_ghost` | depois do ⊘ o par nomeia **membros**, e colapsa numa forma só |
| `the_reconcile_keeps_what_the_canvas_shows` | tirar uma forma **invisível** não muda o que se vê |
| `the_ghost_does_not_come_back_on_the_next_tick` | a máquina viva não repõe a forma que saiu |
| `the_arrow_click_reaches_the_world` (censo) | passa a exigir a chamada do `reconcile` na shell |

⚠️ A agulha do censo é o **nome da porta** e não a chamada inteira: o `cargo fmt` quebra-a em cinco
linhas assim que ela passa da largura, e uma agulha multi-linha fica refém da formatação em vez de
medir a fiação.

**Três mutações, todas sangram.**

---

## §14 — ⭐⭐⭐ W11h: o ⊘ QUEBRAVA a animação de States (4.º report)

> Enio, 2026-08-26: *"com um morph states com 3 shapes dentro de uma animação de States,
> desconectei uma shape do morph state e quebrou a animação do state. (…) se o usuário desconectar
> uma shape, coloque outra shape do conjunto em seu lugar de modo a não quebrar as anims."*

### §14.1 — O que quebrava, medido

A pose do hospedeiro guarda **qual forma o conjunto mostra** (`morph_shape`). Sonda com
`Default = forma 0` e `Hover = forma 1`, tirando a `0`:

```
antes:  Default=Some(0)  Hover=Some(1)
depois: Default=Some(0)  Hover=Some(1)   (membros agora: [1, 2])
morph_steps(0.5) = [MorphStep { from: 0, to: 1, t: 0.5 }]
```

⇒ o motor a **cozer a partir de uma forma que saiu do conjunto** — e cujo `Transform` já é de
MUNDO, não do referencial dele (o `recook` de um conjunto lê as poses **locais** dos filhos). O
morfo saía de um sítio que não era o dela nem o do conjunto.

⚠️ **A W11g arrumou o MUNDO** (o par desenhado) e **não podia** arrumar isto: a pose é dado
**autorado**, e reescrevê-lo só é legítimo dentro de um gesto explícito do artista.

### §14.2 — A cura tem DUAS metades, e as duas são precisas

1. **O GESTO substitui** — `vec_ui_state_table::replace_morph_shape_in_all_states`, chamada pelo
   `disconnect_row`. É o pedido do Enio, palavra por palavra.
   ⭐ **A substituta é uma que NENHUM outro estado nomeia**: pôr a do `Hover` no `Default` deixaria
   os dois na mesma forma, e a animação **sobreviveria ao ficheiro para morrer na tela** — o defeito
   com outro nome. Com três formas há sempre uma livre.
   ⚠️ **Uma substituição por FORMA, não por estado:** dois estados que nomeavam a mesma forma
   continuam a nomear a mesma; escolher por estado partiria uma igualdade autorada.
2. **O CONSUMIDOR blinda** — o `apply_ui_steps` **ignora** um passo cuja ponta não é estado. ⛔ O ⊘
   não é a única rota: **arrastar um membro para FORA na Hierarquia** tira-o do conjunto sem passar
   por lá. *Não morfar é uma resposta; morfar a partir de um estranho não é.*

⏳ **NOMEADO, não curado:** a rota do arrasto **não substitui** — ela degrada para *«aquele estado
não morfa»*. Substituir ali obrigaria a reescrever dado autorado **sem gesto**, numa varredura por
quadro que o `ProjectState` capturaria (o `StateSets` viaja nele) ⇒ passos de undo espúrios. É
decisão de produto.

### §14.3 — ⚠️ E um achado ao cortar: um doc-comment tinha ENGOLIDO a função vizinha

A inserção da W11d pôs o `forget_object_in_all_states` **entre o doc-comment e a função** do
`shift_host_in_all_states` — que ficou sem doc nenhum, com o dela a documentar a outra. (E acima
havia um **órfão pré-existente**: o doc do `publish`, que se mudou para o `vec_ui_state_host.rs` e
deixou o texto para trás.)

⇒ as três operações de tabela saíram para `vec_ui_state_table.rs`, com o corte por assunto: *o
objecto moveu-se · o objecto saiu · a forma que uma pose nomeia deixou de ser um estado*.

### §14.4 — Gates e mutações

| Gate | Onde |
|---|---|
| `disconnecting_a_shape_does_not_break_the_states_animation` | `morph_set_states_repair_tests.rs` |
| `a_step_naming_a_non_member_is_ignored` | idem |

**Quatro mutações, todas sangram:** o `disconnect_row` não substituir · a escolha ignorar o que já
está em uso · a substituição tocar poses que não nomeavam a forma que saiu · o `apply_ui_steps`
largar a checagem de pertença.
