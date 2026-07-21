# HANDOFF — line/Painter: Wet Paint, W0 FECHADO → continuação (W1..W3)

> Continuação de [`HANDOFF_line_Painter_wet_paint_2026-07-20.md`](HANDOFF_line_Painter_wet_paint_2026-07-20.md)
> (a tarefa, as regras do Enio, o mapa de integração — **releia o §4 e o §5 de lá antes do W1**).
> Protocolo de troca: `cd Worktrees/line-Painter && pwd && git branch --show-current` ANTES de tudo.

## §1 — O QUE ESTÁ FEITO (2026-07-20, HEAD `b35bb036`)

- **[ADR-0134](architecture/decisions/0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md)**
  aceito: supersede o ADR-0096 NESTE ponto; fixa nome (crate `ph2d-wet-paint`, prefixo
  `wetpaint_`, rótulo de UI "Wet Paint"), contrato de neutralidade (OFF byte-idêntico), lei de
  integração e kill criteria. ⚠️ **Tem uma EMENDA pendente de veto do Enio**: a barra da sim viva
  foi re-derivada da MEDIÇÃO (tabela no ADR) — sessão representativa pior classe ≤ 2 ms (medido
  0,84) · flood §18 pior classe ≤ 12 ms (medido 8,3–9,4; o 8 original era palpite pré-medição).
  O solver é **serial POR SEMÂNTICA** (o brake lê `wet` VIVO escrito no mesmo passe; o drying lê
  o vizinho esquerdo pós-update) ⇒ ADR-0109 (bandas bit-idênticas) é inaplicável — não re-derive.
- **W0 — a crate `ph2d-wet-paint`**: porte 1:1 de `docs/Painter/ph2d_wet_paint/js/engine/`,
  módulo a módulo (16 módulos; leia o doc de `lib.rs` pra lei do porte: **aritmética f64,
  storage f32**, semântica JS só via `jsmath`, transcendental só via `libm =0.2.16`).
  - **Suite §18 completa e VERDE** (`tests/acceptance.rs` §18.1–.10, `tests/acceptance_budget.rs`
    §18.11–.12 + integral §7 — os orçamentos *binding* de massa/água/cobertura nas DUAS bitolas
    passam). `tests/perf.rs`: gates de wall-clock são `#[ignore]`, rodam com
    `cargo test -p ph2d-wet-paint --release --test perf -- --ignored --nocapture`
    (metodologia: **mediana por classe de cadência** — max de amostra única é ruído de scheduler).
  - **`tests/fingerprint.rs`**: fingerprint de sessão pinado — TODA reescrita de hot loop se prova
    byte-idêntica contra ele (o pin tem histórico comentado; só move com justificativa semântica).
  - **W0-verify (auditoria adversarial JS↔Rust, workflow 16 pares + verificadores)**: 6
    divergências reais achadas e corrigidas (ordem de soma do advect · lattices do papel eram
    Float32Array · guard falsy do std · `spacing||2` · `reset_group` engolia as notificações de
    rebuild → `Engine::reset_knob_group` é a porta do painel · ToInt32 pleno no opacity). A
    **família NaN é divergência ACEITA e documentada no doc do `jsmath`** (exige estado já
    envenenado; o Rust recupera onde o JS fica envenenado) — não "conserte".
- **Sem runtime JS na máquina** (node/deno/bun ausentes) — irrelevante: a suite Rust É a
  aceitação; o app de referência roda no browser (`python3 -m http.server`) pro smoke visual.

## §2 — W1 (PRÓXIMO): o modo no painter — o blueprint reconhecido

- **`PaintMode::WetPaint`** em `ph2d-tool-painter/src/tool/paint/paint_mode.rs` — variant novo,
  `slot() = 11`, `PAINT_MODE_COUNT = 12` (slot próprio de `BrushSpec`, o padrão que a Faca
  acabou de estabelecer). Chip na lista de tools? — é decisão do painel (W3); o modo primeiro.
- **Master switch**: `BrushSpec::wetpaint` OFF por default + **gate de fingerprint OFF
  byte-idêntico** (padrão `impasto_off_is_byte_identical`). Os smokes aprovados são o contrato.
- **Pendura em `stamp_dabs_inner`** (`stamp_route.rs:250`) com early-return ANTES das rotas de
  cor, como o Sculpt faz (`:276`) — o engine é dono do depósito de pigmento. Isso dá
  Symmetry/Tiling/shape editors/pressão/Jitter de graça.
- **O tick**: `Tool::on_tick` (`trait_impls.rs:541` — o watercolor já seca o papel por ali) +
  acumulador de 40 Hz fixos (clamp 5 passos, SPEC §5). A sim PAUSA com pointer down (menos blow).
- **O Grid do engine** dimensionado ao canvas da camada ativa do painter; o preview compõe o
  `render_region` do engine sobre a base congelada (o modelo do watercolor: composite sobre base).
- **A pressão sintética do SPEC §8 é SUBSTITUÍDA** pela pressão real do stroke engine do painter;
  o trail/§10 do engine CONSOME os dabs do painter (o `stroke.rs` da crate segue existindo pros
  testes §18, mas o produto alimenta `trail::accumulate_paint` direto com dabs reais).
- **Silhueta**: compor `silhouette_at` (dab.rs) com a bristle (fator, como o Grain) — a bristle
  do SPEC §7 é o *default* do modo; Grain do artista substitui.
- **Undo**: a sessão molhada (grid do engine) entra no `ModelSnapshot` **no mesmo commit** que
  criar o estado (lição §10.4 do impasto). O que capturar: `snapshot_grid` já existe na crate.
- Smoke: cada wave entrega cena `PH2D_WETPAINT_SMOKE=N` auto-play.

## §2.4 — W1 FECHADO (2026-07-20, HEAD `13009e54`) — AGUARDANDO SMOKE DO ENIO

**Smoke: `PH2D_WETPAINT_SMOKE=1 cargo run --release -p ph2d-host-desktop`** — canvas branco
1024², brush já armado em Wet Paint (azul, 24 px): escolha o Painter e arraste; solte e ESPERE
(a água segue nivelando/sangrando/secando a 40 Hz). ⚠️ O smoke arma o MODO em código de
propósito e documentado: até o W3 não existe chip/painel que selecione Wet Paint.

O que entrou (commits `2437a0cc..13009e54`):
- **Portas de produto no engine** (`822f7ae4`): `dispatch_pressure_dab` (o §9 com pressão REAL +
  raio REAL; `dispatch_dab` vira wrapper — fingerprint prova delegação byte-idêntica) ·
  `begin/segment/end_direct_stroke` (traço alimentado por dabs reais, sem history do engine) ·
  `render_pigment_only_region` (o full render virou wrapper da região — um corpo por célula).
  Gates: região==full dentro do rect + sentinela fora; depósito + gating do sim.
- **O modo no tool** (`8a89de7a`, `tool/paint/wetpaint.rs`): sessão = engine + base congelada
  (`Arc`) + **guard de identidade de canvas** (o `wet_session_canvas` do watercolor, EAGER — no
  dab E no tick, porque o sim composita sem pen-down; undo/fill/troca de layer matam a sessão
  por `Arc::ptr_eq`). **Display-state, não document-state**: o composite escreve `canvas_rgba`
  por dirty-rect, **encerrar a sessão É o bake** (os pixels já estão lá), e o grid fica FORA do
  `ModelSnapshot` — um `GridSnapshot`/passo seria ~235 MB a 2048² (ADR-0117). Consequência
  honesta, pro smoke julgar: **undo de um traço wet devolve o LOOK e mata a água** (o redo não
  ressuscita a sessão). Rota em `stamp_dabs_inner` ANTES do passe de altura (relevo de tinta que
  ESCOA seria errado 2×); tick 40 Hz clamp 5 em `paint_tick`; pen-up fecha o traço direto.
  Célula 1-based: pixel `p` → célula `p+1` (o `view.toCell` do reference).
- **Depósito só em gesto VIVO + métodos CUMULATIVOS** (Dots/Airbrush/Space): `live_gesture`
  armado no `paint_begin` — ⚠️ o sinal NÃO pode ser `paint.stroke` (o lifecycle `mem::take` o
  stroke durante o stamp; foi o 1º vermelho dos gates). DragDot/Anchored/Line re-carimbam por
  frame e o depósito de fluido NÃO é idempotente (I2) — recusados até o W2 desenhar (idem shape
  editors, que nem chegam ao `paint_begin`).
- **4 gates mutation-tested** (inline em `wetpaint.rs`; 3 mutações sangram: rota · guard do
  tick · teardown de modo) + suíte inteira do tool verde (743).
- **LOC**: `solver.rs` 858→539 (meu débito do W0; split `solver/advect.rs`+`solver/project.rs`,
  fingerprint prova byte-identidade) · `paint.rs` 713→**700 exatos** (re-ancoragem de
  doc-comments pra caber `mod`+campo). ⚠️ **7 ofensores HERDADOS seguem no gate
  `workspace_src_files_under_loc_cap`** (das waves AA/rake anteriores da linha, pré-meus
  commits): `watercolor_render` 751 · `sculpt_tests/w3` 743 · `sculpt` 715 · `watercolor_field`
  709 · `dab` 708 · `spec` 701 — fecham no gate batched do fechamento da linha.

**Decisões registradas p/ Enio validar no smoke:** cor por-TRAÇO (Randomize per-dab = W2) ·
knobs = defaults §16 do reference (painel = W3) · seção Watercolor ligável em modo WetPaint
ainda não é escondida (incompatível; W3, lei #3) · gatilho de commit da sessão = mutação alheia /
troca de modo (secagem completa NÃO encerra sozinha — decisão aberta de produto).

## §2.4b — SMOKE DO W1: OK (Enio) + 2 defeitos FECHADOS · W2 EM ANDAMENTO

**Veredito do Enio:** *"a física do líquido funciona"* · *"A cor está preta"* · *"vc suprimiu a
seção Impasto — uma das regras era não afetar o que já existia"* · **"W1 smoke OK. Siga w2"**.

- **Cor preta FECHADA (`69da962b`)**: o engine fala sRGB **0..255** (boot `[50,140,210]`; o
  render escreve os planos via `clamp_u8`) e o brush guarda 0..1 — faltava o ×255. ⚠️ O gate
  novo teve o próprio oráculo corrigido: varria SÓ a linha do traço e falhou sobre produto
  CORRETO (o trail deposita com estrutura vertical; célula-máxima estava 4 linhas abaixo) — o
  oráculo agora varre o canvas inteiro.
- **Seção Impasto FECHADA (mesmo commit)**: WetPaint entrou em `impasto_section_applies` — a
  régua do Enio VENCE o precedente do watercolor ("thin paint, no body" excluiria), porque a
  seção hoje hospeda a lista das DEZ ferramentas + o Lighting do canvas: escondê-la tira o
  seletor de ferramenta. O radio acende NADA (`IMPASTO_TOOL_NONE` = u8::MAX, out-of-range
  documentado do `SegmentedAdaptive`) e nenhum card de ferramenta pinta (braço `TOOL_NONE` no
  painel) — acender Deposit seria o radio mentiroso que o rail recusou pra Faca.
- **W2.1 (`73cfbb01`) — Symmetry/Tiling provados de graça**: 2 gates de seam (massa suspensa
  espelhada / na borda oposta) com controle de oráculo executado (features OFF ⇒ RED nos dois).
- **W2.2 (`e00786cf`) — Randomize Color per-dab**: porta `Engine::set_stroke_color` →
  `Trail::set_base_color` = **RECARGA da tinta** (reservatório + planos do tip, a metade de cor
  do `start_stroke`). ⚠️ A v1 só-reservatório foi MEDIDA INERTE: o tip só caminha pra base via
  `Knob::TipClean`, **cujo default de boot é 0.0** — gate nasceu vermelho (219 vs 21 de verde).
  Recarga é a semântica honesta (dab jitterado = carga nova; o suor do pickup se re-acumula).
  Tool: `d.color` por dab com detector de mudança (sem jitter ⇒ zero chamadas; a cor de início
  de traço também vem de `dabs[0].color` agora, não do brush).

**W2.3a (`2e88f4cb`) — "Simetria Circular não está correta" (report do Enio) FECHADO — RE-SMOKE
PENDENTE:** causa estrutural: o trail é UMA janela 123² ancorada no 1º dab — a lista intercalada
de cópias (symmetry/tiling) tinha toda cópia fora da janela **descartada em silêncio** e o chord
entre cópias era lixo. ⚠️ **Meus gates do W2.1 eram verdes POR ACIDENTE** (janelas alternando
âncoras salvavam metade da massa de cada lado — a RAZÃO passava sobre traço meio-descartado; a
lição [[feedback_a_green_gate_may_be_green_by_accident]] na prática). Cura: **portas por RAIA**
no engine (`painter/doors.rs`: pool `lane_trails`, raia pode nascer no meio do traço — wrap de
Tiling na borda; §9 extraído p/ `pressure_dab` compartilhado, split byte-idêntico provado pelo
fingerprint) + **pareamento geométrico** no tool (vizinho mais próximo dentro do raio; perto do
centro radial as cópias convergem e trocar de raia é inofensivo — as posições coincidem).
Oráculos endurecidos: circular 6 setores (cada ≥ 0,5× o máx) · espelho ganhou o **oráculo
ABSOLUTO** (lado original ≥ 0,8× um traço solo — a razão sozinha era verde sob o bug) · tiling
0,1→0,5. Mutação tudo-na-raia-0: os 3 sangram.

**W2.3b LANDOU (`298703b0` + splits `f8b3507d`) — a silhueta do PAINTER dirige o stamp:**
falloff × Shape × Flatten&Rotate entram pelo `silhouette_at` (fonte única) via closure por dab
na porta shaped (`dispatch_pressure_dab_lane(.., sil)` → `accumulate_paint_shaped` → UM corpo de
pixel; `accumulate_paint` delega `None`, fingerprint prova o porte intacto). Prep = a receita do
impasto; a sessão é `take()`da durante o batch (borrows disjuntos de `self.paint`). Bristle fica
como fator default (W2.4 troca por Grain). Gate red-first: dab achatado deposita BANDA (mutação
`None` sangra). Splits: `trail/transfer.rs` (movimento puro) + `wetpaint/tests.rs` (gates).
⚠️ Smoke do Circular JÁ aprovado; **o do W2.3b (Flatten/Shape no wet) ainda não** — smoke junto
com o próximo lote. Re-smoke Circular OK (Enio).

**W2.4 LANDOU (`3272e6d1` + fmt-sweep `bab935a9`) — o Grain do artista substitui a bristle:**
a bristle é o grain DEFAULT do fluido, então slot de Grain armado (`brush.texture.is_active()`)
= closure por dab que SUBSTITUI o `sample_bristle` no stamp shaped (multiplicar os dois
escureceria em dobro; em todo o resto do app o Grain É a textura). A lei por-pixel é
`dab::grain_at` — a MESMA porta única da rota de cor e do kernel de altura do impasto (Depth +
granulation + stencil dobrados). Prep espelha o impasto: `grain_basis` DEPOIS do `shape_basis`,
mesmo stream de RNG (ordem Shape-antes-de-Grain). Plumbing: `dispatch_pressure_dab_lane(..,
sil, grain)` → `accumulate_paint_shaped(.., grain)` → `for_each_stamp_pixel_shaped` (`texv =
grain(x,y)` quando Some; `debug_assert` grain-sem-sil no braço unshaped — parâmetro descartado
em silêncio é armadilha). Grain inativo = `None` = caminho byte-idêntico (fingerprint intacto).
2 gates mutation-tested, cada mutação sangra SÓ o seu: engine (`the_hosts_grain_replaces_the_
bristle`, listras vetam colunas ímpares a zero EXATO + controle positivo bristled) · tool
(`the_artists_grain_textures_the_wet_deposit`, Checker **Tiled** depth 1 — canvas-anchored,
senão dabs sobrepostos re-faseiam e enchem os zeros uns dos outros; oráculo tríplice
vetoed/kept/mass — o kept-count mata o dim global). ⚠️ O fmt-sweep é dívida do W0 (a crate
nunca passou pelo fmt PINADO 1.95; `--no-verify` pulava o hook) — commit separado, zero lógica.
⚠️ Smoke do W2.4 pendente junto do lote (Grain procedural/imagem num traço wet).

**W2.5 LANDOU (`50df3c2f`) — Selection/protection/alpha-lock confinam o depósito wet:** os gates
são aplicados no ÚNICO ponto onde o módulo escreve o canvas (`wetpaint_composite`): keep-lerp
para a base congelada via `splat_keep` (a lei do composite watercolor — garantia dura
independente de onde a água chegou; a sim NÃO é gateada de propósito, o fluido flui) + pin de α
no alpha-lock. A referência de α é `sess.base`, servida pela cadeia do `wet_splat_gates` (braço
novo wet-session ANTES do wet_session_base — porta única, nunca 2ª porta). ⚠️ **O achado da
mutação sobrevivente (2 gates nasceram verdes sob mutação):** o wrapper snapshot/restore do
`stamp_dabs` já confinava o stamp E MATAVA a sessão a cada batch sob seleção — `restore_*` roda
`Arc::make_mut` num canvas cujo Arc a sessão também segura (refcount 2 ⇒ clone ⇒ ponteiro novo)
e o guard de identidade lê como mutação estrangeira ⇒ **sob seleção a água nunca vivia além de
um batch**, com todo assert de pixel verde. Fix: o braço WetPaint desvia do wrapper (o
watercolor já desviava pelo mesmo motivo) e os 2 gates ganharam **asserts de sobrevivência da
sessão**. 3 gates novos (`the_selection_confines…` · `the_protection_mask_freezes…` ·
`alpha_lock_pins_the_wet_silhouette…`, este com fixture meio-transparente); 4 mutações
(gsel · gprot · α-pin · desvio de rota), cada uma sangra SÓ o próprio gate. ⚠️ Smoke junto
do lote (selecionar metade + traço wet cruzando; alpha-lock numa camada com transparência).

**W2.6 LANDOU (`5ae1b48b`) — o eraser em modo wet apaga o FLUIDO e a água sobrevive:** o wire
`"eraser"` dentro de WetPaint **FICA em WetPaint** (a exceção nova no `set_paint_tool_mode`;
sair do modo bakearia a pintura que o artista quer corrigir — semântica Rebelle) e os dabs
roteiam para `dispatch_pressure_dab_erase` (porta nova no engine: `apply_erase_shaped`, o corpo
único do `apply_erase` com silhueta + Grain — o erase veste a MESMA forma que o depósito;
lane-less: op direta de grid, cada cópia de Symmetry/Tiling apaga onde pousa; o sim pausa por
um direct stroke aberto no gesto). **Sem sessão viva não há nada molhado** ⇒ cai para o eraser
normal e apaga o BAKED (o que está visível) — e a pergunta *"o wet é dono destes dabs?"* virou
**porta única `wet_owns_the_dabs`**, perguntada pelo desvio do wrapper (W2.5) E pelo braço de
rota (duas cópias divergiriam e a metade divergida é gate de canvas que vaza ou mata a água).
O rádio do rail acende **"eraser"** (braço novo no `active_paint_mode_id`). 2 gates; **5
mutações sangram** — a que quase passou: derrubar o `merge_dirty` da porta de erase deixa o
GRID esvaziando e a **TELA parada** (o composite nunca repinta), e sobreviveu ao oráculo
só-grid ⇒ o gate ganhou o oráculo do CANVAS (`canvas_dev` estritamente decrescente).
⚠️ Decisões pro smoke do Enio: o wire mantém o modo (vetável) · a força default do erase é a
do reference (slider `erase 0.4` — knob de painel é W3) · sem sessão apaga o baked (fall-through).

**W2.7 LANDOU (`8794734b`) — o Paper do artista dirige o TOOTH do fluido:** slot Paper armado ⇒
o plano `paper` do engine (o input de tooth que o depósito e o `wet` byte já leem) é **SEMEADO
no nascimento da sessão** pela lei NEUTRA do painter (`texture::sample_tiled_rot_wrapped` +
`angle_basis` — a MESMA lei que o substrato watercolor lê, mas pela porta neutra da
`ph2d-painter-brush`: **um papel, nunca um 2º sistema, zero acoplamento a `watercolor_*`** —
a advertência do doc 19 honrada). Papel é SUBSTRATO: não re-semeia sob água viva; sessão nova
lê o slot corrente. Sem slot = preset do porte, byte-idêntico. Porta nova
`Engine::seed_paper_with` (domínio EXATO do `bake_paper`, pad ring incluso, clamp 0..1).
⚠️ Gap v1 documentado no código: o seam-snap de Size de bitmap sob sprite-Tiling
(`snap_slot_size`, watercolor-local) não é aplicado — procedurais fecham exato, bitmap pode
ter emenda. 2 gates mutation-tested (cobertura do plano + vales do Checker rejeitam pigmento
contra controle sem alinhamento). ⚠️ Smoke junto do lote (armar um Paper e ver a granulação
do fluido seguir o padrão).

**O CHECKBOX LANDOU (`0317bf2d`, ordem do Enio 2026-07-21 pré-smoke — *"se saio do brush para a
borracha ou para a seleção, ao voltar não estou mais no modo wet"*):** o arm do Wet Paint é
**estado autorado persistente** (`WetPaintState.armed` — UM fato, independente de modo; nunca
campo de `BrushSpec`, que por-slot teria cópias discordando), o padrão Watercolor/Impasto.
Seção **"Wet Paint"** no painel (acima da Watercolor), com Enable
(`PAINTER_WETPAINT_SECTION/_COLOR/_RESET/_ENABLE` em ids/chrome/`painter_wetpaint.rs`; reset =
defaults INCLUINDO o enable — a semântica do reset do Watercolor; W3 põe os ~6 knobs nesta
seção). As leis: **armado, o wire `"brush"` resolve pra WetPaint** (toda volta ao pincel é o
fluido até desmarcar) · **entrar por QUALQUER porta arma** (checkbox OFF com tinta molhada =
rádio mentiroso) · **desmarcar sai e baka** · sair pra outra ferramenta NÃO desarma (o arm é o
autorado; só o checkbox/reset desarmam) · `active_paint_mode_id` de WetPaint virou **"brush"**
(o rail acende Brush — wet é sabor do pincel; com eraser segue "eraser"). ⚠️ **Consequências da
lei nova:** o wire `"brush"` JÁ NÃO SAI do modo (2 fixtures antigos atualizados — sair de
verdade = outra ferramenta ou desmarcar) · o Deposit da lista do Impasto com wet armado é
no-op (o checkbox governa; desarme antes) · eyedropper segue Paint momentâneo (janela curta de
checkbox-ON-pincel-digital enquanto o picker está armado — gap conhecido). O smoke arma pela
**porta de produto** (`set_wetpaint_armed`) — o arm em código morreu; o item do W3 "aposentar o
arm do smoke" está FEITO. Rotas: `route_brush_wetpaint_event` no `handle_panel_event` +
populate (clicks + `mark_collapsible_section` + swatch). 5 gates (round-trips
selection/eraser/smear red-first sobre o bug + panel Click); 4 mutações sangram.

**O SMOKE DO LOTE (2026-07-21): W2.4/2.5 OK; dois reprovados, os dois FECHARAM (`8127c06d`):**
(a) *"não consigo sair do modo wet"* — **o Enable estava MORTO sob o mouse**: pintado,
registrado, e AUSENTE da **allowlist de forward** do `event.rs` do painel (o clique morria
antes do bus; sem desarmar, não há saída pelo caminho que eu mesmo indiquei). ⚠️ **Meu gate do
checkbox era SINTÉTICO** (`handle_panel_event` direto pula o forward — a cegueira exata que o
header do `seam.rs` descreve) e ficou verde sobre o defeito. Fix: `PAINTER_WETPAINT_CLICKS` na
allowlist + **gate de seam pelo `apply_event` REAL** (red-first: reinstalar o defeito o derruba
sozinho) + **gate-matriz** no tool (todo wire não-brush sai do wet, um a um — o rail sempre
soube sair; a porta morta era o checkbox). (b) *"Paper não aparece no wet"* — a seção Paper só
pintava sob watercolor ⇒ o W2.7 era **inalcançável à mão**; agora `brush.watercolor ||
brush.wetpaint` a oferece, com gate de presença E ausência (Paper oculto no brush comum —
*"deve ser assim mesmo"*, confirmado). (c) *"onde ativo a borracha?"* — resposta de produto,
não de código: é o **chip Eraser do rail** (com água viva ele levanta o FLUIDO — remoção
multiplicativa, gradual; sem sessão apaga o baked). 4 mutações novas, cada uma sangra só o seu.

**W2.8 — NÃO construído: é um FORK DE PRODUTO, decisão do Enio (2026-07-21).** Hoje em modo wet
os shape editors e os métodos não-cumulativos (DragDot/Anchored/Line) **não desenham nada** (a
rota wet recusa e os dabs morrem — nem preview flat existe). As duas saídas:
**(1) INTEGRAR** — preview FLAT pela rota de cor normal (a maquinaria dos editors intacta) e,
no Apply/pen-up, descascar o flat e depositar a lista final de dabs UMA vez pelo engine (o
fluido então escoa). Custo: o salto visual flat→fluido no Apply, e mexer na dança
`drag_preview`/`commit_drag_preview` (o commit hoje não tem "stamp final" — o último re-stamp
do preview É o commit). **(2) ESCONDER (lei #3)** — em modo wet só métodos cumulativos e mão
livre (o idioma do Rebelle, que não tem shape tools); os métodos/editors incompatíveis somem
do painel no W3, com gates de presença E ausência. A (2) dissolve o W2.8 dentro do W3 e é a
recomendação (um preview que mente sobre o resultado é pior que um método ausente); a (1) é
construível se o Enio quiser shapes no fluido.

**W2.3b — o desenho original (histórico):**
`for_each_stamp_pixel_shaped` JÁ EXISTE em `brush.rs` (sem chamador): closure
`sil(x,y)->f64` substitui falloff+footprint internos; bristle fica como fator. Falta: (a)
`Trail::accumulate_paint_shaped` (corpo único — closure nomeada passada a um dos 2 iteradores) +
`dispatch_pressure_dab_lane` ganhar a variante shaped; (b) tool constrói a closure por dab do
padrão do impasto (`impasto.rs:207-240`: `spec` com radius do dab → `dab_rotor` → `dab_footprint`
→ `shape_basis` com `ShapeFrame::Stroke` + `tex_rng` via `dab_rng.enter(&groups, di)` →
`ShapeInput{basis, image, ramp_lut}`; por pixel `t = fp.falloff_t(dx/r, dy/r)` →
`silhouette_at(&spec, shape, t, px, py, center, r)`; célula−1 = px do painter). o stamp do
engine é `for_each_stamp_pixel` (`brush.rs:169`) — `fall = radial_falloff(d², hardness)` +
footprint elíptico interno (`BrushShape::axes`) + `texv = sample_bristle(tile wrap, coords
ROTADAS)`. A composição: **`fall`+footprint cedem à silhueta do PAINTER** (`silhouette_at`:
falloff × Shape image × flatten/rotate) via variante do iterador com closure
`silhouette(dx, dy) -> f64` (0 = skip; a elipse interna vira caso default), threading por
`Trail::accumulate_paint`/tools numa variante `_shaped` — os caminhos próprios do engine ficam
intocados (fingerprint pina). `texv` (bristle) FICA como fator default estilo-Grain; W2.4 troca
a bristle pelo Grain do artista quando houver um. Perf: silhouette por pixel = o que as rotas
de cor já pagam.

**W2 restante:** só o W2.8, e ele virou FORK do Enio (ver bloco W2.8 acima) — a saída (2) o dissolve no W3.

## §2.5 — W1 (histórico do andamento; superado pelo §2.4)

- **Inc.1 COMMITADO (`c329d126`)**: `PaintMode::WetPaint` (slot 11, `PAINT_MODE_COUNT` 12) + as
  DUAS portas de wire-string (`set_paint_tool_mode("wetpaint")` / `active_paint_mode_id()`).
  Nada seleciona o modo ainda (sem UI, sem smoke) — em modo WetPaint hoje os dabs caem nas rotas
  normais de cor; a rota própria é o próximo incremento.
- **Decisão de desenho (documente no gate)**: SEM `BrushSpec::wetpaint` — o modo É o switch
  (precedente Knife; um flag por cima do modo seria 2ª porta pra mesma pergunta). O contrato
  "OFF byte-idêntico" vira: (a) arch-gate "nenhum outro modo alcança o engine wet" (irmão de
  `no_other_paint_mode_touches_the_relief`) + (b) fingerprint de pintura normal antes/depois.
- **O modelo de display é o do watercolor, generalizado** (`watercolor_render.rs:47` é o molde):
  sessão congela a base (`Arc` dos pixels pré-sessão, padrão `wet_session_base`); cada frame
  recompõe SÓ o dirty rect — para o wet paint: `render_pigment_only` do engine (RGBA straight
  sobre transparente) alpha-over a base, escrito em `canvas_rgba`. Commit da sessão derruba a
  base (dentro da transação de undo). A sessão do wet paint atravessa TRAÇOS (a água segue viva)
  — decidir o gatilho de commit: secou-completamente / troca de layer / troca de modo / Apply.
- **`Dab` do painter** (`ph2d-painter-brush/src/stroke.rs:27`): `center [f32;2]` · `radius_px` ·
  `coverage` (strength×pressão) · `color [f32;3]` (Randomize já resolvido!) · `rotation` ·
  `dir` · `arc_len` · `stroke_radius_px`. Mapeamento pro engine: center→(x,y), radius_px→r,
  coverage→intensity (o §9 do SPEC vira: pressão REAL já embutida em coverage), color→cor do
  dab (o trail do engine usa a cor por-célula — Randomize de graça).
- **A sessão mora em módulo IRMÃO novo** (`tool/paint/wetpaint.rs` + `wetpaint_session.rs` se
  crescer): `paint.rs` está a 713 linhas com teto congelado — só cabe o campo
  `wetpaint: WetPaintState` no struct (e talvez precise re-ancorar um doc-comment pra caber).
- **Rota**: em `stamp_dabs_inner` (`stamp_route.rs:250`), logo após o braço do Sculpt:
  `if matches!(self.paint.paint_mode, PaintMode::WetPaint) { self.stamp_dabs_wetpaint(dabs, &brush); return; }` —
  antes das rotas de cor. O tick: `paint_tick` (`stroke_lifecycle.rs:215`) ganha
  `self.wetpaint_tick(dt_s)` ao lado do `dry_canvas_wet` (acumulador 40 Hz, clamp 5 passos).
- **Undo §10.4**: o grid da sessão entra no `ModelSnapshot` NO MESMO commit que criar o estado
  (o bug do `mats` se escondia na tela vazia — teste onde o fato pode ser CONTRADITO).

## §3 — W2/W3 (depois)

W2 = integração total recurso a recurso com gate de seam cada (Shape/Grain/Paper/Falloff/Blend/
Randomize/Selection/alpha-lock/Symmetry/Tiling/stroke methods). Paper: os 3 presets do SPEC §4
viram fontes do slot `BrushSpec::paper` — leia `docs/Painter/19_relevo_do_papel_investigacao.md`
ANTES (a extração de substrato quer ADR; o Wet Paint é o 2º consumidor que a justifica).
W3 = seção "Wet Paint" do painel (espelho de `paint_watercolor.rs`): ~meia dúzia de knobs curados
(a tabela §16 do SPEC é a fonte; o resto vira constante nomeada), incompatíveis ESCONDIDOS com
gate de presença+ausência. O reset de grupo do painel chama `Engine::reset_knob_group` (nunca
`Tuning::reset_group` direto — ela devolve os defs mudados e o caller TEM de reagir).

## §4 — Avisos operacionais

- ⚠️ O cwd do shell RESETOU pro primário no meio da sessão sem aviso — toda mutação por caminho
  ABSOLUTO com `/Worktrees/line-Painter/` (a memória `sed_relative_path` salvou esta sessão);
  `pwd` antes de cargo/git.
- ⚠️ 5 subagentes do workflow morreram em "session limit" — verificações deles foram feitas à mão
  (advect confirmado por inspeção; oráculos de teste endurecidos). Economize subagentes.
- O commit do W0 é `75606759`; os fixes do verify `b35bb036`; ADR `f36a533a` + `40a023b3` (lock).
