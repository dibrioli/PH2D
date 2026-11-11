# «A performance caiu depois da segunda rodada?» — A×B no app (11/11, 19:47–20:17)

- **A** = `e81e56bb1f` (`bfe7c348e5^`) · **B** = HEAD `57b4ddbac7`. Dois binários `--profile smoke`, cada um com o seu
  target; cada variante corre o roteiro (`mede_pen_up_4k.sh matriz`, `CURTO=1`) da SUA árvore. Radeon 610M RADV, Xvfb,
  papel Textured (relevo do dono). Intercalados por bloco (ímpar AB, par BA), 5 blocos. Roteiro e agregação:
  [`segunda_rodada/`](segunda_rodada/) (`ab.sh`, `agrega.py`, `tabela.md` completa, `carga.txt`).
- **Carga:** bloco 1 a 4K Filbert/Wet 28,6→6,8 (o build acabara), resto 0,5–2,0; o mínimo descarta o bloco 1.

| cena | fase | A mín / med | B mín / med | n | B−A mín |
|---|---|---|---|---|---|
| Filbert 4K papel | toque | 70,9 / 72,2 | 76,8 / 78,1 | 5/5 | **+5,9** |
| Filbert 4K papel | traço p50 · p95 | 18,9 · 22,9 | 18,9 · 21,2 | 5/5 | 0 · −1,7 |
| Filbert 4K papel | soltar | 47,8 / 49,0 | 34,0 / 36,1 | 5/5 | **−13,8** |
| Filbert 4K papel | quadros 3 s pós-up p50 · p95 | 10,3 · 11,3 | 10,3 · 11,4 | 5/5 | 0 |
| Filbert 4K papel | Ctrl+Z | 54,5 / 57,3 | 61,8 / 64,8 | 5/5 | **+7,3** |
| Filbert 1080p papel | toque | 26,2 / 26,4 | 27,3 / 28,2 | 5/5 | +1,1 |
| Filbert 1080p papel | traço p50 | 19,1 | 17,9 | 5/5 | −1,2 |
| Filbert 1080p papel | soltar | 37,8 / 38,9 | 22,0 / 23,1 | 5/5 | **−15,8** |
| Filbert 1080p papel | Ctrl+Z | 37,1 | 37,0 | 5/5 | 0 |
| Wet (omissão) 4K papel | toque | 57,3 / 57,7 | 62,8 / 62,9 | 5/5 | **+5,5** |
| Wet (omissão) 4K papel | traço p50 · soltar · pós-up p95 | 12,0 · 20,9 · 12,1 | 12,0 · 21,0 · 12,1 | 5/5 | 0 |
| Wet (omissão) 4K papel | Ctrl+Z | 46,8 | 47,9 | 5/5 | +1,1 |
| Impasto (omissão) 4K papel | toque | 11,7 / 11,9 | 20,8 / 20,9 | 5/5 | **+9,1** |
| Impasto (omissão) 4K papel | traço p50 · soltar | 11,0 · 18,1 | 11,0 · 18,0 | 5/5 | 0 |
| Impasto (omissão) 4K papel | Ctrl+Z | 22,0 / 22,9 | 14,0 / 14,1 | 5/5 | **−8,0** |

- **Chaves no MESMO binário B** (Filbert 4K papel, 3 rodadas, `faixas SO="M1|M3"`; carga 1,1–21,7, mínimos):
  `PH2D_WET_MISTURA` espectral × km: toque 76,8 × 77,0 · Ctrl+Z 61,6 × 61,8 ⇒ a mistura B32 não custa.
  `PH2D_COMPOSITE_LUZ_GESTO` simples × cheia: Ctrl+Z 62,2 × 66,0 · toque 77,6 × 77,6 ⇒ a dívida Cheia (C2/C3) não é o
  +7 do Ctrl+Z nem o +6 do toque.
- **Bissecção do toque do Impasto 4K** (3 rodadas intercaladas, carga 0,5–1,6): `355869fd78^` = 11,7 / 12,0 ·
  `9a7337ac68` = 21,0 / 21,1 ⇒ os +9 ms entram em `355869fd78..9a7337ac68` (39 commits; a cadeia B29.1 do tecto largo
  — u16 + pré-traço recomposto no carimbo — é o suspeito, não provado ao commit). O Ctrl+Z do Impasto em `9a7337ac68`
  ainda 22,9: a melhora −8 é a C1 (`23afcc19e0`).
- **Veredito:** durante o traço e com a água a correr depois de soltar, **nada caiu** (iguais ao 0,1 ms). Soltar o Filbert
  ficou **14–16 ms mais rápido**. Caíram só os **instantes isolados**: o 1.º toque a 4K (+5,5 a +9 ms; Impasto 12 → 21) e
  o Ctrl+Z do Filbert 4K (+7 ms, sem chave que o explique).
