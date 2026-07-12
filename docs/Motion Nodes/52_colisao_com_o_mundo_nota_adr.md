# 52 — **Colisão com o mundo** (`sim.collide`) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **O4** (parte 5 — fecha o vocabulário da sim)
**Status:** implementado, testado (mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. O que faltava (e por que só agora dava para ter)

O `motion.collide` que já existia é **push-apart**: instâncias se afastando umas das outras (PBD). Ele não sabe o
que é um chão — e **não podia saber o que é velocidade**, porque **fora de uma zona não existe velocidade a
refletir**: um stream re-autorado do zero a cada frame tem posições e nenhuma história.

Dentro da zona existe. A `vel` mora no estado — então um colisor pode fazer a coisa que faz uma colisão **parecer**
uma colisão: **refleti-la**.

## 2. O contato, escrito UMA vez

Toda forma se reduz aos mesmos dois números — **normal unitária de saída `n`** e **profundidade `d`** — e a
resposta é escrita **uma única vez**, para todas:

```text
  p += n·d                      // para fora da parede, exatamente sobre a superfície
  vn = v·n                      // …só se ainda estiver ENTRANDO nela (vn < 0)
  v  -= (1 + restitution)·vn·n  // reflete a componente normal
  vt *= 1 - friction            // e sangra a tangencial
```

Escrever a resposta por-forma é como um colisor cria **um bug por forma**. Este tem **três formas e uma resposta**:
**Floor** (o mundo é acima) · **Disc** (obstáculo sólido: o mundo é fora) · **Bowl** (recipiente: o mundo é dentro)
— a mesma matemática, a normal invertida.

## 3. Os três mutantes

### 3.1 Só empurrar a posição = **atravessa mesmo assim**

O colisor "clássico errado" clampa a posição e deixa a velocidade apontando **para dentro** — então o tick seguinte
a enfia de volta na parede, e ela **vaza** pelo chão ou o **rala** para sempre. Guarda:
`a_particle_hitting_the_floor_comes_back_up` (sai **exatamente** sobre a superfície, subindo, com `restitution` da
velocidade com que chegou).

### 3.2 Refletir em TODO contato = **JITTER**

Só quem está **entrando** na superfície é refletido (`vn < 0`). Uma partícula que já está deslizando ou **parada**
no chão tem `vn ≈ 0` — refleti-la mesmo assim é o *jitter* clássico: a coisa **zumbe no chão para sempre**,
alimentada pelo próprio teste de contato, e a pilha de partículas assentadas **ferve**. Guarda:
`a_resting_particle_does_not_jitter`.

### 3.3 Restituição > 1 = **máquina de energia**

Devolver mais do que tomou põe a cena em órbita. Clampado, não confiado — e a guarda mede o que importa: **nenhum
quique ganha velocidade**, em nenhuma forma, em nenhuma restituição (`a_bounce_never_gains_speed`).

Bônus: o **centro exato de um disco** não tem "lado de fora" — qualquer direção serve. Ele escolhe uma em vez de
dividir por zero e transformar o elemento num **NaN que envenena o estado inteiro**.

HR-5: as normais são **geometria**, não ângulos (a do chão é para cima, a do disco é radial). Nada aqui precisa de
seno.

## 4. Demo: a neve **assenta**

`sim.step` → **`sim.collide` (Floor, y = −2)** → `sim.lifetime` → … Os flocos caem, **pousam**, dão um pulinho,
deslizam e **derretem onde estão** (o chão fica bem dentro do disco de kill, então quem pousa morre de **velhice**,
não ceifado na borda).

Guarda de produto no doc de boot: **nada fica abaixo do chão** (o mutante 3.1 vazaria) **e há floco pousado nele**
(senão o chão seria só desenho).

## 5. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| crate nova | **`ph2d-node-sim-collide`** (`sim.collide`) → **86 crates-nó** |
| shapes | `SHAPE_FLOOR = 0` · `SHAPE_DISC = 1` · `SHAPE_BOWL = 2` |
| shell | a neve do doc de boot pousa no chão |

## 6. A lição

**Um nó só pode ser tão inteligente quanto o estado que ele consegue ver.** O `motion.collide` não era burro: ele
vivia num mundo sem velocidade. A zona não deu "mais um nó" — deu um **substrato** em que colisão, idade, morte e
nascimento passam a ser expressáveis, e cada um deles é pequeno **porque** o substrato existe.
