---
name: the-quantity-that-accuses-a-product-is-the-one-it-does-not-normalise
description: Uma régua sobre a grandeza que o produto normaliza nunca o acusa; e cobertura booleana é cega à sobreposição.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-09-01T00:51:04.374Z
---

Ao construir a régua que há-de acusar um defeito, escolha uma grandeza que o produto
**não regula**. Se ele resolve um factor para pôr *X* numa rampa, toda sonda sobre *X*
concorda com ele — inclusive quando os dois estão errados.

⚠️ **E a segunda metade, que só apareceu na 2.ª tentativa: uma régua de COBERTURA
(rasterizar e perguntar que células foram tocadas) é cega à SOBREPOSIÇÃO.** Traço
desenhado duas vezes sobre o mesmo caminho toca as mesmas células e não move um pixel
da métrica — mas move a imagem, porque a tinta acumula.

**Why:** L-System, 2026-08-31 (report *«dá pequenos pulos»*). O `build` normaliza a
**largura média** da figura para o tamanho crescer numa recta; as três sondas que
existiam (`probe_flicker`, `probe_drift`) mediam exactamente largura média, span de
eixo e centroide, e as três diziam **liso**. A grandeza livre era a **TINTA** (soma dos
comprimentos desenhados): ela saltava `67 %` num intervalo de `1e-3`. Uma sonda de
imagem escrita a seguir também não o via — os `5` segmentos colineares que nascem sobre
o caminho do pai tocam as mesmas células que ele.

**How to apply:** antes de escrever a sonda, liste o que o código do produto
**resolve/normaliza/clampa**, e escolha o observável fora dessa lista. Depois pergunte
se ele soma ou satura: contagem de células **satura**, soma de comprimentos **acumula**.
E use o discriminador **salto vs movimento** — afine o passo `k×` e veja se a diferença
encolhe `k×` (movimento) ou fica onde estava (descontinuidade); uma leitura só não
distingue as duas. Ver [[a-cure-measured-through-a-noisy-consumer-reads-as-refuted]] e
[[measure-the-defects-structure-before-designing-its-cure]].
