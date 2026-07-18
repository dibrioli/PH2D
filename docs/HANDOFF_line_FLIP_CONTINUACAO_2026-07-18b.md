# Handoff — `line/FLIP`, continuação (2026-07-18 **b**) · **COMECE AQUI**

> Sucessor do [`…2026-07-18`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-18.md) (aquele descreve
> o estado *antes* desta rodada; este é o delta).
> **Regime:** Modo L, worktree `Worktrees/line-FLIP`, branch `line/FLIP`.
> **Você NÃO integra nem pusha** (§0.7 do CLAUDE.md) — fecha a fatia, escreve o handoff, PARA.
>
> **Leia primeiro:** `CLAUDE.md` §0 → [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
> → **este arquivo** → **[`Flip/09_colorize.md`](Flip/09_colorize.md)** (o plano da wave —
> é ele que impede a fatia 3 redescobrir a decisão da fatia 1) → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md).

---

## 0. ⚠️ O 1º smoke JÁ ACONTECEU e derrubou um bug do W4 (BUGS #19) — **RE-SMOKE PEDIDO**

O Enio smokou a C1 e reportou: *"independente do valor de gap ou trap o fill se ajusta
perfeitamente à linha até o momento em que se sobreponham duas linhas"*. **Não era o Trap**: é
o `filled_shape_target` (BUGS #16/#17) disparando onde não devia — ele roda depois do solver e
**descarta o contorno traçado**, que é justamente o que Gap e Trap movem. Daí os dois parecerem
inertes.

Corrigido no commit seguinte (o critério de área virou um **abraço nos dois sentidos**, com
tolerância medida). **A C1 continua pendente de smoke, agora junto com o fix do #19** — o
roteiro do §1.1 vale, mais os dois casos das fotos: a forma cujo rabo cruza a própria descida,
e a região fechada por dois traços com barriga.

---

## 0.1 — O que os smokes seguintes derrubaram (leia antes de tocar no balde)

| # | defeito | estado |
|---|---|---|
| **#19** | `filled_shape_target` disparava em traço que se CRUZA e pintava o polígono dele (a cunha entre as pontas). Critério era ÁREA, e a forma quebrada passava com 0,7% num teto de 15% | ✅ critério virou **abraço nos dois sentidos**, tolerância medida (fosso de 150×) |
| **alvo vivo** | o último traço/fill era REESCRITO por qualquer mudança de painel, a partir de uma `base` congelada | ✅ **removido por ordem do Enio** (feature inteira, 177 LOC + 7 sítios) |
| **#20a** | `FILL_TUCK_PX` (px) somado a `mean_line_width` (MUNDO desde o §4.C.6) = **100× a dilatação pretendida** | ✅ convertido (`fill_tuck_world`) |
| **#20b** | a dilatação era a **média global** das espessuras, uniforme em todo ponto do contorno ⇒ atravessava a linha fina | ✅ virou **local** (`local_line_width`) |

⚠️ **E o achado de método que vale mais que os três:** OITO gates de pixel do `gpu_fill_fit`
estavam **verdes** com a cor 100 px fora da linha — porque eles **calculam a própria
dilatação** (`width_px + 2.0*tuck`, `:233`/`:596`) em vez de consumir a do produto. Dois deles
se chamam *"a cor nunca transborda a linha"* e *"a linha macia nunca mostra o fundo"*.
**GAP ABERTO nomeado no BUGS #20:** fazer o `gpu_fill_fit` consumir a dilatação do shell, ou um
gate que afirme que os dois números coincidem. Sem isso os oito seguem cegos para a classe.

---

## 1. Estado: a wave COLORIZE começou. **C1 landou, PENDENTE DE SMOKE.**

1 commit novo sobre a base sincronizada. `main` não andou (`HEAD..main` = 0).

**O plano da wave inteira está escrito** em [`Flip/09_colorize.md`](Flip/09_colorize.md) —
arquitetura, constantes já cravadas pela pesquisa, fatiamento, medições e **kill-criterion**.
Leia-o antes de tocar em qualquer coisa; ele responde a maioria das perguntas que aparecem.

### 1.1 ⚠️ SMOKE PEDIDO AO ENIO (é o que destrava a fatia seguinte)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
cargo build --release -p ph2d-host-desktop
PH2D_FLIP_DEMO=1 ./target/release/ph2d-host-desktop
```

1. Modo **Draw**: desenhe um círculo **deixando um vão visível** no contorno (não feche).
2. Modo **Fill** (o balde), clique dentro → deve dar o toast **"Fill leaked"** (é o
   comportamento de sempre, e é o controle: sem ele o teste seguinte não prova nada).
3. Suba o **Trap** (o slider novo, entre Gap e Grow) até ~4-6 px e clique dentro de novo →
   **agora tem de preencher**, sem tocar no Gap Closure.
4. Suba o Trap ao máximo (20) e clique numa região **fina** do desenho → tem de aparecer
   **"Fill: Trap is wider than this area — lower it"** (e não "leaked", que mandaria para o
   lado errado).
5. **Trap de volta a 0 ⇒ o balde tem de ficar exatamente como era** (é opt-in ao bit).

**O que eu NÃO consigo julgar sozinho e é a pergunta do smoke:** com o Trap ligado, a borda
da cor continua encaixando na linha como o §4.C aprovou? A bola muda *quais pixels são a
região*; o resto do pipeline (a dilatação do BUGS #15, a âncora no eixo do #14) é o mesmo
código, então a expectativa é que sim — mas isso é olho, não gate.

## 2. O que a C1 entrega, e as duas coisas que decidem tudo

**O TRAP** — um slider no card do balde. Uma bola de raio `r` não atravessa um vão mais
estreito que `2r` (Zhang et al., TVCG 2009), então o balde para de vazar por line-art aberto
**sem o artista caçar o vão**. `0` = desligado = o balde do W4, byte a byte.

1. **Front-end novo, back-end INTOCADO.** O que a bola muda é *quais pixels são a região*; a
   geometria continua saindo do `trace_contours`/`simplify_ring` do W4, chamados como estão.
   Isto não é economia de digitação: é a garantia de que a borda de uma região do Colorize e a
   de um balde **não podem divergir**. Não reabra essa costura.
2. **A pergunta é de DISTÂNCIA, não de morfologia.** "Cabe uma bola de raio `r` aqui?" é
   `dist(p, tinta) ≥ r`, então uma **EDT exata** (Felzenszwalb-Huttenlocher, `O(N)`) responde
   **para todo raio de uma vez** — o laço de raios decrescentes do paper vira um re-threshold
   do mesmo buffer, em vez de uma rodada nova de erosão por raio. É isto que faz a C1 caber, e
   é o que a C2 vai reusar.

**Arquivos novos:** `ph2d-flip-fill/src/{edt,edt_tests,ball,ball_tests}.rs`.
**A crate `ph2d-flip-colorize` foi criada e DESFEITA** no meio da rodada: "cabe uma bola aqui"
é pergunta **do balde**, mora ao lado do `flood` — e uma crate separada criaria dependência
**circular** (o `fill_at` precisa da bola). A crate volta na C2, para o LazyBrush, que é
pergunta nova de verdade ("de quem é este pixel").

## 3. Sítios FOUNDATIONAL / de colisão provável (§1.5.9)

| sítio | o que entrou | risco |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/flip.rs` | **`FLIP_TRAP`**, **`FLIP_TRAP_NUM`** (ids novos) | baixo — append no bloco do Flip; `node_id_collisions` verde |
| `.typos.toml` | +1 palavra pt-BR (`dependente`) | **arquivo compartilhado** — só ADIÇÃO; funda contra a main do dia ([[feedback_a_shared_list_is_merged_against_todays_main]]) |
| `ph2d-flip-fill` | `FillParams.trap_px` (campo novo) + `FillError::BallTooFat` (variant novo) | **quebra struct-literal**: 4 sítios atualizados (`tests.rs`, `gpu_fill_fit.rs`, shell). Quem construir `FillParams` sem `..Default::default()` precisa do campo |
| `ph2d-tool-flip` | `TRAP_MAX_PX`, `FlipStyleSnapshot.trap` | baixo |

**Schemas: NENHUM bump.** `PROJECT_SCHEMA` **18**, `FLIP_SCHEMA_VERSION` **8**,
`VEC_SCENE_SCHEMA_VERSION` **8**, pin `(18, 8, 8)` — intactos. O Trap é parâmetro de
ferramenta, não estado de documento.

## 4. Gates, e o LEDGER DE MUTAÇÃO (nenhum gate entrou sem prova)

47 no `ph2d-flip-fill` (eram 38) · 21 no seam do painel (eram 20) · flip 101 · tool 14 ·
arquitetura (node_id_collisions, wiring parity, no_magic, LOC caps ×3) · clippy limpo ·
typos limpo · shell sem falha.

| mutação | o que morre |
|---|---|
| a EDT larga a passada de coluna | exatidão, ponto-único, faixa-f32, conjunto-vazio (4) |
| leitura da linha invertida | exatidão, faixa-f32 (2) |
| a sentinela vaza como distância medida | **só** o conjunto-vazio — prova por camada |
| o raio da bola é ignorado | não-atravessa-vão, fora-da-forma, não-cabe (3) |
| "segurança": nunca reporta vazamento | **só** o converso (vão largo ainda vaza) |
| a janela da dilatação aperta (`pad = 0`) | a equivalência com a grade inteira + o vão |
| o `trap_px` é aceito e **ignorado** | os 2 gates comportamentais da C1 |
| o arm de `FLIP_TRAP` sai do `event.rs` | o seam ("falta o arm em event.rs") |

⚠️ **Um gate foi REMOVIDO por ser verde decorativo** (espelhamento na EDT): nasceu verde,
**nenhuma** mutação o derrubou, e a força bruta já cobre a propriedade em grades
não-quadradas. O porquê ficou escrito em `edt_tests.rs` para ninguém o reescrever achando
que falta. *Um gate que mutação nenhuma mata aumenta a confiança sem aumentar a proteção.*

## 5. Perf — MEDIDA, e a alavanca que sobrou está BARRADA por disciplina

Régua no repo (`measure_the_product_grid_and_ball_cost`, `--release -- --ignored --nocapture`),
com os números do PRODUTO; tabela completa em [`09_colorize.md` §7.1](Flip/09_colorize.md).

| arte na tela | grade | Mpix | antes | **agora** |
|---|---|---|---|---|
| 1080 px | 1768² | 3,13 | 61,5 ms | **33,3 ms** |
| 1920 px | 3113² | 9,69 | 216,0 ms | **110,5 ms** |
| 3840 px | 4096² | 16,78 | 744,2 ms | **321,9 ms** |

Duas alavancas **single-thread**, as duas byte-idênticas e gateadas: o buffer da EDT é `u32`
(a passada é limitada por BANDA, e a maior soma é ~84 M) · a dilatação de volta roda só na
**bbox do núcleo folgada de `r`** (janela, não aproximação — há gate comparando com a EDT de
grade inteira).

> ⚠️ **A alavanca restante é `rayon`, e ela precisa de ORDEM DO ENIO.** Sobra a EDT global
> (67% do custo). O [ADR-0109](architecture/decisions/0109-rayon-exception-watercolor-composite.md)
> sancionou `rayon` **só** no composite do Painter e diz que **não** o abre para o resto do
> codebase. As três invariantes que qualificam a exceção (sem redução entre pixels · sem
> estado mutável compartilhado · sem RNG/transcendental) a EDT **cumpre**, e o precedente
> exigia alavancas single-thread esgotadas — as duas acima acabaram de ser colhidas. **Não
> abra `rayon` por conta própria** ([[feedback_documented_decision_chesterton_fence]]): peça,
> com esta tabela, e um ADR novo.

## 6. ⚠️ O backlog estava MENTINDO (corrigido, e vale a lição)

O `06_fill_balde.md` §8 listava **"T4.5 — Fill multiframe"** como carry-over, e o handoff
desta rodada (§3.1) repetiu: *"o que falta é o wiring do RANGE"*. **Está feito desde o W7** —
`shells/desktop/src/flip_fill.rs:491-519` já roda o balde em todas as chaves selecionadas na
tira, com as duas decisões difíceis tomadas e comentadas (falloff sempre 1.0; quadros
vizinhos preenchem em silêncio). O `06 §8` foi corrigido.

Isto quase me fez construir o que existe — que é exatamente o que a lição do módulo de áudio
diz (*"uma lista de pendências velha faz a próxima LLM propor construir o que já está lá"*).
**Antes de implementar um item de backlog, `grep` o produto.**

## 7. A fila (depois do smoke da C1)

- **C2 — LazyBrush num quadro.** É o coração: rabisco + paleta → N regiões coloridas por um
  **corte multiway**, resolvido por **cortes binários guloso um-contra-todos** (9-18× mais
  rápido que α-expansion, ΔE ≤ 0,04% — `04 §3`; **não** implemente α-expansion). Precisa de
  um **max-flow**, que **não existe no repo** (verificado com controle positivo) — escreve-se
  do paper de **Boykov & Kolmogorov, PAMI 2004**; ⚠️ a implementação de referência deles é
  **GPL**: só comportamento, nunca os bytes (a mesma disciplina do Blender e do Ciallo).
  **Meça o corte binário na grade do §7.1 ANTES de construir a UI** — é o número que decide
  entre síncrono e o padrão `progress` do `editor-core`, e o kill-criterion está escrito.
  ⚠️ E o `04 §3` ordena **trapped-ball antes do LazyBrush** por uma razão **estrutural**: a
  pré-segmentação transforma o corte de *milhões de pixels* em *dezenas de regiões*. A C1 que
  acabou de landar pode ser o que torna a C2 tratável — considere isso antes de montar o grafo
  no pixel.
- **C3 — onion fill.** O range **já está encanado** (§6): o que é novo é a **semente** (um
  rabisco em coordenadas de mundo por cima das poses empilhadas, alimentando o `D_p` de cada
  quadro), não o laço multiframe.
- **A UI de cada fatia entra COM a fatia** — nunca uma fatia "C4 = a UI" no fim (DIRETIVA §2).

Segue adiado, sem mudança: **W6** (timeline global — só com ordem explícita do Enio) · os
carry-overs conscientes do §4 do handoff anterior · congelar o contrato do `ph2d-flip`.

## 8. Comandos

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP

# inner loop
cargo check -p ph2d-flip-fill -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-host-desktop

# a régua de perf (a tabela do §5)
cargo test -p ph2d-flip-fill --lib --release measure_the_product_grid -- --ignored --nocapture

# fechamento do bloco (1× sobre o diff acumulado)
rustfmt --edition 2024 <os SEUS arquivos>   # ANTES de medir LOC — o fmt re-expande
cargo test -p ph2d-flip -p ph2d-flip-fill -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-host-desktop
cargo test -p ph2d-editor-core --test node_id_collisions \
  --test architecture_panel_wiring_parity --test no_magic_numeric \
  --test architecture_panel_loc_cap --test architecture_workspace_file_loc_cap
cargo test -p ph2d-host-desktop --test file_loc_caps
cargo clippy -p <suas crates> --all-targets && typos
```

**LOC a vigiar** (medido pós-fmt): `raster.rs` **679**/700 · `flip_fill.rs` **571**/600 ·
`tool.rs` 566/700 · `paint_sections.rs` **559**/600. Os dois em negrito têm pouca folga —
campo novo ali orça módulo irmão, não crescimento.
