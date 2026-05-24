# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** ATIVO desde 2026-05-23 — Wave 10 / Etapa 0
**Pastas que vai tocar:**
- `scripts/` (slot-env.sh, git-stage-guard.sh — Etapa 0)
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` (amendment §1.1 — Etapa 0.3)
- `.github/workflows/spike.yml` (CI paralelo — Etapa 0.4)
- `docs/archive/handoffs-completed/` (arquivar HANDOFF stale — Etapa 0.7)
- `docs/plans/2026-05-wave-10-perfection.md` (proposta v4 — salva)
- `tools/arch-gate-time-budget/` (Etapa 0.5)

**ETA:** Etapa 0 completa em 3-4 dias (DIA 1 de execução em curso).

**Vai pisar no Coord-B?** Nesta etapa, não — Coord-B ainda não foi convocado. Wave 10 / Etapa 4 abrirá Coord-B (panel-sync / chrome-sync / widget-sync).

---

## Coord-B (baldes)

**Status:** STANDBY — convocação prevista para Wave 10 / Etapa 4 (semana 7-8 do cronograma).

**Pastas reservadas (quando ativar):**
- `tools/ph2d-panel-sync/` (Etapa 4.1)
- `tools/ph2d-chrome-sync/` (Etapa 4.2)
- `tools/ph2d-widget-sync/` (Etapa 4.3)
- `crates/ph2d-panel-*` (sweep de gates UI — Etapa 5.1)
- `crates/ph2d-editor-core/src/screens/hero/chrome/*` (refactor Outcome — Etapa 4.2)
- `crates/ph2d-editor-core/src/widget/*` (proc-macro showcase_group — Etapa 4.3)

---

## Convenções

- Atualize sua seção ao **iniciar** sessão e ao **terminar** ou pausar longo.
- Se vai tocar pasta da seção do outro Coord, **PARE e renegocie** — adicione comentário `**!!! CONFLITO: ...**` na entrada dele e espere ack.
- Implementadores: sua pasta exclusiva é declarada no briefing do Coord — não precisa aparecer aqui.
- Quando ambos Coords inativos, deixe ambas as seções com `**Status:** INATIVO` mas mantenha as Pastas reservadas como referência.
