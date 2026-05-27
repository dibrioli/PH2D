# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** ATIVO 2026-05-27 noite — KTX2 Fase 2 W0 retomada após 2ª opinião de 3 LLMs externas convergir em Opção 4 (ADR enxuto + plano vivo canônico). Reescrevendo ADR-0055-v4 ≤200 LOC strategic-only; arquivando v3 (660 LOC com snippets de código); migrando tabela canon pro plano vivo.

**Pastas tocadas nesta sessão (NÃO TOCAR — Coord-A own):**
- `docs/architecture/decisions/0055-cooked-texture-compression-pipeline.md` (reescrita v4)
- `docs/archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md` (novo — backup v3)
- `docs/plans/2026-05-texture-compression-waves.md` (header v4 + §Symbol Registry)
- `docs/HANDOFF_ktx2_phase2.md` (§12 status)
- `docs/SESSION_ACTIVE.md` (este arquivo)

**Pastas reservadas (quando ativar futuro):**
- `scripts/` · `.github/workflows/` · contratos congelados (`crates/ph2d-nodegraph/`, `crates/ph2d-editor-core/src/tool.rs`)
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` · arch-gates em geral

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
