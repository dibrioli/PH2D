# Fontes de pixel de fábrica do Pixel Art (`ph2d-pixel-text`)

Copiadas **byte a byte** de `docs/Pixel Art/app/vendor/fonts/` (o MVP W27; sha256 conferido na cópia, 02/11)
e embutidas por `include_bytes!` (`src/lib.rs`, `BUILTIN`). Nenhum arquivo foi mudado.

| Arquivo | Fonte | Formato | Autoria e licença | De onde |
|---|---|---|---|---|
| `tom-thumb.bdf` | Tom Thumb 4×6 | BDF | Brian J. Swetland (1999), modificada por Robey Pointer (2010). **CC0 ou CC-BY 3.0** (o Brian autorizou, 2015); as modificações do Robey, **MIT** | https://robey.lag.net/2010/01/23/tiny-monospace-font.html |
| `unscii-8.hex` | Unscii 8 (8×8) | HEX | viznut. **Domínio público / CC0** (só os derivados do Unifont — `unscii-16-full.*` — são GPL, e não estão aqui) | http://viznut.fi/unscii/ |
| `unscii-16.hex` | Unscii 16 (8×16) | HEX | viznut. **Domínio público / CC0** (é o `unscii-16`, não o `unscii-16-full`) | http://viznut.fi/unscii/ |
| `cozette.bdf` | Cozette 6×13 | BDF | Samhain / Ines e contribuidores. **MIT** — texto em `LICENSE-cozette.txt` | https://github.com/the-moonwitch/Cozette (release v.1.30.0) |

A triagem de licença foi feita no ARTEFATO (o arquivo e a página do autor), não no nome do projeto
(registo original: `docs/Pixel Art/app/vendor/fonts/README.md`).
