# ARQUIVO — DIRETRIZ.md (história, 161 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`DIRETRIZ.md`](../../IntegracaoMultiAgente/DIRETRIZ.md) em 2026-08-18, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> ⚠️ **Os links relativos do CORPO abaixo apontam a partir de `docs/IntegracaoMultiAgente/`, não
> daqui** — o corpo é verbatim, e mover o texto uma pasta para baixo não reescreve os `../`. Um
> `../SESSION_ACTIVE.md` daqui é `docs/SESSION_ACTIVE.md`. É consequência direta da garantia de
> ser verbatim: preferimos o texto intacto ao link resolvido, e os gates de doc não varrem
> `docs/archive/` por isso.
>
> **Conteúdo:** três recortes do doc vivo, na ordem original — **§1 + §1.1–§1.4** (papéis
> Coordenador/Implementador e a infra anti-colisão do **Modo C**) · **§6.6.B** (a pesquisa de
> alavancas de build de 2026-05-29, cujo veredito é sobre um **Mac de 8 GiB** — `ld64.lld`,
> `/opt/homebrew`, *"mold é incompatível"* — e portanto diz o oposto do que vale numa workstation
> Linux) · **§7.1–§7.4** (o protocolo git de shared tree, que o `git worktree` do Modo L torna
> desnecessário). **A numeração das seções foi preservada:** o doc vivo continua a citá-las por
> esses números e é aqui que elas estão.
>
> Recorte: linhas fora de `1-62,116-901,958-965,1015,1019-1261` do original.

---

## 1. Papéis + infra multi-agente

> **Todo o §1 (papéis abaixo) + §1.1–1.4 descrevem o Modo C** (tier `constrained` — Mac mini
> 8 GiB): o "Coordenador (único)" só existe no Modo C. No tier `workstation` vale o **Modo L
> (§1.5)** — SEM Coordenador de plantão; a infra anti-colisão (slots CoW, arbitragem de posse,
> índice compartilhado) é substituída por isolamento físico de `git worktree`.

**Coordenador (único).** Um só por jornada. Absorve o que antes eram Coord-A (foundational) e Coord-B (baldes). Autoridade **exclusiva** sobre: contratos congelados, arch-gates, foundational crates (`ph2d-render`, `ph2d-editor-core`, `ph2d-host`, `ph2d-tokens`, …), codegen tools, `shells/*` plumbing compartilhado, scaffolds de painel/widget/chrome, ADRs, `CLAUDE.md`/DIRETRIZ, `.github/workflows/`. É o **único** que toca arquivo foundational/compartilhado — isso serializa a superfície de colisão (causa-raiz dos incidentes que motivaram o modelo). Mexe nos 2 contratos congelados (§4) só via amendment ADR, nunca cap-bust ad-hoc. Responsabilidades do modelo multi-implementador:
- (a) escrever um **sub-handoff focado por implementador** (estado + pasta exclusiva + task + anti-colisão);
- (b) manter o **mapa de posse** em SESSION_ACTIVE (§1.1) — quem é dono de quê;
- (c) **arbitrar colisões** e **sequenciar dependências** entre implementadores (ex.: liberar `ph2d-render` ao módulo B só quando o módulo A soltar);
- (d) **ship-de-jornada** (ship.sh + commit + push + babysit CI — §8), incluindo limpar fmt-drift e ship-blockers cross-session no fim.

Não implementa feature de módulo — **coordena**.

**Implementador (sempre vários).** Sessão isolada, **uma por módulo físicamente disjunto** (uma crate-pasta ou um cluster de crates do mesmo módulo). Caminho **(A)**: cria pasta + roda sync + testa, sem Coordenador. Caminho **(D)**: edita dentro de pasta de módulo existente. Caminho **(B)**: recebe pasta já scaffoldada pelo Coordenador, edita **só** dentro dela. A não-colisão é garantida pela arquitetura física (glob `workspace.members` + codegen splice em marcadores) **somada** à regra de posse exclusiva arbitrada pelo Coordenador. **Precisou de QUALQUER coisa fora da sua pasta** (foundational, shell plumbing, contrato congelado, outro módulo)? **PARA e reporta ao Coordenador** — não edita, e **nunca renegocia direto com outro implementador**.

**Enio.** Humano que orquestra: abre sessões Claude Code, cola mensagens entre elas, roda smoke visual quando Coord pede. **Não decide nada operacional.**

### 1.1 Protocolo SESSION_ACTIVE (mapa de posse mantido pelo Coordenador)

[`docs/SESSION_ACTIVE.md`](../SESSION_ACTIVE.md) é o post-it vivo da orquestração. **Só o Coordenador escreve;** os implementadores **leem antes de cada burst** e não editam. O Coordenador mantém ali:

1. O **mapa de posse**: qual implementador é dono de qual pasta/módulo (escrita exclusiva) + seu slot.
2. Os **pontos compartilhados** e como estão resolvidos (ex.: crate X é escrita do Impl-N, leitura dos demais).
3. Os **itens que o Coordenador segura** (ship-blockers, foundational, sequenciamento de dependências).
4. **Pre-existing failures cross-session** a NÃO fixar (com owner identificado).

Implementador que precise tocar pasta fora da sua: **PARA e reporta ao Coordenador** — nunca renegocia direto com outro implementador. O Coordenador limpa os itens concluídos ao encerrar a jornada.

### 1.2 Isolamento físico — `scripts/slot-env.sh`

Cada sessão roda `source scripts/slot-env.sh <slot-id>` no início para isolar `CARGO_TARGET_DIR` por slot. Sem isso, dois agentes paralelos serializam no lock de `target/`. Slot IDs: `coord` + um por implementador nomeado pelo módulo (`impl-sprite`, `impl-painter`, `impl-vector`, …).

**RAM 8 GiB → máximo realista = 2-3 slots cargo-ativos simultâneos.** Com N implementadores, isso NÃO autoriza N cargos simultâneos: o Coordenador **escalona quem compila quando** (lê SESSION_ACTIVE). 4º cargo ativo causa swap thrashing.

*(Modo L: slots dispensáveis — cada worktree já tem `target/` próprio; vide §1.5.1.)*

### 1.3 Anti-colisão git — `scripts/git-stage-guard.sh`

Pre-commit roda o guard que **rejeita stage fora da pasta declarada** (env `PH2D_SLOT_FOLDER`). Coords legítimos exportam `COORD_OVERRIDE=1` na sessão pra bypass. Padroniza a disciplina §7 sem depender de memória humana.

### 1.4 As 3 obrigações do Implementador (sempre)

1. **ISOLAMENTO.** Edita **só** dentro da pasta exclusiva. Precisa algo fora? **Reporta** — não edita.
   ⚠️ **Isto é Modo C.** No **Modo L** a obrigação 1 **não** proíbe foundational: ele é editável pela
   sua linha sob o protocolo testado — **§1.5.2.1** (ADR-0107). Só contrato congelado (§4) e
   mesmo-símbolo de tipo-núcleo continuam fora de limite nos dois modos.
2. **UI canônica.** Toda cor/espaço/raio/tipografia/stroke passa por tokens. Zero hex, zero `f32` literal de UI (§5).
3. **Codificação rápida.** `cargo check -p <crate>` no editing burst. Sem `--workspace` em loop (§6).

Pra violar uma? **Pare e reporte.** Quase certo o Coord não fez scaffold direito.

#### B. Alavancas (deep-research 2026-05-29, verificada 3-votos; run `wf_8d23212a-39e`)

Separadas por confiança. Linux-benchmarks **não transferem direto** pro Apple Silicon
8 GiB → as marcadas "pilotar" exigem medição local antes de virar mandato.

**🥇 Tier 1 — diagnóstico via LSP (MEDIDO 2026-05-29 — leia o veredito):**
- **Diagnóstico via LSP, não saída crua.** O agente NÃO deve ler output bruto do cargo
  e adivinhar tipos (desperdício de token + erro).
- **⛔ FULL rust-analyzer / MCP type-query = BLOQUEADO por RAM nesta máquina.** Spike de
  medição: 8 GiB físicos, **swap 5187/6144 MiB usados, ~89 MiB livres** com editores +
  1 agente e os rust-analyzers dos editores **dormentes (3 MiB)**. Um RA *indexando* o
  workspace (~30 crates wgpu/vello/bevy) custa ~1.5–4 GB → **não cabe nem ×1, quanto mais
  ×3**. Só viável num Mac de 32 GB (dispensado). NÃO adotar rust-analyzer-as-oracle / MCP
  de tipo aqui.
- **✅ Caminho viável nesta máquina = `scripts/cargo-check-narrow.sh` ON-DEMAND.** O
  agente checa quando quer, recebe só os erros (corta tokens), **zero processo residente**.
  É o Tier-1 prático no teto de 8 GiB.
- **⚠️ `bacon-ls` (backend cargo) — pesar, não adotar cego:** dá diagnósticos via LSP sem
  o índice do RA, MAS roda `cargo check`/clippy **continuamente** em background; com ≤3
  agentes = 3 loops de check contínuos = pressão constante numa máquina já em swap. Pode
  ser PIOR que o check on-demand. Só vale se medido folgado.
  *(MCP de terceiros rust-mcp/cursor-rust-tools = EXPERIMENTAL hobby E type-query = RAM-blocked.)*

**🥈 Tier 2 — build/test loop (PILOTADO 2026-05-29: já capturado, ver status):**
- **Linker rápido = ✅ JÁ ATIVO.** `~/.cargo/config.toml` global usa
  `-fuse-ld=/opt/homebrew/bin/ld64.lld` (lld para Mach-O) — corta ~30-50% do link
  incremental. `mold` é **incompatível com macOS** (ELF-only, erro fatal) — não usar.
- **Redução de debug-info = ✅ JÁ NO GATE.** `[profile.ci-test] debug = false`, e o gate
  (`nextest-impacted.sh` + `ship.sh`) roda `--cargo-profile ci-test` → debug-info já
  cortado onde importa. ⚠️ **E a frase que estava aqui — *"o `[profile.dev] debug = true` só afeta
  `cargo check` (irrelevante — não linka) e builds ad-hoc (que evitamos)"* — era FALSA, medida em
  2026-08-16:** o `cargo test -p`, que é o gate de fechamento que a CLAUDE.md §2 prescreve, LINKA
  binários de teste sob o `dev` — e eles somavam **8,3 GB em 40 binários, 208 MB cada**, no target
  do primário. Não eram *ad-hoc*: eram o fechamento. A cura shipou como
  `split-debuginfo = "unpacked"` no `[profile.dev]` (**2,5× medido**, A/B na
  [`DIRETIVA_FIM_DE_DIA.md`](DIRETIVA_FIM_DE_DIA.md) §2-bis Regra 3), e o preço é pequeno porque a
  `.debug_line` sobrevive no binário — o backtrace mantém `file:line`.
- **`prefer-dynamic` (dynamic-linking) = ❌ NÃO adotar.** Só ajuda LINK (gate infrequente),
  não o inner loop (check). Com lld + debug=false já ativos, o link deixou de ser dominante
  → ganho marginal. Custo: mudar RUSTFLAGS invalida a **base CoW warm** (rebuild completo) +
  quirks de prefer-dynamic no macOS. Net-negativo nesta máquina. (O ~5× do `bevy_dylib` é da
  feature whole-Bevy, que não usamos — só `bevy_ecs` standalone.)

**🥉 Tier 3 — pilotar + medir (ganho real M-series incerto):**
- **`cargo-hakari`** (workspace-hack): mata a **cascata de recompile por feature-unification**
  ("check ganha mais que build"; até 100× em comando isolado, ~1.7× cumulativo em Linux).
  Custo: crate central novo (acoplamento leve) + entrada em `cargo-machete` ignore. Medir
  ganho no nosso slot CoW antes de adotar.

**🚫 NÃO fazer (achados contrários verificados):**
- **Cranelift:** irrelevante ao inner loop (check já não faz codegen); no macOS unwinding
  de panic **não-suportado** (força `-Cpanic=abort`) + `std::arch` SIMD parcial = ruim p/
  wgpu/vello/rapier. Experimental.
- **`mold`:** incompatível com macOS.
- **`-Zthreads`** (frontend paralelo): não-provado em RAM-bound; aumenta pico de memória.

## 7. Anti-colisão git (Modo C — shared tree)

> **Modo L:** esta seção inteira descreve o problema que o worktree elimina — cada linha tem
> índice/HEAD/tree próprios. No Modo L valem só as tabelas 1.5.5 (conflitos de merge) e
> 1.5.6 (proibições). No Mac (Modo C), esta seção vale INTEIRA.

`git commit` é serializado pelo índice global do git. Duas sessões com arquivos staged ao mesmo tempo: uma roda commit e agarra os arquivos da outra junto.

### 7.1 Protocolo atômico stage→commit

```bash
# 1) Antes de stage: confira working tree
git status
#    Há M/?? que não são seus? PARE. Outro agente em vôo.

# 2) Stage só os seus. NUNCA -A / -a / git add .
git add <arquivos-específicos>

# 3) Antes de commit: confere índice
git status --cached
#    Arquivo que não estagiou? Vazamento.
#    git restore --staged <não-meus>

# 4) Commit. Hook tiered roda automaticamente.
git commit -m "<descrição em inglês, imperativo, <70 char>"
```

Stage→commit é **operação contínua**. Não pause entre os dois passos.

### 7.2 Proibições

- **Nunca** `git push --force` em main
- **Nunca** `--no-verify` no commit de **SHIP** (ali o hook é o gate — se falha, fix root cause).
  Nos **checkpoints de dia** ele é o **padrão**, não a exceção (§8.1 fast mode)
- **Nunca** `git commit --amend` (sempre novo commit)
- **Nunca** `git config` mudando settings do repo
- **Nunca** `git restore --staged --worktree` em path fora da sua pasta sem coordenar

### 7.3 Sintomas de colisão

| Sintoma | Recuperação |
|---|---|
| `fatal: cannot lock ref 'HEAD'` no commit | Outra sessão commitou no meio. `git status` → diagnose |
| `git status` mostra M que você não tocou | Outro agente paralelo. NÃO comite. Reporte |
| `git log -1` mostra mensagem fundida (2 títulos) | Colisão. Se NÃO pushado: `git reset --soft HEAD~1` + split + recommit |
| Hook trigga T2 quando esperava T1 | `git status --cached` — vazamento de outro agente |

### 7.4 Armadilhas conhecidas


**Cargo lock entre sessões.** Se rodar `cargo` enquanto outra sessão Claude Code paralela está rodando, a 2ª **espera silenciosamente** pelo lock. Não é crash, só lentidão. Use `slot-env.sh` pra isolar (§1.2).

