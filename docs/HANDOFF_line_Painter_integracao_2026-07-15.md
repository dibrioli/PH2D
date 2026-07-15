# HANDOFF de INTEGRAÇÃO — `line/Painter` (2026-07-15)

> **Para o agente INTEGRADOR** (DIRETRIZ §1.5.9). A linha está FECHADA e **parada**: integração e ship
> só por ordem explícita do Enio. Jornada de hoje: **P0 (retângulo residual do Inflate) + W4 (família
> advectiva do Deform)**.

## 1. Base e commits

- **Base:** `main` = `12ccaecd` (a linha foi REBASEADA sobre ela hoje — os 5 commits docs/memory do main
  já estão embaixo; fast-forward possível se o main não andou).
- **Commits da jornada** (sobre os 10 pré-existentes da linha, que o main ainda não tem):

| | |
|---|---|
| `8b21acb8` | **fix(sculpt): P0** — o retângulo residual do Inflate era a dilatação AMBIENTE do terreno; 3 camadas seguram o suporte |
| `cac2db77` | **feat(deform): W4** — o warp carrega o CORPO da tinta (`h`+`covers`+`mats`) junto dos pixels |
| (este) | docs: handoffs + CLAUDE.md |

## 2. Superfície tocada

- **`ph2d-tool-painter`** (módulo da linha): `sculpt_blur.rs` (P0) · `warp/{mod,apply,reconstruct}.rs` +
  **novos** `warp/relief.rs`, `warp/relief_tests.rs` (W4) · `undo.rs` (**`DeformSnap` substitui a tupla**
  `deform_disp/pre/active` do `ModelSnapshot` — mesmos dados + os planos congelados do relevo) ·
  `layers/undo.rs` (2 call-sites) · `brush_settings.rs` + `snapshot.rs` (2 campos novos no snapshot) ·
  **novo** `sculpt_tests/inflate_support.rs`.
- **`ph2d-panel-painter-layers`**: `paint_deform.rs` (row nova) · `populate.rs` (1 registro) ·
  `event_brush_forward.rs` (1 forward) · `brush_fallback.rs` (2 campos) · **novo** `tests/seam_deform.rs`.
- **`ph2d-editor-core`** (foundational, append-only): **id novo `PAINTER_DEFORM_RELIEF`** =
  `hash_node_id("painter_deform.relief")` em `ids/chrome/painter_deform.rs`. Hash de string — sem
  contador compartilhado; colisão só se outra linha criar o MESMO nome (grep antes de fundir).
- **Contratos congelados: intactos** (`Tool`/`NodeOp`/`NodeManifest`/`CanvasPaintTool` não tocados).
  Nenhum schema bumpado. Nenhum arquivo de outra linha tocado.

## 3. Gates no fechamento (medidos, não lembrados)

- `cargo test -p ph2d-tool-painter --lib` → **687 passed / 0 failed** (23 ignored = GPU/perf).
- `cargo test -p ph2d-panel-painter-layers` → **40 lib + 22 integração** (inclui `seam_deform`).
- `cargo test --workspace` → rodado no fechamento (resultado no relatório da linha).
- clippy `--all-targets` nas 3 crates → **0 warnings**.
- Perf (`--release --ignored`): **Inflate 3,36 ms/move @2048² · 3,73 @4096²** (era 4,57; kill 8) —
  o P0 DEIXOU O INFLATE MAIS RÁPIDO (janela de escrita menor + rim-resample com sqrt morreu).
  Impasto 4,22/4,10 · Smooth 3,47/3,73 · Scrape 2,83/3,20 — tudo dentro.
- **Placar de mutação da jornada: 11 provadas + 1 born-red** (P0: sentinela/budget/taper + gate-repro
  nascido vermelho com 11.830 texels; W4: 8/8).

## 4. O que landou (uma linha cada)

**P0 — o retângulo residual do Inflate (4º smoke).** O mecanismo era a *dilatação ambiente*: fonte
NÃO-tocada entrava no envelope enraizada no próprio chão (`g = pre`) e dilatava os vizinhos morro-abaixo
dentro do cap circular — em toda a janela `kr`, cuja borda é um retângulo. Fix em 3 camadas independentes
em `render_inflate` (cada uma com gate + mutação): **sentinela** (não-tocado não compete; sem ela uma
parede alta não-tocada SOMBREIA a fonte legítima) · **orçamento por-fonte** (`reach² = 2ρ²·amount` — a
bola de cada fonte termina onde o pico dela se esgota) · **taper²** (do equador ao alcance o lift cai a
zero com gradiente zero — C¹; sem ele a borda do suporte é uma parede de `m·R`). + piso-próprio (vencedor
desqualificado não apaga a bola do próprio texel — o gate da erosão pegou ao vivo) + blur mascarado (o
Smooth borra o campo absoluto SÓ onde a bola agiu; `m=0` exato preserva `pre` ao bit) + janelas honestas
(`kr = rect ⊕ (⌈ρ√2⌉+smooth)`, `cr = kr ⊕ o mesmo`). **Fora do suporte a lei é BYTE-IDENTIDADE.**

**W4 — a família advectiva.** O warp do Deform (Push/Twist/Pinch/Wrinkle/Fold + Reconstruct) carrega os
3 planos do impasto pelo MESMO `disp` da sessão, na porta única `warp_render_relief` (chamada pelos dois
renders — corpo e cor não podem divergir). Sessão congela os planos junto do `pre` (sempre; toggle só
gateia a ADVECÇÃO); Reset devolve tudo; Apply&Keep rebasa tudo; `DeformSnap` carrega os baselines pelo
undo (a lição do `deform_disp`). Toggle **Affect Relief** default ON, pintado só quando a camada tem
relevo, costurado nos 7 sites com seam test que CLICA.

## 5. As armadilhas (pagas hoje; não re-pague)

1. **O Up carimba dabs de CAUDA** (`paint_end` → `stroke.finish` → `stamp_dabs`) e mata a sessão — gates
   que medem o suporte capturam `amount` E `heights` **antes do pen-up**. Um gate meu mediu pós-Up e o
   "anel" era a secante legítima fantasiada de bug (2,87 loads de susto).
2. **Defesas em camadas não sangram uma por vez.** O gate-repro do P0 só sangra com as DUAS camadas
   removidas (sentinela + cap) — cada camada tem gate PRÓPRIO. Se uma mutação não sangra, ou o comentário
   mente, ou falta o gate da camada — as duas aconteceram hoje e viraram gates.
3. **Fixture de falloff Smooth não exercita o taper** (amount→0 na borda ⇒ o orçamento por-fonte já
   estrangula); o gate do taper exige **Constant**. Escrito no próprio gate.
4. **O secante em flanco íngreme é GRANDE e está CERTO** (lift = Depth·√(1+G²), pinado pelo gate da
   rampa): num flanco de 0,57 load/px são ~10 loads. Oráculo de anel com barra absoluta pequena está
   errado; o que se gateia é o degrau na FRONTEIRA (último texel escrito), não o meio da fade.
5. **`Checkbox` no populate emite `Toggled` e morre** — registrar como Button (o W4 seguiu; escrito no
   populate).

## 6. Aberto (a fila que sobra)

1. **P1 — smoke do Enio** (bloqueia declarar o Sculpt smokado): roteiro no handoff de continuação.
   P0 e W4 estão **pendentes de smoke**.
2. **W5 — Conserve (bow wave)**: o kernel já computa o volume deslocado (`sculpt_displaced_volume`,
   gateado); é um flag + o DESENHO da pilha na borda — e o desenho é exatamente onde o Push falhou:
   **começar renderizando e olhando** no app vivo, não pela teoria. Plano §6.
3. **D — bugs de display** (relevo anchored some no pen-up; jitter estica): provados corretos na tool;
   vivem no pipeline GPU de preview do shell. Precisam do app vivo.
4. **A TINTA EMPURRADA (Push)**: fim da fila, ordem do Enio.
5. **Perf do Deform NÃO é gateada** (nunca foi): o W4 adiciona 3 amostragens/texel no bbox do dab quando
   a camada tem relevo + toggle ON. Nenhum critério de kill existia para o warp; se o smoke sentir peso,
   medir antes de otimizar.

## 7. Como rodar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
