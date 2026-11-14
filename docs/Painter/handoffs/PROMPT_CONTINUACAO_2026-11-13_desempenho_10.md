# Prompt de continuação — linha PainterWatercolor · 13/11 n.º 10 (depois da janela n.º 9)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-13_desempenho_10.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`, sobre o `main`
`4f46a3573c` (não faça `git rebase main` se `git log HEAD..main` for vazio). Ordem do dono (continua): «vamos seguir até ter o app
plenamente otimizado» · «faça o que for melhor para resolver tudo logo» · 13/11: **«não integre»**. Não integre, não envie.

## Leia primeiro
1. O handoff de integração destas janelas: [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_N8_N9_DESEMPENHO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_N8_N9_DESEMPENHO.md)
   (§0 colisão, §3 ABERTO, §5 regras). Ele soma-se ao [n.º 7](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_ONDA7_DESEMPENHO.md).
2. [`../missao_4k/medicoes/N9_NOTAS.md`](../missao_4k/medicoes/N9_NOTAS.md) e [`N8_NOTAS.md`](../missao_4k/medicoes/N8_NOTAS.md).

## Feito na janela n.º 9
| commit | o quê |
|---|---|
| `8a1714c8d0` | item 2: a absorção do escorrido COMPÕE o vidro do topo com o journal — pen-down 46–64 → 14,2 ms, Ctrl+Z 71–93 → 35,6 (sonda 4096²) |
| `f3ab7e8f5a` | os vermelhos do filtro `wet` são da base, não do item 2 |
| `c166588404` | item 3: a troca com a tinta viva do Wet dobra na placa só a parte do relevo (`mark_dirty_so_a_figura`, `preview_gpu_relevo`; chave `PH2D_DOBRA_A_REGIAO=1`) |
| (fecho) | N9_NOTAS, este prompt, o handoff N8_N9, o link no CLAUDE.md §5.1 |

## Aberto, por ordem
1. **Régua de CALMA** da matriz NOVA=1 (Impasto, WB, e o Wet com o item 3): `target/n9/bateria1.sh` / `bateria2.sh` são os moldes;
   só com `uptime` < 5 antes e depois de cada pincel (nesta janela outra linha subiu a carga a 19–35 a meio).
2. **O resto dos gestos do Wet** (handoff §3.2): a composição da troca na placa (a ótica muda na janela da tinta inteira — medir se
   a janela pode ser só a das células vivas da sessão); o Ctrl+Z na camada nova (dobra 12 ms: legítima, medir a janela); os quadros de
   35 ms FORA do Painter (perfil da shell no quadro: `PH2D_PAINT_PERF` não a cobre).
3. **Impasto duplicar**: 7 de 35 ms no Painter; o resto fora.
4. Itens 6–10 do prompt n.º 8 (toque Filbert/Composite, CPU parado = shell, B19.10/B20.6/rayon, espera do pen-down, `fork_par_raw`).
5. Vermelhos anteriores: só listados (handoff §3.5).
6. Fecho de cada onda: fmt/clippy das crates tocadas (`PH2D_SUITE=fecho`), alarmes de pincel por nome, handoff NOVO + link no §5.1,
   `rm -rf target/*/incremental`, binário smoke 2×.

## Regras novas (n.º 9)
- Oráculo que partilha uma etapa com a pista nova não a testa (o `SEMPRE_INTEIRA` partilhava a dobra) — refaça CADA etapa tocada.
- Mutação verde: confira primeiro se o CENÁRIO exerce o caminho (imprima a decisão por passo), depois desconfie da cura.
- Teste da placa sem adaptador «passa» calado: confirme com `--nocapture`.
- O cwd de um teste é a pasta da crate: `target/…` relativo vai parar a `crates/<c>/target/`.
- Um comando recusado por um hook (o `commit-barato`) leva TUDO o que vinha na mesma linha — confira o que ficou por criar.
