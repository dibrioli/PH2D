# Prompt de continuação — linha PainterWatercolor · 13/11 n.º 8: o Painter plenamente otimizado (o que sobra depois da onda 7)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-13_desempenho_8.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`** a menos que o `main` tenha andado (`git log HEAD..main`
não vazio): a linha está sobre o `main` `4f46a3573c`. Cópia de segurança de antes do rebase de 12/11: ramo
`backup/PainterWatercolor-antes-rebase-2026-11-12` (`8246bd897a`). Quem funde é o integrador que o dono nomear.

## Ordem do dono (verbatim, continua)

12/11: «vamos seguir até ter o app plenamente otimizado». «Se esforce para resolver todos os problemas de performance do módulo a todo
custo. … Não esqueça do handoff de integração ao final de tudo. Não integre.»

13/11 (as três decisões, verbatim «1- tente · 2- pode · 3- sim»): 1) sim a 6 traços em tempo real (mudar a aritmética da `assenta`);
2) desfazer em janela/tiles do vidro e do canvas por traço; 3) o «+» a 4K com o desfazer a lembrar menos passos. **Feitas na onda 7**
(ver o estado). Pendente do dono: ligar ou não `PH2D_ASSENTA_PALPITE=1` (as pranchas estão em `../missao_4k/medicoes/n7_assenta/`), e se
libera `unsafe` num módulo isolado para AVX2 (política).

## Leia primeiro

1. [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_ONDA7_DESEMPENHO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_ONDA7_DESEMPENHO.md)
   (n.º 7: §0 colisão e o tecto de LOC, §5 a matriz final, §6 o ABERTO, §7 recusas, §10 regras).
2. O n.º 6 [PRIMEIRO_TOQUE_4K](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_PRIMEIRO_TOQUE_4K.md) e os anteriores nele linkados.
3. As notas do bloco que for tocar: `../missao_4k/medicoes/N7_CAMADAS.md` (gestos de camada), `N7_ITEM3_mais.md` («+», fundo do toque,
   item 5), `N7_ITEM2_vidro_tiles.md` + ADR-0205 (vidro), `N7_ITEM1_sim6.md` (sim a 6 traços).
4. `../missao_4k/BACKLOG.md`: B18.6–B18.12, B13.12, B19.10, B20.6, B23.2 (`grep -n '^| B18\.'` + `Read` com offset).

## Estado (depois da onda n.º 7)

| item | estado |
|---|---|
| sim a 6 traços: 0,77 → ~0,91 sim-s/s ao bit; palpite 0,97 atrás de chave | FEITO o que cabe sem `unsafe`; o palpite espera o dono |
| ADR-0205: vidro no journal por tile (cópias traços 2–8 `[1;7]` → `[0;7]`) | FEITO; aberto o `fork_par_raw` da tela |
| «+» sem diff, tela pelo mmap, compressão no silêncio, janela por palavras | FEITO («+» Wet 34–39 → 6–10 ms) |
| fundo do toque aquecido em fio (troca, «+», troca de pincel) | FEITO (1.º pen-down WB na nova 33 → 12–18 ms) |
| gestos de camada: base digital/WB/Wet ~11–23 ms; nova digital 13–18 | FEITO; abertos a nova WB/impasto/Wet |
| alarmes de pincel 77 = 77; os 7 vermelhos idênticos à base | sem regressão; aprovar é do dono |
| 35 gates novos verdes por nome (31 + 3 + 1), na HEAD depois do clippy | FEITO |
| tecto de LOC 700: `undo_planes.rs` 764 · `watercolor_vidro_journal_tests.rs` 781 · `watercolor_backdrop.rs` 737 | FEITO (`b41453c3c5`, por corte) |

## O que fazer, por ordem (até ao fim, sem parar)

1. ~~**Tecto de LOC** (B18.9)~~ — FEITO em `b41453c3c5` (por corte, só movimento, 3190 testes antes e depois).
2. **WB duplicar/apagar/grupo na camada NOVA: 72–90 ms** (B18.6; vidro + papel de cor): a prova «a figura só muda em tinta ∪ vidro»
   com o vidro de cada camada na janela; gate ao bit contra a recomposição inteira, controlo e mutação.
3. **Impasto na camada nova: 37–46 ms** (duplicar 46,2 · apagar 37,4 · grupo 41,1 · pen-up 40,4): medir antes, achar o mecanismo.
4. **Wet a trocar na camada nova: 35–60 ms** (Filbert 59–61; Wet Liner 50–65; Express Oil 34–45): ler o `[quadro-lento]` dos logs da
   matriz; hipóteses: o fio da compressão entre 300–760 ms depois do «+», o 1.º aquecimento.
5. **Ctrl+Z do Wet 46–51 ms** e **Finger Tip Ctrl+Z 35 ms** (B18.11): decompor o quadro do Ctrl+Z.
6. **Toque do Filbert 43 e do Composite Brush 37 ms** (34–43 na nova).
7. **CPU parado 50–54 %** (B23.2: o resto da shell — outro módulo; registar, não invadir).
8. **B19.10**, **B20.6** (anel da placa), a **espera activa ~10 % do rayon** (`crossbeam_epoch`).
9. **A espera do pen-down pelo motor** (item 5, 3,8 + 2 ms): documentada em `N7_ITEM3_mais.md`; medir com `sonda_partes::parte` antes de
   qualquer cura; adiar o pen-down inteiro é arriscado.
10. **`fork_par_raw`** (B18.10, aberto do ADR-0205): confirmar o `strong` da tela no pen-down Wet; fork por tile só se medir.
11. **Os 7 vermelhos antigos e os 77 alarmes**: decisões do dono. Só ele regrava (`PH2D_PINCEIS_APROVA=1`).
    ✅ **Dono 13/11 (n.º 8), decidido:** «Ambas SIM, mas como opções a mais e não a substituição do que já existe. OU seja: 2
    checkbox numa seção nova: performance (card novo no painel).» ⇒ cartão novo «Performance» no painel do Painter com 2
    checkboxes, DESLIGADOS por omissão: (a) o palpite da `assenta`; (b) AVX2 num módulo `unsafe` ISOLADO do motor Wet (ao bit
    contra o escalar, cai no escalar sem AVX2). Feito logo depois do item 2.
12. Por item: medir antes (A×B no mesmo binário, mínimo; tempo real = MÉDIA), curar AO BIT, gate por nome com mutação e controlo, medir
    depois, comitar com a tabela. **Fecho:** fmt (só os ficheiros da linha) / clippy `PH2D_SUITE=fecho`, alarmes de pincel por nome,
    a matriz do app normal e `NOVA=1` (o coordenador), handoff NOVO + link no CLAUDE.md §5.1, `rm -rf target/*/incremental`, binário
    smoke 2×. Não integre, não envie.

## Regras aprendidas

Novas (13/11 n.º 7):
- **O limiar do `mmap` do glibc tem tecto de 32 MiB:** a tela 4K (31,6 MiB) depois de um `free` sai do heap com `memset` (18–29 ms).
  Peça 32 MiB + 4 KiB e faça `truncate(n)`.
- **Um fio de fundo com `into_par_iter` rouba o pool rayon:** a contenção aparece em quem corre a seguir. Espere o silêncio da caneta
  antes de lançar, ou dê-lhe um pool próprio.
- **Teste com relógio fixo não vê a marca suja pendente da sim viva do app:** só a matriz do app a mostra (onda 4 das camadas).
- **O subagente não corre `--ignored` nem a fila pesada:** o coordenador corre as sondas (num script).
- **A matriz normal e a `NOVA=1` escrevem no MESMO directório** (`target/mede-pen-up/matriz/`): mova os logs entre as corridas.
- **`unsafe_code = "forbid"` na workspace barra AVX2:** é política, não decisão da linha.
- **Código só de debug chamado da produção parte o build `smoke`:** confira com `cargo check --profile smoke`.
- **Num zsh, `$N` sem aspas não se parte em palavras:** vários nomes chegam ao libtest como UM filtro e o script diz «0 testes» (exit 4).
  Use `${=N}` ou um array.
- **`*_tests.rs` dentro de `src/` conta no tecto de LOC de 700.**

De 13/11 n.º 6:
- «MB novos» de `/proc/self/stat` contam o PROCESSO inteiro (e faltas de leitura da página-zero); para o fio, `/proc/thread-self/stat`.
- Cache por `Weak` da imagem de cada camada + `PartialEq` da pilha é fresca por construção.
- Chave A×B global em teste precisa de trava (ou ser por fio).

De 12/11 n.º 4–5 (ainda valem):
- A régua do tempo real é a MÉDIA, nunca a mediana.
- Um `Weak` como chave de identidade morre quando alguém faz `make_mut` com ele vivo.
- `perf -e page-faults` sem `-g` acha a cópia mas não diz quem: instrumente por fases.
- `cargo fmt --all` formata ficheiros de outras linhas vindos do main: formate só os da linha.
- Relógio com load > 5 só por A×B intercalado ou a 1 fio.
- Um conserto que tira trabalho de um gesto pode só MUDÁ-LO de lugar: meça o gesto seguinte.
- Vermelhos por decisão (não mexer): os 7 por nome e os 77 alarmes enquanto o dono não decidir.
