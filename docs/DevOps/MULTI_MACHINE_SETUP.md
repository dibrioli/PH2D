# Multi-máquina — projeto idêntico em Mac · Linux · Windows

> Runbook para manter **o mesmo PH2D idêntico** em três máquinas **sem perda de informação**
> (código, docs, ADRs, **memória do Claude**, tokens, handoffs). Escrito 2026-07-04.
> Papéis das máquinas: **Linux (128 GB RAM) = desenvolvimento rápido** · **Mac mini = testes da
> game engine (Metal/Apple)** · **Notebook Windows = build/teste do target Windows**.

---

## 1. Arquitetura de sincronização (a regra de ouro)

**GitHub é a fonte única de verdade. Cada máquina tem seu PRÓPRIO clone local nativo.**

```
                    ┌──────────────────────────┐
                    │  GitHub  dibrioli/PH2D    │  ← hub (tudo tracked)
                    └────────────┬─────────────┘
          git pull/push  ┌───────┼────────┐  git pull/push
                    ┌─────▼───┐ ┌─▼──────┐ ┌▼─────────┐
                    │ Linux   │ │ Mac    │ │ Windows  │
                    │ NVMe    │ │ mini   │ │ SSD      │
                    │ (dev)   │ │ (test) │ │ (build)  │
                    └─────────┘ └────────┘ └──────────┘
```

- ❌ **NÃO** compartilhar o mesmo drive externo entre máquinas. APFS (Mac) não é lido/escrito
  nativamente por Linux/Windows; e um único drive derrota o objetivo de clones nativos rápidos.
  O beast Linux precisa de **NVMe local** (o external USB seria o gargalo justamente na máquina
  que deve voar).
- ✅ Cada máquina: `git clone https://github.com/dibrioli/PH2D.git` num disco **local**.
- ✅ Sincronizar = `git pull` (início) / `git push` (fim). Nada de copiar pastas à mão para código.

---

## 2. O que sincroniza como (mapa completo — nada fica de fora)

| Categoria | Como viaja | Observação |
|---|---|---|
| **Código, docs, ADRs, tokens, handoffs, `.vscode/` compartilhado** | **git** (tracked) | idêntico por construção após `pull` |
| **`backups/`** | **git** (gitignored mas **já commitado**) | vem no clone; lint pula por gitignore |
| **Memória do Claude** (93 `.md` + `MEMORY.md`) | **git via relocação + symlink** (§4) | hoje fora do repo — **maior risco**, resolver primeiro |
| **Secrets** (`docs/_api-claude.md`, `.env`, `.env.local`) | **fora do git** — cópia segura (§5) | gitignored; NUNCA commitar |
| **`reference/`** (Blender GPL vendored, clean-room) | **fora do git** — cópia manual se precisar | gitignored (proprietário não commita GPL); só estudo, não é build-required |
| **`target/`, `target-slots/`, `.claude/` (worktrees), `~/.cargo` linker** | **NÃO sincroniza** — per-máquina | recompila/reconfigura local |
| **`assets/sprites/*.png`** | **NÃO** — auto-gerado determinístico | regenera no 1º launch |

---

## 3. Setup por máquina (bootstrap one-time)

### 3.1 Comum às três
```bash
git clone https://github.com/dibrioli/PH2D.git
cd PH2D
rustup show                      # confirma toolchain 1.95 (rust-toolchain.toml auto-instala)
cargo check --workspace 2>&1 | tail -5   # baseline compila?
```
`rust-toolchain.toml` já pina `1.95` + componentes → **rustc idêntico nas 3**. O `targets` lista
só Apple+wasm; o **host target nativo é auto-instalado pelo rustup**, então build nativo funciona
em Linux/Windows sem editar o arquivo. (Opcional: adicionar `x86_64-unknown-linux-gnu` /
`x86_64-pc-windows-msvc` se for cross-compilar.)

### 3.2 Linux (dev rápido — desktop 128 GB / 32 threads = tier `workstation`)

Este é o tier **`workstation`** ([ADR-0104](../architecture/decisions/0104-hardware-tiered-speed-strategy.md)):
a estratégia de velocidade dos 8 GiB do Mac fica **sobrescrita**. Confirme o tier:
```bash
bash scripts/hw-profile.sh          # deve imprimir: tier = workstation
```

**Deps de sistema** (exemplo Arch/CachyOS — `pacman`; em Debian/Ubuntu troque por `apt`):
```bash
sudo pacman -S --needed mold clang meson ninja nasm cmake pkgconf sccache
#   mold  = linker ELF rápido      meson/ninja/nasm/cmake = build nativo do stack AVIF
#   (dav1d/rav1e/libavif)          sccache = cache de compilação transparente
```

**Linker mold + sccache — GLOBAL no usuário, NUNCA no repo:**
```bash
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml <<'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[build]
rustc-wrapper = "sccache"      # transparente, determinístico — NÃO fere paridade-CI do ship.sh

[env]
SCCACHE_CACHE_SIZE = "30G"
EOF
```
- ⚠️ **NUNCA** no `.cargo/config.toml` DO REPO — cerca de Chesterton: pinar linker/target-dir no repo
  já **quebrou o CI do Mac**. Linker/wrapper são per-usuário/global.

**`target/` em tmpfs (RAM-disk) — grande ganho de link/IO:**
```bash
bash scripts/target-on-tmpfs.sh     # move target/ -> /dev/shm, imprime a regra de reboot-safety
# reboot-safety (uma vez, o script imprime o comando exato):
echo 'd /dev/shm/ph2d-target 0755 <user> <user> -' | sudo tee /etc/tmpfiles.d/ph2d-ramtarget.conf
sudo systemd-tmpfiles --create /etc/tmpfiles.d/ph2d-ramtarget.conf
```
- **Por quê a regra tmpfiles.d:** o `/dev/shm` esvazia no boot → o symlink `target/` fica pendurado →
  `cargo` (`create_dir_all`) **NÃO** cura symlink pendurado (`mkdir -p` retorna EEXIST). A regra
  recria o dir no boot. O sccache (cache em disco, sobrevive reboot) repovoa o target pós-reboot.

**rust-analyzer como oráculo** (era RAM-blocked no Mac): instale a extensão + configure nas **User
settings** do VS Code (não no `.vscode/` do repo, senão liga RA pesado no Mac de 8 GiB):
```bash
code --install-extension rust-lang.rust-analyzer
```
```jsonc
// ~/.config/Code/User/settings.json
"rust-analyzer.check.command": "clippy",
"rust-analyzer.cargo.targetDir": true,     // target-dir próprio — não briga pelo lock com os cargos dos agentes
"rust-analyzer.check.workspace": true
```

- **Slots CoW = opcionais aqui.** `slot-seed.sh` usa clone APFS (macOS); no Linux depende de reflink
  (Btrfs/XFS — este FS é Btrfs, tem). Com 128 GB + `target/` em tmpfs, um `target/` único já basta;
  slots só se rodar builds de fato paralelos que briguem pelo lock.
- **`target-cpu=native` fica OFF** mesmo aqui (diverge do cache do CI; zero ganho no check) — ADR-0104.

### 3.3 Windows (build/teste target Windows)
```powershell
git config --global core.longpaths true    # paths Rust profundos estouram MAX_PATH
git config --global core.autocrlf false     # deixa o .gitattributes (§6) mandar
```
- Linker já resolvido: `.cargo/config.toml` do repo já usa `rust-lld.exe` p/ `x86_64-pc-windows-msvc`
  (target-gated, seguro). Precisa das **Build Tools do MSVC** instaladas.
- Symlinks (p/ a memória, §4) exigem **Developer Mode ON** ou terminal admin (`mklink /D`).

### 3.4 Mac mini (testes da game engine — Metal)
- Já configurado. Linker lld global no `~/.cargo/config.toml` do usuário (não no repo). Mantém.
- Papel = rodar os testes GPU/Metal `--features gpu -- --ignored` (headless, ver memória
  `reference_gpu_tests_run_headless_metal`) e o smoke visual do Enio.
- Decisão: manter o clone no **drive externo** (ok p/ testes) ou mover p/ disco interno (mais rápido).
  Para "idêntico", o que importa é o git — o local físico é indiferente.

---

## 4. Memória do Claude — relocar para o git (resolve o maior risco)

**Problema:** a memória vive em `~/.claude/projects/<KEY>/memory/`, onde `<KEY>` = caminho absoluto
do projeto com `/`→`-`. Cada máquina tem `<KEY>` diferente → a memória **não** é encontrada
cross-machine, e não está no repo nem no drive.

**Solução:** tornar a memória **versionada no git** e ligar o local que o Claude usa por **symlink**.
Assim: escrita do Claude → pasta no repo → `git push` → outras máquinas `pull` → memória idêntica,
com histórico, à prova de migração.

### 4.1 One-time NO MAC (origem — fazer com o índice do git livre; **sem agente em vôo**)
```bash
cd /Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva
KEY=-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva
SRC=~/.claude/projects/$KEY/memory

# 1) move a memória viva p/ dentro do repo (pasta NÃO gitignored)
mkdir -p project-memory
cp -a "$SRC"/. project-memory/

# 2) substitui o dir original por um symlink -> pasta do repo
rm -rf "$SRC"
ln -s "$(pwd)/project-memory" "$SRC"

# 3) commit (quando o Enio liberar) — escopado, isolado
git add -- project-memory/
git commit -m "chore(memory): vendorizar memória do Claude no repo p/ multi-máquina"
```
> Efeito: daqui pra frente o Claude lê/escreve em `project-memory/` via symlink; toda memória nova
> entra no git naturalmente. **Converta os links de `MEMORY.md`** de `file:///Users/.../.claude/...`
> para **repo-relativos** (`./project-memory/...`) para não quebrarem nas outras máquinas (cosmético).

### 4.2 Nas máquinas novas (Linux/Windows) após clonar
```bash
# Linux/macOS
KEY=$(pwd | sed 's:/:-:g')                       # a KEY desta máquina
mkdir -p ~/.claude/projects/$KEY
rm -rf ~/.claude/projects/$KEY/memory
ln -s "$(pwd)/project-memory" ~/.claude/projects/$KEY/memory
```
```powershell
# Windows (Dev Mode ON): a KEY usa o path desta máquina; ligar por junction/symlink
$key = (Get-Location).Path -replace '[:\\/]','-'
$dst = "$env:USERPROFILE\.claude\projects\$key"
New-Item -ItemType Directory -Force $dst | Out-Null
cmd /c rmdir "$dst\memory" 2>$null
cmd /c mklink /D "$dst\memory" "$(Get-Location)\project-memory"
```
Resultado: as três máquinas apontam o `memory` do Claude para a **mesma pasta versionada** →
memória idêntica, sincronizada por `git pull`/`push`.

*(Alternativa sem relocação, se preferir não versionar a memória: copiar `project-memory/` à mão
via pendrive/rsync a cada máquina. Funciona, mas sem histórico e propenso a drift — não recomendado.)*

---

## 5. Secrets (fora do git — transferir com segurança)

Estes são **gitignored de propósito** e **não** viajam pelo git. Transfira por gerenciador de
senhas / canal seguro (NÃO e-mail/commit):

| Arquivo | O que é |
|---|---|
| `docs/_api-claude.md` | chave da API Anthropic (dev/test, ADR-0061) — lida em debug pelo `ph2d-host-desktop` |
| `.env` / `.env.local` | env local |

Checklist por máquina nova: recriar esses arquivos **antes** de rodar debug builds que os leem.

---

## 6. Higiene cross-platform (para ser REALMENTE idêntico)

### 6.1 Adicionar `.gitattributes` (FALTA hoje — fazer como commit próprio)
Sem isso, o Windows converte para CRLF e o repo inteiro parece modificado + fura golden-hashes de
determinismo. Conteúdo recomendado na **raiz** do repo:
```gitattributes
* text=auto eol=lf
*.rs   text eol=lf
*.toml text eol=lf
*.md   text eol=lf
*.wgsl text eol=lf
*.sh   text eol=lf
*.command text eol=lf
*.png  binary
*.ktx2 binary
*.ico  binary
*.ttf  binary
*.otf  binary
```
Aplicar (uma vez, como commit dedicado, índice livre): criar o arquivo, depois
`git add --renormalize .` para normalizar o histórico de EOL. **Não** misturar com outro commit.

### 6.2 Config git por máquina
- Todas: `git config core.autocrlf false` (deixa o `.gitattributes` decidir).
- Windows: `git config core.longpaths true`.
- macOS: FS case-insensitive por padrão vs Linux case-sensitive — evitar arquivos que diferem só
  por maiúscula/minúscula (improvável aqui, mas fica o alerta).

### 6.3 Linker = per-usuário, nunca no repo
Reforço da cerca de Chesterton em `.cargo/config.toml`: Mac=lld, Linux=mold, Windows=rust-lld
(este último já target-gated no repo). Os dois primeiros vivem no `~/.cargo/config.toml` de cada
máquina (§3), fora do git.

---

## 7. Fluxo de trabalho diário (evita divergência entre máquinas)

1. **Início de sessão (qualquer máquina):** `git pull` antes de tocar em qualquer coisa.
2. **Dev acontece no Linux** (rápido). Commits locais no fast-mode; push no fim (ou o Coordenador).
3. **Mac** faz `git pull` → roda testes de game engine / Metal / smoke visual → se precisar de fix
   Mac-específico, commita e push de volta.
4. **Windows** faz `git pull` → build/teste do target Windows.
5. **Fim de jornada:** um `git push` por máquina que trabalhou. Nunca deixe commits locais órfãos
   numa máquina — senão as três divergem.
6. **Uma máquina "dona" por vez** de um trabalho (o modelo Coordenador/Implementador já assume
   isso). Não editar o mesmo módulo simultaneamente em duas máquinas sem push/pull entre elas.

> Regra prática: **memória e código só ficam idênticos se toda máquina fizer `pull` no início e
> `push` no fim.** O symlink da memória (§4) faz a memória seguir esse mesmo trilho do git.

---

## 8. Checklist "está idêntico?" (rodar após setup de cada máquina)
- [ ] `git rev-parse HEAD` igual nas três (após pull).
- [ ] `git status` limpo (sem CRLF-churn → `.gitattributes` aplicado).
- [ ] `rustup show` → toolchain 1.95 nas três.
- [ ] `ls -l ~/.claude/projects/<KEY>/memory` → é **symlink** para `project-memory/` do repo.
- [ ] `cat project-memory/MEMORY.md` → mesmo conteúdo (memória sincronizada).
- [ ] Secrets recriados (`docs/_api-claude.md`, `.env`) onde o debug precisa.
- [ ] `cargo check --workspace` verde.
- [ ] Smoke: `./play.command` (Mac) / equivalente por OS.

---

## 9. Ordem de execução recomendada (quando o Enio liberar; **sem agente em vôo**)
1. **Commitar o que está untracked e é importante** (índice livre, escopado): `docs/Deform/`,
   `docs/DevOps/`, e a relocação da memória (§4.1). Push.
2. **Adicionar `.gitattributes`** como commit dedicado + `git add --renormalize .`. Push.
3. **Linux:** clone + §3.2 + §4.2 + secrets. Rebaseline de velocidade.
4. **Windows:** clone + §3.3 + §4.2 + secrets.
5. Rodar o §8 nas três.

Até lá: **nada é commitado** (há agente trabalhando; pedido do Enio). Este runbook é só o plano.
