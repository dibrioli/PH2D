# HANDOFF — continuação da linha `line/Painter` (pós-integração 2026-07-11)

> **Para o próximo agente-de-linha.** A jornada anterior fechou e **já foi integrada ao `main`**.
> A worktree está **pronta e sincronizada** — você NÃO precisa rodar o setup do
> [`MODELO_ABERTURA_LINHA.md`](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) (já rodei).
> Leia §1 (regras do Modo L — **inegociáveis**) e §3 (backlog) e comece.

---

## 0. Estado — a linha está PRONTA

| Item | Estado |
|---|---|
| **Worktree** | `Worktrees/line-Painter/` — existe, limpa (0 arquivos sujos) |
| **Branch** | `line/Painter` |
| **HEAD** | `3805f650` — **idêntico ao `main`** (rebase feito; 0 commits fora do main) |
| **Integração anterior** | ✅ **concluída** — os 26 commits da jornada passada estão no main |
| **hw-profile** | `workstation` (32 cores, 123 GiB) ⇒ **Modo L confirmado** |
| **mergiraf** | configurado |
| **`target/`** | quente (`cargo check -p ph2d-core` = 0.57s) |
| **Sanidade pós-rebase** | `cargo check -p ph2d-tool-painter -p ph2d-painter-brush` ✅ |

**Sua 1ª ação:** leia §1, escolha do backlog (§3) e faça a TRIAGEM (DIRETRIZ §2). Nada de setup.

---

## 1. ⚠️ REGRAS DO MODO L — inegociáveis (do `MODELO_ABERTURA_LINHA.md`)

Estas valem até o fim da sessão, **sem exceção**:

- **A — Todo comando dentro da worktree.** TODO read/edit/git/cargo acontece em
  `Worktrees/line-Painter/`. A raiz do repo é o **checkout primário compartilhado**: o MESMO path
  relativo existe nas duas árvores — editar `crates/...` na raiz é editar a **árvore ERRADA**.
  **Na prática (isto MORDEU nesta linha, várias vezes):** o cwd do Bash **reseta pro repo MAIN a cada
  turno** ⇒ **prefixe `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter &&` em TODO
  comando Bash**. Sem isso você vê `PH2D/target ... Not a directory` — ou pior, edita o main em silêncio.
- **B — Foundational você PODE e DEVE tocar** (com cuidado), sob o protocolo testado (ADR-0107). A
  integração roda `foundational-integrate.sh` + Mergiraf. **PARE e reporte ao Enio SÓ se:** (a) for
  **contrato congelado** (CLAUDE.md §6 — `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`; exige
  ADR), ou (b) o rebase conflitar em código **FORA dos seus arquivos** (mesmo-símbolo com outra linha).
  **Nunca negocie com outra linha.**
- **B' — Ao CRIAR foundational novo, projete-o para ISOLAMENTO:** módulo/arquivo **IRMÃO** novo em vez
  de engordar um compartilhado; ponto de extensão **append-only**. Todo id/const/variant novo → **anote
  no handoff** (regra H) pro integrador detectar colisão.
- **C — Commits locais frequentes:** `git commit --no-verify -m "..." -- <paths>`.
  **NUNCA `push`. NUNCA `--force`. NUNCA `git add -A`.**
- **D — `git rebase main`** no início de cada jornada e antes de integrar. Conflito em `Cargo.lock` ou
  arquivo **GERADO** (registry-init): **nunca resolva na mão — regenere.**
- **E — Fechamento = gate batched**, e então **PARE**. Você **NÃO integra** e **NÃO roda
  `foundational-integrate.sh`**. Quem funde é um **agente integrador dedicado**, só por **ordem
  EXPLÍCITA do Enio**.
- **F — Ship (`ship.sh` + push + CI): NUNCA por conta própria.** Integrar/pushar sem ordem =
  **violação do protocolo**.
- **G — UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded → tokens / i18n (HR-15).
- **H — HANDOFF DE INTEGRAÇÃO é entregável obrigatório** ao fechar (DIRETRIZ §1.5.9): branch/HEAD/base ·
  foundational tocado + por quê · ids/consts/variants novos **com valores** (colisão!) · contratos
  congelados encostados (deve ser **nenhum**) · **o que só o `ship.sh` pega** · o que smoke-testar.
  Modelo pronto: [`HANDOFF_line_Painter_integracao_2026-07-11.md`](HANDOFF_line_Painter_integracao_2026-07-11.md).

**A CADA passo de implementação, releia** [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
Regra-mãe: **verde-de-compilação é velocidade; no audit vale ZERO.** Todo fix precisa de um **RED
refutável** — e **verifique o RED** (desligue o fix, veja falhar).

---

## 2. O que a jornada anterior entregou (contexto de 1 minuto)

**(a) Tiling seamless — FECHADO.** Texturas de slot (imagem · presets Paper · procedurais lattice em
qualquer tamanho · 10 analíticos via `analytic_tile_period` · Dots/Scales via hash-wrap) tilam sem
costura. Formas dinâmicas atravessam a barreira (edit-in-tile multi-shape, overlay contínuo, badges).
Smudge wrapa. **(b) Fixes de textura:** Grain casa a escala do brush (ViewPlane→canvas por radius) ·
Paper **reseta params** ao trocar kind · Paper procedural ganhou **default de Size** por classe de escala.
**(c) Z-order:** tiles do Repeat Image cobriam o chrome de edição → corrigido + **gate novo**.
**(d) [BUGS #11](Painter/BUGS_painter.md) — investigação do "retângulo" per-layer-color: ABERTO.**

---

## 3. BACKLOG — o que fazer (com recomendação)

### 🥇 PRIORIDADE 1 — **Smoke pendente que JÁ ESTÁ NO MAIN** (risco real)

**`e3ff4f27` — default de Size do Paper procedural: 1 → 12.** Foi integrado **SEM o Enio confirmar o
smoke** (ele mudou de assunto). É mudança **visível**. **Peça o smoke antes de qualquer coisa:**
selecionar **Voronoi no slot Paper** deve nascer em **Size 12** (tooth fino, celular), não blobs de
256px; trocar entre dois procedurais **preserva** um Size ajustado à mão; preset (Cold/Rough/Hot) volta
a Size 1. Se o Enio reprovar, é um revert de 1 commit (`watercolor_settings.rs`).

### 🥈 PRIORIDADE 2 (recomendada) — **#16 PESQUISA: traço de aspecto 3D**

**Por que esta é a jogada estratégica.** O objetivo REAL do Enio com o Per-Layer Color é *"pintar brushes
com aspecto 3D como os artistas do Procreate fazem"*. Só que o Per-Layer Color é **exatamente** onde
moram os dois problemas abertos (o muro de perf `O(D·N·S)` e o Bug #11). A alternativa mapeada no doc 13
#16 — **height-map + lighting pass** — entregaria o **mesmo look** por uma fração do custo, **sem** o
`×N` e **sem** o caminho que produz o retângulo. **Atacar #16 pode tornar #11 e #15 irrelevantes.**
Comece por **pesquisa** (como Procreate/Rebelle/Painter fazem) → proposta → só então código.

### 🥉 PRIORIDADE 3 — **Bug #11 (retângulos per-layer)** — **só quando REPRODUZIR**

**Leia [`Painter/BUGS_painter.md` #11 INTEIRO antes de tocar** — ele tem a tabela do que **já foi
descartado** (composite CPU · upload parcial · tiling · slot GPU · upload por versão) e economiza rounds.

- **Estado: ABERTO e DORMENTE.** **Intermitente** — 3 runs seguidas sem reproduzir.
- **NÃO declare resolvido por não-reprodução.** Nada foi corrigido (o diff da jornada foi +21 linhas,
  todas dentro de `if env::var_os(...)`). Memória: [[feedback_nonreproduction_is_not_proof_of_fix]].
- **Só sobraram 2 suspeitos:** o **produtor GPU** (`painter_gpu_preview::try_drive`) ou um **OVERLAY**
  desenhado por cima. Condição provável: **canvas grande** (~900+ quando apareceu; 512² fica limpo).
- **A armadilha JÁ ESTÁ ARMADA** (no main, custo zero desligada):
  ```bash
  PH2D_PREVIEW_DIAG=1 ./target/release/ph2d-host-desktop 2>/tmp/diag.log   # qual produtor tem o slot
  mkdir -p /tmp/dump && PH2D_PREVIEW_DUMP=/tmp/dump ./target/release/ph2d-host-desktop  # composite CPU, 1 PNG/frame
  ```
- **NÃO cace às cegas.** A jornada passada gastou **9 tentativas headless, todas verdes** — o harness
  reproduzia o *mecanismo*, não o *contexto*. **Espere o bug reaparecer** no uso do Enio com a armadilha
  ligada, e trabalhe com os PNGs. ([[feedback_harness_reproduces_mechanism_not_context]])

### Demais itens abertos (doc 13 / `HANDOFF_watercolor_tiling_shape_overlay.md` §2)

| # | Item | Nota |
|---|---|---|
| **15** | Perf do Per-Layer Color | **DESPRIORIZADO pelo Enio** — smoke em `--release` limpo: *"performance muito boa… só mude se for para melhorar"*. Os números confirmam: **só o canto extremo** (2048²·r100·**N16**·dinâmico = 56 ms) fura os 16,7 ms; **todo o resto já está a 60fps**. As alavancas de fator-constante (opt-level, ADR-0109, kernel fundido) **já estão gastas**; o que sobra troca **feel do arraste** — não mexa sem ordem. |
| **2 fu** | Tiling — Fase 3 | **RESTA:** papel procedural on-the-fly · caso **rotacionado** (limitação fundamental de todo grid) |
| **7** | Shape Tone ramp / Per-Layer Color em **aquarela** | semântica indefinida (tone da silhueta no wash?) |
| **12(c)** | Secagem influenciando a MESCLA | **DEFERIDO** — precisa de um sinal de **molhabilidade** separado da cobertura de pigmento (tentado e revertido: quebra `clean_water_backrun`) |

---

## 4. Gotchas operacionais desta linha (cada um custou tempo real)

1. **`cd` absoluto em TODO comando Bash** (regra A). O cwd reseta pro main a cada turno.
2. **`fmt` = `rustup run 1.95 rustfmt --edition 2024 <arquivos>`** (o crate é edition 2024, let-chains).
   `cargo fmt` plain = **skew**. E **`fmt` re-expande** → rode fmt **ANTES** de medir LOC.
3. **★ Memória grava no repo ERRADO.** O symlink `~/.claude/projects/<key>/memory` aponta pro **repo
   PRIMÁRIO**, não pra worktree. Escrever memória pela ferramenta **grava no main**. ⇒ escreva, depois
   **copie pra worktree** e **remova do main** antes de commitar.
4. **LOC caps no teto** — `watercolor_render.rs` **699/700**, `painter_bridge.rs` 658, `patterns.rs` 645,
   `stroke_multi.rs` 644. Inserir 1 linha estoura. **Split em módulo irmão, NUNCA allowlist.**
5. **NÃO remova a armadilha** `PH2D_PREVIEW_DIAG`/`PH2D_PREVIEW_DUMP` do `painter_bridge.rs` — é
   intencional (Bug #11), custo zero desligada, documentada no BUGS #11.
6. **Cercas de Chesterton — não re-exponha sem novo smoke:** o **Blur do Wet Mix** (pickup FIXO em r×0,5)
   e o **Paper Colors ramp** (revertido; papel volta ao grayscale).

---

## 5. Fechamento da sua jornada

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo test -p ph2d-tool-painter -p ph2d-painter-brush --lib
cargo clippy -p ph2d-tool-painter -p ph2d-painter-brush --all-targets
rustup run 1.95 cargo fmt --all --check
# LOC cap:
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
```

**Dica que vale ouro** ([[project_integrator_ship_catches_latents_budget_iterations]]): o gate
per-linha **não** roda fmt-workspace / clippy-all / machete / deny — **rode você mesmo antes de fechar**.
A jornada passada pré-drenou tudo e o integrador gastou **1 iteração em vez de 2-4**.

Depois: **atualize o tracker** (`docs/Painter/13_fila_integracao_watercolor_secoes.md`), **escreva o
handoff de integração** (regra H, modelo em `HANDOFF_line_Painter_integracao_2026-07-11.md`) e **PARE**.
Reporte *"linha pronta + handoff"* e **espere a ordem do Enio**.

---

*Linha `Painter` pronta pra continuar. HEAD `3805f650` (== main). Aguardo a tarefa.*
