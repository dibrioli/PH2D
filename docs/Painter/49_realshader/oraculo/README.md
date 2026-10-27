# Oráculo Cycles do Óleo Real (W1c, 26/10)

> Pergunta do dono: «porque o render parece mais um Eevee antigo que um Cycles moderno?». Resposta medida
> correndo o Cycles **sobre a nossa tinta** (CLAUDE.md §0.9): corre-se o binário, nunca se lê o fonte.

## Triagem de licença (por artefacto instalado)

| Artefacto | Pacote | Licença | Uso |
|---|---|---|---|
| `/usr/bin/blender` | `blender 17:5.2.3-3` (extra) | GPL-2.0+ (app) · Cycles Apache-2.0 | só CORRIDO por script; a saída é nossa |
| `studiolights/world/{studio,interior,courtyard}.exr` | idem | **CC0** (Poly Haven, `license.txt`) | pré-filtrados para o produto: `crates/ph2d-wet-paint/ambientes/*.amb` + `LICENSE-HDRI-CC0.txt` |
| `libcudart.so.13` | wheel PyPI `nvidia-cuda-runtime 13.0.96` em `~/.cache/ph2d-oraculo/cudart/` | NVIDIA EULA (redistribuível) | só para o Blender arrancar: o pacote `blender` liga-o e o `cuda` não está instalado |

sha256: `studio.exr c36c7fd3…` → `estudio.amb b8c05711…` · `interior.exr e945ff5c…` → `interior.amb e19be8f2…` ·
`courtyard.exr 6690b477…` → `patio.amb 7d944ee8…`.

## Como se corre (uma rodada ≈ 30 s na RTX)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
O=docs/Painter/49_realshader/oraculo
PH2D_OLEO_PNG=$PWD/$O/entrada bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_oraculo -- --ignored --nocapture
PH2D_GPU=1 LD_LIBRARY_PATH=$HOME/.cache/ph2d-oraculo/cudart/x/nvidia/cu13/lib bash scripts/ph2d-run.sh \
  blender -b --factory-startup -P $O/cycles.py -- $PWD/$O/entrada $PWD/$O/saida h_fina 256
python3 $O/compara.py $O/entrada $O/saida studio_sol10      # Clássico: PH2D_OLEO_AF="ambiente=0" e sol10
```

- `cycles.py`: a altura vira malha (1 vértice/célula), a cor sem relevo é o albedo, Principled IOR 1,5 com
  a nossa rugosidade, câmara ortográfica; céu uniforme `ceu` + sol `(1−ceu)·π/sin(alt)` (plano = albedo).
  Os termos separam-se pelo MATERIAL (`cinza`, `cinza_direto` com 0 ricochetes, `brilho` base preta):
  ⚠️ o Blender 5.2 desta máquina não grava os passes do Cycles no EXR (medido: nem `write_still` nem `save_render`).
- `calibra_ao.py` / `candidatas_ao.py`: a visibilidade do céu e o multi-ricochete contra o Cycles.
- `prefiltra_ambiente.py`: HDRI → irradiância + brilho GGX em 4 rugosidades (formato no cabeçalho).
- Régua: [`regua_W1c.txt`](regua_W1c.txt) · folhas: [`../out/W1c_folha.jpg`](../out/W1c_folha.jpg),
  [`../out/W1c_zoom_bristle.jpg`](../out/W1c_zoom_bristle.jpg).

## O que a régua respondeu (cena do dono: vermelho empilhado Dabber + Oil Bristle)

| | W1b (lei M2) | W1c Clássico | W1c Estúdio | Cycles (mesma receita) |
|---|---|---|---|---|
| pixels «pretos» no vermelho | **4,44 %** | 0,00 % | 0,00 % | 0,00 % |
| fenda vermelha: luminância p50 · R p50 | 41 · 148 | 47 · 168 | 49 · 168 | 54 · 165 (sol) · 47 · 167 (estúdio) |
| difuso na fenda (cinza 0,5) p50 | 0,52 | 0,53 | — | 0,48 |

1. **Pontos pretos = a lei M2** (`fenda = 0,25·albedo^2,1` por canal derrubava o próprio canal dominante).
   O difuso na fenda já batia (0,54 × 0,55). Cura: visibilidade GTAO + multi-ricochete de Jimenez, que bate
   com o Cycles a 2–7 % (o gate `o_multi_ricochete_bate_com_o_cycles` guarda os números).
2. **Linhas brancas finas = uma lâmpada só.** O ambiente dá o reflexo largo (Cycles estúdio: área de brilho
   forte 2,0 % × 0,29 % da lâmpada). O ambiente entra SÓ no brilho (o «Environment»/«Reflection Map» do Rebelle).
3. **Realce**: o Cycles com estúdio põe 23 % da tinta acima de 1 → `rola` enche a folga até 1 (neutro sem brilho).
4. **Penumbra**: a marcha mede o horizonte em ELEVAÇÃO — já é um sol com tamanho (alarga com a distância).

## Custo na CPU (o caminho da pintura até haver passe de placa)

`medida_oleo_real_custo` (smoke, 1 núcleo, ambientes intercalados, mínimo de 7, loadavg 3,7–5,9):
**Clássico 1 547 ns/célula · Estúdio/Interior/Pátio ≈ 1 870**. O composite reparte as linhas pelo rayon
(ADR-0109): uma pincelada de ~40 mil células ≈ 75 ms num núcleo, ≈ 5 ms em 16.

## ⛔ Recusas MEDIDAS

| Tentativa | Porque saiu |
|---|---|
| SH9 para a irradiância | 20 % de erro no plano virado para cima com a metade de baixo zerada → mapa de irradiância exacto |
| Difuso do HDRI × visibilidade média | a janela do estúdio entrava nas fendas que as paredes tapam: difuso 1,09 × 0,55 do Cycles |
| `(0,5+0,5·nz)` junto com a visibilidade | inclinação contada duas vezes: difuso da tinta 0,60 × 0,83 |
| Reflexo do ambiente × visibilidade média | a fenda reflectia o estúdio: luminância 74 × 47 → horizonte na direcção reflectida (49 × 47) |
| `V × multi_ricochete(V)` | Jimenez devolve a visibilidade CORRIGIDA, não um ganho (V² escurecia tudo) |
| Horizonte de 64 células na CPU | só ajuda o papel ao pé de paredes altas (corr. 0,48 → 0,64); fica para a pirâmide de máximos da placa |

## Fica aberto

- Brilho do Estúdio mais fraco que o do Cycles (p99 0,26 × 0,49; área > 0,3: 0,73 % × 2,0 %).
- Difuso da tinta 0,71 × 0,83: o rebote da LÂMPADA nas paredes vizinhas (Cycles: +11 % p50).
- Oil Bristle: a nossa visibilidade vê mais céu que o Cycles (viés +0,11; os sulcos dos fios).
- Metallic Materials do Rebelle (Reflectivity + Reflection Map): o mesmo ambiente pré-filtrado, F0 da cor.
