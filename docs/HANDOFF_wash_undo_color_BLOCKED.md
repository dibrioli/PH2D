# HANDOFF (BLOQUEADO) — Wash: undo quebrado + cores erradas

> **Status: 2 bugs CRÍTICOS não resolvidos.** O agente anterior tentou ~4 rodadas de fix em cada um
> (commits abaixo) e o Enio confirmou que **NENHUM funcionou** ("vc não corrigiu nem um nem outro").
> Pare de remendar incrementalmente — releia o §"Causa-raiz suspeita" e o §"Recomendação" antes de
> tocar em código. Provavelmente o caminho certo é **reconsiderar a arquitetura** (ADR-0088), não
> mais um patch.

Contexto base: [`HANDOFF_wash.md`](HANDOFF_wash.md) + [`ADR-0088`](architecture/decisions/0088-wash-persistent-pigment-canvas-and-undo.md)
+ [`ADR-0086`](architecture/decisions/0086-watercolor-minimal-core-wash.md)/[`0087`](architecture/decisions/0087-wash-integration-parallel-watercolor-mode.md).
Postmortem dos bugs já resolvidos (B1–B6, bordas/checkerboard): [`Painter_projeto/wash_solucao_de_erros.md`](Painter_projeto/wash_solucao_de_erros.md).

Build/run: `cargo run -p ph2d-host-desktop --features wash` · liga **Wash** + **Pigment** no Brush Studio.
Slot CoW: `CARGO_TARGET_DIR=$PWD/target-slots/slot-1 cargo ...`.
Testes GPU: `cargo test -p ph2d-painter-wash --features gpu --test wash_invariants -- --ignored --nocapture`.

---

## 1. Os dois bugs (relato do Enio, verbatim)

### BUG-U — Undo/redo não funciona, "estado antigo insiste em voltar"
> "UNdo não funciona direito. Salva estado antigo que insiste em voltar. BUg muito crítico. Impede
> qualquer trabalho." + "undo/redo estão bugados de Evaporation = 0".

Sintoma: pinta wash, dá Ctrl+Z → o conteúdo não some / volta sozinho. Pior em Evaporation 0. Bloqueia
o trabalho. **Tentado 3×, ainda quebrado.**

### BUG-C — Cor pintada ≠ cor selecionada
> "as cores selecionadas não são as cores pintadas... onde está laranja eu pintei com o vermelho" →
> depois: "Com pigment vermelho fica laranja, sem pigment vermelho fica amarelo." (seletor em R=1,G=0,B=0)

Sintoma: vermelho puro do color-picker pinta **laranja** (modo Pigment/K-M) ou **amarelo** (modo
Linear). **Tentado 2×, ainda errado.** A FÍSICA e as misturas (azul+amarelo=verde) o Enio aprovou —
só a FIDELIDADE de cor de uma cor única é que está errada.

---

## 2. Arquitetura atual (o que foi construído — e provavelmente é a causa)

Decisão do Enio (ADR-0088): **canvas de pigmento PERSISTENTE** para permitir "transformar a obra
inteira ao vivo" ao trocar Linear↔K-M. Consequências de design que criaram os bugs:

- **Campo = SEMPRE concentrações de 4 pigmentos** (CMY+K), `vec4` por célula. Os dois modos de cor
  leem o MESMO campo (`composite.wgsl`: `linear_compose` aditivo vs `km_compose` espectral). Isso foi
  necessário pro toggle ao vivo, MAS:
  - perde fidelidade de cor (gamut de 4 pigmentos + o unmix RGB→concentrações distorce). **→ BUG-C**
- **Bridge `painter_wash_bridge.rs`** mantém uma sessão GPU persistente (`WASH_SESSION` thread_local):
  campo acumula entre traços; base backdrop capturada 1×; bake assíncrono no `canvas_rgba` no settle.
- **Undo** (ADR-0088): o tool conta `wash_active_strokes` (com flags por entrada do undo stack); o
  bridge guarda snapshots do campo por traço (`committed[i]`) e re-sincroniza o campo GPU ao count do
  tool. **→ BUG-U** (o bake assíncrono + estado dividido tool/bridge + o modelo de snapshot do painter
  brigam entre si).

**O conflito de fundo:** o undo do painter é **snapshot síncrono de `canvas_rgba`** tirado no fim do
traço (`UndoController::record_pre_stroke`, chamado em `end_stroke`, lifecycle.rs:~1119). Mas o wash
pinta o `canvas_rgba` **30 frames DEPOIS** (bake no settle, no bridge). E o "estado real" do wash vive
num campo GPU separado, não no `canvas_rgba`. Os dois modelos são estruturalmente incompatíveis.

---

## 3. O que JÁ foi tentado (NÃO repetir — não funcionou)

| Tentativa | Commit | Resultado |
|---|---|---|
| Undo via contagem `wash_active_strokes` + snapshots no settle | `60250016` | falhou (colapso de traços rápidos) |
| Cor: refinar unmix em espaço de cor + cap duro no composite | `9535f97e` | falhou (matiz ainda desloca) |
| Cor: mover cap pro solver (`PIG_CAP`), composite lê direto | `b2fe43d9` | falhou (Enio: ainda laranja/amarelo) |
| Undo: snapshot no pen-up + sessão persistente + reset-gen | `afb6a979` | falhou (Enio: "nem um nem outro") |
| Evap-0 undo: restore com zero substeps | `9535f97e` | insuficiente |

**Conclusão:** patches no bridge/composite não estão resolvendo. O agente anterior NÃO conseguiu
reproduzir interativamente (sem GUI headless) — todos os testes GPU passam (10/10) mas o **comportamento
no app está quebrado**. Isso é o padrão clássico "[unit-verde ≠ funciona no produto]" — só audit e2e /
o Enio pega. **Você precisa de instrumentação real no app, ou repensar a arquitetura.**

---

## 4. Causa-raiz suspeita (melhor diagnóstico até agora)

### BUG-U (undo)
Hipóteses ordenadas por probabilidade:
1. **Bake assíncrono quebra o snapshot do undo.** No `end_stroke`, `pending_pre_stroke` = `canvas_rgba`
   ANTES do traço — mas o wash ainda não baked, então o pre-image e o post-image do undo controller
   não correspondem ao que o wash realmente pintou. O `undo_last_stroke` (lifecycle.rs:1603) restaura
   `canvas_rgba` E dropa `wet_field`, mas o **campo GPU persistente do bridge não é tocado** → re-bake
   → "volta". O sync por `wash_active_strokes` deveria cobrir, mas talvez o count não esteja batendo.
2. **`wash_active_strokes` não incrementa pro traço wash** — VERIFICAR: o traço wash gera `samples`
   não-vazios? Se `samples.is_empty()` (lifecycle.rs:1018) o `end_stroke` retorna ANTES de gravar o
   undo entry → count nunca sobe → bridge nunca sincroniza → campo persiste → "volta". **Cheque isto
   PRIMEIRO** (instrumente `eprintln!` em end_stroke: samples.len() + wash_enabled + wash_active_strokes).
3. **Override sempre ativo** mascara o `canvas_rgba` restaurado: se o bridge devolve `PreviewOverride`
   (slot em cache) mesmo após undo, o usuário vê o slot (estado antigo), não o canvas restaurado.

### BUG-C (cor)
- O unmix RGB→4-pigmentos (`km.rs::rgb_to_concentrations`) + o composite espectral têm **fidelidade
  ruim** pro gamut e, pior, **a normalização/escala da quantidade desloca a matiz** (escalar um espectro
  = `T^s`, muda a forma → vermelho vira laranja/amarelo). O agente tentou cap-no-solver pra evitar
  escala no composite, mas o Enio diz que continua errado → ou o campo ainda satura e escala, ou o
  unmix em si já dá laranja, ou o `linear_compose` (aditivo) dá amarelo por outro motivo.
- **Cheque empírico que falta:** logar, no bridge, a `d.color` recebida vs `rgb_to_concentrations(d.color)`
  vs `compose_over(white, conc)` — ver onde a matiz se perde (no unmix? no cap? no deposit-mass?).

---

## 5. Recomendação (forte)

O agente anterior PERGUNTOU ao Enio se devia voltar ao modelo por-traço (undo robusto + cor fiel,
perdendo o transform-ao-vivo-de-tudo); o Enio escolheu **insistir no canvas persistente**. Mas após
mais 2 rodadas falhas, reconsidere honestamente com ele:

**Opção A — Instrumentar e consertar no lugar (1 dia de debug e2e).** Pré-requisito: rodar o app e
logar (não dá pra resolver no escuro). Ordem: (1) confirmar se o traço wash cria undo entry + incrementa
`wash_active_strokes` (BUG-U hip.2); (2) logar a cadeia de cor (BUG-C). Só então corrigir.

**Opção B — Reverter pro modelo por-traço (recomendado se o Enio aceitar perder o live-transform).**
Cada traço wash assa no `canvas_rgba` **sincronamente no pen-up** e entra no undo padrão (síncrono) →
BUG-U some. Modo RGB volta a guardar a cor DIRETO (sem unmix→concentrações) → BUG-C some (fiel). K-M
fica como modo separado por-traço (vibrante, gamut-limitado), sem transform retroativo. É a arquitetura
robusta; os dois bugs são efeitos colaterais do canvas-persistente que ela elimina. Reverter ≈ os
commits `f7fd7279`+`60250016`+`9535f97e`+`b2fe43d9`+`afb6a979`; manter `8036a53b`/`7b4e60de` (K-M core)
mas voltar o bridge ao bake-por-traço (`aa1ee50f`..`97ea380c` era a base estável de física/bordas).

**Decisão é do Enio** — apresente A vs B com custo/risco antes de codar.

---

## 6. Mapa de arquivos

| Arquivo | Papel |
|---|---|
| `crates/ph2d-painter-wash/src/km.rs` | núcleo K-M: unmix RGB→4 conc (`rgb_to_concentrations`), `compose_over`. **Cor: começar aqui.** |
| `crates/ph2d-painter-wash/src/shader/{splat,wash,composite}.wgsl` | splat (deposita+clampa `PIG_CAP`), step (física+clampa), composite (Linear aditivo \| K-M espectral) |
| `crates/ph2d-painter-wash/src/solver.rs` | `WashSolver`: `upload_pigment` (restore undo), `read_pigment` (snapshot), `pig_buffer` |
| `shells/desktop/src/render_loop/painter_wash_bridge.rs` | **o coração.** Sessão persistente, bake assíncrono, restore de undo (`want` vs `applied`, `committed[]`), snapshot no pen-up. **Undo: começar aqui.** |
| `crates/ph2d-tool-painter/src/tool/lifecycle.rs` | `end_stroke` (grava undo + `wash_active_strokes`), `undo_last_stroke`/`redo_last_stroke`, `wash_active_strokes()`, `wash_reset_generation()` |
| `crates/ph2d-tool-painter/src/tool/mod.rs` | campos `wash_active_strokes`/`wash_undo_flags`/`wash_redo_flags`/`wash_reset_generation` |
| `crates/ph2d-tool-painter/src/undo.rs` | `UndoController` (snapshot de `canvas_rgba` por traço) — o modelo que briga com o async-bake |

Profiler do bridge: `PH2D_WASH_PROFILE=1` (cpu/gpu/seed/dirty/err a cada 120 frames).

## 7. Invariantes a NÃO quebrar
- Física aprovada (bloom/edge/wet-on-dry) — não mexer no transporte (`cs_step`) sem motivo; ver
  `wash_solucao_de_erros.md` §0 (borda tem ≥3 causas; CFL combinada; evap-0 é o pior caso).
- Modo RGB **não substitui**, é OPÇÃO (Enio, 3×). Os dois modos coexistem.
- Gate de contrato `architecture_painter_contract_surface` (RenderingParams cap 14 — NÃO adicionar
  campo; o seletor de cor reusa `PigmentMode`/toggle "Pigment").
- Sem push/CI sem o Enio pedir; commits locais; `git add -- <paths>` (nunca `-A`).
