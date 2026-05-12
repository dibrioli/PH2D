# PARALLEL_AGENTS.md — Política de fan-out de agentes LLM

**Versão:** 1.0 — 2026-05-12
**Audiência:** LLM que vai trabalhar isolada num branch enquanto outros agentes trabalham em outros branches.

## Modelo em uma frase

Você trabalha em **uma feature isolada** (ex: nova Tool, popular um crate stub) sem tocar a estrutura central. Quando terminar, **espera o Enio confirmar que nenhum outro agente está ativo** e só então faz a integração com o resto da engine.

## Quando este modelo se aplica

- Tarefa do tipo "implemente a Tool Painter dentro do editor".
- Tarefa do tipo "popule o crate stub ph2d-audio com mixer básico".
- Tarefa do tipo "adicione widget Calendar ao ph2d-editor".

**Não se aplica** a: mudanças em `ph2d-core`, `ph2d-ecs`, `ph2d-host`, `ph2d-tokens`, ou qualquer refactor que cruza ≥3 crates. Para essas, o Enio designa um único agente e nenhum paralelismo roda em paralelo.

## O que você PODE tocar

**Para "nova Tool" no editor:**
- Arquivos novos em [crates/ph2d-editor/src/tools/](../crates/ph2d-editor/src/tools/)
- Eventual widget novo em [crates/ph2d-editor/src/widget/](../crates/ph2d-editor/src/widget/) (lembre HR-12 + HR-15 — os testes [hr12_widgets_a11y.rs](../crates/ph2d-editor/tests/hr12_widgets_a11y.rs) e [hr15_no_hardcoded_ui_strings.rs](../crates/ph2d-editor/tests/hr15_no_hardcoded_ui_strings.rs) vão pegar regressão)
- Ícone novo em [crates/ph2d-editor/src/icons.rs](../crates/ph2d-editor/src/icons.rs) (apenas adicionar variante; sem renumerar enum)
- Testes correspondentes em `crates/ph2d-editor/tests/`

**Para "popular crate stub":**
- Qualquer arquivo dentro do crate folha (`crates/ph2d-{audio,fluids,light,save,sdf,telemetry,physics-soft,net,i18n}/`)
- Adicionar deps em **`Cargo.toml` daquele crate**, nunca no workspace raiz
- Testes próprios em `crates/<crate>/tests/`

## O que você NÃO PODE tocar

Lista curta. Tocar qualquer item desta lista **paralisa seu trabalho** — sinalize o Enio antes:

- `Cargo.toml` raiz (workspace members, workspace.dependencies)
- `Cargo.lock`
- `clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `.typos.toml`
- `SKILL_Stack_PH2D_Definitiva.md`
- `CLAUDE.md`
- `docs/plans/*.md`
- `crates/ph2d-core/`, `crates/ph2d-ecs/`, `crates/ph2d-host/`, `crates/ph2d-tokens/`
- `.github/workflows/`
- `runtime/luau/ph2d.d.luau`, `runtime/mcp/schema.json` (gerados — só regerar na janela de integração)
- ADRs em `docs/architecture/decisions/`

## Fluxo de 5 passos

1. **Enio cria worktree** para você em branch `feature/<nome>` partindo de `main`.
2. **Você implementa** a feature dentro da whitelist. Compila local, testes passam, clippy clean.
3. **Você reporta** "pronto, esperando integração" — não faz push ainda, ou faz push pro branch mas não abre PR.
4. **Enio confirma** que nenhum outro agente está ativo (checa worktrees) e abre janela de integração.
5. **Você (ou agente novo) integra**: rebase em main, ajusta Cargo.toml raiz se feature exigir nova dep, regenera bindings via `cargo run -p ph2d-bindgen` se mudou catálogo, atualiza SKILL + plans, merge.

## Quando você descobre que precisa tocar a blacklist

**Pare imediatamente.** Não tente improvisar workaround. Reporte ao Enio com:
- Qual arquivo da blacklist você precisa mudar e por quê.
- Qual o mínimo de mudança suficiente.
- Se dá pra adiar até a janela de integração.

O Enio decide: (a) ajustar escopo da feature pra não precisar; (b) pausar outros agentes e abrir janela de integração agora; (c) substituir você por um agente Integrador dedicado.

## Como verificar que está sozinho

```sh
ls .claude/worktrees/
```

Se há `agent-*` ativo além do seu, você não está sozinho. Pergunte ao Enio antes de integrar.

## Hard Rules que continuam valendo (mesmo isolado)

Todas as HR-1..HR-17. Em particular as que CI pega automaticamente:
- **HR-3** (no alloc em hot path): [interaction_no_alloc.rs](../crates/ph2d-editor/tests/interaction_no_alloc.rs) + [propagate_no_alloc.rs](../crates/ph2d-ecs/tests/propagate_no_alloc.rs).
- **HR-5** (determinismo): clippy.toml workspace-wide bane `HashMap`.
- **HR-12** (a11y): [hr12_widgets_a11y.rs](../crates/ph2d-editor/tests/hr12_widgets_a11y.rs).
- **HR-15** (i18n): [hr15_no_hardcoded_ui_strings.rs](../crates/ph2d-editor/tests/hr15_no_hardcoded_ui_strings.rs).

Rodar `cargo test -p <crate>` localmente antes de reportar "pronto" evita surpresa em CI.
