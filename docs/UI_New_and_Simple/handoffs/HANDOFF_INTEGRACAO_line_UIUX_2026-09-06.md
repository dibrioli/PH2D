# HANDOFF DE INTEGRAÇÃO — `line/UIUX` — 2026-09-06

> **O redesenho plano, de ponta a ponta.** Doze waves: a família moderna derivada, a porta da
> moldura, a porta do vão, o **cartão** que substituiu o risco azul e a **lei do grupo** do Blender.
> ⚠️ Este documento é para o **agente integrador**. O mecanismo de cada wave vive em
> [`pesquisa/08 §7`](../pesquisa/08_modelos_com_codigo_para_seguir.md); aqui está só o que evita
> conflito e regressão.

---

## 1 — Identidade

| | |
|---|---|
| branch | `line/UIUX` |
| HEAD | `e65a9ffd9` |
| merge-base com `main` | `53832c884` |
| commits | **20** |
| ficheiros tocados | **262** |

⚠️ **A tabela do §3 é REFERÊNCIA, não EVIDÊNCIA** — ela mede contra o `main` de **2026-09-06**.
Re-rode `collision-surface.sh` nesta worktree imediatamente antes de fundir; a divergência entre as
duas leituras é ela própria um achado.

---

## 2 — Foundational / partilhado tocado, e porquê

| onde | o que mudou | aditivo? |
|---|---|---|
| `docs/design/tokens.json` | **`chrome.row-h` `28 → 22`** (única mudança de VALOR) | ⛔ **não** |
| `crates/ph2d-tokens` | `derive.rs` e `visuals.rs` **novos** (a família moderna e a tabela de estados); `spacing.rs` ganha `row_gap_px`/`row_pitch_px`; `lib.rs` re-exporta | quase todo aditivo |
| `crates/ph2d-editor-core` | 91 ficheiros. **Novos:** `paint_rounded.rs`, `published.rs`, `widget/section_cards/`, `widget/button_surface/group.rs`, `screens/hero/theme_menu.rs`. O resto são pintores a passar pelas portas novas | aditivo + conversões |
| `shells/desktop` | `theme.rs`, `main.rs`, `project_tokens*.rs`, `render_loop/tokens_bridge_dtcg.rs` + 2 gates | pequeno |
| `tools/ph2d-widget-sync` | `PUB_MODULE_OVERRIDE` ganha `section_cards` | aditivo |
| `crates/ph2d-tokens-dtcg` | `export.rs` acompanha os tokens novos | aditivo |
| 24 crates de painel | conversões para as portas (moldura · vão · cartão · grupo · raio) | conversões |

⚠️⚠️ **O `chrome.row-h` é o único número partilhado que MUDA DE VALOR, e ele move o app inteiro.**
Se outra linha tiver escrito um retrato de geometria (altura de dock, contagem de linhas que
transborda, posição de um popover), **ele vai estar errado depois da fusão** — ver §6.

---

## 3 — Superfície de colisão (saída da sonda, colada)

```
SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 53832c884   ·   19 commit(s)   ·   260 arquivo(s)
  ⚠️ corrida ANTES do commit `e65a9ffd9` (as duas fixturas da `panel-authored`);
     hoje sao 20 commits e 262 ficheiros — nenhuma LINHA da tabela muda com eles.
▸ SCHEMAS
    PROJECT_SCHEMA            114   (base: 114)
      └ tripla do gate   (114, 13, 18)   (base: (114, 13, 18))
    VEC_SCENE_SCHEMA           18   (base: 18)
    FLIP_SCHEMA                13   (base: 13)
    DOC_VERSION (timeline)     18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)      80   (base: 80)
    ph2d-script (espelho)      80   (base: 80)
▸ CONTRATO CONGELADO (§6)      intocado (node.rs · tool.rs)
▸ ADR                          esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock                   nenhum '+name' novo
▸ MARCADORES DE CONFLITO       nenhum
▸ TETOS DE LOC                 nenhum arquivo da linha passa do teto
```

⭐ **Zero schema mexido, zero ADR, zero dependência nova, zero contrato congelado encostado.**
A superfície de colisão desta linha é **numérica de UI**, não de dados.

### Símbolos NOVOS que outra linha pode ter criado com o mesmo nome

`ph2d_tokens`: `visuals::{Feel, Chrome, Widgets, Frame, radius, frame, MODERN_CORNER_RADIUS_PX,
MODERN_SELECTED_W}` · `derive::Roles` · `spacing::{row_gap_px, row_pitch_px}` ·
`Theme::{Dark, Gray, Light, Oled}` + `Theme::MODERN`/`CLASSIC`.

`ph2d_editor_core::paint`: `stroke_frame`, `fill_ring`, `frame_radius`, `fill_rounded_rect_radii`.

`ph2d_editor_core::widget`: `section_cards::{begin_section_cards, end_section_cards,
with_section_cards, close_section, close_subsection, skip_section_header, CardDepth}` ·
`{GroupPos, GroupCell, SEGMENT_HAIRLINE, segment_rects, block_cells, grid_cells, grid_height}`.

⚠️ **`ph2d_editor_core::published`** é um ficheiro NOVO que recebeu os quatro `thread_local` de
aparência que viviam no `paint.rs` (tecto de LOC). **Os caminhos `paint::…` continuam a valer por
re-export** — uma linha que chame `paint::set_ui_look` funde limpo.

---

## 4 — Contratos congelados (§6)

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`
intocados (a sonda confirma). ⇒ **nenhum ADR exigido por esta linha.**

---

## 5 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- **`typos`** — os docs desta linha estão em português com muito acento e vocabulário próprio
  («cartão», «vão», «moldura», «catraca»); nunca passaram pelo `typos`.
- **`cargo machete`** — nenhuma dependência nova foi acrescentada a nenhum `Cargo.toml`, então o
  risco é zero; mas a linha **apagou** consumidores (o `paint_section_separator` perdeu 23
  chamadores), o que pode deixar um `use` órfão numa crate que eu não corri.
- **`cargo deny` / `RUSTSEC`** — sem `Cargo.lock` mexido, sem exposição nova.
- **`fmt`** — corrido em toda a árvore (`cargo fmt --all`) no fecho.
- ⚠️ **Clippy latente:** corri `--all-targets -D warnings` nas **29 crates do diff**. Uma crate
  FORA do diff que dependa da API nova não foi coberta.

---

## 6 — ⚠️⚠️ O QUE VAI PARTIR NA FUSÃO, e é previsível

**A altura de linha desceu de 28 para 22 px.** Isso não é cosmético para os gates: **todo número de
geometria escrito à mão numa fixtura de outra linha muda de valor.** Nesta linha, ele esvaziou
**cinco** fixturas, e cada uma foi curada por DERIVAÇÃO, nunca por reescrever o número:

| gate | o que continha | cura |
|---|---|---|
| `motion_bridge_dock_height` (`NAMED_OVERFLOW`) | `bezier_warp` 969 → 873 → 825 → **777**; `spline_wrap` saiu | retrato movido, 3× |
| idem, fixtura de rolagem | nomeava `source.shape` à mão | **derivada** de `height_census().first()` |
| `ph2d-panel-timeline::scroll_tests` (2) | `4` linhas / `300 px` transbordavam | **procuram** o limiar |
| `ph2d-panel-authored::seam_authored_long_list` | `OPTIONS = 30` | **contado** da janela ÷ `ROW_H_PX` |
| `ph2d-panel-authored::seam_authored_popover` | janela de `420 px` | **procura** o joelho |

⭐ **Os cinco falharam ALTO, dizendo que perderam o fenómeno** — nenhum passou por vácuo. Espere o
mesmo comportamento nas fixturas das outras linhas: *se um gate de outra linha reprovar com uma
mensagem sobre geometria, a causa mais provável é esta linha, e a cura é derivar o número — não
reescrevê-lo.*

⚠️ **E o que esta linha aprendeu sobre censos que lêem o FONTE aplica-se à fusão:** três censos
ficaram **vácuos** durante estas waves — um lia a própria assinatura como se fosse uma chamada,
outro isentava uma porta pelo **nome do ficheiro** (que mudou ao cortar por LOC), o terceiro
testemunhava a presença de uma função lendo um **comentário**. Todos foram apanhados por prova de
mutação, nunca por leitura. *Se um censo desta linha ficar verde depois da fusão, quebre-o de
propósito antes de acreditar nele.*

---

## 7 — Ordem, dependências e o que smokar

**Ordem:** os 20 commits são sequenciais e cada um fecha uma wave; **não há dependência entre
ficheiros que exija reordenar**. Fundir o ramo inteiro é o caminho.

**Já smokado e APROVADO pelo dono:** a família moderna e o tema de arranque · a porta da moldura ·
a porta do Assets · o contraste cartão/painel · a linha compacta · a folga · o **cartão** no editor
de áudio · o **grupo** nas duas direcções.

**NÃO smokado (nasceu depois do último veredito dele):**
1. os **cartões** no **Inspector**, **Painter Layers**, **Vector** e na **galeria de widgets**
   (wave 12) — só o editor de áudio foi visto;
2. o tema **CLÁSSICO** (`PH2D_UI_NEW=0`) depois da wave 12. A lei diz que ele continua a desenhar o
   risco e há gate a prová-lo, mas **ninguém o viu com os olhos** desde que o cartão existe.

**Como smokar:**
```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```
e, para o clássico:
```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_UI_NEW=0 cargo run -p ph2d-host-desktop --release
```

---

## 8 — ⏳ ABERTO (não corrigir na integração; é decisão do dono)

1. ⭐ **O cartão não aparece no Painter** (report do dono, 2026-09-06, com foto: *«em Audio Editor:
   Effects temos o card. Já o card de Painter: Jitter não se vê mais»*). **Medido:** o corpo do
   pincel **chama a porta** — são **13** `close_section` no caminho (`paint_brush_sections.rs` ×9,
   `paint_deform.rs` ×2, `paint_brush.rs`, `paint_selection.rs`) — e o par `begin`/`end` está em
   `paint.rs::paint_brush_view`, dentro do `push_clip`. ⛔ **A causa NÃO está medida**, e são duas
   candidatas: (a) o corpo pinta um fundo próprio por cima do cartão, ou (b) o `begin` está numa
   rota que aquele ecrã não percorre. *O instrumento é a mesma régua dos gates do cartão — contar
   caminhos da cena antes e depois do `end`.*
2. **A `chrome.hier-row-h` continua em 32 px** contra as 22 de formulário — a linha mais alta do
   app, e agora destoa mais. Baixá-la aperta ícone + nome + olho + cadeado na mesma linha: é
   medição de uma wave, não um número a mudar.
3. **`chrome.section-gap` (14 px) não tem UM consumidor da pergunta que nomeia** — os quatro usos
   reais tratam-no como TAMANHO DE ÍCONE. Mexer no valor encolhe quatro ícones. Cura própria.
4. **O fim de um GRUPO ainda tem duas respostas** (`Md` = 8 em 4 sítios, `Lg` = 12 em 1); a lei do
   Godot é `base·2` = **8**.
5. **50 raios ainda passam ao lado da porta**, e é DELIBERADO: metade é canvas (régua da timeline,
   células do Flip, gizmo 3D, tiras de clip), onde o raio é desenho do documento. A partição
   cromo/canvas é uma wave própria.
6. **O agrupamento de botões só chegou ao editor de áudio** — os outros painéis ainda espaçam.
7. **Duas superfícies de LISTA não seguem a lei do Godot** (linhas de lista encostam, `v_sep = 0`):
   a hierarquia avança `HIER_ROW_H + 2` e a lista de variações do áudio `22 + 4`.

---

## 9 — Estado do portão, no fecho

| | |
|---|---|
| `nextest-impacted.sh --no-fail-fast` | **12 807 testes / 0 falhados** (1 280 saltados) |
| clippy `--all-targets -D warnings` | limpo nas **29 crates do diff** |
| `cargo fmt --all` | corrido |
| binário de smoke | **compilado** (`--release`, 2ª corrida em 0,20 s) |
| `target/*/incremental` | reclamado |

⚠️ **O `nextest` cancela na 1.ª falha e esconde o resto** — a corrida que vale é a `--no-fail-fast`.
A primeira corrida deste fecho parou em 7 479 de 12 807 com **5 328 por correr**.

---

## 10 — Uma linha para o `CLAUDE.md §5`

> **UI/UX — redesenho plano (Godot 4.6 «Modern» + modelo de painel do Blender):** quatro temas
> derivados, a moldura e o vão de uma linha por PORTA, o **cartão** no lugar do risco azul (zero
> riscos no produto) e a **lei do grupo** — botões vizinhos encostam e só as quinas de fora do
> bloco arredondam. `PH2D_UI_NEW=0` volta ao clássico.
> **Aberto:** o cartão não aparece no Painter · a linha da hierarquia (32 px) destoa das 22 ·
> o agrupamento de botões só chegou ao editor de áudio ·
> [handoff](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-06.md).
