# HANDOFF — Pós-Phase-C: babysit CI até zero erro

**Data:** 2026-05-19.
**Estado:** Phase C inteira fechada e pushada para `origin/main`. CI da run pós-push ainda em curso. **Sua missão:** ficar de babá da CI até a run conclua, corrigir qualquer falha até zero erros.

## 0. Verificação rápida

```bash
git log --oneline -5
# 88cce53 fix(panels): sync Cargo.lock after ph2d-tokens drop from widget-gallery
# c4c6905 fix(panels): drop unused ph2d-tokens dep from ph2d-panel-widget-gallery
# a873d8f refactor(panels): ADR-0029 Phase C.4 — Grid Snap typed Panel migration (Phase C closes)
# 6ebe7f0 docs: HANDOFF_WAVE_8_PHASE_C4 — pickup guide + Phase C closing instructions
# 4a8e361 refactor(panels): ADR-0029 Phase C.3 — Widget Gallery typed Panel migration

git status -sb
# ## main...origin/main   (working tree clean)
```

Se diverge, pare e pergunte ao Enio.

## 1. Run de CI ativa

`https://github.com/dibrioli/PH2D/actions/runs/26090388703` — commit `88cce53`.

Comandos úteis:

```bash
# status corrente:
gh run view 26090388703 --json status,conclusion --jq '"\(.status) \(.conclusion)"'

# jobs que falharam (com os steps específicos):
gh run view 26090388703 --json jobs --jq '.jobs[] | select(.conclusion=="failure") | {name, steps: [.steps[] | select(.conclusion=="failure") | .name]}'

# logs do(s) job(s) que falharam (verboso, grep antes):
gh run view 26090388703 --log-failed 2>&1 | grep -E "error\[|error:|warning:" | head -50
```

## 2. Histórico de fixes pós-Phase-C

Em ordem cronológica, cada um disparou uma run nova:

| Commit | Erro original na CI | Fix |
|--------|---------------------|-----|
| `a873d8f` (C.4 close) | `cargo machete` flagou `ph2d-tokens` como unused em `ph2d-panel-widget-gallery` | `c4c6905` removeu a dep do Cargo.toml |
| `c4c6905` | MSRV job `cargo check --workspace --locked` recusou atualizar Cargo.lock | `88cce53` regenerou + commitou o lockfile |
| `88cce53` | **em curso** — você acompanha | (a definir) |

## 3. Como corrigir cada categoria de falha (atalhos)

### 3.1 `cargo machete` (lint job)

Reproduzir localmente:

```bash
cargo machete 2>&1 | tail -10
```

Solução: remover a dep do `Cargo.toml` flagada, ou se for falso positivo (raro — `#[derive]` que reexpande tipos de um dep transitivo), adicionar:

```toml
[package.metadata.cargo-machete]
ignored = ["nome-da-dep"]
```

Após editar Cargo.toml, **commite junto** a regeneração do `Cargo.lock` (rode `cargo check --workspace` antes do `git add`).

### 3.2 MSRV `--locked` (Cargo.lock fora de sync)

Reproduzir:

```bash
cargo check --workspace --locked 2>&1 | tail -3
```

Solução: `cargo check --workspace` (sem `--locked`) para atualizar; depois `git add Cargo.lock` e commit.

### 3.3 `cargo fmt --check`

```bash
cargo fmt --all --check 2>&1 | head -30
# para corrigir:
cargo fmt --all
```

### 3.4 `cargo clippy -- -D warnings`

```bash
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -E "warning:|error\[" | head -30
```

Resolva cada warning como erro (esse é o gate da CI).

### 3.5 `cargo deny check`

```bash
cargo deny check 2>&1 | tail -30
```

Normalmente são security advisories ou license issues — siga o output literal pra decisão.

### 3.6 Tests (matrix linux/macOS/windows)

CI matrix roda ~30min total. Reproduza com:

```bash
cargo test --workspace 2>&1 | grep -E "FAIL|^test result"
```

Se for falha plataforma-específica (`#[cfg(target_os = ...)]`), pode precisar gate.

## 4. Padrão de commit pra fixes de CI

```
fix(panels): <descrição curta>

<por que isso foi necessário; link pro job que falhou se possível>

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

`type` típico:
- `fix(panels)` — se relacionado às migrações Phase C
- `fix(ci)` — se infra (workflow YAML, deny.toml, etc)
- `fix(deps)` — se Cargo.toml/Cargo.lock crud

## 5. Cadência

- 1 fix → 1 commit → 1 push → 1 run nova.
- Não acumule múltiplos fixes locais sem push (o ciclo de feedback CI é caro: ~30min de run; cada erro deve ser visto rápido para refazer).
- Após cada push, **forneça o link da nova run ao Enio** (CLAUDE.md §CI Default).
- Loop até a run completar com `conclusion: success`.

## 6. Quando você terminou

Quando a CI ficar verde:

1. Atualize memória: criar `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_phase_c_ci_green_2026_05_19.md` documentando os fixes que foram necessários (vide tabela §2).
2. Adicione linha em `MEMORY.md` substituindo a entrada `project_phase_c_complete_2026_05_19.md` por uma `project_phase_c_ci_green_<data>`.
3. Reporte ao Enio: hash do commit final, link da run verde.

## 7. Memória persistente (LEIA ANTES DE COMEÇAR)

Caminho: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`. Leia em particular:
- `feedback_ci_handling.md` — Enio confere CI visualmente; LLM **não fica em polling loop** sem instrução explícita. Hoje o Enio pediu explicitamente "monitore CI até zero erros", então polling está autorizado.
- `feedback_ci_batching.md` — durante waves, não push/CI a cada PR; **mas isso já passou** (Phase C inteira foi pushada). Estamos no babysit-mode.
- `feedback_phase_cascade_2026_05_19.md` — autoriza o cascade de agentes que aconteceu. Você é o agente de babysit pós-cascade.
- `feedback_communication_style.md` — pt-BR direto, opções concretas.
- `project_phase_c_complete_2026_05_19.md` — sumário do que C entregou.

## 8. Comandos úteis (atalhos)

```bash
# nova run após cada push:
gh run list --workflow=spike.yml --limit=1 --json databaseId,headSha,status,conclusion

# tail dum job específico em tempo real:
gh run watch <run-id>

# (NÃO use `gh run watch` em loop — gasta token. Use o polling Monitor abaixo.)
```

Para polling não-bloqueante (rola em background, te notifica quando concluir):

```python
# pseudo — equivalente Bash/Monitor:
# use Monitor com timeout 3600000 (1h) e command:
#   while true; do
#     s=$(gh run view <RUN_ID> --json status,conclusion --jq '"\(.status) \(.conclusion)"')
#     case "$s" in
#       "completed "*) echo "DONE $s"; break;;
#     esac
#     sleep 60
#   done
```

Ou no tooling do Claude Code: ferramenta `Monitor` com `command:` igual ao acima.

## 9. Saída do agente atual

O agente que escreveu este handoff parou o monitor que estava rodando (task `bgh0ivsxx` foi `TaskStop`'d) para não confundir. A nova sessão pode armar seu próprio polling.

GO — pegue o link da §1, cheque status, corrija o que aparecer.
