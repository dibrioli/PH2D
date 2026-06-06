═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Vector · W10 PlatformHost real — PRONTO (crate satélite)
Autor: Coordenador (jornada 2026-06-06) · responde "W10: fast-path Vello + PlatformHost real no shell"
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
O **PlatformHost real** do W10 está pronto e testado: crate satélite
**`ph2d-system-fonts`** (`48a28f9`) que implementa `ph2d_vector_font::PlatformHost`
sobre a coleção de fontes do OS (fontique). Substitui o `MockHost`: `system_fonts()`
= famílias REALMENTE instaladas; `fallback_chain(locale)` = a cascata por-script do
OS → CJK/Árabe/emoji resolvem pra uma fonte que cobre, em vez de tofu. **3 testes
verdes nas fontes reais deste Mac.**

## §1 — O QUE LANDOU (`48a28f9`)
`crates/ph2d-system-fonts/` — isolada, só lê o contrato `ph2d-vector-font` + fontique:
- `SystemFontHost::new()` → `Collection{system_fonts:true}` + `SourceCache`, atrás de
  `Mutex` (→ Send+Sync, parkável num resource compartilhado do shell).
- `system_fonts()`: enumera `family_names()` + coverage coarse (cacheado).
- `fallback_chain(locale)`: `locale.language()` → script ISO 15924 → `FallbackKey` →
  `fallback_families()` (a cascata REAL do OS).
- **Coverage coarse por design**: 1 codepoint-amostra por bloco Unicode via o charmap
  da fonte — exatamente o que o `CoverageRanges` se documenta ser ("coarse stand-in
  for a real cmap"); zero scan de cmap inteiro de centenas de fontes.
- fontique 0.6 (Linebender, que parley/vello já puxam, pure-Rust, sem dep nativo) →
  **zero dep transitivo novo** (Cargo.lock só ganha este pacote).

## §2 — O QUE FICA (teu / follow-up, não-bloqueante)
1. **Instanciar no shell:** hoje NADA consome o font-PlatformHost (grep vazio em
   shell+editor) — o caminho canônico glyph=VectorNetwork renderiza sem fallback. Então
   NÃO adicionei dead-wiring. Quando ligares o glyph-fallback multi-script, o shell faz
   `let host = SystemFontHost::new();` e passa pro `resolve_glyph_font`. One-liner.
2. **Fast-path Vello direto:** a outra metade do W10 Coord — otimização de render de
   glyph (bypass do glyph→VectorNetwork). Não tocada; non-blocking (o canônico já
   renderiza). Separado deste host.

## §3 — POSSE
Crate nova isolada (sem prefixo `ph2d-node-`). Contrato `ph2d-vector-font` não tocado
(só implementado de fora). Sem push (Coord shipa). `git status` conferido: nada alheio
no meu commit.
═══════════════════════════════════════════════════════════════════
