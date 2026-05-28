# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** ATIVO 2026-05-27 noite — Painter W1 T1.8 (Stroke Vector History, ADR-0046). Sessão única absorve Coord-A + Implementador (Enio 2026-05-27). Slot: `impl-1`.

**Pastas reservadas (sessão atual):**
- `crates/ph2d-painter-stroke/` (NOVO, drop-crate caminho A — DIRETRIZ §3.A)
- `crates/ph2d-host/src/` SOMENTE para amend de `MemoryBudget { painter: PainterMemoryBudget }` ao final (ADR-0046 §2.10) — Coord-A only
- Read-only: `crates/ph2d-painter-contracts/` (gates já presentes, ativam vacuous-pass → real ao nascer o crate)

**Pastas reservadas (genéricas, quando voltar a contexto foundational):**
- `scripts/` · `.github/workflows/` · contratos congelados (`crates/ph2d-nodegraph/`, `crates/ph2d-editor-core/src/tool.rs`)
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` · arch-gates em geral
- `tools/asset-cooker/` · `crates/ph2d-asset/` · `crates/ph2d-render/`

**Contexto pausado (retomar pós-Painter):** KTX2 Fase 2 W0 fechada e W1.T0 destrancada. ADR-0055-v4 Accepted (101 LOC strategic-only, audit 9.3/10). 2 commits locais: `971e237` (v4 + plano vivo + HANDOFF) + `db6971c` (W1.T0: ctt dep + sweep-grep). Próxima retomada lê [HANDOFF §12](HANDOFF_ktx2_phase2.md) antes de W1.T1.

---

## Coord-B (baldes)

**Status:** INATIVO

**Pastas reservadas (quando ativar):**
- `tools/ph2d-{panel,chrome,widget,node,tool}-sync/` (codegens)
- `crates/ph2d-panel-*` · `crates/ph2d-editor-core/src/screens/hero/chrome/*` · `crates/ph2d-editor-core/src/widget/*`

---

## Convenções

- Atualize sua seção ao **iniciar** sessão e ao **terminar** ou pausar longo.
- Se vai tocar pasta da seção do outro Coord, **PARE e renegocie** — adicione comentário `**!!! CONFLITO: ...**` na entrada dele e espere ack.
- Implementadores: sua pasta exclusiva é declarada no briefing do Coord — não precisa aparecer aqui.
- Quando ambos Coords inativos (como agora), deixe ambas as seções com `**Status:** INATIVO` mas mantenha as Pastas reservadas como referência.
