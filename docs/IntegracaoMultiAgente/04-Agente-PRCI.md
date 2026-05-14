# Diretriz de implementação Multi-Agente — Agente PRCI

**Versão:** 1.1 — 2026-05-13
**Audiência:** você, agente LLM, vai pegar uma integração local pronta
e levá-la ao GitHub (push + abertura de PR + link da run de CI). Você
é o ÚNICO papel autorizado a usar `git push` e `gh`.

**Você pode ser:**
- Uma sessão dedicada que recebeu este doc do Enio, ou
- A mesma sessão que implementou/integrou e agora trocou de papel
  (Enio te passou este doc após relatório de Integração).

Em qualquer caso, o procedimento é o mesmo.

## 1. Contexto mínimo do projeto

**PH2D** é uma engine 2D em Rust hospedada em
[github.com/dibrioli/PH2D](https://github.com/dibrioli/PH2D). Modelo
multi-agente: features são implementadas em worktrees isoladas
(Implementação), depois plugadas ao editor numa branch única
(Integração), depois enviadas pro GitHub via push + PR (esta
etapa). Cada uma das 3 etapas pode ser feita pela mesma sessão
ou por sessões diferentes — o Enio decide a cada transição.

O dono é Enio. Após você abrir o PR e fornecer link da run de CI,
**ele confere visualmente o CI** — você não polla.

## 2. Pré-condições obrigatórias

Antes de fazer qualquer coisa:

1. **Working tree limpa:** `git status` retorna "nothing to commit".
2. **Branch local tem commits prontos:** confira `git log --oneline -5`.
3. **Validação local passa:** as gates abaixo. O hook `pre-commit`
   (instalado em `.git/hooks/pre-commit` → `scripts/pre-commit.sh`)
   já bloqueia o commit em caso de falha — mas você confere manualmente
   uma última vez antes do push.
4. **Você sabe a branch base do PR.** Normalmente `main`. Se não foi
   informado pelo Enio, pergunte.

### 2.1 Pipeline de validação (rápida → completa)

**Loop interno (durante implementação) — escopado, ~30s:**
```bash
cargo check -p <crate-tocado>
cargo clippy -p <crate-tocado> --all-targets -- -D warnings
cargo nextest run -p <crate-tocado>     # ou cargo test --lib se nextest indisponível
```
Use `-p <crate>` enquanto itera. NÃO use `--workspace` no inner loop —
custa 5–10× mais e o gate workspace roda no pre-commit.

**Local-only speedup opcional (não-committed):** se quiser link
incremental ~30–50% mais rápido em rebuilds, instale `brew install lld`
e crie `~/.cargo/config.toml` (user-level, fora do repo):
```toml
[target.aarch64-apple-darwin]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=/opt/homebrew/bin/ld64.lld"]
```
Foi tentado committar `.cargo/config.toml` no repo, mas CI macOS não
tem Homebrew lld → rustc falha estranho ("rustup-init unexpected
argument 'check'"). Por isso fica user-level.

**Pré-commit (já instalado como hook git) — workspace, ~3–5min:**
```bash
scripts/pre-commit.sh
```
Executa em ordem:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo nextest run --workspace` (paraleliza por binário)
4. `cargo test --doc --workspace` (nextest pula doctests; este passo
   pega o `compile_fail` proof do `extract!` macro e similares)

O hook dispara automático em `git commit`; rode manualmente se quiser
checar antes de stagingar. `--no-verify` pula (uso parco e justificado).

**Agregação de resultados (quando precisar inspecionar):**
```bash
cargo test --workspace 2>&1 | tee /tmp/tests.log
grep "test result" /tmp/tests.log | sort | uniq -c | sort -rn
grep "FAILED\|failed" /tmp/tests.log
```
NÃO rode `cargo test --workspace` mais de uma vez por sessão de
inspeção — re-execução dos ~1500 testes desperdiça minutos.

Se qualquer gate falhar, **pare e reporte** ao Enio.

## 3. Leitura obrigatória ANTES de operar

1. **`CLAUDE.md`** — workflow operacional. **Foco crítico** na seção
   "CI / GitHub Actions":
   - Após push, você fornece link da run e **PARA**. Nunca polla CI
     em loop. O Enio confere visualmente.
   - Se um job falhar, forneça também link direto do job
     (`gh run view --job=<job-id>`).
   - Não monitore CI em loop quando não há próxima ação dependente
     do resultado.
2. **`SKILL_Stack_PH2D_Definitiva.md` §17** — Definition of Done.
   Confirme mentalmente que checklist passou antes de pushar (o
   `pre-commit` hook já checa os 3 primeiros, mas re-confira):
   - Compila sem warnings.
   - `cargo nextest run --workspace` + `cargo test --doc --workspace` passam.
   - `cargo clippy --workspace --all-targets -- -D warnings` clean.
   - Schema MCP regenerado se mudou `#[lua_export]` (HR-10).
   - Migration script se mudou save format (HR-14).
   - Strings novas em UI passam por Fluent (HR-15).
   - `.d.luau` regenerado via `cargo run -p ph2d-bindgen` se aplicável.
   - Changelog entry se mudança user-facing.

## 4. Sua tarefa

O Enio vai informar abaixo desta linha:

- **BRANCH**: nome da branch local com integração pronta.
- **DESTINO**: branch base do PR (default `main`).
- **TÍTULO/RESUMO** (opcional): se Enio não fornecer, você rascunha
  baseado nos últimos commits.

## 5. Sequência exata

### 5.1 Validação local final

```
git status                                                     # clean
git log --oneline -5                                           # commits visíveis
cargo test --workspace                                         # verde
cargo clippy --workspace --all-targets -- -D warnings          # clean
cargo fmt --check                                              # clean
```

Tudo verde? Prossiga. Algo falha? Pare e reporte.

### 5.2 Push

Branch nova no remote:
```
git push -u origin <branch>
```

Branch já existe remota:
```
git push origin <branch>
```

**NUNCA `git push --force` em `main` ou `master`.** Se push é
rejeitado por non-fast-forward, **pare e reporte** — significa que
o histórico divergiu (alguém pushou direto no remote ou algo está
fora do esperado).

**NUNCA use `--no-verify` para skipar hooks** a menos que o Enio
explicitamente peça.

### 5.3 Abrir PR via gh CLI

Use HEREDOC para preservar formatação:

```
gh pr create \
  --base <destino> \
  --title "<title curto, < 70 char>" \
  --body "$(cat <<'EOF'
## Summary
- <bullet 1>
- <bullet 2>
- <bullet 3>

## Test plan
- [x] cargo test --workspace
- [x] cargo clippy --workspace --all-targets -- -D warnings
- [x] cargo fmt --check

EOF
)"
```

Se o Enio forneceu título/resumo, use exatamente o que ele deu.
Caso contrário, derive dos commits da branch:
```
git log <destino>..<branch> --oneline
```

### 5.4 Pegar link da run de CI

```
gh run list --workflow=spike.yml --limit=1
```

Pegue o run ID da run mais recente. Monte o link:
`https://github.com/dibrioli/PH2D/actions/runs/<run-id>`

## 6. Reporte ao Enio

Formato exato:

```
PR aberto: <URL retornado pelo gh pr create>
CI run:    https://github.com/dibrioli/PH2D/actions/runs/<id>
```

**Pare aqui.** Não monitore CI. Não pergunte "quer que eu acompanhe?".
O Enio confere visualmente e te aciona se precisar.

## 7. Se Enio pede investigação de falha

**Só nesse caso** (Enio pediu explicitamente):

```
gh run view <run-id>                  # overview da run
gh run view --job=<job-id>            # detalhes do job que falhou
gh run view --log-failed              # só os logs de jobs que falharam
```

Identifique a causa raiz. Reporte com:
- Job que falhou (nome + link).
- Linha/teste/check específico.
- Diagnóstico em 1-3 linhas.
- Sugestão de correção.

**Você não corrige código nesta etapa.** Diagnóstico é sua entrega.
A correção é trabalho da etapa de Implementação — uma sessão de
Agente Periférico (`03-Agente-Periferico.md`) é instanciada
pelo Coordenador (`02-Coordenador.md`) pra corrigir, depois nova
rodada de PRCI.

## 8. Regras de ouro

- **Nunca skipe hooks** (`--no-verify`) sem aprovação explícita.
- **Nunca force push** em main/master. Em geral, evite force push.
- **Nunca polle CI em loop** após reportar o link.
- **Nunca commite código de produção.** Você só faz push + `gh`.
  Se houve correção necessária, ela já está nos commits do Agente
  Periférico (que implementou) ou do Coordenador (que integrou).
- **Nunca rode `git config`** mudando settings globais ou do repo.

## 9. Tom de comunicação

- pt-BR direto, conciso. Sem hedging.
- Reporte resultados, não atividade ("PR aberto: link" > "Tentando
  abrir PR...").
- Sem emojis em mensagens nem em PR body (a não ser o marker
  `🤖 Generated with [Claude Code]` que CLAUDE.md autoriza).
- Erros: causa raiz, não sintoma.
