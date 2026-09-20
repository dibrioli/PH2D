# HANDOFF DE INTEGRAÇÃO — `line/components` · **AS ÂNCORAS DO HUD** (2026-09-19)

> Leitor: a próxima LLM e o agente INTEGRADOR. Denso de propósito (`CLAUDE.md` §0.8).
> Sonda do §5.0:
> [`mede_o_que_falta_as_ancoras_do_hud`](../../../crates/ph2d-app-components/tests/it/mede_o_que_falta_as_ancoras_do_hud.rs)

## §1 — O que a jornada entrega, em uma frase

**O placar cola-se ao canto de verdade.** Um filho de um canvas de HUD com regra de âncora segue as
bordas **REAIS** da vista quando a janela muda de aspecto, em vez de ficar a uma banda de letterbox
delas. É o item que o [handoff do #20](HANDOFF_INTEGRACAO_line_components_HUD_2026-09-17.md) §7
deixou aberto — e com ele o **#20 fecha**.

## §2 — ⛔⛔ A sonda do §5.0 CORRIGIU a redacção do item aberto

Ele escreve *«as quatro âncoras do `VecAnchors` **não estão LIGADAS** ao canvas»*. Medido:

| bloco | a resposta MEDIDA |
|---|---|
| **A) a BANDA** | com `Keep`, `21:9` deixa **`5,0`** de mundo por preencher em `x`; `4:3` deixa `3,0` em `y`; `16:9` deixa **zero** |
| **B) o CANTO** | um filho no canto da caixa de referência pousa em `16,0` e a borda real está em `21,0` — **faltam `5,0`, que é exactamente a banda** |
| **C) a ÂNCORA de hoje** | `delta_local` devolve **`0,0`** — e **por construção** |
| **D) o CONTRAFACTUAL** | com a caixa **EFECTIVA**, o erro à borda é `0,000000` nas três janelas |

⭐⭐⭐ **Elas estão LIGADAS e são INERTES.** O `delta_local` pergunta *«a moldura mudou de
tamanho?»* e a caixa de um canvas é **a mesma em toda janela** — o que muda é a **ESCALA** da raiz
⇒ o delta sai `0,0` por subtracção de iguais. *Não era um fio por ligar: era a régua a medir uma
grandeza que não se mexe.*

## §3 — Os CONTADORES (⛔ nunca o literal)

| contador | delta |
|---|---|
| `PROJECT_SCHEMA` | **0** |
| registo do `ph2d-ecs` + os dois espelhos | **0** |
| componentes novos | **0** — o `VecAnchors` já existia e já era registado |

⭐ **A wave é uma PORTA e uma SELECÇÃO, e nada mais.** Nenhum tipo novo atravessa o ficheiro.

## §4 — As decisões

### §4.1 — ⭐ ZERO lei de âncora nova

O `LayoutLive::anchor_kid` do passe das molduras é o **mesmo**, e é por isso que o **PINO**
(`min == max`) e o **ESTICÃO** (`min != max`) saem os dois de graça. Construir uma segunda lei
teria sido *«a forma mais cara de ignorar a §5.0»* — a frase que o próprio `ph2d_ecs::hud` já
escreve.

### §4.2 — A caixa EFECTIVA, e o neutro EXACTO

[`ph2d_hud::effective_box`] = a de referência **mais a banda**, em unidades locais. A aritmética
simplifica para **`half / escala`** — *a vista, vista de dentro do canvas*.

⭐ No aspecto da própria caixa a banda é zero, a efectiva **É** a de referência, o afim é a
identidade e o `anchor_kid` sai no `is_identity` **sem pagar a cópia da geometria** ⇒ o que se
desenha é **byte-idêntico** ao de antes. *Sem isto, toda cena de HUD já autorada mudava de imagem.*

### §4.3 — ⭐⭐ E a recusa declarada do `Fit::Expand` DISSOLVE-SE

O cabeçalho da `ph2d-hud` recusava o `expand` porque *«fazer os filhos chegarem à borda exigiria
redimensionar a moldura por quadro, que é escrever no DOCUMENTO»*. ⚠️ **A premissa caiu:** a caixa
efectiva é **derivada por quadro**, como a pose da raiz, e não toca no documento. *Quem move o
número que tornava algo inalcançável tem de reconferir a nota* (§0.0) — a nota está reescrita no
doc da porta, e o `expand` fica como wave possível e não como impossibilidade.

### §4.4 — Onde cada metade mora, e porquê

| metade | onde | razão |
|---|---|---|
| a caixa efectiva | `ph2d-hud` | é aritmética pura sobre dois rectângulos |
| a moldura que um canvas oferece | `hud_bridge::anchor_frame_of` | a escala é a que **conduziu** a raiz (`place`), nunca uma segunda medição |
| **quem** ancora quem | `ph2d_app_components::hud_anchors::ancorados` | pergunta sobre COMPONENTES ⇒ gates sem cena, sem device |
| publicar a pose | `shells/desktop/src/layout_live_anchors.rs` | é a maquinaria que o passe das molduras já tem |

⭐ As duas populações — filho de moldura, filho de canvas — são **disjuntas por CONSTRUÇÃO**: um
filho tem UM pai, e ele ou tem `VecFrame` ou tem `UiCanvas`.

## §5 — ⚠️⚠️ O que este diff MEXE em ficheiro PARTILHADO

| ficheiro | o que muda | risco |
|---|---|---|
| `shells/desktop/src/layout_live_anchors.rs` | **+ uma função no FIM** (`anchor_canvases`) | aditivo — nenhuma linha existente mexida |
| `shells/desktop/src/layout_live.rs` | `+1` campo (`vista`) e `+1` chamada | aditivo |
| `shells/desktop/src/render_loop/fase_hud.rs` | `+1` linha (guarda a vista) | ⚠️ **a ORDEM é load-bearing** e tem gate |
| `shells/desktop/src/render_loop/fase_vector_layout_recook.rs` | `+1` linha (lê a vista) | idem |
| `shells/desktop/src/app_state_components_smokes.rs` | `+1` campo em `HudShell` | ⭐ **zero campos novos na `App`** — a catraca `the_app_only_sheds_fields` não se mexe |

## §6 — ⭐⭐ A CATRACA DA SHELL reprovou, e a cura foi MOVER

`the_shell_only_shrinks` leu **`197 288`** contra `196 990`. ⛔ **Subir o número está fora.** Três
movimentos, todos por RESPONSABILIDADE:

1. a **selecção** (*quem ancora quem*) foi para a crate da família, onde tem gates baratos;
2. a **cena** das âncoras foi para casa (`hud_smoke_anchors`), como as irmãs desta linha — ela
   nasceu na shell por inércia;
3. ⭐ saiu a sonda **`probe_cursor_grab`** — `295` linhas que **nada chamava** (o único sítio que a
   nomeava era o `mod` do `main.rs`), agora [`ph2d-probe-cursor-grab`](../../../crates/ph2d-probe-cursor-grab/)
   com o gate dela. *O candidato veio de uma MEDIÇÃO e não do tamanho.* ⚠️ A pergunta que ela
   responde é do plano da **UI** — isto é uma mudança de ENDEREÇO, não de dono.

Shell: **`196 835`**.

⛔⛔ **E apagar aquele `mod` re-ligou o `#[cfg(test)]` dele ao VIZINHO** — `19` erros de compilação
sobre o `profile_live`. *A armadilha está escrita no repo (handoff da física, Fase A §9) e aqui foi
**BARULHENTA**, que é a única sorte da história.*

## §7 — A prova de fecho

| o quê | onde | número |
|---|---|---|
| a caixa efectiva | `ph2d-hud/src/tests.rs` | **5** gates |
| a selecção | `ph2d-app-components/src/hud_anchors_tests.rs` | **5** gates |
| o que se DESENHA | `shells/desktop/src/layout_live_anchors_canvas_tests.rs` | **4** gates |
| a ORDEM no quadro | `shells/desktop/tests/it/o_hud_conduz_a_raiz_antes_de_as_ancoras_a_lerem.rs` | **2** gates |
| a CENA | `…/o_rotulo_do_botao_do_hud_e_filho_dele.rs` | **+1** gate |
| a caixa efectiva contra o ORÁCULO | `ph2d-hud/src/tests.rs` | a tabela do bloco **L4**, verbatim |
| o selector do `Fit` | `ph2d-app-components/src/hud_inspector_tests.rs` | **2** gates |
| **provas de mutação** | `docs/Components/ferramentas/mutacao_ancoras_hud_2026-09-19.sh` | **11 de 11 sangram** |
| **portão** | `scripts/nextest-impacted.sh` | **15 465**, com a única reprovada a ser membro confirmado da família de flakes de FAN-OUT (3/3 verde sozinho a `load 22`, zero linhas de diff) · clippy `-D warnings` a zero |

## §8 — ⛔⛔ DUAS mutações sobreviveram primeiro, e as duas eram MINHAS

| # | o que aconteceu | a lição |
|---|---|---|
| **a cerca do PAI** | apagá-la não sangrava porque o chamador **voltava a perguntar** `get::<UiCanvas>(root)` — a cerca estava escrita **DUAS vezes** | *Uma mutação **neutralizada** por uma segunda guarda lê-se como sobrevivência num relatório e não é.* ⇒ a cura não é um gate a mais, é a **segunda resposta a menos**: a porta devolve o PAR |
| **a escala da moldura** | trocá-la por `1` não mudava nada, porque a fixtura estava no ponto **NEUTRO** (`Keep` a `42×18` dá `s = 1,0`) | *Um corpus no ponto NEUTRO de um knob não testa esse knob* ⇒ gate novo com `s = 2,0` |

⚠️ E **um gate meu era fraco**: ele dizia só *«o filho de moldura não andou `+5`»*. Com a cerca
apagada ele era ancorado pelos DOIS passes e o valor final continuava a não ser `5` ⇒ a asserção
passou a ser **EXACTA** (`[16, 26]`). *Um gate que recusa UM valor não afirma qual é o certo.*

## §8-bis — ⛔⛔ A SEGUNDA METADE: o `Keep` estava a fazer o trabalho do `expand`

⚠️⚠️ **A 1.ª versão desta wave shipou um defeito por uma hora:** o `effective_box` crescia a caixa
no **`Keep`**, e isso fazia o `Keep` comportar-se como o **`expand`** do alvo — uma divergência
**silenciosa** que retirava a capacidade de confinar o HUD à área segura.

⭐⭐⭐ **O oráculo decidiu, e ao NÚMERO.** A sonda ganhou o bloco **L4** (um `Control` preso ao canto
inferior-direito, janela `720×450` do headless):

| aspecto | ref | a caixa que o filho lê | o canto dele no ecrã |
|---|---|---|---|
| `keep` | `640×360` | **`(640, 360)`** — a referência | `(720, 428)`, a **`22 px`** da borda |
| `keep` | `1280×360` | `(1280, 360)` | `(720, 326)`, a `124` |
| `expand` | `640×360` | **`(640, 400)`** | `(720, 450)` — **a borda** |
| `expand` | `1280×360` | `(1280, 800)` | `(720, 450)` |
| `expand` | `320×480` | `(768, 480)` | `(720, 450)` |

⭐ A nossa caixa efectiva reproduz o `expand` **nos três casos** (`janela/escala`), e o gate
`a_caixa_efectiva_bate_o_oraculo_ao_numero` tem a tabela **verbatim**.

⇒ **`Keep` devolve a referência** (byte-idêntico ao de antes) e **`Expand` é o modo novo**. A POSE
dos dois é a mesma **ao bit** — o que muda é só até onde uma âncora pode ir.

⚠️ **`Fit` ganha uma variante APENDADA** (a posição é a tag do postcard ⇒ ficheiros gravados leem-se
na mesma): `PROJECT_SCHEMA` **0**.

⭐⭐ **E o `Fit` deixou de ser mapeado À MÃO.** O Inspector fazia `u8::from(fit == Stretch)` e
`if *i == 1` ⇒ o modo novo existiria com lei e gates e **o artista não lhe chegava** — o defeito que
o `Density` da escultura e o verbo `Destroy` pagaram. Hoje há `Fit::ALL` + `label()` +
`index()`/`from_index`, e o menu tem **uma linha por modo**, com gate a prender as duas listas.

### As três armadilhas que esta metade pagou

| # | o que aconteceu | a lição |
|---|---|---|
| **o selector** | a mutação que punha `ALL = [Keep, Stretch, Stretch]` **sobreviveu** ao gate de ida-e-volta | *Um `ALL` com DUPLICADO fecha a volta* — o `index()` devolve a posição do **primeiro** igual. Quem o apanha é o `dedup` dos rótulos, noutra crate ⇒ a mutação mudou de alvo |
| **a âncora** | `'…\n…'` num `'…'` do bash são **dois caracteres**, não uma quebra ⇒ casou zero vezes | o arnês **ABORTOU alto** (`⛔ ANCORA`), que é para o que ele existe ⇒ `$'…'` |
| **o censo do HR-15** | ele acusou a minha própria **string de teste** (`panel.inspector.hud.fit_`) | *Um gate que procura chaves não pode CONTER uma* ⇒ a agulha monta-se por pedaços com `concat!` |

⚠️ **E a cena mudou de modo:** ela abre em `Expand`, porque com `Keep` **arrastar a borda não move
um pixel** e o roteiro manda arrastar (gate `a_cena_do_hud_abre_em_expand`). ⭐ O **CONTROLO** é o
próprio selector — o passo (5) manda trocá-lo para `Keep` e alargar outra vez.

**11 de 11** mutações sangram.

## §9 — ABERTO, e de quem é cada item

| item | de quem |
|---|---|
| **no EDITOR** um HUD colado às bordas cai atrás dos painéis | **decisão de produto** (a vista da câmera é a da JANELA) |
| o `UiButton` só é alcançável por caminho **vectorial** | nomeado, não construído |
| a âncora de um filho armada **na janela errada** grava a caixa daquela janela | a cena arma-a contra a de REFERÊNCIA; um gesto de painel para HUD ainda não existe |

## §8-ter — ⛔⛔⛔ A TERCEIRA METADE: o LADO de cada peça, e a escada sem fim (report de 20/09)

Dois reports do dono no mesmo dia, e **nenhum era a lei das âncoras**.

### (a) *«infinitos logs. melhor tirar.»*

A auto-conferência do dedo corria no quadro em que `hud_raise` chegava a `0` — e
`levanta_o_inspector(0)` devolve `0` **para sempre** a partir daí ⇒ ela corria em **todos** os
quadros seguintes, a `60 Hz`, com uma linha de diagnóstico cada.

> *Um contador que SATURA não é um estado terminal.*

⇒ [`proximo_estado`](../../../shells/desktop/src/hud_smoke.rs), **pura**, com `FEITO` a ser **ponto
fixo**, e a cauda a correr na **TRANSIÇÃO** e nunca no estado. ⚠️ A lei é uma função porque *um gate
que precisa de janela, GPU e superfície para medir dois números está no sítio errado*. Medido depois
da cura: o log da cena inteira tem **24 linhas**.

### (b) *«em expand ... podem ir para seu próprio lado ou para o lado oposto e até se cruzar»*

A **REGRA** estava certa e a **POSIÇÃO** não: a pontuação prendia-se à aresta DIREITA e era autorada
em `x = −7`; a contagem prendia-se à ESQUERDA e estava em `+8`. Com `Fit::Keep` a caixa efectiva é a
de referência, o delta sai `0,0` por subtracção de iguais e nada se move — logo *«os demais modos
OK»*; com `Fit::Expand` cada uma anda para a **sua** borda.

Medido pelas portas do produto (`anchor_frame_of` + `delta_local`, ref `32 × 18`, meia-altura `4,5`):

| aspecto da vista | pontuação | contagem | |
|---|---|---|---|
| `1,78` (`16:9`) | `−3,50` | `+4,00` | cada uma no lado ERRADO |
| `2,22` | `−1,50` | `+2,00` | a aproximarem-se |
| `2,67` | `+0,50` | `0,00` | **cruzam-se** |
| `4,00` | `+6,50` | `−6,00` | já do outro lado uma da outra |

⭐⭐ **A cura não é mover dois literais: é o SINAL passar a sair da REGRA.** [`Canto::local`] deriva-o
de [`Canto::fraccao`] ⇒ *uma peça autorada do lado oposto à âncora dela deixa de ser exprimível*.

### ⚠️ E o gate que devia ter apanhado isto media só a REGRA

O `as_duas_pecas_de_baixo_prendem_se_a_cantos_opostos` lia as fracções `min` e comparava-as — e elas
**estavam** opostas. *Ele nunca perguntou ONDE a peça está.* Reescrito como
`cada_peca_de_baixo_e_autorada_do_lado_a_que_se_prende`, com a premissa morta visível no diff: ele
**CRUZA** as duas metades (que canto cada peça usa para nascer × a quem a cena dá cada regra), e lê a
tupla que liga peça↔entidade em vez de confiar no nome.

⭐ E a folha ganhou o **CONTROLO** que prova que a régua contém o fenómeno
(`o_controlo_a_autoria_espelhada_de_facto_se_cruza`): com a autoria **espelhada** as duas TROCAM de
ordem ao longo do varrimento. ⛔ Mais `prende_os_cantos_da_a_regra_da_esquerda_a_contagem`, o elo do
meio — *uma lei verificada nas duas pontas ainda pode ser contrariada no meio*.

### ⛔⛔ E a FOTO apanhou mais DOIS defeitos, com todos os gates verdes

1. **Com a timeline ABERTA o HUD cai atrás do painel.** Medido numa janela `1930×1012`: o botão é
   alcançável na caixa de ecrã `y 728..848` e o painel começa em `~720` ⇒ a sonda do produto lê
   **`on_canvas=false (painel=Some(true))`** e o gesto do dono **nem chega ao ramo do HUD**. A cena
   passa a **FECHÁ-LO** — ⚠️ e não a «deixar de o abrir»: a arrumação vive em `~/.ph2d/layout.txt`,
   FORA do repositório. Fechado, a mesma sonda lê `on_canvas=true` e o clique é consumido nos dois
   lados.
2. **`Pontos: 16` passava por baixo do Inspector.** ⚠️ O recurso **não** é a caixa de referência
   (`±16`, e os rótulos cabem: `Pontos: 1230` tem meia-largura `3,391`) — é o **sub-rectângulo entre
   os painéis**, que é o item aberto do #20. `DENTRO` desce de `7` para `5` (a ponta do placar ia a
   `6,29` de mundo contra uma borda de canvas em `5,84`; passa a `5,31`), **com a calibração e o
   limite dela escritos ao lado**: as larguras dos painéis não vivem no repositório.

⭐ E o [`fotografa_cena.sh`](../ferramentas/fotografa_cena.sh) ganhou `FOTO_LOG=<ficheiro>`: a janela
do `awk` mostra do 1.º anúncio ao 2.º, logo **todo diagnóstico posterior é invisível** — foi essa
janela que escondeu a auto-conferência, e é a **3.ª vez** que ela custa uma volta.

### ⛔ A catraca da shell, outra vez — e o 1.º candidato foi REVERTIDO

Ela cobrou **114 linhas**. Curada por **CORTE**: o [`hier_group`](../../../crates/ph2d-app-vec/src/hier_group.rs)
(a lei de AGRUPAR/DESAGRUPAR, `238` linhas com os gates) foi para a `ph2d-app-vec` — ele tocava
**só** `ph2d_vec_entities`/`ph2d_ecs`/`ph2d_editor_core`, e o modelo de grupo que ele governa já
vivia lá (ADR-0075). ⚠️ **O 1.º candidato (`morph_set_tests`) foi revertido:** ele declara um irmão
que usa `crate::morph_live` — *um gate que viaja sem o irmão não compila*, e a §5.0 já nomeia a
família.

**Prova de fecho desta metade:** `24 878 / 24 878` verdes (a mesma contagem antes e depois do
move ⇒ nada evaporou) · clippy `-D warnings` a zero · **23 de 23** mutações sangram.

## §10 — O SMOKE

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_HUD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

A contagem (`N.N s`) fica em baixo à **esquerda** e os pontos em baixo à **direita** — ⚠️ desde
2026-09-20, porque até aí cada uma estava autorada do lado OPOSTO àquele a que se prende (§8-ter).
**Arraste a borda da janela** para a alargar: as duas seguem as bordas reais e **nunca se cruzam**.

⭐ E o **CONTROLO**: na secção *HUD* do Inspector troque **`Fit`** de `Expand` para `Keep` e alargue
outra vez — agora elas param na **área segura**, a uma banda da borda. *Os dois modos existem de
propósito, e é o que o alvo faz.*
