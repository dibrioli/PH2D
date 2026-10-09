# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-09 — F7b: O QUICKSHAPE DEPOIS DO SMOKE DO DONO

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior (o QuickShape inteiro): [F7, 08/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F7_QUICKSHAPE.md)
> — citada, não reescrita. F0–F6: os handoffs dela.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `2a925859d` (o `main` não andou — sem rebase)
  · F0–F7 continuam por integrar e vão JUNTAS com esta jornada.
- Commits desta jornada: `393e874c6..HEAD`.
- **O pedido (smoke do dono, 09/10):** *«Funciona, sem muita sensibilidade, mas funciona. Formas
  pequenas devem funcionar.»*

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro (a frase e as cenas não mudam).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **formas pequenas** | `MIN_SIZE_PX` 94 → **12 px** (o joelho medido, §2); a quina passa a `min(94 px, 0,2 × diagonal)` — o 94 do oráculo só manda nas formas grandes. As fixtures linha 30/60 px e rectângulo 70×50 (cruas no Xournal++) passam a excepções do DONO (22 → 25) | `quickshape.rs` (`Law`), `quickshape_tests.rs` |
| ⭐ **o parado com folga** | `HOLD_SLOP_PX = 10` (= `SLOP_PT` do toque, gate): o meio segundo conta desde que a mão ENTROU na folga; antes, todo tremor > 1,5 px (o passo mínimo do lápis) reiniciava-o — a «pouca sensibilidade» | `pencil.rs` (`hold_tick`), `quickshape_hold.rs` |
| **a lei no meio das faixas** | as constantes passam a um `Law` que a sonda varre NO MESMO PROCESSO; três estavam na BORDA da faixa válida e foram ao meio: elipse `0,04 → 0,065` (faixa `0,02–0,11` — a outra metade da sensibilidade), fusão de viragens `25° → 29°` (`24–44`), recorte do ajuste `0,15 → 0,175` (`0,15–0,20`) | `quickshape.rs`, `quickshape_measure_tests.rs` |
| ⛔ **ACHADO e curado: um S virava zigue-zague** | a sonda das formas pequenas mostrou um S de mão lido como polilinha em 8–38 % das mãos, a TODOS os tamanhos (as cristas cabiam nas quinas-faixa). Cura: numa polilinha ABERTA a quina interior ≤ `0,3 ×` o lado vizinho mais curto | `quickshape.rs` (`open_corner_of_side`) |
| **cena 12** | ganha um 4.º traço: um círculo PEQUENO (raio 15 px) | `vector_mode_quick.rs` |
| organização | `quickshape_geom.rs` (a geometria pura) e `quickshape_hand_tests.rs` — irmãos pelo tecto de 700 LOC | `ph2d-vec-edit/src/` |

## §2 — Medido e decidido (técnico, delegado)

As três sondas (`cargo test --release -p ph2d-vec-edit --lib measure_ -- --ignored --nocapture
--test-threads=1`; tabelas inteiras em [`00_resultados.md`](../vetor_nova_ui/oraculo_quickshape/00_resultados.md) §3d):

- **Formas pequenas, tremor ±1 px** (40 mãos/célula): rectângulo, triângulo, linha, zigue-zague e L
  a 100 % desde 12 px; círculo desde 20 px (95 %); elipse 2:1 desde 32 px (98 %). Abaixo disso a lei
  falha PARA traço livre; «outra forma» só salta a 8 px ⇒ piso **12 px**.
- **O S**: livre em 62–65 % das mãos antes; **95–98 %** com a quina/lado a 0,3 (vale 0,2–0,4: a 0,2 o
  zigue-zague de quinas arredondadas cai a 88 %).
- **As faixas** (constante · medida · faixa válida):

| constante | medida | faixa válida contígua |
|---|---|---|
| `corner_px` | 94 | 70.0000 … 200.0000 |
| `corner_of_diag` | 0.2 | 0.1200 … 0.3200 |
| `rdp_of_diag` | 0.04 | 0.0250 … 0.1000 |
| `merge_turn_deg` | 29 | 24.0000 … 44.0000 |
| `bow_max` | 0.055 | 0.0400 … 0.0750 |
| `bow_trim` | 0.1 | 0.0000 … 0.2000 |
| `fit_trim` | 0.175 | 0.1500 … 0.2000 |
| `close_of_length` | 0.06 | 0.0450 … 0.0750 |
| `close_window` | 0.1 | 0.0500 … 0.3000 |
| `right_tol_deg` | 8 | 7.0000 … 10.0000 |
| `axis_snap_deg` | 5 | 4.0000 … 5.5000 |
| `ellipse_rms_max` | 0.065 | 0.0200 … 0.1100 |
| `round_ratio` | 1.1 | 1.0500 … 1.5000 |
| `turn_tol_deg` | 90 | 50.0000 … 300.0000 |
| `open_corner_of_side` | 0.3 | 0.0500 … 0.5500 |
| `min_size_px` | 12 | 0.0000 … 28.0000 |
| test quickshape::measure_tests::measure_the_s_curve ... |

**Recusas MEDIDAS** para o S (não se reconstroem): a fracção do traço dentro das quinas (o S chega a
0,37 e o rectângulo chanfrado, que tem de passar, a 0,28 — sobrepõem-se); o termo CÚBICO do lado (com
tremor um lado recto ondula igual nas formas pequenas: parar o S custava 88 % dos rectângulos); o
cúbico com significância (o tremor suave é correlacionado e engana o erro-padrão); o cúbico com piso
em px (a inflexão do lado do meio de um S tem ~2 px). A régua que separa é a PROPORÇÃO quina/lado nas
polilinhas abertas: vale `0,2–0,4`.

## §3 — Foundational tocado e contratos

- **`ph2d-vec-edit`**: `quickshape::Law` (`pub(crate)`) e `recognise_with`; `MIN_SIZE_PX` passa a 12;
  `quickshape_hold::HOLD_SLOP_PX` (novo, `pub`); `Hold::tick` recebe o instante de REPOUSO, não o da
  última amostra. Módulos novos `quickshape_geom` (privado), três ficheiros de teste.
- **`ph2d-app-vec`**: gate `HOLD_SLOP_PX == SLOP_PT`; a cena 12 com 4 traços.
- **Contratos congelados (§6): nenhum.** Shell: **intocada** nesta jornada.

## §4 — Superfície de colisão e contadores

Nada que some entre linhas: sem ids, chaves, cenas, schema. Shell `+0`.

## §5 — Gate final

Uma corrida do `verificador` (`BASE=2a925859d`, HEAD `bcf5d13d6`): fmt ✓ · warnings-deny ✓ · **clippy ✗ 2×**
(`needless_range_loop` no `solve`, `type_complexity` na sonda — curados em `b5c12e000`, clippy das crates
re-corrido limpo) · machete ✓ · standalone-optional ✓ · workflow-packages ✓ · censos 12/12 ✓ · `doc-index` ✓ ·
`nextest-impacted` **17 006 / 17 006** (9 770 pulados) · catracas ✓ (`the_shell_only_shrinks`, LOC, CLAUDE.md,
índice da memória). Depois do gate só entraram testes e docs (`6922304ac` e seguintes), corridos à parte.

### Mutação

Agente `mutacao`, verde visto antes, âncoras 1×, árvore limpa no fim. **12 de 15 sangraram**: o piso
(dois sítios), a quina relativa, a regra quina/lado (duas), a elipse, o parado com folga (dois), o gate
da folga, a cena. Sobreviventes: **2 buracos reais** — o tecto absoluto da quina nas formas grandes e o
pivô do solver — fechados por `a_long_thin_rectangle_keeps_its_short_sides` e
`the_solver_pivots_past_a_zero_on_the_diagonal` (vistos vermelhos sob a mutação, verdes limpos);
**2 tolerâncias** (fusão 29→25°, recorte 0,175→0,15: dentro da faixa medida — a sonda documenta, um
teste não deve travar um valor que a medição diz indiferente); **1 equivalente** (`tried_at`).

## §6 — Smoke do dono

Foto: [`f7b_01`](../vetor_nova_ui/fotos/f7b_01_cena12_quatro_tracos_com_um_circulo_pequeno.png).

## §7 — O que fica ABERTO

- **Uma mão de verdade outra vez**: a sensibilidade foi a queixa — o parado com folga e a elipse no
  meio da faixa são as duas curas medidas; só o dono diz se chega.
- **Formas abaixo de 12 px** não reconhecem (o tremor manda). **Elipses** pequenas precisam de ~32 px
  com tremor de ±1 px (círculos ~20 px) — §2.
- O S de mão ainda escapa como zigue-zague em ≤ 5 % das mãos (§2).
- Do F7: arcos e setas; «Edit Shape».

## §8 — Binário de smoke (2.ª corrida)

Depois do `rm -rf target/*/incremental` (5,6 GB) — `bash scripts/ph2d-run.sh cargo build -p
ph2d-host-desktop --profile smoke`, 1.ª corrida 30 s; a 2.ª:

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.25s
```

Varredura no fecho: zero processos meus.

## §9 — `agent-loop-profile.sh`

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                178   alvo: <= 800
  ✗ cargo test : cargo check                452 : 247   alvo: <= 1,0  razao 1.8x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%
  ✗ contexto relido por passo (media)         404 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
(Esta jornada correu na MESMA janela da F7 — contra a lei «uma janela nova por onda»; a próxima onda
desta linha abre janela nova.)
