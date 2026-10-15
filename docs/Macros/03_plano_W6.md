# Macros — plano da W6: o tutorial «FAZER» (2026-10-15)

> Pedido: [plano da onda 1 §2, §4, §6](01_plano_onda1.md) — «**Fazer** = o passo mostra o balão + o realce e espera
> `StepGate` (ferramenta activa · nó do tipo X ligado a Y · área pintada ≥ p% numa região · parâmetro numa faixa)»;
> cena `PH2D_MACRO_SMOKE=3`: «balão + realce, espera o aluno ligar o nó; desfazer volta o passo». ABERTO do
> [handoff W5](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W5.md) §10. Oráculos de desenho (o PRODUTO, não o
> fonte): Unity IET, Godot Tours, Adobe Discover — o modo «fazer» valida ESTADO, não gesto (`01_plano_onda1.md` §1).

## 1. O que existia (conferido no código, 15/10)

Nada de tutorial: nenhum portão, nenhum balão, nenhum realce. `ModoDaMacro { Fiel, Explicativo }`
(`ph2d-ecs/src/macro_doc.rs:14`) diz como a trilha do Motion TOCA; a macro toca SEMPRE num palco próprio
(`ph2d-app-motion/macro_track/palcos.rs`, `MotionState` do palco), nunca no documento. O Motion tem UM `MotionDoc`
global; cada nó tem dono (`owners`, `ph2d-motion-doc/src/owner.rs:100-136`); entrar em Edit só põe
`objeto.editing`. As portas do grafo já se registam no `hit_index` por quadro (`ph2d-panel-motion-graph/src/hits.rs:397`,
ids `motion_graph/sock_{in,out}/{nó}/{porta}`, privados) — `rect_for(id)` (`ph2d-editor-core/src/interaction/hit.rs:238`)
devolve-os. Os nomes das etapas já vêm de re-tocar a gravação num palco e comparar os tipos antes/depois
(`macro_track/etapas.rs:259-330`).

## 2. A lei: o progresso é uma função PURA do estado

- **O aluno faz cada etapa.** O tutorial NUNCA escreve no documento por si. ⛔ Recusado: «as etapas sem portão o
  tutorial aplica-as» — com o progresso derivado do estado, o Ctrl+Z do aluno desfaz a etapa aplicada, o tutorial
  volta a aplicá-la no quadro seguinte: um laço com o desfazer. Com um cursor guardado em vez do estado, o desfazer
  deixa cursor e documento a discordar (duas verdades).
- **O critério de cada etapa deriva-se da gravação** (o autor não escreve portões à mão): Δk = o grafo gravado
  ANTES × DEPOIS da etapa k, re-tocado no palco (a mesma reprodução que já dá os nomes): nós acrescentados (tipo),
  ligações feitas/desfeitas, params mudados (valor), nós apagados. Etapa com Δ vazio (mover, renomear, fundos) não
  tem critério: não espera, não mostra balão.
- **Casar nós gravados com os do aluno — por TIPO e ORDEM de criação** entre os nós do dono: o i-ésimo nó gravado
  do tipo T é o i-ésimo nó do aluno do tipo T (ids crescem com a criação). Os ids crus nunca se comparam (o
  documento do aluno tem outros nós e outro `next_id`).
- **Um item de Δk cumpre-se** se o estado do aluno o tem, OU tem o de uma etapa POSTERIOR sobre a mesma chave (o
  mesmo param, a mesma ligação): o aluno que salta para o valor final não fica preso no intermédio.
- **Progresso = a 1.ª etapa com critério por cumprir.** O Ctrl+Z volta o passo sem gancho nenhum (o estado deixa
  de o cumprir); o refazer avança.
- **Param «cumprido»** = o valor que o aluno VÊ no campo é o alvo: a mesma arredondação do campo do cartão.

## 3. O produto

- **Modo `Fazer`** no fim do `ModoDaMacro` (o seletor do cartão Macro; «acrescentar é no fim»). Em `Fazer` o palco
  não toca: o objecto mostra o que o aluno fez.
- **Balão** (chrome do herói, por cima de tudo, molde `chrome/fill_modal.rs`): «Passo n de N», a frase da etapa
  (derivada, i18n: «Liga {a} a {b}» …) e os botões **«Mostra-me»** (faz ESTA etapa pelo aluno, pelas portas do editor:
  um passo de undo, só por clique — o aluno vê o fio nascer) e **«Recomeçar»** (o grafo do dono volta ao estado
  inicial da trilha: um passo de undo). No fim: «Feito!».
- **Realce**: um anel à volta do alvo da etapa (a porta de saída e a de entrada de uma ligação; o cartão do nó de um
  param) por `rect_for` dos ids que o painel do grafo já regista (tornados públicos). Alvo sem rect neste quadro
  (painel fechado, nó fora de vista) ⇒ o balão fica, sem anel, e a frase diz onde.
- **Fora do editor-core:** a família publica a vista do balão por `macro_edits` (thread-local, como o painel MACROS);
  os cliques voltam por `EditorAction::Macro(MacroRequest::MostraMe | Recomeca)` (ADR-0075).

## 4. Ondas

| Onda | O quê | Gates (vermelho primeiro + mutação) |
|---|---|---|
| W6a | Motion: critérios por Δ, casar por tipo, progresso, «Mostra-me», «Recomeçar», modo `Fazer`, balão + anel, cena `=3` | critérios da cena (cada etapa cumpre-se só quando feita; desfazer volta; saltar para o valor final cumpre os dois; nós a mais de outro dono não contam); «Mostra-me» cumpre a etapa e o Ctrl+Z a desfaz; `Fazer` ⇒ sem palco; balão: cliques chegam como pedido, anel no rect da porta (seam, `ph2d-ui-testkit`) |
| W6b | Painter: a pincelada cumpre-se pela ÁREA pintada na região dela; a ferramenta activa como pré-condição; «Mostra-me» = a pincelada pelas portas | a medir ANTES de fixar a fracção (pinceladas de aluno × gravada) |
| W6c | Os outros meios do Painter (Digital, Watercolor, Impasto, Composite) na gravação | estado inicial por meio; a cerca do Wet Painter: os PNG da sonda desses meios ficam iguais ao byte |

**Cena `=3` — «Wind, your turn»:** o mesmo vento da `=2`, o grafo do aluno já com as formas, o integrador, o arrasto,
a caixa e o vento SOLTOS; a macro grava as três ligações que faltam e o «width» final. A cena abre em Edit, com o
painel do grafo à vista, e o balão aponta a 1.ª porta. Cada ligação muda o que se vê; o vento sopra na última.
