# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** INATIVO 2026-05-27 noite — KTX2 Fase 2 W0 fechada e W1.T0 destrancada. ADR-0055-v4 Accepted (101 LOC strategic-only, audit 9.3/10). 2 commits locais: `971e237` (v4 + plano vivo + HANDOFF) + `db6971c` (W1.T0: ctt dep + sweep-grep). Próxima retomada lê [HANDOFF §12](HANDOFF_ktx2_phase2.md) + [[project-ktx2-phase2-v4-accepted-2026-05-27]] antes de W1.T1.

**Pastas reservadas (quando ativar):**
- `scripts/` · `.github/workflows/` · contratos congelados (`crates/ph2d-nodegraph/`, `crates/ph2d-editor-core/src/tool.rs`)
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` · arch-gates em geral
- `tools/asset-cooker/` (W1+ texture cook sub-command) · `crates/ph2d-asset/` (W1.T4 Asset variant) · `crates/ph2d-render/` (W2 wgpu pipeline)

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
