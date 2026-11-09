# Rodada dos secos — 10/11 (Pastel com o grão do dono · Chalk/Crayon a encher o dente)

Uma compilação `smoke`, 10 corridas (pressão 1 e rampa × 1–5 passadas) da sonda `sonda_csp_v3_revisao`.

- `var_secos.tsv` — as variantes: `P_hoje`, `P_dono` (Noise 120×120, params `[0.55,0.55,0.30,0.50,0.50,0.5]`, depth 1,0, fill 0,5);
  `Ch_hoje`/`Cr_hoje` (Multiply de hoje) e `Ch_H03/H06/H10`, `Cr_H03/H06/H10` (Height + fill 0,5 a depth 0,3 · 0,6 · 1,0).
- `corre.sh <pasta>` — corre as 10 corridas. A cena (`<pasta>/cena`) foi copiada de um scratch antigo; `mede_fill.py`/`carrega.py`
  têm «Pastel» fixo no nome, por isso Chalk/Crayon entraram por links `Pastel__<suf>` → `Chalk__<suf>`.
- `folha_secos.png` — a folha lado a lado, para o DONO escolher.

Cobertura na rampa (p 0,35 · 0,5 · 0,65 · 0,8), 1.ª → 5.ª passada: o Height só presta com depth 1,0 (0,3 não pinta; 0,6 fica magro);
`H10` mostra dente na rampa e enche em 3–5 passadas; o Multiply de hoje enche mas liso. `P_dono` enche mais depressa que `P_hoje`
e o grão sai mais fino e salpicado (é o contraste 0,55 no Height). Números completos: prompt `PROMPT_CONTINUACAO_2026-11-10b_mistura_previa_secos.md`.

Nada foi regravado: receitas (`csp_secos.rs`) e miniaturas esperam a escolha do dono.
