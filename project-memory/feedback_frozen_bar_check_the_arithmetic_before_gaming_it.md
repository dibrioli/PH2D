---
name: feedback-frozen-bar-check-the-arithmetic-before-gaming-it
description: Barra de aceite congelada ficou vermelha? Cheque se a barra era POSSÍVEL antes de mexer no código — e nunca passe por um caminho que o produto não usa
metadata:
  type: feedback
---

Congelei o aceite do [ADR-0117] antes de implementar (que é o certo). A barra A1 era
"pico ≤ 128 MB". A implementação mediu **156 MB** e ficou vermelha.

Os dois erros que eu quase cometi, nessa ordem:

1. **Culpar a implementação.** Fiz a conta primeiro: 65,9 (clipe) + 65,9 (buffer novo) + 23,4
   (deltas) = 155,2. **Dois buffers cheios são irredutíveis** — um buffer novo tem de existir
   antes que o antigo possa ser solto. `2 × 65,9 = 131,8` **já estourava os 128 sozinho, com
   histórico zero**. A barra era aritmeticamente impossível. Eu a escrevi antes de entender que
   o buffer de preview é estrutural, não desperdício.

2. **Passar o gate pelo caminho errado.** Existia um caminho in-place (`samples_mut`, CoW) que
   faria `apply_effect` escrever por cima do clipe: o gate passaria em ~90 MB. Mas **o produto
   não usa `apply_effect`** — ele renderiza um preview (o mixer TOCA o preview antes do commit)
   e depois commita. Otimizar o caminho que só o teste percorre é **maquiar o número**, não
   consertar a memória. O gate ficaria verde e o usuário continuaria com o mesmo pico.

**Why:** uma barra congelada é uma âncora contra auto-engano — mas ela também pode estar
simplesmente errada, e aí a disciplina vira teatro. A âncora só vale se, quando ela quebra, eu
investigo **os dois lados**: o código E a barra. E a tentação de "fazer o gate passar" sempre
tem um atalho que otimiza o caminho medido em vez do caminho real.

**How to apply:** barra congelada ficou vermelha → (a) refaça a **aritmética do piso**: qual é o
custo IRREDUTÍVEL desse cenário? Se o piso já estoura a barra, a barra estava errada — emende-a
**em voz alta, com a conta**, nunca em silêncio. (b) Antes de otimizar, pergunte: **o produto
percorre este caminho?** Se o caminho medido ≠ o caminho do produto, passar o gate é fraude
consigo mesmo. (c) Prefira barras **estruturais** ("≤ 2×clipe + 32 MB") a absolutas ("≤ 128 MB")
— a estrutural diz a propriedade que você quer (*o editor segura o clipe, um buffer em
construção, e deltas — não N clipes*) e não depende da fixture.

Parente de [[feedback_measure_perf_symptom_scale]] (meça a escala antes da causa) e de
[[feedback_tool_unit_green_integration_dead]] (verde no unit, morto no produto).
