---
name: feedback-a-probe-that-arms-a-module-by-env-var-measures-another-program-than-the-pill
description: Quatro jornadas de sondas do undo 3D passaram VERDES sobre um produto partido porque armavam o módulo por `PH2D_FIELD_SMOKE` e o dono arma-o pelo pill — a ordem "cena nasce vs. primeira captura" invertia-se, e o cache incremental nunca vigiava as colunas do módulo.
metadata:
  type: feedback
---

Enio, 2026-09-04, quinto report: *«o undo é um bosta, não melhorou nada — auditoria completa»*.
Todas as sondas anteriores (`PH2D_FIELD_UNDO_PROBE` v1–v5) armavam o modelador por
`PH2D_FIELD_SMOKE=1`. A cena de demo é a mesma nos dois caminhos; a **ordem** não é: com a variável
ela nasce **antes** da primeira captura de undo, pelo pill nasce **depois**. O cache incremental
resolvia a lista de colunas vigiadas **uma vez**, na primeira captura, a partir de
`world.component_id::<T>()` — `None` para um tipo que o mundo ainda não usou. Pelo pill a primeira
captura vê a cena vazia ⇒ `FieldPose` nunca entrava na lista ⇒ mover com o gizmo, arrastar um
slider, digitar um número (escritas **no lugar**) eram invisíveis; só spawn/despawn/troca de
archetype viravam passo. Era, letra por letra, *«não obedece cada etapa, principalmente se
transformação»* e *«um Ctrl+Z apaga tudo»*.

⛔⛔ **SEGUNDA INSTÂNCIA, 2026-09-08 — a sonda escolhe a ROTA em vez do arme, e é o mesmo
defeito.** Report do dono, três vezes (*«sumiu com o gizmo do Bezier Warp»* → *«ainda invisível»* →
a linha da sonda no app). A sonda que escrevi para diagnosticar corria
`MotionCookPump::advance_or_scrub_scoped` — a marcha da **CPU** — e deu `resolve = Some` na cena
EXACTA do report, enquanto o app dava `None`. O app corre a rota do **device**, e na rota
`FullyGpu` a ponte **retorna antes de qualquer marcha**: as tomadas (que o gizmo lê para ter a
caixa envolvente) nunca são cozidas.

⚠️ **Duas curas foram gastas em hipóteses lidas do código antes de eu instrumentar** — mover o
desenho no quadro, e reverter essa mudança. As duas erradas, e a segunda provada errada só quando
o defeito sobreviveu à reversão.

⚠️ **A resposta veio do app do dono**, por um `PH2D_WARP_DIAG=1` no sítio onde a tinta sai — e a
linha que a deu foi o ramo do **`None`**, que só existe porque eu já tinha errado duas vezes e
precisei de distinguir *«não há retrato»* de *«a sonda não corre»*. ⇒ **uma sonda de duas
respostas tem de imprimir as duas**, senão o silêncio dela é ambíguo e não conclui nada.

⭐ E o defeito de fundo é da forma que este repo já conhece: uma lei implementada **num braço** do
`match` e não no outro. A rota HÍBRIDA cozinhava as tomadas (uma cura anterior, com o doc a dizer
que elas *«cavalgam a marcha que houver»*) — e na `FullyGpu` **não há marcha nenhuma**.

**Why:** uma sonda que arma o sistema de outra maneira que o utilizador **mede outro programa** —
e quanto mais verde ela passa, mais convence. O que muda não é a cena, é a **ordem** de nascimento
relativa a um cache que se prime uma vez.

**How to apply:** a sonda entra pela **mesma porta** que o dono (aqui: o pill, `ask_open_panel`,
sem a variável; na 2.ª instância: a **rota de cozimento** que o app escolhe, não a que o teste
prefere — e se a rota exige um device que o teste não tem, o instrumento tem de viver **no app**,
atrás de uma env, e imprimir os DOIS lados da pergunta). E todo cache que se "prime" sobre *o que existe agora* tem de responder à pergunta
*«e o que nascer depois?»* — aqui a cura é re-resolver quando `world.components().len()` cresceu.
Gate headless que reproduz o pill: `a_component_type_born_after_the_first_capture_is_still_watched`.
Ver [[feedback_a_correct_undo_queue_without_the_selection_reads_as_a_broken_queue]] e
[[feedback_where_new_objects_are_born_is_the_fixture_your_gates_are_missing]].

---

## ⭐ A CURA DA 2.ª INSTÂNCIA — partir a fixtura no sítio onde ela armava o sistema (2026-09-08)

O portão que faltava a esta família não pôde nascer enquanto a fixtura fosse a da sonda: ela
**montava a cena e marchava a bomba na CPU na mesma função**, e é a marcha que cozinha as tomadas
de passagem. ⇒ *uma fixtura que pré-aquece, por outro caminho, exactamente o estado cuja ausência
**é** o defeito, não consegue produzir o defeito* — e passa a verde sobre o produto partido.

A cura é estrutural e barata: **partir a fixtura na costura**. Hoje o módulo de fixturas monta e
arma **sem marchar**, e **quem chama escolhe a rota** — a sonda marcha na CPU, o portão entrega o
quadro a um adapter de verdade (`the_fully_gpu_route_cooks_the_taps_it_never_marched`, com prova
de mutação: apagar a cura devolve `tomada armada e NAO cozida — []`, que é o report do dono à
letra).

⚠️ **E o gate lê a rota do PRODUTO** (`MotionState::route_said`), nunca a replaneia: um segundo
cálculo seria uma segunda opinião sobre que rota aquele documento toma, e o gate deixaria de medir
a do app — a mesma cerca que fez a sonda mentir, virada do avesso.

⚠️ **Sem adapter ele ESTOURA em vez de saltar** (§5.0: *skip gracioso não é verde*): um verde sem
device afirmaria sobre um programa que não é este.
