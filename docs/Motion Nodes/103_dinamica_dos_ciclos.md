# 103 — A DINÂMICA DOS CICLOS (o protocolo desta obra até ao fim)

> **Ordem do Enio, 2026-09-05 — literal, e vale até terminarmos:**
> *«A cada ciclo você deve escolher um grupo de nós já pensando num tutorial ultra interessante e
> didático sobre aquele grupo de nós. Uma vez escolhidos os nós você vai redesenhar e fazer o
> upgrade buscando superar o estado da arte existente no mundo e redesenhar para ficar tão belo
> como o MiniCavalry. Desejamos super performance, facilidade de uso, poder. Ao final do
> redesenho e do upgrade você escreve num PDF de tutos o tutorial referente ao grupo de nós
> reconstruídos. O smoke será o tutorial. Salve essa dinâmica pois assim será até finalizarmos.
> Como no Blender, os parâmetros dos nós devem ser desenhados nos nós e vamos retirar o painel
> lateral.»*

⚠️ **Este doc é o PROTOCOLO, não um plano de uma wave.** Um agente que assume esta linha lê:
este doc · o [102](102_o_outro_patamar_plano_dos_nos_2026-09-04.md) (as portas do código, §0) ·
o [101](101_pesquisa_cartoes_ricos_2026-09-04.md) (o cartão) · e o ciclo aberto na §5 daqui.

---

## §1 — O ciclo, em sete passos (todo ciclo faz os sete, nesta ordem)

| # | passo | entregável | quem valida |
|---|---|---|---|
| 1 | **Escolher o GRUPO** — uma família que o artista reconhece, escolhida **já com o tutorial em mente** | a linha do ciclo na §5 (nós + premissa do tutorial) | — |
| 2 | **Auditar o grupo contra o estado da arte** — params que faltam, poder que falta, o que as referências fazem e por quê | uma secção no doc do ciclo, com fonte por afirmação | — |
| 3 | **REDESENHAR o cartão** dos nós do grupo — params **no cartão**, beleza do Mini Cavalry (§2) | código + gates | o tutorial |
| 4 | **UPGRADE dos nós** — os params/poder que a auditoria achou, com o caminho no **device** | código + gates + prova de mutação | o tutorial |
| 5 | **MEDIR** — a tabela de performance do grupo (CPU · device · passes · objectos/ms) com `loadavg` ao lado | tabela no doc do ciclo | §0.0 |
| 6 | **Escrever o TUTORIAL em PDF** — `docs/Motion Nodes/tutoriais/` (§3) | `<n>_<slug>.pdf` + a fonte `.html` | — |
| 7 | **O SMOKE É O TUTORIAL** — o Enio segue o PDF do princípio ao fim; cada passo é um passo do tutorial | o report dele | **Enio** |

⛔ **Um ciclo não fecha com o passo 4.** Sem a tabela (5) e sem o PDF (6) o ciclo está **meio
feito**, e meio-feito é pior que não começado ([memória](../../project-memory/feedback_perfection_no_deferrals.md)).

⛔ **O tutorial não é documentação do que foi feito: é o SMOKE.** Ele tem de ser executável do
princípio ao fim por quem **nunca viu** aquilo (§0.8): comando completo com o `cd`, o que clicar
com o nome que aparece **na tela**, o que tem de acontecer, e como saber que deu errado. Se um
passo do tutorial não é possível no app, **o ciclo não acabou** — isso é a régua.

## §2 — As quatro leis que valem em TODO ciclo

1. **PERFORMANCE — somos uma game engine** (Enio, 04/09). Todo nó do grupo tem de dizer onde
   corre: no **device** (4,19 M objectos em 3,85 ms) ou na CPU, e **porquê**. Um nó do grupo que
   caia para a CPU sai do ciclo com a razão nomeada e o preço medido
   ([doc 98](98_auditoria_de_performance_2026-09-01.md) · [doc 102 §2](102_o_outro_patamar_plano_dos_nos_2026-09-04.md)).
2. **OS PARAMS VIVEM NO CARTÃO, e o painel lateral SAI** (Enio, 05/09 — decisão de produto).
   Ver §4: a medição que eu tinha escrito contra isto respondia a **outra** pergunta.
3. **A beleza é a NOSSA spec** — a do [plano 01](01_plano_modulo_motion_nodes.md), que o Mini
   Cavalry implementou (o `visual-tokens.js` dele abre com *«Doc PH2D §6»*): silhueta por papel ·
   cabeçalho por categoria · pino por espécie · fio por tipo. Do Blender vêm três
   **comportamentos**: o widget inline que some quando o pino é ligado · a forma do pino = a
   estrutura do dado · o LOD por zoom. Tabela executável: [doc 102 §1.1](102_o_outro_patamar_plano_dos_nos_2026-09-04.md).
4. **Produto final, nunca MVP** — cada nó do grupo shipa o conjunto de params que um profissional
   espera ([memória](../../project-memory/feedback_final_product_every_node_ships_the_full_pro_param_set.md)),
   e o que ficar de fora fica **nomeado com o preço**, nunca em silêncio.

## §3 — O TUTORIAL: formato, ferramenta e regra

- **Fonte** (diffável, versionada): `docs/Motion Nodes/tutoriais/src/<n>_<slug>.html` — HTML com
  os tokens do design system embutidos e CSS de impressão (`@page`).
- **Saída**: `docs/Motion Nodes/tutoriais/<n>_<slug>.pdf`, gerado por
  `bash scripts/tutorial-pdf.sh "docs/Motion Nodes/tutoriais/src/<n>_<slug>.html"`.
- **Ferramenta medida nesta máquina (2026-09-05):** ⛔ não há `typst`, `pandoc`, `weasyprint`,
  `wkhtmltopdf`, `xelatex` nem `libreoffice`. ✅ Há **`google-chrome-stable`** (headless
  `--print-to-pdf`), `rsvg-convert`, `inkscape`, `convert` (ImageMagick) e `pycairo`.
  ⇒ **HTML → Chrome headless → PDF** é o caminho, e é o único que não pede instalação.
- **Estrutura de todo tutorial** (a mesma, para o artista aprender o formato uma vez):
  1. **O que você vai fazer** — uma frase e a imagem do resultado.
  2. **Abrir** — o comando completo com o `cd`, copiável de uma vez.
  3. **Os passos numerados** — cada um: *o que fazer · o que aparece · como saber que errou*.
  4. **O que cada botão faz** — a tabela dos params do grupo, com a unidade e a faixa.
  5. **Vá além** — 3 variações que o artista tenta sozinho.
  6. **Se algo não bater** — os sintomas conhecidos e o que reportar.
- ⚠️ **As imagens do tutorial saem do PRÓPRIO app** (capturas do smoke), nunca desenhos à mão de
  um ecrã que não existe — um tutorial que ensina o contrário do que acontece é pior que nenhum
  ([§5.0 do CLAUDE.md](../../CLAUDE.md)).

## §4 — ⚠️ A minha medição contra «params no cartão» respondia a OUTRA pergunta

O [doc 101 §4](101_pesquisa_cartoes_ricos_2026-09-04.md) recusou *«todos os params no cartão»*
com a tabela *altura do cartão ÷ altura do ecrã* (48 % do iPad mini no `motion.oscillator`,
80 % no `motion.bezier_warp`). **A régua estava errada, e a decisão do Enio é a melhor das duas:**

- o **painel lateral** ocupa o slot do Inspector = **`chrome/inspector-w = 304 px` ABSOLUTOS**,
  em todo quadro, em toda cena: **22,3 %** da largura no iPad 12.9 · **25,5 %** no 11 ·
  **26,8 %** no mini. É área **permanente** e não se recupera;
- o **cartão alto** custa altura **só onde o artista está a trabalhar**, num canvas **infinito,
  panorâmico e com zoom** — e ainda dobra (secções + o nó inteiro).

⇒ *Uma recusa medida responde UMA pergunta, e a minha respondeu «cabe no ecrã?» quando a pergunta
era «o que custa área PERMANENTE?»*
([memória](../../project-memory/feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another.md)).
A recusa do doc 101 §4 fica **REVOGADA por medição**, e o que a substitui:

> **Lei do cartão (nova):** o cartão hospeda **todos** os params do nó, em **secções dobráveis**
> (`ParamGroup`, que já existem), com **LOD por zoom** e widgets vivos **só no cartão quente**.
> O painel lateral de params **sai**. A régua deixa de ser «% do ecrã» e passa a ser
> **`alcance × custo`**: todo param é alcançável sem abrir painel nenhum (gate de censo), e
> nenhum cartão frio regista um widget (gate de contagem).

## §5 — A FILA DOS CICLOS (o ciclo aberto é sempre o primeiro sem ✅)

Os grupos saem do que o **artista** vê na paleta (categoria + sub-cluster), nunca de uma lista
inventada. Contagens do registry em 2026-09-05.

| # | ciclo | nós | tutorial |
|---|---|---|---|
| **1** | **ARRANJO — pôr muitos objectos na tela** | `motion.grid` · `motion.scatter` · `motion.distribute_radial` · `motion.fibonacci` · `motion.lattice` · `motion.voronoi` · `motion.distribute_poisson` · `motion.distribute_curve` · `motion.path` · `motion.clone` (**10**) | **«Do primeiro objecto ao milhão»** |
| 2 | ANIMADORES — fazer andar | `motion.oscillator` · `value.lfo` · `motion.wiggle` · `motion.noise` · `motion.stagger` · `motion.orbit` · `motion.spring` · `motion.delay` | «O tempo entra no grafo» |
| 3 | TRANSFORMES & DEFORMADORES | `move` · `rotate` · `scale` · `transform` · `mirror` · `look_at` · `bend` · `twist` · `spherize` · `four_point_warp` · `bezier_warp` · `kaleidoscope` · `spline_wrap` | «Dobrar o mundo» |
| 4 | FOCO — quem é afectado (campos) | `motion.falloff` · `field.box` · `field.radial_sweep` · `field.index_range` · `field.remap` · `field.combine` · `field.shape` | «Nem todos ao mesmo tempo» |
| 5 | SIMULAÇÃO | `sim.zone` · `sim.spawn` · `sim.step` · `sim.lifetime` · `sim.collide` · `motion.integrate` · as `force.*` | «Deixar a física decidir» |
| 6 | VALOR & PULSO — o cérebro | a família `value.*` e `pulse.*` | «Um número que manda em tudo» |
| 7 | APARÊNCIA (Fx) | `tint` · `color_ramp` · `color_array` · `trail` · `strobe` · `glow` · `drop_shadow` · `rgb_split` · `sub_uv` · `slit_scan` | «A cor e o rasto» |
| 8 | FONTES & DADOS | `source.shape` · `source.object` · `source.text` · `source.table` · `source.lsystem` · `motion.emitter` | «De onde vêm as coisas» |
| 9 | RIG & CORPOS MOLES | `rig.*` · `soft_body` · `verlet_rope` · `wave` · `boids` | «Coisas que se seguram» |

⚠️ **O ciclo 1 carrega o SUBSTRATO** (o cartão passa a hospedar params e o painel sai) — é a única
vez; os ciclos 2+ só pagam o grupo deles. ⚠️ **A ordem dos 2..9 pode mudar** por decisão do Enio;
a do 1 não, porque o resto assenta nela.

## §6 — Onde cada coisa fica (para o agente não procurar)

| coisa | caminho |
|---|---|
| este protocolo | `docs/Motion Nodes/103_dinamica_dos_ciclos.md` |
| o plano técnico (portas do código) | `docs/Motion Nodes/102_o_outro_patamar_plano_dos_nos_2026-09-04.md` |
| o doc de um ciclo | `docs/Motion Nodes/1xx_ciclo_<n>_<slug>.md` |
| a fonte do tutorial | `docs/Motion Nodes/tutoriais/src/<n>_<slug>.html` |
| o PDF | `docs/Motion Nodes/tutoriais/<n>_<slug>.pdf` |
| o gerador | `scripts/tutorial-pdf.sh` |

---

## §7 — CICLO 1 · a MEDIÇÃO do substrato (2026-09-05, load 2,29 / 2,66 — §5.0 ok)

Antes de pôr um param dentro de um cartão (§0.0: medir antes de limitar). Duas sondas
`#[ignore]`, mesma tela (1200×800), `--release`, melhor de 3×200 pinturas:

| sonda | comando | resultado |
|---|---|---|
| `measure_row_cost` (params) | `cargo test -p ph2d-panel-motion-params --release -- --ignored --nocapture measure_row_cost` | **13,5 µs por row** (marginal: 13,46 · 13,32 · 12,89 · 13,72 · 14,18 de 4 a 33 rows) |
| `measure_card_cost` (grafo) | `cargo test -p ph2d-panel-motion-graph --release -- --ignored --nocapture measure_card_cost` | **11,3 µs por cartão** nu (10,63 · 10,63 · 11,23 · 11,18 · 11,33 · 11,88 de 5 a 120) |

⭐⭐ **A leitura que decide o desenho: uma ROW custa MAIS que um cartão inteiro** (13,5 contra
11,3 µs) — as duas são dominadas pelo TEXTO. Um cartão com 5 rows custa **7×** um cartão nu.

**O orçamento** (16,67 ms de quadro; a conta é `N × (11,3 + R × 13,5) µs`):

| cenário | custo | % do quadro |
|---|---:|---:|
| 120 cartões **nus** (hoje) | 1,41 ms | 8 % |
| 20 cartões × 5 rows | 1,58 ms | 9 % |
| 40 cartões × 5 rows | 3,15 ms | 19 % |
| 20 cartões × 13 rows | 3,74 ms | 22 % |
| 40 cartões × 13 rows | 7,49 ms | **45 %** ⛔ |
| 120 cartões × 5 rows | 9,46 ms | **57 %** ⛔ |

⇒ **O LOD por zoom não é polimento, é o orçamento** — e ele fecha sozinho: as rows só são
legíveis a partir de `zoom ≈ 0,85` (fonte de rótulo ~11 px × zoom ≥ 9 px), e a esse zoom o
recorte do viewport deixa **~18 cartões** na tela ⇒ `18 × (11,3 + 8×13,5) = 2,1 ms` = **13 %**.
*Aproximar mostra params e esconde cartões; afastar faz o contrário — a mesma alavanca paga as
duas coisas.*

⚠️ **E a consequência para o modelo:** as rows do cartão **não podem ser `ParamRow`** (que
carrega `String` por rótulo e por valor): 20 cartões × 5 rows seriam 200 `String` por quadro.
O cartão leva `CardParam` — `&'static str` do registry + o `f32` vivo, **zero alocação** — e o
PINTOR formata o texto só dos cartões que de facto desenha.
