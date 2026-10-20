# ArtRage 6 Demo — Gloop Pen (J3, família `tintas`, 2026-10-20)

Mesma porta e licença da fixture-mãe ([`../README.md`](../README.md)): o `.arscript` do próprio app pela
`ferramentas/fotos_artrage.py`, a bateria `ferramentas/bateria.json`, só a foto A (o ArtRage não tem tempo: B ≡ A).

| campo | valor |
|---|---|
| ferramenta | Gloop Pen, `ToolID 4918` (`bateria_artrage.py`) |
| presets | `default` (Reset Current Tool) · `Gloop Pen/Expanding Rim` (bytes do `.prs` INSTALADO, lido em tempo de corrida) |
| tamanho | tabela `0.0336=1,0.268=18,1.342=128,2.5=240` (`lista_gloop.tsv`), MEDIDA numa 1.ª corrida a `k = 149`: s 0,268 → 18 px · 1,342 → 128 px (não linear) |
| comando | `bash ~/Apps/artrage/inicia.sh` + `python3 -I ferramentas/fotos_artrage.py fixtures/artrage_bateria/GloopPen/lista_gloop.tsv <saída>` pela vez `vez_do_app.sh artrage` |

O que se vê: `default` = corpo translúcido (~60 %) com a ORLA escura e nítida de 1–2 px, sobreposições que
multiplicam, o traço rápido mais fino (o ArtRage responde à velocidade); `Expanding Rim` = só a orla (oco).
A «gota que expande parada» do catálogo não se mede na bateria (nenhum gesto pára com a caneta em baixo).
