# HANDOFF DE INTEGRAÇÃO — `line/Painter` · a UI completa do Wet Paint (doc 22)

> DIRETRIZ §1.5.9. A linha está FECHADA e **NÃO integra nem faz ship** (o Enio cancelou
> explicitamente a integração/ship/CI desta jornada em 2026-07-22 — outros agentes vão
> trabalhar em suas linhas; este handoff espera o integrador da próxima janela).

## 1. Identidade

- **Branch:** `line/Painter` · **base do fork:** `13a04c7aa` (o main integrado de 2026-07-22).
- **Commits:** 19 (W1 engine → W2 tool → W3 seção básica → W4 painel lateral → fechamento →
  **fix pós-smoke: painel arrastável/redimensionável + heading engole o clique** →
  **doc 23: estudo + implementação — o pigmento responde às tools** →
  **fix pós-smoke do Impasto, em 4 commits**: TODOS os cards com Enable ON (a estreiteza
  selected-tool-only revertida; Material fan-out pros 3 slots de relevo) + os 3 refinos do
  Enio (card Knife só com a faca · card Sculpt logo abaixo do TOOL e só com um verbo em mãos ·
  Filter Layer/Stroke só nos verbos que os têm) →
  **o dropdown de MODO DE PINTURA + o bug do modo órfão** (§2.1) →
  **o canvas do Painter que era derrubado e nunca re-pushado** (§2.2), ver §2/§6;
  checkpoint de reversão do doc 23: tag `checkpoint-pre-wet-tools-rework`).
- **Plano:** [`docs/Painter/22_plano_wet_tuning_ui.md`](Painter/22_plano_wet_tuning_ui.md).
- **Gate batched:** `nextest-impacted` **5053/5053** · clippy `--all-targets` **0 warnings** nas
  8 crates tocadas · engine debug **E** release verdes (a lição do voronoi) · fingerprint pinado
  intacto · 13 arch-gates verdes · 3 mutações dirigidas sangram (porta de tool → RED de paridade
  bit-exata; rota de SetValue do tuning → RED; véu do Show Wet bakando → RED).

## 2. Foundational/compartilhado tocado (tudo aditivo)

| Arquivo | O quê |
|---|---|
| `ph2d-editor-core/src/ids/chrome/painter_wetpaint.rs` | +7 tool ids, tilt pad/toggle/ring/spoke, 4 ações, paper_visual, tuning; `PAINTER_WETPAINT_CLICKS` **2→16** |
| `ph2d-editor-core/src/ids/chrome/wet_tuning.rs` | **NOVO** — painel/scroll/close + headers/resets/eye/km + a família dinâmica `wet_tuning_*_id(key)` (fnv runtime) + `WET_TUNING_DRAG_HANDLE`/`RESIZE_HANDLE`/`RESIZE_HANDLE_BL` (chrome de drag/resize, 1º smoke do Enio) |
| `ph2d-editor-core/src/widget/scrollbar.rs` + `widget/mod.rs` | `WET_TUNING_SCROLLBAR_ID = NodeId(837)` — **próximo livre: 838** |
| `ph2d-editor-core/src/interaction/dispatch/scroll.rs` | braço 837 → `WET_TUNING_PANEL` no `scrollbar_panel_for_id` |
| `ph2d-editor-core/src/screens/hero/paint.rs` | `WET_TUNING_PANEL` na fallback de z-order |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | entrada `PANEL_A11Y_DELEGATE_OK` p/ `paint_wetpaint_tilt.rs` (classe do `paint_shape_dab.rs`) |
| `ph2d-i18n/src/lib.rs` | +51 chaves `panel.wet_tuning.*` (título, 6 grupos, 40 knobs, K–M, nota) |
| `ph2d-panel-registry-init` | via `ph2d-panel-sync` (gerado) + `EXPECTED_TYPED` +1 (braço `panel-wet-tuning`) |
| `shells/desktop/Cargo.toml` | dep `ph2d-panel-wet-tuning` + `default` + feature `panel-wet-tuning` (a lição do physics: registry-init tem `default-features=false`) |
| `shells/desktop/tests/every_panel_the_shell_drives_is_in_its_registry.rs` | row `("wet_tuning", "panel-wet-tuning")` |
| `shells/desktop/src/forwarding.rs` | `WET_TUNING_PANEL` no `cursor_over_hero_panel` (wheel intercept) |
| `shells/desktop/src/render_loop/painter_bridge.rs` | publish do snapshot + espelho de visibilidade (`tuning_open`; OFF escrito FORA do downcast) + z-bump no edge |
| `ph2d-wet-paint` (engine) | portas ADITIVAS: `dispatch_pressure_dab_lane_blend` · `dispatch_pressure_dab_tool` (prev explícito) · `render_pigment_region_visual` (+`PigmentVisual`; `render_pigment_only_region` delega, off byte-idêntico) · `wet_canvas_now`/`dry_canvas_now`/`fast_dry_now` (sem `capture_history` — o clone de grid por aperto seria a doença do ADR-0117) · `tilt_dir_for_spoke` (cardinais EXATOS) · `knob_defaults()` const · `Tuning::default` delega |
| `ph2d-wet-paint` (engine, **doc 23**) | **MUDANÇA DE COMPORTAMENTO dos tools** (P1-P4, [doc 23](Painter/23_estudo_tools_wet_pigmento.md)): Wet dissolve `sett` sob o stamp · Smear arrasta o seco · Blend molhado re-suspende · Erase resiste por staining — porta única `drying::lift_settled` (extração VERBATIM do re-wet passivo) + `tools::active_lift_gain`. **`Knob::WetLift` apendado (`KNOB_COUNT` 53→54)** + `extStaining` Hidden→Paint (painel Tuning 40→42 rows; i18n +2 chaves). ⚠️ **O pin `fingerprint.rs::PINNED` MUDOU** (justificado no histórico do pin; o pin antigo virou o gate `wet_lift_zero_is_the_old_model_to_the_byte`). Gates: `tests/product_rewet.rs` (5, mutação-provados 5/5) |

## 2.1 O DROPDOWN DE MODO DE PINTURA (2026-07-22) — e o bug que ele fechou

Enio: *"temos na seção de modo da pintura 3 checkbox … no lugar dos checkbox coloque um dropdown
para o modo de pintura com os 4 modos. O padrão é o Digital normal"* — mais o report **"ao entrar
em Impasto e depois sair e selecionar Wet Paint, widgets como o seletor de cor sumiram"**.

**Os dois são a mesma coisa.** `Knife` e `Sculpt` existem só porque o Impasto está ligado, e
sobreviviam a ele ser desligado: o artista escolhia Wet Paint e continuava **com a espátula na
mão** — `paints_no_color()` verdadeiro, Cor e Blend fora da tela, sob um painel que nomeava outro
meio. Medido (probe): `colour=false blend=false` via Knife E via Sculpt; ligar/desligar sem pegar
ferramenta **não** reproduz, que é por que passou despercebido.

| Peça | O quê |
|---|---|
| `ph2d-tool-painter/src/tool/paint/media.rs` (**NOVO**) | `PaintMedia{Digital,Watercolor,Impasto,WetPaint}` — **derivado** das 3 flags, nunca guardado + `PainterTool::paint_media`/`set_paint_media` (a porta única que sabe que são exclusivos) |
| `ids/chrome/painter.rs` | `PAINTER_BRUSH_MEDIA` + `painter_brush_media_option_id(u8)` |
| `ids/chrome/painter_{wetpaint,watercolor,impasto}.rs` | **os 3 `*_ENABLE` REMOVIDOS** (id + array de CLICKS + `populate` + a rota de `Click` no tool) |
| `BrushSettings.media: u8` | o valor do chip; o painel pinta **exatamente uma** seção |
| `apply_brush_preset` | passa a terminar na MESMA porta (era a 2ª porta — ver abaixo) |

**A auditoria achou a SEGUNDA PORTA.** O dropdown **Preset** (uma linha acima) escreve
`watercolor` direto no `BrushSpec`. Medido: Wet Paint armado + *"Watercolor Basic"* deixava
`watercolor` **E** `wetpaint` verdadeiros — o chip lendo *Wet Paint* sobre um pincel de aquarela;
e vindo do Impasto limpava só o slot VIVO, com `brush_by_mode[Knife]` guardando `impasto = true`
para ressuscitar no próximo switch de ferramenta. Fix: o preset termina em `set_paint_media`, e as
3 slots de relevo passam a ser escritas **incondicionalmente** (o early-return *assumia* um
invariante em vez de estabelecê-lo).

⚠️ **O `Preset` continua sendo uma pergunta diferente** (*semeie meu pincel com uma configuração
pronta*) e ficou **fora de escopo**: hoje ele e o Paint Mode mostram os dois a palavra
"Watercolor", o que é adjacência a decidir com o Enio — fundir, renomear, ou deixar.

**Gates:** `ph2d-panel-painter-layers/tests/seam_paint_media.rs` (3, dirigidos por **ponteiro
real**: abre o chip, repinta, clica a opção) + `tool/paint/media/tests.rs` (6). **7 mutações, 7
sangram**; 1 sobrevivente **documentado** (o registro em `populate` é redundante —
`paint_dropdown_chip` já faz `register_if_absent`; a entrada fica pelo `wiring_parity`).

⚠️ **Duas lições ficaram anotadas nos próprios gates:** (a) o repro **nasceu VERDE** com o defeito
reinstalado, porque a regra de ENTRADA (*"escolher um meio USA ele"*) mascara a de SAÍDA — só o
destino **Digital** (que não tem regra de entrada, porque não possui modo nenhum) testa a de saída
sozinha; (b) o gate exigia o chip **Blend** em Watercolor, onde ele é escondido **de propósito**
(doc 13 #4) — over-claim corrigido em vez de contrabandeado.

## 2.2 A SPRITE SE MOVIA EM VEZ DE SER PINTADA (2026-07-22)

Enio: *"o app sai de modo de pintura e não volta mais nem se selecionar a sprite e nem se sair e
entrar novamente no modo de pintura … a sprite se move no canvas e não conseguimos pintar"*.

A sprite **mover** é a assinatura exata de `deliver_canvas_pointer` recusando o Down: ele cai
adiante e quem o pega é o gizmo. Ele recusa com `canvas_size() == (0,0)` — e o canvas fica assim
porque **sair do Painter sem edições pendentes derruba o canvas** (`RasterEditTool::deactivate`
zera `canvas_rgba`/`source_size`) **sem desfazer o binding**.

⚠️ **Um fato, duas cópias, e as duas mentiam.** A shell guardava `last_painter_pushed_entity` (cópia
do `bound_doc` do tool) e a condição de re-push era `memo != Some(bits)`: depois do teardown o memo
ainda nomeava a sprite, então o re-push que consertaria tudo era **pulado justamente porque o memo
dizia que já tinha sido feito**. E mesmo forçando um `bind_document`, a guarda de "mesma sprite" só
re-semeia quando `doc_is_disposable()` — falso para qualquer pilha multi-layer ou esculpida.

| Peça | O quê |
|---|---|
| `PainterTool::needs_document_bind(entity)` (**NOVO**, `tool/documents.rs`) | *outro doc* **ou** *sem pixels* — as duas maneiras de não ter documento |
| `bind_document` | canvas vazio = **não há documento**: não entra na guarda de mesma-sprite nem é stashado (guardá-lo devolveria um doc sem pixels na linha seguinte) |
| `render_loop/painter_bridge.rs` | a decisão de bind passa a ser `painter.needs_document_bind(bits)` |
| `render_loop/mod.rs` | o memo é limpo **sempre** que o Painter sai, não só quando havia bake diferido |

⚠️ **Achado do arredor — um controle MORTO shipado no commit anterior:** segurando o **Smear** do
rail e escolhendo **Watercolor**, nada acontecia. Cada modo tem `BrushSpec` próprio, e a ordem dos
passos punha a flag no slot do *Smear* e só depois pegava o pincel, cujo slot ainda lia `false`.
Impasto passava por **sorte** (espelha em 3 slots) e Wet Paint por **desenho** (`armed` não é
por-slot) — a forma exata que deixa um meio apodrecer sozinho. `set_paint_media` agora **pega a
ferramenta ANTES de armar o meio**.

**Gates:** `tool::documents::rebind_tests` (o repro, com 1 e 2 camadas — a 2ª derrota o
`doc_is_disposable`) + `shells/desktop/tests/the_painter_asks_the_tool_whether_it_needs_a_document.rs`
(2 arch-gates: a decisão pergunta ao tool · a limpeza do memo está **fora** do ramo do bake) + 2 de
modelo (`the_painter_opens_on_the_plain_digital_brush`, `picking_a_medium_while_holding_a_rail_tool_arms_it`).
**5 mutações, 5 sangram.**

⚠️ **Lição de gate:** a 1ª versão do arch-gate só pedia que a limpeza viesse **depois** do teardown,
e a mutação (limpeza de volta para dentro do `if`, uma linha abaixo) **passou** por ele — *"depois"*
e *"fora"* não são a mesma pergunta. Agora ele lê o `}` que fecha o bloco.

## 3. Símbolos que podem COLIDIR

- `NodeId(837)` (scrollbar) — **hand-assigned**; se outra linha tomou 837, renumere (próximo 838)
  e o comentário em `scrollbar.rs` diz a regra.
- Família dinâmica `"wet_tuning.*"` (fnv de strings) — gate `wet_tuning_ids_dont_collide` roda
  sobre as chaves REAIS no crate do painel.
- Chaves i18n `panel.wet_tuning.*` (match do `tr()` — Mergiraf funde adições disjuntas).
- Feature cargo `panel-wet-tuning` (registry-init GERADO + shell) e o id de painel `"wet_tuning"`.
- `EXPECTED_TYPED` +1 — se outra linha também adicionou painel, **conte, não escolha**.
- `PAINTER_BRUSH_MEDIA` + `painter_brush_media_option_id` (ids novos em `chrome/painter.rs`) e a
  **remoção** de `PAINTER_{WETPAINT,WATERCOLOR,IMPASTO}_ENABLE` — se outra linha os referencia, ela
  quer `set_paint_media` / `PaintMedia`. Os 3 arrays de `*_CLICKS` encolheram (16→15, 7→6, 17→16).

## 4. Contratos congelados encostados

**Nenhum.** `Tool=12`/`CanvasPaintTool=1`/`PanelEvent=4` intactos (tudo viaja pelos canais
genéricos); `NodeOp`/`NodeManifest` intocados.

## 5. O que só o `ship.sh` pega

- fmt: rodado com o pin sobre as 8 crates tocadas; risco residual só em arquivo NÃO tocado.
- `cargo-machete`: `ph2d-panel-wet-tuning` usa todas as deps declaradas; o shell USA
  `ph2d_panel_wet_tuning::set_current_brush` (não é dep morta).
- deny/audit: **zero dependência externa nova** (só crates internas).
- typos: prosa nova em i18n/docs — vocabulário do modelo (Kubelka-Munk etc.).

## 6. O que SMOKE-testar (nada foi smokado — tudo pendente)

`env PH2D_WETPAINT_SMOKE=1 cargo run -p ph2d-host-desktop --release` (a cena arma o wet):

1. **Seção básica:** o rádio de tools (Erase acende com o chip do rail e vice-versa — duas vistas
   de um rádio); o TILT dial (arrastar snapa e LIGA; toggle preserva direção; com Gravity>0 o
   pingo corre na direção do dial); Wet canvas (o próximo traço sangra em qualquer lugar — ligue
   Show Wet pra VER a folha úmida); Dry canvas (assenta na hora); Fast dry (seca com os anéis de
   borda); Show Wet (véu frio + brilho de menisco; **desarmar o wet NÃO pode bakar o véu**);
   Paper (o grão entra na tinta; **baka** de propósito); Tuning (abre o painel lateral).
2. **Painel Tuning:** os 40 sliders vivos (ex.: Leveling/Brake mudam o fluxo com água na tela;
   Bristle count re-textura o depósito; Contrast/Fibres/Grooves re-cozem o papel do ENGINE — e
   **somem** quando o Paper slot do artista arma); resets por-knob e por-grupo; o olhinho do
   PAPER = o checkbox Paper; K–M mixing muda a mistura de cores; Glaze muda lavagem-sobre-seco;
   fechar no X = desmarcar Tuning. **Chrome (fix do 1º smoke):** arrastar pela BARRA DE
   TÍTULO move o painel; os grippers dos 2 cantos inferiores redimensionam (mesma maquinaria
   `BlenderHit` do Inspector, deltas sob `WET_TUNING_PANEL`, clamp ao viewport); e o heading
   ENGOLE o clique — com o corpo rolado, a row que fica atrás do título não pode ser
   scrubada (a banda de drag é registrada por ÚLTIMO no paint: last-registered-wins; gates
   em `tests/panel_chrome.rs`, 2 mutações provadas — banda registrada cedo · handle fora do
   `populate`).
3. **As 5 tools novas:** Smear arrasta, Blend remistura tinta SECA, Wet molha sem pigmento, Dry
   sela, Blow empurra o filme (a sim fica viva sob o gesto do Blow). Com Symmetry ligada, Smear/
   Blow deslocam LOCALMENTE em cada cópia (o prev por-lane).
4. **O pigmento responde às tools (doc 23):** pinte com tinta pura (Water 0), Fast dry,
   e passe o **Wet** — a tinta seca DISSOLVE sob o pincel (o reporte original); com a área
   molhada, **Blow** empurra e **Smear** arrasta a cor re-suspensa; **Smear no seco**
   esfrega como smudge de raster (espalha por repetição); **Blend** sobre seco re-mistura
   (como antes) e sobre molhado re-suspende (sangra depois); **Staining** no painel Tuning
   (grupo PAINT) controla tudo — a 1.0 a tinta seca fica pinada; **Rewet lift** (grupo
   TOOLS) é a força do dissolve, 0 = modelo antigo. ⚠️ Molhar e NÃO mexer re-assenta o
   pigmento quando a água morre — é física, não bug.
5. **O dropdown de MODO DE PINTURA (§2.1):** o chip **Paint Mode** no topo da metade de aparência
   lista *Digital · Watercolor · Impasto · Wet Paint* e abre **Digital** num projeto novo; escolher
   um pinta **só** a seção dele (as três checkboxes `Enable` sumiram — o chip é o interruptor).
   ⚠️ **O repro do bug:** entre em **Impasto**, clique a **faca** (ou um verbo do Sculpt), e então
   escolha **Digital** — o seletor de cor e o Blend têm de continuar na tela, e a ferramenta na mão
   volta a ser o pincel. Idem indo para **Wet Paint**. E confira o **Preset** logo acima: com Wet
   Paint armado, escolher *"Watercolor Basic"* tem de deixar o chip lendo **Watercolor**, não os
   dois meios ligados.
6. **O canvas volta (§2.2):** com a sprite selecionada, entre no Painter, **saia sem pintar nada**,
   e volte — tem de dar para pintar. (Antes: a sprite era **arrastada** pelo gizmo, e nem
   re-selecionar nem re-entrar no Painter devolvia a pintura.) Confira também o par que a auditoria
   achou: pegue o **Smear** do rail e escolha **Watercolor** no Paint Mode — o chip tem de FICAR em
   Watercolor (antes voltava sozinho para Digital).
7. **Zero regressão:** o modo Paint comum segue byte-idêntico (G0b verde); wet Paint padrão
   idem (boot equivalence + fingerprint com pin justificado; `wetLift=0` reproduz o pin
   antigo ao byte).

## 7. Decisões que o smoke pode reabrir (nomeadas, não escondidas)

- Métodos de stroke NÃO-incrementais (Line/shapes) com tool ≠ Paint: o preview flat mostra
  PIGMENTO e o commit aplica a TOOL (o esboço derrete na ação) — coerente com o doc 21, mas o
  preview "mente" a cor para Wet/Dry/Blow; se incomodar, a saída é restringir o Method sob tools.
- O véu do Show Wet sobre camada transparente é uma ARDÓSIA translúcida (adaptação nomeada do
  modelo, que escurece uma folha opaca); constantes em `wetpaint/composite.rs`.
- Tooltips ricos (KNOB_DOCS do modelo) ficaram fora (§4 do plano).
- A visibilidade do painel via bridge não tem gate de shell dedicado (o espelho é 3 linhas no
  `painter_bridge`; o gate de registry cobre a metade estrutural).

*Linha `Painter` pronta (19 commits). Aguardo ordem de integração.*
