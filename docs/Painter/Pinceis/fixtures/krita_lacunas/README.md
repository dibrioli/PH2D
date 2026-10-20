# Fotos do Krita 6 sobre a bateria das LACUNAS (2026-10-20)

> Oráculo: **Krita 6.0.4.1** (`pacman -Qi krita`: GPL3 ⇒ só se corre; nenhum `.kpp`, ponta ou textura vem para o
> repo), a porta de [`../krita_bateria/`](../krita_bateria/README.md): caneta injectada no Qt (`handleTabletEvent`,
> pressão real), perfil próprio, Xvfb próprio (`:87`+), llvmpipe, 10 ms por evento em tempo real.
> Bateria: [`../../ferramentas/bateria_lacunas.json`](../../ferramentas/bateria_lacunas.json) v1, sha256
> `5ce10f13ea586664353cc43125c7e2293df91517f156af2e2b99dc328bb39f6b` (no `manifesto.json`). 0 falhas.

```bash
K=docs/Painter/Pinceis/ferramentas/krita_bateria.py; B=docs/Painter/Pinceis/ferramentas/bateria_lacunas.json
python3 -I $K --bateria $B --cenas velocidade --saida docs/Painter/Pinceis/fixtures/krita_lacunas \
  --presets "d) Ink-2 Fineliner,d) Ink-1 Precision,e) Marker Chisel Smooth,e) Marker Medium (mypaint)"
python3 -I $K --bateria $B --cenas carimbos --saida docs/Painter/Pinceis/fixtures/krita_lacunas \
  --presets "z) Stamp Leaves,z) Stamp Herbals,z) Stamp Hearts"
python3 -I $K --bateria $B --cenas fundo --saida <fora do repo>      # os 144: fundo_varredura_144.csv
```

| fotos | preset | gesto (cena) | lacuna |
|---|---|---|---|
| `d__Ink-2_Fineliner_velocidade.png` · `d__Ink-1_Precision_velocidade.png` | d) Ink-2 Fineliner · d) Ink-1 Precision | rectos a 3·6·12·25·50 px/evento | L12 |
| `e__Marker_Chisel_Smooth_velocidade.png` · `e__Marker_Medium__mypaint__velocidade.png` | e) Marker Chisel Smooth · e) Marker Medium (mypaint) | idem | L12 (controlos: neutros) |
| `z__Stamp_{Leaves,Herbals,Hearts}_carimbos.png` | z) Stamp Leaves · Herbals · Hearts | 45 toques a 1,5·D (pressão 1 · 0,6 · 0,3) | L19 |
| `fundo_varredura_144.csv` | os 144 do perfil limpo | `fundo` (frente azul, fundo AMARELO) | L11 |
| `v__Texture_Pointillism_fundo.png` · `z__Stamp_Hearts_fundo.png` · `b__Basic-5_Size_fundo.png` | 3 da varredura | `fundo` | L11 (matiz SORTEADA ≠ 2.ª cor; e um neutro) |

Os nomes são os do `krita_bateria.py` (`[^A-Za-z0-9._+-]` → `_`). Data de todas: 2026-10-20; o fundo por omissão do
perfil (branco) só muda na cena `fundo` (`cor_fundo`, op nova em `krita_bateria_dentro.py`).
**Achado:** **0 dos 144** presets de fábrica usa a 2.ª cor (o único > 0, `v) Sketching-1 Chrome Thin` 10 %, é a matiz
ruidosa de px quase pretos de um traço de 1 px); 25 não pousam tinta. As matizes «próprias» (Hearts, Pointillism) são
sorteio de HSV, não a cor de fundo.
