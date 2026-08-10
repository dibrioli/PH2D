# Handoff de integração — linha `line/Painter` (Watercolor, jornada 2026-07-10/11)

> **DIRETRIZ §1.5.9.** A linha está FECHADA. NÃO integrei nem fiz ship. Este doc passa ao **agente
> integrador** (quando o Enio ordenar) os pontos que evitam conflito/regressão. Aguardo ordem de integração.

## 1. Identidade

- **Branch:** `line/Painter`
- **HEAD:** `95f9370e`
- **Base (merge-base com `main`):** `7d4da8ec` (2026-07-10, `chore(typos): allowlist pt-BR 'retangular'`)
- **Commits desde o fork:** **19**
- **Escopo:** módulo **Watercolor** dentro de `ph2d-tool-painter` (render-path óptico) + sua UI de painel
  + 1 overlay de shell + ids de chrome + 1 campo de `BrushSpec`. **Fork é PÓS-cutover do Vector** (nenhuma
  crate deletada envolvida → sem drift de cutover, [[project_integration_prefork_lines_ship_drift]]).

## 2. Foundational / compartilhado tocado (tudo ADITIVO, painter-scoped)

| Arquivo | Natureza | Por quê |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/painter_watercolor.rs` | aditivo | 5 `NodeId` de chrome novos (Opacity / Wet-Preview / Dry-Time / Dry-Now / Wet-Now) + arrays `PAINTER_WATERCOLOR_FIELDS:[;27]` e `PAINTER_WATERCOLOR_CLICKS:[;6]` estendidos. Arquivo painter-específico. |
| `crates/ph2d-painter-brush/src/spec.rs` | aditivo | **1 campo novo** em `BrushSpec`: `opacity: f32` (default `0.0` = byte-idêntico; o render-path usa `0.4` só com Watercolor ON). |
| `crates/ph2d-panel-painter-layers/src/{paint_watercolor,populate,brush_fallback}.rs` | aditivo | UI dos cards **Wash** (linha Opacity) e **Wetness** (Preview slider, botões Dry/Wet, Drying-Time slider) + register no `populate`. |
| `shells/desktop/src/render_loop/painter_bridge_overlays.rs` | aditivo | overlay on-canvas de umidade (`draw_wetness_overlay` + helper `box_blur_f32`). |

Nenhuma lógica compartilhada foi reescrita — só extensões append-only da superfície do painter.

## 3. Símbolos que podem COLIDIR com outra linha (grep de mesmo-símbolo, §1.5.5)

- **`NodeId` novos** (editor-core `painter_watercolor.rs`), todos content-addressed via `hash_node_id("painter_brush.watercolor_*")` — colisão numérica só se outra linha hashear a MESMA string (namespace `painter_brush.*`, improvável):
  `PAINTER_WATERCOLOR_OPACITY`, `…_DRY_TIME`, `…_WET_PREVIEW`, `…_DRY_NOW`, `…_WET_NOW`.
- **Arrays ordenados** `PAINTER_WATERCOLOR_FIELDS` (len **27**) e `PAINTER_WATERCOLOR_CLICKS` (len **6**) — se outra linha editar os MESMOS arrays → conflito textual (Mergiraf resolve; o integrador confere os counts 27/6).
- **`BrushSpec.opacity: f32`** (`ph2d-painter-brush/src/spec.rs`) — se outra linha também adicionar campo a `BrushSpec`, conflito textual adjacente em spec.rs (append-only). **Sem id numérico.** `BrushSpec` NÃO é serializado por schema versionado neste diff (nenhum `SCHEMA_VERSION` foi bumpado; os testes da crate — incl. round-trips — passam).
- **Sem** `IconId` novo, **sem** `NodeId(numérico)`, **sem** chave de token, **sem** entrada em registry-init.

## 4. Contratos congelados encostados (§4/§6)

**NENHUM.** Sem toque em `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` (tool contract), nem
`NodeOp`/`OpResolver`/`NodeManifest` (node contract), nem `ph2d-vector-doc`/`-traits` (vector contract).
`BrushSpec` não é contrato congelado (os ABIs de pintura foram revogados por ADR-0099; `ph2d-painter-brush`
é não-gateada). **Nenhum ADR exigido.**

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

Rodei nesta jornada: `cargo test -p ph2d-tool-painter --lib` (**519 verdes**), `cargo clippy -p ph2d-tool-painter --lib --all-features` (0), e o gate **`architecture_workspace_file_loc_cap`** (2/2 verde — vide §6-split). **NÃO rodei** (ship deve rodar):

- **fmt** — não rodei `cargo fmt` nas 4 crates tocadas; o código novo é escrito à mão no estilo rustfmt mas **não verificado contra o rustfmt pinado** → possível fmt-skew. Use `rustup run <pin> cargo fmt` ([[feedback_ci_direct_lint_gates_and_fmt_skew]]).
- **typos** — comentários novos têm pt-BR (`byte-idêntico`, `costura`, `célula`, `período`…) + doc-comments em inglês. O fork base (`7d4da8ec`) foi justamente um allowlist de typos; **palavras pt-BR minhas podem precisar de allowlist**. (typos escaneia comentários.)
- **clippy `--all-targets` + features (workspace)** — só rodei `--lib` do `ph2d-tool-painter`. As mudanças de **painel/shell/editor-core** não passaram por clippy `--all-targets`.
- **machete/deny/audit** — **nenhuma dep de crate nova** (só um campo em struct) → machete deve ficar limpo e sem RUSTSEC novo; mas a advisory-db local envelhece.
- **testes das outras crates** — `ph2d-editor-core` (só rodei o LOC gate), `ph2d-panel-painter-layers`, `shells/desktop` não tiveram suíte completa RE-rodada nesta jornada (foram testadas quando cada mudança landou nas sessões anteriores).

## 6. Ordem / dependências + o que smoke-testar

**Ordem:** commits são fixes/features independentes em sequência cronológica; a única dependência é o
**split de LOC (`95f9370e`) vem DEPOIS** do commit de textura #2 (`f8257ddc`) — já ordenado. O split é
**refactor puro byte-idêntico** (novo `watercolor_noise.rs` + helpers `warp_offset`/`fill_substrate_cache`/
`session_maxima`), feito para reancorar `watercolor_field.rs` (737→~560) e `watercolor_render.rs` (730→699)
sob o cap 700 que as mudanças de #2 estouraram.

**Já smokado pelo Enio (OK):** #17 Opacity · #3 botão Wet · #9/#10/#11 Dry/Wet/Drying-Time · #12 umidade
ao-vivo + erosão · #13 substrato por-dono · #18 junção suave · undo limpa umidade · #2 round-1 (wash embrulha).

**Ainda NÃO smokado (smoke antes/depois de integrar):**
- **#8 Alpha-lock (aquarela):** pinte uma forma, ligue Alpha-lock da camada, pinte um wash cruzando a borda
  → deve entrar só onde já há tinta; transparência intacta; umidade só dentro da forma.
- **#2 round-2 (texturas seamless):** Tiling X + **RaggedEdge alto**, cruze a costura → a **borda rasgada**
  deve continuar perfeitamente do outro lado (era o bug do último smoke).

**Follow-ups menores anotados (não-bloqueantes, doc 13 #2):** (a) `smear_wet_base` (Smudge>0) no chain cru
sem wrap de tiling; (b) texturas de **SLOT** (Paper/Grain via `sample_tiled_rot`) ainda não são seamless com
a sprite (só as procedurais são) — exige decisão de design. Default não usa slot.

---
*Linha `line/Painter` pronta (HEAD `95f9370e`, 19 commits). Aguardo ordem de integração do Enio.*
