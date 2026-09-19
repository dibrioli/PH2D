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
