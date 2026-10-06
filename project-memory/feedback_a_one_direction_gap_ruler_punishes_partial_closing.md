---
name: a-one-direction-gap-ruler-punishes-partial-closing
description: Uma régua de vão que mede só numa direcção (fio vertical a 1 px) penaliza o fecho PARCIAL de uma cunha, e reverteu uma lei certa; fixe o vão no render de referência e pontue cada variante nele.
metadata:
  type: feedback
---

⛔⛔ **A «regressão» que reverteu o A5-a a 04/10 era a RÉGUA, não a lei.** A régua `buracos_de`
(`smoke_bone_par_fresta_tests.rs`, fio só na vertical dentro de 1 px) PENALIZAVA o fecho parcial de uma
cunha: a −149,5° a lei de hoje dá 4 amostras na ponta e a marcha 12, mas numa cunha MENOR (fechou parte
dela). Quatro desenhos foram medidos e caídos por ela;
a 06/10 a régua de vão FIXO (vão = tinta da imagem sem costura fechada por um disco de 1 px, menos a
tinta; fixo por pose) mostrou que nenhuma pose fica pior que a lei de hoje e a cúspide baixa
(27 poses: sem costura 3114 · Velha 2368 · lei nova 1852).

**Why:** uma régua cujo denominador muda com a variante (o vão depende de quanto a variante fechou)
premeia quem fecha menos. E um segundo mentiroso na mesma obra: `maior_vao_cosido` lia os pontos da
costura de 4 em 4 depois de os remendos passarem a 8 pontos ⇒ media METADE do vão.

**How to apply:** (1) fixe o vão no render de REFERÊNCIA (sem a lei) e pontue cada variante nesse mesmo
conjunto de amostras; (2) a régua de sobreposição tem de respeitar a ORDEM de pintura (`visiveis_sobre_tinta`,
ordem dos triângulos), não só a geometria; (3) depois de mudar a FORMA de um dado (4 → 8 pontos), releia
todo helper que o conta ou desagrupa por formato, e todo leitor que o descodifica (4 leitores liam a
struct nova directamente e deitariam fora todo bind antigo). Fila `docs/Skeleton/01_a_fila.md` §F65.
