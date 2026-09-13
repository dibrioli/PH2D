# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-09-13 (2.ª volta: o INSPECTOR)

> ⚠️ **Este handoff é o SEGUNDO do dia e não substitui o primeiro.** O
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-09-13.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-13.md) fecha a
> volta do **Painter + Hierarquia** (e o smoke dela foi aprovado pelo dono). Esta continua a mesma
> linha, no mesmo ramo, depois da ordem *«siga»*: o painel seguinte da fila.

## 1 — Identidade

| | |
|---|---|
| linha | `line/UIUX` · worktree `Worktrees/line-UIUX` |
| merge-base | `1d43da737` (o `main` enviado de 13/09) |
| commits desta volta | ver §12 (os da 1.ª volta vão no handoff irmão) |
| contrato congelado (§6) | **intocado** |
| schemas / registos partilhados | **nenhum** se move (`collision-surface.sh` em §4) |
| ADR | nenhum |

## 2 — O que esta volta fez

1. ⭐⭐⭐ **O painel INSPECTOR fala INTEIRO pela tabela de strings**: `626 → 1` literal com cara de
   língua (o único é o `Panel::TITLE`, excepção declarada com mecanismo, a mesma do Painter e da
   Hierarquia). **602 chaves** `panel.inspector.<secção>.<nome>`, **46 frases** com peças do código
   por `tr_with` (marcadores nomeados), e as leituras em minúsculas que a régua não vê por desenho
   (`air`/`steep`/`ground`, `left`/`right`, `repeats`/`once`) migradas à mão, lidas do pintor.
2. ⭐⭐⭐ **`ph2d_i18n::TextKey` — a chave TIPADA.** Metade do vocabulário do painel vive em tabelas
   `const` (tipos de junta, cards da §14, modos de mistura), onde o `tr` não compila (E0015).
   Guardada como `&str`, *uma chave e um texto são o mesmo tipo*: o consumidor que se esquece de
   traduzir compila e pinta `panel.inspector.joint.pin` no ecrã. Com `TextKey` isso é **erro de
   compilação** — 40 tabelas trocadas de tipo, ~50 consumidores obrigados a `.map(TextKey::tr)` ou
   `.tr()`.
3. ⛔⛔ **A régua lexical corrigiu-se DUAS vezes**, e as duas foram achadas por esta migração:
   - **a palavra GRITADA** (`"LEG"`, `"WALK"`, `"OPERATION"`) não era língua para ela — **+151** no
     repo, entre eles o trilho esquerdo inteiro da `editor-core` e quatro títulos de card do Painter;
   - **a BARRA numa frase** era lida como caminho — **+47** fora do Inspector e **+32** dentro
     (`"Speed (m/s)"`, a legenda do 9-slice, `"Swap A / B"`). ⚠️ Quem os apanhou **não foi a régua:
     foi o COMPILADOR**, porque a tabela deles já era `TextKey` e os treze ficaram `&str` no meio.
   Cada correcção tem controlo dos DOIS lados no `the_language_test_has_both_sides_of_every_border`.
4. ⭐ **O censo de chaves aceita VÁRIAS tabelas** (`keys_used` / `keys_declared` tomam `&[&str]`), e o
   gate do Inspector ganhou a metade *«uma chave mora em UMA tabela, e a §14 na dela»* — sem ela, a
   mesma chave nas duas metades seria um braço morto que nenhum conjunto acusa.
5. ⛔ **E o gate ACUSOU-SE A SI PRÓPRIO na 1.ª corrida**: o literal `"panel.inspector.player."` que
   ele escrevia para separar as metades é lido pelo censo como chave em uso. Duas curas: o prefixo
   passa a ser derivado (`format!("{PREFIX}player.")`), e **uma chave não acaba em ponto**
   (`looks_like_a_key`, com controlo próprio).
6. **Painter e Hierarquia** receberam o que a régua corrigida passou a ver: os quatro títulos de card
   (`MODE`, `TOOL`, `OPERATION`×2), as duas frases com barra da Selecção, e o selo `ENT` da
   Hierarquia. O `RGB` do separador de curvas fica como excepção **declarada** (nome de modelo de
   cor, irmão de `R`/`G`/`B`).
7. **A dívida da `ph2d-editor-core` subiu por CORRECÇÃO DA RÉGUA** (`641 → 692`), ficheiro a ficheiro,
   com a conta escrita no gate. *Uma correcção da medição, nunca licença para crescer.*
8. **Sete entradas do Inspector saíram da baseline antiga do HR-15** — os `placeholder` migraram, e a
   dívida saiu do binário, não só do alcance do scanner.
9. **Cinco cortes por responsabilidade** (nunca subir número) — todos causados pela MIGRAÇÃO, porque
   o `tr("…")` alonga a chamada e o `rustfmt` parte-a:
   - por FICHEIRO (600): `sections/physics_area_rows.rs` (o bloco de ZONA saiu do `physics_rows.rs`,
     `643 → 521`) e `sections/joint_kind_rows.rs` (as rows de cada TIPO saíram do `joint.rs`,
     `632 → 458`);
   - por FUNÇÃO (200): `paint_enabler_rows` (a moldura de ecrã sai da `paint_visibility_section`,
     `213`), `paint_flip_rows` (as caixas de espelhar saem da `paint_sprite_sheet_section`, `212`) e a
     porta `entity_badge` na Hierarquia (o `apply_event` estava **no** tecto desde 19/08 e a chamada
     em três linhas punha-o em `203`);
   - por TABELA (700, workspace): as chaves partidas em `inspector.rs` + `inspector_player.rs`
     (`861 → 558 + 325`).
10. ⛔⛔ **E o `no_magic_numeric` acusou CINCO números que já tinham isenção**: o `rustfmt` reflowou a
    chamada, o número ficou sozinho numa linha e o `// LITERAL-PX-OK` ficou na linha do parêntese —
    o gate exige os dois na MESMA linha. *Um marcador de isenção que se separa do que isenta é uma
    isenção que evapora*, e isso só aparece no dia em que alguém reformata o ficheiro.
11. **O migrador ficou mais honesto**: marca a tabela `const` INTEIRA (a heurística lia só a linha do
    `const`, e um array multi-linha escapava), escreve `TextKey::new` nela, e **JUNTA** à tabela em vez
    de a reescrever — uma segunda passagem apagaria a primeira.

## 3 — Foundational / partilhado tocado, e porquê

| ficheiro | o quê | porquê aqui |
|---|---|---|
| `crates/ph2d-i18n/src/lib.rs` | `TextKey` novo · `mod inspector` + `mod inspector_player` na cadeia · chave `panel.hierarchy.badge.entity` | a tabela é da casa; um painel novo entra por irmão, nunca por um segundo `match` |
| `crates/ph2d-i18n/src/inspector*.rs` | tabelas novas (602 chaves) | — |
| `crates/ph2d-i18n/src/painter_layers.rs` | 6 braços novos (títulos gritados + frases com barra) | fora dos marcadores, à mão |
| `crates/ph2d-label-census/src/lexical.rs` | `is_language`: palavra gritada · barra numa frase | a régua é a folha partilhada dos 3 gates |
| `crates/ph2d-label-census/src/keys.rs` | `keys_used`/`keys_declared` tomam `&[&str]` · chave não acaba em ponto | vocabulário partido por secção |
| `crates/ph2d-editor-core/tests/it/no_label_of_this_crate_is_written_in_the_painter.rs` | dívida re-medida (`641 → 692`) + a conta | a régua mudou, o código não |
| `crates/ph2d-editor-core/tests/it/hr15_no_hardcoded_ui_strings.rs` | −7 entradas do Inspector | os `placeholder` migraram |
| `shells/desktop/tests/it/the_filter_segmented_tells_the_truth.rs` | lê `FILTER_LABELS` já traduzido | o tipo da tabela mudou |

## 4 — Superfície de colisão (corrida nesta worktree, `collision-surface.sh` do primário)

```
merge-base 1d43da737 · PROJECT_SCHEMA 128 (base 128) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 18 ·
FIELD_DOC 22 · registos 85/86/86 (todos iguais à base) · contrato §6 intocado · ADR nenhum ·
Cargo.lock: só o pacote INTERNO `ph2d-label-census` (da 1.ª volta) · zero marcadores de conflito ·
nenhum ficheiro da linha passa tecto de LOC
```

⇒ **nada desta linha soma com outra**: nenhum contador partilhado se move, e o que ela toca fora dos
painéis são gates e a tabela de strings.

## 5 — O que só o portão de fecho apanhou

⚠️ **Os DOIS vermelhos desta volta vivem na `ph2d-editor-core`, e nenhuma corrida por crate do
Inspector os acorda** — é a mesma cegueira que o handoff da `line/motion-value` de 07/09 já nomeou
(*«um fecho que só corre as crates EDITADAS é cego aos gates de arquitectura»*):

| vermelho | o que era | cura |
|---|---|---|
| `architecture_panel_loc_cap::panel_functions_under_loc_cap` | **três funções** passaram as 200 LOC **sem uma linha de lógica nova** — o `tr("…")` alonga a chamada e o `rustfmt` parte-a: `paint_visibility_section` 213 · `paint_sprite_sheet_section` 212 · `apply_event` da Hierarquia 203 | três cortes por responsabilidade (§2.9) |
| `no_magic_numeric::no_magic_numeric_in_widget_or_screens` | **cinco números que já estavam isentos**: o reflow do `rustfmt` deixou o `// LITERAL-PX-OK` na linha do parêntese e o número sozinho noutra | marcador de volta à linha do número |

⇒ *uma migração de texto não muda o comportamento e mexe em tectos e isenções* — quem migrar o
painel seguinte corre o `architecture_panel_loc_cap` (ficheiro **e** função) e o `no_magic_numeric`
ANTES de dar a volta por fechada.

## 6 — ⚠️⚠️ O QUE VAI PARTIR NA FUSÃO, e é previsível

1. **`keys_used`/`keys_declared` mudaram de assinatura** (`&str` → `&[&str]`). Uma linha que copie o
   gate do Painter para outro painel passa `&[TABLE]`. Falha ALTO (compila).
2. **Tipos públicos do Inspector passaram a `TextKey`**: `FILTER_LABELS`, `TILE_MODE_LABELS`,
   `PLAYER_CARDS`, `PLAYER_BUTTON_TIPS`, `PlayerRow`, `LAYER_LABELS` (ordering), `SHAPE_LABELS`…
   Quem os leia como `&str` **não compila** — é esse o ponto. O único consumidor fora da crate era o
   gate da shell, já curado aqui.
3. **`paste_label` / `bake_label` / `rig_button_label` continuam a devolver `String` com o MESMO
   texto** (agora por `tr_with`): um gate que compare a frase inteira continua verde.
4. **A baseline do `hr15_no_hardcoded_ui_strings` perdeu 7 entradas.** Uma linha que acrescente um
   `.placeholder("literal")` no Inspector fica vermelha — e deve.
5. **A `DIVIDA` da `editor-core` subiu por correcção da régua.** ⛔ Quem trouxer números próprios
   daquele ficheiro **re-mede**, nunca funde os dois lados: o número certo não está em nenhum deles.
6. **A tabela do Inspector está PARTIDA**: uma chave `panel.inspector.player.*` mora em
   `inspector_player.rs`, e o gate reprova a chave na metade errada ou nas duas.
7. **Três ficheiros novos em `sections/`** (`physics_area_rows.rs`, `joint_kind_rows.rs`) e um módulo
   `#[path]` novo no `joint.rs`: um merge que traga rows novas para `physics_rows.rs`/`joint.rs`
   aterra em ficheiros mais curtos, e o bloco de zona/os params de tipo mudaram de casa.
8. ⚠️ **O `paint_frame.rs` do Inspector está em 600 LOC exactos** (o tecto). Qualquer linha que lhe
   acrescente uma linha fica vermelha — a cura é corte, e o ficheiro já tem candidatos óbvios.
9. ⚠️ **Toda migração de texto EMPURRA tectos**, e não é preciso escrever uma linha de lógica: cinco
   deles nesta volta. Quem migrar outro painel orça o corte **antes**, e corre o
   `architecture_panel_loc_cap` (ficheiro **e** função) no fecho — ele foi o primeiro vermelho aqui.

## 7 — Ordem e o que smokar

**Ordem:** cronológica, `--ff-only`, depois do handoff irmão (a 1.ª volta do dia).

Nada muda à vista. O modo de falha a procurar é um rótulo que apareça como `panel.inspector.…`.

| onde | o que ver |
|---|---|
| **Inspector**, com um objecto escolhido | os cabeçalhos e as linhas de **Transform**, **Render Source** (incl. `Storage`/`Source`), **Sampling**, **Ordering**, **Visibility**, **Color & Tint**, **Material & Blend**, **Sprite Sheet** |
| **Inspector** de um corpo físico | **Physics Body** (Body/Collider/Collision/Bake), as linhas de **material** e de **massa**, e a **ZONA** de um sensor (Force X/Y, Force Axes, Torque, Falloff, Drag) |
| **Physics Joint** | o selector de **Kind** (Pin…Custom), **Limits/Travel**, **Motor** (Target/Speed com a unidade), **Breakable**, e o `Custom` com os três eixos (`X`, `Y`, `Rotation`) |
| **Platform Player** (§14) | os doze cards (`LEG`, `WALK`, `JUMP`, `FORGIVENESS`…), as dicas de hover das linhas, os botões *Fit to Collider (needs > …)* e *Clear Recorded Run (… s)*, e a leitura viva (`Posture: ground`, `Facing: right`) |
| **Timers · Signal Actions · Animation · Anchors** | os títulos com contagem (`Timers  (3)`), os avisos (*«This timer never fires…»*) e os campos vazios (`timer_name…`, `on signal…`) |

**Comando (copiável de uma vez):**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --profile smoke
```

✅ **O binário já está compilado nesta worktree**, sobre a árvore final e DEPOIS de `rm -rf
target/*/incremental` (**16 GB** reclamados: 13 GB do `debug`, 2,9 GB do `smoke`): a 1.ª compilação
refez só a `ph2d-app-painter` e a shell (**19,59 s**), e a 2.ª terminou com **zero `Compiling`**:

```
$ cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

Binário: `target/smoke/ph2d-host-desktop`, **79,6 MB**. ⚠️ Nenhum smoke desta volta pede `--features`,
e `~/.ph2d/layout.txt` guarda a arrumação dos painéis — um ficheiro velho abre o app com o Inspector
fechado, e apagá-lo é o reset.

## 8 — ⏳ ABERTO (não corrigir na integração)

| item | o que falta |
|---|---|
| **as `ph2d-app-*` e a shell** (`917` + `843` + `577` + `450`…) | o grosso dos **4 638** que sobram; três espécies misturadas (avisos de interface · nomes de objectos de cenas de smoke · narração de smoke em português) ⇒ **triagem antes de virar dívida** |
| **`ph2d-editor-core`, 692** | dívida ao número, por ficheiro; `menu_rows.rs` sozinho tem 164. A triagem move para `NOT_LANGUAGE` o que não é língua (`RGB`/`HSV`/`OKLCH`, os nomes da cena de amostra) |
| **`ph2d-panel-vector`, 225** | o painel seguinte da fila |
| **`Panel::TITLE` dos 26 painéis** | a aba lê um `const &'static str`; fazer as abas falarem pela tabela é mudar o contrato do painel — obra própria |
| **as réguas que leem LÍNGUA para inferir semântica** | `the_filter_segmented_tells_the_truth` decide por `label.starts_with("Near")`; com uma segunda língua isso deixa de valer. A cura é decidir pela CHAVE |
| **minúsculas e siglas** | a régua não conta uma palavra só em minúsculas nem uma sigla de 2 letras: são indistinguíveis de identificadores. As do Inspector foram achadas **lendo o pintor**, não o censo |

## 9 — As premissas que a medição derrubou

1. *«A régua lexical vê o painel inteiro»* — **falso duas vezes** no mesmo dia (gritadas, barra).
2. *«Ela erra para o lado ALTO»* (o doc dela dizia-o) — errou para **baixo**, nas duas.
3. *«Guardar a chave como `&str` numa tabela `const` basta»* — refutado: foi a chave TIPADA que expôs
   treze rótulos que a régua não via.
4. *«Um gate por crate não se acusa a si próprio»* — acusou, pelo prefixo que ele escreve.
5. *«O esqueleto marca as tabelas `const`»* — marcava só a linha do `const`; um array multi-linha
   passava por `tr` e não compilava (E0015, falha alta — e foi assim que se descobriu).
6. *«`aplicar` escreve a tabela»* — escrevia, e uma segunda passagem apagaria a primeira.
7. *«`const RECT_STEP` na linha de cima não engana»* — enganou: a heurística subia e casava o `const`
   antes de ver que a instrução já tinha fechado.
8. *«Uma migração de texto não mexe em tectos»* — mexeu em **cinco**, e o primeiro vermelho do portão
   foi o tecto de FUNÇÃO, não o de ficheiro.
9. *«Uma isenção escrita ao lado do número protege-o»* — protege enquanto ninguém reformatar a linha.

## 10 — Para o `CLAUDE.md` §5 — UMA linha de estado (a `Aberto:` do módulo UI/UX)

> ⛔ os **4 638** literais de UI pintados fora da tabela (censo `cargo run -q -p ph2d-label-census
> --example censo -- --resumo`): o **Painter**, a **Hierarquia** e o **Inspector** estão a zero, com
> gate por crate; sobram a shell (917), a `app-physics` (843), a `editor-core` (692, ao número por
> ficheiro) e o `panel-vector` (225) — ⚠️ e a régua corrigiu-se **duas** vezes em 13/09 (a palavra
> GRITADA · a BARRA numa frase), por isso um número medido antes disso é um piso.

## 11 — As leis que esta volta pagou

1. **Uma chave e um texto do mesmo tipo é um defeito à espera.** O tipo separa-os e o compilador
   cobra — e foi ele que achou o que a régua não via.
2. **Uma régua que se declara «a errar para cima» tem de ser falsificada nos dois sentidos.** As duas
   correcções vieram de casos REAIS, não de leitura do critério.
3. **Um gate escreve no fonte o que ele próprio mede.** Prefixo, chave de exemplo, nome de ficheiro:
   ou se deriva, ou o censo lê o gate como produto.
4. **Uma heurística sobre fonte tem de saber onde a instrução ACABA**, não só onde ela começa.
5. **Uma ferramenta de migração que REESCREVE não sobrevive à segunda passagem** — e a segunda
   passagem é a regra, porque a régua melhora.
6. **Um tecto de LOC cura-se por corte**: três nesta volta, nenhum número subido.

## 12 — Estado do portão, mutações e higiene

**Commits desta volta** (sobre `949672125`, a nota do smoke aprovado da 1.ª volta):

| commit | o quê |
|---|---|
| `fb1ecf9ec` | a régua: palavra GRITADA · barra numa frase · chave que não acaba em ponto · censo de chaves com várias tabelas · o migrador (tabela `const` inteira, `TextKey::new`, JUNTA em vez de reescrever) |
| `e41fc118b` | o Inspector pela tabela (626 → 1), a `TextKey`, as duas tabelas de chaves, o gate por crate, Painter + Hierarquia + o gate da shell |
| `6825dd606` | a dívida da `editor-core` re-medida (`641 → 692`) e as sete entradas do Inspector fora da baseline antiga |
| `dc700032e` | os três tectos de função curados por corte e os cinco marcadores de isenção de volta à linha do número |
| _este_ | a medição (§2-quinquies de `medicoes/07`) e este handoff |

**Portão batched, 2.ª corrida (a 1.ª deu os dois vermelhos do §5), TODOS verdes:**

| passo | resultado |
|---|---|
| `doc-index.sh` + `--check` | ✅ |
| `cargo fmt --all -- --check` | ✅ |
| censo lexical (`--resumo`) | ✅ **4 638** no repo (era 5 035 com a régua cega) |
| censo do `ph2d-tool-painter` | ✅ (inalterado) |
| `cargo check --workspace --all-targets` (aviso = erro) | ✅ |
| `cargo clippy --workspace --all-targets` (`-D warnings`) | ✅ |
| `cargo machete` · `check-standalone-optional` · `check-workflow-packages` | ✅ |
| `typos` | ✅ |
| `nextest-impacted` (`BASE=1d43da737`) | ✅ **13 867 testes, 13 867 passaram** |

**Provas de mutação — 9/9 MORTAS, cada uma pelo NOME do teste que a devia apanhar** (carga impressa
ao lado de cada corrida; restauro por cópia + `touch` + comparação byte a byte):

| # | defeito injectado | quem o matou |
|---|---|---|
| M1 | um literal volta ao pintor (`"Visible"`) | `every_word_this_panel_shows_comes_from_the_string_table` |
| M2 | a chave usada sai da tabela | `every_inspector_key_exists_on_both_sides` |
| M3 | uma chave órfã entra na tabela | `every_inspector_key_exists_on_both_sides` (a outra metade) |
| M4 | uma chave da §14 na tabela geral | `every_inspector_key_lives_in_exactly_one_table` |
| M5 | o consumidor esquece o `.tr()` de uma tabela | **o compilador** (E0308) — a prova de que a chave TIPADA trabalha |
| M6 | a palavra GRITADA volta a não ser língua | `the_language_test_has_both_sides_of_every_border` |
| M7 | uma barra volta a ser sempre caminho | idem |
| M8 | um prefixo de secção volta a ser chave | `a_section_prefix_is_not_a_key` |
| M9 | a dívida descreve mais do que existe | `the_debt_only_describes_what_is_still_there` |

**Higiene (DIRETRIZ §1.5.9 itens 7 e 9):** `rm -rf target/*/incremental` e o binário do smoke
compilado em perfil `smoke` sobre a árvore final, com a 2.ª compilação a não compilar nada — os
números estão no §7 e na resposta ao dono.

⚠️ **A máquina esteve partilhada com outra sessão durante o fecho** (carga entre 4 e 54): os
relógios desta corrida não servem de medição de desempenho, e os vereditos não dependem deles —
os dois vermelhos foram gates estruturais, reprodutíveis, não flakes de recurso.
