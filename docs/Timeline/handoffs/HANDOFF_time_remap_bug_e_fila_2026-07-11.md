# HANDOFF — Time remap ainda quebra a animação · fila W5 · protocolo de integração (linha `line/anim`)

> **✅ RESOLVIDO (2026-07-11, commit `72803d18`):** o fix do §3 foi aplicado, com o repro do §2
> vermelho→verde no caminho real do shell + varredura do §4.2 completa + gate batched verde.
> Prova e detalhes no **[handoff de integração §13](HANDOFF_line_anim_integracao_2026-07-11.md)**.
> **Pendente só o smoke do Enio no app** (§4.1 / integração §13.3) — o DoD. O restante deste
> arquivo fica como registro da caçada; a **fila do §5** (itens 2–5) segue aberta.

> **Para:** o próximo agente que assumir a **linha `line/anim`** (Timeline) em **Modo L**.
> **De:** agente anterior (fechou time remap v1, o fix "§11.1" que NÃO resolveu, e roving keys).
> **Data:** 2026-07-11. **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim/` · **branch:** `line/anim`.
> Leia primeiro o **CLAUDE.md §0** (inegociáveis) e o **[handoff de integração](HANDOFF_line_anim_integracao_2026-07-11.md)** (estado completo da linha, §1–§12).

---

## §0 — Estado da linha (não perca isto)

- **17 commits** sobre a base `1c7c9a22` (== `main` HEAD no fork) → **fast-forward puro**, árvore limpa.
- Ordem relevante do topo: `83f4867b` (docs roving) · `5b8b6e7f` (**roving keys W5**) · `d0fcbf0c` (docs) · `56217b44` (**fix "Time bugado" — INSUFICIENTE, ver §2**) · `9a6ba0ee` (**time remap W5**).
- **Modo L, protocolo (CLAUDE.md §0.2/§0.7):** você **NUNCA integra nem faz push**. Fecha a linha, escreve o handoff de integração e **PARA**. Integração/ship só por **ordem EXPLÍCITA do Enio**, via **agente integrador dedicado**. Ver §6.
- **Regra-mãe desta caçada (DIRETIVA_IMPLEMENTACAO):** **verde-de-compilação/unit vale ZERO no audit.** O time remap já foi declarado "consertado" DUAS vezes (`9a6ba0ee`, `56217b44`) com suíte verde — e continua quebrado no app. **Não repita: valide NO APP RODANDO antes de dizer "pronto".**

---

## §1 — O bug ativo (relato do Enio, 2026-07-11)

> "time ainda não funciona e **anula a animação de posição (dentre outras)**."

Bindar **Time** numa entidade animada e criar keyframes de Time **congela** a animação de posição (e das outras tracks da entidade) — em vez de retimá-la. O "(dentre outras)" bate com o mecanismo: **todas** as tracks da entidade amostram no relógio remapeado; se o relógio congela, tudo congela.

---

## §2 — Causa-raiz **CONFIRMADA** (com repro reproduzível)

O seed do keyframe de Time e a amostragem do remap usam **transforms de tempo DIFERENTES** — inconsistência que meu fix `56217b44` deixou passar (consertei o `remapped_time`, mas **não** o seed).

- **Amostragem** (`crates/ph2d-timeline/src/apply.rs`, `pub fn remapped_time`): fora do intervalo de keys **extrapola em slope 1** (identidade) — corrigido no `56217b44`.
- **Seed do K** (`shells/desktop/src/render_loop/timeline_bridge.rs:159-171`, `key_value_for`, branch `TimeRemap`): usa **`tr.sample(t_secs)`** = **flat-clamp** fora do intervalo. **NÃO** foi atualizado.

**Consequência (o freeze):** o fluxo natural do usuário é K em dois tempos pra ter duas âncoras (K em t=0, depois scrub e K em t=2/t=4):
1. K@0 (track vazia) → seed identidade → `value=0 @ t=0`.
2. K@2 (track já tem 1 key) → `tr.sample(2)` **flat-clampa** no último valor → `value=0 @ t=2`.
3. Track Time = `{(0,0),(2,0)}` = **remap PLANO** = **freeze na fonte 0**. Toda track da entidade amostra em t-fonte 0 → **pose de t=0 constante = "animação anulada".**

### Repro confirmado (rodei através do caminho REAL do shell — `key_value_for` + `key_insert_time` + `apply_from_doc`):
`x@1 = 0` (congelado; identidade daria 2.5) · `x@3 = 2.5` (identidade daria 7.5). Cole este teste em `timeline_bridge.rs` (`mod tests`) pra reproduzir vermelho — é o **alvo irrefutável** desta correção:

```rust
#[test]
fn time_remap_double_k_must_not_freeze_position() {
    use ph2d_anim::AnimValue::Float;
    use ph2d_ecs::{Transform, World};
    use ph2d_core::Vec2;
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let eb = e.to_bits();
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (4.0, 10.0)] {
        ph2d_timeline::apply_intent(&mut st, &mut ph, TimelineIntent::AddKey {
            entity: eb, prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(t), value: Float(v),
            interp: ph2d_anim::Interp::Linear });
    }
    ph2d_timeline::apply_intent(&mut st, &mut ph,
        TimelineIntent::Bind { entity: eb, prop: PropKind::TimeRemap });
    // Duas prensadas de K pelas MESMAS funções que o handler de K do shell usa.
    for playhead_t in [0.0f64, 2.0] {
        let v = key_value_for(&w, &st, eb, PropKind::TimeRemap, playhead_t).unwrap();
        let t = key_insert_time(&st, eb, PropKind::TimeRemap, playhead_t);
        ph2d_timeline::apply_intent(&mut st, &mut ph, TimelineIntent::AddKey {
            entity: eb, prop: PropKind::TimeRemap, t, value: v, interp: default_interp() });
    }
    ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, 1.0);
    let x1 = w.get::<Transform>(e).unwrap().translation.x;
    assert!((x1 - 2.5).abs() < 1e-4, "posição congelada: x@1 = {x1}, esperado 2.5");
}
```

---

## §3 — Fix candidato (**confirmado no repro**, mas **NÃO commitado** — é seu, valide no app)

Fazer o seed usar a MESMA transform da amostragem. Em `key_value_for`, branch `TimeRemap` (`timeline_bridge.rs:159-171`), trocar o `tr.sample(t_secs)` flat-clamp por `remapped_time`:

```rust
if prop == PropKind::TimeRemap {
    let source = ph2d_timeline::remapped_time(&timeline.doc, entity, t_secs);
    return Some(ph2d_anim::AnimValue::Float(source as f32));
}
```

Com isso, no repro acima: `x@1 = 2.5`, `x@3 = 7.5` (identidade restaurada — verde). K **dentro** do intervalo continua caindo **na-curva** (`remapped_time` in-range == `tr.sample(t)`), track vazia continua **identidade** (`remapped_time` empty == `t`) — só o caso **fora do intervalo** muda (flat-clamp → slope-1). O teste existente `k_seeds_a_time_remap_key_on_its_curve_or_at_the_identity` sobrevive (checar).

**Por que NÃO commitei:** é exatamente o tipo de one-liner unit-green que já falhou no app 2×. **A "busca da solução" que estou te passando é a VALIDAÇÃO REAL, não a linha.** Ver §4.

---

## §4 — Protocolo de verificação OBRIGATÓRIO (a lição que custou 2 fixes)

Não declare pronto sem TODOS:

1. **Reproduzir NO APP RODANDO** (não só no unit):
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop`
   Anime X de um sprite (2 keys, 0→4s) → **+Track → Time** → **K em t=0**, scrub p/ t=2, **K de novo** → **antes do fix a posição congela**; depois do fix deve tocar identidade; então arraste as âncoras de Time no graph pra ver slow-mo/freeze/reverse **de verdade**.
2. **Varredura dos OUTROS caminhos que autoram valor de Time** (o freeze pode ter mais de uma fonte — não pare no primeiro):
   - **Graph editor — criar/arrastar âncora** na track Time: de onde vem o valor? (`crates/ph2d-panel-timeline/src/anchor_drag.rs` + `graph.rs`). Arrastar valor é user-driven (ok), mas **criar** key novo por gesto no graph pode reusar um seed flat.
   - **Auto-key**: confirmado que NÃO toca TimeRemap (`sample_prop_value(TimeRemap)=None`) — re-confirme que segue assim.
   - **Duplicate/Paste** de keys de Time: preserva valores (ok em teoria, teste).
   - **`key_value_for` com track HOLD no último key**: `remapped_time` respeita `Hold` (freeze deliberado). Garanta que o seed não vira freeze acidental aqui.
3. **Instrumentar o caminho real** se o app contradisser o unit (lição [[feedback_harness_reproduces_mechanism_not_context]]): logue `remapped_time`/seed no frame real (`PH2D_...` env), não confie no teste isolado.
4. Gate de fechamento: `cargo test -p ph2d-timeline -p ph2d-host-desktop` + clippy `--all-targets` + `rustup run 1.95 cargo fmt` + LOC caps + dhat `no_alloc_bridge`.

**Regressão a cobrir com teste:** o repro do §2 (vermelho→verde) **e** um teste in-range provando que K entre keys ainda cai na-curva (não regride o design original).

---

## §5 — Próximas tarefas da fila (prioridade do Enio; ele decide a ordem)

1. **(TOP) Consertar o Time remap de verdade** — §2–§4. É o bloqueador atual.
2. **Smoke pendente do Enio:** roving keys (`5b8b6e7f`) ainda não foi smokado no app. Ver o fluxo no [handoff integração §12](HANDOFF_line_anim_integracao_2026-07-11.md).
3. **W4 (cauda, coordena leve com Motion):** T4 (docar timeline no `motion_timeline_slot`) · T7 (relógio único: `MotionTransport` deriva do `Playhead`, remover transporte duplicado em `motion_bridge.rs`). T6/B5 (save cena+timeline unificado + id estável de entity) **fica deferido** (cross-cutting).
4. **W5 restante:** performing (gravar no play) · NLA / multi-clip (dado já é `Vec<NamedClip>`, UI deferida) · markers→signals · MCP/Luau · bake · export.
5. **Dívida de UX (W2.E9):** ~14 ColorTokens + chaves i18n `panel.timeline.*` — hoje strings do painel hardcoded em inglês (checar [[feedback_app_ui_english_only]]: labels de app SÃO inglês; o débito é serem hardcoded, não traduzir).

---

## §6 — Como tocar ESTA linha e entregar pra integração (Modo L)

- **Isolamento:** edite dentro do worktree, **sempre por caminho absoluto** ([[feedback_sed_relative_path_hits_primary_cwd]] — o CWD reseta pós-compact; `cat >>`/`sed`/`cargo` com path relativo escrevem no `main`). Nunca `git add -A`: `git add -- <meus paths>` + `git commit --no-verify -m "..." -- <meus paths>`.
- **Texto acentuado (docs pt-BR):** só via **Edit tool**, nunca `perl -e`/`sed` com literal não-ASCII ([[feedback_perl_utf8_mojibake_use_edit_tool]] — já corrompeu o plano da Timeline uma vez).
- **Foundational editável** com cuidado (`ph2d-anim`/`ph2d-timeline` são foundational; ao mexer, projete pra isolamento — módulo irmão / campo apendado). O time remap + roving já seguiram isso (variant/campo apendados, `DOC_VERSION` bumpado).
- **Você NÃO faz `ship.sh` nem `git push`.** Ao terminar (Time consertado + validado no app): **atualize** o [handoff de integração](HANDOFF_line_anim_integracao_2026-07-11.md) (adicione uma §13 com o fix do Time + símbolos novos + prova + a §11.1 marcada como SUPERSEDED) e o CLAUDE.md §5 da linha, feche, e **PARE**.
- **Integração com o main:** só quando o **Enio ordenar explicitamente**. Aí um **integrador dedicado** roda `scripts/foundational-integrate.sh` (gate da árvore combinada) e o **ship completo** ([[project_integrator_ship_catches_latents_budget_iterations]]: o gate per-linha NÃO roda fmt/clippy-all/machete/deny — só o ship pega latentes; orce 2–4 iterações). **`DOC_VERSION` foi 1→2** nesta linha: se outra linha também bumpou schema, é colisão de mesmo-símbolo → **reporte ao Enio** (não renegocie).
- Base = `main` HEAD no fork (`1c7c9a22`); os 17 commits são FF puro. Se o `main` andou, o integrador rebaseia; conflito **fora dos seus arquivos** (mesmo-símbolo) → PARE e reporte.

---

## §7 — Mapa rápido dos arquivos do time remap

| O quê | Onde |
|---|---|
| Amostragem remapeada (extrapola slope-1) | `crates/ph2d-timeline/src/apply.rs` → `pub fn remapped_time` |
| **Seed do K (O BUG)** | `shells/desktop/src/render_loop/timeline_bridge.rs:159-171` → `key_value_for` |
| Tempo de inserção do K | `timeline_bridge.rs` → `key_insert_time` |
| Handler do K (itera bindings da entidade) | `shells/desktop/src/render_loop/mod.rs:754-790` |
| +Track → Bind | `shells/desktop/src/render_loop/mod.rs:1170-1176` |
| PropKind::TimeRemap = 6 (fora do `ALL`) | `crates/ph2d-timeline/src/prop.rs` |
| Auto-key (nunca toca TimeRemap) | `shells/desktop/src/render_loop/autokey_pass.rs` |
