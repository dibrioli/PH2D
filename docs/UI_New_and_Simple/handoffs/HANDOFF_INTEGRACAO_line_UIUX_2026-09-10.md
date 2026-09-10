# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-09-10

> Entregável que **fecha a linha** (`CLAUDE.md` §0.7, DIRETRIZ §1.5.9). A linha **não integra e não
> pusha**: entrega isto e para.
>
> ⚠️ **Leia o §6 antes do primeiro `git merge`.** Esta jornada acrescentou um **const obrigatório a
> um trait que 26 crates implementam** — uma linha paralela que traga um painel novo **não
> compila** até acrescentar uma linha, e o erro não diz isso com clareza.

## 1 — Identidade

| | |
|---|---|
| branch | `line/UIUX` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX` |
| HEAD | `558962538c284c263af58e8b0fa13858d5c67773` |
| merge-base com `main` | `39d48cd765653fee57d4162e2b8ca48e31f97ef4` |
| commits | **43** |
| ficheiros | **136** |
| `main` no fecho | `39d48cd76` — **igual à base** ⇒ nada aterrou no `main` desde o fork, e a linha é candidata a `--ff-only` limpo |

⚠️ **Isso vale para o `main` de 2026-09-10.** Se outra linha integrar antes desta, a coluna «base»
da tabela do §3 envelhece **em silêncio** — re-rode a sonda (DIRETRIZ §1.5.9 item 3).

## 2 — Foundational / partilhado tocado, e porquê

| onde | linhas | porquê |
|---|---:|---|
| `crates/ph2d-editor-core` | +3 073 / −323 | é a crate desta linha: abas de encaixe, faixa, transbordo, arrasto, chrome, ritmo, HR-15 |
| `crates/ph2d-panel-registry-init` | +898 / −16 | gates de população (só `tests/`) |
| `shells/desktop` | +701 / −184 | o gesto da borda (w49), 3 gates novos em `tests/` |
| `crates/ph2d-panel-painter-layers` | +335 / −121 | wave w42: os modos do Painter viram grupo segmentado; corte por responsabilidade |
| `crates/ph2d-text` | +228 / −36 | **a cache de layout roda em vez de se deitar fora** (w50) |
| `crates/ph2d-i18n` | +64 | tabela nova `chrome.rs` + **uma linha** na cadeia de despacho |
| `crates/ph2d-panel-inspector` | +67 / −14 | wave w39b: as seis secções da foto passam pela porta do ritmo |
| **os outros 22 `ph2d-panel-*`** | **1 linha cada** | o `const ICON` — ver §6.1 |
| `crates/ph2d-tool-painter` | +16 / −1 | o rótulo da aba do Painter (w42) |
| `scripts/censo-texto-pintado.py` | +165 | instrumento novo, sem consumidor automático |

⛔ **Nenhuma alteração de contrato congelado** (§4), **nenhum ADR**, **nenhum pacote externo novo**,
**nenhum schema movido**.

## 3 — Superfície de colisão (saída da sonda, colada)

```
SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 39d48cd76   ·   43 commit(s)   ·   136 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        123   (base: 123)
      └ tripla do gate               (123, 13, 22)   (base: (123, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  80   (base: 80)
    ph2d-script (espelho)                  80   (base: 80)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3)
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
```

⭐ **Zero em todas as colunas que costumam colidir.** O risco desta linha **não está nos números** —
está na **forma** (§6).

## 4 — Contratos congelados (§6)

**Nenhum encostado.** O `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` e o
`NodeOp`/`OpResolver`/`NodeManifest` estão intocados, e a sonda confirma-o.

⚠️ **O `Panel` NÃO é contrato congelado** (§6 não o lista), e é por isso que o §6.1 abaixo é
legítimo — mas ele é implementado por **26 crates**, o que o torna o ponto de fusão mais caro desta
linha.

## 5 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

| | estado nesta worktree |
|---|---|
| `cargo fmt --all -- --check` | ✅ limpo |
| `clippy --all-targets` nas crates do diff | ✅ zero avisos (`editor-core` · `i18n` · `panel-registry-init` · `host-desktop`) |
| `cargo machete` | ⚠️ **não corrido** — esta linha não acrescentou dependência nenhuma ao `Cargo.toml`, logo o risco é o de deriva pré-fork |
| `cargo deny` / `audit` (RUSTSEC) | ⚠️ **não corrido** — deriva pré-fork ([[project_integration_prefork_lines_ship_drift]]) |
| `typos` | ⚠️ **não corrido**. ⛔ Esta linha escreve muito português com acento em doc-comments e mensagens de gate; foi o `typos` que apanhou dois vermelhos na integração de 07/09 |

## 6 — ⚠️⚠️ O QUE VAI PARTIR NA FUSÃO, e é previsível

### 6.1 ⛔⛔ `Panel::ICON` é um **const OBRIGATÓRIO NOVO**, e 26 crates o declaram

```rust
// crates/ph2d-editor-core/src/panel/panel_trait.rs
const ICON: crate::icons::IconId;      // ← sem default, como o TITLE
```

⇒ **uma linha paralela que traga um painel NOVO não compila** até acrescentar uma linha ao `impl`
dela. O erro do compilador é `not all trait items implemented`, que não diz *«o dono da UI mudou o
trait»*.

⚠️ **A ausência de default é DELIBERADA e não se deve "curar" na integração.** Um default (um glifo
genérico ou `None`) compila e entrega **duas abas indistinguíveis** meses depois, num defeito que
nenhum gate apanha — está escrito no doc do próprio const, com a citação do report do dono.

**Cura na integração:** para cada painel novo de outra linha, uma linha
`const ICON: ph2d_editor_core::icons::IconId = …::IconId::<X>;` — e o `X` tem de ser um glifo que
**mais ninguém usa** (o gate `no_two_panels_share_a_glyph` mede-o).

### 6.2 ⛔ `IconId::Mixer` entra **no meio** de um enum cuja ORDEM é load-bearing

`crates/ph2d-editor-core/src/icons.rs` — a variante nova fica **entre `Minus` e `Modify`**, porque
a ordem é alfabética **por slug** e *a posição da variante É o índice nas tabelas geradas*.

⇒ se outra linha acrescentar um ícone, as duas edições caem **no mesmo bloco**. ⛔ **A resolução é
RE-ORDENAR alfabeticamente, nunca concatenar os dois lados** — concatenar compila e desloca o índice
de todos os glifos a jusante, em silêncio.
⚠️ E o ficheiro `docs/design/icons/mixer.svg` (Lucide *sliders-vertical*, ISC) vem junto; o gate
`every_declared_glyph_draws_something` reprova se ele faltar.

### 6.3 O `slot_tabs` partiu-se em **quatro** ficheiros

`slot_tabs.rs` (639) + `slot_tabs_face.rs` + `slot_tabs_overflow.rs` + `slot_tabs_drag.rs`.
⭐ **Os re-exports mantêm `slot_tabs` como o endereço único** (`pub use super::slot_tabs_drag::{…}`,
`pub use super::slot_tabs_face::{…}`), logo **nenhum chamador de fora precisa de mudar**.
⚠️ Mas quem editar `slot_tabs.rs` funde contra um ficheiro que **encolheu** — o conflito lê-se como
"código apagado" e não é: mudou de casa.

Símbolos que mudaram de ficheiro (todos re-exportados): `drop_targets` · `resolve_tab_drop` ·
`paint_drag_overlay` · `tab_pad_x` · `natural_w` · `tab_bg` · `tab_radii` · `tab_dividers`.
Símbolos cuja **assinatura** mudou: `tab_layout` (ganhou `text_system`) · `paint_slot_tabs` (ganhou
`slot: Slot`) · `tab_rects` **deixou de existir** (a largura passou a ser por conteúdo).

### 6.4 `ph2d-text`: `system.rs` 741 → 663, com `layout_cache.rs` extraído

Corte imposto pelo teto de LOC. A API pública **cresce** (`TextSystem::shapes()`,
`layout_cache_len()`, `pub use layout_cache::LAYOUT_CACHE_CAP`) e não perde nada.

### 6.5 `shells/desktop/src/dock_resize.rs`: funções **APAGADAS** por ordem do dono

`close_column` e `dock_width_choice` **não existem mais** (w49 — *«Vamos retirar a opção de colapsar
arrastando»*), e `SeamDrag` perdeu `may_close` e `width_at_start`.
⇒ quem os chamar não compila. ⛔ **Não os reponha na integração**: a remoção é uma ordem do dono
depois de o gesto prender o app por um minuto.

### 6.6 20 literais migrados na `ph2d-editor-core`, em 12 ficheiros

Cada um é **uma linha** (`"Fill"` → `ph2d_i18n::tr("chrome.fill.title")`). Conflito **textual** com
qualquer linha que toque a mesma linha; **semântico, nenhum**.
⚠️ Se um conflito for resolvido a favor do literal, o gate
`no_label_of_this_crate_is_written_in_the_painter` reprova **e nomeia o ficheiro e a porta**.

### 6.7 `ph2d-i18n`: a cadeia de despacho é **UMA linha** que várias linhas querem editar

```rust
k => vector::tr(k).or_else(|| sculpt3d::tr(k)).or_else(|| model3d::tr(k))
    .or_else(|| chrome::tr(k))                       // ← esta linha
    .unwrap_or_else(|| leak_key(k)),
```
⛔ **Resolução: ACUMULAR todas as tabelas, nunca escolher um lado.** Escolher um lado deixa a tabela
da outra linha órfã, e o sintoma é o app a pintar identificadores crus (`panel.x.y`) — com
vazamento por quadro, porque o `leak_key` faz `Box::leak`.

### 6.8 ⚠️⚠️ As catracas desta linha estão a ZERO e **vão morder as superfícies das outras**

Não é defeito: é o desenho. Mas o integrador tem de saber que é **esperado**, e que a cura é
**converter, nunca alargar a lista**:

| gate | o que reprova na árvore combinada |
|---|---|
| `every_stack_of_rows_asks_the_rhythm` (`MUTE_OK` **vazio**) | uma superfície de UI nova que empilhe linhas sem passar pela porta do ritmo |
| `no_two_panels_share_a_glyph` | um painel novo que escolha um ícone já usado |
| `every_declared_glyph_draws_something` | um `IconId` declarado sem SVG |
| `the_binary_carries_every_panel_crate_that_exists` | uma crate `ph2d-panel-*` nova que o shell não ligue (a população é **contada no disco**) |
| `no_label_of_this_crate_is_written_in_the_painter` | um literal novo pintado dentro da `ph2d-editor-core` |
| `every_chrome_key_exists_on_both_sides` | uma chave `chrome.*` com erro de escrita, ou órfã |
| `the_look_is_a_widget_skin_never_an_area_model` | um ficheiro de `screens/` que passe a consultar `ui_look()` |

⚠️ **O `every_stack_of_rows_asks_the_rhythm` é o mais provável de reprovar**, e a lista de dívida
dele está **vazia de propósito** — o `CLAUDE.md` §5 já registava *«as superfícies de UI que as
outras cinco linhas de 07/09 trouxeram foram escritas contra a lei de espaçamento ANTIGA»*.
⛔ A cura de um vermelho ali é **chamar a porta** (`Spacing::*`, `ROW_H_PX`, `section_gap_px()`),
nunca acrescentar uma entrada a `MUTE_OK` — *uma catraca sem censo de obsolescência não desce: ela
vira licença* (§5.0).

## 7 — Ordem, dependências e o que smokar

**Ordem:** nenhuma dependência entre commits além da cronológica. `--ff-only` numa vez.

### O que o dono JÁ smokou e aprovou
w34–w49 — as abas (largura por conteúdo · ordem · arrasto entre encaixes · piso quadrado ·
encolher · divisória · ícone · aba sozinha · transbordo), a face vazia, e a retirada do
colapsar-por-arrasto.

### ⏳ O que NÃO foi smokado
| wave | o que ver |
|---|---|
| **w50** — a cache de texto | abrir **todos** os painéis (menu *Window*) e mexer no ecrã: era `182 ms`/quadro e é `7,8`. Sem número à vista, o sintoma é o app deixar de "engasgar" com tudo aberto |
| **os 20 rótulos migrados** | *New Image* · *New Sprite Sheet* · *Size* · *Background* · *Resolution* · *No matches* · *Cancel* · *Done* · *Fill* · *IMG* · *Hex* · *Read-only* · *Shape options* · *Mask options* · *Probe A/B* — todos têm de aparecer **em inglês, como antes**. ⛔ **Se algum aparecer como `chrome.alguma.coisa`, é uma chave que não chegou à tabela** |

**Comando (copiável de uma vez):**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --release
```
⚠️ **Preferência fora do repo:** `~/.ph2d/layout.txt` guarda a arrumação (XOR contra o
`DEFAULT_VISIBLE`) — um ficheiro velho abre o app com um painel fechado, e **apagá-lo é o reset**.
`~/.ph2d/prefs.txt` com `reduced_motion=1` reprova smokes sobre produto correto.

## 8 — ⏳ ABERTO (não corrigir na integração)

| item | de quem depende |
|---|---|
| os **`418`** literais pintados nas outras **18** crates (`painter-layers` sozinha tem `166`) | da linha dona de cada crate — [`medicoes/07`](../medicoes/07_o_buraco_do_hr15_o_texto_PINTADO.md) tem o instrumento e a ordem |
| **esvaziar os painéis** (degrau `G`, a foto 3 do dono) | ⛔ **bloqueado pelo item acima**: `19` de `26` painéis não declaram entrada nenhuma, logo não há lista para classificar ([`medicoes/07 §3-bis`](../medicoes/07_o_buraco_do_hr15_o_texto_PINTADO.md)) |
| **pose 2D ou 3D** | do **dono** — ✅ o número existe agora: `722` sítios de produto (`1 799` com testes), **5** crates, `71 %` no `shells/desktop` ([`medicoes/04 §6`](../medicoes/04_o_alcance_das_timelines.md)) |
| **partir o `DrawMode`** nos dois eixos | do **dono** — e a contagem PIOROU: `17` variantes vivas (eram `14`) |
| os **9 toggles de módulo** → Layout | do **dono** |
| o **«travou por um minuto»** de 2026-09-09 | ⛔ **sem reprodução**. O gatilho (colapsar por arrasto) saiu na w49. O penhasco de `182 ms` foi medido e **NÃO é ele** — três ordens de grandeza |
| o **cartão** do `ph2d-panel-motion-graph` | superfície de linha viva; a régua do ritmo não o alcança |

## 9 — Estado do portão, no fecho

```
Summary [1313.817s] 22208 tests run: 22208 passed (49 slow), 2166 skipped
```

- **22 208 / 22 208** verdes, `--no-fail-fast`, workspace inteira.
- ⚠️ **Excluído:** `binary(the_census_of_every_primitive)` (`ph2d-field-eval`) — **2 609 s** sozinho,
  corrido em separado na jornada anterior desta linha: **28/28 verdes**
  ([[reference_one_field_eval_test_costs_55_minutes_of_the_debug_suite]]).
- `cargo fmt --all -- --check` ✅ · `clippy --all-targets` ✅ nas crates do diff.

### ⚠️ Os TRÊS flakes de carga que esta linha mediu, e que pedem PROMOÇÃO ao §5.0

A linha pede, **o integrador escreve** (precedente: `the_cost_of_a_player_is_linear_in_their_number`,
promovido em 2026-09-07 «a pedido da `line/UIUX`»). Corpo em
[[reference_flip_fit_cache_ratio_is_a_load_flake]].

| teste | espécie | confirmação |
|---|---|---|
| `the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke` | razão de dois relógios | 3/3 a `load 4,69` |
| `interaction_dispatch_no_alloc` | contador de alocações | 3/3 a `load 2,20`; o ficheiro **não menciona texto** e o diff acusado é da `ph2d-text` |
| `the_ui_clock_does_not_allocate_per_frame` | contador de alocações | **3/3 a `load 18,20`/`18,99`/`18,99`** ⇒ o discriminador é o **fan-out**, não o relógio |

⛔ O doc-comment do terceiro **declarava-se imune** (*«uma contagem de blocos é determinística e não
flaka»*) — é o **quarto** deste repo a fazê-lo. Corrigido no ficheiro, com a medição dentro.

## 10 — ⛔ Para o `CLAUDE.md §5` — UMA linha de estado, e **duas correcções**

### 10.1 A linha `Aberto:` do módulo UI/UX passa a ser

> **Aberto:** ⛔ os **418** literais de UI pintados nas outras 18 crates — e eles **bloqueiam** o
> degrau `G` (esvaziar os painéis), porque `19` de `26` painéis não declaram entrada nenhuma para
> classificar ([`medicoes/07`](docs/UI_New_and_Simple/medicoes/07_o_buraco_do_hr15_o_texto_PINTADO.md);
> instrumento: `python3 scripts/censo-texto-pintado.py`) · **três decisões do dono**, todas com o
> número ao lado: a pose 2D/3D (`722` sítios, 5 crates), partir o `DrawMode` (`17` variantes vivas)
> e os 9 toggles de módulo → Layout · o **«travou por um minuto»** de 09/09 segue **sem reprodução**
> (o gatilho saiu na w49; o penhasco de 182 ms foi medido e **não** é ele).

### 10.2 ⛔⛔ CORRIGIR a linha de **Smokes** do módulo — ela afirma uma falsidade

Ela diz hoje:

> `PH2D_UI_NEW=0` (o clássico tem de ficar byte a byte)

**É falso, e já era falso no dia em que foi escrito.** Medido (com `git log`): o sistema de abas de
encaixe nasceu em **2026-08-30** (`c7c5c653a`) sem consultar a aparência em sítio nenhum, e a
cláusula entrou no roteador em **2026-09-07** (`5fe3c51cc`) — **oito dias depois**. Substituir por:

> ⚠️ `PH2D_UI_NEW=0` **não é o ecrã de antes do redesenho** — devolve **seis pintores de widget**, a
> família de temas da barra do topo e o tema de arranque. A estrutura (colunas, encaixes, faixa de
> abas, transbordo) é a **mesma** nas duas desde 2026-08-30, e há gate a mantê-la assim
> (`the_look_is_a_widget_skin_never_an_area_model`).

Mecanismo: [`medicoes/10`](docs/UI_New_and_Simple/medicoes/10_o_classico_nao_e_um_ecra_anterior.md).

### 10.3 E a lista de flakes do §5.0 recebe os **três** do §9

## 11 — As leis que esta jornada pagou (para a próxima LLM)

1. ⭐⭐⭐ **Um censo textual tem de saber todas as formas do que lê — e «todas» inclui as do próprio
   leitor.** O censo de literais pintados errou **duas** vezes no mesmo dia: primeiro por conhecer
   **1** porta de **127** (`108` publicados contra `418`), depois por o leitor não conhecer o
   **literal de carácter** — `find('"')` é Rust legítimo, e ele lia aquela aspa como uma string a
   abrir, virando o resto do ficheiro do avesso (`438` contra `418`). ⭐ Quem apanhou o segundo erro
   foi **a implementação irmã**: *ter duas réguas da mesma grandeza, em linguagens diferentes, não é
   redundância — é a única forma de uma acusar a outra*, e o acordo delas em `32` é o que torna o
   número acreditável.
2. ⭐⭐⭐ **Uma promessa escrita DEPOIS do facto que a desmente não é uma regressão de quem veio a
   seguir.** A cláusula «o clássico tem de ficar byte a byte» entrou no roteador oito dias depois de
   as abas já correrem nas duas aparências. *Antes de tratar um vermelho como regressão, date a
   promessa contra o código.*
3. ⭐⭐ **Um número que o corte de dependência trunca lê-se como pequeno.** A 1.ª medição do alcance
   do `Transform` renomeou o campo e leu `57` — todos numa crate só, porque um **erro** pára a crate
   e esconde tudo o que depende dela. Com `#[deprecated]` (um **aviso** não pára nada) a árvore
   inteira responde numa passagem: `1 799`. *Escolha o instrumento pelo que ele deixa correr, não
   pelo que ele deteta.*
4. ⭐⭐ **O `grep` sabe como um campo se escreve; o compilador sabe de que tipo ele é.** A varredura
   textual acusava **19** crates a ler `Transform`; são **cinco**. Os nomes `translation`/`rotation`
   /`scale` são partilhados por outros tipos.
5. ⭐⭐ **Dois itens abertos podem ser um só, e o segundo pode ser um INSTRUMENTO que falta.** O
   *«nenhum outro painel foi censado»* não era desleixo: `19` de `26` painéis não declaram entrada
   nenhuma, logo o censo teria de ler pintor a pintor, sem catraca e sem saber quando acabou. *Um
   censo cuja população se conta lendo prosa não é um censo — é uma opinião com tabela.*
6. ⭐⭐ **Uma catraca partilhada e uma catraca replicada medem a mesma coisa; só a segunda respeita o
   isolamento.** O plano pedia um gate global de HR-15 com baseline por crate — e isso poria a
   próxima linha vermelha por causa de um gate desta. O que shipou é **por crate, a zero**, feito
   para ser **copiado**.
7. ⭐⭐ **Trocar o eixo em que uma régua é frágil não é o mesmo que a tornar robusta.** O
   `the_ui_clock_does_not_allocate_per_frame` trocou o relógio por um contador e escreveu no doc
   *«não flaka»* — cura a deriva da máquina, **não** cura o fan-out. Ele passa `3/3` a `load 19` e
   reprova sob `1 600` testes em paralelo.
8. ⭐ **Uma palavra escrita em dois sítios ainda não é uma palavra do app — só uma PORTA é.** A
   migração de rótulos achou *No matches*, *Shape options*, *Mask options* e *Hex* cada um escrito
   em **dois** pintores: mudar um deixava o outro a dizer a antiga, e nenhum gate o via.
9. ⭐ **Uma lista aberta acumula trabalho já pago — e pode mandar CONSTRUIR o que o dono mandou
   RETIRAR.** A auditoria da spec do módulo achou quatro linhas já feitas e uma invertida; a pior
   delas pedia *«um gesto de recolher»*, que existiu, prendeu o app por um minuto e saiu na w49 por
   ordem dele.
10. ⭐ **Uma migração mecânica entra em prosa.** O `replace` dos rótulos escreveu `ph2d_i18n::tr(…)`
    dentro de **três comentários** — compila na mesma. *Toda substituição em massa acaba com uma
    verificação que separa código de prosa.*

## 12 — Higiene de fecho (DIRETRIZ §1.5.9 itens 7 e 9)

- ✅ `incremental/` reclamado: **30 GB** libertados (`target/debug/incremental`).
- ✅ Binário do smoke **compilado em `--release`**, com a prova da 2ª corrida colada abaixo.

```
$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 0.39s
$ cargo build -p ph2d-host-desktop --release      # ← a PROVA
    Finished `release` profile [optimized] target(s) in 0.22s
```
**ZERO linhas `Compiling`** — o dono não paga build nenhum ao correr o smoke do §7.
Binário: `target/release/ph2d-host-desktop`, 67,6 MB.

⚠️ **Nenhum smoke desta linha exige `--features`** ⇒ há **um** binário, e é este.

---

## ⛔ Recusas MEDIDAS desta jornada

| o que | por que não |
|---|---|
| gatear a faixa/o ícone/o transbordo das abas pela aparência | seriam **dois modelos de áreas** vivos para sempre, por uma bissecção que o interruptor nunca ofereceu ([`medicoes/10 §3`](../medicoes/10_o_classico_nao_e_um_ecra_anterior.md)) |
| um gate que arme as duas aparências no MESMO processo | **vácuo**: o `paint_hero_screen` chama `set_ui_look(ui_look_from_env())` a cada quadro e o `ui_look_from_env` é um `OnceLock` ⇒ o produto reescreve o que a sonda armou |
| um dar `Default` ao `Panel::ICON` | compila e entrega duas abas indistinguíveis meses depois; o defeito só aparece no smoke |
| um gate GLOBAL de HR-15 com baseline por crate | põe a próxima linha vermelha por causa de um gate desta (§0.2) — o que shipou é por crate, a zero |
| subir o `LAYOUT_CACHE_CAP` | só muda **onde** fica o penhasco; a cura é rotação entre duas gerações |
| um portão de TEMPO sobre o quadro | moldagens são determinísticas; um relógio entraria na família das flakes de carga (§5.0) |
| migrar os `418` literais nesta janela | 19 crates, 11 de outras linhas — catástrofe de merge por diff mecânico |
| censar o degrau `G` antes de os literais migrarem | **impossível com instrumento**: 19 de 26 painéis não declaram entrada nenhuma |
| repor `close_column` / o colapsar-por-arrasto | ordem explícita do dono (09/09), depois de o gesto prender o app por um minuto |
