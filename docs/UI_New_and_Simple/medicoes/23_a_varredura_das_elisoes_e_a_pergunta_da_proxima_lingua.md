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
