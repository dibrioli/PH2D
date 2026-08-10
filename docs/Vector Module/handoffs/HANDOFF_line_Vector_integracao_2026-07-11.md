# HANDOFF de integração — linha `line/Vector` (texto vetorial + tipografia + **Live Shapes**) — smoke aprovado 2026-07-11

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade

- Branch **`line/Vector`** · **HEAD `6153a6f4`** · **21 commits**.
- **Base do fork = merge-base com `main` = `1c7c9a22`** — que é o **tip ATUAL do `main`** ⇒ fork
  fresco, **zero drift pré-fork**, e **`git merge --ff-only line/Vector` deve dar fast-forward limpo**
  se o main não andar.
- **Auto-contida:** commits lineares, sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado — **tudo ADITIVO**

| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| **`crates/ph2d-ecs/src/vec_shape.rs`** (NOVO) + `lib.rs` (+2) | Componente **`VecShape`** (enum: `Rectangle`/`RoundRect`/`Ellipse`/`Polygon`/`Star`/`Spiral`/`Line`/`Arc`/`Text(VecTextParams)`) — os parâmetros de uma forma paramétrica VIVA. Só **primitivos** (sem deps novas em `ph2d-ecs`), projetado p/ isolamento (módulo irmão, DIRETRIZ §1.5.2.1) | módulo NOVO → sem conflito textual |
| **`crates/ph2d-ecs/src/scene/registry.rs`** (+5/−2) | `reg.register::<VecShape>("ph2d::ecs::VecShape")` em `register_ecs_components` + **`assert_eq!(reg.len(), 24 → 25)`** | ⚠️ **PONTO DE MERGE**: outra linha que registre componente conflita aqui (append list + o `len()`). Resolução: manter AMBOS os `register` e somar o `len()` |
| `crates/ph2d-editor-core/src/ids/chrome/vector.rs` (+~60) | 17 `NodeId` consts novos + 2 fábricas de id dinâmico + `MAX_TEXT_VARIATION_AXES` | aditivo no fim do arquivo de ids do vetor |
| `crates/ph2d-editor-core/tests/node_id_collisions.rs` (+~30) | 17 linhas em `CHROME_IDS` + teste `vector_dynamic_ids_dont_collide_with_chrome_or_each_other` | ⚠️ lista hand-maintained — conflito textual se outra linha adicionar ids |
| `crates/ph2d-editor-core/src/widget/dropdown.rs` (1 linha) + `widget/mod.rs` (1 linha) | `fn opaque` → **`pub fn opaque`** + re-export `opaque as resolve_opaque` na lista `pub use dropdown::{…}` | ⚠️ merge textual se outra linha editar essa lista de re-export |
| `crates/ph2d-editor-core/tests/arch_mode_has_reconcile.rs` (+4) | linha p/ `VectorTool::set_mode` | aditivo |
| `crates/ph2d-text/src/{lib.rs,system.rs}` (+10) | expõe **`inter_variable_ttf()`** (bytes crus da fonte embutida) — o texto VETORIAL precisa dos contornos, não do pipeline parley/vello | aditivo |
| `crates/ph2d-vec-{scene,edit,render}`, `ph2d-tool-vector`, `ph2d-panel-vector`, `ph2d-vector-font` | módulos da própria feature (ver §3) | módulo-owned; baixo |
| `shells/desktop/*` | 8 módulos novos + plumbing (`render_loop`, `input_dispatch`, `undo`, `app_state`, `main`) | shell da feature; conflita só com outra linha que mexa no mesmo bloco vetorial do `render_loop` |
| `shells/desktop/Cargo.toml` (+6) · `Cargo.lock` | 2 deps: `ph2d-vector-font` (path) + **`fontique = "0.6"`** | ver §5 — **NÃO é crate externa nova** |

### 3. Símbolos que podem COLIDIR (grep de mesmo-símbolo, §1.5.5)

**ECS (o mais importante):**
- **`VecShape`** / **`VecTextParams`** (`ph2d-ecs`) + a string canônica **`"ph2d::ecs::VecShape"`**.
- **`ComponentRegistry::len() == 25`** (era 24) — `crates/ph2d-ecs/src/scene/registry.rs:290`.
  Grep: `grep -rn 'reg.len()' crates/ph2d-ecs/`.

**NodeIds novos** (todos `hash_node_id`, namespaced `vector.*` → **colisão de VALOR é improvável**;
o risco é **textual** na lista `CHROME_IDS`):
`VECTOR_MODE_TEXT` · `VECTOR_TEXT_{SIZE,SIZE_NUM,WEIGHT,WEIGHT_NUM}` ·
`VECTOR_TEXT_FONT_{PREV,NEXT,IMPORT,DD}` · `VECTOR_TEXT_ALIGN_{LEFT,CENTER,RIGHT}` ·
`VECTOR_TEXT_{LINE_HEIGHT,LINE_HEIGHT_NUM,TRACKING,TRACKING_NUM}` · `VECTOR_CONVERT_TO_CURVES`.
**Fábricas de id dinâmico** (FNV runtime, espelham as do Painter): `vector_text_font_option_id(i)`
e `vector_text_axis_id(i)` + `MAX_TEXT_VARIATION_AXES = 6`. Gate de colisão dinâmica incluído.

**editor-core widget:** `dropdown::opaque` virou `pub` + re-export **`resolve_opaque`** em
`widget/mod.rs`. Grep: `grep -rn 'resolve_opaque' crates/`.

**Outros (module-owned, baixo risco):** `ph2d_tool_vector::TextAlign` (+ ~12 consts/fns de params
`TEXT_{SIZE,WEIGHT,LINE_HEIGHT,TRACKING}_*`) · `ph2d_vec_scene::ghost_handle` (export novo) ·
`ph2d_vec_edit::ShapeTool::{kind,params,bounds,px_to_world}` + campo `cur` · `ph2d_text::inter_variable_ttf`.

**Zero** token/i18n/IconId novo. **Zero** variant em enum congelado.

### 4. Contratos congelados encostados: **NENHUM**

Gates verdes no fechamento: `architecture_contract_surface` (nodes) ·
`architecture_tool_contract_surface` (`Tool`=12/`RasterEditTool`=5/`CanvasPaintTool`=1/`PanelEvent`=4) ·
`architecture_vector_contract_surface` (`ph2d-vector-doc`/`-traits` intocados) — **4/4 pass**.
Sem ADR necessário.

### 5. O que só o `ship.sh` pega (o `foundational-integrate.sh` NÃO roda)

- **Dep nova `fontique = "0.6"` — MAS não é crate externa nova.** Verificado:
  `fontique` **já estava no `Cargo.lock` pré-fork** (transitiva via `parley`/`ph2d-text`), e o diff do
  lock **não adiciona NENHUM pacote** — só arestas (`git diff <base>..HEAD -- Cargo.lock | grep '^+name ='`
  = vazio). ⇒ **machete/deny/audit/RUSTSEC praticamente sem risco novo.** `cargo machete` verde aqui.
- **Drift pré-fork: NULO** — fork == tip atual do main (`1c7c9a22`) ⇒ fmt/typos batem com o main de
  hoje ([[project_integration_prefork_lines_ship_drift]] não morde). Ainda assim rode **`ship.sh`
  completo** na árvore combinada.
- `nextest-impacted` **funciona** (a linha só ADICIONA — nenhum package renomeado/removido).
- `advisory-db` local pode envelhecer → o `ship.sh` roda audit fresco.

### 6. Ordem/dependências + o que smoke-testar

- **Se o main não moveu de `1c7c9a22`:** `git merge --ff-only line/Vector` → fast-forward limpo → `ship.sh`.
- **Se o main moveu:** rebase; esperar conflito textual em **(a)** `ph2d-ecs/scene/registry.rs`
  (register + `len()`), **(b)** `editor-core/tests/node_id_collisions.rs` (`CHROME_IDS`),
  **(c)** `editor-core/src/widget/mod.rs` (lista de re-export do dropdown), **(d)** `Cargo.lock`
  (regenera). Todos **aditivos** → Mergiraf/hand-merge mantendo os dois lados. Depois
  `scripts/foundational-integrate.sh` + `ship.sh`.
- **Smoke: APROVADO pelo Enio (2026-07-11)**, incremento a incremento (ver §2 do log abaixo).
- **NÃO smokado / gap conhecido:** os sliders de forma (**Sides/Points/Inner/Radius/Turns/Degrees**)
  ainda agem no *default de desenho*, **não** na shape VIVA selecionada (o texto já faz isso). É o
  follow-up nomeado — mesmo padrão do "alvo do painel" do texto.

**Resumo:** *Linha `Vector` pronta (HEAD `6153a6f4`, 21 commits, fork em `1c7c9a22` = tip do main).
Aditiva: texto vetorial + tipografia completa + **Live Shapes** (componente `VecShape` novo em
`ph2d-ecs`, registrado). Pontos de merge: `registry.rs` (register + `len()` 24→25), `CHROME_IDS`,
re-export do dropdown. Zero contrato congelado, **zero crate externa nova**. Smoke aprovado.
Aguardo ordem de integração.*

---

## O que a linha entrega

### A. Texto vetorial (do zero) + tipografia
- **Pipeline glyph→`VecPath`** (skrifa via `ph2d-vector-font`): cada glyph é uma forma vetorial como
  outra qualquer (render, gizmo, snap, undo, Hierarquia). Modo **Text** de 1ª classe no painel; o
  texto herda o **Style** do painel (fill/stroke/width/cap/join) em tempo real.
- **Fonte:** escolha de família (**fontique**, fontes do sistema) · **import** de `.ttf`/`.otf`
  (seletor nativo) · **dropdown com preview REAL** — cada linha da lista é desenhada **nos contornos
  da própria fonte** (glyph→`BezPath` + `Affine`), construído **lazy** (o scan+parse do sistema só
  paga na 1ª abertura do dropdown).
- **Eixos variáveis:** **Weight** (`wght`) + um **editor genérico por-fonte** — um campo por eixo que
  a fonte expõe (Optical Size/Width/Slant/GRAD…), no **range real** dela (a Inter embutida mostra `opsz`).
- **Tipografia:** alinhamento **L/C/R** · **entrelinha** · **tracking** (layout por-linha: mede,
  alinha, soma tracking, desce por `line_height × size`).

### B. **Live Shapes** (a mudança arquitetural)
Modelo Figma/Illustrator: toda forma é um **objeto paramétrico vivo** até virar curva.
- **Componente foundational `VecShape`** (ph2d-ecs, registrado no `ComponentRegistry`): os parâmetros
  são a fonte da verdade AUTORADA; a geometria é **derivada** (re-cook).
- **Texto = UM objeto**: um `VecPath` **compound** (todos os glyphs num path só) + `VecShape::Text` —
  uma entrada na Hierarquia, um pick, um gizmo. (Antes: N letras soltas, impossível de reeditar.)
- **Modelo cook-centered**: a geometria de uma forma viva **nasce centrada no local 0** (= o pivô) e o
  `Transform` guarda a pose ⇒ **pivô no centro do objeto** (rotaciona em torno do centro), re-cook
  idempotente (preserva a pose/move do usuário), e o `settle` vira no-op (pula `VecShape`).
- **Todas as shapes** (rect/ellipse/polygon/star/rrect/spiral/line/arc) **nascem vivas** ao soltar o
  gesto (o `w`/`h` vem do retângulo AUTORADO, não da bbox — num polígono ímpar elas diferem).
- **"Convert to Curves"**: o **texto** explode num **grupo de paths por-letra** (cada letra com o pivô
  no próprio centro; Ungroup separa); uma **paramétrica** só **descarta o `VecShape`** → path cru.
- **Painel edita a SELEÇÃO**: as configs de texto ficam visíveis e **editam o objeto selecionado**
  (mesmo na ferramenta **Select**), enquanto ele for texto — re-cook ao vivo, sem sair do lugar.
- **Undo/redo**: o restore **encerra a sessão de texto** (senão o upsert por-frame reescrevia o
  `VecShape` e desfazia o undo no frame seguinte).

### C. Outros
- **Snap** ao redimensionar (canto encaixa nas formas vizinhas) + guias no modo Select.
- **Alças "ghost"** em pontos Smooth: tocos agarráveis onde a alça é zero (a curva só muda ao
  arrastar) — e os glyphs de fonte marcam as **junções curva↔reta como Smooth**, então as alças
  laterais aparecem desde a criação da letra.

## Gates no fechamento

**nextest do impacted set = 1329/1329 pass** (ph2d-ecs · vec-scene · vec-edit · vec-render ·
vector-font · tool-vector · panel-vector · editor-core · host-desktop) · **contratos 4/4** ·
arch-gates verdes (`node_id_collisions` incl. o teste de colisão dinâmica novo ·
`architecture_panel_{wiring_parity,loc_cap}` · `file_loc_caps` HR-18 · `no_{magic_numeric,literal_color,tofu_glyphs}` ·
`architecture_widget_mod_in_sync`) · **clippy `--all-targets` = 0** · **fmt** (pin 1.95) · **typos 0** ·
**machete 0**.

**Nota HR-18:** `vec_glyph.rs` (738) e `vec_text.rs` (911) estouraram o teto de 600 LOC e foram
**DIVIDIDOS por responsabilidade** (não allowlist), commit `6153a6f4`: `vec_glyph` (layout) +
`vec_glyph_build` (o builder puro glyph→`VecPath`); `vec_text` (a sessão) + `vec_text_object` (o lado
objeto: `VecShape::Text`, alvo do painel, Convert). Cada um re-exporta o irmão → zero churn nos
call-sites.

## Follow-ups nomeados (NÃO feitos)

1. **Sliders de forma editarem a shape VIVA selecionada** (Sides/Points/Inner/Radius/Turns/Degrees) —
   hoje só afetam o próximo desenho. É o mesmo padrão do "alvo do painel" já feito para o texto
   (`vec_text_object::panel_text_target` + `edit_selected_text`). **É o que dá sentido pleno a "live
   shape"** — o follow-up mais valioso.
2. **Resize pelo gizmo reescrever os PARÂMETROS** (w/h) em vez de aplicar scale no `Transform`
   (bake-on-release, espelhando `settle_origins`). Hoje o resize escala a pose; o raio do RoundRect,
   por ex., escala junto (o correto seria ficar constante em px, à la Figma).
3. **Reabrir a string** de um texto finalizado (duplo-clique → sessão) — hoje as *propriedades* são
   editáveis na seleção, mas a *string* não.
4. **Virtualizar as previews do dropdown de fonte** — a 1ª abertura carrega+parseia todas as fontes do
   sistema de uma vez (hitch único; aberturas seguintes são cacheadas).
5. Persistência: `vec_save` não serializa pose/nome/parentesco (gap pré-existente, ver CLAUDE.md §5).

*"Linha `Vector` pronta (HEAD `6153a6f4`, 21 commits). Handoff acima. Aguardo ordem de integração."*
