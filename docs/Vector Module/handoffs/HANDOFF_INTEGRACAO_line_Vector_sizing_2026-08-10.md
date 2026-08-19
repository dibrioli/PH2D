# HANDOFF DE INTEGRAÇÃO — `line/Vector` · o SIZING (2026-08-10)

**Status:** FECHADO 2026-08-10 · no `main` em `198200a11` (o commit que trouxe este arquivo).

> Para o agente **integrador**, por ordem do Enio (DIRETRIZ §1.5.9). A linha está **fechada**;
> ela **não** integra e **não** faz ship.

---

## §0 — O que esta linha entrega, em três linhas

Os **itens 1-4 do estudo dos contêineres**, na ordem em que ele os mediu:

1. **O SIZING** — o vocabulário de tamanho do Figma (**Fixed · Hug · Fill · Min · Max ·
   Absolute position**), a única metade em falta do auto layout.
2. **A ROLAGEM** — uma moldura que recorta e cujo conteúdo não cabe **rola com a roda**.
3. ⭐ **A TABELA SINAL → PAPEL** — um nome gritado noutro lugar (um marker, um contato, um botão)
   **move a cena**. Era o consumidor que o R0 deixou por fazer.
4. ⭐ **A GRADE** — o item 5, o último da tabela do estudo. Ela honra os quatro degraus (motor ·
   documento · fatia · painel), e com isso **mata o argumento vivo do ADR-0153**.

⚠️ Ela nasce de um **estudo medido** (`docs/Vector Module/Estudos/ESTUDO_containers_e_catalogo_minimo_de_UI_2026-08-10.md`),
que o Enio pediu para decidir se faltavam `VBoxContainer`/`HBoxContainer`/`Grid`. O veredito:
os dois primeiros **não faltam** (são `LayoutDir::Row`/`Column`, já shipados), o que faltava de
verdade era o sizing — e o Grid, que o estudo classificou como *dispensável mas não descartado*,
fechou a lista no fim da jornada.

⚠️ **E o estudo tinha um NÚMERO ERRADO que esta jornada corrigiu no próprio estudo:** ele declarava
o custo de build do `grid` em *"+11 ms, razão 1,07×"* e o argumento do ADR **morto**. Re-medido com
a máquina calma, o `+11 ms` é impossível — a metade maior (~440 ms) é o módulo de grid do próprio
`taffy`, então um A/B que limpe só a nossa crate mede a nossa crate duas vezes. O argumento morre
por outra via, e mais honesta: **~0,47 s absolutos, uma vez por build limpo, 0,03% de uma corrida
de CI** — o `cargo check -p` do inner loop nunca os paga.

---

## §1 — Identidade

| | |
|---|---|
| Branch | `line/Vector` |
| HEAD | `590effb20` |
| Base | `76788440a` (o `main` de 2026-08-10) |
| Commits | **20** |
| Diff | **76 arquivos, +6964/−219** |
| Smokes aprovados | `=66` (sizing) · `=67` (rolagem) · `=68` (a tabela) — ⚠️ **`=69` (a grade) PENDENTE** |

---

## §2 — ⚠️ Superfície de COLISÃO (o que o integrador tem de conferir)

| item | valor nesta linha | nota |
|---|---|---|
| **`PROJECT_SCHEMA`** | **70 → 72** ⚠️ **PROVISÓRIO** | dois degraus: **v71** `HostStates.on_signal` (a tabela sinal → papel) e **v72** `LayoutDir::Grid` + `VecLayout::columns`. ⚠️ **O valor se CONTA contra o `main` do DIA da integração**, nunca se escolhe: nesta janela há outras linhas vivas, e esta colisão passa **muda** quando as duas escrevem o mesmo literal — confira os DOIS arquivos (`project.rs` **e** `project_schema_tests.rs`, tripla `(72, 13, 14)`). ⚠️ E os dois degraus **já estão escritos na escada**, a lição que o v69 pagou |
| **`VEC_SCENE_SCHEMA`** | **INTOCADO** (14) | |
| **Registro do `ph2d-ecs`** | **55 → 57** | ⚠️ **e os DOIS espelhos 56 → 58** (`ph2d-render`, `ph2d-script`) — o contador é **TRÊS casas**, cada uma na suíte da própria crate. A grade **não** o move (ela não traz componente novo) |
| Componentes novos | `VecLayoutSize` · `VecLayoutAbsolute` | |
| **`PanelEvent`** (contrato CONGELADO) | **4 variantes, INTOCADO** | ⚠️ mas o **doc** do `SelectOption` foi corrigido: ele dizia *"RadioGroup option selected"* e isso é falso desde que o Painter carrega `"layer:channel:index:x:y"` nele. É o comentário que mandaria a próxima wave gastar um ADR num variante que este já dá. Contagem intacta, gate verde |
| **Contrato congelado** | **intacto** | `git diff` vazio em `ph2d-nodegraph` e `ph2d-core/src/tool.rs`; os seis arch-gates rodados verdes |
| **ADR** | **nenhum NOVO** ⇒ a linha fica **FORA de toda disputa de número** | ⚠️ mas o **ADR-0153 é EMENDADO** (Emenda 1): a recusa do grid caiu, e o número novo que entrou no lugar do velho é o teto de faixas do `taffy`. Se outra linha desta janela tocar o 0153, é um merge de PROSA, não de decisão |
| **`Cargo.toml`** | **UM** (`ph2d-vec-layout`) | a feature `grid` do `taffy`. ⚠️ **`Cargo.lock` INTOCADO** — o lock é agnóstico de feature para deps opcionais de uma dep, então a crate `grid` já lá estava e **nenhum pacote novo entra** |
| **Cenas de smoke** | `=66` · `=67` · `=68` · `=69` | ⚠️ **próxima livre: 70**; o gate `no_two_smoke_scenes_claim_the_same_level` a tranca |
| ⚠️ **Módulo de nome parecido** | `signal_table_smoke.rs` | **NÃO** é o `signal_smoke.rs` (o `PH2D_SIGNAL_SMOKE` do R0): ali o assunto é a SAÍDA, aqui é o CONSUMIDOR. Eu escrevi por cima do primeiro por engano e restaurei — o nome curto estava tomado |
| Ids novos | **11**, todos `hash_node_id` | fora de todo contador |
| `MAX_FX_KINDS` · scrollbar id · `WidgetKind` | **intocados** | |

⚠️ **E há um SEGUNDO ponto de merge, este na shell:** `shells/desktop/src/layout_live.rs` bateu
605 > 600 e foi PARTIDO — `Box2`, `Reading` e `FlowSlots` mudaram-se para o irmão
`layout_live_slots.rs`. Uma linha que acrescente um campo ao `FlowSlots` **funde limpa contra um
arquivo de onde a struct saiu** (o modo de falha exacto que o corte do `project.rs` produziu em
04/08 e o do `ph2d-i18n` em 10/08). Os `pub(crate) use` mantêm todo caminho de chamada.

⚠️ **O ponto de merge sensível é UM:** `crates/ph2d-vec-layout/src/lib.rs` teve o campo
`Node::size` **trocado de tipo** (`[f64; 2]` → `[Len; 2]`) e ganhou `min`/`max`. Uma linha que
construa um `Node` não compila até acrescentar `..Node::default()`. Há **16** construções na
árvore, todas em testes desta crate, na sonda e na fatia da shell — todas já convertidas.

---

## §3 — O que foi construído, e a lei de cada peça

### 3.1 O motor (`ph2d-vec-layout`)

- **`Len::{Fixed(f64), Hug}`** — enum e não `f64` + `bool`, porque as duas respostas são
  **exclusivas**: um eixo que abraça não tem um número que alguém escreveu.
- **`min`/`max` por eixo.**
- **`LayoutError::HugWithoutFlow`** — uma FOLHA que pede para abraçar é **recusada**. ⚠️ Acomodar
  seria pior que falhar: sem filhos o conteúdo mede zero, e a forma **desapareceria sem erro**.
- O **espaço oferecido à raiz** passa a ser a pergunta do abraço (`MaxContent` × `Definite`).

### 3.2 O documento (`ph2d-ecs`)

- **`VecLayoutSize { size, min, max }`** — do **NÓ**, sobre si. ⚠️ O `LayoutSize::Fixed` do
  documento **não carrega número**, ao contrário do motor: o número de uma forma é a **geometria
  dela**, e guardá-lo aqui seria a segunda resposta a *"que tamanho tem esta moldura?"*.
- **`VecLayoutAbsolute`** — marcador; a presença é o booleano (o idioma do `Ccd` da física).

### 3.3 A fatia (`shells/desktop/src/layout_live.rs`)

- **`size_of`** é porta única, irmã do `frame_style`, e **recusa o abraço a quem não flui** (cai
  para o tamanho medido).
- **O fora-do-fluxo sai da FATIA, não do motor.** Um nó que o motor nunca vê fica com a pose
  autorada e continua a andar com o pai e a ser recortado por ele. Dizê-lo ao motor pediria um
  *inset* — quatro números derivados que ninguém autorou.
- **A RAIZ entra no laço de colocação**, porque um `Hug` muda o tamanho dela e o tamanho novo tem
  de ser **desenhado**. Sem abraço o afim sai identidade e o `is_identity` salta-a antes de
  qualquer cópia ⇒ **byte-intocado** para quem nunca pediu abraço.
- ✅ **O `clip` veio de graça:** o `frame_clip` já lê a `LiveGeometry`, então a moldura que encolhe
  leva o recorte junto.

### 3.4 A UI (`ph2d-panel-vector`)

**Width/Height: Fixed | Hug** (um par por eixo) · **Min/Max** (zero = ausência) · **Absolute
position**, que **esconde Grow/Shrink** — quem saiu do fluxo não reparte sobra nenhuma.

---

### 3.5 A ROLAGEM (item 3) — `layout_live::scroll` + `layout_scroll_gesture`

**A sonda decidiu o desenho antes de qualquer código de produto** (`ph2d-vec-layout::overflow_probe`,
pela porta do PRODUTO): os filhos **transbordam** em vez de encolher (`y = 0/40/80/120/160` numa
moldura de 100), o par `Hug + Max` transborda igual, e o **controle** (moldura de 400) reporta
−200 ⇒ o excedente é derivável, e rolar é deslocar os filhos por um número.

- **O excedente é DERIVADO**, nunca autorado — um knob ao lado discordaria do primeiro filho novo.
- **O deslocamento é VISTA, não documento:** o undo deste editor é por DIFF do mundo, então um
  scroll no ECS faria **cada tique de roda virar um passo de undo**. Preço honesto: ele não viaja
  no arquivo, e reabrir mostra o topo da lista.
- **A roda não rouba o zoom:** só uma moldura que **recorta** *e* **transborda** é alvo.

### 3.6 ⭐ A TABELA SINAL → PAPEL (item 4) — `ph2d-ui-state::binding` + o frame

O R0 deu ao app uma saída com três produtores e **um** consumidor (um toast). O que faltava é a
ligação, e ela é conteúdo autorado: `HostStates.on_signal`.

**Três decisões, todas DERIVADAS:**

1. **Ela mora dentro do `HostStates`**, não numa tabela global: o `retain_hosts` já corre por
   frame ⇒ uma forma apagada leva as ligações dela **sem uma linha a mais**.
2. **A única ação é *ir para um papel*, e o limite é o que a preview consegue DESFAZER** (ela
   captura a pose de todo id mencionado em qualquer estado). Uma ação nova **traz consigo a
   metade que a desfaz, ou não entra**.
3. **O cursor anda sempre; só a AÇÃO é gateada na preview.** ⚠️ E isto **não** contradiz o botão
   *Show*: a diferença é *quem pediu*.

⚠️ **A metade de PRODUTOR não custou campo nenhum:** na preview, um clique completo sobre um
hospedeiro publica o **`Name` da entidade**.

---

### 3.7 ⭐ A GRADE (item 5) — `Dir::Grid` + a régua que lê em LINHAS

**Quatro degraus, e é o quarto que mata o argumento do ADR-0153:** motor
(`Dir::Grid { columns }` → `Display::Grid` + `repeat(N, 1fr)`) · documento (`LayoutDir::Grid` +
`VecLayout::columns`) · a fatia (o `match` exaustivo do `layout_live_style`) · o painel (o 5º chip
+ a row **Cols** + o Justify estreitado).

**Três decisões, e as três saíram de medição:**

1. **O teto de faixas é `i16::MAX = 32767`, e ele morde pelas LINHAS.** Bissectado contra o motor:
   1 coluna → 32767 filhos, 2 → 65534, 3 → 98301 — sempre 32767 linhas, a largura do
   `OriginZeroLine(i16)` do `taffy`. ⚠️ Um cap na contagem de COLUNAS teria sido *taste* **e não
   teria evitado o pânico**: as linhas ninguém autora, elas nascem da contagem de filhos. Acima do
   teto o `solve` **recusa** (`LayoutError::GridTooManyTracks`) — acomodar ali não dá uma pose
   errada, dá um `unwrap` a estourar dentro do `taffy` e o app a cair.
2. **A contagem é um CAMPO do `VecLayout`, não o corpo do variante.** Com ela no variante, ir a
   `Row` e voltar **destruiria** o número; como campo ela sobrevive à ida e à volta, como o vão e o
   recuo — e o `DIRS` da shell continua a ser **uma** tabela lida nas duas direções por igualdade.
3. **O `align` governa DUAS propriedades do CSS.** Sem o espelho em `align_content` a grade herda
   `align-content: normal`, que numa grade **é** `stretch`: as linhas seriam esticadas para encher
   a moldura, e a mesma cena que um `Row` encosta no topo sairia espalhada. ⚠️ O gate vivo nasceu
   **VERMELHO** exactamente aí (topo 17,5 onde a aritmética pede 25).

⚠️ **NARROWING, e ele é geometria:** numa grade as duas DISTRIBUIÇÕES não são oferecidas — com
colunas iguais não sobra espaço horizontal para repartir. O valor guardado **não é reescrito**: um
documento vindo de um `Row` com *Between* pinta a fileira sem nada aceso, e volta intacto.

⚠️ **E o GESTO de reordenar era ERRADO numa fila em linhas, não impreciso.** Medido antes de uma
linha ser escrita: numa grade 3×3 as três células da coluna 0 partilham o mesmo `x`, então
*"quantos centros estão antes do cursor"* devolvia o **slot 3** para uma soltura na célula (0,0), e
**6** para a última. O `RowWrap` shipa com o mesmo defeito **desde que nasceu**
(`main_x = !matches!(dir, Column)`); ficou invisível ali porque uma faixa de wrap raramente alinha
duas linhas. A régua passa a publicar a **CAIXA** de cada filho (o facto cru) e o `FlowSlots` ganha
`Reading::{RowX, ColumnY, Rows}` — **esta wave conserta os dois**.

⚠️ **MEDIDO e NOMEADO, não escondido:** `Align::Stretch` **não estica** um filho que traz tamanho
explícito — e isso **não é divergência da grade**, é a mesma lei do `Row` (6,0 nos quatro casos
medidos). Ele é vivo para um filho `Hug`, e ali vale nos dois.

---

## §4 — ⚠️ O que o integrador deve levar aos DOCS (não é código)

**O ADR-0153 JÁ ESTÁ EMENDADO** (Emenda 1), e o integrador não tem nada a escrever — só a
conferir que o merge da prosa fundiu limpo. O que a emenda diz:

⚠️ **Uma versão anterior deste handoff mandava o integrador anotar que o custo de build era
`+11 ms (1,07×)` e que o argumento do ADR estava MORTO. O número era IMPOSSÍVEL.** Re-medido com a
máquina calma e **decomposto** — que é o passo que faltava:

| o quê | ms |
|---|---|
| sem `grid` (taffy + a crate, cold) | 282-295 |
| a crate `grid` sozinha | 304 |
| taffy + a crate, com `grid` **já quente** | 720-748 |
| tudo cold, com `grid` | 760-1151 |

A metade **MAIOR** (~440 ms) é o **módulo de grid do próprio `taffy`**, logo um A/B que limpe só a
nossa crate e deixe o `taffy` quente **mede a nossa crate duas vezes e nada mais**. O número do ADR
(0,20 → 0,63 s) estava **essencialmente certo**; foi a refutação dele que estava errada — e ela
esteve escrita aqui e no estudo por uma jornada.

⇒ **O argumento morre, mas por outra via:** 3× sobre 0,28 s são **~0,47 s absolutos, uma vez por
build limpo**, contra ~30 min de CI (**0,03%**) — e o `cargo check -p` do inner loop nunca os paga.
*Uma razão sobre uma base minúscula não é um custo; o ADR escreveu uma razão e tratou-a como
orçamento.*

⚠️ **E um número NOVO entrou no lugar do velho**, este medido por bisseção contra o motor: o
`taffy` **PANICA** acima de **32767 faixas** por eixo. Ele está no ADR, na const
`MAX_GRID_TRACKS` e num gate.

---

## §4b — ⚠️ As DUAS rodadas de smoke do Enio, e o que elas acharam

As duas foram sobre o **mesmo controle** (*Absolute position*), por causas **diferentes** — a
segunda só ficou visível depois de a primeira ser curada.

**(1) *"Não achei Absolute Position no painel do quadrado âmbar"***. O produto estava certo: o
toggle é escondido quando o pai não empilha, e **escondê-lo em silêncio** deixava o artista a
olhar para um painel que não dizia o que faltava. `LayoutItem` ganhou `in_flow`, e o painel passou
a **escrever** *"Set the parent frame to Row or Column first"* (o precedente do Falloff dos Motion
Nodes: *inerte é dito, não omitido*).

⚠️ **A primeira tentativa de cura REMOVIA um caso que funcionava** — trocar o predicado do sujeito
de `VecLayout` para `VecFrame` parece a leitura óbvia e está errada: o passe de layout recolhe os
filhos de **qualquer** entidade com `VecLayout`, tenha ela moldura ou não. Dois gates sangraram na
hora; o predicado é a **união**.

**(2) *"O checkbox não aceita ser checado, pode não estar linkado na UI"***. O id **estava**
linkado em todas as seis pontas da costura (id · `populate` · `paint`+hit · `LAYOUT_CHIPS` ·
`forwards_plain_click` · o roteador da shell), e o gate de seam que o prova estava **verde**. O que
o matava era uma **ORDEM**: o `apply_layout_edit` abria com *"resolve a moldura, ou desiste"*, e
`frame_of_selection` **recusa um filho sozinho** por desenho (o doc-comment do `vec_frame_edit`
di-lo). Todo edit daquela porta é da MOLDURA — **menos este, que é do FILHO** — então o único edit
que é pedido com o filho selecionado era exatamente o único que o guard matava, e o braço que o
honrava vinha depois dele.

⚠️ **A função IRMÃ já fazia certo:** o `apply_layout_field` **não tem guard no topo** — cada braço
resolve o próprio sujeito (`Grow`/`Shrink` pelo filho, `Min`/`Max`/`Gap`/`Pad` pela moldura). Era o
`apply_layout_edit` que estava fora de linha, e é por isso que a cura é mover **um** caso, não
afrouxar o guard: abri-lo para todos faria um `Dir` pedido sobre um filho solto ligar fluxo na
entidade errada (gate irmão `a_frame_edit_asked_with_only_the_child_selected_does_nothing`).

---

## §5 — Gates e mutações

### 5.0 A wave da TABELA (item 4)

| onde | quantos |
|---|---|
| `ph2d-ui-state::binding_tests` | **6** |
| `ph2d-panel-vector/tests/seam_signals` | **4** (clicam com ponteiro REAL) |
| `shells/.../vec_ui_state_signal_tests` | **3** (a porta de autoria) |
| `render_loop::ui_preview::tests` | **4** (o produtor + a restauração) |
| `render_loop::ui_state_bridge::tests` | **1** (a composição) |
| `shells/desktop/tests/the_signal_table_is_wired_into_the_frame` | **2** arch-gates |

**8 mutações, 8 sangram.** ⚠️ **DUAS sobreviveram na 1ª rodada e as duas eram FIXTURE minha:**
soltar no **vazio** deixa `chain.first()` em `None` dos dois lados (o caso que exercita a guarda
do alvo é soltar sobre **outro** alvo), e sem o rato **passar por cima antes do aperto** o `hot`
está vazio no `Down`, então a guarda do alvo devolve `None` sozinha e a do *soltar* fica coberta
por ela — **defesas em camada**, e uma fixture sem o hover mede só a primeira.

⚠️ **Os arch-gates existem porque o consumidor mora dentro do `run_render_frame`**, que precisa de
janela e GPU: a composição tem gate próprio e ele passa com a feature **inteiramente desligada**.

### 5.0b A wave da GRADE (item 5)

| onde | gates | o que carregam |
|---|---|---|
| motor | **8** | a aritmética das células · ⭐ **a grade alinha a coluna 1 e o `RowWrap` NÃO** (o oráculo de *por que a feature existe*, com o wrap como CONTROLE) · o `Hug` sobre `1fr` · os dois vãos · o alinhamento dentro da célula · ⭐ o `align` a governar o BLOCO de linhas · o teto de faixas com o par acima/abaixo · zero colunas · **as três direções flexbox INTOCADAS** |
| seam | **2** | a row *Cols* nasce com o modo que a lê **e** o COMMIT dela chega ao barramento · as duas distribuições somem na grade **E sobrevivem** na linha |
| porta | **2** | a contagem atravessa uma troca de direção · o clamp é `1..=MAX_GRID_TRACKS` |
| vivo | **3** | as células na aritmética exacta com o eixo trocado uma vez · trocar a contagem RE-COLOCA · ⭐ **o passe publica como cada direção se lê** |
| régua | **5** | a repro medida **com o CONTROLE que nomeia o defeito** · o vão entre linhas · as duas leituras 1-D intocadas · o gesto REAL pela porta do produto |
| cena | **2** | a premissa do controle (o wrap parte 3+3) · as larguras são desiguais |

**9 mutações, 9 sangram.**

⚠️ **Três fixtures minhas nasceram erradas, e as três acusaram o produto de um erro que era do
oráculo:** a do vão media entre BORDAS de filhos numa célula maior que eles (18,5 contra 7) · a do
wrap assumia que ele partiria 3+3 como a grade, quando um wrap parte onde a **LARGURA** acaba
(partiu 5+1, e o índice 3 nem era o começo da 2ª faixa) · e a do `align` contou `10` onde o filho
mede `6` (a largura dele, não a altura). As premissas passaram a ser **AFIRMADAS** pelos gates.

⚠️ **E a mutação mais importante SOBREVIVEU à primeira rodada, também por fixture:** o gate do
gesto real **publica à mão** o `Reading::Rows` que ele queria testar, então nunca atravessa o
roteamento. *A régua certa* e *o produto escolher a régua certa* são duas afirmações, e só o passe
responde a segunda — daí o gate novo em `layout_live_tests`.

⚠️ **Uma bisseção minha mentiu por intervalo:** o primeiro teto medido foi **65534** porque o meu
`hi` era 65535, logo `lo` nunca podia passar dele. Eu quase escrevi esse número como o teto.

---

### 5.1 As waves anteriores

| onde | gates | mutações |
|---|---|---|
| motor | **19** (7 novos) | **5 / 5 sangram** |
| fatia | **57** (6 novos) | **6 / 6 sangram** |
| seam do painel | **16** (4 novos) | ponteiro REAL (Down+Up), nunca `Click` sintético |

⚠️ **Uma mutação sobreviveu e a cura foi um gate, não a barra:** trocar `MaxContent` por
`Definite(0.0)` no espaço da raiz passava por tudo. O gate que faltava
(`a_hugging_frame_that_wraps_does_not_wrap`) testa uma afirmação **minha** de doc-comment que
estava escrita sem nada a segurá-la.

⚠️ **E o gate de seam do toggle era VERDE sobre um controle MORTO** (§4b.2), o que nomeia o que
ele pode e o que ele **não** pode provar: *o clique chega ao barramento* é uma afirmação sobre o
PAINEL, e ela continua verdadeira quando quem morre é a porta do outro lado. O par que faltava
mede a outra ponta — **o componente aparece no filho** — e é ele que sangra com a ordem revertida.

---

## §6 — O que SÓ o `ship.sh` pega

- `clippy --all-targets` das crates **não impactadas** pelo meu filtro.
- `machete` / `deny` — **nada a temer**: zero `Cargo.toml` tocado.
- ⚠️ **Rode a suíte também em DEBUG.** Precedente registrado nesta linha (o pânico do
  `ph2d-flip-colorize` só aparecia ali).

---

## §7 — Os smokes

```
cd Worktrees/line-Vector
env PH2D_BUILD_SMOKE=66 cargo run -p ph2d-host-desktop --release   # o SIZING     (aprovado)
env PH2D_BUILD_SMOKE=67 cargo run -p ph2d-host-desktop --release   # a ROLAGEM    (aprovado)
env PH2D_BUILD_SMOKE=68 cargo run -p ph2d-host-desktop --release   # a TABELA     (aprovado)
env PH2D_BUILD_SMOKE=69 cargo run -p ph2d-host-desktop --release   # a GRADE   ⚠️ PENDENTE
```

⚠️ **As quatro cenas imprimem o que montaram, e a linha é a pré-condição:** se `[sizing]`,
`[scroll]`, `[signal-table]` ou `[grid-smoke]` não aparecer, PARE — a autoria não correu e o resto
do roteiro não diz nada.

**O que a `=69` pede:** armar a moldura de CIMA (`Grid` → `Cols` = 3). ⚠️ **O passo que decide é o
2** — as três CORES formam colunas na de cima e ficam ESPALHADAS na de baixo, que é o CONTROLE (o
mesmo conteúdo, já em `Wrap`). Depois: mudar `Cols` para 2 (refluxo), **arrastar o último filho
para a primeira célula** (ele tem de ir para o começo — antes desta wave ia para o meio), e o
Justify na grade tem **três** chips, não cinco. ⚠️ A cena imprime `1 moldura(s) ARMADA(s)`; **se
não for 1, PARE** — a de cima tem de nascer sem layout, senão o smoke pula o passo que ele existe
para provar.

**O que a `=68` pede (o roteiro está na própria cena):** autorar a ligação à mão (`+ Signal` →
digitar `Open` → chip `Pressed`), ligar a Preview, e clicar no botão da esquerda. ⚠️ **O passo que
decide é o 4** — o retângulo da DIREITA (`Plain`) tem as MESMAS duas poses e **nenhuma ligação**;
se ele se mexer junto, o sinal está a mover tudo e eu errei a busca. E o **passo 6**: fora da
Preview o mesmo clique **não faz nada**, de propósito.

---

## §8 — Aberto, com o preço ao lado

- **Escalar deforma o raio.** O `fit` aplica um afim, então uma moldura arredondada que abraça sai
  com cantos ovais. ⚠️ **Não é regressão desta wave** — é o que o `grow` já fazia desde a W2; o que
  esta wave faz é tornar o caso mais comum. A cura é re-cozinhar a `VecShape::Param` no tamanho
  novo em vez de a escalar, e é **wave própria**.
- **Scroll numa moldura** (o item 3 do estudo). `Hug + Max` já cobrem *"cresce até um teto"*; o que
  falta é o que passa DO teto.
- **`Fill` continua exposto como número** (`grow`), e não com o nome do Figma. Renomeá-lo é
  decisão de produto: o número diz mais (repartição em razão), o nome diz melhor.
- **A grade não tem *track sizing* autorado.** As colunas são iguais (`repeat(N, 1fr)`) — o que o
  único consumidor MEDIDO deste repo faz à mão (`paint_catalog.rs`) e o default do Figma. Um
  vocabulário de pistas (`fr` × `auto` × comprimento) é wave própria, e hoje seria um controlo que
  ninguém move. ⚠️ **Corolário:** o `justify_content` de uma grade é **inerte por construção** com
  colunas iguais (não há sobra horizontal), então distribuir as COLUNAS só faz sentido junto com o
  track sizing — é por isso que as duas distribuições são escondidas em vez de mapeadas.
- **`Align::Stretch` não estica um filho de tamanho autorado**, e isto é **anterior a esta wave**:
  medido, o `Row` faz o mesmo. Ele é vivo para um filho `Hug`. Se algum dia o `Stretch` tiver de
  esticar artwork, a cura é no `size_of` (o filho entraria como `auto`), não na grade.
