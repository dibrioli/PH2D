# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** ATIVO — Crisp Text Rendering Fase 0 (plano em `docs/UI_Plans/2026-05-24-crisp-text-rendering.md`)

**Pastas reservadas (em uso AGORA):**
- `crates/ph2d-tokens/src/typography.rs` + `lib.rs` (F1)
- `crates/ph2d-text/src/system.rs` (F2)
- `crates/ph2d-editor-core/src/paint.rs` (F3)
- `crates/ph2d-editor-core/src/screens/hero.rs` + `screens/hero/state.rs` (F4)
- `crates/ph2d-editor-core/src/ids.rs` + `screens/hero/pre_populate.rs` + `screens/hero/chrome/settings_text.rs` (F5)
- `screens/hero/chrome/mod.rs` (entrada nova em `dispatch_all`)

**Pastas reservadas (referência geral, quando trabalho foundational):**
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
