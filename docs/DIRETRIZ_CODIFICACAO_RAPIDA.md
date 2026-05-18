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

## 12. LOC threshold para validação (v1.1)

**Regra:** não rode `cargo check` antes de ter editado/movido pelo
menos ~1200 LOC novos/movidos OU completado uma operação lógica coesa
(ex: criar um módulo inteiro). Cada `cargo check` custa 10-30s +
redirecionamento mental — antes de ter um chunk significativo, é
noise.

| LOC editados/movidos | Comando OK |
|----------------------|-----------|
| 0-400 | NADA. Continue editando. |
| 400-1200 | `cargo check -p <crate>` opcional (se você está incerto). |
| 1200+ ou módulo inteiro | `cargo check -p <crate>` — sane stop. |
| Phase fechada (múltiplos crates) | `cargo check --workspace` (NÃO test). |
| Antes do commit | **nada** — hook valida. |

**Não rode `cargo test` durante editing burst.** Tests só no:
1. Fim do commit (via hook), OU
2. Diagnóstico de uma falha específica que o hook reportou.

---

## 13. Batch maior de commits (v1.1)

**Regra atualizada:** Wave 8 Phase 2 (originalmente brief = 5
commits 2.A/.B/.C/.D/.F) pode ser **1 commit único** se as
mudanças formam um endgame coerente.

Cada commit dispara o pre-commit hook (~5-10min em T2). 5 commits =
~25-50min só de hook. 1 commit = ~5-10min. Margem de 4x.

**Trade-offs:**
- ✅ Tempo: 1 commit é 5x mais barato em hooks.
- ✅ Coerência: o leitor vê o endgame de uma vez (mais útil que 5
  commits intermediários "WIP").
- ❌ Bisect: mais difícil isolar regressão. Mitigação: Enio testa
  manualmente e não usa bisect em rotina.
- ❌ Revert: rollback derruba tudo. Mitigação: refactor preserva
  comportamento — improvável precisar reverter.

**Quando granular AINDA faz sentido:**
- Mudança que altera comportamento + mudança que preserva → 2
  commits separados (smoke pode passar/falhar atestando uma sem
  a outra).
- Trabalho em paralelo entre agentes coordenados (commit fronteira).
- Wave inteiro fecha + closeout docs separado.

**Default:** 1 Phase = 1 commit. Múltiplos sub-stages comprimem.

---

## 14. Editing burst — não interrompa o flow (v1.1)

**Regra:** edit 5+ arquivos seguidos sem rodar cargo entre.

```
[Edit a.rs] [Edit b.rs] [Edit c.rs] [Edit d.rs] [Edit e.rs]
            ↓
[cargo check -p <crate>]   ← UMA vez no fim
            ↓
[Fix all errors em 1 burst novo]
            ↓
[cargo check -p <crate>]   ← validação final
```

NÃO faça:

```
[Edit a.rs] [cargo check] [Edit b.rs] [cargo check] ...   ← 5x overhead
```

Compilador erra em batch também. Não precisa one-at-a-time.

---

## 15. Delete > back-compat re-exports (v1.1)

**Regra:** quando você move algo de `crate::a::foo` para
`crate::b::foo`, prefira **atualizar todos os call sites direto**
em vez de criar `pub use crate::b::foo;` em `crate::a::*`.

Custo do re-export:
- +N linhas no arquivo origem
- +1 ponto de manutenção quando o nome mudar de novo
- Disfarça onde a verdade vive (debug fica mais difícil)
- A LLM gasta tempo escrevendo o re-export + a documentação dele

Custo de atualizar call sites:
- N edits, mas todos triviais (path swap)
- `grep -l 'old::path' | xargs sed -i 's|old::path|new::path|g'`
  faz em 1 comando

**Exceção:** API pública estável consumida por terceiros (ex:
`ph2d_editor::HeroScreen` re-exportado de `crate::screens::hero`).
Internal: delete.

---

## 16. Doc comments / commit messages — menos é mais (v1.1)

**Doc comments:** não escreva mid-refactor. O código é a doc. Save
escrita longa para o **closeout commit** de fim de Wave (1 commit
de docs cobrindo tudo).

**Commit messages:** 1-3 parágrafos no body, não 30 linhas. O título
+ 5 linhas no body já carrega o "what + why".

```
RUIM:
fix(editor): Wave 8 Phase 2.A — physical showcase tree to ph2d-editor-core::widget::showcase

Audit S1 + A3 + P1 step 2. Following Phase 2.A.0 (chrome hoist),
this commit physically moves the entire Widget Gallery showcase
tree out of `ph2d_editor::screens::hero::inspector::showcase` into
`ph2d-editor-core::widget::showcase`. Goal: panel crates can paint
the showcase without depending on `ph2d-editor`.

Moves:
- `showcase/{actions,body,card,...}.rs` (10 section painters + ...)
...
[40 more lines]

BOM:
refactor: move showcase tree → editor-core::widget::showcase (Wave 8 Phase 2.A)

12 showcase files + notes + 5 helpers + 7 ID constants now live in
editor-core. Inspector re-exports for back-compat. Widget Gallery
panel imports from editor-core, no longer reaches into ph2d-editor
internals. Audit S1 + A3 closed.

cargo test workspace 1315 green.
```

---

## 17. Reads cirúrgicos (v1.1)

❌ `Read large_file.rs` (sem offset/limit) só pra "ver a estrutura".
✅ `Bash: grep -n 'pub fn|pub struct|^use' file.rs | head -20` →
   `Read offset=X limit=20` na seção relevante.

❌ Re-Read arquivo que acabou de Edit/Write.
✅ Confiar na confirmação do tool. Próximo erro do compilador é
   evidência mais barata.

❌ Read 5 arquivos sequencialmente.
✅ 5 Read tools em paralelo na mesma mensagem.

---

## 18. Skip "validation matrix" antes do hook (v1.1)

**Regra reforçada:** o pre-commit hook T2 é a matriz oficial. Rodar
`cargo build + clippy + test` manualmente antes é gastar o tempo
duas vezes.

Caso especial OK:
- Mudou Cargo.toml (deps/features) → `cargo build -p <consumer>
  --features <combo>` UMA vez pra confirmar features resolvem.
- Caso contrário, deixa o hook decidir.

---

## 19. Versão + revisão

**Versão atual:** 1.2 (2026-05-18 sessão noite)
**Origem:** v1.1 estabilizou cadência mas ainda lenta para Enio.
v1.2 dobra o LOC threshold pra ~1200 (de 600), mais raros checkpoints,
mais editing burst entre validações.

**Revise este doc se:**
- Pre-commit hook mudar de comportamento.
- Workspace ganhar crates novos significativos que afetem tempo de
  build.
- Padrão de smoke/push mudar.

Vide [`CLAUDE.md`](../CLAUDE.md) §CI / GitHub Actions e
[`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md)
§5.2 pre-commit hook tiered + §6 Smoke/PR/CI.
