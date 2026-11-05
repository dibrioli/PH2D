# ADR-0198 — O motor do SFX Lab é de kernels próprios sobre `ph2d-det-math`; o FunDSP fica fora do render canônico

- **Status:** Accepted — desenho técnico delegado (padrão-ouro); sem IA por decisão do dono (05/11).
- **Data:** 2026-11-05
- **Linha:** `line/SfxLab`
- **Amparo:** [medição de W0](../../SfxLab/02_W0_medicao.md), com a [sonda](../../SfxLab/ferramentas/sonda_w0/corre.sh) · [plano](../../SfxLab/01_plano.md)

> ⚠️ Número por recontar na integração (§5.0): escrito como `0198` com o `main` em `0197`.

## Contexto

A receita de um som é a fonte, e `render(receita, semente, versão_do_motor) -> PCM` é a porta única: a prévia toca o buffer dessa função, então prévia
e arquivo não têm como divergir. Para isso o render tem de ser estável: editar uma camada não muda as outras, e a mesma receita dá os mesmos bytes.

O FunDSP 0.23 é a única biblioteca mantida e permissiva com grafo de verdade, e era a candidata. O plano mandou medir antes.

## Decisão

1. **O render canônico é de kernels próprios em `ph2d-sfx-dsp`, em `f32`, escalares, com toda função transcendente vinda de `ph2d-det-math`.**
   A crate entra na lista do gate `o_motor_nao_chama_a_libm_do_sistema` no commit em que nascer (W2).
2. **A semente de uma camada é `f(semente_da_receita, id_estável_da_camada)`**, nunca a posição dela.
3. **O FunDSP não entra no `Cargo.lock`.** A pergunta do som calculado ao vivo (W8) é outra e reabre-se lá, com medição própria.
4. **A promessa ao dono tem duas partes, e só uma está provada:**
   - «O som aprovado nunca muda»: vale em qualquer máquina, porque o áudio aprovado fica guardado.
   - «A mesma receita sem efeito dá os mesmos bytes em qualquer máquina»: provada só em AMD x86-64 de base, x86-64-v3 e wasm32. **Intel e ARM: NÃO PROVADO.**
   - Receita com efeito: não há garantia entre máquinas. Portar o rack exige antes `exp2`, `log2` e `tanh` em `ph2d-det-math`, e fica fora deste ADR.
5. **Cache por camada antes de W3:** o pior caso medido (38,8 ms) passa do limite proposto de 30 ms.

## Alternativas recusadas (medidas)

| Alternativa | Por que não |
|---|---|
| FunDSP em blocos, com semente e fase fixadas | Editar um vizinho muda o alvo quando há `envelope`, e `seed`/`phase` não o alcançam. Os bytes mudam com o alvo de compilação (AVX2) |
| FunDSP amostra a amostra | Os bytes são estáveis entre alvos, mas custa 165 ms contra 9,6 ms, e a armadilha do `envelope` continua |
| FunDSP sem o `envelope`, com envelope próprio por fora | Deixa de ser usar a biblioteca: restam os kernels dele, e os nossos custam o mesmo ou menos (9,6 × 11,9 ms na típica; 38,8 × 50,6 ms na pior) e passam na regra da semente |

## Preço

- Antialiasing, filtros e reverb são nossos, e cada um pede medição: `aliasing` com tom agudo varrido já é gate de W2.
- Os kernels ainda não foram ouvidos. O kill-criterion de W2 continua: corpo vibrante que o dono reprove no ouvido volta ao desenho antes de W3.
- O escalar deixa SIMD por fazer. Se o cache por camada não bastar, a cura tem de manter os bytes: SIMD automático muda o resultado com o alvo, como mediu o FunDSP.

## Reabrir quando

- O hash de Intel ou ARM divergir de `e262c4b2a017544c`: a promessa passa a ser só por máquina, e o dono é avisado.
- O ouvido do dono reprovar os kernels em W2.
