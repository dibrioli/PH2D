---
name: a-ported-law-carries-the-source-apps-premise-about-its-own-layout
description: «O LineEdit do Godot assenta um degrau abaixo do painel» estava certa sobre o Godot e errada sobre nós — ele põe o campo no painel, nós pomo-lo num cartão, e a lei portada mediu 0/255.
metadata:
  type: feedback
---

Portar uma lei de outro app traz junto **a premissa dele sobre a própria disposição** — e essa
premissa não viaja escrita: ela viaja no número. A frase fica verdadeira sobre o alvo e falsa sobre
nós, e um doc-comment correcto ao lado dela faz o defeito parecer auditado.

**Caso medido (`line/UIUX`, 2026-09-14).** O fundo de um campo derivava-se com
`dark_1.lerp(BLACK, max(contrast,0) * 0.5)` e o comentário dizia, com razão, *«o `LineEdit` do Godot
assenta num degrau abaixo do painel»*. Só que o Godot põe um campo **sobre o painel**, e as linhas
do nosso Inspector assentam num **CARTÃO** — que uma wave anterior deliberadamente pôs um degrau
ACIMA do painel, para o cartão se ler. O campo ficava a `4/255` do painel e a `8` do cartão; e os
dois pintores que nem a essa tabela perguntavam enchiam com o token **do próprio cartão**: `0/255`,
sem moldura de repouso. Report do dono: *«caixas de input numérico sem cor de fundo»*.

A cura não foi outro número: foi trocar a âncora de *«abaixo do painel»* para *«abaixo da superfície
mais funda em que um campo pode assentar»*, com as superfícies **nomeadas**. ⚠️ E qual delas é a
mais funda **muda com o tema** (no claro a escada sobe) — escrever a âncora na polaridade de um tema
é o mesmo defeito um nível abaixo.

**Why:** uma lei portada é uma resposta à disposição do ALVO. Quando a nossa disposição diverge —
e ela diverge sempre que alguém aqui move uma superfície, o que é trabalho legítimo — a lei
continua a compilar, continua a ter o `paper` certo citado ao lado, e passa a medir outra coisa.
É a mesma família do §0.0 do `CLAUDE.md`: *quem move o número que tornava algo inalcançável tem de
reconferir a nota* — aqui, quem move uma SUPERFÍCIE tem de reconferir todas as leis ancoradas nela.

**How to apply:** ao portar, escreva a âncora em termos **do nosso** modelo (*«a superfície em que
isto assenta»*), não do deles (*«o painel»*), e ponha as superfícies numa lista nomeada que um gate
varre. Se a âncora do alvo não tem tradução aqui, isso é a descoberta — não um detalhe de
implementação. E um valor **derivado** (não um token) é invisível aos censos da tabela de tokens:
ele precisa do gate dele.

Vizinhos: [[feedback-a-door-the-neighbour-does-not-call-is-not-a-door-yet]] ·
[[feedback-a-blocker-written-over-an-argument-the-result-ignores-is-a-fake-price]] ·
[[reference-topic-oracle-discipline]] · [[reference-topic-control-design-hazards]]
