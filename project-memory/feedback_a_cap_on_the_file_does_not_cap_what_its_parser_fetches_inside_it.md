---
name: feedback_a_cap_on_the_file_does_not_cap_what_its_parser_fetches_inside_it
description: Um tecto de bytes no ficheiro de TOPO não limita o que o parser descomprime/carrega DENTRO dele — o usvg descomprime um SVG aninhado num data: em gzip sem limite
metadata:
  type: feedback
---

Medido na auditoria do fecho da W15 do Quadro (11/10): a porta das imagens SVG punha o `.svgz` de topo por um
`plain()` com tecto (`MAX_ARCHIVE_TEXT_BYTES`), e um SVG de 1 KB com `<image href="data:image/svg+xml;base64,…">`
cujo conteúdo é o gzip de 2 GB de espaços passava por baixo: o `usvg 0.48` segue o `data:` →
`Tree::from_data_nested` → `decompress_svgz` (um `read_to_end` sem limite), e a recursão volta ao MESMO resolvedor.
Um par na rede local derrubava o processo dos outros; o `catch_unwind` não apanha falta de memória.

**Why:** o tecto foi escrito onde o ficheiro ENTRA, e o parser tem as suas próprias portas de entrada (resolvedores
de `href`, `data:`, includes) que nenhum teste da porta de topo exercita.

**How to apply:** ao pôr um tecto num formato com conteúdo embebido (SVG, PDF, ZIP, `.excalidraw`), liste as portas
INTERNAS do parser (resolvedores, descompressões aninhadas) e passe-as pelo mesmo tecto — no usvg: um `resolve_data`
próprio que chama o `plain` antes do resolvedor de omissão (`ph2d-imageio-svg/src/raster.rs`
`capped_data_resolver`, gate `a_nested_gzip_data_uri_is_bounded`). Ver [[reference_topic_code_pattern_gotchas]].
