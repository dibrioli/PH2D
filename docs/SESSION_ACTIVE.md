# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** INATIVO

**Pastas reservadas (quando ativar):**
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
