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

**Pré-commit hook TIERED — auto-detecta tier pelo diff staged:**

| Tier | O que ativa | Etapas | Tempo |
|---|---|---|---|
| **T0** | só docs / `*.md` / `.gitignore` / scripts / `.github/` | fmt + typos | ~5s |
| **T1** | só uma pasta sob `crates/<X>` ou `tools/<X>` ou `tests/spike` ou `shells/<X>` (exceto desktop) | fmt + typos + `check/clippy/nextest -p <X>` | ~30s |
| **T2** | Cargo.toml/lock, `.cargo/`, `shells/desktop/`, multi-crate, foundational (`ph2d-core`/`ecs`/`host`/`tokens`/`a11y`) | fmt + typos + workspace clippy + workspace nextest + doctests | ~3-5min |

O hook está em `scripts/pre-commit.sh`, instalado em
`.git/hooks/pre-commit`. Dispara automático em `git commit`.

**Janela vulnerável a colisão entre sessões:** o tempo entre seu
`git add` e o fim do `git commit` (incluindo o hook) é janela onde
outra sessão Claude paralela que rode `git commit` pode agarrar
seus arquivos staged junto. Coordene via Enio antes de commitar
algo que dispare T2.

**Bypass** (`--no-verify`): pula tudo. Use APÓS validação manual
com `cargo check/clippy/nextest -p <crate>`. Comum em iteração
rápida de uma só pasta. Em pré-push final (você, PRCI), evite.

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

## 6. Reporte ao Enio + entrada no babysit

Formato exato:

```
PR aberto: <URL retornado pelo gh pr create>
CI run:    https://github.com/dibrioli/PH2D/actions/runs/<id>

Entrando em modo babysit da CI (polling a cada 15min). Reporto
quando concluir success ou após 3 ciclos de falha do mesmo job.
```

Agora vá direto pra §7. Não pergunte "quer que eu acompanhe?" — o
modelo já te designou pra isso (CI ~30min é longo demais pro Enio
ficar de olho; é fim de jornada).

## 7. Modo babysit da CI

CI roda matrix completa (linux + macOS + windows + replay hash +
bench) e demora **~30min**. O push é fim de jornada diária; sua
responsabilidade agora é **ficar até a CI passar**, corrigindo
falhas que aparecerem.

### 7.1 Loop de polling — intervalo de 15min

Você pode usar `Monitor` com `sleep 900` (preferido — emite
notificação quando estado muda) OU `gh run watch <id>` (bloqueia
até a run terminar). Exemplo com Monitor:

```bash
bash -c '
RUN_ID=<id-da-run>
prev_st=""
prev_failed=""
while true; do
  out=$(gh run view "$RUN_ID" --json status,conclusion,jobs 2>&1)
  st=$(echo "$out" | jq -r ".status // \"unknown\"")
  cc=$(echo "$out" | jq -r ".conclusion // \"null\"")
  failed=$(echo "$out" | jq -r ".jobs[]? | select(.conclusion==\"failure\" or .conclusion==\"cancelled\") | .name + \"(\" + .conclusion + \")\"" | sort -u | paste -sd "," -)
  if [ "$st" != "$prev_st" ] || [ "$failed" != "$prev_failed" ]; then
    if [ -n "$failed" ]; then
      echo "[$(date +%H:%M:%S)] status=$st conclusion=$cc FAILED=$failed"
    else
      echo "[$(date +%H:%M:%S)] status=$st conclusion=$cc"
    fi
    prev_st="$st"; prev_failed="$failed"
  fi
  if [ "$st" = "completed" ]; then
    echo "[$(date +%H:%M:%S)] DONE conclusion=$cc"
    break
  fi
  sleep 900
done
'
```

Quinze minutos é o sweet-spot: CI evolui o suficiente entre
checks pra você ter sinal novo, e você não queima contexto LLM
fazendo polling de minuto em minuto.

### 7.2 Cenários e respostas

**Cenário A — CI termina success:**
```
✓ CI conclui success em <duração>. Run: <URL>
Modo babysit fechado. Disponível para próxima ordem.
```

Aí PRCI vai pra modo de espera (próxima jornada).

**Cenário B — Falha em algum job:**

1. **Diagnostique a causa raiz:**
   ```bash
   gh run view <run-id>                                      # overview
   gh run view --job=<job-id-falho>                          # detalhes
   gh api repos/dibrioli/PH2D/actions/jobs/<job-id>/logs \
     | grep -iE "error|FAIL|panic|test result.*failed" | tail -30
   ```

2. **Aplique fix mínimo localmente:**
   - Typo / fmt / clippy → corrigir o ponto exato; commit T0/T1.
   - Teste flaky em runner específico (mac/windows) sem mudança
     de código → `gh run rerun <run-id> --failed`; aguarde nova
     run; volte ao polling.
   - Test real → corrigir ou whitelist em `.typos.toml`;
     commit no tier apropriado.
   - Erro em config / CI / Cargo → ajuste cirúrgico; commit T2.

3. **Não simule fix sem entender:** se o erro é misterioso
   (ex: "unexpected argument 'check' found / rustup-init"),
   PARE e use as ferramentas de diagnóstico. Não force `--no-verify`
   nem `git push --force` na esperança de "limpar".

4. **Re-push:**
   ```bash
   git push origin <branch>
   # ou (raro, só em main com colisão já documentada):
   git push --force-with-lease origin main
   ```

5. **Cancele a run antiga** se ainda tem jobs in-progress
   pendurados:
   ```bash
   gh run cancel <run-id-antiga>
   ```
   Isso economiza minutos de runner.

6. **Pegue o novo run id** e volte ao polling §7.1.

**Cenário C — Falha repetida no mesmo job 3× consecutivas:**

```
✗ CI falhou 3× consecutivas no job <nome>. Erros idênticos.
Run histórico:
- Run #1: <URL>
- Run #2: <URL>
- Run #3: <URL>

Diagnóstico: <causa raiz>
Fix tentado: <descrição>
Por que não funciona: <hipótese>

Escalando — preciso de orientação do Enio.
```

Aí PRCI **PARA**. Não tente uma 4ª vez sem input do Enio.

### 7.3 O que conta como "ciclo de falha"

- **Falha de código** (clippy / fmt / test / lint) = ciclo conta.
- **Falha de infra do runner** (cache restore, network, "rustup-init
  unexpected argument", post-job cleanup) = NÃO conta. Re-rode o
  job (`gh run rerun --failed`) e siga o polling — flaky de infra
  é comum e auto-resolve.
- **Falha cancelada por você** (rerun → run antiga vira cancelled)
  = NÃO conta.

A escalação de 3 ciclos é pra **falhas reais de código** que você
tentou corrigir e ainda falham.

### 7.4 Quando o modo babysit termina

| Termina | O que faz |
|---|---|
| CI success | Reporta + entra em modo espera (próxima jornada) |
| 3 ciclos de falha real | Escala pro Enio + entra em modo espera |
| Enio explicitamente cancela ("para de babysit") | Reporta status atual + entra em modo espera |
| Working day acabou pro Enio | Continua babysit em background; reporta o que aconteceu no início da próxima sessão |

## 8. Regras de ouro

- **Nunca skipe hooks** (`--no-verify`) sem aprovação explícita
  E sem ter feito validação manual equivalente (`cargo check/
  clippy/nextest -p <crate>` no crate tocado).
- **Nunca force push** em main/master, EXCETO em §7.2 cenário B
  quando você documentou explicitamente o "force-with-lease" no
  fix loop e ninguém baseou trabalho em cima do SHA ruim.
- **Polle CI APENAS no modo babysit (§7).** Fora do babysit, não.
- **Nunca commite código não-relacionado** ao fix que está
  aplicando. Cada commit faz uma coisa só (typo → 1 commit,
  flaky-job-only → rerun sem commit).
- **Nunca rode `git config`** mudando settings globais ou do repo.

## 9. Sintomas de colisão entre sessões — diagnóstico

Antes de cada `git add` / `git commit`, rode `git status` e
`git status --cached`. Se algo nesses outputs te surpreender, é
provavelmente colisão.

| Sintoma | Causa | Recuperação |
|---|---|---|
| `fatal: cannot lock ref 'HEAD': is at X but expected Y` no `git commit` | Outra sessão fez commit no meio do seu (durante hook ou na janela stage→commit) | `git status`. Se working tree clean = arquivos seus já foram parar no commit do outro (vide próxima linha). Se staged ainda lá = re-tente commit, vai dar fast-forward. |
| `git log -1` mostra mensagem fundida (dois títulos colados, corpo truncado, dois `Co-Authored-By`) | Sua sessão e outra commitaram em paralelo, segundo `git commit` vacuou o índice global todo | Se NÃO pushado: `git reset --soft HEAD~1` → `git restore --staged <não-meus>` → re-commit com mensagem limpa → re-commit deles separado. Se pushado: avalie `git push --force-with-lease` IFF ninguém baseou trabalho em cima do SHA ruim. Coordene com Enio. |
| `git status` mostra `M`/`??` que você não tocou | Outro agente paralelo na mesma working tree fez mudanças | Não comite. Reporte ao Enio pra serializar. |
| Hook do pre-commit roda T2 (~5min) num commit que você esperava T1 (~30s) | Provavelmente arquivos staged de outro agente entraram pelo seu `git add` global ou pela janela vulnerável | Cancele com Ctrl+C. `git restore --staged <não-meus>` e re-comite. |

Referência adicional: memória da sessão LLM em
`feedback_parallel_agent_collision.md` (não está no repo, mora
no `~/.claude/projects/.../memory/`).

## 10. Tom de comunicação

- pt-BR direto, conciso. Sem hedging.
- Reporte resultados, não atividade ("PR aberto: link" > "Tentando
  abrir PR...").
- Sem emojis em mensagens nem em PR body (a não ser o marker
  `🤖 Generated with [Claude Code]` que CLAUDE.md autoriza).
- Erros: causa raiz, não sintoma.
