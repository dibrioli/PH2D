# HANDOFF de integração — linha `line/Painter` (Tiling seamless + Paper/Grain + Bug #11 ABERTO) — 2026-07-11

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff vai
> pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter`.

## 1. Identidade

- **Branch:** `line/Painter` · **HEAD:** `373821a3` (*docs(memory): não-reprodução não é prova de correção*).
- **Base do fork (merge-base com main):** `1c7c9a22`.
- **★ O `main` NÃO ANDOU desde o fork** — `git rev-parse main` == merge-base. ⇒ **`--ff-only` é trivial,
  zero conflito vindo do main, `foundational-integrate.sh` de árvore combinada só é necessário se OUTRA
  linha for integrada junto** (vide §2/§3 pro que grepar).
- **Commits da linha:** **26**, lineares, sem dependência de outra linha.
- **Gates rodados no fechamento (todos VERDES):**

  | Gate | Resultado |
  |---|---|
  | `cargo test -p ph2d-tool-painter --lib` | **542** passed / 0 failed / 16 ignored |
  | `cargo test -p ph2d-painter-brush --lib` | **231** passed / 0 failed / 1 ignored |
  | `cargo test -p ph2d-panel-painter-layers --lib` | **40** passed / 0 failed |
  | `cargo test -p ph2d-host-desktop` (gates do shell) | **206** passed / 0 failed |
  | `cargo clippy --all-targets` (as 4 crates) | **0 warnings** |
  | `rustup run 1.95 cargo fmt --all --check` | **limpo** (workspace inteiro) |
  | `typos` | **limpo** |
  | LOC caps | ok — **mas vide o aviso §6** |

## 2. Foundational/compartilhado tocado — **SIM (2 áreas)**

Diferente do handoff anterior desta linha (que não tocou nada fora do crate). Verificação:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
git diff --name-only $(git merge-base main line/Painter)..line/Painter
```

### (a) `shells/desktop/` — 6 arquivos (o bridge do Painter + 1 gate novo)

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `src/render_loop/painter_bridge.rs` | **z-order:** `draw_repeat_image` movido pra ANTES de `draw_selection_overlay`/`draw_overlays` (tiles opacos cobriam o chrome de edição) + a **armadilha de diagnóstico** env-gated (§3) | Reordenação + adições |
| `src/render_loop/painter_bridge_overlays.rs` | Overlay 3×3 nos tiles (Ellipse/Polygon/Line), badges tiladas | Aditivo |
| `src/render_loop/painter_bridge_curve_overlay.rs` | Overlay editável contínuo na costura do tiling | Aditivo |
| `src/render_loop/painter_bridge_line_overlay.rs` | idem (Line) | Aditivo |
| `src/render_loop/painter_bridge_op_badges.rs` | idem (badges) | Aditivo |
| `tests/repeat_image_tiles_draw_under_the_editing_chrome.rs` | **Gate NOVO** (arquivo novo) — trava a ordem de draw acima | Arquivo novo |

**Risco de colisão:** só se outra linha tiver editado os MESMOS arquivos de `shells/desktop/src/render_loop/`.
Grep sugerido pelo integrador:
```bash
git log main..line/<outra> --name-only | grep -E "shells/desktop/src/render_loop/painter_bridge"
```

### (b) `project-memory/` — **conflito textual PROVÁVEL**

- `project-memory/MEMORY.md` — **+1 linha** no índice (seção "Auditoria").
- `project-memory/feedback_nonreproduction_is_not_proof_of_fix.md` — **arquivo novo**.

**★ Toda linha que salvar memória edita o MESMO `MEMORY.md`** ⇒ conflito textual quase certo se houver
outra linha na jornada. **Resolução: manter AS DUAS linhas** (é um índice append-only; não é mesmo-símbolo,
não exige ADR). O Mergiraf resolve; se pedir, aceite ambos os lados.

> **Gotcha do Modo L (custou um passo aqui):** o symlink `~/.claude/projects/<key>/memory` aponta pro
> **repo PRIMÁRIO**, não pra worktree. Escrever memória pela ferramenta grava no main. O arquivo foi movido
> pra worktree e o rastro removido do main — **o main está limpo** (as mudanças que ele tem em
> `project-memory/` são PRÉ-EXISTENTES, não desta linha).

### (c) Crates da família Painter (fora do crate da tool, mas não-foundational)

`crates/ph2d-painter-brush/` (`texture/tiled.rs`, `texture/patterns.rs` + módulo novo
`texture/patterns/tileable.rs`) e `crates/ph2d-panel-painter-layers/` (`paint_brush.rs`).
Mesma regra de colisão: só grepando outra linha pelos mesmos arquivos.

## 3. Símbolos que podem colidir — inventário completo

| Símbolo | Onde | Escopo | Risco |
|---|---|---|---|
| `PAPER_PROCEDURAL_DEFAULT_SIZE: f32 = 12.0` | `tool/paint/watercolor_settings.rs` | **`pub`** | baixo (nome único) |
| `PREVIEW_DUMP_MAX_FRAMES: u32 = 240` | `shells/desktop/.../painter_bridge.rs` | privado | baixo |
| `PH2D_PREVIEW_DIAG` (env) | `painter_bridge.rs` | env var | baixo |
| `PH2D_PREVIEW_DUMP=<dir>` (env) | `painter_bridge.rs` | env var | baixo |
| `repeat_image_tiles_draw_under_the_editing_chrome` | `shells/desktop/tests/` | nome de gate | baixo (arquivo novo) |
| `analytic_tile_period` / `analytic_needs_hash_wrap` / `lattice_tileable` / `hash2w` | `ph2d-painter-brush/src/texture/patterns{,/tileable}.rs` | `pub(super)`/crate | baixo |

- **ZERO** `NodeId` / `IconId` / token de UI / chave de i18n / entrada em lista ordenada novos.
- **ZERO** `Cargo.toml` ou `Cargo.lock` tocado ⇒ **zero dependência nova** (machete/deny tranquilos).

## 4. Contratos congelados encostados — **NENHUM**

`Tool` / `RasterEditTool` / `CanvasPaintTool` / `PanelEvent` **intocados** (implementações internas
mudaram; assinaturas/superfícies não). Nodes / Vector: n/a. **Nenhum ADR necessário.**

## 5. O que só o `ship.sh` pega — **PRÉ-DRENADO** (rodei antes de fechar)

Ciente da [[project_integrator_ship_catches_latents_budget_iterations]] (o gate per-linha NÃO roda
fmt/clippy-all/machete/deny), **rodei esses gates aqui pra não queimar tuas iterações**:

- **fmt:** `rustup run 1.95 cargo fmt --all --check` no **workspace inteiro** → **limpo**.
- **typos:** → **limpo** (apesar do BUGS #11 ser um doc longo em pt-BR).
- **clippy `--all-targets`:** nas 4 crates da linha → **0 warnings**.
- **machete/deny:** **zero deps novas** (nenhum `Cargo.toml` tocado).
- **RUSTSEC:** risco genérico do dia do ship, não desta linha.
- **bindgen:** n/a (nada de FFI).

⇒ **Expectativa realista: 1 iteração de ship**, não 2-4. Se estourar, o suspeito nº1 é o **LOC cap** (§6).

## 6. ★ AVISOS OPERACIONAIS (leia antes de mergear)

1. **`watercolor_render.rs` = 699/700 LOC — NO TETO.** Se o merge/Mergiraf inserir **UMA linha**, o cap
   estoura. **Resolva extraindo pro módulo irmão** (`watercolor_rewet_px.rs` / `watercolor_noise.rs`),
   **nunca por allowlist** ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]). Outros no limite:
   `patterns.rs` 645, `stroke_multi.rs` 644, `painter_bridge.rs` 658 (tem marker de exceção), `watercolor_noise.rs` 628.
   **`fmt` re-expande** → rode fmt ANTES de medir.
2. **A armadilha de diagnóstico é INTENCIONAL — NÃO remova.** `PH2D_PREVIEW_DIAG` / `PH2D_PREVIEW_DUMP`
   em `painter_bridge.rs` são a armadilha armada do **BUGS #11 (ABERTO)**. Custo **ZERO** desligadas
   (tudo dentro de `if std::env::var_os(...)`). Estão documentadas em `BUGS_painter.md` §Armadilha.
3. **O Bug #11 NÃO está corrigido.** A linha entrega **testes + armadilha + doc**, não um fix. Se alguém
   ler "9 testes verdes" e concluir "resolvido", está errado — o doc diz explicitamente **ABERTO/dormente**.
4. **cwd reseta pro repo MAIN a cada turno** — toda mutação por **caminho absoluto do worktree**
   ([[feedback_sed_relative_path_hits_primary_cwd]]). Mordeu nesta linha também.

## 7. Ordem/dependências + **o que smoke-testar**

**Sem ordem interna** (commits lineares), **sem dependência de outra linha**.

### Já smoke-aprovado pelo Enio nesta jornada ✅
Tiling seamless (imagem + presets Paper + procedurais lattice + analíticos + Dots/Scales hash-wrap) ·
formas dinâmicas atravessando a costura · edit-in-tile multi-shape · overlay contínuo · overlay na costura
(z-order) · Smudge wrapando · wash re-renderizando ao vivo · escala do Grain · reset de params do Paper.

### ⚠️ NÃO smoke-confirmado — **peça ao Enio antes do ship**
- **`e3ff4f27` — Paper procedural: default de Size 1 → 12.** Reportei e o Enio mudou de assunto (foi pro
  per-layer color) **sem confirmar o smoke**. É mudança **visível** (o default do Paper procedural nasce
  com tooth fino em vez de blobs de 256px). **Smoke:** selecionar Voronoi no slot **Paper** → deve nascer
  em Size 12 (celular fino), não blobs gigantes; trocar entre dois procedurais **preserva** um Size ajustado
  à mão; preset (Cold/Rough/Hot) volta a Size 1.
- **`1f430d17` / `373821a3`** — só testes + doc + instrumentação env-gated ⇒ **nada a smokar**.

## 8. O que a linha entrega (contexto de 1 parágrafo)

**(a) Tiling seamless completo** — texturas de slot (imagem, presets Paper, procedurais lattice em
qualquer tamanho, analíticos, e Dots/Scales via hash-wrap) agora tilam sem costura; as **formas dinâmicas**
atravessam a barreira do tiling (edit-in-tile multi-shape, overlay contínuo, badges tiladas) e o **Smudge**
wrapa na costura. **(b) Fixes de textura** — o Grain passou a casar a **escala do brush** (ViewPlane→canvas
por radius), o **Paper reseta params** ao trocar de kind (Voronoi no Paper agora casa o do Grain) e ganhou
**default de Size** por classe de escala. **(c) Z-order do overlay** — os tiles do Repeat Image cobriam o
chrome de edição; corrigido + **gate novo**. **(d) BUGS #11 (ABERTO)** — investigação do "retângulo"
per-layer-color: **não corrigido**, mas o composite CPU foi **provado limpo** (9 testes), o espaço de busca
foi reduzido a **2 suspeitos** (produtor GPU / overlay) e ficou uma **armadilha re-ativável**.

— *Linha `Painter` pronta (HEAD `373821a3`, 26 commits). Aguardo ordem de integração — não integro nem pusho por conta própria.*
