# HANDOFF DE INTEGRAÇÃO — `line/Vector` **MESTRE** (2026-08-15)

**Status:** FECHADO 2026-08-15 · no `main` em `084c26266` (o commit que trouxe este arquivo).

> Para o agente **integrador**, por ordem do Enio (DIRETRIZ §1.5.9). A linha está **FECHADA**;
> ela **não** integra e **não** faz ship.

⚠️ **Este doc SUPERSEDE o [`HANDOFF_INTEGRACAO_line_Vector_sizing_2026-08-10.md`](HANDOFF_INTEGRACAO_line_Vector_sizing_2026-08-10.md)
apenas como *o que integrar agora*** — o **detalhe de mecanismo** dos itens 1-5 do estudo dos
contêineres continua LÁ, medido e escrito, e **não foi copiado para cá**. Leia-o para o bloco (A).

---

## §0 — O que a linha entrega, em três blocos

**(A) OS CONTÊINERES** — os itens 1-5 do [estudo](../Estudos/ESTUDO_containers_e_catalogo_minimo_de_UI_2026-08-10.md):
o **SIZING** (Fixed · Hug · Fill · Min · Max · Absolute), a **ROLAGEM** de uma moldura, a tabela
**SINAL → PAPEL** (o consumidor que o R0 deixou por fazer) e a **GRADE**.

**(B) A UI VIVA — o [plano](../Estudos/PLANO_UI_viva_2026-08-12.md) INTEIRO.** As cinco waves
numeradas do §6 dele estão feitas: **F0** o substrato · **F1+F2+R1** · as **preferências de
utilizador** · **E1** o scrub numérico · **C1** a corda. Mais o catálogo da wave 6: **F4a** (o
chevron roda) · **F5** (a cascata da paleta) · **E2** (rolagem suave) · **E3** (a paleta de
comandos, Ctrl+K, 62 verbos) · **C3** (o readout que segue a mão) · e a **grade de pixels**.

**(C) A CAUDA DA W6** do [plano 25](../25_plano_ferramentas_de_desenho.md) — o **rótulo de
distância** (W6.6) · o **X/Y numérico do nó** (W6.9) · o **LAÇO** · a **posse** da selecção de nós ·
e o **PICK** a ler o mapa que foi DESENHADO.

⚠️ **A metade (B) é maior do que a lista sugere, e o padrão dela é UM:** *o pintor já sabia, o
estado nunca chegava*. Nove waves seguidas acharam a mesma doença em superfícies diferentes — o
mixer, o painel de áudio (50 botões inertes), o zoom do canvas, os campos e chips, a trilha, o
polegar da barra, o painel gerado, as tags, e por fim as **22 barras de rolagem**. Nenhuma delas
estava no plano: cada uma saiu de um **censo** feito antes de escolher item novo.

---

## §1 — O estado da linha, e o número que mais importa

| | |
|---|---|
| commits (`main..HEAD`) | **92** |
| diffstat (`main...HEAD`) | **397 ficheiros, +30.672 / −3.815** |
| ⚠️ **atrás do `main`** | **203 commits** |

⚠️ **A tabela do §2 foi medida contra o `main` de HOJE, não contra o ponto de fork.** É a lição
que a `line/sculpt3d` pagou em 10/08 (*a tabela de colisão do handoff descrevia o `main` do
fechamento dela*), e aqui ela morde de verdade: **o `main` moveu o `PROJECT_SCHEMA` de ficheiro**.

---

## §2 — A superfície de colisão, MEDIDA (não auto-relatada)

| item | base | **linha** | **`main` de hoje** | veredito |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | 70 | **72** (v71+v72) | **82** | ⚠️ **CONTAR para 83 e 84** — ver §3 |
| tripla do pin | — | `(72, 13, 14)` | `(82, 13, 14)` | segue a mesma aritmética |
| `VEC_SCENE_SCHEMA` | 14 | 14 | 14 | **intacto** |
| `FLIP_SCHEMA` | 13 | 13 | 13 | **intacto** |
| registo `ph2d-ecs` | 55 | **57** | 55 | ✅ os dois degraus pousam |
| os **DOIS espelhos** (`ph2d-render` · `ph2d-script`) | 56 | **58** | 56 | ✅ idem — ⚠️ o contador é **TRÊS** |
| contrato congelado | — | — | — | **intacto** (`git diff` vazio em `ph2d-core/src/tool.rs`, `ph2d-nodegraph/`, `ph2d-vector-doc/`, `ph2d-vector-traits/`) |
| ADRs criados | — | **nenhum** | — | ⇒ a linha fica **FORA de toda disputa de número** |
| scrollbar ids | 841 | **841** | 841 | **nenhum id novo** |
| cenas de smoke | — | **66..73** | máx. **65** | ✅ **sem colisão** |
| i18n | — | edita `vector.rs` | tem `vector.rs` | ✅ o ficheiro **já existe nos dois** (não é add/add) |

**Os dois componentes novos do ECS** são `VecLayoutSize` e `VecLayoutAbsolute` (a wave do SIZING),
e ⚠️ **o `main` não tocou nenhum ficheiro do `ph2d-ecs` que a linha tocou** (interseção vazia).

### `Cargo.toml` — 5 ficheiros, e **nenhum pacote externo novo**

| ficheiro | o que muda |
|---|---|
| `crates/ph2d-spring/Cargo.toml` | ⭐ **crate NOVA** (o substrato de mola da F0). ⚠️ Conferido: o `main` **não tem** `ph2d-spring` ⇒ não há add/add |
| `ph2d-editor-core` · `ph2d-ui-state` | aresta interna → `ph2d-spring` |
| `ph2d-panel-timeline` | aresta interna → `ph2d-text` |
| `ph2d-vec-layout` | ⚠️ **`taffy` ganha a feature `"grid"`** — dep que já existia, feature nova |

⚠️ **O `Cargo.lock` ganha UM `+name`, e é a própria crate** (`ph2d-spring`): zero pacote externo
novo. A feature `grid` do `taffy` fica **dentro da contenção que o [ADR-0153](../../architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)
declara** (a crate continua a ÚNICA porta do `taffy` na árvore), e o ADR foi editado pela linha
para o registar — ⚠️ **ele é o único ADR tocado, e é EMENDA, não ADR novo**.

---

## §3 — ⚠️ O ÚNICO ponto que decide esta integração: o `project.rs` foi **PARTIDO**

O `main` de hoje quebrou `shells/desktop/src/project.rs` em **quinze** ficheiros
(`project_schema.rs`, `project_load.rs`, `project_save.rs`, `project_assets.rs`, `project_tokens.rs`,
…), e a constante mudou de casa **e de visibilidade**:

```
base  : shells/desktop/src/project.rs        const PROJECT_SCHEMA: u32 = 70;
linha : shells/desktop/src/project.rs        const PROJECT_SCHEMA: u32 = 72;
main  : shells/desktop/src/project_schema.rs pub(crate) const PROJECT_SCHEMA: u32 = 82;
```

A **escada** (os 81 doc-comments `/// vNN`) vive hoje toda no `project_schema.rs`. A linha escreveu
os seus **dois degraus** — **v71** (a tabela SINAL → AÇÃO) e **v72** (a GRADE, o `LayoutDir`) — no
`project.rs`, onde a constante já não está.

⚠️ **O modo de falha é o de 04/08, e ele é SILENCIOSO:** um merge textual limpo põe os dois degraus
num ficheiro que já não é dono da escada, e o bump **evapora com a suíte verde** — foi exactamente
o que aconteceu ao `project_tokens::install` da `line/Vector` quando a `line/sculpt3d` partiu este
mesmo ficheiro. **Confira pelo CONSUMIDOR, não pelo diff.**

**O que o integrador tem de fazer, nesta ordem:**

1. **CONTAR** os dois degraus contra o `main` do dia: 82 → **83** e **84**.
   ⚠️ *O valor RE-CONTA-SE na hora* — se outra linha pousar antes, ele muda outra vez. Os dois
   doc-comments da linha já dizem **`⚠️ PROVISÓRIO`** por escrito.
2. Mover os dois degraus para **`project_schema.rs`**, no fim da escada.
3. Actualizar a **tripla do pin** em `project_schema_tests.rs`: `(82, 13, 14)` → `(84, 13, 14)`.
4. ⚠️ **Conferir nos DOIS ficheiros.** Esta colisão passa **muda** quando os dois lados escrevem o
   mesmo literal — o git não sabe o que o número significa; foi o conflito do
   `project_schema_tests.rs` ao lado que denunciou a colisão FLIP↔physics em 01/08.

---

## §4 — A interseção real: **13 ficheiros**

Os únicos ficheiros que a linha **e** o `main` mudaram desde a base, com o tamanho de cada lado:

| ficheiro | linha | `main` | nota |
|---|---|---|---|
| `shells/desktop/src/project.rs` | +22/−1 | **+1/−366** | ⚠️ **o corte** — §3 |
| `shells/desktop/src/project_schema_tests.rs` | +14/−1 | +32/−1 | a tripla do pin |
| `shells/desktop/src/render_loop/mod.rs` | **+214/−16** | +129/−0 | os dois lados CRESCEM; o maior ponto de conflito depois do §3 |
| `shells/desktop/src/input_dispatch.rs` | +73/−29 | +1/−0 | o `main` quase não toca |
| `shells/desktop/src/app_state.rs` | +39/−4 | +12/−0 | |
| `shells/desktop/src/main.rs` | +31/−0 | +24/−0 | as duas listas de smoke |
| `shells/desktop/src/render_loop/snapshots.rs` | +3/−0 | +14/−0 | |
| `ph2d-panel-inspector/src/sections/player.rs` | +7/−24 | **+208/−269** | ⚠️ o `main` reescreveu; a linha só ajusta unidade |
| `ph2d-panel-inspector/src/sections/physics_rows.rs` | +3/−1 | +49/−0 | |
| `ph2d-panel-inspector/src/sections/physics_body.rs` | +1/−1 | +1/−0 | |
| `ph2d-panel-painter-layers/src/paint_brush.rs` | +5/−6 | **+73/−136** | ⚠️ idem |
| `ph2d-panel-painter-layers/src/number_field.rs` | +1/−1 | +2/−0 | |
| `Cargo.lock` | +10/−0 | +3/−0 | |

⚠️ **Nos quatro ficheiros de painel a linha só faz uma coisa** — passar o **par visual** ao pintor
(`.visual((estado, t))`) ou corrigir a unidade de um número. São edições de **uma linha** dentro de
regiões que o `main` reescreveu: resolva **pelo consumidor** (o pintor recebe o par?), nunca por
marcadores.

⚠️ **O `render_loop/mod.rs` é o segundo ponto sensível:** +214 de um lado e +129 do outro. A linha
acrescenta a fiação do relógio da UI viva e das cenas de smoke novas; confirme que o **`tick_motion`
continua a ser chamado uma vez por quadro** e que **a ordem** dele em relação ao `dispatch` não se
perde no merge (a ordem é load-bearing e está documentada no `screens/hero/live.rs`).

---

## §5 — Mudanças de comportamento (todas smokadas pelo Enio)

**O app inteiro passa a ter movimento onde antes tinha uma função escada.** Concretamente:

1. **Todo hover interpola** — botões, botões de ícone, caixas, interruptores, sliders, tags, campos
   de texto, áreas de texto, chips numéricos, dropdowns, a pele do painel gerado, e os **polegares
   das 22 barras de rolagem**. ⚠️ O extremo **DURO** não muda: `Pressed`/`Focused`/`Disabled` e o
   `Accent` do arrasto continuam instantâneos, e um id que o relógio nunca viu pinta o token duro
   **byte a byte o mundo pré-substrato**.
2. **A rolagem de painel é suave** (a roda mexe um ALVO) — ~130 leitores herdaram sem uma linha.
3. **O chevron de secção RODA** em vez de trocar de glifo (F4a); ⚠️ o **corpo** da secção continua
   a saltar, de propósito — ver §8.
4. **A paleta de comandos entra em cascata** e existe (`Ctrl+K`, 62 verbos).
5. **O zoom do canvas anima.**
6. **As labels pousam na grade de pixels** (a cintilação dos filetes some de graça).
7. ⚠️ **O CARÁCTER é uma preferência de utilizador, persistida FORA do repo** —
   `~/.ph2d/prefs.txt`. Se ele disser `motion_character=expressive`, o app abre em Expressivo, e
   **é nesse regime que a mola ultrapassa**. O interruptor de *reduced motion* é eixo independente.
8. **O scrub numérico** lê o MESMO intervalo do clamp (43 campos saíram de 20 px para 250 px de
   curso) e o **readout segue a mão** num arrasto de gizmo.
9. Do bloco (A): uma moldura **abraça** o conteúdo, **rola**, e um **sinal move a cena**.

---

## §6 — Smokes

| comando | o que julga |
|---|---|
| `env PH2D_UI_MOTION_SMOKE=1 …` | ⭐ **o CARÁCTER** — ⚠️ a cena **NÃO arma** o carácter, ela manda escolher no pill Settings |
| `env PH2D_UI_MOTION_SMOKE=2 …` | **a CORDA** (C1) |
| `env PH2D_BUILD_SMOKE=66 …` | o **SIZING** |
| `=67` | a **ROLAGEM** da moldura |
| `=68` | a tabela **SINAL → PAPEL** |
| `=69` | a **GRADE** |
| `=70` | editar nós de **VÁRIAS** formas |
| `=71` | o **LAÇO** |
| `=72` | o **rótulo de distância** (W6.6) |
| `=73` | o **X/Y numérico do nó** (W6.9) |
| `=62` | ⚠️ **cena PRÉ-EXISTENTE** — é a que o Enio usou para julgar o hover das últimas seis waves |

Todos com `ph2d-run cargo run -p ph2d-host-desktop --release`.

⚠️ **A cena `=62` imprime `[ui-panel] … N row(s)`** — se essa linha não aparecer, **PARE**: o resto
do smoke não diz nada.

---

## §7 — O gate que a LINHA rodou (e o que ela NÃO rodou)

- `cargo test -p` sobre **as 36 crates que a linha tocou** → **292 binários verdes, 0 vermelhos**;
  mais `ph2d-host-desktop` → **148 verdes, 0 vermelhos**. Total **440**.
- `cargo clippy -p ph2d-editor-core --all-targets` → **0**.
- `cargo fmt --all -- --check` → limpo.

⚠️ **E o único vermelho desta corrida era o MEU harness, não o código** — o laço derivava o nome da
crate com `basename shells/desktop`, que dá `desktop`, e o cargo respondeu `package ID
specification 'desktop' did not match any packages`. *Um vermelho que não nomeia um teste é uma
falha de ferramenta até prova em contrário* ([[feedback_a_negative_search_needs_a_positive_control]]);
com o nome certo (`ph2d-host-desktop`) são 148 verdes.

⚠️ **O que a linha NÃO rodou, e o integrador TEM de rodar:**

1. **O gate da ÁRVORE COMBINADA** — é a única coisa que vê a falha emergente. Esta linha toca
   `crates/ph2d-editor-core/tests/` e `shells/desktop/tests/`, e **esses gates só correm na
   varredura impactada**: um fechamento por `cargo test -p` **não os alcança**. É a causa
   estrutural que `line/physics`, `line/motion-value` e esta mesma linha já documentaram três
   vezes (o `file_loc_caps`, o `no_tofu_glyphs`, os dois arch-gates de shell de 23/07).
2. **`cargo test --workspace`** depois do rebase — o `main` andou 203 commits.
3. ⚠️ **Nenhuma leitura de RELÓGIO desta workstation vale com `load average` acima de ~5.** Durante
   este fecho o load foi de 1,8 a 34; os gates de razão desta árvore são poucos, mas os
   `--ignored` do Painter e o `measure_normals_parallel_speedup` do `ph2d-mesh` são **flakes de
   carga conhecidas** — re-rode isolado antes de suspeitar do merge.

---

## §8 — Aberto, com o preço ao lado

| item | preço, medido |
|---|---|
| **F4b** — o CORPO da secção interpola | ~20 pintores, cada um com aritmética de `y` própria ⇒ **medir-lembrar-recortar** por painel. Conferido em 15/08: o `open_t` **só veste o cabeçalho** nos 10 painéis que o lêem, **nenhum corpo interpola**. ⚠️ O `open_t` é neutro ⇒ migração parcial é segura, mas meia-migração deixa **metade dos painéis a deslizar e metade a saltar**. ⚠️ **O gatilho do plano (*"o smoke decide se ele é sentido em falta"*) NÃO disparou** em três smokes |
| **F5** cauda (hierarquia · rows do inspector) | desbloqueado (a F2 fechou). ⚠️ A cascata é **ENTRADA**, não realce — a cerca do estudo §6.2 (*varrer uma lista não deve amaciar*) **não a alcança** |
| **E4** menu radial · **C2** realce de proveniência | **M cada, e são FEATURES**, não polimento |
| **D1** som de UI · **D2** partículas | eixo 4; ⛔ som **nunca** ligado por omissão (§7 do plano) |
| canvas: zoom **suave** por gesto contínuo | ⚠️ **o dono não é a `editor-core`** — pede fiação no shell |
| **X1** pressão da caneta | ⛔ **bloqueado FORA do repo**: winit 0.30.13 crava `force: None` nos três backends de desktop |
| sondas do §8 do plano | `set_cursor_grab` por plataforma (por CORRER) · `Ctrl = encaixa no step` (por **DECIDIR**) · o tecto dos 146 campos sem intervalo |

⚠️ **E um resíduo estrutural que fica NOMEADO, sem gate:** o eixo do hover está fechado para tudo
o que hoje o **lê** — os tipos registados (pelo `the_pointer_and_the_clock_agree_on_who_lights_up`)
e os polegares (pelo mapa do despachante). Uma superfície **`Plain`** nova que comece a ler
`hover_live` sem estar nesse mapa **nasceria muda outra vez**, e isso **não tem gate**: o registo e
a leitura vivem em crates diferentes com o id numa variável.

⛔ **E não "complete" a cura alargando o censo a todo `Plain`:** as rows da Hierarquia são `Plain`,
e amaciá-las revive a cerca do estudo §6.2. Isto está **gateado** (`the_thumb_census_names_the_thumb_and_leaves_the_list_row_alone`),
e a mutação que o tenta deixa **três dos quatro** gates de produto verdes.
