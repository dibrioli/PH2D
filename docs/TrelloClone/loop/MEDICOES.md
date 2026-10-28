# Medições (§0.0) — nenhum MAX_* sem a tabela ao lado

## W2 — 60 fps da aba (herdada)
Cabe a 60 fps até ~20 000 cartões (CPU do layout/pintura); tabela no handoff W2→W3.

## W3 D2 — a frente do cartão com capas de IMAGEM (28/10)

Régua: `paint_card::tests::medir_capas` (ignorada; `bash scripts/ph2d-run.sh cargo test -p
ph2d-app-kanban --profile smoke --lib -- --ignored medir_capas --nocapture`). O tamanho da cena 2
(2 000 cartões numa lista), 20 fotos PNG 1600×900 distintas, ¼ capas inteiras; com × sem capa no
MESMO processo, 7 blocos intercalados, o MÍNIMO. ⚠️ `loadavg` 35,9 na corrida (máquina partilhada):
vale a DIFERENÇA, não o absoluto.

| Fase (2 000 cartões, um quadro) | com capas | sem capas |
|---|---|---|
| `front()` (a capa sai da cache por `FileId`) | 0,38 ms | 0,30 ms |
| altura SEM a cache (a disposição só a refaz quando a frente muda) | 22,07 ms | 21,92 ms |
| pintura (`draw_image_rgba` + recorte por cartão) | 7,87 ms | 6,69 ms |
| descodificar UMA foto 1600×900 (fio de fundo, nunca no desenho) | 13,9 ms | — |

- A descodificação corre num fio `kanban-covers` (um só, nasce à primeira capa); o desenho mostra a
  cor de espera até ela chegar. A cache é por `FileId` (a chave é o conteúdo): 20 ficheiros = 20
  descodificações, nunca 2 000.
- **`cover::TEXELS_W = 512`** — a foto reduz-se por metades enquanto a largura ≥ 2 × 512 (1600 → 800).
  Recurso: MEMÓRIA e o atlas do Vello (um quadrado de 8192² partilhado por todas as imagens à vista,
  `vello_encoding-0.10.0/src/image_cache.rs:10`). 512 = o cartão (256 lógicos) a 200 %; acima disso o
  Vello amplia bilinear. 800×450×4 B = 1,4 MB por foto; 20 fotos à vista = 29 MB, 3 % do atlas.
- O véu da capa inteira é medido sobre a imagem REAL: cada linha da miniatura guarda a luminância
  mais clara; o alfa sobe até o título se ler a 4,5:1 (gate
  `a_full_image_cover_keeps_the_title_legible_on_the_real_image`: imagem branca ≥ 4,5:1, escura =
  o véu da maquete `0,34`).
