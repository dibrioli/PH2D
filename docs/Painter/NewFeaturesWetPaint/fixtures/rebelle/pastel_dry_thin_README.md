# Fixtures — Rebelle «Pastel Dry / Dry Thin» NO TEMPO (prompt 35, 14/10)

| campo | valor |
|---|---|
| oráculo | Rebelle 8 Pro (Trial), Wine, tela virtual `kwin_wayland --virtual` 1920×1080 (`gotas_inicia.sh`) |
| preset | `PASTEL` · `Pastel Dry/Dry Thin`, opacidade e pressão do próprio preset, cor `(23, 101, 179)` |
| tamanho | `pastel_dry_thin_29/`: `size_px 29` (o número do nosso Soft Pastel) · `pastel_dry_thin_70/`: `size_px 70` (a MESMA largura VISTA que o nosso, ~30 px) |
| tela | `NEW_ARTWORK` 1000×640, papel `Default/EX00 White Simple`, semente 7, zoom 100 % (campo do Navigator) |
| gestos | `pastel_eventos.py`: recto (60,70)→(280,70) e zig-zague de 7 vértices, 4 px por evento, 16 ms entre eventos, pressão 1 |
| fotos | `foto.sh` (`import -window`), recortadas à tela (1000×640 em (480, 220)); `meio` = a meio do gesto, `up` = a 1.ª foto igual à anterior depois do RELEASE, `0.5 · 1 · 3 · 6` s depois do RELEASE (`tempos.tsv` = o instante REAL) |
| régua | `ferramentas/pastel_regua.py <esta pasta> <PPMs da sonda sonda_pastel_no_tempo>` |

**Medido (14/10):** o Rebelle NÃO muda nada de 0,5 a 6 s depois do traço (0 píxeis, nos dois tamanhos). A
`size_px 29` o bastão mostra **13 px** de largura (é fino: `size_px 60` → 25). Ver o doc
[35](../../35_PROMPT_janela_pastel_contra_o_rebelle.md) e o handoff da linha para os números de cada régua.

⚠️ O Rebelle desenha o fim do traço com atraso: a foto logo após o RELEASE ainda não tinha os últimos 140 px
(a 70 px de pincel) — por isso o `up` espera duas fotos iguais.
