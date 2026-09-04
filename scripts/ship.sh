#!/usr/bin/env bash
# ship.sh — local "is this CI-clean?" gate. Run ONCE before a push (or at
# the end-of-day "ship"), so the GitHub `spike.yml` lint + test jobs go
# GREEN on the first try.
#
# WHY THIS EXISTS: the local pre-commit hook (scripts/pre-commit.sh) does
# NOT match CI. The 2026-05-19 perf audit dropped `clippy --all-targets`
# from the T2 workspace tier, and the hook never ran `cargo machete` /
# `cargo deny` / `cargo audit`. So a change can pass every local commit
# and still redden CI (test-code clippy lints, unused deps, ...). This
# script runs the EXACT CI commands so there are no surprises at push.
# See docs/IntegracaoMultiAgente/DIRETRIZ.md §8 (fast mode / ship).
#
# Runs EVERY check (does not stop at the first failure), prints a summary,
# and exits non-zero if any failed. It does NOT commit or push — the agent
# does that AFTER this is green (then babysits CI; DIRETRIZ §8).

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2

fail=0
results=()

run() { # run "<label>" <cmd...>
    local label="$1"
    shift
    printf '\n\033[1m▶ %s\033[0m\n' "$label"
    if "$@"; then
        results+=("✓ $label")
    else
        results+=("✗ $label")
        fail=1
    fi
}

run_optional() { # run_optional "<label>" <tool-binary> <cmd...>
    local label="$1" tool="$2"
    shift 2
    if command -v "$tool" >/dev/null 2>&1; then
        run "$label" "$@"
    else
        results+=("⚠ $label — SKIPPED ($tool not installed; CI runs it · cargo install $tool)")
    fi
}

echo "ship.sh — mirroring spike.yml lint + test jobs (this compiles a lot; ~minutes)"

# ⚠️ PARIDADE COM O CI: o `spike.yml` põe `CARGO_BUILD_WARNINGS: deny` no `env` do
# workflow inteiro (2026-08-29, tarefa A8 de `docs/Atualizar Stack/`), então TODA
# corrida de cargo lá nega aviso — não só os passos de lint. Sem esta linha o ship
# ficava verde num aviso que o CI reprova, que é o modo de falha exacto que este
# script existe para prevenir.
# ⛔ NÃO troque por `RUSTFLAGS="-D warnings"`: aquele entra no fingerprint do cargo e
# recompila tudo (medido: 2 399 ms contra 105 ms num crate só). O `env` do spike.yml
# tem a tabela.
export CARGO_BUILD_WARNINGS="${CARGO_BUILD_WARNINGS:-deny}"

# ── CI `lint` job parity (spike.yml) ────────────────────────────────────
run "fmt --check" cargo fmt --all -- --check
run "clippy (workspace, all-targets, CI features)" \
    cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
run_optional "cargo-machete (unused deps)" cargo-machete cargo machete
run_optional "cargo-deny (licenses/advisories/bans/sources)" cargo-deny \
    cargo deny --all-features check
run_optional "cargo-audit (CVE scan)" cargo-audit cargo audit
# typos: CI's lint job runs a project-wide scan (config in .typos.toml).
# The pre-commit hook DOES run it, but a `--no-verify` fast-mode session
# bypasses the hook — so without this row a stray typo lands in CI lint
# red after the push. Same engine + config as spike.yml.
run_optional "typos (project-wide typo scan)" typos typos

# ── índices DERIVADOS: em dia? ──────────────────────────────────────────
# ⚠️ Estes índices existem porque a alternativa (lista mantida à mão) envelhece na
# primeira semana — foi o que aconteceu com a tabela "Estado por-wave" do tracker
# da física, que parou em 2026-07-20 enquanto o doc seguia até 08-15. Mas um
# gerador que ninguém invoca produz exatamente a mesma dívida: medido em
# 2026-08-18, `cargo-check-narrow.sh` está no CLAUDE.md §2 e foi chamado 5 vezes
# em 101 sessões, contra 13.791 `cargo check` à mão. Ferramenta só é adotada
# quando um passo escrito a chama pelo NOME — e este é o passo.
run "índice de ADRs em dia" bash scripts/adr-index.sh --check
# ⚠️ Mesmo argumento, outros 14 diretórios: o `docs/Motion Nodes/` tinha 99 arquivos
# e ZERO índice, e 45% dos markdowns do repo eram inalcançáveis a partir do roteador.
run "índices de docs em dia" bash scripts/doc-index.sh --check
for _a in docs/archive/*/; do
    [ -d "$_a" ] || continue
    # só as pastas que têm README derivado (as antigas, escritas à mão, ficam de fora)
    grep -q 'INDICE-DERIVADO' "${_a}README.md" 2>/dev/null || continue
    run "índice de $(basename "$_a") em dia" python3 scripts/archive-index.py "${_a%/}" --check
done

# ── CI `test` job parity (nextest covers arch gates + cook-hash) ─────────
# ⚠️ `CARGO_INCREMENTAL=0` nos DOIS: o perfil `ci-test` só roda em BATCH, então
# incremental não colhe nada ali e paga 11 GB (CLAUDE.md §2, medido 2026-08-16).
# O `env` é por-comando de propósito — o clippy acima roda no perfil `dev`, cujo
# `incremental/` é o que faz o `cargo check -p` do inner loop voar no dia
# seguinte; um `export` no topo o mataria junto. A §2 protege essa metade
# explicitamente ("o `cargo check -p` do inner loop fica em paz, de propósito").
if command -v cargo-nextest >/dev/null 2>&1; then
    run "nextest run --workspace (ci-test)" \
        env CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test
else
    run "cargo test --workspace (ci-test)" \
        env CARGO_INCREMENTAL=0 cargo test --workspace --profile ci-test
fi

# ── inventário de TETOS (informativo — NUNCA reprova) ────────────────────
# ⚠️ **Isto NÃO é um `run`, e a diferença é o ponto.** O `stack-audit.sh` sai `0`
# sempre — é uma sonda, não um portão —, então um `✓`/`✗` diria apenas «correu»
# e escondia a única coisa que interessa: **o que ele produziu**. É a memória
# `feedback_an_automatic_tools_exit_code_says_nothing_about_what_it_produced`
# (medida 3× num dia), e é por isso que a saída vai INTEIRA para o ecrã.
#
# ⚠️ E nunca pode reprovar: o ship não pode ficar vermelho porque o mundo
# publicou uma versão nova, nem porque a rede caiu.
#
# Por que AQUI: é o instante em que a pergunta importa — mesmo antes do
# «safe to push». Um teto responde *«o mais recente possível ≠ o mais
# recente»*, e quem for subir uma dependência amanhã lê esta lista hoje.
printf '\n\033[1m── tetos de dependência (informativo, não reprova) ──\033[0m\n'
_tetos="$(timeout 90 bash scripts/stack-audit.sh --tetos 2>/dev/null || true)"
if [ -n "$_tetos" ]; then
    printf '%s\n' "$_tetos" | sed 's/^/  /'
else
    printf '  (sem rede, ou 90 s esgotados — o inventário não foi lido desta vez)\n'
    printf '  para o ver:  bash scripts/stack-audit.sh --tetos\n'
fi

# ── sanidade da MÁQUINA (informativo — NUNCA reprova) ───────────────────
# ⚠️ **Por que aqui, e não «quando alguém lembrar»:** medido em 2026-09-01 sobre
# 239 209 comandos reais de 83 sessões, as sondas de saúde deste repo foram
# invocadas **177 vezes = 0,07%** — e o `ph2d-check-memoria`, escrito depois dos
# travamentos de 08/08 para exatamente este fim, **1 vez na vida**. As quatro
# ferramentas que sobrevivem aqui têm uma coisa só em comum: um passo
# obrigatório chama-as pelo nome. Esta é a lei do §2 do CLAUDE.md aplicada a si
# própria — *ponteiro não é adoção*.
#
# ⚠️ Informativo, como os tetos: uma máquina apertada não pode reprovar um push.
# O vigia de 15 min (`sanidade.sh --instalar`) é quem avisa a tempo; isto aqui é
# a rede para o caso de o timer estar desarmado nesta máquina.
printf '\n\033[1m── sanidade da máquina (informativo, não reprova) ──\033[0m\n'
if [ -x scripts/sanidade.sh ] || [ -f scripts/sanidade.sh ]; then
    _san="$(timeout 30 bash scripts/sanidade.sh 2>/dev/null || true)"
    if [ -n "$_san" ]; then printf '%s\n' "$_san"
    else printf '  (não correu — veja: bash scripts/sanidade.sh)\n'; fi
else
    printf '  (scripts/sanidade.sh ausente nesta árvore)\n'
fi

# ── summary ─────────────────────────────────────────────────────────────
printf '\n\033[1m── ship.sh summary ──\033[0m\n'
for r in "${results[@]}"; do printf '  %s\n' "$r"; done
if [ "$fail" -ne 0 ]; then
    printf '\n\033[1;31m✗ NOT CI-clean — fix the ✗ rows above, re-run, before pushing.\033[0m\n'
    exit 1
fi
printf '\n\033[1;32m✓ CI-clean. Safe to commit + push (then babysit CI).\033[0m\n'
