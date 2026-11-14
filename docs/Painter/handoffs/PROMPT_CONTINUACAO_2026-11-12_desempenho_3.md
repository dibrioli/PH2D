# Prompt de continuação — linha PainterWatercolor · 12/11 n.º 3: o Painter plenamente otimizado

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_3.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`** (o main andou ~593 commits de
OUTRAS linhas; merge-base `bc4a6ce930`; quem funde é o integrador que o dono nomear).

## Ordem do dono (verbatim)

12/11: «vamos seguir até ter o app plenamente otimizado». Antes (11–12/11): «Se esforce para resolver todos os problemas de
performance do módulo a todo custo. Se necessário, auditoria exaustiva. Não esqueça do handoff de integração ao final de tudo.
Não integre.» · «A próxima janela deve ir até o fim da implementação sem parar. Ao final … escrever o handoff de integração pois
outra linha vai integrar.»

## Leia primeiro

1. [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md)
   (o da janela n.º 2: §7 decisões do dono, §8 aberto, §9 riscos) — e o anterior, `…_DESEMPENHO_TOTAL.md`, que continua a valer.
2. [PLANO_DESEMPENHO_TOTAL_2026-11-12.md](../missao_4k/PLANO_DESEMPENHO_TOTAL_2026-11-12.md) — as 7 ondas e as recusas medidas;
   os itens em `../missao_4k/BACKLOG.md` e `BACKLOG_AUDITORIA.md`.

## Estado das ondas (12/11, fim da janela n.º 2)

| onda | estado |
|---|---|
| 1 toque e Ctrl+Z | FEITA (Wet Ctrl+Z 34 ms). Resto = §8 do handoff (troca do motor presa à memória → chunk esparso como tile 64×64, ADR-0204 adenda 2) |
| 2 cerdas largas | parcial: W5 ainda ~0,3 s/dab; depósito fora do quadro = DONO; cortes ao bit esgotados (ver recusas) |
| 3 **mistura de cor** | **NÃO ATACADA**: B28.1 · B28.2 · B28.4 · B32.A1 — Wet blend 21,6 ms/q (alvo 16,7) · Wash Mix 24 ms/q. (B28.3 `omp critical` = recusa) |
| 4 água do Wet | parcial: B11.3 feito; **B13.5/B13.6 (o passo cresce com os traços: 26 → 8 img/s no 4.º traço) por atacar**; B13.7 = dono |
| 5 camadas e compositor | B18.1–18.3, B39.A3 feitos; **por atacar: B19.10, B01.A1, B20.6 (upload Full com máscara viva 17–28 ms/q), B20.11 (formigas da seleção), B40.A3–A5** |
| 6 cadência e caneta | B27.1 = recusa; B21.1 Mailbox = linha da UI (só handoff); **por atacar: B22.1/22.4, B23.2** |
| 7 memória | B42.3/B17.9 = orçamentos = DONO (só se a régua Z10 mostrar penhasco) |

## O que fazer, por ordem (até ao fim, sem parar)

1. **Onda 3 (mistura de cor)** — o maior ganho aberto sem decisão do dono. Sondas: `sonda_custo_no_app`, as da `mistura_opaca`.
2. **Onda 4: B13.5/B13.6.** 3. **Onda 5: B20.6, B20.11, B19.10, B01.A1, B40.A3–A5.** 4. **Onda 6: B22.1/22.4, B23.2.**
5. Por cada família: medir antes, curar AO BIT (ou com a decisão do dono registada), medir depois, comitar com a tabela.
   Delegue a agentes em worktrees próprias (`.ph2d-ag-<x>`, target próprio, prazo 40 min, `cherry-pick -x`); o subagente não
   pode correr sondas `--ignored` (o hook recusa): ele deixa um script e a janela principal corre-o.
6. **Fecho:** fmt/clippy `PH2D_SUITE=fecho` das crates tocadas, tectos, testes por nome, alarmes de pincel (largura: Oil Wash,
   Smear Wet; miniatura: 74 — nunca regravar sem o dono), matriz final do app (roteiro
   `docs/Painter/ferramentas/mede_pen_up_4k.sh matriz`; como foi feita: §4 do handoff n.º 2), handoff de integração NOVO + link no
   CLAUDE.md §5.1, `rm -rf target/*/incremental`, binário smoke compilado 2×, limpar `.ph2d-*`. Não integre, não envie.

## Regras aprendidas (12/11)

- ⛔ **A × B entre DUAS compilações: `CARGO_INCREMENTAL=0` nas duas** — o incremental do perfil smoke muda o código gerado até
  1,3 % (dois «−2,9 %» comitados eram isso; `docs/DevOps/MEDIR_VELOCIDADE.md` passo 6). Melhor ainda: variantes no MESMO binário.
- Máquina carregada (outra sessão integra): instruções (`perf stat -e instructions:u`) ou A×B intercalado, mínimo.
- RTX proibida nas medições (Radeon: `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json`). Pesado por
  `bash scripts/ph2d-run.sh`; testes por NOME; o `perf` não grava pilha (binário com símbolos em target à parte, `perf annotate -l`).
- Vermelhos por decisão (não mexer): `a_vista_a_400_por_cento…` (wet-paint), `o_oleo_pouco_diluido…`, `impasto_shine_glints…`,
  `wax_bleeds…` (tool-painter).
- Resposta ao dono: curta, PT-BR, sem jargão, smoke em passos numerados com o `cd`.
