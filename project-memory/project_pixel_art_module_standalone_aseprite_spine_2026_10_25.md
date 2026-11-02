---
name: project_pixel_art_module_standalone_aseprite_spine_2026_10_25
description: "Dono 25/10 — o módulo Pixel Art da engine é NOVO e AUTOCONTIDO (não é o Painter, não depende dele); o Aseprite é a referência das etapas; o Trial não salva nem corre scripts"
metadata:
  node_type: memory
  type: project
  originSessionId: c142c8eb-76c4-4200-a92d-b3bc73e244a0
  modified: 2026-10-25T20:26:17.616Z
---

Dono, 2026-10-25: *«construir nosso módulo pixel art na ph2d engine… O módulo não será parte do
painter já existente, mas módulo novo e auto contido. talvez no futuro eles dialoguem, mas não
agora pois o painter ainda está em desenvolvimento»*. A referência de etapas é o **Aseprite** (padrão
da indústria). Plano em `docs/PixelArt/02_plano_modulo.md`, estudo em `docs/PixelArt/01_estudo_aseprite.md`.

**Why:** o Painter ainda muda; acoplar o pixel art a ele travaria as duas linhas. Isto REVOGA o
mapa de porte do MVP (`docs/Pixel Art/02_spec_engine_e_porte.md` §2/§11), que punha os índices
como planos sidecar no `PainterTool`.

**How to apply:** nenhuma crate `ph2d-pixel-*`/`ph2d-app-pixel` depende de `ph2d-app-painter`;
diálogo futuro só por ponte explícita (importar/exportar imagem). As leis numéricas do MVP
(kernels inteiros, AB==BA, stamps, splitmix32) continuam valendo.

⚠️ Medido 25/10: o **Aseprite Trial 1.3.18.6** (Linux AppImage em `~/.local/opt/aseprite-trial/`)
**não salva/exporta** (nem em `-b`) e **não tem `--script`** ⇒ serve para OBSERVAR (Xvfb + fotos),
não como oráculo que se corre. Oráculo de saída exige a licença (US$ 19,99) — decisão do dono,
perguntada no §7 do plano. Ver [[feedback_the_owners_reference_product_decides_the_law]].

**Dono 25/10 (onda P2): «não vou comprar ainda. tente fazer o melhor com outras referências
abertas».** Oráculos que se CORREM, instalados: LibreSprite 1.3 (GPL-2, ancestral do Aseprite;
`-b --script` abre/salva/`loadPalette`, mas a API JS NÃO converte o modo de cor — `colorMode` sem
setter, `app.launch('ChangePixelFormat')` no-op mesmo sob Xvfb) · ImageMagick 7 (dither ordenado,
cinza por fórmula) · Pillow 13 (MIT-CMU: quantização, `L`) · GIMP 3.2 console (GPL, só corrido).
Nunca ler o fonte dos GPL.
