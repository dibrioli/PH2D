# Prompt de continuação — linha PainterWatercolor · 13/11 n.º 7: o Painter plenamente otimizado (o que sobra)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-13_desempenho_7.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`** a menos que o `main` tenha andado (`git log HEAD..main`
não vazio): a linha está sobre o `main` `4f46a3573c`. Cópia de segurança de antes do rebase de 12/11: ramo
`backup/PainterWatercolor-antes-rebase-2026-11-12` (`8246bd897a`). Quem funde é o integrador que o dono nomear.

## Ordem do dono (verbatim, continua)

12/11: «vamos seguir até ter o app plenamente otimizado». «Se esforce para resolver todos os problemas de performance do módulo a todo
custo. … Não esqueça do handoff de integração ao final de tudo. Não integre.»

## Leia primeiro

1. [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_PRIMEIRO_TOQUE_4K.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_PRIMEIRO_TOQUE_4K.md)
   (n.º 6: Wet e WB a 4K, o compositor, as recusas medidas §4, o ABERTO §6).
2. [HANDOFF_CONTINUACAO_2026-11-12_ONDA5_VIDRO_E_MEDIA.md](HANDOFF_CONTINUACAO_2026-11-12_ONDA5_VIDRO_E_MEDIA.md) (n.º 5) e os anteriores nele linkados.
3. `../missao_4k/BACKLOG.md` (B18.4 «Cont. 7», B19.10, B13.12) e `../missao_4k/medicoes/DEPOIS_N4.md` (a matriz do app, com o aviso de carga).

## Estado (depois da onda n.º 6)

| item | estado |
|---|---|
| Wet 1.º traço camada nova (3.º evento 20→14–16 ms; pen-down 2.º traço 10–12→6,7–7,1 ms) | FEITO (n.º 6) |
| WB pen-down (base 2.º 28→14–15 ms; camada nova 2.º 31–35→14,7–15,7 ms) | FEITO (n.º 6) |
| compositor: `cache_hit_skips_below_layers` vermelho e resumo flaky | FEITO (n.º 6) |
| alarmes de pincel: 77, lista idêntica antes e depois do rebase | FEITO o diagnóstico; aprovar é do dono |
| matriz do app | NÃO medida (load 11–17 na sessão) |
| «+» a 4K 55–65 ms · vidro por traço 115 MB · espera do pen-down 3,8 ms · sim a 6 traços 0,77 | abertos (ver abaixo) |

## O que fazer, por ordem (até ao fim, sem parar)

1. **Matriz do app com calma** (load < 5): `bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel` (e `NOVA=1`).
2. **B19.10 ruído**, **B20.6 anel da placa**, **espera activa de ~10 % do rayon** (pedem calma/placa).
3. **O «+» a 4K** (55–65 ms; `UndoEntry::split` 14 ms): só com decisão sobre a contabilidade do desfazer — pergunte ao dono em palavras simples.
4. **O vidro por traço = desfazer em janela/tiles** (115 MB no 2.º pen-down): ADR antes + a palavra do dono.
5. **A espera do pen-down pelo motor** (3,8 ms + `abre_o_passo` 2 ms): adiar o pen-down inteiro; arriscado, meça antes.
6. **A sim a 6 traços** (0,77 sim-s/s pela MÉDIA): só com decisão do dono (muda a aritmética da `assenta`).
7. **Os 3 vermelhos antigos** (`o_classic_e_o_paper_edge_zero_sao_o_byte_de_hoje`, `slot_paper_preset_tiles_seamlessly_under_tiling`, `a_primeira_marca_do_wet_refaz_a_imagem_inteira`) e **os 74+3 alarmes**: decisão do dono; só ele aprova regravar (`PH2D_PINCEIS_APROVA=1`), relatório em `target/relatorio-pinceis/index.html`.
8. Por item: medir antes (A×B no mesmo binário, mínimo; tempo real = MÉDIA), curar AO BIT, gate por nome com mutação e controlo, medir depois, comitar com a tabela. **Fecho:** fmt (só os ficheiros da linha) / clippy `PH2D_SUITE=fecho`, alarmes de pincel por nome, handoff NOVO + link no CLAUDE.md §5.1, `rm -rf target/*/incremental`, binário smoke 2×. Não integre, não envie.

## Regras aprendidas

Novas (13/11 n.º 6):
- **«MB novos» de `/proc/self/stat` contam também faltas de LEITURA da página-zero:** um plano só lido parece memória nova.
- **A 4K a tela (33,2 MB) fica logo abaixo do limiar de `mmap` do glibc (32 MiB):** as faltas migram entre buffers conforme os `free` anteriores — meça o gesto TOTAL, não uma fase.
- **Cache por `Weak` da imagem de cada camada + `PartialEq` da pilha é fresca por construção.**
- **Chave A×B global em teste precisa de trava** (testes em paralelo viram a rota um do outro): `trava_das_chaves()`.
- **`scripts/cargo-test-narrow.sh` imprime «0 passaram» quando algum teste falha:** confirme com `cargo test` directo.

De 12/11 n.º 5:
- **A régua do tempo real é a MÉDIA, nunca a mediana.**
- **Um `Weak` como chave de identidade morre quando alguém faz `make_mut` com ele vivo:** quem move tem de religar.
- **Alocação ≥ 32 MB é `mmap`:** libertada e pedida por quadro = faltas de página em todo quadro.
- **`perf -e page-faults -c 1` acha a cópia mas sem `-g` não diz quem:** instrumente por fases com `/proc/self/stat`.
- **`cargo fmt --all` nesta árvore formata ficheiros de outras linhas vindos do main:** reverta-os, formate só os da linha.

De 12/11 n.º 4 (ainda valem):
- O subagente não corre `--ignored` nem a fila pesada: as sondas pesadas são do coordenador.
- Relógio com load > 5 só por A×B intercalado ou a 1 fio (`RAYON_NUM_THREADS=1`).
- Uma fixture pode perder o fenómeno quando uma cura muda o que o pen-up relê: o controlo do oráculo («o velho reprova») existe em cada gate.
- Um conserto que tira trabalho de um gesto pode só MUDÁ-LO de lugar: meça o gesto seguinte.
- Antes de construir um salto/sono, meça se os candidatos mudariam o bit.
- Vermelhos por decisão (não mexer): os do handoff n.º 2 §7, as 3 miniaturas do n.º 4 §7 e os 74+3 enquanto o dono não decidir.
