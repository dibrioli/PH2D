# Pesquisa — motion path (interpolação espacial)

> Insumo do [ADR-0141](../architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md).
> Molde: [`03_pesquisa_nesting.md`](03_pesquisa_nesting.md) — pesquisar ANTES de decidir, e trazer
> a **evidência negativa** junto com a positiva.

---

## §0 — A pergunta

Hoje `PropKind::TranslationX` e `TranslationY` são **duas tracks escalares independentes**. Uma
curva no espaço *emerge* delas (X linear + Y com ease já entorta a trajetória), mas ela não é
**autorável**, não é **visível** e não é **agarrável**. Fazer um arco é brigar com dois gráficos
até que o canvas pareça certo.

Arcos são um dos 12 princípios da animação. A pergunta desta pesquisa não é *"queremos motion
path?"* — é **"onde ele mora, dado que já temos dois eixos separados?"**

---

## §1 — O achado que fecha a pergunta

**Dois produtos, independentemente, dizem que os dois modos são MUTUAMENTE EXCLUSIVOS — e que a
escolha é do OBJETO, não do app.**

| Produto | O que diz |
|---|---|
| **After Effects** | *"Separate Dimensions feature **precludes** having Spatial Keyframes"* — separar X/Y **remove** as alças de bézier da trajetória; resetar a propriedade Position as devolve. O default é Position como **uma** propriedade 2D com interpolação espacial **Auto Bezier**. |
| **Toon Boom Harmony** | O peg tem um *Position mode*: **Separate** ou **3D Path**. No 3D Path *"os eixos X, Y e Z estão contidos dentro de uma ÚNICA função, com a facilidade do movimento determinada por uma ÚNICA função de velocidade"*. O Separate *"permite mais controle durante animação de cut-out e rigging"*. |

⚠️ **Isto responde a pergunta sem precisar de opinião nossa.** Uma tangente espacial é um objeto
**2D**; duas curvas escalares independentes não têm onde guardá-la — e, pior, editar a curva de X
sozinha moveria uma trajetória que o artista não autorou. Os dois produtos chegaram à mesma
conclusão por caminhos diferentes, e **nenhum dos dois tentou enxertar tangente espacial em eixos
separados**.

**Corolário para nós:** não existe "adicionar motion path às tracks que temos". Existe **um modo
novo**, ao lado do que temos, com a escolha por objeto — e o modo que temos hoje é exatamente o
que a Harmony recomenda para rigging de cut-out, ou seja **não é um erro a corrigir, é metade da
resposta que já está construída**.

---

## §2 — Como o tempo e o espaço convivem (os dois eixos ortogonais)

**A regra do AE:** *temporal* é a interpolação de valores **no tempo**; *spatial* é a interpolação
**no espaço**. Elas são **independentes**: a temporal define a taxa de mudança (o graph editor), a
espacial define a trajetória (o painel de composição, com alças de bézier na tela).

**A regra da Harmony:** num 3D Path a velocidade é **UMA função** (`Position: Velocity`) para os
três eixos — não uma por eixo. A forma do gráfico indica a velocidade, e a trajetória é outra
coisa, editada na Camera View.

⚠️ **É a mesma decisão, dita duas vezes: UMA curva de tempo para o movimento inteiro, e a
geometria à parte.** Nosso graph editor, os weighted tangents, os presets de easing e o speed
graph já são exatamente essa "uma curva de tempo" — eles não precisam de nada novo.

**Como o AE mostra os dois de uma vez:** a trajetória é uma linha pontilhada e **os PONTOS são o
tempo** — o espaçamento entre eles é a velocidade. Uma única figura carrega as duas informações:
*por onde* (a linha) e *quão rápido* (a densidade dos pontos). É a resposta de UI e ela é barata.

---

## §3 — Roving: nós já construímos a sombra dele

O AE só oferece *Rove Across Time* para propriedades **espaciais** (Position, Mask Path);
*"propriedades não-espaciais como Opacity não podem rovar"*. O gesto existe **para servir o motion
path**: as keys do meio deslizam no tempo para dar velocidade constante ao longo da trajetória,
com a primeira e a última pinadas.

⚠️ **O nosso [`rove.rs`](../../crates/ph2d-anim/src/rove.rs) diz isso de si mesmo, na linha 6:**
*"|Δvalue| ao longo do percurso — **o modelo espacial do AE aplicado a uma track escalar**"*.
Portamos o parceiro júnior e deixamos o sênior de fora. Em modo Path, roving passa a significar o
que ele significa no produto de origem, **sem uma linha de código nova no motor de roving**.

---

## §4 — Auto-orient (girar ao longo do caminho) — e o modo de falha publicado

O AE tem `Layer > Transform > Auto-Orient > Orient Along Path`. É o acompanhamento canônico do
motion path (o peixe que aponta para onde nada).

⚠️ **Modo de falha documentado na comunidade do próprio AE:** *"auto-orient flips when stopping
motion"* — quando a velocidade chega a zero a tangente é **indefinida**, e o objeto gira
bruscamente. Quem porta auto-orient tem de responder o que faz na velocidade zero (segurar o
último ângulo válido é a resposta usual) **antes** de o artista descobrir na tela.

⚠️ **E o conflito que é nosso, não deles:** auto-orient **escreve rotação**, e nós temos uma track
`Rotation`. Dois autores de um fato, com o de trás vencendo em silêncio — a falha que este módulo
catalogou meia dúzia de vezes ([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]). O
ADR tem de dizer quem vence **e** tornar isso visível.

---

## §5 — Evidência negativa (quem NÃO tem, e o que isso nos diz)

| Produto | Motion path? | Leitura |
|---|---|---|
| **Blender** | **Não como autoria.** O que ele chama *Motion Paths* é **visualização** (desenha o caminho que as F-curves já produzem, read-only). A trajetória autorada exige um objeto `Curve` + constraint `Follow Path` | O modelo de F-curve por-eixo, que é o nosso, **não estende** a motion path — o Blender resolveu por fora, com outro objeto |
| **Spine** | Não | Esqueletal: a trajetória emerge do rig, não é autorada |
| **Rive** | Não expõe bézier espacial em keys de posição | Editor=runtime, X/Y independentes |
| **Lottielab** | **Sim** — tem *Motion Path* nos controles de camada | A geração web nova considera isto obrigatório |

⚠️ **O contraste Blender × AE é o dado mais útil da seção.** O Blender é o produto cujo modelo de
dados mais se parece com o nosso (uma F-curve escalar por canal) — e ele **não** conseguiu enxertar
trajetória autorável nesse modelo: ofereceu visualização, e mandou quem quer o caminho de verdade
usar outra coisa. Isso é um aviso sobre o custo de tentar o enxerto, e um argumento a favor do modo
explícito do AE/Harmony.

---

## §6 — O que a pesquisa NÃO respondeu (e fica para a medição)

1. **Conversão entre modos.** AE e Harmony deixam trocar; nenhum dos dois publica o que acontece
   com as curvas na conversão. É medição nossa (§Fatia 0 do plano).
2. **Custo de amostragem.** Nenhum produto publica número. O nosso `apply` é linearítmico hoje
   ([`clock.rs`](../../crates/ph2d-timeline/src/clock.rs)) e o kill-criterion do ADR mede.

---

## Fontes

- [Keyframe interpolation in After Effects — Adobe Help](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html)
- [Understanding Spatial and Temporal Interpolation in After Effects — PremiumBeat](https://www.premiumbeat.com/blog/understanding-spatial-and-temporal-interpolation-in-after-effects/)
- [How to Use Spatial Interpolation for Motion Animation in After Effects — Envato Tuts+](https://photography.tutsplus.com/tutorials/how-to-use-after-effects-spatial-interpolation--cms-41298)
- [No spatial bezier keyframes available — Adobe Community](https://community.adobe.com/questions-529/no-spatial-bezier-keyframes-available-67524)
- [What is the difference between a 3D path and a Separate path? — Toon Boom Help Centre](https://helpcentre.toonboom.com/hc/en-ca/articles/44801963905299-What-is-the-difference-between-a-3D-path-and-a-Separate-path)
- [Harmony 21 Premium — Displaying Velocity Curves](https://docs.toonboom.com/help/harmony-21/premium/motion-path/display-velocity-curve.html)
- [Harmony 22 Advanced — About Functions](https://docs.toonboom.com/help/harmony-22/advanced/motion-path/about-function.html)
- [After Effects Hidden Gems Weekly: Roving Keyframes — ProVideo Coalition](https://www.provideocoalition.com/after-effects-hidden-gems-weekly-roving-keyframes/)
- [Roving Keyframes in Adobe After Effects — Richard Harrington](https://www.richardharrington.com/blog/2025/12/9/roving-keyframes-in-adobe-after-effects-set-it-and-forget-it-constant-speed)
- [After Effects auto-orient flips when stopping motion — Adobe Community](https://community.adobe.com/t5/after-effects-discussions/after-effects-auto-orient-flips-when-stopping-motion/m-p/13331301/highlight/true)
- [Motion Path — Lottielab Docs](https://docs.lottielab.com/editor/canvas/layer-controls-huds/motion-path)
