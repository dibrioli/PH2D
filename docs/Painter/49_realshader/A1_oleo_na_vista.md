# A.1 — o Óleo Real iluminado à resolução da VISTA (CPU, 30/10)

Código: `crates/ph2d-wet-paint/src/oleo_vista.rs` (`PreparoVista::novo` + `linha`/`render`), portões em
`oleo_vista_tests.rs`, prancha e custo em `examples/prancha_oleo_vista.rs`.

## A lei
Cada pixel da vista pede o documento em `u = x0 + (x+0,5)/s − 0,5` (centros). A cor da pilha ANTES da luz
(`render::PreLuz`, devolvida pela própria linha do óleo), a luz/céu (`LuzCeu`) e o Gloss vêm por Catmull-Rom 4×4
separável preso aos 16 vizinhos (sem anel); a altura fina por Catmull-Rom SEM prisão; normal e hessiana pelo Sobel
sobre a altura AMPLIADA com o passo da vista (declive ×s, hessiana ×s²). Material, Subsurface e reflexo livre: a
célula mais próxima. Fecha com a MESMA óptica (`oleo_real::termos_da_amostra` — `termos_com` passou a ser a
`Amostra::da_celula` dela) e a MESMA mistura (`PreLuz::fecha_oleo`, que a linha de hoje também chama).
- `s < 1` fica FORA (afastar quer mip, não interpolação). Grade com `ratio` 1; com outro, `s` em células.
- A sombra NO PAPEL (o buffer do composite) não é desta porta — fica para a A.2/GPU.
- Nada guardado em resolução maior: as células da janela uma vez por quadro, e 3 linhas de altura ampliada por linha.

## Números (fixture nossa: `oleo_vista::prova::faca`, 3 traços grossos + faca diagonal, recorte 64×64 em (16,16))
| | valor |
|---|---|
| identidade s=1 | 0 bytes diferentes (`a_vista_a_100_por_cento_e_a_linha_de_hoje_ao_byte`) |
| var. laplaciano 400 %, bilinear de hoje | 54,64 |
| var. laplaciano 400 %, vista nova | 135,24 → **2,47×** (alvo do briefing 3×: **não chega**; portão prende 2×) |

Custo, UM fio, `--release`, load 1,1, 15 blocos intercalados aquecidos (ns/píxel de saída, mín · mediana → ms
por quadro extrapolado linearmente, porque o recorte é 64×64):
| variante | ns/px | 1920×1080 | 3840×2160 |
|---|---|---|---|
| hoje (resolução do doc) | 1050 · 1051 | 2177 | 8707 |
| nova s=1 | 2393 · 2396 | 4961 | 19844 |
| nova s=2 | 1312 · 1313 | 2720 | 10878 |
| nova s=4 | 936 · 936 | 1940 | 7760 |

O custo é a óptica por pixel (~0,9 µs; `luz_ceu` por célula sem a cache do campo); a s=1 paga a linha de hoje no
preparo e outra vez por pixel. As linhas são puras: o composite reparte-as pelo rayon como a de hoje (ADR-0109;
este crate não depende do rayon). Na CPU isto NÃO é tempo real à resolução do ecrã — é a lei de referência.

## Prancha
`capturas/a1_prancha.png` (2×2): hoje 100 % | nova 100 % (×4 nearest, idênticas) / hoje 400 % bilinear | nova 400 %.
A nova a 400 % tem realces finos nas cristas onde a bilinear borra; aparecem alguns pontinhos escuros/claros
(brilho GGX sobre a hessiana ampliada) — a olhar na A.2.

## Comandos
- `bash scripts/cargo-test-narrow.sh ph2d-wet-paint oleo_vista`
- `bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo run --release -p ph2d-wet-paint --example prancha_oleo_vista [-- --mede]`

## Para a GPU
O mesmo passe por pixel: `PreLuz`/`LuzCeu`/Gloss/altura como texturas à resolução do documento, Catmull-Rom +
Sobel no shader; a s=1 tem de dar o byte da CPU (por placa). Abertos: o alvo 3× de nitidez, os pontinhos do
brilho, a sombra no papel na vista, `s < 1` (mip).
