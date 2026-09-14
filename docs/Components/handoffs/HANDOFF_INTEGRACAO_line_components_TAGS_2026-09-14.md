# Handoff de integração — `line/components` · **TAGS** (TOP-20 #9), 2026-09-14

> **Estado:** a linha FECHOU. ⛔ **Não integrada, não enviada** (`CLAUDE.md` §0.7).
> **Merge-base:** `1d43da737` · **12 commits** · 228 ficheiros · +11 882 / −770.
> **Plano:** [`docs/Components/08_plano_tags.md`](../08_plano_tags.md) (W1→W4, as quatro fechadas).

---

## §1 — O que o artista passa a conseguir fazer

1. **Marcar um objecto** com uma ou mais tags, no Inspector (secção *Tags*: chips com `×`, caixa de
   escolha com busca que dobra maiúsculas e acentos, e `+ Create "…"` que cria e marca num gesto).
2. **Mandar um sinal acertar em TODOS os que pertencem a uma tag** — uma linha de *Signal Actions*
   no lugar de uma por nome, e ela alcança a **subárvore**.
3. **Pôr uma armadilha a disparar só para quem tem certa tag** (*Only for tag*, na §11 Physics).
4. **Editar a taxonomia**, no painel *Tags* novo: criar, aninhar, renomear, **arrastar para dentro
   de outra**, apagar (com a subárvore e a pertença, num passo de `Ctrl+Z`) e **escolher na cena**
   todos os que pertencem.

**Smoke:** `PH2D_TAGS_SMOKE=1|2` — as duas cenas foram **corridas no binário** antes deste handoff
(§7).

---

## §2 — Os contadores, em DELTA (⛔ nunca o literal — `CLAUDE.md` §5.0)

| contador | base (`1d43da737`) | a linha | delta |
|---|---:|---:|---:|
| `PROJECT_SCHEMA` | 128 | 129 | **+1** |
| a tripla de `project_schema_tests` | `(128, 13, 22)` | `(129, 13, 22)` | **+1 / 0 / 0** |
| registo do `ph2d-ecs` (`registry_tests`) | 85 | 86 | **+1** (o `Tags`) |
| espelho do `ph2d-render` | 86 | 87 | **+1** |
| espelho do `ph2d-script` | 86 | 87 | **+1** |
| `widget::*_SCROLLBAR_ID` | máx. 846 | **847** (`TAGS_SCROLLBAR_ID`) | **+1** |

⚠️ **O `PROJECT_SCHEMA` tem migração** (o v128 lê, e toda linha de `SignalActions` nasce
`SignalTarget::Named`). ⚠️ **O `FIELD_DOC_VERSION` e o `VEC_SCENE_SCHEMA` não se mexem.**

⛔ **Catracas que DESCERAM neste fecho** (e a razão está no commit):

| catraca | antes | agora |
|---|---:|---:|
| `the_app_only_sheds_fields::TETO_CAMPOS` | 187 | **183** |
| `fn_loc_caps` `main.rs::new` | 237 | **233** |

⚠️ **E uma que ficou onde estava, de propósito:** `architecture_the_foundation_modules_form_a_dag`,
aresta `action_bus → screens`, tecto **24**. Sem a cura desta wave ela teria ido a **26**.

---

## §3 — Crates novas

| crate | o que é |
|---|---|
| [`ph2d-label-fold`](../../../crates/ph2d-label-fold/) | a DOBRA (ICU): maiúscula e acento não importam (decisão D2 do dono) |
| [`ph2d-label-path`](../../../crates/ph2d-label-path/) | a álgebra de caminhos (`Enemy/Flying`), sem ECS e sem UI |
| [`ph2d-tags`](../../../crates/ph2d-tags/) | a `TagTree` — identidade (`TagId`) + árvore, o modelo do **Blender** (decisão D1) |
| [`ph2d-panel-tags`](../../../crates/ph2d-panel-tags/) | o painel docado (categoria MUNDO, nasce fechado) |

E um módulo novo na fundação: [`ph2d_editor_core::tags_edits`](../../../crates/ph2d-editor-core/src/tags_edits.rs)
— o **vocabulário** das tags, abaixo do `action_bus` e do `screens`. Ver §5.3.

---

## §4 — Os SETE achados, cada um apanhado por um instrumento diferente

### 4.1 ⛔⛔ `tagged` respondia «NINGUÉM» num mundo que nunca viu um `StableId`

O `try_query` do `bevy_ecs` devolve `None` quando **qualquer** componente da consulta é desconhecido
do mundo — e um `Option<&StableId>` conta. Um sinal dirigido a uma tag **não alcançava nada, e nada
o dizia**. A identidade passa a ser lida **por ACERTO** (`world.get::<StableId>(e)`), fora da query.

⚠️ **O gate irmão não continha o fenómeno:** a fixtura dele spawna *com* `StableId`, porque é ela
que prova a ordem. A fixtura nova spawna *sem*, e tem o controlo (`all(|e| e.get::<StableId>().is_none())`).

### 4.2 ⛔⛔ O id da barra de rolagem nasceu do lado do PAINEL

Ali ele é invisível ao despacho: o `scrollbar_panel_for_id` não o mapeia, o `begin_scrollbar_drag`
nunca arma, e **o polegar fica pintado e impossível de agarrar**. Desceu para
`widget::TAGS_SCROLLBAR_ID` (**847**, contado) com a entrada no mapa. *O dono de um id de barra é o
DESPACHO, nunca quem a desenha.* ⚠️ É a mesma auditoria que o `INPUT_MAP_SCROLLBAR_ID` já pagou.

### 4.3 ⛔⛔ As LINHAS do painel nasciam MORTAS sob o dedo

A 1.ª redacção copiou o molde da Hierarquia (a shell regista as linhas por quadro). Ali os ids são
**alocados** pela shell (`EntityNodeMap`); aqui são **derivados** da identidade da tag, logo
**quem as pinta é quem as regista**. Um registo do outro lado da fronteira é um registo que alguém
esquece — e foi o gate de costura com **ponteiro real** que o disse (um `WidgetEvent::Click`
sintético passa com a linha morta).

### 4.4 ⛔⛔ SETE gates de arquitectura estavam vermelhos desde a W3, em silêncio

O portão daquela wave correu `-p ph2d-panel-inspector` e `-p ph2d-host-desktop`; estes vivem em
**`ph2d-editor-core/tests/it/`**. É a **quinta** ocorrência registada da mesma cegueira
(memória `feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests`), agora com uma volta a
mais: *o gate mede uma crate que a linha editou e vive noutra que ela não correu.*

Todos curados por **CORTE**, nunca por isenção nem por subir número:

| gate | o que acusou | a cura |
|---|---|---|
| `panel_files_under_loc_cap` | `physics_rows` 677 · `paint_frame` 606 · `state` 601 (tecto 600) | 3 irmãos por ASSUNTO |
| `panel_functions_under_loc_cap` | `paint_deferred_popovers` 210 (tecto 200) | os 3 popovers de tag saem |
| `the_foundation_modules_form_a_dag` | `action_bus → screens` iria a 26 (tecto 24) | o `tags_edits` (§5.3) |
| `hr15_no_hardcoded_ui_strings` | 7 literais | a tabela do `ph2d-i18n` (§4.5) |
| `no_magic_numeric` | o `2.5` do chip | `// LITERAL-PX-OK` com a razão |
| `the_gap_between_two_rows_is_one_answer` | 2 sítios | `row_pitch_px()` |
| `the_tail_of_a_block_is_one_answer` | 2 sítios | `control_gap_px()` |

### 4.5 ⭐ `Pick a tag…` estava escrito em TRÊS pintores

E `Only for tag…  (any)` em dois. *Uma palavra escrita em dois sítios ainda não é uma palavra do
app — só uma PORTA é* (a lei que o `chrome.rs` do `ph2d-i18n` já escreve, com quatro duplicados
medidos). Colapsados em [`ph2d-i18n/src/tags.rs`](../../../crates/ph2d-i18n/src/tags.rs).

⛔ **As FRASES DE RECUSA ficam FORA daquela tabela, e a ausência é a decisão:** elas nascem em
`ph2d_tags::TagError::message`, ao lado da lei que as produz, e atravessam o painel **sem serem
interpretadas**. Duplicá-las no i18n daria duas frases para a mesma recusa.

### 4.6 ⭐⭐ O arrasto de linha de painel tinha DUAS cópias, e esta ia ser a terceira

A Hierarquia (que **muta a árvore no próprio despacho**) e as camadas do Painter. Em vez da terceira
cópia, a máquina generalizou-se: `PanelRowFamily` · `PanelRowDrop` · `PanelRowReparent { family, … }`
· `store.set_panel_row_ids(family, …)` · `panel_row_drag()`. ⛔ **A Hierarquia fica FORA** da família
— o arrasto dela tem semântica de irmãos e mutação no despacho; estas só resolvem geometria.

⚠️ **UM slot de arrasto para todas as famílias**, e é de propósito: só um arrasto pode estar vivo de
cada vez (há um ponteiro). ⚠️ E o painel de camadas ganhou o filtro por família na leitura do
fantasma — sem ele, arrastar uma tag desenharia o fantasma das camadas ao lado.

### 4.7 ⛔⛔ Um gate que RE-DERIVA o valor que devia LER não mede o produto

A mutação que apontava a armadilha da `=2` à tag **errada** SOBREVIVEU: o gate
`the_trap_lets_the_hero_through_and_ignores_the_goblin` escrevia `let filtro = TagId(a.player.0)` —
o valor da FIXTURA — em vez de o ler do mundo. Ele media a lei da árvore e **não a fiação da cena**.
Curado: o filtro sai de uma consulta ao mundo, com o `assert_eq!` a dizer que é o esperado.

### 4.8 ⛔⛔ Uma lei com DOIS guardas não tem mutação de UM sítio

`«um id órfão não conta para ninguém»` é garantida duas vezes — o `ancestry` devolve vazio para um
id que a árvore não conhece, **e** o mapa de saída só tem as chaves de `tree.tags()`. Apagar
**qualquer um** deles deixa o gate VERDE; foram precisas **três** redacções para achar a alavanca
(escrever o id **directo** no mapa, saltando os dois). ⇒ *uma mutação sobre um guarda defendido pelo
outro mede a redundância, não a lei*. Os dois ficam, com a nota ao lado.

### 4.9 ⛔⛔ `git commit -- <paths>` NÃO apanha um ficheiro por rastrear

O commit da W4a disse **54 ficheiros** e a árvore dele **não compilava**: os **11 ficheiros NOVOS**
daquela wave ficaram de fora. O `-- <paths>` é um filtro sobre as mudanças **rastreadas** (o mesmo
`pathspec` do `git add -u`), e um `??` só entra se alguém o **stage** antes — e a regra §0.4 do
`CLAUDE.md` (*«nunca `git add .`»*) empurra exactamente para esta forma, porque ela é a segura
contra ficheiro alheio. Curado por `--amend` (o commit era `HEAD`): **65 ficheiros**.

⭐ **A prova barata é `git status --short | grep '^??'` DEPOIS do commit** — se ainda sobrar um
ficheiro da wave, ele ficou de fora.

### 4.10 ⚠️ A `=2` precisava do TRANSPORTE armado pela shell

Sem `timeline.flags.simulate_physics = true` e `playhead.play()`, os dois corpos ficam pendurados no
ar e o artista lê *«a armadilha não dispara para ninguém»* — o **veredito errado sobre um filtro que
funciona**, que é exactamente o que o `CLAUDE.md` §5.0 proíbe. ⚠️ E a **RÉGUA** abre junto, pelo
motivo que a cena 67 da física pagou em produto (*«que régua?»*).

---

## §5 — Cinco coisas que uma leitura rápida do diff entende ao contrário

### 5.1 A porta rápida da contagem **PERDE** quando a árvore é pequena, e isso não é um defeito

`counts` custa `0,88 ms` contra `0,42` do laço com 10 tags, e `2,97` contra `21,19` com 584. *Ela não
é «a versão rápida» — é a que não EXPLODE*: troca um produto `tags × objectos` por uma soma. A tabela
inteira está no doc-comment dela.

### 5.2 A contagem só é derivada com o painel **ABERTO**, e isso não é preguiça

No extremo ela custa `2,97 ms` contra um quadro de `16,7`. Fechado, o painel recebe o instantâneo
antigo e **não pinta nada** (o `paint` sai na primeira linha).

### 5.3 O `TagsFieldEdit` **MUDOU DE MÓDULO**, e não é arrumação

Ele e o `TagTreeEdit` vivem em `crate::tags_edits`, que é **o módulo de vocabulário que a catraca do
DAG prescreve por escrito** na própria entrada dela. ⚠️ As vinte irmãs (`TimerFieldEdit`, …) **ficam
onde estão**: esta é a primeira a descer, e o resto da migração é de quem a pagar.

### 5.4 `Before`/`After` e `Inside` são **DUAS** respostas, não três

A `TagTree` ordena-se sozinha pela chave dobrada, então *«antes da Flying»* e *«depois da Flying»*
significam os dois **irmã da Flying**. Fingir três destinos daria ao artista um gesto cujo efeito ele
não consegue ver.

### 5.5 Os seis latches das cenas viraram uma struct porque **o gate pediu**

A falha do `the_app_only_sheds_fields` diz, no próprio texto, *«um campo novo tem DONO: ponha-o no
estado da família do assunto dele»*. ⚠️ Eles continuam na **shell** e não na crate: um latch dentro
da família seria ela a ter opinião sobre **quando** o quadro a chama.

---

## §6 — As premissas que a medição derrubou

1. **«O painel regista as linhas como a Hierarquia»** — ali os ids são alocados pela shell; aqui são
   derivados. O molde não servia, e o gate de costura disse-o (§4.3).
2. **«A fixtura do gate da consulta cobre o caso»** — ela spawna *com* `StableId`, que é exactamente
   o que escondia o defeito (§4.1).
3. **«Um id de barra é do painel»** — é do DESPACHO (§4.2).
4. **«A `line/components` só toca em crates que corre»** — sete gates noutra crate diziam o contrário
   (§4.4).
5. **«Largar uma raiz na raiz prova o filtro de família»** — não prova: é o no-op que o gate vizinho
   mede, e ele esconderia a diferença entre *«a família filtrou»* e *«o gesto não mudou nada»*. A
   fixtura teve de trocar de sujeito.
6. **«31 mutações, todas à primeira»** — três não. Duas sobreviveram e uma tinha a âncora errada; as
   duas sobreviventes eram achados sobre os GATES, não sobre o produto (§4.7 e §4.8).

---

## §7 — O smoke, CORRIDO no binário antes deste handoff

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components
env PH2D_TAGS_SMOKE=1 PH2D_SIGNAL_LOG=1 cargo run -p ph2d-host-desktop --profile smoke
```

```text
[tags-smoke] =1 Enemy(5) > Flying(3) > Boss(1) · Statue(1) · Player(1) — aos 2 s somem CINCO
[signal] 5 accao(oes) aplicada(s), 0 inerte(s)
[signal] alarm <- timer do objecto 4294967287, 1 periodo(s)
```

```
env PH2D_TAGS_SMOKE=2 PH2D_SIGNAL_LOG=1 cargo run -p ph2d-host-desktop --profile smoke
```

```text
[tags-smoke] =2 armadilha *Only for tag* `Player`: o Goblin atravessa (nada), o Hero abre a porta
[signal] 1 accao(oes) aplicada(s), 0 inerte(s)
[signal] trap <- fisica, 4294967293 tocou 4294967290
```

⭐ **`5` na `=1` e `1` na `=2`** — os dois números que as cenas existem para produzir.

---

## §8 — O que fica ABERTO

- ⏳ **O painel não mostra o CAMINHO inteiro num balão.** A linha mostra o rótulo com recuo; um
  `Enemy/Ground/Flying` lê-se pela indentação. O balão existe no Inspector (nos chips) e não aqui.
- ⏳ **Não há busca no painel.** Com dezenas de tags a lista rola; com centenas, rolar é o único
  caminho. A caixa do Inspector já tem busca dobrada — o painel herdaria a mesma porta.
- ⏳ **O `Ctrl+Z` de um gesto do painel não foi medido por gate de ponta a ponta.** A árvore viaja no
  `ProjectState` (gate 18 da W2) e o apagar leva a pertença no mesmo passo (gate da W4), mas o
  *arnês* que carrega no `Ctrl+Z` depois de um arrasto não existe.
- ⛔ **A Hierarquia continua fora da `PanelRowFamily`** — por desenho (§4.6), e mudá-la é uma wave
  com gates próprios.
- ⚠️ **ACHADO NUMA CRATE ALHEIA, não curado:** o `ph2d-panel-skeleton` pinta a barra de rolagem dele
  com o `VECTOR_SCROLLBAR_ID`, que o `scrollbar_panel_for_id` mapeia ao **painel de vetor**. Com o
  painel de vetor fechado, `panel_content_h(VECTOR_PANEL)` é `None` e **o arrasto do polegar dele
  nunca arma**. É o mesmo defeito do §4.2, noutra linha; a cura é um id próprio (`848`) e a entrada
  no mapa. ⛔ Não foi tocado aqui: é território de outra linha.

---

## §9 — Onde ler

- [`08_plano_tags.md`](../08_plano_tags.md) — o plano, os dois oráculos corridos, as 4 decisões do dono.
- [`ferramentas/mutacao_tags_w3.sh`](../ferramentas/mutacao_tags_w3.sh) (32, todas sangram) · [`_w4.sh`](../ferramentas/mutacao_tags_w4.sh) (**31**, todas sangram) — versionadas, com a razão de cada redacção corrigida escrita ao lado dela.
- `ph2d_ecs::tags::counts` — a tabela da medição.
- `ph2d_editor_core::tags_edits` — porque o vocabulário desceu.
