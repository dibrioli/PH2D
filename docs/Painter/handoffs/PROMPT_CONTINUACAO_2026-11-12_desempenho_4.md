# Prompt de continuação — linha PainterWatercolor · 12/11 n.º 4: o Painter plenamente otimizado (ondas 4–6)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_4.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`** (merge-base `bc4a6ce930`; quem funde é o
integrador que o dono nomear).

## Ordem do dono (verbatim, continua)

12/11: «vamos seguir até ter o app plenamente otimizado». «Se esforce para resolver todos os problemas de performance do módulo a todo
custo. … Não esqueça do handoff de integração ao final de tudo. Não integre.»

## Leia primeiro

1. [HANDOFF_INTEGRACAO_…_MISTURA_E_ENVIO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_MISTURA_E_ENVIO.md) (n.º 3: §4 o que
   entrou, §8 aberto) e os dois anteriores (`…_LUZ_E_CAMADAS.md`, `…_DESEMPENHO_TOTAL.md`), que continuam a valer.
2. `../missao_4k/BACKLOG.md` (B13.5/B13.6, B19.10, B20.6, B20.11 — a análise da n.º 3 está na própria linha) e `BACKLOG_AUDITORIA.md`
   (B01.A1, B40.A3–A5).

## Estado (fim da n.º 3)

| onda | estado |
|---|---|
| 3 mistura de cor | **FEITA** (B28.1 fechado sem mudar; B28.2, B28.4, B28.6, B28.7, B32.A1) — os 12 de fábrica com Color mixing < 16,7 ms/q a 360 px/q |
| 4 água do Wet | B13.5: o quadro do app já coberto (B14.1 + `Nada`); B13.3 ε>0 RECUSADO (medido); **B13.6 (o passo da sim, 20–22 ms/passo a 4K com 3 traços) por atacar** |
| 5 camadas | B20.6 a CPU feita (11,6 → 1,8 ms); **B20.11 com a cura desenhada (por implementar)**; **B19.10, B01.A1, B40.A3–A5 por atacar**; B20.6 a parte da placa por medir |
| 6 cadência e caneta | **B22.1/22.4, B23.2 por atacar** |

## O que fazer, por ordem (até ao fim, sem parar)

1. **B20.11** pela cura desenhada no BACKLOG (imagem de id estável em `ph2d-vector` + fase pelo tempo, passo 100 ms + impressão digital).
2. **B40.A3–A5, B01.A1, B19.10** (onda 5). 3. **B22.1/22.4, B23.2** (onda 6). 4. **B13.6** (perfil: `sonda_sono_ligado_e_desligado`
   com `PH2D_SONO_RES=4k`; o solver é série por ADR-0134 — paralelo só onde o ADR-0145 já deixa).
5. Por item: medir antes (A×B no mesmo binário, mínimo), curar AO BIT, gate por nome com mutação, medir depois, comitar com a tabela.
6. **Fecho:** fmt/clippy `PH2D_SUITE=fecho` das crates tocadas, alarmes de pincel por nome (comparar o fnv das miniaturas com a lista da
   n.º 3 — a mesma = ao bit), matriz final do app (`docs/Painter/ferramentas/mede_pen_up_4k.sh matriz`), handoff de integração NOVO +
   link no CLAUDE.md §5.1, `rm -rf target/*/incremental`, binário smoke compilado 2×, limpar `.ph2d-*`. Não integre, não envie.

## Regras aprendidas (12/11 n.º 3)

- **Perfil a 1 fio** (`RAYON_NUM_THREADS=1`) separa o custo do paralelismo: a pegada do Wet blend custava 55 ns/texel e o Shape caía
  na rota em série (1,4× com 32 fios). Binário com símbolos: `CARGO_TARGET_DIR=target/perfsym CARGO_PROFILE_SMOKE_STRIP=none`.
- **zsh não parte `$files`**: listas de ficheiros para `git add/commit/checkout` vão em `bash -c '…'` (um `git checkout HEAD~1 -- $files`
  falhou em silêncio e a «base» correu sobre a árvore nova).
- A cache de imagens do Vello é por id do `Blob`: `Blob::new` a cada quadro sobe a imagem inteira.
- Sobrepor as pegadas da fatia seguinte à película (`rayon::join`) foi RECUSADO medido (13,4–14,4 × 12,8 ms/q).
- Vermelhos por decisão (não mexer): os do handoff n.º 2 §7.
