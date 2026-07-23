# HANDOFF DE INTEGRAÇÃO — `line/Painter` · Wet Paint (doc 22/23) + reorg do Impasto + o modo de pintura

> DIRETRIZ §1.5.9. A linha está FECHADA e **NÃO integra nem faz ship** (o Enio cancelou
> explicitamente a integração/ship/CI desta jornada em 2026-07-22 — outros agentes vão
> trabalhar em suas linhas; este handoff espera o integrador da próxima janela).

> **O escopo cresceu além do doc 22** (o nome do arquivo é histórico). Em ordem:
> **(A)** a UI completa do Wet Paint — doc 22 (seção básica + tools + TILT + ações de canvas + Paper +
> o painel lateral **Wet Tuning** com a tabela inteira de knobs) — e **(B)** o **doc 23** (o pigmento
> seco responde a Wet/Smear/Blend/Erase). Depois, disparados por smokes do Enio:
> **(C)** a reorganização dos cards do **Impasto** numa lista só · **(D)** o **dropdown de Modo de
> Pintura** (os 4 meios, que fechou o bug do modo órfão, §2.1) · **(E)** o **canvas derrubado** que
> impedia pintar (§2.2) · **(F)** os smokes abrindo em **Digital** (§2.3) · **(G)** os **defaults de
> produto** do Wet Paint (§2.4). Tudo aditivo ou contido no módulo, exceto o que a §2/§3 nomeiam.

## 1. Identidade

- **Branch:** `line/Painter` · **base do fork:** `13a04c7aa` (o main integrado de 2026-07-22).
- **Commits:** **24** — `13a04c7aa..HEAD` (`HEAD` = `021b89263`). Waves A/B nos commits `233797b5e`
  (W1) → `82dfeac3f` (Wet Tuning arrasta/redimensiona), com `b782226c7`+`cd31c44ca` = doc 23;
  wave C em `939d4de6e`+`2b4ac8b87`+`9512fb400`+`068b08789`; D em `3a8afa3fa`; E em `11ad184bd`;
  F em `5ed714db0`; G em `da9af82f3`. O resto são commits `docs(painter)` (handoff + CLAUDE.md).
  Checkpoint de reversão do doc 23: tag `checkpoint-pre-wet-tools-rework`.
- **Planos:** [`docs/Painter/22_plano_wet_tuning_ui.md`](Painter/22_plano_wet_tuning_ui.md) ·
  [`docs/Painter/23_estudo_tools_wet_pigmento.md`](Painter/23_estudo_tools_wet_pigmento.md).
- **Crates tocadas (8):** `ph2d-wet-paint`, `ph2d-tool-painter`, `ph2d-panel-painter-layers`,
  `ph2d-panel-wet-tuning` (**NOVA**), `ph2d-editor-core`, `ph2d-i18n`, `ph2d-panel-registry-init`,
  `shells/desktop` (+ `Cargo.lock`, docs, CLAUDE.md).
- **Gate batched (rodado no fechamento, 2026-07-22):**
  - `scripts/nextest-impacted.sh` → **5058 passed, 223 skipped, 0 failed**.
  - `cargo clippy --workspace --all-targets` → **0 warnings**.
  - `cargo fmt --all --check` (pin 1.95) → **limpo**.
  - `typos` (raiz, com `.typos.toml`) → **0 erros**.
  - LOC caps: `architecture_workspace_file_loc_cap` **+** `shells/desktop file_loc_caps` → **verdes**
    (`wetpaint.rs` a 700 exatos, `paint.rs` a 700).
  - `cargo machete` nas 3 crates de código-novo → **0 deps mortas**.
  - `cargo test -p ph2d-wet-paint` (engine, inclui o **fingerprint** pinado) → **14/14** —
    o pin do doc 23 mudou COM justificativa; nada nesta jornada tocou o engine depois disso.
  - **Contagem de mutação da jornada:** cada wave traz seus gates mutação-provados (doc 22 = 3 ·
    doc 23 = 5 · dropdown/§2.1 = 7 · canvas/§2.2 = 5 · smokes/§2.3 = 1 · defaults/§2.4 = 5), com os
    sobreviventes DOCUMENTADOS onde há (o `populate` redundante do chip de meio).

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
| `ph2d-editor-core/src/ids/chrome/painter.rs` (**§2.1**) | **+** `PAINTER_BRUSH_MEDIA` + `painter_brush_media_option_id(u8)` (o chip de meio). ⚠️ **NÃO-aditivo:** `painter_{wetpaint,watercolor,impasto}.rs` **PERDERAM** `*_ENABLE` (id + row em `*_CLICKS`, que encolheu 16→15 · 7→6 · 17→16). Ver §3 |
| `shells/desktop/src/render_loop/{painter_bridge,mod}.rs` (**§2.2**) | a decisão de bind virou `painter.needs_document_bind(bits)` (não o memo `last_painter_pushed_entity`); o memo é limpo em TODA saída do Painter. Arquivos de shell já desta linha (doc 22 também os toca) |
| `shells/desktop/src/{impasto,wetpaint}_smoke.rs` (**§2.3**) | os `arm_brush_once` **pararam de forçar o meio** (abrem em Digital); só-shell, sem foundational |

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

## 2.3 OS SMOKES ABREM O PAINTER EM DIGITAL (2026-07-22)

Enio: *"quando abro o painter o Wet paint ainda é o que aparece primeiro mas deveria ser o digital
como o padrão inicial do app"*.

O `PainterTool::default()` **já** abre em Digital (§2.1); o culpado era o **smoke**. Os dois
`arm_brush_once` chamavam `set_paint_media(<seu meio>)`, então `PH2D_WETPAINT_SMOKE=1` abria o
Painter direto em Wet Paint e `PH2D_IMPASTO_SMOKE=1` em Impasto. ⚠️ **Era a cicatriz que o doc do
`impasto_smoke` já descrevia** (*"nothing here is armed in code … the smoke that arms state under the
table skips exactly the seam it was supposed to prove"*) — e o código o contradizia.

Fix: os smokes dão o canvas e abrem em **Digital**; o artista escolhe o meio no dropdown.
- **impasto:** mantém `set_brush_size_px(40)` (cai no slot **Paint compartilhado** — o Deposit do
  Impasto **é** `PaintMode::Paint`), dropa o `set_paint_media`.
- **wetpaint:** o slot do Wet Paint (11) é separado, então tamanho por-slot não chega lá sem armar
  (dropado); a **cor** é sincada entre todos os slots (`sync_brush_color_across_modes`), então o azul
  chega na água de graça. **Nenhuma API nova.**

**Gate:** `shells/desktop/tests/the_smokes_open_the_painter_in_digital.rs` (a decisão é env-gated ⇒
nenhum unit test a alcança) — lê a fonte dos 2 smokes e recusa `set_paint_media(PaintMedia::<não-
Digital>)` + controle positivo. 1 mutação, sangra.

## 2.4 DEFAULTS DE PRODUTO DO WET PAINT (2026-07-22)

Enio: Spacing **0.025** (só wet), Pigment **800**, Paper Gate **0.4**, Felt **0.03**, Bristle Size
**2.0**, Bristle Count **2000**.

⚠️ **São defaults de PRODUTO, não do modelo.** O engine é um port 1:1 do reference JS e os `KNOB_DEFS`
(SPEC §16) são os defaults DELE — mexer neles quebraria o fingerprint e a fidelidade. Então os valores
do Enio vivem no **TOOL**, não no engine (`ph2d-wet-paint` intocado, fingerprint 14/14).

| Peça | O quê |
|---|---|
| `state_default.rs` | slot próprio do WetPaint com `spacing: 0.025` (mais denso que o 0.05 de Smear/Blur/Clone); os outros modos intactos |
| `WetKnobs::ENGINE_BOOT` (**NOVO**) | o boot real do engine (`Engine::new` / SPEC) — a **baseline** do reconcile |
| `WetKnobs::DEFAULT` | passou a ser o **perfil de produto** (o boot + 5 tweaks); usado por `default()`, `reset_group`, `set(NaN)`, `FALLBACK_BRUSH` |
| `WetEngineFacts::BOOT.knobs` | `ENGINE_BOOT` (não `DEFAULT`) — o `applied` da sessão inicia na baseline |

⚠️ **A DESACOPLAÇÃO é a espinha.** `ENGINE_BOOT` e `DEFAULT` eram a MESMA const. Se colapsados, o
reconcile faria early-return no 1º batch (`applied == facts`) e o engine ficaria em 600 sob um painel
lendo 800. O reconcile só empurra o produto porque ele **difere** da baseline (`applied` inicia em
`ENGINE_BOOT`, o 1º `facts` é `DEFAULT` produto ⇒ delta ⇒ push).

**Gates** (`wetpaint/tests_doc22.rs`): `the_reconcile_baseline_is_the_engines_own_boot` (baseline==SPEC,
difere do produto, engine boota na SPEC ANTES do reconcile) · `the_tool_default_reaches_the_engine_exactly`
(o produto chega ao engine pós-stroke, bit a bit) · `the_wet_paint_knob_defaults_are_the_ones_enio_chose`
(pina os 5 + prova que os outros == boot) · `wet_paint_opens_at_the_dense_default_spacing`. **5 mutações,
5 sangram** — colapsar a baseline sangra **duas** (a baseline E a entrega).

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
5. **Os defaults do Wet Paint (§2.4):** entre em Wet Paint (pelo dropdown) e confira o Stroke =
   **Spacing 0.025** e o Tuning = **Pigment 800 · Paper Gate 0.4 · Felt 0.03 · Bristle Size 2.0 ·
   Bristle Count 2000**. O reset de grupo/seção volta a ELES (não ao boot do engine).
6. **O padrão de abertura é DIGITAL (§2.1 + §2.3):** abra o Painter (mesmo sob `PH2D_WETPAINT_SMOKE=1`
   ou `PH2D_IMPASTO_SMOKE=1`) e o chip **Paint Mode** tem de ler *Digital*, sem nenhuma seção de meio
   pintada. Escolha o meio no dropdown para trabalhar.
7. **O dropdown de MODO DE PINTURA (§2.1):** o chip **Paint Mode** no topo da metade de aparência
   lista *Digital · Watercolor · Impasto · Wet Paint* e abre **Digital** num projeto novo; escolher
   um pinta **só** a seção dele (as três checkboxes `Enable` sumiram — o chip é o interruptor).
   ⚠️ **O repro do bug:** entre em **Impasto**, clique a **faca** (ou um verbo do Sculpt), e então
   escolha **Digital** — o seletor de cor e o Blend têm de continuar na tela, e a ferramenta na mão
   volta a ser o pincel. Idem indo para **Wet Paint**. E confira o **Preset** logo acima: com Wet
   Paint armado, escolher *"Watercolor Basic"* tem de deixar o chip lendo **Watercolor**, não os
   dois meios ligados.
8. **O canvas volta (§2.2):** com a sprite selecionada, entre no Painter, **saia sem pintar nada**,
   e volte — tem de dar para pintar. (Antes: a sprite era **arrastada** pelo gizmo, e nem
   re-selecionar nem re-entrar no Painter devolvia a pintura.) Confira também o par que a auditoria
   achou: pegue o **Smear** do rail e escolha **Watercolor** no Paint Mode — o chip tem de FICAR em
   Watercolor (antes voltava sozinho para Digital).
9. **Zero regressão:** o modo Paint comum segue byte-idêntico (G0b verde); wet Paint padrão
   idem (boot equivalence + fingerprint com pin justificado; `wetLift=0` reproduz o pin
   antigo ao byte).

## 7. Decisões que o smoke pode reabrir (nomeadas, não escondidas)

- **`Preset` vs `Paint Mode` mostram os dois "Watercolor"** (§2.1): o Preset semeia o `BrushSpec`
  inteiro, o Paint Mode troca o meio — perguntas diferentes, mas o rótulo colide. Fundir/renomear/
  deixar é decisão do Enio.
- Métodos de stroke NÃO-incrementais (Line/shapes) com tool ≠ Paint: o preview flat mostra
  PIGMENTO e o commit aplica a TOOL (o esboço derrete na ação) — coerente com o doc 21, mas o
  preview "mente" a cor para Wet/Dry/Blow; se incomodar, a saída é restringir o Method sob tools.
- O véu do Show Wet sobre camada transparente é uma ARDÓSIA translúcida (adaptação nomeada do
  modelo, que escurece uma folha opaca); constantes em `wetpaint/composite.rs`.
- Tooltips ricos (KNOB_DOCS do modelo) ficaram fora (§4 do plano).
- A visibilidade do painel via bridge não tem gate de shell dedicado (o espelho é 3 linhas no
  `painter_bridge`; o gate de registry cobre a metade estrutural).

## 8. Ordem de operação para o integrador

1. `git merge --ff-only line/Painter` (ou rebase por cima do main de HOJE — a §3 lista o que pode
   colidir; o `ph2d-panel-wet-tuning` é crate nova, então o conflito real seria em `Cargo.lock` /
   `ph2d-panel-registry-init` gerado / i18n / os ids `chrome/painter*.rs`).
2. Rode `scripts/foundational-integrate.sh` (o gate da árvore combinada) — os números da §1 são
   desta árvore isolada; o gate do main-de-hoje pode surfar latentes de OUTRAS linhas (é o esperado,
   DIRETRIZ §1.5.9).
3. ⚠️ **`PROJECT_SCHEMA` / contratos:** esta linha **não bumpou schema nem contrato** (§4). Se outra
   linha bumpou o `PROJECT_SCHEMA`, **o valor se CONTA, não se escolhe** — esta não entra na soma.
4. `EXPECTED_TYPED` +1 e `NodeId(837)` são as duas colisões numéricas — **conte, não escolha** (§3).
5. Só então o ship do integrador (`./scripts/ship.sh` → push → babysit), **por ordem do Enio**.

*Linha `Painter` FECHADA (24 commits, `13a04c7aa..021b89263`). Gates de fechamento verdes (§1).
Aguardo ordem de integração — não integro nem pusho por conta própria.*
