# Travadas do app (mouse-stutter / pan / animação) — diagnóstico & correção

Registro de como as travadas de renderização do PH2D foram diagnosticadas
(empiricamente, via `PH2D_PROF`) e corrigidas. Estrutura: **sintoma →
o que já tinha sido feito → diagnóstico medido → causa raiz → correção →
trade-off → como evitar regressão**. Se a travada voltar, comece aqui.

---

## Sintoma

Trancos ocasionais ao mover o mouse, **piores varrendo o painel
Hierarchy**. Intermitente, não constante. Depois (com a 1ª correção de
present mode) surgiram dois sintomas relacionados: **pan dando saltos** e
**animação dos sprites de teste não perfeitamente fluida**.

## O que já tinha sido feito (fix #1, anterior)

Cache de shaped-layout em `crates/ph2d-text/src/system.rs`
(`layout_cache`, clone-on-hit) — eliminou o re-shaping de texto por-frame.
**Ajudou mas não zerou** o stutter. Hit-rate medido depois: 96.8%
(funcionando). A hipótese de "clear-spike" do cache foi **refutada** com
dados (12 wholesale-clears observados, nenhum gerou frame gap).

## Diagnóstico medido (PH2D_PROF, instrumentação revertida)

Instrumentou-se com timers `Instant`/`elapsed` env-gated (`PH2D_PROF`),
logando só hitches (>4ms), em 3 fases de CPU + o `acquire_frame`:

| Fase | Hitches durante a varredura |
|---|---|
| `snapshots::publish` (snapshot hierarquia) | **0** (nunca) |
| `paint_hero_screen` (shaping + encode) | **0** (a única espiga foi cold-start) |
| `render_to_intermediate` (encode Vello) | só 2, no startup |
| **`acquire_frame()` (present/vsync)** | **14 gaps**, os severos 30–94ms |

**Conclusão: a travada NÃO é CPU.** Os frame gaps severos são 1:1
explicados por bloqueio do `acquire_frame()` (present), sem espiga de CPU
concorrente.

## Causa raiz

`surface.acquire_frame()` bloqueia até o próximo texture do swapchain.
Sob `PresentMode::Fifo` (vsync), com a fila de present **saturada**, esse
bloqueio empilha por vários intervalos de vsync → tranco. A fila satura
porque **o app renderiza continuamente**:

- `shells/desktop/src/render_loop/present.rs` chama `request_redraw()`
  **todo frame** (loop contínuo sob `ControlFlow::Poll`).
- E `shells/desktop/src/sim_populate.rs` spawna sprites de demo com
  `Velocity` (a "bouncing-motion" M5); `sim_extract::run` move toda
  entidade com Velocity **todo frame** → a cena anima sempre.

Pior no **Hierarchy** porque a cena dele é a maior (mais widgets/texto) →
rasterização GPU por-frame mais perto de saturar o pipeline.

## Por que "render event-driven" NÃO resolveu (e ficou adiado)

A ideia óbvia — `ControlFlow::Wait` + `request_redraw` só em input/
animação — **não ajuda aqui**: com os bouncers do demo animando a cada
frame, `is_animating()` seria sempre `true` → redraw contínuo → o
`acquire` continua empilhando. Event-driven só rende quando a cena pode
ficar **estática** (sem demo bouncers / sim pausável no modo edição), e aí
é ganho de **idle-CPU**, não fix do stutter. **Adiado** como marco futuro
(precisa de play/pause do sim primeiro).

## Correção (commits 2026-05-21)

1. **Present mode não-bloqueante** (`crates/ph2d-gpu/src/surface.rs`):
   `acquire_frame` deixa de bloquear sob `Immediate` (o Metal não expõe
   `Mailbox`). Boot loga `[ph2d-gpu] surface present mode: …`.
2. **Toggle em runtime** `Config → Display → {VSync | Immediate}`
   (`SurfaceContext::set_present_mode`, `chrome/settings_present.rs`,
   `EditorAction::SetPresentMode`). Mata o stutter (Immediate) OU dá
   motion perfeitamente fluido (VSync).
3. **Bounce frame-rate-independent** (`render_loop/mod.rs`): o demo
   integrava com o `fixed_dt` uma vez por frame → a centenas de fps
   (Immediate descapa o loop) os sprites corriam + jitter. Trocado por
   `wall_dt` clampado (real-time, suave em qualquer fps).
4. **Default = VSync (`Fifo`)** por escolha do Enio (motion suave por
   padrão); Immediate é opt-in pra quem prioriza zero-stutter.

## Trade-off (importante)

Neste backend Metal **não há `Mailbox`** (não-bloqueante **e** sem
tearing). Então é escolha do usuário, exposta no Config → Display:

| Modo | Stutter do mouse | Fluidez pan/animação | CPU idle |
|---|---|---|---|
| **VSync (`Fifo`)** — default | volta (acquire bloqueia sob carga) | **perfeita** (paced por hardware) | normal |
| **Immediate** — opt-in | **zerado** (não-bloqueante) | imperfeita (sem vsync = judder) | alta (uncapped) |

Um frame-limiter NÃO resolveria a fluidez do Immediate: sem vsync, mesmo
capando a 60fps os frames não ficam phase-locked com o refresh → judder.
VSync é a única forma de motion perfeitamente fluido aqui.

## Como evitar regressão / onde está o código

- Present mode default + seleção: `crates/ph2d-gpu/src/surface.rs`
  (`SurfaceContext::new` + `set_present_mode`).
- `request_redraw` contínuo: `shells/desktop/src/render_loop/present.rs`
  (comentário "IF MOUSE STUTTER RETURNS").
- Bounce do demo: `shells/desktop/src/render_loop/mod.rs` (usa `wall_dt`,
  NÃO `fixed_dt` — se voltar a `fixed_dt`, a animação re-escala com o fps
  sob Immediate).
- Toggle: `chrome/settings_present.rs` + `EditorAction::SetPresentMode` +
  drain em `render_loop/mod.rs`.

**Se a travada voltar:** re-instrumente com `PH2D_PROF` (timer de
wall-delta do frame inteiro + split do bloco de `acquire`) — o bloco do
acquire é o sinal, não as fases de CPU. Confirme o present mode ativo no
log de boot (se caiu pra `Fifo` num backend sem Immediate, o stutter
volta).

## Marco futuro (idle-CPU)

Render event-driven (`ControlFlow::Wait`) + pausar o sim no modo edição →
edição fica estática → zero render ocioso (mata os ~100% CPU idle) + sem
stutter sob vsync (não renderiza quando nada muda). Não é fix do stutter
(é economia de energia); só rende depois que a cena pode ficar parada.
