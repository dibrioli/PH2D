# HANDOFF — bootstrap da máquina Linux (dev rápido)

> **Para o Claude Code rodando no VS Code do PC Linux.** Objetivo: deixar este clone
> **idêntico** ao Mac e pronto para dev. Execute os passos na ordem; PARE onde indicado.
> Detalhe/racional completo em [`MULTI_MACHINE_SETUP.md`](MULTI_MACHINE_SETUP.md) (leia se algo falhar).

## Pré-condições (confirme primeiro)
```bash
pwd                         # deve ser a raiz do clone do PH2D
git remote -v               # origin = https://github.com/dibrioli/PH2D.git
git pull                    # traga o HEAD mais novo
git rev-parse --short HEAD  # anote; deve bater com o Mac
```
Se `pwd` não é o repo: `git clone https://github.com/dibrioli/PH2D.git` num disco **local NVMe**
(não em drive externo), `cd` nele, e recomece.

## 1. Toolchain (rustc idêntico)
```bash
rustup show                 # rust-toolchain.toml auto-instala 1.95 + clippy/rustfmt/rust-src
```
O host target linux é auto-instalado. **Não edite** `rust-toolchain.toml`.

## 2. Linker rápido (mold) — GLOBAL, nunca no repo
```bash
which mold || sudo apt install -y mold clang     # ou o pkg manager da distro
mkdir -p ~/.cargo
grep -q "fuse-ld=mold" ~/.cargo/config.toml 2>/dev/null || cat >> ~/.cargo/config.toml <<'EOF'

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF
```
⚠️ **NUNCA** ponha linker no `.cargo/config.toml` DO REPO — há cerca de Chesterton lá (pinar path
de linker no repo já quebrou o CI do Mac). Linker é per-usuário/global.

## 3. Memória do Claude — symlink para a pasta versionada
A memória vive em `project-memory/` no repo; ligue o dir que o Claude usa a ela:
```bash
KEY=$(pwd | sed 's:/:-:g')
mkdir -p ~/.claude/projects/$KEY
rm -rf ~/.claude/projects/$KEY/memory
ln -s "$(pwd)/project-memory" ~/.claude/projects/$KEY/memory
ls -l ~/.claude/projects/$KEY/memory          # deve ser symlink -> project-memory
head -1 ~/.claude/projects/$KEY/memory/MEMORY.md   # deve imprimir o índice
```
Depois disso, reinicie o Claude Code para ele carregar a memória sincronizada.

## 4. Secrets (gitignored — NÃO vêm no clone) — PARE e peça o zip ao Enio
O único secret (`docs/_api-claude.md`, chave da API Anthropic lida em debug builds) vem num zip
que o Enio te passa: **`ph2d-secrets.zip`**. PARE, peça esse arquivo, e extraia a partir da raiz
do repo (ele já contém o caminho correto):
```bash
unzip -o /caminho/para/ph2d-secrets.zip     # recria docs/_api-claude.md no lugar
test -f docs/_api-claude.md && echo "secret OK"
```
**Não invente valores.** Sem o zip, não prossiga para builds de debug que leem a chave.

## 5. Baseline compila
```bash
cargo check --workspace 2>&1 | tail -5        # verde?
```

## 6. Rebaseline de velocidade (128 GB RAM — informe o Enio, não mude nada ainda)
O teto de "≤3 cargos simultâneos" e o bloqueio de rust-analyzer/LSP (DIRETRIZ §6.6) eram pelos
**8 GiB do Mac**. Com 128 GB, rust-analyzer full + nextest paralelo + mais slots passam a valer.
Reporte ao Enio que isso pode ser reativado; não altere configs de velocidade sem o OK dele.
Slots CoW (`scripts/slot-seed.sh`) usam clone APFS (Mac) — no Linux dependem de reflink (Btrfs/XFS);
com 128 GB talvez nem precise de slots. Reveja antes de usar.

## Checklist final "está idêntico?"
- [ ] `git rev-parse HEAD` == Mac.
- [ ] `git status` limpo (sem CRLF-churn — `.gitattributes` já no repo).
- [ ] `rustup show` → 1.95.
- [ ] `~/.claude/projects/<key>/memory` é symlink → `project-memory/`; `MEMORY.md` lê.
- [ ] Secrets criados (`docs/_api-claude.md`, `.env`).
- [ ] `cargo check --workspace` verde.

## Regras do projeto (valem aqui igual)
Leia [`CLAUDE.md`](../../CLAUDE.md) (roteador) antes de codar. Git anti-colisão: `git add -- <seus paths>`,
nunca `-A`/`git add .`. Fast-mode: commit local; push só quando o Enio mandar.
Próximo trabalho técnico previsto: implementação do **Deform** — handoff em
[`docs/Deform/HANDOFF_deform_impl.md`](../Deform/HANDOFF_deform_impl.md).
