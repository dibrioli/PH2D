# A sonda dos traços a tamanho 28, em `main` 2a925859d (2026-10-07)

O «antes» da linha Wet Painter e a imagem da cerca: Digital, Watercolor e Impasto **não podem mudar
um byte** (teste `tests/it/os_outros_meios_nao_mudam.rs`, que guarda o FNV-1a dos pixels RGBA; a
metade que lê ficheiros é `ferramentas/cerca.sh`). O `wetpaint.png` é o ponto de partida que as
ondas mudam de propósito.

```
PH2D_SONDA_OUT=<dir> PH2D_SONDA_SIZE=28 cargo test -p ph2d-tool-painter --profile smoke --test it \
    sonda_tracos_por_meio -- --ignored --nocapture        # por scripts/ph2d-run.sh, num ficheiro .sh
magick <dir>/<meio>.ppm <meio>.png
```

Tela 1000×640 branca, os cinco gestos de `tests/it/sonda_gestos.rs`, `set_brush_size_px(28)`,
`strength 1`, azul `#1765b3` e vermelho `#bf0012`, 8 s de relógio no fim. Duas corridas e os perfis
`test` e `smoke` dão os mesmos bytes.

| meio | FNV-1a (RGBA de `run_full`) | sha256 do PPM (16) |
|---|---|---|
| digital | `0x9b4c55cff56567bd` | `4edad6b8c81256d2` |
| watercolor | `0xc4fdb261ca6e97af` | `701f41dd8db08f77` |
| impasto | `0xd6f72b022a530144` | `f5d7e59944a6e798` |
| wetpaint | `0x2c123ed0afcfd14b` | `80fa9eba50e03731` |

As medidas destas imagens (a régua `ferramentas/mede_traco.py`) estão no [05 §1](../../05_plano_da_linha_e_o_antes.md)
e no [03 §2](../../03_tracos_medidos_rebelle_x_ph2d.md).
