# Prompt de continuação — linha PainterWatercolor · 12/11 n.º 5: o Painter plenamente otimizado (o que sobra)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_5.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`** (merge-base `bc4a6ce930`; quem funde é o
integrador que o dono nomear).

## Ordem do dono (verbatim, continua)

12/11: «vamos seguir até ter o app plenamente otimizado». «Se esforce para resolver todos os problemas de performance do módulo a todo
custo. … Não esqueça do handoff de integração ao final de tudo. Não integre.»

## Leia primeiro

1. [HANDOFF_INTEGRACAO_…_SELECAO_CAMADAS_E_RITMO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md)
   (n.º 4: §4 o que entrou, §8 aberto) e os três anteriores (`…_MISTURA_E_ENVIO.md`, `…_LUZ_E_CAMADAS.md`, `…_DESEMPENHO_TOTAL.md`),
   que continuam a valer.
2. `../missao_4k/BACKLOG.md` e `BACKLOG_AUDITORIA.md`; `../missao_4k/medicoes/DEPOIS_N4.md` (a matriz do app, com o aviso de carga).

## Estado (fim da n.º 4)

| onda | estado |
|---|---|
| 3 mistura de cor | FEITA |
| 4 água do Wet | B13.6 feito até ao piso serial do advect (ADR-0145); além disso só com paralelo novo + ADR |
| 5 camadas e seleção | FEITA: B20.11, B01.A1, B40.A3–A5, B19.10 (resta o ruído espalhado ¼ α=0, +4–10 %); B20.6 a parte da placa por medir |
| 6 cadência e caneta | B22.1 FEITO (passos fixos de 1/60 s, o mesmo bit a 30/60/144 Hz); B22.4 RECUSADO (argumentado); B23.2 por atacar |

## O que fazer, por ordem (até ao fim, sem parar)

1. **Re-medir com calma** (load < 5): a matriz do app, `bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel`, e o
   **ruído espalhado do B19.10** (`mede_as_tiras_de_cache_4k`). A matriz a load 9–90 não compara com o ANTES; o soltar da aquarela
   (27,9 → 35–40 ms nos dois binários) é anterior à janela ou carga — confirme.
2. **B20.6, a parte da placa:** `write_texture` e o anel de envio (não há StagingBelt); meça antes, A×B no mesmo binário.
3. **O que a matriz nova mostrar como pior célula**, e **B23.2**.
4. Por item: medir antes (A×B no mesmo binário, mínimo), curar AO BIT, gate por nome com mutação, medir depois, comitar com a tabela.
5. **Fecho:** fmt/clippy `PH2D_SUITE=fecho` das crates tocadas, alarmes de pincel por nome (comparar o fnv das miniaturas com a lista do
   n.º 4 §5 — a mesma = ao bit), matriz final do app (`mede_pen_up_4k.sh matriz`), handoff de integração NOVO + link no CLAUDE.md §5.1,
   `rm -rf target/*/incremental`, binário smoke compilado 2×, limpar `.ph2d-*`. Não integre, não envie.

## Regras aprendidas (12/11 n.º 4)

- **O subagente não corre `--ignored` nem a fila pesada:** as sondas pesadas são do coordenador.
- **Relógio com load > 5 só por A×B intercalado ou a 1 fio** (`RAYON_NUM_THREADS=1`): a matriz do app a load 60–90 inventou uma regressão
  da aquarela, desmentida por A×B (`986aab640d`).
- **Resumo/cache por identidade `Weak` do `Arc` não fica velho:** `make_mut`/`get_mut` respeitam o `Weak`.
- **Uma fixture pode perder o fenómeno quando uma cura muda o que o pen-up relê** (`060b3fe257`: o pen-up só relê traço ⊕ pad). O
  controlo do oráculo («o velho reprova») tem de existir em cada gate de oráculo.
- Herdadas: perfil a 1 fio separa o custo do paralelismo; a cache de imagens do Vello é por id do `Blob`; listas de ficheiros para
  `git add/commit` vão em `bash -c '…'` (zsh não parte `$files`).
- Vermelhos por decisão (não mexer): os do handoff n.º 2 §7, mais as 3 miniaturas do n.º 4 §7 enquanto o dono não decidir.
