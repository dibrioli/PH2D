# Oráculo do Shape «Outline» — Godot 4.7.2 (MIT), corrido sobre as NOSSAS imagens (08/11)

Triagem: `pacman -Qi godot` → `godot 4.7.2-2.1`, **MIT**. APIs corridas (o que o editor usa em Sprite2D ▸
«Create CollisionPolygon2D Sibling»): `BitMap.create_from_image_alpha` + `BitMap.opaque_to_polygons(rect, epsilon)`
e `Geometry2D.decompose_polygon_in_convex`. Grelha: limiar de alfa {0.1, 0.5} × epsilon {0.5, 2, 4}.

## Correr de novo (≈ 1 s)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02
python3 -I docs/GameEngineFuncional/oraculo_contorno/gera_sinteticas.py          # entradas/ (determinístico)
PH2D_GPU=1 bash scripts/ph2d-run.sh godot --headless -s docs/GameEngineFuncional/oraculo_contorno/contorno_godot.gd -- "$PWD" /tmp/x/contorno.json
python3 -I docs/GameEngineFuncional/oraculo_contorno/sobrepoe.py /tmp/x/contorno.json /tmp/x   # cobertura + PNGs
```
O `.gd` grava também `_alfa_heroi_*.png` (a máscara recortada / unida) ao lado do JSON; o `sobrepoe.py` lê-as dali.
A geometria é determinística (duas corridas → `cobertura.txt` idêntico); só `us_min` e `data` variam.

## Ficheiros

| ficheiro | o que é |
|---|---|
| `fixture_contorno_godot_4.7.2.json` | a fixture: cabeçalho (versão, comando, data, sha256 das entradas) + por caso × (limiar, epsilon): nº de polígonos, vértices, pontos, área, peças convexas e pontos de cada peça, µs (mínimo de 15) |
| `cobertura.txt` | por caso × grelha: píxeis opacos (α ≥ 0,5) FORA do contorno e píxeis vazios DENTRO dele (teste do centro do píxel) |
| `sobreposicao_*.png` | limiar 0,5, epsilon 0,5 · 2 · 4: imagem ×5, contorno branco, peças convexas a cores |
| `entradas/sint_*.png` | sintéticas nossas, borda suave (supersampling 8×): L, anel (buraco), crescente, moeda + ilha solta |
| `contorno_godot.gd` · `gera_sinteticas.py` · `sobrepoe.py` | os três scripts |

Casos do herói (`docs/GameEngineFuncional/arte/heroi_azul.png`, 8×4 células de 100 px, recortadas no script):
`f00_parado` (0,0) · `f11_corre` (3,1) · `f20_agachado` (4,2) · `uniao_corrida_8` = OR (máximo) das alfas da
linha 1 · `uniao_folha_32` = OR das 32 células — a lei do dono «um contorno para todos os quadros».

## O que a Godot faz MAL (as armadilhas que a nossa versão tem de tratar)

1. **Buraco vira sólido.** `sint_anel`: 1 polígono só (o de fora); 43,7 % da área coberta é vazio (`cobertura.txt`).
2. **Epsilon 0,5 = escada de píxeis.** 146–168 vértices e a decomposição explode em **57–102 lascas** finas
   (`heroi_uniao_corrida_8` 83, `sint_anel` 102, a moeda CONVEXA 57). É exacto (0 píxeis de erro) mas inútil para a física.
3. **A simplificação come a arte para DENTRO.** Epsilon 4: 4–9 % dos píxeis opacos ficam fora (`f20_agachado` 8,8 %,
   moeda 9,4 %) e só ~1 % de vazio entra — o pé afunda no chão até ~epsilon px. Epsilon 2: 1–4 % fora.
4. **Decomposição não mínima.** O disco (convexo) sai em 7–9 peças com epsilon 2–4: qualquer vértice um
   nada reflexo parte a peça. Sem fusão por tolerância.
5. **Ilhas soltas** viram polígonos separados (`sint_moeda_ilha`: 2) — correcto, mas o brilho de 6 px vira colisor.
6. Não há tecto de vértices nem de peças; nem opção de envolvente convexa.

Tempo (µs, mínimo, total das 3 chamadas, 100×100): epsilon 0,5 ≈ 180–310 (a decomposição domina);
epsilon 2–4 ≈ 80–140. Barato: refazer ao mudar a imagem não pesa.
