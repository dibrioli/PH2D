---
name: reference_topic_architecture_north_perf_lessons
description: Arquitectura, norte e performance — as lições dobradas do índice (leis de desenho pagas com medição), verbatim, uma linha por memória
metadata:
  type: reference
---

# Arquitetura / norte / perf — lições dobradas do índice (2026-09-10)

> Cada linha é uma memória com o gancho ORIGINAL do índice; abra o ficheiro para o mecanismo.
> Dobradas aqui porque o `MEMORY.md` passou o teto suportado (32 KB > 24 KB): ficam a dois saltos.

- ⛔⛔⛔ [Numa CPU uma poda é grátis; numa GPU ela cobra DIVERGÊNCIA, e o imposto tem o TAMANHO da poda — `11,8×` contra `12,1×`, e a wave cancelou-se](feedback_on_a_gpu_a_prune_charges_divergence.md)
- [Conteúdo de asset é PARTILHADO; per-objeto é QUAL asset](feedback_the_content_of_an_asset_is_shared_only_which_asset_is_per_object.md)
- ⭐ [Porta que o VIZINHO não chama ainda não é porta — a mesma pergunta errada 4×, a última a 15 linhas dela; só o CENSO a fecha](feedback_a_door_the_neighbour_does_not_call_is_not_a_door_yet.md)
- [«O mais recente possível» ≠ «o mais recente»: conte os TETOS](feedback_the_newest_possible_is_not_the_newest_count_the_ceilings_first.md) · [duas cópias podem ser o MECANISMO, não resíduo](feedback_two_copies_of_a_dependency_can_be_the_mechanism_not_the_residue.md)
- [Dois motores, um estado](feedback_two_engines_one_state_is_worse_than_a_slow_engine.md) · [contrato congelado escolhe a arquitetura](feedback_frozen_contract_can_pick_the_architecture.md)
- [Tipo em N sítios → componente opcional](feedback_widely_constructed_type_favors_optional_component_over_appended_field.md) · [a representação apaga o caso especial](feedback_the_representation_can_delete_the_special_case.md)
- ⭐ [Antes de acrescentar um CAMPO, veja se a AUSÊNCIA já é o estado — e se ela já tem leitores](feedback_before_adding_a_field_ask_whether_the_absence_is_already_the_state.md)
- ⭐ [Cópia profunda que leva TODO componente leva o ELO: duas entidades com a mesma identidade = sósia que não se move](feedback_a_deep_copy_that_copies_every_component_also_copies_the_identity_link.md)
- [Invariante na DERIVAÇÃO](feedback_enforce_the_invariant_at_the_derivation_not_at_each_gesture.md) · [marca de evento é canal próprio](feedback_a_transient_event_marker_is_its_own_channel.md)
- [A recusa que responde é a do knob VIZINHO — grepe a MÉTRICA](feedback_the_measured_refusal_you_need_is_in_the_neighbouring_knob.md)
- [Rejeição cuja explicação descreve outra obra = PRÉ-REQUISITO](feedback_a_rejection_whose_explanation_describes_another_work_is_a_prerequisite.md)
- [Sonda depois do passo que ARRUMA mede a arrumação](feedback_a_ruler_placed_after_the_tidying_step_measures_the_tidying.md)
- [Sonda no ramo do FRACASSO de A não vê os acertos de A](feedback_a_probe_in_the_failure_branch_cannot_see_the_other_sides_successes.md)
- ⭐ [Defeito ESCONDIDO atrás de outro: o gate faz `continue` sobre entrada inválida e fica cego — curar o 1.º descega o instrumento, e o que aparece NÃO é regressão](feedback_a_defect_can_hide_behind_another_defect_and_blind_the_very_gate_that_would_find_it.md)
- [Correlação sem contra-exemplos pode descrever DISPONIBILIDADE, não correcção](feedback_a_correlation_with_zero_counterexamples_may_describe_another_question.md)

## Descidas do índice em 2026-09-17 — a rodada de seis linhas pôs o `MEMORY.md` a **224 linhas / 36,5 KB** contra o tecto dele (140 / 17 KB), e o carregador cortou **66 linhas em silêncio**. Estas entradas descem VERBATIM; o ponteiro para esta família continua no índice.

- ⛔⛔ [Uma vista NOVA entra ao LADO da que os consumidores já lêem, nunca no lugar dela — 24 leitores tratavam `contours()` como a figura e estavam certos; 2 gates velhos apanharam-no](feedback_a_new_view_cannot_replace_the_one_consumers_read.md)
- ⛔ [Sonda que arma o módulo por env var mede OUTRO programa que o pill (5 reports) — e a do arco cronometrava a PLACA com o dono no modo MODEL, que traça na CPU (12 196 facetas contra 0)](feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill.md)

- ⛔⛔⛔ [Comutar duas leis por um LIMIAR não dá um salto: dá CHATTER — ajuste a DIFERENÇA, que é zero onde nada há a corrigir](feedback_a_boolean_over_a_continuous_quantity_is_a_step.md)

- ⭐⭐⭐ **Quando um MECANISMO está completo e o produto não o usa, o que falta é uma POLÍTICA — e o sítio onde ela mora mede-se em gates partidos** (2026-09-21, o Inspector): a dobra de secção tinha tudo (chevron, clique, animação, recorte) e **nenhuma secção nascia dobrada**, logo o painel desenhava `2,5`–`6,5` ecrãs contra uma dobra de `880 px`. ⚠️ Posta no `Panel::populate` do painel, a política reprovou **342** gates da crate dele; movida para a porta de arranque do EDITOR, **4**. ⇒ *o painel declara o que PODE mostrar; quem compõe o editor declara como ele ABRE* — e o painel nem conhece a altura da janela. ⛔ E o valor de fábrica não foi escolhido: o orçamento da dobra dá para UMA secção, e medidas uma a uma só a `Transform` cabe (`849 px`; a `Render` no lugar dela dá `1 000`).
## ⭐⭐⭐ Um caminho DERIVADO pode trocar a contagem de pontos; o caminho do ARTISTA não (2026-09-20)

Report do dono: *«como é que o `Effects: Arc` com tão poucos pontos fica tão perfeito?»*

**Porque os pontos que ele vê não são os que saem.** O envelope não deforma os pontos de controlo —
ele amostra a curva verdadeira `W(C(t))` com a jacobiana em forma fechada e **re-ajusta** com o
`fit_to_bezpath`, que subdivide até bater uma tolerância (`0,1 %` da diagonal da bbox). Medido pela
porta do produto: uma estrela de `10` nós **sai com `26`**; um rectângulo de `4` sai com `12`.

⭐⭐ **E a elipse é a linha que separa as duas coisas que o refit compra:** `4 → 4`, sem um ponto
novo, e ainda assim **`126×`** mais fiel que mover os pontos de controlo. Ou seja: (a) as alças no
óptimo e (b) os pontos que faltarem são ganhos **independentes**, e só o (b) precisa de acrescentar.

⭐⭐⭐ **E o preço da alternativa está medido:** para igualar o Arc, a lei ingénua precisa de **`80`**
nós onde ele usa `26` (`10 → 20 → 40 → 80` por subdivisão uniforme; só a 3.ª ronda desce abaixo).

⛔⛔ **A pele do esqueleto NÃO pode fazer o mesmo, e a razão não é matemática — é de PROPRIEDADE.**
Medido: a contagem de pontos do Arc **muda 5 vezes** ao longo do curso do slider (`18 → 22 → 26` em
40 passos). Ninguém repara porque aquele caminho é **derivado**: o artista edita a GAIOLA, e a fonte
do filho foi assada e congelada na criação. A forma presa a ossos é a forma que ele tem **na mão** —
selecciona, arrasta, acrescenta um ponto — e re-decidir a contagem dela a cada quadro foi à letra o
report de 19/09 (*«o path muda repentinamente, como se o handle mudasse de tipo»*).

⚠️ **E a atribuição daquele report é mais fina do que ficou escrita:** o que dava *chatter* era o
**LIMIAR** (*«o desvio passou da tolerância?»*), um booleano sobre uma grandeza contínua. O envelope
re-ajusta **sempre**, sem decisão nenhuma — e por isso a contagem dele muda em silêncio em vez de
piscar. ⇒ *um refit incondicional sobre um caminho derivado é seguro; um refit condicional sobre o
caminho do artista é o pior dos dois mundos.*

⇒ **antes de copiar uma técnica de um subsistema para o outro, pergunte de QUEM é o caminho de
saída.** É essa a variável, não o algoritmo.

## ⭐⭐⭐ Acrescentar pontos ao PRENDER ganha de acrescentá-los por QUADRO (2026-09-20)

Ordem do dono: *«vamos tentar dar à forma presa dois corpos SE o custo em performance não for muito
alto»* — o segundo corpo sendo um re-ajuste adaptativo (a técnica do `Effects: Arc`, que emite
tantos nós quantos a tolerância pedir). Construído, medido, **recusado**:

| lei | nós | µs/forma | desvio ao padrão-ouro (p90) |
|---|---:|---:|---:|
| a lei de HOJE (o bind subdivide, o ajuste corrige) | `54` | **`105`** | **`0,00094`** |
| o refit adaptativo (`C¹`, tol `0,03 %`) | `61` | `4 889` | `0,00221` |
| *(sobre a fonte SEM o bind)* a lei de hoje | `8` | `53` | `0,22281` |
| *(sobre a fonte SEM o bind)* o refit | `40` | `4 401` | `0,00256` |

⭐ **As duas últimas linhas são a razão inteira:** sobre `8` nós o refit é **`87×`** mais fiel — o
mecanismo é real. ⇒ **mas o passo de PRENDER já acrescenta os pontos**, e com eles a lei de hoje é
`2,7×` mais fiel que o refit por **`1/46`** do preço.

⇒ ***antes de copiar um mecanismo de outro subsistema, procure o passo ÚNICO que já o faz.*** Aqui
o `Bind` é esse passo; o `Arc` re-ajusta por quadro porque não tem nenhum ([[a-derived-path-may-change-its-point-count]]).

## ⭐⭐⭐ Procurar o preço de uma feature RECUSADA achou o tecto do módulo (2026-09-20)

O refit custava `16`–`27 ms` por forma. Antes de escrever «caro» (§0.0: *nunca deixe o caminho
lento definir o tecto*), perguntei **de que é o custo**. Medido:

| | µs por chamada |
|---|---:|
| `campo.linha()` — **varre os 878 triângulos da malha** | **`0,8048`** |
| a LEI que ela alimenta (`weights_corrected` + `blend`) | `0,0344` |

⇒ **a busca era `96 %` do custo de amostrar um ponto, e `20×` a lei que ela serve** — e paga por
amostra, por forma, por quadro, **também pela lei que ship**. Não havia índice espacial nenhum;
os dois leitores do campo (o baricêntrico e o `C¹`) eram a mesma varredura.

Uma grelha de baldes (lado = `√(triângulos)`, inserção pela **CAIXA** de cada triângulo e nunca
pelo centro — *um índice que pode falhar precisa de uma varredura de reserva, e aí não há índice*):
consulta `0,8048 → 0,0428 µs` (`18,8×`), e **o recook da forma presa de `336` para `105 µs`** — na
lei que já shipava, sem mudar um bit do desenho.

⇒ ***o preço de uma rota nova mede-se antes de a construir, e a conta pode não ser dela.*** A
feature foi recusada e a medição dela pagou um `3,2×` no que já corria.

## ⛔⛔ Uma régua de correspondência MATERIAL não atravessa uma mudança de contagem de nós (2026-09-20)

A serpentina e o excesso de curvatura desta casa passam pela `b_no_passo`, que reamostra a um passo
de arco medido no REPOUSO e interpola no **espaço de ÍNDICE partilhado** — é isso que torna a
correspondência material em vez de geométrica. ⇒ **um caminho com outra contagem de nós não tem
esse espaço de índice**, e as duas réguas leram `0,000000` sobre ele: *verde a medir nada*.

A régua que atravessa é a geométrica (ponto a POLILINHA contra o padrão-ouro). ⚠️ E ela tem o seu
próprio alçapão: medida ponto a PONTO ela lê o **espaçamento da própria amostragem** como erro —
`1,28`–`3,19` numa peça de `400` cuja tolerância era `0,46`, o que se lê como *«o motor não cumpre
a tolerância dele»* e é falso.

## ⭐⭐⭐ ASSAR e depois AJUSTAR ganha de AJUSTAR a chamar a lei — `7,7×` e mais fiel (2026-09-20)

Ideia do dono: *«e se fizer um bake para imagem e usar a imagem como referência para usar a técnica
de arc»*. Ela inverte a ordem de um refit adaptativo, e a inversão é a coisa toda.

| | nós | µs/forma | desvio ao padrão-ouro (p90 / máx) |
|---|---:|---:|---:|
| a lei que ship (um corpo só) | `54` | **`106`** | `0,00094` / `0,00341` |
| refit adaptativo — **chama a lei de dentro do fitter** | `61` | `3 993` | `0,00221` / `0,00466` |
| **bake — amostra FIXO, depois ajusta** | `110` | **`658`** | **`0,00070` / `0,00194`** |
| o CHÃO do modelo aos `54` nós | `54` | — | `0,00095` / `0,00334` |

**O mecanismo:** um fitter adaptativo amostra onde QUER e muitas vezes (`8 371` consultas), e cada
consulta paga a lei cara. Assando primeiro, a lei corre um número **fixo** de vezes com a leitura
barata, e o fitter trabalha sobre pontos já calculados.

⭐⭐ **E o bake dá de graça a continuidade que o fitter exige:** a tangente sai de uma
**Catmull-Rom sobre as amostras**, não do gradiente do campo — o que dissolveu a pré-condição `C¹`
que eu tinha acabado de nomear ([[forcing-a-property-after-the-optimum-is-what-ripples]]).

⛔⛔ **E o achado que não se adivinha: amostrar MAIS piora.** A `128` amostras por segmento o pior
desvio salta `4,8×` (`0,00456 → 0,02191`), porque a amostragem passa a **resolver** os bicos que o
elemento finito linear deixa em cada uma das `119` arestas da malha, em vez de os alisar. *O
alisamento de um bake não é efeito colateral — é o que o protege, e existe uma amostragem ÓPTIMA.*

⛔ **Ele não substitui um passo de PREPARAÇÃO que põe nós onde a ESTRUTURA precisa:** sobre a fonte
crua de `8` nós, mesmo a `64` amostras ele fica `2,9×` mais caro e `1,3`–`1,8×` pior que
`bind + lei de hoje` — a subdivisão do bind é graduada pelas juntas, e isso é informação que o
fitter não tem.
