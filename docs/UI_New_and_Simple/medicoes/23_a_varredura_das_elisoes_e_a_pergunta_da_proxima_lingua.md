# 23 — A varredura das elisões, e a pergunta da próxima língua

**2026-09-18 · `line/UIUX`**

O censo de elisões nasceu ontem ligado à lei do CORTE. Ele via metade da pergunta: **o rótulo que
cabe hoje nunca passa pela lei do corte**, logo não deixava rasto — e é exactamente sobre ele que se
faz a pergunta que interessa a seguir, *«e quando alguém traduzir?»*.

---

## 1. O censo passa a ouvir a pergunta «cabe?»

`ph2d_editor_core::text_elide`:

| antes | agora |
|---|---|
| `Cortado { texto, pintado, largura }` | `Medido { texto, pintado, largura, **fonte**, **peso** }` |
| registado só em `elide` (o corte) | **mais** em `coube()` (o «sim») |
| `cortados()` | `medidos()` + `cortados()` (vista filtrada) |

⭐ **A fonte e o peso são a razão de isto responder pelas duas línguas.** O idioma de teste é uma
**função pura** do inglês (`ph2d_i18n::pseudo::deforma`), logo uma pintura em inglês basta: o gate
re-mede a palavra deformada **no mesmo orçamento, na mesma fonte e no mesmo peso**, sem tocar no
`PH2D_LANG` do processo — que faria da suíte mais um membro da família de flakes de fan-out.

⚠️ *Medir numa espessura e pintar noutra* é o defeito que este ficheiro já pagou duas vezes; sem os
dois campos a previsão seria um palpite.

### 1.1 — A pergunta é uma PORTA, e não uma comparação repetida

Havia **dois** pintores a comparar à mão (`fit_weighted` e `paint_elided_weighted`). Hoje os dois
chamam `text_elide::coube(..)`, que é o único sítio onde o censo ouve o «sim».

⛔⛔ **E isto foi forçado por uma mutação SOBREVIVENTE:** pôr o segundo pintor a comparar à mão
deixa a varredura do app inteira **VERDE**, porque o piso de população dela é GLOBAL — *um piso
sobre a soma é cego a uma parcela*. ⇒ o gate novo `os_dois_pintores_perguntam_pela_mesma_porta`
pergunta **à porta**: cada pintor, um rótulo que cabe, um registo.

---

## 2. ⛔⛔ A bandeira do censo era GLOBAL e o armazém por THREAD

O `ARMADO` era um `AtomicBool` estático e os registos um `thread_local`. Sob **`cargo test`** — que
corre os testes em threads do mesmo processo — o `desarma` de um gate apanhava o vizinho entre o
`arma` dele e a pintura, e o vizinho lia **zero cortes** sobre um corte que aconteceu.

⚠️⚠️ **O `nextest` não a podia mostrar:** ele dá um PROCESSO a cada teste, e ali não há vizinho
nenhum — e é com ele que os portões deste repo correm.

> *Um instrumento mede-se na ferramenta MAIS FRACA que o corre*, senão ele é correcto só no sítio
> onde ninguém olha.

Cura: a bandeira é `thread_local`. Régua: duas threads, uma a medir 2 000 vezes e a outra a armar e
desarmar em ciclo — com a bandeira global, a primeira perde registos.

---

## 3. A varredura: 28 painéis, 3 viewports, 4 022 rótulos

`ph2d-panel-registry-init/tests/it/nenhum_rotulo_do_app_pinta_nada.rs` pinta **cada painel do
REGISTO** (a população, nunca uma lista escrita à mão — `ErasedPanel` carrega o estado dele) pela
porta nova do arnês `MockPanelHost::medindo_a_pintura_do_registo`.

| grandeza | `-p` sozinho (24 painéis) | árvore inteira (28) |
|---|---:|---:|
| rótulos medidos | 3 146 | **4 022** |
| a pintar **NADA** em inglês | **0** | **0** |
| a pintar **NADA** no idioma de teste | **0** | **0** |
| cortados (`prefixo…`) em inglês | 15 | **17** |
| cortados no idioma de teste | ~130, em 16 painéis | — |

⭐ **As duas leis a ZERO são a catraca mais apertada que existe:** já não há linha onde escrever um
rótulo mudo. A prova de que ela morde é o defeito de ontem: repondo o respiro constante, a varredura
acusa os quatro botões do mixer.

### 3.1 — ⛔⛔⛔ A população depende das FEATURES, e isso apanhou o gate no dia em que nasceu

Quatro painéis (`flip`, `flip_frames`, `painter_layers`, `wet_tuning`) **não** estão no `default`
da crate do registo — eles chegam pelo shell. A lista de dívida capturada com `-p` sozinho estava
**incompleta por construção**, e o `flip_frames` trouxe dois cortes (`"Linear"`, `"No Cycle"`) que
só apareceram na corrida da árvore inteira.

⚠️ *Uma crate testada sozinha é testada num mundo que o produto não habita* (`CLAUDE.md` §2) — aqui
isso lê-se como uma catraca completa.

⇒ duas consequências no gate: o piso de painéis conta-se **no registo** (um painel que não pinte um
rótulo elidível não aparece nos achados), e o censo de obsolescência **salta** as linhas cujo painel
esta build não liga — *uma linha invisível não é uma linha obsoleta*.

### 3.2 — ⚠️ O que NÃO se traduz também não se deforma

A 1.ª redacção da previsão acusou `"3"` numa caixa de `9,0 px` da timeline. O censo regista o que
foi **PINTADO** e não sabe de onde a string veio ⇒ a previsão precisa da lei que separa: **uma
palavra tem LETRAS**. É a mesma cerca que o corpus do idioma de teste declara (as 89 832 traduções
medidas excluem o que não tem letra).

---

## 4. A dívida que a varredura achou (17 linhas, nomeadas)

Três famílias:

- ⛔⛔ **VALORES cortados** — `+0.00 EV`, `0.500`, `1.500`. *Um número que se lê `0.…` mente sobre si
  próprio.* A caixa de número gasta `rect − stepper − respiro` e o que sobra é **menos de metade**
  dela (`56 → 24 px`, medido). É a família de ontem um andar acima.
- ⛔ **Opções e rótulos** que não cabem no chip deles — `PingPong`, `Line / Neighbors`, `Linear`,
  `No Cycle`, `Float`, `Color`, `filter`, `Geometry Offset`.
- ⚠️ **Frases de estado vazio** elididas a UMA linha (Inspector, Tags, e as legendas dos dois
  painéis de laboratório). A cura provável não é largura: é elas **quebrarem** (`Lines::Wrap` já
  existe), que é decisão de desenho.

E `Master` no mixer, que é a troca já declarada em 18/09.

---

## 5. Provas de mutação — 6 de 6 sangram

| # | mutação | gate que acusa |
|---|---|---|
| M1 | o respiro volta a ser subtracção constante | `nenhum_rotulo_do_app_pinta_nada` |
| M2 | o «sim» deixa de ser registado | o piso de população |
| M3 | o pintor cortado contorna a porta | `os_dois_pintores_perguntam_pela_mesma_porta` |
| M4 | a bandeira volta a ser global | `o_censo_de_uma_thread_nao_e_desarmado_pela_vizinha` |
| M5 | uma linha obsoleta na dívida | `nenhuma_linha_da_divida_ficou_obsoleta` |
| M6 | um corte novo sem nome | `nenhum_corte_novo_entra_sem_ser_nomeado` |

⚠️ **M3 SOBREVIVEU na primeira corrida** e foi isso que produziu o gate ao nível da porta.

⚠️ E o arnês da mutação **conta quantos testes de facto correram**: um filtro que casa zero imprime
`ok` e lê-se como «sobreviveu» — a lição que este repo já pagou quatro vezes, e que mordeu outra vez
aqui (a mutação M4 na 1.ª forma não compilava).

---

## 6. Portão

`nextest-impacted` **19 706** testes, **19 705** verdes. O único ✗ é
`the_cost_of_a_player_is_linear_in_their_number`, membro **confirmado** da família de flakes de
fan-out do §5.0 — **3 de 3 verde sozinho a `load 72,89`**, que é quatro vezes a carga em que
reprovou, e **zero linhas** do diff desta linha naquela crate.

`cargo fmt --all --check` limpo · `clippy -D warnings` a zero nas três crates tocadas, nas duas
configurações de features.

---

## 7. A primeira cura que a varredura pagou: a caixa de número

Dos 17 cortes, **três eram VALORES** — `+0.00 EV`, `0.500`, `1.500`. *Um número que se lê `0.…`
mente sobre si próprio*, e a conta era esta:

| grandeza | valor |
|---|---:|
| caixa do chip (Design Tokens) | `56,0 px` |
| coluna do stepper (`(h·0,6).clamp(16,22)`) | `16,0` |
| área de texto | `40,0` |
| **orçamento que ela pedia** (`label_budget`) | **`24,0`** |
| `0.500` na fonte do chip | `31,0` (`28,2` com as fontes do sistema) |

⛔ **A área de texto já está recuada da borda direita pela coluna INTEIRA do stepper**, e pedir por
cima dela o orçamento de um RÓTULO conta essa borda **duas vezes**: `40 − 8 − 8 = 24`.

⇒ `paint_text_centered_com_orcamento` (a irmã com o orçamento dado), e a caixa de número conta
**uma** borda: `24,0 → 32,0`. Com o texto centrado sobram `Md/2 = Xs` de cada lado — o recuo que
esta caixa **já usa** na vertical para a selecção.

⚠️ **O que NÃO muda:** a caixa continua do mesmo tamanho, o stepper continua na coluna dele, nada
se move de sítio, e um valor genuinamente longo (`-141.881`, `42,6 px`) continua **elidido** — *a
reticência é mais honesta que um recorte a meio de um dígito, que se lê como outro número*.

⛔ **A fronteira que fica, com o número:** um valor **negativo** de três decimais (`-0.500`,
`36,1 px`) ainda não cabe em `56 px`. A alavanca ali não é o respiro — é a **largura da caixa**,
que é do painel que a escolhe. Está num gate que reprova no dia em que alguém a alargar.

⭐⭐ **E quem apagou as três linhas da dívida foi o próprio gate:** o censo de obsolescência acusou-as
como já não descrevendo corte nenhum. A catraca desceu **17 → 14** sem ninguém a editar à mão.

### 7.1 — Duas mutações a mais, uma delas sobrevivente

| # | mutação | gate |
|---|---|---|
| M7 | a caixa volta a pagar o respiro de um rótulo | `a_caixa_de_numero_*` (4 testes) |
| M8 | o orçamento passa a ser a área INTEIRA | `o_respiro_da_caixa_conta_uma_borda` |

⚠️ **M8 SOBREVIVEU** à primeira redacção, que só tinha a metade de baixo (*«não paga o respiro de
um rótulo»*): um orçamento maior passa-a sempre. ⇒ a régua ganhou a metade de cima (*«ainda há
respiro»*), e as **duas** grandezas lêem-se do produto — uma pintura centrada que elide deixa dois
registos no censo, o orçamento e a área.

### 7.2 — ⛔ Tecto de LOC, curado por CORTE

O `paint.rs` foi a `712` contra `700`. Cura: o **respiro de um rótulo e a centragem que o gasta**
saíram para `paint_label_box.rs` (`610` + `119`) — as quatro funções são uma lei só, com ida
(`rect_for_label`), volta (`label_budget`) e os dois pintores que a gastam. ⛔ Nunca uma entrada
nova no `FILE_OVERAGE_OK`.

**Portão final: `19 710` impactados, `19 710` verdes.**

---

## 8. A segunda cura: as duas frases de estado vazio QUEBRAM

Das 14 que sobravam, duas são **frases do produto** — e a primeira delas é literalmente a primeira
coisa que se lê ao abrir o app:

| painel | o que saía | coluna |
|---|---|---:|
| Inspector (nada escolhido) | `Select an entity in the Hierarchy to inspec…` | `252 px` |
| Tags (sem tags) | `No tags yet. Press + New to make the fi…` | `240 px` |

⚠️ **A causa nasceu de uma CURA:** o `paint_text` elide para uma linha desde 2026-09-06 (report do
dono: *«a palavra que não cabe passa para baixo e some»*). Aquilo estava certo para um **rótulo de
linha** — e estas duas não são rótulos, são **frases**. ⇒ hoje passam pelo `paint_text_block`, que é
o pintor que existe PARA quebrar e que **devolve a altura**.

⭐ **A linha do painel de Tags cresce com a frase**, porque a função devolve o avanço a quem empilha
por baixo — a lei que o `paint_text_block` escreve no doc dele.

⛔⛔ **E a régua NÃO pode ser o censo de elisões:** um texto que quebra não passa pela lei da
reticência, logo **não deixa registo** — e um painel que deixasse de pintar a frase ficaria
igualmente mudo. *Zero lê-se como aprovação.* ⇒ a régua é a **TINTA** (os glifos que a cena
recebeu), com o controlo do lado oposto: o pintor de rótulo **continua** a elidir a mesma frase.

### 8.1 — ⛔⛔⛔ O furo que a primeira cura abriu no meu próprio gate

O censo de obsolescência saltava as linhas cujo painel *«esta build não liga»*, e eu derivei essa
população **do que a pintura produziu**. O Inspector mede **UM** rótulo — a frase. Ao fazê-la
quebrar, ele deixou de registar seja o que for ⇒ **saiu da população**, e a linha de dívida dele
passou a ser saltada para sempre, **em silêncio**.

> *Uma catraca cuja população encolhe com a própria cura vira licença.*

⇒ os presentes lêem-se do **REGISTO**. É a mesma forma que o piso do `every_host_that_rewrites_verts`
já tinha pago (*o piso segurou o número enquanto a população trocava por baixo dele*).

### 8.2 — Duas mutações minhas testavam a coisa errada

| # | mutação | resultado |
|---|---|---|
| M9 | anular a altura medida no Inspector | ✗ **sobreviveu** — isso quebra a CENTRAGEM, não a quebra de linha |
| M10 | o avanço do Tags volta a `ROW_H_PX` | ✗ **sobreviveu** — isso é SOBREPOSIÇÃO, e o censo vê cortes |
| M9-bis / M10-bis | trocar o pintor de volta (`paint_text`) | ✓ as duas sangram |
| M11 | o avanço do Tags volta a `ROW_H_PX` | ✓ sangra **no gate novo** que as duas anteriores obrigaram a escrever |

⭐ O que as duas primeiras produziram foi o gate que faltava: **a linha do vazio cresce com a frase,
e só quando ela quebra** — com a metade de baixo (numa coluna larga o avanço é o de sempre) a
impedir a cura barata de abrir um buraco no painel.

⚠️ E a 1.ª redacção do avanço somava o recuo de cima **outra vez em baixo** e devolvia `24,5` para
UMA linha (o `alta` de uma linha é a **altura de linha** do parley, maior que o corpo da fonte). Quem
o apanhou foi esse gate, na primeira corrida.

**Dívida: 17 → 14 → 12. Portão: `19 713` impactados, `19 713` verdes.**

---

## 9. A terceira cura: o chip de escolha derivа da LISTA que ele oferece

A barra da tira do Flip reservava o literal `CYCLE_W = 84 px` para os dois chips de escolha
(o ciclo e a curva do tween). A conta:

| grandeza | valor |
|---|---:|
| chip | `84,0 px` |
| recuo × 2 (`Spacing::Lg`) | `24,0` |
| vão até ao chevron (`Spacing::Md`) | `8,0` |
| chevron (`(h·0,6).clamp(14,20)`) | `14,0` |
| **sobra para o rótulo** | **`38,0`** |
| `No Cycle` | `56,4` ⇒ saía `No…` |

⭐⭐⭐ **E o censo só via METADE do defeito.** Ele mede o que está **PINTADO**, que é a opção
ESCOLHIDA — `Ping-Pong` (`65,5`) e `Ease In-Out` (`72,9`) vivem nas mesmas listas, nunca são o valor
de fábrica, e **nunca tinham sido medidos por ninguém**.

> *Quem dimensiona um chip de escolha mede a LISTA, nunca o item em mãos.*

⇒ `chip_w(ts, row_h, nomes)` = a maior opção + o invólucro, com o invólucro a vir da porta que o
**pintor** usa: `dropdown_chip_width_for` ↔ `dropdown_label_budget`, o par ida-e-volta que o
`rect_for_label`/`label_budget` já é para uma caixa de rótulo. O chip passa de `84` a `118,9`.

⚠️ A barra **quebra em linhas** (`toolbar_plan`), logo alargar o chip não esconde controlo nenhum —
no pior caso a tira ganha uma linha, que é o que ela já faz ao estreitar.

### 9.1 — Dois gates de arquitectura que a cura acordou, os dois com razão

⛔ **`no_magic_numeric`** apanhou o `0.01` de um `debug_assert!` meu — que era uma **segunda cópia da
lei** a comparar a porta com a conta inline. Apagado: com o pintor a usar a porta, o que resta a
afirmar é a ida-e-volta, e isso é um gate.

⛔⛔ **`every_stack_of_rows_asks_the_rhythm`** acusou o `toolbar_plan.rs` — e ele **tinha razão desde
sempre**: a barra empilha LINHAS com `Spacing::Xs` escrito à mão nos dois vãos, que é textualmente o
que a mensagem daquele gate prescreve. ⚠️ **Ele não o via porque a altura da linha chegava por
ARGUMENTO**: o sujeito do censo é quem menciona o `ROW_H_PX`, e este ficheiro nunca o nomeava. Foi o
meu teste novo — que o nomeia — que o tornou visível.

> *Um censo cuja população é «quem nomeia a constante» é cego a quem a recebe por argumento.*

Cura: os dois vãos passam a pedir `control_gap_px()`.

### 9.2 — Provas de mutação

| # | mutação | gate |
|---|---|---|
| M12 | o chip volta ao literal `84` | `toda_opcao_das_listas_cabe_no_chip` |
| M13 | a largura mede o item EM MÃOS e não a lista | idem — é o defeito que o censo não vê |
| M14 | o pintor volta a calcular o orçamento à mão | a varredura do app (`nenhum_corte_novo`) |

**14 de 14 sangram no dia. Dívida: 17 → 14 → 12 → 10. Portão: `19 716` impactados, `19 716` verdes.**

⏳ **E a varredura deixou uma pergunta de PRODUTO, com a medição ao lado:** na barra da timeline o
mesmo botão chama-se **`PingPong`** e no menu da timeline ele chama-se **`Ping-Pong`** — duas
grafias para a mesma coisa, e a primeira é a que não cabe na coluna dela (`54,9` contra `52,0`).

## 10. A quarta cura: a coluna de rótulo de uma FAMÍLIA mede a família

**Data:** 2026-09-19. **Gatilho:** a pergunta de produto que fecha a §9.2 — o dono respondeu
**`Ping-Pong`**, unificando as duas grafias que o app tinha para a mesma coisa.

### 10.1 — A decisão do dono, e porque ela sozinha não bastava

`panel.timeline.ping_pong` era **`PingPong`** (colado) e os outros quatro sítios que nomeiam a mesma
coisa — o menu da própria timeline, a tira do Flip, a direcção de animação do Inspector, o eco do
áudio — já escreviam **`Ping-Pong`**. A grafia colada saiu.

⚠️ **Mas ela era a palavra que não cabia, e com hífen ela cabe AINDA MENOS:** `54,90 px` colada,
**`60,45`** com hífen, contra uma coluna de `52,0`. *Uma decisão de grafia que não olha para a
coluna troca um rótulo cortado por outro.*

### 10.2 — ⛔⛔ E o censo só via UM dos dez

A coluna de rótulo dos **dez** toggles da barra de transporte era o literal `TOGGLE_LABEL_W = 52,0`,
escolhido pela palavra `AutoKey` (`48,44`) no dia em que alguém a escreveu. Medida a família inteira
pelo caminho do produto:

| rótulo | inglês | idioma de teste |
|---|---:|---:|
| **`Ping-Pong`** | **60,45** | **91,34** |
| `AutoKey` | 48,44 | 71,37 |
| `Physics` | 44,51 | 67,97 |
| `Record` | 40,66 | 63,70 |
| `Speed` | 36,69 | 56,09 |
| `Onion` | 33,91 | 53,67 |
| `Snap` | 29,20 | 45,33 |
| `Loop` | 28,70 | 44,82 |
| `Keys` | 27,96 | 44,09 |
| `Path` | 25,78 | 42,02 |

⛔ Em inglês **um** estourava os `52`. ⛔⛔ **No idioma de teste estouravam SEIS dos dez** — e a
varredura do §3 **não o dizia**, porque as duas leis dela perguntam se um rótulo pinta **NADA**, e
um rótulo cortado pinta alguma coisa. *A dívida nomeada tinha uma linha onde a medição tem seis.*

> **Um número escolhido pela palavra mais larga do dia é uma aposta na tradução que ainda não
> existe.**

### 10.3 — A cura: uma porta, e a lista a alimentá-la

**Porta** (`ph2d_editor_core::paint::label_column_width`, colada ao `paint_text` que a consome): a
largura de uma coluna de rótulo partilhada por uma família é a do **membro mais largo**, medida no
**peso em que se pinta** (`FontWeight::MEDIUM`).

⭐ Irmã da `dropdown_label_budget` da §9, um nível acima: *lá a lista é a das OPÇÕES de um chip,
aqui a dos RÓTULOS de uma família — a mesma lei, dois sujeitos.*

**Lista** (`transport_labels.rs`): os rótulos viviam inline, um `tr(...)` por braço do `match` do
pintor. Hoje há **uma** tabela `Item → chave`, lida pela régua **e** pelo pintor, e um `Item` novo
esquecido dela pinta **sem rótulo nenhum** — que se vê — em vez de pintar um rótulo cortado, que
não se vê.

**Resultado:** coluna `52,0 → 60,45` em inglês (e `91,34` no idioma de teste, sozinha), os dez
rótulos inteiros nas duas línguas, e a barra continua a quebrar em linhas como sempre fez.

### 10.4 — A coluna é TIGHT, e isso é metade do gate

Sem essa metade, repor um literal generoso (`120 px`) passaria o gate do corte: nada seria cortado e
a barra gastaria meia linha por célula. *Uma folga escondida é onde o próximo rótulo cabe por sorte
e o seguinte não — e ninguém sabe qual dos dois casos tem em mãos.*

O par ida-e-volta vive na crate do pintor: com a coluna da porta nada é elidido; **com um pixel a
menos, o mais largo É** — e o gate nomeia qual, senão a porta podia estar a medir outra coisa que
por acaso chega ao mesmo número.

### 10.5 — ⛔ E o tecto de LOC mordeu, curado por CORTE

`transport.rs` foi de `577` a `663` contra o tecto de `600`. **Nunca uma entrada no
`FILE_OVERAGE_OK`:** a lei dos rótulos saiu para `transport_labels.rs` (`663 → 592`), e o corte é
por RESPONSABILIDADE — aquele ficheiro dispõe a barra, este responde *que palavra* e *quanto espaço
ela pede*; as duas crescem por motivos diferentes.

### 10.6 — Provas de mutação

| # | mutação | o que sangra |
|---|---|---|
| M15 | a grafia volta a `PingPong` | a decisão do dono, que é um gate |
| M16 | a coluna volta ao literal `52` | o corte no produto **e** a tightness |
| M17 | a coluna mede o item em mãos (`.take(1)`) | idem — é o defeito que o §9 já pagara |
| M18 | o pintor encolhe a coluna que recebeu | «uma coluna só» e a tightness |
| M19 | a tabela esquece o `PingPong` | o rótulo não chega à tinta |
| M20 | a porta devolve folga infinita | a VOLTA (ida sozinha passaria) |
| M21 | a porta mede noutro peso | a VOLTA, pela fronteira do corte |

**7 de 7 sangram. Dívida: 10 → 9. Portão: `19 724` impactados, `19 724` verdes.**

⚠️ **Promoção pedida à lista de flakes de carga do `CLAUDE.md` §5.0:**
`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` (`ph2d-tool-painter`) — único ✗ de
`19 724` no pico do fan-out, **zero linhas do diff naquela crate**, e `3 de 3` verde sozinho a
`load 6,5`–`9,9`; a re-corrida da suíte inteira também passou. ⛔ É mais um cujo doc-comment se
declara imune por escrito (*«medidos juntos, os dois números sobem e descem juntos»*) — **verdade
sobre a deriva da MÁQUINA e falso sobre o FAN-OUT**, que é exactamente a distinção que aquela lista
existe para guardar.

## 11. A quinta cura: um ORÇAMENTO de texto é a largura de um ESPAÇO

**Data:** 2026-09-19. **Gatilho:** a régua da §10 ficou escrita, e a pergunta seguinte é *quem mais
escreve um orçamento à mão?*

### 11.1 — O censo da família, e ela tinha TRÊS membros

A varredura mede as chamadas de pintor de texto (`paint_text`, `paint_text_elided`,
`paint_text_block`, `paint_text_title`) em `ph2d-editor-core/src`, nos `ph2d-panel-*` e na shell, e
pergunta se o **7.º argumento** — o orçamento — é um número escrito ali. De `~120` chamadas
varridas, **três**:

| sítio | orçamento | o espaço REAL | o que se via |
|---|---:|---:|---|
| secção *Inspect* do Grid Snap | `80,0` | `Line / Neighbors` mede **`94,48`** | `Line / Neigh…` |
| etiqueta de selecção, o NOME | `80,0` | **`60,0`** até ao emblema | as letras **por cima** dele |
| etiqueta de selecção, a POSIÇÃO | `100,0` | **`104,0`** | apertado sem razão |

⛔⛔ **O do meio é o instrutivo, e é o OPOSTO do defeito que se procurava.** Um orçamento **maior**
que a coluna não corta nada: `Platform Player` mede `79,88`, cabia nos `80` que lhe davam, e era
desenhado **inteiro — `19,88 px` por baixo do emblema**.

> **A reticência que não aparece não é boa notícia: é a prova de que o número não descreve o
> espaço.**

⚠️ E a etiqueta de selecção mostra o **nome do objecto na Hierarquia**, que é texto do artista —
`Platform Player` é literalmente o nome de um componente que este app ship.

### 11.2 — ⛔⛔ Porque as réguas TEXTUAIS não os viam

O `the_label_column_is_one_answer` enumera **por NOME**: ele procura uma atribuição cujo nome
contenha `label_col`, ou `label_w` **só dentro de `crates/ph2d-panel-*`** (a extensão a tudo foi
medida e recusada: `16` acusações falsas para `5` verdadeiras). O doc dele já narra **quatro**
grafias da mesma pergunta.

> **A quinta é a AUSÊNCIA de grafia.** Um número passado *inline*, como argumento, não tem nome
> nenhum — logo passa por baixo de toda régua que enumere por nome.

⇒ aquele ficheiro ganha a metade `no_text_budget_is_a_bare_literal_at_the_painting_site`, com
**piso de população** (`≥ 120` chamadas varridas) e a lista de tolerância a nascer **vazia**.

### 11.3 — ⭐⭐⭐ E um QUARTO membro só o gate que PINTA podia achar

Curada a secção *Inspect*, o gate novo reprovou com `Probe A` a receber `70` enquanto os irmãos
recebiam `94,48`. A causa: as **linhas de sonda** daquela secção (`label | [x][y]`) tinham uma
**segunda** coluna, `let label_w = 70.0`, no mesmo ficheiro.

⛔ Ela escapava às **duas** réguas textuais ao mesmo tempo: a primeira só aceita `label_w` em
`ph2d-panel-*` (isto é `ph2d-editor-core`) e a segunda procura um número **sem nome**, e este tinha
um.

> *Uma régua que LÊ o fonte mede o que alguém escreveu; uma que PINTA mede o que o artista vê — e
> só a segunda vê duas colunas onde o desenho tem uma.*

Hoje a coluna é medida uma vez da lista dos quatro rótulos e **chega às linhas de sonda por
argumento**, o que também devolve `~12 px` a cada um dos dois campos numéricos delas.

### 11.4 — A geometria da etiqueta não se move nem um pixel

O emblema continua a começar a `60` do início do nome e o texto de posição onde sempre esteve; o
que muda são os dois ORÇAMENTOS, que passam a ser as colunas (`60 − Sm = 54` e `104`). ⛔ **As
colunas ficam FIXAS de propósito:** dar ao nome o que sobra faria o emblema **dançar** a cada
objecto escolhido, e o texto de posição muda quando o objecto se MOVE — a etiqueta tremeria a
arrastar. *Uma coluna elástica é certa numa linha de formulário e errada numa etiqueta que paira
sobre o canvas.*

### 11.5 — Provas de mutação

| # | mutação | o que sangra |
|---|---|---|
| M22 | a coluna do *Inspect* volta ao literal `80` | o rótulo mais largo é cortado |
| M23 | o literal reposto **no sítio da pintura** | a régua TEXTUAL nova |
| M24 | a coluna ganha folga (`+40`) | a metade *tight* |
| M25 | as linhas de sonda voltam a ter coluna própria | «uma coluna só» |
| M26 | o nome da etiqueta volta ao orçamento de `80` | ele deixa de ser cortado |
| M27 | a coluna da posição volta ao literal `100` | ela deixa de ser o que sobra |
| M28 | o emblema muda de sítio | a geometria que não se move |
| M29 | a régua textual deixa de achar chamadas | o **piso de população** |

**8 de 8 sangram. Dívida: 9 → 8. Portão: `19 729` impactados, `19 728` verdes.**

⚠️ O único ✗ foi `the_cost_of_a_player_is_linear_in_their_number` (`ph2d-physics-ecs`), **membro já
listado** da família de flakes de fan-out do `CLAUDE.md` §5.0: zero linhas do diff naquela crate,
reprovou a `load 33,57` no meio do fan-out e passa **`3` de `3` a `load 106`–`109`** — *o triplo da
carga em que reprovou*, que é a assinatura da família.

---

## 12. A sexta cura: uma PÍLULA mede a palavra que carrega — e uma INVERSA em `f32` não fecha por álgebra

**2026-09-19**, a seguir à §11. A dívida nomeada estava em **oito** e as cinco linhas que sobravam
eram dos dois painéis de LABORATÓRIO (`widget_gallery`, `widget_lab`). Ao medi-las, a varredura
devolveu **dois defeitos VIVOS no produto que ela própria nunca poderia ver** — e uma terceira
família de aritmética que atravessa a casa inteira.

### 12.1 — O que a medição deu

Sonda no caminho do produto (os dois painéis, três viewports):

| painel | rótulo | mede | orçamento | saía |
|---|---|---:|---:|---|
| gallery | `Float` | `30,72` | `27,82` | `Fl…` |
| gallery | `Color` | `33,54` | `27,82` | `C…` |
| gallery | `filter` | `24,26` | `21,10` | `fil…` |
| gallery | *«Canonical widget showcase · …»* | `290,76` | `268,00` | `…peripheral…` |
| lab | *«Bar · the fill is …»* | `399,79` | `384,00` | `…the n…` |
| lab | *«268 = today's Inspector · …»* | `386,58` | `384,00` | `…= tab…` |
| lab | `Geometry Offset` | `95,16` | `70,00` | **a demonstração** |

### 12.2 — ⛔⛔⛔ Os dois defeitos que a varredura do app NÃO PODIA ver

A varredura pinta **cada painel do registo com o estado de FÁBRICA**. Um Inspector de fábrica não
tem objecto seleccionado e uma Hierarquia de fábrica tem uma linha sem selo — logo **nenhum dos dois
pinta uma pílula**. *Um censo que varre painéis vazios mede o painel vazio.*

O `filter` da vitrina é a MESMA pílula desses dois painéis, e ao medi-lo apareceu o mecanismo:

> **O `paint_tag` cobrava o respiro DUAS vezes.** A pílula reserva `pad_x = max(h/2, 8)` de cada
> lado (o recuo dela), e depois entregava essa faixa ao `paint_text_centered`, que **volta a
> descontar** o respiro de uma caixa de rótulo (`Md·2 = 16`). Numa pílula esse respiro **já foi
> pago**, e o `pad_x` dela é maior do que ele.

| sítio | rótulo | mede | orçamento antigo | saía |
|---|---|---:|---:|---|
| Inspector · secção *Tags* | `Ground` | `38,95` | `22,95` | `Grou…` |
| Inspector · secção *Tags* | `Enemy` | `35,64` | `19,64` | `Ene…` |
| Hierarquia · selo | `CAM` | `25,66` | `20,00` | `CA…` |
| Hierarquia · selo | `ENT` | `22,13` | `20,00` | `EN…` |
| Hierarquia · selo | `GRP` · `LNK` · `SPR` · `PRF` | `20,67`–`22,41` | `20,00` | cortados |
| Hierarquia · selo | `ISO` | `18,54` | `20,00` | **cabia** |

**Seis dos sete selos da Hierarquia e TODO chip da secção *Tags* saíam com reticência.**

### 12.3 — ⭐⭐ E a lei mudou de dono: a pílula mede a palavra

A secção *Tags* tinha uma **segunda cópia** da geometria da pílula, com o doc dela a dizê-lo por
escrito (*«o inverso EXACTO da geometria do `paint_tag`»*). *Uma lei escrita em dois sítios ainda
não é uma lei — só uma PORTA é.* Hoje:

- `pad_x(h)` · `close_size(h)` · `removable_chrome(h)` são **funções**, e o `close_rect`, o pintor e
  as duas inversas chamam-nas — antes eram **três** cópias.
- `Tag::label_rect` / `Tag::label_budget` — o que a pílula GASTA.
- `Tag::width_for` / `Tag::natural_width` — o caminho INVERSO, com **três** consumidores (a vitrina,
  a secção *Tags*, o selo da Hierarquia).
- ⭐ **A pílula LISA é uma caixa de rótulo**, logo o par dela é o `rect_for_label`/`label_budget` que
  a casa já tinha — e o caminho dela fica **byte a byte** como estava.
- ⭐ **O selo da Hierarquia tem PISO**, que é a ranhura do cacho de ícones (`ICON_BTN_SIZE_PX`): o
  selo é uma daquelas ranhuras, logo nunca encolhe abaixo dela e só CRESCE quando a palavra pede.

### 12.4 — ⛔⛔⛔ A terceira família: uma inversa em `f32` não fecha por álgebra

Curada a coluna da espécie do `VariantEditor` (`45 %` da linha → a FAMÍLIA), apareceu um corte
**novo** que a álgebra diz ser impossível:

```
widget_gallery  CORTE "Dictionary" -> "Dictiona…"  | orçamento 63.62 | precisa 63.62
```

A coluna pedia **exactamente** o que a palavra mede e recebia esse número de volta — e ela era
cortada. Em `f32`, `(t + c) − c` fica **abaixo** de `t`, e a elisão compara `<=`: *um défice de um
ULP corta a palavra inteira.* Varrido o domínio em passos de `~1e-3 px`:

| par ida/volta | falhas | pior défice |
|---|---:|---:|
| `rect_for_label` / `label_budget` (**já shipava**) | `22 812` de `859 927` (**`2,65 %`**) | `3,05e-5 px` |
| `Tag::width_for` / `Tag::label_budget` | `440 535` de `460 599` (**`~96 %`**) | `3,05e-5 px` |
| `dropdown_chip_width_for` / `dropdown_label_budget` | apanhado pelo **PRODUTO** | — |

⛔⛔ **E a prova que guardava o primeiro par amostrava SEIS pontos, e passava nos seis.** *Uma prova
por amostras sobre uma lei que falha em `2,65 %` do domínio lê-se como prova.*

⇒ a inversa passou a **conferir-se contra a LEI** (`if orçamento(w) < t { w.next_up() }`), nos três
pares, com gate a varrer `~460 000` pontos por altura. Depois da cura: **`0` falhas em `7 361 552`
pontos**.

### 12.5 — A coluna de uma escolha mede a FAMÍLIA (a 3.ª vez)

| espécie | mede | orçamento antigo | saía |
|---|---:|---:|---|
| `Dictionary` | `63,62` | `27,82` num filho | — |
| `Integer` | `44,01` | `27,82` | `Inte…` |
| `Color` | `33,54` | `27,82` | `C…` |
| `None` | `33,14` | `27,82` | `No…` |
| `Float` | `30,72` | `27,82` | `Fl…` |
| `Text` | `26,72` | `27,82` | cabia |

⚠️ **O censo via DOIS dos seis** — ele mede a opção ESCOLHIDA de cada linha. ⛔ E a coluna da
**CHAVE** continua a ser uma fracção, que é a decisão: *a chave é dado do ARTISTA e não tem tamanho
conhecido; o que não pode ser uma fracção é a coluna que a CASA escreve.*

### 12.6 — As três legendas QUEBRAM, e a régua fica

As três frases de prosa passaram a `paint_text_block` — a mesma cura das duas frases de estado vazio
do produto em 18/09 — e as três **devolvem a altura ao chamador**: sem isso a 2.ª linha escreveria
por cima do risco do cabeçalho da vitrina e da fileira seguinte da bancada.

⛔ **O `Geometry Offset` FICA na lista, e não é dívida: é a DEMONSTRAÇÃO.** A §2 do laboratório
chama-se *«the chosen design, squeezed»* e desenha a mesma linha a `268 · 184 · 140 · 110` px com um
rótulo comprido de propósito. A `110` a coluna do rótulo fica com `70,00` e a palavra mede `95,16`
⇒ ela **tem** de sair cortada. *Curar aquela linha seria apagar a medição que o painel existe para
fazer.*

### 12.7 — Provas de mutação

| # | mutação | o que sangra |
|---|---|---|
| M30 | a pílula volta a pagar o respiro duas vezes | os chips da secção *Tags* |
| M31 | o selo volta a ser o slot do ícone | seis dos sete selos |
| M32 | o selo perde o piso do slot | a coluna da direita fica irregular |
| M33′ | o Inspector re-deriva a geometria e erra **um vão** | os chips |
| M34 | a pílula confia na álgebra | a varredura da pílula |
| M35 | a caixa de rótulo confia na álgebra | a varredura da caixa |
| M36 | o chip de escolha confia na álgebra | a varredura do chip |
| M37 | a coluna da espécie volta a ser fracção | `Float`/`Color` |
| M38 | a coluna mede o item EM MÃOS | `Dictionary` |
| M39 | a legenda da vitrina volta a ser cortada | a catraca de cortes do app |
| M40 | a legenda da bancada volta a ser cortada | a catraca de cortes do app |

⚠️⚠️ **E a M33 ORIGINAL — restaurar a cópia EXACTA — SOBREVIVEU, e isso é um facto e não um
buraco:** a cópia calcula hoje o mesmo número que a porta. *O mal de uma segunda cópia não é o
número de hoje, é o de AMANHÃ* — e é isso que a M33′ mede, contando os vãos como a cópia os contaria
no dia em que o recuo da pílula mudasse.

**11 de 11 sangram. Dívida: 8 → 2** (uma decisão de produto + uma demonstração).

---

## 13. A fronteira que o §5 chamava de dívida NÃO é dívida — e o censo mentia nos DOIS sentidos

**2026-09-19**, depois de a caça às elisões fechar. O `CLAUDE.md` §5 nomeia a fronteira seguinte do
HR-15 como *«os `62` literais que só o censo de PORTA vê»*. Fui medi-la. **Nenhum dos 62 é dívida**,
e o caminho até essa resposta achou três defeitos no instrumento.

### 13.1 — As três curas do `scripts/censo-texto-pintado.py`

| # | o que ele fazia | o que isso valia |
|---|---|---|
| 1 | contava um `#[cfg(test)] mod` **em linha** como produto | `+5` falsos (uma delas `"Um nome absurdamente comprido para uma caixa"`, uma FIXTURA) |
| 2 | **não seguia** uma tabela `const [&str; N]` até ao pintor | `−31` reais, e **nunca contados por ninguém** |
| 3 | lia *«tem uma letra»* como *«é uma palavra»* | `+22` falsos (`R Y G C B M W N K`; e as setas, porque ele lia o **escape do fonte**, que tem um `u` e um `b`) |

⚠️ **A nº 2 estava escrita no cabeçalho dele havia semanas** (*«literais que chegam por `const`, por
tabela de `&str`… continuam a ser texto pintado»*). ⛔ *Uma cegueira declarada num doc-comment não é
uma medição — é uma nota que envelhece*, e enquanto ela lá esteve ninguém soube que havia **31**
palavras pintadas fora de toda contagem.

⚠️ **E a nº 3 é a mesma grandeza medida por duas réguas que discordavam:** a régua REGISTADA
(`ph2d_label_census::lexical::cegueira`) já pedia **duas letras ASCII seguidas**. *Duas réguas da
mesma grandeza que discordam sobre o que é uma PALAVRA não são comparáveis* — e era esta que estava
a mais.

### 13.2 — ⭐ O controlo positivo, `--autoteste`

*Uma cura de instrumento sem controlo positivo é uma afirmação sobre o instrumento antigo.* As três
leis são testáveis sem a árvore, e o modo novo planta um fonte sintético com as três formas e as
suas negativas.

⚠️⚠️ **E uma mutação SOBREVIVEU, e a lição é do controlo e não da lei:** o meu *«uma letra não é
palavra»* usava `"R"`, e numa cadeia de **um** carácter a exigência da segunda letra é
**inobservável** — não há par nenhum para testar. Apagar a adjacência passava. O controlo que
discrimina é `"a b"`: duas letras SEPARADAS. *Um controlo tem de conter o fenómeno que ele afirma.*

### 13.3 — ⛔⛔⛔ E a medição PAROU o trabalho que eu ia fazer

Com o censo honesto, os `52` que restam são:

| onde | quantos | o que são |
|---|---:|---|
| `ph2d-editor-core/src/widget/showcase/` | 28 | a **BANCADA** — isenta por **decisão do DONO** (2026-09-17, com a foto na mão: *«nenhum — deixe como está»*) |
| `ph2d-panel-widget-lab` | 1 | a mesma bancada, o mesmo estatuto |
| `ph2d-app-motion` demos da conferência | 23 | **as palavras do DONO**, em português, nas cenas dele |

E as 23 eram exactamente o que eu ia curar — o app a pintar português, contra a lei de que *toda
string que o artista lê é inglês*. ⛔ **Não são dívida, e há duas provas escritas:**

1. o cabeçalho da tabela `app.motion` declara que **as CENAS ficam fora da i18n de propósito**, e o
   gate da família isenta-as por marcador de nome de ficheiro, com piso de população;
2. ⭐⭐⭐ uma delas foi **escolhida pelo próprio Enio num smoke**:

   > *«`SOLTA`, e não «RASGA» — a palavra foi corrigida por um smoke (Enio, 2026-08-21: "funciona
   > mas não rasga o pano")»*

*Traduzir aquelas palavras era desfazer uma decisão do dono sobre texto que ele lê.* A medição
custou uma hora e poupou uma fatia inteira de trabalho errado — que é exactamente o que o §5.0 pede
quando manda MEDIR se a composição já exprime o item antes de o construir.

### 13.4 — Provas de mutação

| # | mutação | o que sangra |
|---|---|---|
| M44 | o `#[cfg(test)]` em linha deixa de ser apagado | o controlo do módulo interno |
| M45 | a tabela de `&str` deixa de ser lida | o controlo da tabela |
| M46′ | uma letra solta volta a ser palavra | o controlo `"a b"` |

**3 de 3 sangram. Dívida do censo de porta: `62` → `0`** (tudo o que resta é isenção declarada).

---

## 14. «Tudo em inglês» — a ordem do dono, e a régua que nenhuma catraca tinha

**2026-09-19.** A §13 acabou com uma pergunta ao dono: as `23` palavras que as cenas da conferência
do Motion pintam em português ficam como estão (são dele) ou vão a inglês? Resposta:

> **«tudo em inglês»**

### 14.1 — ⛔⛔⛔ E a pergunta abriu uma porta que nenhuma catraca do HR-15 guardava

As **trinta** catracas perguntam *«esta palavra veio da TABELA?»*. **Nenhuma pergunta em que
LÍNGUA a tabela está.** Medido sobre as `5 226` entradas:

| onde | quantas | o que são |
|---|---:|---|
| `app_sculpt3d.rs` | **8** | frases que o artista lê na caixa de saída da escultura — **em português** |
| cenas da conferência (5 ficheiros) | **23** | palavras pintadas no canvas — **em português** |
| `shell.undo_app.*` | 5 | ⛔ **NÃO são defeito**: diagnóstico que só corre com o log ligado |

⚠️ **As oito viviam FORA dos marcadores do script de migração**, e o comentário ao lado delas
declara-as *«a frase que o artista lê, e não diagnóstico de consola»* — *um texto escrito à mão
depois da migração entra na tabela pela porta que a migração não guarda*.

### 14.2 — A linha que separa as duas audiências

⭐ **O que o ARTISTA lê no ecrã é inglês; o que o DONO lê no terminal é a língua dele**
(`CLAUDE.md` §0.8). É por isso que a prosa de consola das cenas — as instruções de smoke, que são
para ele — **fica em português**, e as cinco chaves do `undo_app` ficam isentas **com o mecanismo**.

### 14.3 — As 23 palavras, e a que herdou uma decisão

| cena | era | ficou | porquê |
|---|---|---|---|
| todas | `ANTES` · `DEPOIS` | `BEFORE` · `AFTER` | |
| goal | `ALVO` · `MIRA` | `TARGET` · `AIM` | |
| operator | `RASTRO` | `TRAIL` | o *Echo Operator* |
| rank | `CORTE` · `BANDA` · `RAMPA` · `FORMA` | `CULL` · `RANGE` · `RAMP` · `SHAPE` | cada um NOMEIA o nó da linha |
| sim | `SOLTA` · `DESVIA` | **`RELEASE`** · `AVOID` | ⬇️ |
| style | `BORDA` · `APARADO` · `PICOTADO` | `STROKE` · `TRIM` · `DASH` | |

⛔⛔ **`RELEASE`, e não `BREAK` — a decisão do dono ATRAVESSA a tradução.** A palavra portuguesa já
era uma correcção dele por smoke (2026-08-21: *«funciona mas não rasga o pano»*): o que rompe é o
**PREGO**, e o pano sai INTEIRO. O knob chama-se `Break Above`, e traduzir `SOLTA` pelo nome do knob
reintroduzia, em inglês, exactamente a promessa que ele mandou tirar. *Um rótulo promete o que o
modelo entrega — e uma tradução herda a promessa, não só a palavra.*

### 14.4 — A régua vive numa PORTA, com os dois controlos

[`ph2d_label_census::portuguese_tokens`] — classe fechada do português ∪ morfologia
(`-ção/-cao`, `-ões/-oes`, `-ão/-ao`, `-mente`, `-ando/-endo/-indo`), com os marcadores de `format!`
retirados primeiro (foi isso que fez a 1.ª sonda acusar `"never fires · {verbo} · {alvo}"`, que é
uma frase inglesa).

⚠️ **Ela declara-se um PISO**: `"colorize a recalcular"` passa-lhe ao lado. *Uma régua heurística
que se declara é utilizável; uma que se julga completa é uma licença.* E vive na folha partilhada
porque tem **dois** consumidores — a tabela e as cenas.

### 14.5 — ⛔⛔ E o LEITOR mentiu antes da régua

A 1.ª sonda lia as entradas com um `.` que não casa quebra de linha ⇒ **toda entrada com `\` de
continuação evaporava**: ela leu `3` das `8`, e as cinco que faltavam eram as mais compridas. ⚠️ E
a 2.ª redacção estourou num `á` (`end byte index … is not a char boundary`) por contar a linha
fatiando o `&str` com um índice de **carácter**. *Um índice de carácter e um índice de byte leem-se
igual num `usize`.*

⇒ o gate ganhou uma metade **sobre o leitor** (`a_leitura_junta_uma_entrada_partida_em_duas_linhas`)
— os outros três não a apanham: o piso conta ENTRADAS (que não mudam), o controlo positivo usa
fixturas escritas à mão (que não passam pelo leitor), e a tabela está curada (logo não há português
que se perca).

### 14.6 — Provas de mutação

| # | mutação | o que sangra |
|---|---|---|
| M47 | uma frase da escultura volta ao português | o gate da tabela |
| M48′ | a régua da língua deixa de ver português | o controlo da própria régua |
| M49′ | o leitor da tabela trunca na continuação | a régua sobre o leitor |
| M50 | a isenção de consola fica órfã | o censo de obsolescência |
| M51 | o cabeçalho da cena volta ao português | o gate das cenas |
| M52 | os rótulos de linha voltam ao português | o gate das cenas |
| M53 | o leitor da cena deixa de ver a TABELA | o controlo da leitura |

**7 de 7 sangram. Português que o artista lê: `31` → `0`.**
