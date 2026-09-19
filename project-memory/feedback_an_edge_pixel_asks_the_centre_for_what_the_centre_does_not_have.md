---
name: feedback-an-edge-pixel-asks-the-centre-for-what-the-centre-does-not-have
description: Num pixel de silhueta o CENTRO pode falhar a peca — e ai o ponto, o ceu e a oclusao dele sao o valor INICIAL ou um sentinela; quem os pede ao centro pinta com um material que nao esta la
metadata:
  type: feedback
---

⛔⛔⛔ **Um pixel de silhueta cujo CENTRO falha a peça não tem ponto, nem céu, nem oclusão próprios —
e o código que os pede ao centro não falha: ele lê o valor INICIAL ou um sentinela, e pinta.**

Medido 2026-09-19 (`line/3DModeling`, report do dono com foto: *«toda forma apresenta uma falsa
outline branca de 1 pixel»*). O anti-serrilhado da silhueta corre num laço à parte que declara, por
escrito, *«a vista e o material do CENTRO servem às quatro sub-amostras»* — uma aproximação legítima
**que supõe que o centro acertou**. Quando ele falha:

* na CPU a marcha não escreve `point[i]`, que fica no inicial **`[0,0,0]`** (a origem do mundo);
* no dispositivo fica pior: `origem + direcção × t` com **`t < 0`** — **atrás da câmara**. E o mesmo
  shader já testava `centro[i].x < 0.0` duas funções acima, noutra lei.

⭐⭐⭐ **O material sai da POSIÇÃO** (`mix_of(p, …)` / `dono_mix(p, …)`), logo os dois pediam-no a um
ponto fora da superfície. No dispositivo aquele ponto caía numa folha **EMISSIVA**, e uma emissiva
depois da exposição é **BRANCA**: um fio branco de um pixel à volta de toda a silhueta, *visível só
nas peças escuras porque a cor que ele põe é sempre a mesma*.

Sobre os `555` pixels em causa: Δ verde médio dispositivo−CPU **`+56,3` → `0,0`**; pico da silhueta
no dispositivo **`+72,8` → `0,0`**.

⛔⛔⛔ **E empresta-se SÓ o PONTO — a cura maior foi construída, MEDIDA e REVERTIDA.** Emprestar
também o céu, a sombra e o ricochete (o ÍNDICE, e não só a posição) argumenta-se sozinho — *um
centro que falha não tem nenhuma das três* — e **não move o rebordo um byte** (`Δ` da silhueta
`+0,0` das duas maneiras), enquanto parte **NOVE** paridades entre os motores. *Uma cura maior do
que a medição pede é uma regressão com um bom argumento ao lado.*

**Why:** o defeito é invisível a toda régua de paridade e a todo golden **do interior**, porque ele
vive só na fronteira; e é invisível a olho em tudo o que não é escuro. Ele sobreviveu meses.

**How to apply:**
- Num laço de borda/anti-serrilhado, pergunte **o centro acertou?** antes de usar qualquer coisa
  derivada dele. Se não acertou, empreste o **PONTO** do primeiro vizinho de cruz que acertou (a
  mesma ordem nos dois motores) — e **meça antes de emprestar mais do que isso**.
- ⚠️ Um sentinela (`t < 0`) e um valor inicial (`[0,0,0]`) **não falham**: eles produzem uma resposta
  plausível. Procure-os onde um buffer é escrito só para uma parte dos índices.
- A régua que apanha isto compara **os dois motores na mesma fronteira**, não o interior.

Irmãos: [[feedback-a-gesture-written-in-two-halves-accepts-a-new-variant-in-only-one]] ·
[[reference-topic-measurement-discipline]] (a régua com LIMIAR de brilho que comparava dois motores
mediu a diferença de brilho DELES: o dispositivo pinta a barra `19` bytes mais escura, o limiar caía
dentro da peça num e fora no outro, e eu li a CPU como limpa sobre o mesmo defeito).
