# 3D Modeling — modelagem por **campo implícito** (porta do módulo)

> ⚠️ **Não confundir com [`docs/3D/`](../3D/README.md)**, que é o módulo de **escultura**
> (`ph2d-sculpt3d`, malha + verbos, ADR-0150). Este aqui é **modelagem**: booleana e arredondamento
> de aresta que **não podem falhar**, porque são aritmética de campo e não topologia de malha.
> São dois módulos, duas linhas — e eles **se encontram** (uma malha esculpida pode entrar na
> booleana deste, via `ph2d-sdf`).

**A promessa, em uma frase:** o modelo é uma **função**, não uma malha — então união e arredondamento
são `min` e um operador sobre dois números, e o **raio do fillet fica editável para sempre**.

| Doc | O que é |
|---|---|
| [`03_plano_implicito.md`](03_plano_implicito.md) | ⭐ **O PLANO VIVO.** A rota escolhida: a tese, o motor (`fidget`, medido), o arredondamento e sua armadilha, a quina viva, arquitetura, waves e kill-criteria |
| [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md) | **Por que esta rota.** Mede que o Blender 4.5 já resolveu a booleana e que o buraco é o arredondamento. As 3 famílias candidatas |
| [`00_plano_port.md`](00_plano_port.md) | ⛔ **Rota substituída** — não execute as waves. Continuam fonte: o **§1** (estudo do original: 9 leis, 19 operações) e o **§2** (inventário da PH2D). O **§7** segue válido: por que **não** se escreve um kernel do zero |

**Estado:** plano escrito, zero código. A **W0 bloqueia tudo** e entrega **uma tabela e uma imagem**
— a peça que quebra o Bevel do Blender (três volumes no mesmo vértice), arredondada pelos dois
caracteres, para o Enio julgar olhando. Kill-criteria congelados em `03_` §6.

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (clone de UX do
[MoI 3D](https://moi3d.com)). O **fluxo** dele é o alvo de UX; o kernel NURBS dele, não.
