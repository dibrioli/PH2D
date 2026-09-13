# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-09-13

> Entregável que **fecha a jornada** (`CLAUDE.md` §0.7, DIRETRIZ §1.5.9). A linha **não integra e não
> pusha**: entrega isto e para.
>
> ⚠️ **Leia o §6 antes do primeiro `git merge`.** Três coisas desta jornada mordem uma linha paralela
> em silêncio: o elo novo na cadeia do `ph2d_i18n::tr`, a catraca de dívida da `editor-core` (agora
> exacta nos DOIS sentidos) e o helper «é teste?» que mudou de casa.

## 1 — Identidade

| | |
|---|---|
| branch | `line/UIUX` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX` |
| HEAD | o commit deste handoff, sobre `6107d66e5` (o último de código) |
| merge-base com `main` | `1d43da737` (o `main` enviado, CI verde, da refatoração final) |
| commits | **5** de código (`47ebe5eba` · `6b23407d8` · `63dc167f4` · `f3b2450d9` · `6107d66e5`) + o de docs deste handoff |

## 2 — O que a jornada fez

1. **A régua do texto de interface estava cega à maior parte dele**, e agora não está: crate-folha nova
   [`ph2d-label-census`](../../../crates/ph2d-label-census/src/lib.rs) — a régua LEXICAL, o censo de
   chaves dos dois lados e o helper «é este ficheiro um módulo de teste?» —, usada por três gates, com
   13 controlos (um por forma que já custou um censo errado) e o exemplo `censo` (TSV com índices de
   carácter, e `--resumo`).
2. **O painel Painter e a Hierarquia falam pela tabela de strings**: `376 + 6` textos a zero, com gate
   por crate a zero e excepções com mecanismo.
3. **O gate molde da `editor-core` deixou de afirmar «língua de produto 0»**: a dívida real (`546` em
   `46` ficheiros) entrou ao número, numa catraca exacta nos dois sentidos.

## 3 — Foundational / partilhado tocado, e porquê

| onde | o quê | porquê |
|---|---|---|
| `crates/ph2d-label-census` | **crate NOVA**, zero dependências, `[lints] workspace = true` | a régua partilhada: o gate molde mandava COPIAR ~700 linhas de leitor para cada crate |
| `crates/ph2d-i18n/src/lib.rs` | `mod painter_layers;` · **UM elo** na cadeia do `tr` · `pub fn tr_with` · 5 chaves `panel.hierarchy.*` | a tabela irmã do Painter; a porta das frases com peças do código |
| `crates/ph2d-i18n/src/painter_layers.rs` | **ficheiro NOVO**, 364 chaves, braços entre marcadores `ph2d-migrar-texto` | a tabela do Painter, por assunto (como `vector.rs`/`sculpt3d.rs`/`chrome.rs`) |
| `crates/ph2d-panel-painter-layers` | 38 ficheiros de `src/` + 2 novos (`paint_seg_row.rs`, `paint_texture_tiling.rs`) + gate + dep `ph2d-i18n` (e dev `ph2d-label-census`) | a migração; os dois novos são CORTES de tecto (§6.6) |
| `crates/ph2d-panel-hierarchy` | `src/paint_head.rs` + gate + dep `ph2d-i18n` (e dev `ph2d-label-census`) | a migração |
| `crates/ph2d-editor-core/tests/it/no_label_of_this_crate_is_written_in_the_painter.rs` | **reescrito** sobre a folha, com a `DIVIDA` exacta e um teste novo (`the_debt_only_describes_what_is_still_there`) | §6.2 |
| `crates/ph2d-editor-core/tests/common/cfg_test_modules.rs` | passa a **re-exportar** `ph2d_label_census::cfg_test` (dev-dep nova da `editor-core`) | §6.3 |
| `crates/ph2d-editor-core/tests/it/hr15_no_hardcoded_ui_strings.rs` | **sai** `("ph2d-panel-hierarchy/src/paint_head.rs", 1)` | o `"Search…"` saiu do BINÁRIO, não só do scanner |
| `crates/ph2d-editor-core/tests/it/hr12_widgets_a11y.rs` | **entra** `ph2d-panel-painter-layers/src/paint_texture_tiling.rs` no `PANEL_A11Y_DELEGATE_OK` | o corte delega em `number_field`, como o `paint_stencil.rs` |
| `scripts/migrar-texto-pintado.py` | **NOVO** | o migrador que confere cada literal no sítio (§11.6) |
| `scripts/censo-texto-pintado.py` | varre a árvore inteira + diz, no topo, que é a régua PARCIAL | varria por prefixo (§9.6) |
| `docs/UI_New_and_Simple/` | `ferramentas/seccoes_painter_layers.tsv` (novo) · `medicoes/07` §2-quater · este handoff | — |

⛔ **Nenhum contrato congelado**, **nenhum ADR**, **nenhum pacote EXTERNO novo** (o `+ph2d-label-census`
do `Cargo.lock` é a crate nova, `path`), **nenhum schema** e **nenhum registo de componentes** movido.
⭐ **`shells/desktop`: zero linhas** — o tecto `the_shell_only_shrinks` não se mexe.

## 4 — Superfície de colisão (colada; corrida do primário sobre `f3b2450d9`)

```
SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 1d43da737   ·   4 commit(s)   ·   70 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      22   (base: 22)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               85   (base: 85)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-label-census"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
```

⚠️ O `⚠ 1 pacote` é a crate NOVA desta linha (`path = "../ph2d-label-census"`), não um externo.

## 5 — O que só o `ship.sh` apanha

Corridos TODOS nesta worktree, pelo portão batched (§12), sobre `6107d66e5`:

| | estado |
|---|---|
| `cargo fmt --all -- --check` | ✅ — a 1.ª corrida acusou o gate molde reescrito (curado em `6107d66e5`) |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ✅ 0 avisos — ⚠️ a 1.ª corrida reprovou NA FOLHA e não lintou nada acima dela (§11.9) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ 0 avisos |
| `cargo machete` | ✅ nenhuma dependência sem uso (as arestas novas `ph2d-i18n` e `ph2d-label-census` são usadas) |
| `bash scripts/check-standalone-optional.sh` | ✅ as 10 crates com dependência interna opcional |
| `bash scripts/check-workflow-packages.sh` | ✅ 32 nomes citados contra 360 membros |
| `typos --force-exclude` nos 70 ficheiros tocados | ✅ 0 |
| `bash scripts/doc-index.sh --check` | ✅ 19 índices — o `handoffs/README.md` regenerado vai no commit deste handoff |
| `cargo deny` / `cargo audit` | ⚠️ não corridos: nenhum pacote EXTERNO novo nesta linha |

## 6 — ⚠️⚠️ O QUE VAI PARTIR NA FUSÃO, e é previsível

### 6.1 A cadeia do `ph2d_i18n::tr` ganhou UM elo

```rust
k => vector::tr(k)
    .or_else(|| sculpt3d::tr(k))
    .or_else(|| model3d::tr(k))
    .or_else(|| chrome::tr(k))
    .or_else(|| painter_layers::tr(k))   // ← este
    .unwrap_or_else(|| leak_key(k)),
```

⛔ **Resolução: ACUMULAR os elos, nunca escolher um lado** (a regra do handoff de 10/09 §6.7).
⛔⛔ **E NENHUM gate apanha o elo esquecido:** o `every_painter_key_exists_on_both_sides` lê a TABELA,
não a cadeia — o sintoma seria o painel Painter a pintar `panel.painter_layers.x.y` na tela, com um
`Box::leak` por quadro. Confirme o elo por leitura depois do merge.

### 6.2 ⛔⛔ A catraca de dívida da `editor-core` é EXACTA nos dois sentidos

`DIVIDA: &[(&str, usize)]` — 46 ficheiros, contados pela régua lexical. Na árvore combinada:

- **uma linha que ACRESCENTE texto** a um ficheiro da `editor-core` (um item de menu, um aviso)
  reprova o `every_label_this_crate_paints_comes_from_the_string_table`. ⛔ **A cura é migrar esse
  texto para a tabela** (`chrome.rs` ou o irmão do assunto), **nunca subir o número** — e é um achado
  a devolver à linha que o trouxe;
- **uma linha que APAGUE texto** reprova o `the_debt_only_describes_what_is_still_there` — a cura é
  **descer o número** para o que a régua conta (a mensagem diz qual).

⚠️ Recontar: `cargo run -q -p ph2d-label-census --example censo -- crates/ph2d-editor-core/src | cut -f1
| grep -v '^widget/showcase/' | sort | uniq -c`.

### 6.3 O helper «é este ficheiro um módulo de teste?» mudou de casa — e ficou mais forte

`ph2d-editor-core/tests/common/cfg_test_modules.rs` é agora uma re-exportação de
`ph2d_label_census::cfg_test`. Uma linha que tenha editado o corpo antigo conflita no ficheiro inteiro:
⛔ a cura é levar a edição para `crates/ph2d-label-census/src/cfg_test.rs`.

⭐ **Ele passou a reconhecer `x/mod.rs`** (§9.4). O `hr12`, o `hr15` e o `node_id_collisions` usam-no e
passaram nesta árvore — mas uma linha paralela cuja contagem dependesse de um `…/tests/mod.rs` ser lido
como produto veria o número mudar.

### 6.4 Os dois gates por painel estão a ZERO

`every_word_this_panel_shows_comes_from_the_string_table` em `ph2d-panel-painter-layers` e em
`ph2d-panel-hierarchy`. Uma linha paralela que acrescente texto a um desses painéis reprova na árvore
combinada — a cura é uma chave `panel.painter_layers.<secção>.<nome>` / `panel.hierarchy.<nome>`.

### 6.5 O `hr15` perde uma entrada e o `hr12` ganha uma

Conflito TEXTUAL possível com linhas que editem as listas vizinhas; semântico, nenhum.

### 6.6 Dois cortes de tecto no painel Painter, os dois por responsabilidade

- `paint_impasto.rs` `604 → 539`: a fileira segmentada (`seg_row` / `seg_row_owned`) saiu para
  `paint_seg_row.rs` — ela tinha **três** consumidores (Impasto, rig de luz, Wet Paint) e a morada de um
  deles. Os chamadores nomeiam `crate::paint_seg_row::…`; ⛔ sem fachada.
- `paint_texture_section` `205 LOC → ~155`: Offset/Size/Depth saíram para `paint_texture_tiling.rs`.

⚠️ Nenhuma linha viva é dona do painel Painter nem da Hierarquia (medido na abertura: as cinco linhas
reabertas a 0 commits; a `line/motion-value` com ficheiros sujos só em `ph2d-app-motion`).

## 7 — Ordem e o que smokar

**Ordem:** cronológica, `--ff-only` numa vez.

Nada muda à vista — o smoke prova que **continua tudo igual**, e o modo de falha a procurar é o de §6.1:
um rótulo que aparece como `panel.painter_layers.…` ou `panel.hierarchy.…`.

| onde | o que ver |
|---|---|
| **Painter** (os dois lados do cabeçalho: *Brush* e *Layers*) | todas as secções abertas: Stroke (Tiling · Jitter · Grid · Offset), Shape, Grain (Offset/Size/Depth — o bloco que mudou de ficheiro), Symmetry, Taper, Impasto (Sculpt · Material · Body · Lighting · Knife), Watercolor + Paper, Wet Paint (e a leitura `fluid …x… - flow …x…`), Selection, Mask, Deform, Clone, Inpaint, Line (o menu de tipos e as rows de cada tipo) |
| **Painter**, a trocar de ferramenta no trilho | o TÍTULO do cabeçalho muda: Brush · Eraser · Blur · Smear · Clone · Mask · Inpaint · Select · Sculpt · Deform |
| **Shape → Use Texture Colors** | as caixas dizem `Layer 1 Color`, `Layer 2 Color`, … |
| **Hierarquia** | o título `Hierarchy`, a contagem `N entities · M components`, o `Search…` vazio e a dica do botão `Add` |

**Comando (copiável de uma vez):**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --profile smoke
```
O binário já está compilado nesta worktree (§12). ⚠️ `~/.ph2d/layout.txt` guarda a arrumação dos painéis —
um ficheiro velho abre o app com um painel fechado, e apagá-lo é o reset.

✅ **Smoke APROVADO pelo dono em 2026-09-13** (*«smoke OK»*), sobre o binário `smoke` desta worktree compilado
da árvore de `9bdf86631` (o commit desta nota só toca este ficheiro). Aprovar o smoke **não** é ordem de
integrar (CLAUDE.md §0.7): a linha continua parada à espera dela.

## 8 — ⏳ ABERTO (não corrigir na integração)

| item | de quem depende |
|---|---|
| **`ph2d-panel-inspector`: 582 literais** — o próximo painel na ordem da `medicoes/07` §4 (aberto o dia inteiro) | ⚠️ a `line/components` escreve secções nele: por rodada, ou pela linha dona |
| **`ph2d-editor-core`: 546 em 46 ficheiros** (`menu_rows` 164 · `ids/menus_timeline` 50 · `left_rail` 45 · `topbar/chip_name` 35 · `topbar/tooltips` 28 …) — e por TRIAR (`screens/hero/fixture.rs` são nomes da cena de amostra) | desta linha |
| **as `ph2d-app-*` e a shell**: o grosso dos 5 035, com três espécies misturadas (avisos de interface · nomes de objectos de cenas de smoke · narração de smoke) | triagem ANTES de virar dívida; das linhas donas |
| **o `Panel::TITLE` é `const &'static str`, e a ABA não fala pela tabela** | mudança do trait `Panel` nos 26 painéis — não é contrato congelado, mas é uma edição por crate de painel de outras linhas ⇒ item de rodada |
| **texto pintado por MÉTODO de outra crate** (`BrushBlend::name()`, `Falloff::name()`, `TextureKind::name()`, `TextureMapping::name()` na `ph2d-tool-painter`, que não tem gate) | a régua conta **20** literais na `ph2d-tool-painter`, e os `name()` dos enums que o painel pinta por método não têm gate na crate deles; da linha dona do Painter |
| **o degrau `G`** (esvaziar os painéis por âmbito) | ⭐ agora TEM vocabulário no Painter (364 chaves) e na Hierarquia (5) — o censo pode correr nestes dois |
| do handoff de 10/09, sem mudança: pose 2D/3D · partir o `DrawMode` · os 9 toggles → Layout | do **dono** |
| o «travou por um minuto» de 09/09 | sem reprodução |

## 9 — As premissas que a medição derrubou

1. ⛔⛔ **«418 literais pintados em 19 crates»** (o `CLAUDE.md` §5 e o handoff de 10/09) — era o PISO de
   uma régua que só seguia o texto até um pintor. A lexical conta **5 035** nas crates de UI depois desta jornada; nos
   painéis, onde a amostra é quase só língua, **~1 330** ficam por migrar. ⚠️ O protótipo em Python desta jornada dava `5 533` —
   contava ficheiros de teste que o helper não reconhecia; o número de registo é o da folha.
2. ⛔⛔ **«A crate desta linha está curada: língua de produto 0»** (handoff de 10/09 §2-ter) — **546** em
   **46** ficheiros. A cura de 10/09 foi medida pela régua cega; o gate molde afirmava-o por escrito.
3. ⛔ **«Cada linha dona de uma crate copia o gate molde»** — copiar a régua fazia N réguas da mesma
   grandeza na mesma linguagem; ⇒ uma folha partilhada.
4. ⛔ **«O `cfg_test_modules` pergunta ao pai em todas as formas»** — cego a DUAS: a `x/mod.rs` (os 12
   ficheiros de `interaction/dispatch/tests/` e os de `ph2d-tool-painter/src/tool/paint/tests/` eram
   produto) e a um **comentário no fim da linha do `mod`** (`mod line_seam_tests; // o seam do card Line`
   lia-se como o módulo `Line)`, e o `line_seam_tests.rs` e o `emboss_probe.rs` da `ph2d-tool-painter`,
   declarados sob `#[cfg(test)]`, contavam como produto).
5. **«19 de 26 painéis não têm chave»** — agora 17.
6. ⛔ **«O `censo-texto-pintado.py` varre a UI»** — varria por prefixo (`ph2d-panel-*` +
   `editor-core`), e o código de família que a W2 levou para `ph2d-app-*` ficou fora dele em silêncio.
7. ⛔ **(minha) «a régua lexical nasceu certa»** — deitava fora todo texto com um escape do fonte (§11.2).

## 10 — Para o `CLAUDE.md` §5 — UMA linha de estado (a `Aberto:` do módulo UI/UX)

> **Aberto:** ⭐ o texto da interface tem régua que VÊ tudo — a folha
> [`ph2d-label-census`](crates/ph2d-label-census/src/lib.rs) (lexical, 13 controlos; `cargo run -q -p
> ph2d-label-census --example censo -- --resumo`) — e o **Painter** (`376 → 0`) e a **Hierarquia**
> (`6 → 0`) já falam pela tabela, com gate por crate a zero · ⛔ o `418` era um PISO: a régua conta
> **5 035** nas crates de UI (o `ph2d-panel-inspector`, `582`, é o próximo painel; a `editor-core` tem
> **`546`** em 46 ficheiros numa catraca exacta que só desce; as `ph2d-app-*` e a shell pedem triagem
> antes de virar dívida) · o `Panel::TITLE` não fala pela tabela (item de rodada, 26 painéis) · **três
> decisões do dono**, todas já com o número ao lado: a pose 2D/3D (`722` sítios de produto, 5 crates),
> partir o `DrawMode` nos dois eixos (`17` variantes vivas) e os 9 toggles de módulo → Layout · o
> **«travou por um minuto»** de 09/09 segue **sem reprodução** · ⏳ as superfícies de UI que as outras
> linhas trouxeram foram escritas contra a lei de espaçamento ANTIGA.
> **Ler:** … · [handoff de 13/09](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-13.md)
> (o §6 é o que a fusão parte — o elo do `tr` que NENHUM gate apanha, a catraca exacta da `editor-core`
> e o helper «é teste?» que mudou de casa)

## 11 — As leis que esta jornada pagou

1. ⭐⭐⭐ **Declarar um ponto cego não o torna pequeno.** A régua anterior dizia por escrito que não via
   construtores, tabelas e `format!` — e essas três formas eram mais de metade de um painel (Painter
   `166` contra `376`) e 95 % da `editor-core` (`32` contra `641`). *Um ponto cego declarado continua a
   ter tamanho, e o tamanho é a pergunta.*
2. ⭐⭐ **Uma régua nova lê-se contra um caso que se CONHECE.** A régua lexical nasceu a deitar fora
   todo texto com um escape do fonte (`\u{00b7}` lido como caminho) — mudo. Quem o apanhou foi uma
   frase concreta que eu sabia existir e que não aparecia na contagem da Hierarquia.
3. ⭐⭐ **Um helper movido byte a byte leva os pontos cegos junto.** O `cfg_test_modules` foi para a folha
   intacto, e com ele DUAS cegueiras — a `x/mod.rs` e ao comentário no fim da linha do `mod`; foi a
   régua nova que as expôs, a contar `"hello world"` e `"<missing>"` como rótulos da interface.
4. ⭐ **Um censo do repo inteiro lê as fixturas dele próprio.** O controlo da folha usava
   `tr("panel.painter_layers.size")` como amostra, e o gate de chaves do Painter reprovou por uma chave
   «usada e não declarada» dentro do ficheiro de testes da régua.
5. ⭐ **Uma tabela `const` guarda a CHAVE, e quem pinta traduz** — e as duas tabelas do mesmo ficheiro
   (`ParamSlider`/`ParamCheckbox`) ficaram com o MESMO contrato: com um a guardar texto e o outro a
   guardar a chave, uma row chegaria traduzida duas vezes.
6. ⭐ **Uma migração conferida no sítio é a única migração em massa honesta.** O
   `migrar-texto-pintado.py` compara o texto nos índices de carácter que a régua deu com o esperado e
   aborta ANTES de escrever — 373 trocas, zero `replace` mudo.
7. ⭐ **Uma migração que alonga linhas empurra tectos de dentro da própria linha** (`paint_impasto.rs`
   `604`, `paint_texture_section` `205`), e o corte por responsabilidade achou uma porta com a morada
   errada: a fileira segmentada tinha três consumidores e morava num.
8. ⚠️ **Uma prova de mutação lê o NOME do teste que reprovou, nunca só o código de saída.** A M7 saiu
   `1` duas vezes com `0 falharam · 0 passaram`, e a 1.ª leitura minha foi «carga» (`load 79`) — ⛔
   **errada**: a âncora `tr("chrome.done")` casava DENTRO de `ph2d_i18n::tr("chrome.done")`, o mutante
   era `ph2d_i18n::"Done"` e nem compilava. *Uma âncora de mutação tem de ser a expressão inteira, não
   um pedaço dela.* ⚠️ E o `scripts/cargo-test-narrow.sh` devolveu **1** («teste vermelho») para um erro
   de COMPILAÇÃO da lib, contra o contrato escrito dele (**2**) — nomeado, não curado aqui.
9. ⚠️ **Um `clippy --workspace` que reprova numa FOLHA não linta ninguém que dependa dela.** O portão
   batched reprovou com UM erro (`explicit_counter_loop` na `ph2d-label-census`) — e a `editor-core`, os
   painéis e a shell nunca chegaram a ser lintados nessa corrida. *Um vermelho numa folha não é «um
   vermelho»: é o veredito de todas as crates acima dela a ficar por dar.* O portão correu de novo,
   inteiro, sobre a árvore curada.

## 12 — Estado do portão e higiene de fecho

### O portão batched (regra E), sobre `6107d66e5` — um log por passo, o veredito é o EXIT

| passo | resultado | `load` na partida |
|---|---|---:|
| `doc-index.sh` + `--check` | ✅ 19 índices | 14,09 |
| `cargo fmt --all -- --check` | ✅ | 14,09 |
| `censo --resumo` (régua lexical) | ✅ 5 035 nas crates de UI | 13,20 |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ 0 avisos | 8,61 |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ✅ 0 avisos | 8,61 |
| `cargo machete` | ✅ | 15,55 |
| `check-standalone-optional.sh` | ✅ 10 crates | 15,55 |
| `check-workflow-packages.sh` | ✅ 32 nomes / 360 membros | 15,11 |
| `BASE=1d43da737 nextest-impacted.sh` (por `ph2d-run.sh`) | ✅ **13 861 / 13 861**, 11 164 fora do alcance | 15,11 |

⚠️ A 1.ª corrida do portão (sobre `f3b2450d9`) reprovou em `fmt` e em `clippy` e foi **parada** antes do
`nextest` (§11.9); as curas entraram em `6107d66e5` e o portão correu de novo, inteiro.

### Nenhum teste desaparece da suíte

As únicas funções apagadas desde o merge-base são os ajudantes do leitor copiado do gate molde
(`files`, `painted_literals`, `repo_root`), nenhuma um teste; os três nomes de teste que o gate molde
tinha ficam. Tudo o resto é NOVO: 13 na `ph2d-label-census`, 3 no Painter, 3 na Hierarquia e 1 na
`editor-core` (`the_debt_only_describes_what_is_still_there`). ⚠️ Prova por leitura do diff — o
`nextest list` contra o merge-base não foi corrido.

### Prova de mutação — 8 de 8 mortas, cada uma pelo NOME do teste que reprovou

Restauro por `cp` + `touch` e `cmp` byte a byte (um `mv` devolve mtime antigo e o cargo guarda o build
da mutação).

| # | mutação | teste que reprovou |
|---|---|---|
| M1 | a régua deixa de tirar os escapes do fonte | `a_source_escape_is_not_a_path_and_is_not_a_placeholder` |
| M2 | o helper volta a não reconhecer `x/mod.rs` | `every_shape_of_a_test_module_is_test_and_the_product_neighbour_is_not` |
| M3 | padrões e comparações passam a contar | `what_never_paints_is_left_out` |
| M4 | um rótulo do Painter volta a literal (`"Quality"`) | `every_word_this_panel_shows_comes_from_the_string_table` (Painter) |
| M5 | uma chave do Painter com erro de escrita | `every_painter_key_exists_on_both_sides` |
| M6 | a excepção da Hierarquia deixa de abrigar o literal dela | `every_named_exception_still_shelters_a_real_literal` (Hierarquia) |
| M7 | a dívida da `editor-core` CRESCE (`ph2d_i18n::tr("chrome.done")` → `"Done"`) | `every_label_this_crate_paints_comes_from_the_string_table` — *«prefab_bar.rs — 5 literais, a dívida tolerada é 4»* (3.ª corrida, crua; as duas primeiras eram mutante que não compilava, §11.8) |
| M8 | a dívida escrita fica ACIMA do que a régua conta | `the_debt_only_describes_what_is_still_there` |

### Higiene (DIRETRIZ §1.5.9 itens 7 e 9)

- ✅ `rm -rf target/*/incremental`: **16 GB** libertados (0 directórios restantes).
- ✅ Binário do smoke compilado em perfil `smoke` sobre a árvore final (`load 30,52` na partida): a 1.ª
  compilação recompilou só a shell em **1 min 12 s**; a 2.ª, colada inteira — **zero `Compiling`**:

```
$ cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

Binário: `target/smoke/ph2d-host-desktop`, 79,5 MB. ⚠️ Nenhum smoke desta jornada pede `--features`.

---

## ⛔ Recusas MEDIDAS desta jornada

| o que | por que não |
|---|---|
| copiar o leitor para cada crate de painel (o que o gate molde mandava) | N réguas da mesma grandeza na mesma linguagem divergem — a lei que o próprio `cfg_test_modules` já escrevia |
| uma catraca GLOBAL de HR-15 com a dívida de todas as crates | (10/09) põe outras linhas vermelhas por um gate desta — a dívida mora no gate de cada crate |
| `const fn` / `LazyLock` para as tabelas `const` de `paint_line.rs` | a tabela guarda a chave e o pintor traduz: zero estado global, zero custo de arranque |
| `format!("{n} {}", tr("…"))` para as frases com números | fixa a ORDEM das palavras no código; ⇒ `tr_with` com a frase inteira na tabela |
| subir o tecto de `paint_impasto.rs` ou uma entrada no `FN_OVERAGE_OK` para `paint_texture_section` | corte por responsabilidade |
| uma fachada `pub use` do `seg_row_owned` no `paint_impasto` | era a morada de UM dos três consumidores; os chamadores passam a nomear o dono |
| migrar o Inspector nesta jornada | 582 literais numa crate onde a `line/components` escreve secções |
| traduzir o `Panel::TITLE` agora | contrato do trait nos 26 painéis, de outras linhas — obra de rodada |
| isentar o ficheiro INTEIRO do `TITLE` | a excepção nomeia `(ficheiro, texto)`: um ficheiro inteiro isentaria o próximo texto que lá caísse |
| contar a M7 como morta pelo código de saída | `0 falharam · 0 passaram`: o mutante não compilava (§11.8); corrida crua com a âncora inteira (§12) |
