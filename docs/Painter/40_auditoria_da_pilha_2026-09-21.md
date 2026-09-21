# AUDITORIA — o preço e o carimbo da pilha do Composite Brush (2026-09-21)

> **Report do dono, com duas fotos:** *«1) A performance ficou ruim, muito aquém do esperado.
> Pinceladas rápidas quase travam a tool. 2) Temos problemas com o carimbo que fica retangular.
> Provavelmente o principal culpado é Smear. Estude as otimizações e correções do sistema per-layer
> color — talvez descubra lá as soluções.»*
>
> **Ele tinha razão nas duas, e o ponteiro dele para o per-layer color é a cura da primeira.**
> A segunda tem um culpado que não é o que nenhum dos dois supunha.

Sondas: [`diag_auditoria_da_pilha.rs`](../../crates/ph2d-tool-painter/src/tool/paint/diag_auditoria_da_pilha.rs)
(`--release`, canvas `1024²`, `load 3,5`–`5,0`). Prior-art:
[`HANDOFF_per_layer_color_perf_artifacts.md`](handoffs/HANDOFF_per_layer_color_perf_artifacts.md).

---

## §1 — A LEI QUE AS DUAS METADES PAGARAM: **as fixturas estavam no ponto neutro**

⛔⛔ Antes dos dois mecanismos, o que os escondeu, porque é o mesmo dos dois lados:

| régua que existia | o ponto neutro dela | o que ela não pode ver |
|---|---|---|
| [`diag_preco_da_pilha`](../../crates/ph2d-tool-painter/src/tool/paint/diag_preco_da_pilha.rs) | traço **RECTO** | a janela do replay a crescer |
| `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` | canvas **OPACO** + Smear no **TOPO** | o Smear (no fundo ele corre sobre o `pre`) |

⭐⭐⭐ **O CONTROLO POSITIVO que devia existir e não existia:** calar cada camada uma de cada vez e
perguntar se a imagem muda. Sobre a fixtura antiga o **Smear movia `0` pixels** e `warp.active` lia
`false` — *todo A/B desta linha comparava duas corridas de uma camada que não corre*. A causa é de
produto e é simples: na pilha do dono o **Smear é a camada de BAIXO**, logo corre **primeiro**,
sobre o `pre`; numa tela vazia não há nada para esfregar. A fixtura honesta pinta arte ANTES.

| camada calada | tela vazia (a fixtura velha) | tela **com arte** |
|---|---|---|
| Blur | `864` px, pior `2` | `4 456` px, pior `73` |
| Brush vermelho | `36 777`, pior `55` | `36 161`, pior `69` |
| Brush preto | `76 228`, pior `74` | `75 859`, pior `96` |
| **Smear** | **`0`, pior `0`** | **`5 821`, pior `212`** |

---

## §2 — A PERFORMANCE: a premissa do módulo é falsa para um RABISCO

O cabeçalho do [`composite_pilha.rs`](../../crates/ph2d-tool-painter/src/tool/paint/composite_pilha.rs)
declara: *«replayando só os lotes cuja caixa a toca — um número que **não cresce com o traço**, ele
vale `~2/spacing`»*. Medido, isso é **verdade para um traço recto e falso para um rabisco** — e um
rabisco é o que pintar É: um traço que volta à própria vizinhança faz **toda caixa tocar toda
caixa**.

| caminho | passos | ms | **ms/evento** | lotes replayados por lote |
|---|---|---|---|---|
| recto | 60 | 45,6 | 0,76 | 11,9 |
| recto | 480 | 430,6 | **0,90** | 17,6 |
| rabisco | 60 | 179,4 | 2,99 | 16,7 |
| rabisco | 120 | 536,1 | 4,47 | 28,2 |
| rabisco | 240 | 1 533,8 | 6,39 | 40,4 |
| rabisco | 480 | **5 813,6** | **12,11** | **79,4** |

⇒ recto **plano**; rabisco **linear no comprimento** ⇒ o total é **quadrático** (`×3,8` ao dobrar).

### §2.1 — A fase dominante é o REPLAY, não a contabilidade

| caminho | total | guardar+`pre` | Brush | Smear | Blur | restaurar |
|---|---|---|---|---|---|---|
| recto | 276 ms | 0,5 % | 49,6 % | 4,1 % | 44,8 % | 0,3 % |
| rabisco | 2 800 ms | 0,2 % | 49,1 % | 1,9 % | 48,4 % | 0,2 % |
| rápido | 570 ms | 0,2 % | 23,7 % | 5,9 % | **69,8 %** | 0,1 % |

⭐ O save/restore que o doc do módulo defende com tanto cuidado custa **< 1 %**. O preço é
**re-carimbar a história**, e o Blur é a metade mais cara porque ele é re-aplicado uma vez por lote
da janela.

### §2.2 — O TECTO DA CURA, medido por ablação do replay

| caminho | passos | com replay | sem replay | ganho |
|---|---|---|---|---|
| recto | 241 | 276,6 | 31,8 | `8,7×` |
| rabisco | 241 | 2 736,4 | 112,4 | `24,4×` |
| rabisco | 481 | 10 666,3 | 217,9 | **`49,0×`** |
| rápido | 25 | 533,1 | 74,9 | `7,1×` |

⭐⭐⭐ **E sem o replay o custo volta a ser LINEAR** (`112,4 → 217,9` ao dobrar o traço, contra
`2 736 → 10 666`). O ganho **cresce com o traço**, porque o que se remove é o termo quadrático.

### §2.3 — A cura é a que o dono apontou, e a lei é a MESMA

O [`stamp_color_cache::PerLayerStroke`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_color_cache.rs)
resolve **exactamente a mesma lei** — *«o topo pinta acima de TODA a cobertura acumulada da de baixo
ao longo do traço INTEIRO»* — com **mapas de cobertura acumulados** (`{pre, cov}`, 1 B/px por
camada), recompondo `pre ⊕ L₀ ⊕ … ⊕ L_N` a cada lote. **Sem replay.** O custo por evento é
`O(dabs NOVOS) + O(região × N)` e não depende da história.

⭐ **A pilha já tem metade da maquinaria:** `composite_mask[pos]` já é um plano de 1 B/px por camada
(o cap do Accumulate). O que muda por operação:

* **Brush · Erase** — acumulam cobertura; o depósito corre **uma vez** sobre a região.
* **Blur** — em vez de `N` aplicações (uma por lote da janela), **UMA** sobre a região, com a
  cobertura acumulada como peso. É a metade que vale `45`–`70 %` do relógio.
* **Smear** — **já** é um campo por traço resolvido de uma vez; não muda.

⚠️ **Dois ganhos do prior-art que esta pista não tem**, os dois medidos lá e ausentes aqui: o
**kernel fundido** (`3,2`–`4,5×`) e o **paralelismo por bandas** com rayon (`95,4 → 7,9 ms`).
⛔ Eles são factores constantes sobre um custo quadrático — *primeiro mata-se o quadrático*.

---

## §3 — O CARIMBO RECTANGULAR: três suspeitos ilibados e um condenado

### §3.1 — O instrumento: a recomposição GLOBAL forçada

`RECOMPOSICAO_GLOBAL` é o *forced full recomposite* que o handoff do Per-Layer Color §3-1 prescreve:
ligado, `caixa_nova` é o canvas inteiro ⇒ a janela é o traço todo, nada é recortado e a base do
Smear é refrescada em toda parte. O A/B corre **o mesmo fluxo de eventos** nas duas rotas.

### §3.2 — ILIBADOS, com número

| suspeito | régua | veredito |
|---|---|---|
| a recomposição regional (canvas opaco) | A/B contra a global | `|Δ| pior **0**` em todos os regimes |
| o rectângulo sujo | pixels mudados FORA dele, evento a evento | **0 fugas** em recto · rabisco · rápido |
| `refresca_a_base_do_smear` só na `caixa_nova` | ablação: refrescar sobre a `caixa_grande` | **`0` de diferença** — o mosaico da base **não é a causa** |

⚠️ Sobre tela vazia há uma divergência de `pior 255` em **`4 317` bytes** — e ela é **invisível**:
são bytes de **RGB em pixels de alfa 0**. Composto sobre branco, `pior 0`. ⛔ Ela fica **nomeada**
porque não é inócua para sempre: um Blur ou um Smear posterior LÊ aquele RGB.

### §3.3 — ⛔⛔⛔ CONDENADO: `limite_do_smear`, e ele refuta uma nota do repo

Com a fixtura certa (arte por baixo) a rota regional **diverge do que se VÊ**:

| caminho | pior | médio | px visíveis | caixa da diferença |
|---|---|---|---|---|
| rabisco | 48 | 0,003 | 710 | `482,605` **`77×28`** |
| rápido | 39 | 0,007 | 1 510 | `443,538` **`162×92`** |

Ablação, uma de cada vez, contra a global:

| ablação | rabisco | rápido |
|---|---|---|
| nenhuma (o que shipa) | pior 48 · 710 px | pior 39 · 1 510 px |
| só: base sobre a caixa GRANDE | pior 48 · 710 px | pior 39 · 1 510 px |
| **só: render SEM limite** | pior **167** · 11 743 px | pior **0** · **0 px** |

⛔⛔ **A nota do [`state.rs`](../../crates/ph2d-tool-painter/src/tool/paint/state.rs) sobre o
`limite_do_smear` está REFUTADA.** Ela diz, por escrito: *«Ele é um guarda de RELÓGIO e não de
imagem, e a mutação que o apaga SOBREVIVE — declarado com a medição. Com a base refrescada só na
`caixa_nova` ela fica correcta em toda parte, logo o render da união inteira dá a MESMA imagem.»*
Medido na fixtura que contém o fenómeno, **ele É um guarda de imagem**: tirá-lo torna o traço rápido
**EXACTO** (`39 → 0`) e o rabisco **pior** (`48 → 167`).
⚠️ *A medição que o declarou inócuo correu sobre o traço recto e o canvas opaco — onde o Smear é
inerte.* É a mesma lei do §1, agora do lado de uma mutação que «sobreviveu».

### §3.4 — O que fica ABERTO, nomeado

⏳ **No rabisco nenhuma das duas ablações fecha o buraco** (`48 → 167`, não `48 → 0`) ⇒ há um
**terceiro** componente na janela/região que não está atribuído. Ele não é adivinhado aqui: a
próxima wave mede-o antes de tocar em código.

---

## §4 — O que NÃO reproduz, e é honesto dizê-lo

O artefacto que se reproduz é um **degrau rectangular alinhado aos eixos** na zona esfregada
(`pior 48` de 255). As fotos do dono mostram algo **mais forte**. Falta à bancada, nomeado:

* a pista do **preview/GPU** (estes testes não têm device);
* o **relevo** (`heights`/`covers`/`mats`), que a janela da recomposição **não guarda nem repõe** —
  item ABERTO desde o handoff §22.9, e cuja isenção escrita (*«no meio Digital não há relevo»*) não
  tem gate;
* a textura de **Shape/Grain**, o Accumulate e o substrato.

*Uma auditoria que declarasse a causa inteira a partir do que reproduziu estaria a escolher entre
duas medições que não discriminam.*
