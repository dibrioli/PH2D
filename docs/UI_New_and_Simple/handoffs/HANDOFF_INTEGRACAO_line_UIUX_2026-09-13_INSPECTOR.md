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
| **`+` → Add Component → Physics** (o §13) | **três** itens e não trinta: *Physics Body — brings Collision Shape* · *Collision Shape* · *Platform Player — brings Physics Body, Collision Shape*. ⛔ Nenhum *Gravity Scale*, *Damping*, *Lock Position X*, *Joint*… — esses são rows da §11 |
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

Binário: `target/smoke/ph2d-host-desktop`, **79,6 MB**. ⭐ **Recompilado depois do §13** (`fb26bc48a`):
`8,01 s` na 1.ª corrida (cinco crates, porque o `ph2d-component-desc` está debaixo de toda a gente) e
`0,22 s` na 2.ª, com **zero `Compiling`** — `79 588 248` bytes. ⚠️ Nenhum smoke desta volta pede `--features`,
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

⚠️ **E uma segunda linha, para a `Componentes / instâncias`** (o §13 abaixo — o assunto é a F3 do
ADR-0166, não esta linha):

> ⭐⭐ **A paleta do `+` estava PICOTADA na física** (report do dono, 13/09): `30` dos `32` tipos da
> família estavam `Authored` — **35 % da paleta inteira** —, e `27` deles são as **rows** que a
> §11/§12/§13/§14 já pinta e já anexa (presença-override), ou nascem de gestos. Hoje são **três
> portas** (*Physics Body* · *Collision Shape* · *Platform Player*, os dois primeiros renomeados para
> nomear a secção que criam), os outros 27 são `Attach::Intrinsic` — **não se oferecem, continuam a
> editar-se** — e a paleta inteira desce de **85 para 58** itens (*Image · Show all* deixa de precisar
> de rolagem). ⚠️ A régua já estava escrita na razão 2 do `Attach::Intrinsic` e **nunca tinha sido
> corrida sobre a família**: o helper local chamava-se `p` e construía `authored`, logo *classificar
> era não escrever nada*. ⛔ As outras famílias **não** foram auditadas com ela (a `core` oferece 22).
> Catraca nomeada: `the_physics_family_offers_three_doors_and_not_its_rows`.

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

---

## 13 — ADENDO de 13/09 — a paleta de física estava PICOTADA (report do dono, depois do fecho)

> *«Não temos mais Add Physics Body no painel. Os componentes da física foram colocados no modal que
> é aberto ao clicar no `+` do inspector. Contudo foi erroneamente picotado, dividido em inúmeros
> supostos componentes que na verdade são apenas seções das opções de física. … Talvez só Add
> Physics Body no modal seja mais correto, acrescentando todas as opções da física ao mesmo tempo.»*

⚠️ **Isto NÃO é desta linha nem desta volta** — é a F3 do ADR-0166 (`line/components`, 24/08), e cai
aqui porque o dono o reportou a esta janela e a cura é uma tabela de UI. O integrador deve lê-lo como
um item da família **Componentes**, não da migração de texto acima.

### 13.1 — O que estava errado, medido

A família de física do catálogo declarava **30 dos seus 32 tipos como `Authored`** — `30` de `85`
itens da paleta inteira (**35 %**, e a maior família de longe; a seguinte, `core`, oferece 22). E
**27** deles não são uma escolha do artista: são as **rows** que a §11/§12/§13/§14 já pinta e **já
anexa**, pelo idioma da presença-override que o `PhysicsFieldEdit` inteiro usa (`Ccd`,
`LockPositionX/Y`, `LockRotation`, `GravityScale`, `InitialVelocity`, `MaterialCombine`,
`DampingOverride`, `OneWayPlatform`, `NoWallCling`, `WalkSurface`, as **sete** da zona, os dois
toggles da §14, as duas rows de sinal) ou nascem de **gestos** (`PhysicsJoint`, `PulleyWheel`,
`WestonAxle`, `JointWorldAnchor`, `RopeStops`).

⛔ **E um deles anexado pelo `+` é PIOR que ausente:** um `PhysicsJoint::default()` prende
`StableId 0` a `StableId 0` — uma junta que não prende nada, num objeto que pode nem ser corpo. Os
outros 26 são mais benignos e têm o mesmo defeito de fundo: *o componente entra e nada acontece*,
porque o valor neutro é exactamente o que a ausência já dizia.

⭐⭐ **E a prova de que os 27 continuam alcançáveis não é minha: JÁ EXISTE e está verde.** O gate
`every_registered_physics_component_has_a_ui_writer` (`shells/desktop/tests/it/`) afirma, desde as
oito waves da física, que *todo componente de física registado é nomeado por alguém no caminho de
ESCRITA da UI* — nove ficheiros, e a lista falha alto quando um escritor sai. ⇒ tirar os 27 da paleta
**não pode** torná-los inalcançáveis: se tornasse, aquele gate ficava vermelho. *Esta poda apoia-se
num censo que outra linha pagou, e a única coisa nova é ter olhado para ele.*

⭐⭐ **A régua que faltava já estava ESCRITA — no `Attach::Intrinsic`, razão 2**: *«o neutro existe e
anexá-lo seria um NO-OP; a PRESENÇA é que carrega o sentido, e o valor de anexação tem de vir do
CONTEXTO, que a paleta genérica não conhece»*. Ela tinha sido aplicada a **dois** destes
(`Dominance`, `MassOverride`, com a cerca citada no próprio catálogo) e **nunca foi corrida sobre a
família**: o helper local chamava-se `p` e construía `authored`, então classificar era *não escrever
nada*. *Um helper cujo caminho de menor esforço é uma das respostas escolhe a resposta.*

### 13.2 — A cura: TRÊS portas

| Porta | O que ela quer dizer | Traz |
|---|---|---|
| **Physics Body** (`RigidBody`) | *este objeto é simulado* — e a §11 abre com **todas** as opções | `Collision Shape` |
| **Collision Shape** (`Collider`) | *esta forma é mais uma peça do corpo acima* (W-Compound, o antigo *Add Shape to X*) | — |
| **Platform Player** (`PlatformPlayer`) | *este objeto é um personagem que anda e salta* | `Physics Body`, `Collision Shape` |

Os outros 27 passam a `Attach::Intrinsic` — **não se oferecem, continuam a editar-se** (`Intrinsic`
diz *não se escolhe*, nunca *não se edita*; é a distinção que a terceira variante comprou, e a
`PhysicsJoint`/`PulleyWheel` mantêm os `fields` declarados, logo o remap de referências da F4.2 fica
intacto).

⚠️ **Dois rótulos mudaram, e é a regra do próprio campo** (*`display_name` é nomeado pelo resultado,
não pelo tipo Rust*): `Rigid Body` → **Physics Body** (o cabeçalho da §11 que nasce, e a palavra do
botão que o dono conhecia) e `Collider` → **Collision Shape** (neste app «shape» sozinho já é forma
vectorial e forma 3D).

⛔ **O `Collider` NÃO pode ser `Intrinsic`**, e não é opinião: o `RigidBody` exige-o, e o gate
`every_declared_requirement_names_a_real_component` proíbe uma dependência `Intrinsic` — a cascata
teria de construir o que ninguém declarou construível.

⭐ **E a resposta ao *«não temos mais Add Physics Body no painel»* é a que o próprio dono propôs:** a
porta fica no modal, porque a face vazia da §11 é precisamente o que o ADR-0166 retirou (o Inspector
mostra o que o objecto TEM). Repô-la só para a física faria dela a única secção com face vazia, ao
lado de Timer/Áudio/Câmera/Âncoras/Animação que perderam a delas na mesma fase. O que muda é o
**nome**: o item passa a chamar-se como o botão se chamava. ⚠️ **O comportamento é o mesmo ao bit** —
`BodyKind::default()` é `Dynamic` e a semente do `Collider` mede o sprite, que era exactamente o que
o `PhysicsFieldEdit::Add` fazia.

⚠️ **A face vazia da §11 continua VIVA e alcançável** num caso: `build_physics_info` mantém a secção
quando `rig_parts > 0` (o tronco de um personagem), e ali os três botões — *Add Physics Body*, *Add
Shape to X*, *Rig N Parts* — continuam a ser pintados. Não é código morto.

### 13.3 — Medição, antes e depois (sonda `measure_palette`, viewport 1187×953 do report de 25/08)

| caso | antes | depois |
|---|---|---|
| itens oferecidos, total | **85** | **58** |
| `Empty` · aplicável | 56 | **29** |
| `Image` · aplicável | 71 | **44** |
| `Empty` · *Show all* — o que não cabe | 238 px | **46 px** |
| `Image` · *Show all* — o que não cabe | 175 px | **0 px** |

⭐ A queixa de 25/08 (*«a janela não tem scroll … veja que componentes estão inacessíveis fora da
janela»*) foi curada com rolagem; esta poda tira-lhe a causa em metade dos casos.

### 13.4 — O gate, e a mutação que o prova

`the_physics_family_offers_three_doors_and_not_its_rows` (em `catalog/physics.rs`) — uma **catraca
NOMEADA**: a lista das três portas é a afirmação, não um número. Uma quarta entrada autorada reprova
e obriga a decidir *isto é uma intenção do artista ou é uma row?*; uma das três que desapareça
reprova pelo mesmo `assert_eq`, que é a **metade de obsolescência**; e um **piso de população**
(`DESCS.len() >= 30`) impede que apagar a família deixe o gate a comparar dois vazios.

| # | defeito injectado | quem o matou |
|---|---|---|
| M10 | `i("…::Ccd", …)` volta a `D::authored(…)` | `the_physics_family_offers_three_doors_and_not_its_rows` (`left` nomeia o intruso) |

Restauro por cópia + `touch` + comparação byte a byte, verde depois.

### 13.5 — ⏳ ABERTO, e é honesto dizê-lo

- **As outras famílias não foram auditadas com esta régua, e a segunda maior tem sinais da mesma
  doença.** A `core` declara **22** `D::authored`, e à vista há lá candidatos claros a row — `Blend
  Mode` (§Material & Blend), `Texture Filter` / `Texture Repeat` (§Sampling), `Order In Layer` ·
  `Sorting Layer` · `Sorting Group` · `Z Index Override` · `Show Behind Parent` · `Y Sort` ·
  `Visibility Layer` (§Ordering), `Locked` e `Visibility` (o cadeado e o olho da Hierarquia) —, mais
  o `Transform`, que **toda** entidade tem e que por isso só seria oferecido a um objecto sem
  `Transform`, isto é a nenhum. ⚠️ **Mas ela NÃO é uniformemente descuidada como a física**: o
  `InstanceOf` e o `LinkedArt` já são `Intrinsic` **com a razão escrita**, logo alguém correu a régua
  em parte da tabela. ⛔ **E falta-lhe o que autorizou esta poda:** o `ph2d-ecs` não tem o gémeo do
  `every_registered_physics_component_has_a_ui_writer`, então uma poda ali teria de **construir o
  censo primeiro** — é uma wave própria, não um apêndice desta. *A régua é barata; o que não é barato
  é acreditar que a família já está certa, nem podá-la sem o censo do outro lado.*
- **A paleta não sabe dizer *«isto precisa de um corpo acima»***: o `Collision Shape` é sempre
  oferecido, e num objecto sem corpo ancestral a §11 pinta honestamente *«Shape with no body
  above»*. O mecanismo do esmaecido-com-razão existe mas é indexado por `ObjectKind`, não por
  contexto da entidade.

### 13.6 — O que a fusão parte

Nada de estrutura. **Quatro ficheiros**, todos texto de tabela ou prosa:
`crates/ph2d-component-desc/src/catalog/physics.rs` (a tabela) ·
`crates/ph2d-app-components/src/component_palette_tests.rs` (os dois rótulos no gate da cascata) ·
`shells/desktop/src/physics/physics_gesture_tests.rs` + `physics_gesture_zone_tests.rs` (prosa que
citava *Rigid Body*) · `docs/Components/05_plano_de_implementacao.md` (idem).
⚠️ **Uma linha da `line/components` que toque `catalog/physics.rs` colide textualmente** — a cura é
ler a tabela das três portas no cabeçalho e classificar a entrada nova, nunca aceitar o lado que
tiver mais `authored`.

### 13.7 — Portão e commit

**Commit:** `fb26bc48a` (sobre `5af3dbf78`, o fecho da 2.ª volta).

**Portão batched corrido INTEIRO sobre a árvore final, todos verdes:** `doc-index.sh` (+ `--check`) ·
`cargo fmt --all -- --check` · censo lexical (**4 638**, inalterado — os dois rótulos renomeados são
língua dos dois lados) · censo do `ph2d-tool-painter` · `cargo check --workspace --all-targets` (aviso
= erro) · `cargo clippy --workspace --all-targets -D warnings` · `cargo machete` ·
`check-standalone-optional` · `check-workflow-packages` · `typos` ·
**`nextest-impacted` (`BASE=1d43da737`): 13 874 testes, 13 874 passaram** (eram 13 867 na volta
anterior; o delta é a catraca nova e os gates que ela reabilita no grafo de impacto).

⚠️ **A máquina esteve a `load 33–50` durante o portão** (partilhada): nenhum relógio desta corrida é
medição de desempenho. Os vereditos são estruturais.

---

## 14 — ADENDO de 14/09 — UMA porta, e as outras duas voltam para DENTRO das secções

> *«Melhor como eu havia dito: um objeto de física (Physics Body) e todas as opções aparecem com ele
> (inclusive Collision Shape e Platform Player).»* — o dono, 2026-09-14, depois de ver a poda de três
> do §13.

⚠️ **Isto é um VEREDITO DE PRODUTO que reverte, parcialmente e só na física, a peça 1 da F3 do
ADR-0166.** Não é uma correcção de defeito e não deve ser lido como tal: a F3 apagou as faces vazias
porque a rota nova era o `+`; com a física a sair da paleta, a rota volta a ser a secção. *Quem move
o número que tornava algo inalcançável tem de reconferir a nota* (CLAUDE.md §0.0) — aqui quem se
moveu foi a paleta, e as duas notas reconferidas estão nos doc-comments de
`player_section_applies` e de `build_physics_info`.

### 14.1 — O que a paleta oferece agora: **uma** entrada

| Onde | O que | Porque não é um item de paleta |
|---|---|---|
| **`+` → Physics → Physics Body** (`RigidBody`) | *este objeto é simulado* | é a porta |
| **§11, ao nascer** (`Collider`) | forma · meias-extensões · offset · densidade · quique · atrito · camada · Solid\|Sensor | as opções **são** as rows da §11, e ela nasce com o corpo |
| **§11, face vazia** (`Collider` sozinho) | *Add Shape to X* — esta forma é mais uma peça do corpo acima | ⛔ a paleta não sabe **NOMEAR o dono**, e sem o nome o gesto não existe |
| **§14, face vazia** (`PlatformPlayer`) | *Make Platform Player* | ⛔ só faz sentido num corpo `Dynamic` (a mola é um impulso, e um impulso não move massa infinita), e o `+` genérico não sabe perguntar isso |

⭐ **As duas linhas de baixo são exactamente as portas que a F3 apagou**, e voltam **com o código que
ela removeu** — `PlayerFieldEdit::Add`, o `INSP_PLAYER_ADD` no `populate`, a face vazia do
`sections/player.rs`. O `git show 60315cb39` é a remoção original; esta é a inversa dela, com a
diferença de que o *seed* passou a ser uma porta (`attach_player` chama `seed_attached_player`, que o
`+` também usa) em vez das duas construções que existiam antes.

### 14.2 — As TRÊS metades da §14, e porque é um `||`

`player_section_applies(kind, has_player) = has_player || kind == Dynamic`

| metade | o defeito que ela apanha |
|---|---|
| `Dynamic` **sem** o componente ⇒ **com** secção | a porta a desaparecer: o comportamento fica inalcançável, porque já não está na paleta |
| `Static` sem o componente ⇒ **sem** secção | um botão que a física recusa em silêncio |
| `Static` **com** o componente ⇒ **com** secção | um componente presente e **invisível** — o caso de um player que um bake pôs `Kinematic` |

⚠️ **A F3 tinha razão em apagar a 2.ª metade ENQUANTO a porta era o `+`** (mantê-la dava *«o artista
anexa pelo `+` e nada aparece»*, e o doc dela dizia-o). Com a porta de volta à secção, a condição de
**oferecer** volta a ser a condição de **pintar** — e é isso que faz a pergunta *«a secção
aparece?»* e a pergunta *«esta edição é legal?»* serem outra vez a mesma, que é o que torna seguro o
`apply_player_edit` guardar-se pela mesma função.

### 14.3 — A §11 alcança um FILHO de um corpo

`build_physics_info` devolvia `None` para quem não tem `RigidBody`, nem `Collider`, nem `rig_parts`.
Acrescenta-se **`&& part_owner.is_empty()`**: um objecto com um corpo acima na árvore mostra a §11
com a face vazia, onde vive o *Add Shape to X*.

⚠️ **É a MESMA excepção do `rig_parts`, pela mesma razão e não por folga** — aquela já estava
escrita ali: *«um gesto sobre uma SUBÁRVORE, que a paleta de componentes não sabe exprimir»*. Aqui é
um gesto sobre o **corpo ancestral**, que a paleta não sabe **nomear**. ⭐ E o `nearest_body_name`
passa a correr **uma vez** (era chamado em dois ramos), o que é a única mudança de custo.

### 14.4 — `Attach::Intrinsic` **com** `requires`, e o gate que nomeava o recurso errado

⛔⛔ **`every_declared_requirement_names_a_real_component` exigia que o alvo de um `requires` fosse
`Authored`, e a razão escrita ao lado era FALSA:** *«é Intrinsic — a cascata não o consegue
construir»*. Quem constrói a cascata é o **`insert_default` do `ComponentRegistry`**
(`attach_by_name` → `attach_one`), e ele **não consulta o `attach`**. Um `Intrinsic` com `Default` —
o `Collider` é exactamente esse caso — constrói-se perfeitamente.

⇒ o gate passa a exigir o que de facto importa: **o registo do produto sabe construir o alvo**. Ele
fica **mais forte**, porque um `requires` que aponte a um tipo sem `insert_default` passava antes e
reprova agora. *Um limite legítimo diz de que recurso ele é* (§0.0), e este dizia de um recurso de
outro subsistema.

⭐ E nasce `ComponentDesc::intrinsic_requiring`, para o `PlatformPlayer`: **o `attach` responde *quem
escolhe*, o `requires` responde *é inerte sem o quê***. O segundo continua verdadeiro para um
componente que saiu da paleta — e sem ele um chamador de `attach_by_name` (um teste, um script)
deixava o player numa entidade sem corpo, a não fazer nada, em silêncio.

⚠️ **E o piso de população do `the_require_graph_has_no_cycles` ganhou um NOME:** `seen >= 2` não
distingue *«a população encolheu por uma decisão»* de *«alguém apagou o `requires` e o gate anda
sobre nada»*. Hoje ele também **ancora a cascata canónica** (`RigidBody → Collider`) por nome, com a
mensagem a dizer o que fazer se ela mudar mesmo.

### 14.5 — Medição (sonda `measure_palette`, viewport 1187×953 do report de 25/08)

| caso | antes do §13 | depois do §13 | **hoje** |
|---|---|---|---|
| itens oferecidos, total | **85** | 58 | **56** |
| `Empty` · aplicável | 56 | 29 | **27** |
| `Image` · aplicável | 71 | 44 | **42** |
| `Empty` · *Show all* — o que não cabe | 238 px | 46 px | **46 px** |
| `Image` · *Show all* — o que não cabe | 175 px | 0 px | **0 px** |

Família de física: **30 `Authored` → 1**. ⚠️ Os dois últimos passos (`58 → 56`) são pequenos **na
contagem** e são a decisão inteira **na semântica**: o que saiu foram as duas entradas que o artista
lia como *«outra coisa que se adiciona»* quando são partes do mesmo objecto.

### 14.6 — Provas de mutação — 5/5 MORTAS, cada uma pelo NOME do teste que a devia apanhar

Restauro por cópia + `touch` + comparação byte a byte; carga impressa ao lado de cada corrida.

| # | defeito injectado | quem o matou |
|---|---|---|
| M11 | `player_section_applies` volta a `has_player` | `the_section_follows_the_component_or_a_dynamic_body` **e** `the_player_section_belongs_to_every_dynamic_body` |
| M12 | tirar o `&& part_owner.is_empty()` do `build_physics_info` | `the_physics_section_reaches_a_child_of_a_body` (a metade do sprite ÓRFÃO) |
| M13 | `i("…::Collider", …)` volta a `D::authored(…)` | `the_physics_family_offers_one_door_and_not_its_rows` (`left` nomeia o intruso) |
| M14 | o `INSP_PLAYER_ADD` sai do `populate_player` | `the_door_of_the_empty_face_is_registered` |
| M15 | o botão da face vazia deixa de ser pintado | `the_empty_face_paints_the_door_and_nothing_else` |

### 14.7 — Os gates que INVERTERAM, e porquê isso não é fraqueza

Quatro gates afirmavam a lei da F3 e passam a afirmar a lei restaurada. **Nenhum foi apagado** — os
quatro ficam, com as duas metades e com a história dentro:

| gate | era | é |
|---|---|---|
| `seam_player::the_dead_empty_face_paints_nothing_at_all` | nada é pintado sem o componente | `…the_empty_face_paints_the_door_and_nothing_else` — **o botão e mais nada** |
| `seam_player::the_dead_empty_face_leaves_no_registration_behind` | o id NÃO está registado | `…the_door_of_the_empty_face_is_registered` — os **seis** estão |
| `inspector_player_tests::the_section_follows_the_component_not_the_body_kind` | sem componente, nunca | `…_or_a_dynamic_body` — com o `assert_eq` sobre o `kind` |
| `inspector_player_tests::removing_the_behaviour_closes_the_section` | a secção fecha | `…_leaves_the_door_open` — ela fica, com `has_player = false` |

⚠️ **E o `inspector_presence_tests` perdeu uma linha da tabela e ganhou DOIS testes nomeados:** a §14
saiu do `CASES` porque a lei dela deixou de ser *«aparece se e só se o componente está lá»* — é a
mesma forma da §7 Ordering, que já vivia fora da tabela pela mesma razão. A metade *«um objecto
pelado não mostra a §14»* está afirmada **explicitamente** dentro do teste novo, porque sair da
tabela é sair do `an_empty_object_still_shows_transform_and_name`.

⚠️ **Uma nota HISTÓRICA foi corrigida em vez de reescrita:** o
`the_painted_control_reaches_a_consumer` regista que o `INSP_PLAYER_ADD` foi apagado em 30/08 como
**registo órfão**. A entrada continua verdadeira sobre o dia em que foi escrita e seria uma armadilha
lida hoje — leva agora a linha que diz que ele voltou, **com o botão**. *Um órfão cura-se apagando; o
que o des-orfanou foi o botão nascer outra vez.*

### 14.8 — O que a fusão parte

**14 ficheiros**, e a superfície é a mesma do §13 mais a §14 do Inspector:

- `ph2d-component-desc`: `lib.rs` (construtor novo) · `catalog/physics.rs` (a tabela)
- `ph2d-app-components`: `component_palette_tests.rs` (dois gates)
- `ph2d-editor-core`: `inspector_model_player.rs` (`PlayerFieldEdit::Add`) · `tests/it/the_painted_control_reaches_a_consumer.rs` (prosa)
- `ph2d-i18n`: `inspector_player.rs` (**duas chaves novas**)
- `ph2d-panel-inspector`: `sections/player.rs` · `event_player.rs` · `populate_player.rs` · `tests/it/seam_player.rs`
- `ph2d-app-physics`: `inspector/player.rs` · `inspector/body.rs`
- `shells/desktop`: `inspector_presence_tests.rs` · `physics/inspector_player_tests.rs`

⚠️ **Uma linha da `line/components` que toque `catalog/physics.rs` colide textualmente** — a cura é
ler a tabela no cabeçalho e classificar a entrada nova, nunca aceitar o lado com mais `authored`.
⚠️ **E quem integrar uma linha que mexa na §14 vai encontrar `PlayerFieldEdit::Add` de volta**: se o
outro lado o tiver apagado por seguir a F3, o lado certo é este.

### 14.9 — O que só o portão de fecho apanhou

| vermelho | o que era | cura |
|---|---|---|
| `architecture_panel_loc_cap::panel_functions_under_loc_cap` | `paint_player_section` a **203** LOC (teto 200) — a face vazia que voltou | **corte por responsabilidade**: a porta saiu para `sections/player_door.rs` (`213 → 196`) |
| `the_tail_of_a_block_is_one_answer::…_is_never_written_at_the_painting_site` | o corte **criou** o vermelho: dentro do `player.rs` o `+ Spacing::Sm.px()` vivia como ARGUMENTO de uma chamada e a régua textual não o via; como **cauda** de uma função, vê | passa pela porta `ph2d_tokens::control_gap_px()` |
| `ph2d-app-flip …::a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` | **flake de recurso sob fan-out**, já NOMEADA no `CLAUDE.md` §5.0 (a família `flip_smooth::resample_measurement::precisao::orcamento`) | nenhuma — **3/3 verde sozinha a `load 82,88`**, e o diff desta volta tem **zero linhas** naquela crate |

⭐⭐ **O 2.º vermelho é instrutivo por ser CAUSADO pela cura do 1.º:** *mover código não muda o que
ele faz — muda o que as réguas conseguem ver*. É a espécie «falha alto» do HOWTO §2, do lado bom.

⚠️⚠️ **E o valor MUDA com ele: `6 px → 3 px`.** O `Spacing::Sm` é `6`; o `control_gap_px()` é
`widget_margin_y − 2 = 3`. ⛔ **Não é regressão** — é esta face a entrar na escada `1 / 3 / 8` em que
os outros 78 sítios já entraram na wave 20 (ela morreu na F3 **antes** daquela wave e voltou
**depois** dela, logo nunca lá tinha passado). O botão fica 3 px mais colado ao fim da secção.

⛔⛔ **E a tabela do CABEÇALHO daquele gate estava DESACTUALIZADA CONTRA O PRÓPRIO CÓDIGO**, a
ensinar `6` para este degrau — foi ela que me levou a escrever `Spacing::Sm` na 1.ª redacção. A wave
20 fundiu o `block_gap` no `control_gap_px` e corrigiu o doc-comment **da porta**, sem voltar ao doc
**do gate**. Corrigida aqui, com a nota do porquê. *Quando duas páginas imprimem a mesma grandeza e
discordam, a que manda é o código* — e foi preciso um sítio novo acreditar na errada para alguém
reparar.

⚠️ **O teto de FICHEIRO decidiu a forma do corte, e não a aritmética:** o `player.rs` estava
**exactamente** em `600` de `600`, logo o bloco não podia crescer lá dentro de maneira nenhuma —
*dois tectos diferentes a apontar para o mesmo corte é o sinal de que ele é por responsabilidade*. O
irmão segue a linha que o `physics_doors.rs` já desenhava um nível acima: *o que este player É* ×
**o que CRIAR aqui**.

⛔⛔ **E o veredito do portão quase passou como VERDE por uma armadilha da FERRAMENTA, não do
código:** o `grep` dentro do shell deste agente é um wrapper para o **ugrep**, e nele
**`grep -qv PADRÃO` devolve «não encontrei» SEMPRE** — o vigia que eu tinha armado anunciou
*«GATE VERDE: todos os passos exit=0»* sobre um ficheiro com `exit=100`. O que salvou foi ler o
ficheiro à mão. ⚠️ **Os scripts do repo estão a salvo** (correm fora desta shell, onde `grep` é o GNU;
os dois `grep -qv` vivos em `scripts/` estão correctos lá) — *a armadilha é do lado que MEDE*.
Registo: [`feedback_the_grep_in_this_agents_shell_is_ugrep_and_qv_always_says_no`](../../../project-memory/feedback_the_grep_in_this_agents_shell_is_ugrep_and_qv_always_says_no.md).

### 14.10 — ⏳ ABERTO

- **As outras famílias continuam por auditar com esta régua**, e a `core` (22 `D::authored`) tem
  candidatos claros a row (`Blend Mode`, `Texture Filter`, `Order In Layer`, `Locked`, `Visibility`,
  e o `Transform`, que **toda** entidade tem). ⛔ Falta-lhe o gémeo do
  `every_registered_physics_component_has_a_ui_writer`, que é o censo que autorizou esta poda — uma
  poda ali **começa por construir o instrumento**.
- **A §14 aparece em todo corpo `Dynamic`**, o que é um cabeçalho e um botão a mais no Inspector de
  um caixote. É o preço declarado da decisão do dono; se ele o achar ruído, a saída barata é a
  secção nascer **dobrada** (o `SectionFold` já existe), não voltar a escondê-la.
