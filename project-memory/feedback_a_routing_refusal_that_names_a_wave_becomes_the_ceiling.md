---
name: feedback-a-routing-refusal-that-names-a-wave-becomes-the-ceiling
description: "Escada de roteamento que nomeia uma WAVE em vez de um recurso vira o teto do produto — o instrumento é o CENSO da população que ela atinge"
metadata:
  type: feedback
---

Uma escada que atira trabalho do caminho rápido para o lento e se justifica com **escopo de
wave** (*"F2+ territory"*, *"por ora"*, *"fase 1 só faz X"*) não é um limite: é o tecto do
produto, escrito onde ninguém o lê como tecto. O §0.0 exige que um limite diga **de que
recurso** ele é; um que diz de que *wave* ele é passa por toda revisão porque **lê-se como um
plano**, não como um número.

Medido 2026-09-01 (auditoria de performance do Motion): `gpu_route` recusava ao device todo
grafo com **mais de um sink**, com o doc a dizer *"F1.1's scope; F2+ territory"*. O device faz
**4,19 M objectos em 3,85 ms** contra **195,9 ms da CPU** — `50,9×`. Correndo as **109 cenas
que o produto expõe** e perguntando a rota a cada uma: **69,7% caem para a CPU**, `67%` por
aquela escada, e as cenas atingidas são **as 36 a 109** — *tudo o que o módulo construiu desde
que ela existe nasceu do lado lento*.

⭐⭐ **O que a tornou accionável não foi reler o código — foi CONTAR a população.** A nota
estava legível havia meses. O censo (uma sonda `#[ignore]` que planeia cada cena e imprime a
rota) transformou-a numa decisão com preço; e planear **cada sink sozinho** deu o preço de a
levantar: **23 das 73 cenas já teriam TODOS os sinks no device** — falta só compor dois planos
num buffer.

⚠️ **A metade gémea: a queda era MUDA.** `FellThrough` era consumido e não acendia nada; um
grafo a 4 M no device e o mesmo num núcleo só têm a mesma aparência na UI. *Um custo de 50× que
nenhuma superfície nomeia não é escolha de ninguém — é um acidente que se repete.*

**Why:** «recusado por medição» e «ainda não construído» leem-se iguais num comentário, e o
segundo passa a definir o produto sem que ninguém decida.

**How to apply:** ao encontrar uma recusa de roteamento, (1) pergunte *que recurso ela nomeia?*
— se a resposta for uma wave, é dívida, não limite; (2) **conte a população** que ela atinge
sobre o corpus real do produto, com uma sonda que imprime e não julga; (3) meça o que ela
custaria a levantar (cada metade sozinha já passaria?); (4) e faça a queda **dizer o próprio
nome** por trás de uma env de diagnóstico, com disparo por borda. Ver
[[feedback-the-ceiling-is-the-hardwares-never-the-fallbacks]] e
[[feedback-a-tool-is-adopted-only-when-a-written-step-names-it]].
