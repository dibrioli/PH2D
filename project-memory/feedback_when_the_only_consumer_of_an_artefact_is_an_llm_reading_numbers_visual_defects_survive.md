---
name: when-the-only-consumer-of-an-artefact-is-an-llm-reading-numbers-visual-defects-survive
description: Um artefacto cujo único leitor sou EU (a ler números) pode carregar um defeito VISUAL indefinidamente — quem escolhe que defeitos são observáveis é o consumidor, não o código
metadata:
  type: feedback
---

O exportador de SVG do PH2D foi construído em 2026-09-02 a pedido do Enio com uma finalidade
declarada: *"precisamos de um meio de exportar o path para que vc possa analisar melhor"*. O único
consumidor era **eu**, e eu leio coordenadas.

Ele escrevia as coordenadas de MUNDO cruas (Y para cima) dentro de um `<svg>` (Y para baixo), então
**todo ficheiro exportado saía verticalmente espelhado**. O cabeçalho do próprio módulo afirmava o
contrário — *"em coordenadas de MUNDO (Y para baixo, como o SVG)"*, uma frase cujas duas metades se
contradizem — e **nenhum dos seis gates media orientação**: eles mediam tinta, pose, marca de balde,
o corte fill/stroke, o gradiente e a nota do cabeçalho.

O defeito só apareceu três dias depois, ao construir o IMPORTADOR — porque aí a mesma lei passou a
ter um segundo leitor, e os dois tinham de concordar.

**Why:** um defeito só é observado por quem tem o instrumento para o ver. Uma imagem espelhada é
invisível para quem lê números, e uma lista de números certos é invisível para quem olha. Enquanto o
único consumidor de um artefacto for uma LLM, a classe inteira de defeitos "visuais" dele fica sem
régua — e a ausência de queixas não é evidência de correcção, é evidência de que ninguém olhou.

**How to apply:** ao construir um artefacto cujo consumidor declarado é o agente (um dump, um
export de diagnóstico, um relatório), pergunte **que classe de defeito o meu leitor não consegue
ver** e escreva o gate dessa classe no mesmo commit — orientação, ordem, sinal, unidade. E quando
um segundo consumidor aparecer (um importador, um humano, outro programa), **reconfira o artefacto
inteiro**: a chegada dele não é uma feature nova, é a primeira medição.

Relacionado: [[feedback_stale_comment_and_dead_code_lie]] ·
[[feedback_a_gate_that_measures_the_rare_case_leaves_the_normal_one_without_a_ruler]] ·
[[reference_topic_gate_discipline]]
