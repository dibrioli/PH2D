# A.1 — o Óleo Real iluminado à resolução da VISTA (CPU, 30/10)

Código: `crates/ph2d-wet-paint/src/oleo_vista.rs` (`PreparoVista::novo` + `linha`/`render`), portões em
`oleo_vista_tests.rs`, prancha e custo em `examples/prancha_oleo_vista.rs`.

## A lei
Cada pixel da vista pede o documento em `u = x0 + (x+0,5)/s − 0,5` (centros). A cor da pilha ANTES da luz
(`render::PreLuz`, devolvida pela própria linha do óleo), a luz/céu (`LuzCeu`) e o Gloss vêm por Catmull-Rom 4×4
separável preso aos 16 vizinhos (sem anel). A ALTURA fina vem da **B-spline cúbica interpolante** (pré-filtro
recursivo de Unser, polo √3−2, janela + 8 células, borda espelhada): C2, e a normal e a hessiana são as
ANALÍTICAS da spline (sem Sobel na vista) — contínuas através das células. Nada que dependa da hessiana é
interpolado por célula (o brilho GGX lê a hessiana da spline; o Gloss é o campo da célula, não a hessiana).
Material, Subsurface e reflexo livre: a célula mais próxima. Fecha com a MESMA óptica
(`oleo_real::termos_da_amostra`; `termos_com` = `Amostra::da_celula`) e a MESMA mistura (`PreLuz::fecha_oleo`).
- `s = 1` com origem inteira ⇒ a linha de hoje, ao byte por construção.
- `s < 1` fica FORA (afastar quer mip). Grade com `ratio` 1; com outro, `s` em células.
- A sombra NO PAPEL (o buffer do composite) não é desta porta — fica para a A.2/GPU.

## Números (fixture `oleo_vista::prova::faca`: 3 traços grossos + faca diagonal; recorte 64×64 em (16,16), 400 %)
| régua (portão) | bilinear de hoje | Catmull-Rom (ronda 1) | B-spline (agora) |
|---|---|---|---|
| var. laplaciano (nova ≥ 3× bilinear) | 54,64 | 135,24 (2,47×) | **178,58 (3,27×)** |
| pico da célula, |DFT| bin n/s ÷ vizinhos (≤ 1,5) | 1,851 | 1,344 | **1,275** |
| pontinhos, fracção \|lap\| > 6σ MAD (≤ 1,5× bilinear) | 0,02888 | 0,01617 | **0,02026** |
| identidade s=1 | — | 0 bytes | 0 bytes |

⚠️ As réguas (b) e (c) já passavam com o Catmull-Rom: não ficaram vermelhas antes da correcção.

Custo, UM fio, `--release`, load 2,9–3,1, 15 blocos intercalados aquecidos (ns/píxel de saída, mín · mediana →
ms por quadro, extrapolado linearmente do recorte 64×64):
| variante | ns/px | 1920×1080 | 3840×2160 |
|---|---|---|---|
| hoje (resolução do doc) | 1066 · 1201 | 2210 | 8841 |
| nova s=1 (= hoje) | 1061 · 1076 | 2200 | 8799 |
| nova s=2 | 1164 · 1215 | 2413 | 9652 |
| nova s=4 | 778 · 795 | 1612 | 6449 |

As linhas são puras: o composite reparte-as pelo rayon como a de hoje (ADR-0109; este crate não usa rayon). Na CPU
isto não é tempo real à resolução do ecrã — é a lei de referência.

## Prancha
`capturas/a1_prancha.png` (2×2): hoje 100 % | nova 100 % (×4 nearest, idênticas) / hoje 400 % bilinear | nova 400 %.

## Comandos
- `bash scripts/cargo-test-narrow.sh ph2d-wet-paint oleo_vista`
- `bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo run --release -p ph2d-wet-paint --example prancha_oleo_vista [-- --mede]`

## Para a GPU
O mesmo passe por pixel: `PreLuz`/`LuzCeu`/Gloss/altura como texturas à resolução do documento, Catmull-Rom (cor/luz)
+ coeficientes da B-spline (altura, pré-filtro no CPU ou num passe) no shader; a s=1 tem de dar o byte da CPU (por placa). Abertos: a sombra no papel na vista, `s < 1` (mip), as
fileiras de pontos brancos que já estão NO documento (o brilho por célula dos traços).
