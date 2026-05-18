# DIRETRIZ — Codificação rápida (LLM dev cadence)

**Princípio:** o tempo de espera por compilação/teste é o gargalo dominante
em projetos Rust grandes. A LLM que **valida o mínimo necessário e
confia no pre-commit hook** entrega 3-5x mais por sessão que a LLM que
roda `cargo test --workspace` a cada mudança.

Este doc é a referência operacional. Leia se você está fragmentando
demais ou rodando testes redundantes.

---

## 1. Regra-mãe

> **Não duplique o trabalho do pre-commit hook.**

O hook T2 (`pre-commit` em PH2D) já roda `cargo clippy --workspace
--all-targets -- -D warnings` + `cargo test --workspace --exclude
ph2d-asset` antes de aceitar o commit. Se a LLM já rodou esses dois
manualmente antes do commit, **gastou ~5-10min duas vezes pelo mesmo
sinal**.

A única razão legítima pra rodar `--workspace` fora do hook é:
1. Fim da Wave (antes do push), pra ver o resultado completo.
2. Diagnóstico de falha pontual (ex: dois testes flakem, quero confirmar).

Em qualquer outra situação, **use comandos escopados**.

---

## 2. Tabela de validação por escopo

Escolha o comando pela situação, não pelo costume:

| Situação | Comando | Tempo típico |
|----------|---------|--------------|
| Editou 1 arquivo, quer saber se compila | `cargo check -p <crate-tocado>` | 3-15s |
| Editou crate, quer rodar testes dele | `cargo test -p <crate>` | 5-30s |
| Editou crate, quer rodar UM teste | `cargo test -p <crate> --test <nome>` ou `-- <pattern>` | 1-5s |
| Editou `editor-core` (foundation), quer ver downstream | `cargo check --workspace` (NÃO test) | 30-60s warm |
| Mexeu em Cargo.toml de feature | `cargo build -p <consumer> --no-default-features --features <X>` | 10-30s warm |
| Vai commitar (T2 hook vai rodar) | **nada** — deixa o hook validar | 0s |
| Hook falhou | leia output, corrija a causa, comite de novo | — |
| Fim do Wave, antes de push | `cargo test --workspace --exclude ph2d-asset` uma vez | 3-5min |
| Só mudou `.md` / docs | **nada** — o hook é T0 (skip) | 0s |

`cargo check` (sem `test`) pula codegen+linkagem; é 3-5x mais rápido
que `cargo test` quando você só quer "compila?".

`cargo clippy -p <crate>` também é OK pra detectar lints localmente
quando você está incerto.

---

## 3. O que NÃO fazer

❌ **`cargo test --workspace` depois de cada edit.**
   Mata produtividade. O hook faz isso.

❌ **`cargo clippy --workspace --all-targets` antes do commit.**
   O hook também faz isso.

❌ **Rodar a "matriz completa" (build + clippy + test + feature combos)
   antes de cada commit.**
   Reserve isso pro fim da Wave.

❌ **Re-rodar testes que já passaram pra "confirmar".**
   Se você não mexeu em nada, eles ainda passam. Cache de resultados é
   in-your-head.

❌ **Validar o baseline no início da sessão se o sha bom mais recente já
   está verde.**
   Use `git log` pra confirmar o último commit + STATE.md. Só rode
   testes baseline se há motivo (mudou branch, etc).

❌ **`cargo build` antes de `cargo test`.**
   `cargo test` já compila. Duplicado.

❌ **Ler arquivos só pra ver "como eles ficaram" depois do Edit.**
   O Edit tool falha em vez de aplicar mudança quebrada. Se ele
   succedeu, o arquivo está como esperado. Re-Read é desperdício de
   context window.

---

## 4. Cadência de commits durante Waves

Pareando com `feedback_commit_cadence` + `feedback_ci_batching` na
memória do LLM:

✅ **Acumule mudanças por sub-stage (não sub-sub-stage).**
   Wave 8 Phase 2 = ~5 commits (2.A, 2.B, 2.C, 2.D, 2.F). Não
   fragmentar pra 2.A.0/.1/.2/.3 a menos que cada sub-sub seja
   imediatamente útil isolado.

✅ **Múltiplos arquivos no mesmo commit se compõem uma mudança lógica.**
   Ex: hoist + re-export + opt-out de teste = um commit.

✅ **Push único no fim da Wave**, não a cada commit.
   PRCI babysit roda uma vez ao final.

✅ **Smoke do Enio uma vez no fim da Wave inteira.**
   Brief Wave 8 pede smoke por phase porque o refactor é alto risco;
   isso é exceção, não regra.

❌ **Não amend, não force-push, nunca `--no-verify`.**

---

## 5. Quando rodar smoke

Smoke = Enio roda `./play.command` e clica em coisas pra verificar
visualmente.

| Tipo de mudança | Smoke? |
|-----------------|--------|
| Refactor preserva-comportamento (move código, re-export, hoist) | **Não** mid-Wave — só no fim. |
| Mudança visual (paint, layout, cor) | Sim — depois do commit, antes do próximo. |
| Mudança de input/interação | Sim — depois do commit. |
| Mudança lógica que afeta apply_event | Sim. |
| Docs only | Não. |
| Alta complexidade (Wave 8 Phase 2 Stage 4) | Sim per-phase (exceção pelo brief). |

---

## 6. Quando push + CI

Push pro GitHub = **uma vez por jornada, ao fim da Wave** (CLAUDE.md
§"CI / GitHub Actions"). O CI matrix (linux+macOS+windows + replay
hash + bench) demora ~30min — não roda a cada commit.

PRCI é o papel responsável pelo babysit da CI no fim. LLM Periférica
não roda CI.

---

## 7. Tool calls — paralelo onde der

Quando você precisa de N pedaços de informação **independentes**,
faça N tool calls em UMA mensagem:

```
# RUIM (3 round-trips serializados):
Bash: git status
Bash: git log
Read: file.rs

# BOM (1 round-trip):
[Bash: git status][Bash: git log][Read: file.rs]   ← mesma mensagem
```

Aplica pra Read + Grep + Bash em paralelo. Bash + Bash em paralelo
também é OK desde que comandos sejam independentes.

---

## 8. Context discipline

❌ `Read` arquivo inteiro só pra ver as primeiras 100 linhas.
   Use `Read offset=0 limit=100`.

❌ Re-`Read` arquivo que você acabou de editar.
   O harness já te deu confirmação de sucesso.

✅ Grep pra localizar primeiro, `Read` com offset depois.
   ```
   Bash: grep -n "thing" file.rs   →  Read offset=42 limit=30
   ```

✅ Use Agent (Explore subagent) pra busca larga.
   Mantém o context principal limpo de excerpts irrelevantes.

✅ Se um arquivo passa de 800 LOC, considere se você precisa do
   arquivo todo ou só de uma seção. HR-18 também limita isso.

---

## 9. Cheatsheet — comandos mais usados

```bash
# Validar 1 crate compila
cargo check -p ph2d-editor

# Validar 1 crate compila + testes
cargo test -p ph2d-editor

# Rodar 1 teste específico
cargo test -p ph2d-editor --test architecture_cycle_prevention
cargo test -p ph2d-editor -- some_test_name_pattern

# Validar feature combos só do shell
cargo check -p ph2d-host-desktop --no-default-features --features lite

# Workspace check (só compila, não testa)
cargo check --workspace

# Workspace formatar
cargo fmt --all

# Quando hook fmt falhou
cargo fmt --all && git add -u && git commit -m "..."
```

---

## 10. Indicadores que você está lento

🚩 Você está rodando `cargo test --workspace` mais de 1x por commit.

🚩 Você lê o mesmo arquivo 3+ vezes na mesma sessão.

🚩 Você comita 4+ vezes a mesma mudança lógica em sub-sub-stages.

🚩 Você roda validação ANTES do pre-commit hook em vez de deixar o
   hook reportar.

🚩 Mais de 50% do tempo da sessão foi esperando build/test em vez de
   editando.

🚩 Você usa `Bash` pra `cat` arquivos quando deveria usar `Read`.

🚩 Você fica num "loop de polling" via `sleep` esperando coisa
   acabar.

---

## 11. Quando rodar `cargo build --workspace` faz sentido

✅ **Foundation crate mudou** (editor-core, tool-registry, tokens) e
   você precisa garantir que tudo compila depois.

✅ **Você adicionou um novo crate ao workspace** e precisa ver se
   o resolver pega.

✅ **Final do Wave**, antes do push.

✅ **Você suspeita que uma mudança quebrou downstream** que `cargo
   check -p <crate>` não pega (raro — geralmente check pega tudo).

Em qualquer outra situação, escope.

---

## 12. Versão + revisão

**Versão atual:** 1.0 (2026-05-18)
**Origem:** Enio observou que o agente LLM passava mais tempo
esperando testes que codificando durante Wave 8 Phase 2.A.

**Revise este doc se:**
- Pre-commit hook mudar de comportamento.
- Workspace ganhar crates novos significativos que afetem tempo de
  build.
- Padrão de smoke/push mudar.

Vide [`CLAUDE.md`](../CLAUDE.md) §CI / GitHub Actions e
[`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md)
§5.2 pre-commit hook tiered + §6 Smoke/PR/CI.
