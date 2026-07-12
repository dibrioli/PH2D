# HANDOFF — continuação da linha `line/anim` (Timeline)

> **Para:** o **próximo agente-de-linha** que assumir `line/anim`.
> **De:** o agente anterior (fechou W4.cauda + W5 de autoria; **integrado ao `main` em 2026-07-11**).
> **Data:** 2026-07-12 · **Regime:** Modo L (workstation).
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 (inegociáveis) →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e RELEIA a
> cada passo) → [`DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md) §0, §1.5, §2, §6.

---

## §0 — Estado da linha (você começa AQUI)

| | |
|---|---|
| **Branch** | `line/anim` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim/` |
| **Base** | **rebasada sobre o `main` atual (`3805f650`)** — 0 commits à frente, árvore limpa |
| **Tudo o que a linha fez até agora** | **JÁ ESTÁ NO `main`** (integração concluída) |
| **Sanidade da base** | ✅ `ph2d-anim` + `ph2d-timeline` + `ph2d-panel-timeline`: **333/333** · `ph2d-host-desktop`: **292/292** |

O `main` andou **208 commits** desde o fork anterior (6 linhas integradas: Painter, Vector, FLIP,
audio, motion-value, anim + ships). **A timeline sobreviveu intacta** — os testes acima rodam sobre a
base nova. Não há dívida de merge te esperando.

### O MODO L em 6 linhas (o protocolo que você segue)

1. **Trabalhe SEMPRE dentro da worktree** (`Worktrees/line-anim/`). O mesmo path relativo existe na
   raiz do repo — editar `crates/...` na raiz é editar a **árvore errada**. Na dúvida: `pwd`.
2. **Foundational você PODE e DEVE tocar** (`ph2d-anim`, `ph2d-timeline`, `ph2d-editor-core`…), com
   cuidado (ADR-0107). Ao **criar** algo foundational, projete para **isolamento**: módulo irmão novo
   em vez de engordar arquivo compartilhado; ponto de extensão **append-only**; id/const/variant novo
   = pegue o próximo livre e **anote no handoff**.
3. **PARE e reporte ao Enio SÓ em 2 casos:** (a) **contrato congelado** (CLAUDE.md §6 — `Tool`,
   `NodeOp`, `PanelEvent`, vector-doc: exige ADR); (b) **rebase conflita FORA dos seus arquivos**
   (colisão de mesmo-símbolo com outra linha). Nunca negocie com outra linha.
4. **Commits locais frequentes:** `git commit --no-verify`. **NUNCA** `push`, `--force`, `git add -A`.
5. **Fechamento** = gate batched (nextest + clippy `--all-targets` + auditoria ≥2 lentes conforme
   DIRETIVA §3–§5) + **handoff de integração escrito** → e **PARE**.
6. **Você NÃO integra e NÃO faz ship.** Isso é de um **agente integrador dedicado**, e só por **ordem
   EXPLÍCITA do Enio**. Integrar/pushar por conta própria = violação do protocolo.

**Inner loop:** só `cargo check -p <crate>`. Teste/clippy/auditoria **1× no fechamento**, nunca por task.

---

## §1 — O que já está pronto (não reimplemente)

A Timeline está **completa como ferramenta de autoria**. Tudo abaixo está no `main`, testado e
**smokado pelo Enio**. O detalhe técnico (o "porquê", provas, gotchas) está em
[`HANDOFF_line_anim_integracao_2026-07-11.md`](HANDOFF_line_anim_integracao_2026-07-11.md) §1–§17 — **leia a
seção correspondente antes de mexer numa dessas áreas**.

- **Transporte + dope-sheet + graph editor** (W2/W3): régua, faixas, zoom/pan, box-select, copy/paste,
  curva por-faixa, handles bézier, presets de easing, canal Summary.
- **Speed graph** (§8): 2ª vista plotando velocidade; derivada **analítica** (`Interp::slope`, port do
  Chromium `cubic_bezier.cc`); handles de velocidade AE-style com braço de influência.
- **Weighted tangents** (§10): `Interp::BezierW{x1,dy1,x2,dy2}` — dy é offset **absoluto** em valor
  (semântica AE/Blender), então **segmento flat pode curvar**.
- **Time remap** (§11/§13): `PropKind::TimeRemap = 6` — relógio por objeto (playhead→tempo-fonte).
  **Lição que custou 3 tentativas:** *toda coordenada derivada tem que ser semeada pela MESMA função
  que a lê* ([[feedback_derived_coordinate_seed_must_match_sample]]).
- **Roving keys** (§12): tempo derivado para velocidade constante; `DOC_VERSION` 1→2.
- **Auto-key + pin de pose deslocada** (§7/§9/§15): auto-key **inerte no play**; o diff compara no
  **relógio CRU do apply** (não no snapado) — senão pause fora de fronteira de frame minta key.
- **Performing / Record** (§16): grava a pose ao vivo durante o play. **O guard é sagrado:**
  `capturing = if playing { performing && drag_now } else { armed }` — no play só grava com **gesto
  ativo**; a pose passiva da animação **nunca** minta key. Há testes que provam isso; **não os afrouxe**.
- **Simplificação do record** (§17): o record grava denso e, no release, cada track vira uma curva
  Bézier limpa. Pipeline: **low-pass** (`smooth_values`) → **detecção de extremos** (`anchor_indices`)
  → **1 cúbica por trecho entre picos/vales** (`fit_run`) → **colunas alinhadas** entre todas as tracks
  da entidade (`fit_fcurve_at`). Fidelidade é **aproximada por design** (~1-3%) — o Enio escolheu
  poucos keys nos extremos > precisão sub-percentual.
- **Delete Track** por R-click na label (§14).

---

## §2 — A FILA (etapas planejadas, em ordem de prioridade)

> O Enio decide a ordem final. Esta é a recomendação, com o racional. **Confirme com ele antes de
> começar** se for pegar algo fora do topo.

### ETAPA 1 (recomendada) — **W4.T7: relógio único** `MotionTransport` ← `Playhead`
**Por quê primeiro:** é a última dívida ARQUITETURAL da timeline. Hoje existem **dois transportes**
(o `Playhead` da timeline e o transporte próprio do Motion em `motion_bridge.rs`) — dois relógios que
podem divergir. Enquanto isso existir, qualquer feature que cruze Motion×Timeline (incl. a T4 abaixo)
é construída sobre areia.
- **Onde:** `shells/desktop/src/render_loop/motion_bridge.rs` (transporte duplicado) + `ph2d-core::Playhead`.
- **Alvo:** o Motion deriva o tempo do `Playhead` (fonte única); remover o transporte paralelo.
- **⚠️ Coordena com a linha Motion.** O `main` trouxe mudanças no Motion (`81a2f888` splitou
  `motion_bridge_tests`; há um plano novo de "motor de nós GPU-resident"). **Antes de começar,
  pergunte ao Enio se a linha Motion está aberta** — se estiver, o risco de colisão em
  `motion_bridge.rs` é real e ele decide a ordem.
- **DoD:** um só relógio; play/scrub na timeline dirige o Motion; teste que prova que os dois não
  divergem mais.

### ETAPA 2 — **W4.T4: docar a timeline no `motion_timeline_slot`**
Quando o split do Motion está ativo, a timeline deve ocupar o slot dedicado (o `Rect` **já existe**:
`crates/ph2d-editor-core/src/screens/layout.rs:159`).
- **Depende da T7** (fazer antes o relógio único evita retrabalho).
- **Coordena com Motion** — mesmo aviso da T7.
- Leia [[feedback_docked_panel_registration_four_sites]] (painel docado = 4 sites de registro).

### ETAPA 3 — **NLA / multi-clip (UI)**
O **dado já existe**: `TimelineDoc.clips: Vec<NamedClip>` + `active_clip()`
(`crates/ph2d-timeline/src/doc.rs:54,88,131`) — só o clip ativo é exposto. Falta a **UI**: criar,
nomear, trocar de clip; depois empilhar (NLA de verdade).
- **100% isolado** (nenhuma coordenação com outra linha) — é o candidato mais seguro se as linhas
  Motion estiverem abertas.
- Comece pelo **seletor de clip** (o mínimo útil), não pelo NLA completo.

### ETAPA 4 — **Markers → signals**
Markers existem (`Marker` em `doc.rs:30`, `+M` na barra, rename). Falta emitirem **sinal** quando o
playhead cruza (o gancho para gameplay/áudio/eventos).
- Isolado. Pequeno. Bom candidato para uma jornada curta.

### ETAPA 5 — Refinamentos do **fit** (§17, deferidos por escolha, não por preguiça)
Cada um está documentado no topo de `crates/ph2d-anim/src/curve_fit.rs`:
- **Corner pre-pass:** cusps genuínos viram tangentes **BROKEN** (hoje o fit os suaviza sob uma
  tangente central compartilhada).
- **Value-overshoot clamp** para canais limitados (a cúbica pode ultrapassar o range; opacity já é
  clampada no runtime, então é cosmético no graph).
- **Rotation unwrap** antes do fit (spins multi-volta: hoje um wrap em ±π viraria um salto que o fit
  tentaria preservar).
- **Só pegue isso se o Enio reclamar do resultado do record** — o fit atual foi aprovado ("ficou bom").

### ETAPA 6 — W4.T6/B5 (save cena+timeline unificado) · bake · export · MCP/Luau
**Deferidos.** O T6/B5 é cross-cutting (precisa de id estável de entity) — **não landar solo**;
coordene com o dono da persistência.

---

## §3 — Armadilhas desta área (o que já mordeu)

1. **Verde-de-compilação vale ZERO no audit.** O Time remap foi declarado "consertado" **duas vezes**
   com suíte verde e continuava quebrado no app. **Valide NO APP RODANDO** antes de dizer pronto:
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop`
2. **Coordenada derivada: seed = sample.** Se você escreve um valor que depois é LIDO por outra
   transform (tempo remapeado, tempo-fonte, coluna alinhada), use **a MESMA função** nos dois lados.
   Foi a causa-raiz de 3 bugs seguidos.
3. **O guard do Record é sagrado** (§16). Se você mexer no `autokey_pass`, rode
   `a_plain_play_with_autokey_armed_records_nothing` e `performing_without_a_drag_records_nothing` —
   eles existem porque o Enio foi mordido por "autoplay criando keyframes".
4. **Menu de contexto que "não faz nada":** o Down que precede o Click **já fechou** o menu — leia
   `context_menu().or_else(last_context_menu())` ([[feedback_context_menu_closes_on_down_repaint]]).
5. **LOC caps:** shell/painéis = **600**, workspace = **700**. `fmt` re-expande — formate ANTES de
   medir. Estouro = **split em módulo irmão**, nunca allowlist.
6. **Zero string hardcoded / zero hex / zero f32 de UI** (HR-15): tudo por tokens/i18n. Labels de app
   são em **inglês** ([[feedback_app_ui_english_only]]).
7. **Nada de `→` em string literal** (gate `no_tofu_glyphs`) — **inclusive em `assert!`** (me pegou no
   fechamento). Use `->` ou `·`.

---

## §4 — Ao fechar a jornada (o que o Enio espera)

1. Gate batched: `cargo nextest run -p <crates tocadas>` + `cargo clippy --all-targets -- -D warnings`
   + `rustup run 1.95 cargo fmt` + LOC caps + auditoria ≥2 lentes (template da DIRETIVA §3).
2. **Handoff de integração** (DIRETRIZ §1.5.9): branch/HEAD/base · foundational tocado + por quê ·
   **ids/consts/variants novos com valores** (colisão!) · contratos congelados encostados (deve ser
   nenhum) · o que só o `ship.sh` pega · o que smoke-testar.
   Use [`HANDOFF_INTEGRACAO_line_anim_2026-07-11.md`](HANDOFF_INTEGRACAO_line_anim_2026-07-11.md) como **modelo** — funcionou.
3. Reporte **"linha pronta + handoff"** e **PARE**.
