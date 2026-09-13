---
description: Plano primeiro, produto final (não MVP), UI incluída.
argument-hint: [Título] [Objetivo]
---
Wave nova: $1

O que o artista deve conseguir fazer:
$2

Antes de escrever código, me entregue o plano:
1. Outro app já faz isto? Triagem de licença POR ARTEFACTO instalado (porta aberta ⇒ porta-se,
   com atribuição); o alvo é um ORÁCULO que se CORRE sem interface sobre entradas nossas,
   ⛔ nunca um fonte que se lê (`docs/_ComoInvestigarApps/`, CLAUDE.md §0.9). Depois o estado da
   arte (Blender/AE/Illustrator/Rive…), incluindo o que foi TENTADO e abandonado por eles.
1b. Onde o código mora: família em `crates/ph2d-app-<família>` (a shell é composição, com tecto
   que só desce); ids no módulo `ids` da crate que os LÊ.
2. O desenho, com a porta ÚNICA de cada pergunta (duas portas divergem em silêncio).
3. Onde isso encosta em contrato congelado (§6) ou schema — e a prova por grep de que
   não encosta, se for o caso.
4. O que a UI precisa: o componente EXISTE · é pintado e registrado · o clique chega ao
   barramento · e a SEQUÊNCIA leva a algum lugar (as 4 condições são independentes).
5. Os gates, red-first, e a fixture que contém o fenômeno.
6. A cena de smoke com números MEDIDOS (rode a sonda headless ANTES de escrever a msg).

Padrão-ouro sem custo: a melhor opção técnica vence custo de build/cronograma; gaps
in-scope fecham nesta sessão.
