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
| [`01_resultados_spike.md`](01_resultados_spike.md) | ⭐ **O QUE FOI MEDIDO (W0, 19/08).** As imagens, os números e as 6 decisões que eles forçaram. Nenhum kill-criterion disparou; a **quina viva** reprovou e tem mecanismo nomeado |
| [`03_plano_implicito.md`](03_plano_implicito.md) | **O PLANO VIVO.** A rota escolhida: a tese, o motor (`fidget`, medido), o arredondamento e sua armadilha, a quina viva, arquitetura, waves e kill-criteria |
| [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md) | **Por que esta rota.** Mede que o Blender 4.5 já resolveu a booleana e que o buraco é o arredondamento. As 3 famílias candidatas |
| [`00_plano_port.md`](00_plano_port.md) | ⛔ **Rota substituída** — não execute as waves. Continuam fonte: o **§1** (estudo do original: 9 leis, 19 operações) e o **§2** (inventário da PH2D). O **§7** segue válido: por que **não** se escreve um kernel do zero |

**Estado:** **W0 feita e verde** (19/08). O caminho se sustenta: o arredondamento exato entrega o
raio pedido com **0,00 %** de erro, e o vértice triplo — o caso que quebra o Bevel do Blender —
fecha sem falhar. **Um item aberto e nomeado:** a aresta viva sai quantizada à grade
(`01_resultados_spike.md` §2). A W1 começa por ele.

**Rodar o spike:** `cd spikes/field-spike && cargo run --release` (imagens em `out/`).

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (clone de UX do
[MoI 3D](https://moi3d.com)). O **fluxo** dele é o alvo de UX; o kernel NURBS dele, não.
