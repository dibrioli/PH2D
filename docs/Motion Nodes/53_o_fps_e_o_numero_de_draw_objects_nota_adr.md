# 53 — **O FPS é o número de DRAW OBJECTS** (e o doc de boot enxugou) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** perf do editor + limpeza do demo
**Status:** implementado, **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** `ph2d-editor-core::paint_batch` (**módulo NOVO**, aditivo)

---

## 1. O sintoma e a MEDIÇÃO (não o palpite)

> *"o grafo como está tem severa queda de FPS. investigue"*

Medi antes de teorizar (dev profile — o mesmo do `cargo run`):

| O quê | Custo por frame |
|---|---|
| **o cook** (a simulação inteira: 42 nós, 165 instâncias) | **0.44 ms** |
| **o snapshot** que o editor publica (snapshot_from + stamp) | **0.12 ms** |

A simulação **não era o problema** — e essa era a suspeita "óbvia". Sobrou a **pintura**.

## 2. A causa: **4 000 objetos de desenho por frame**

O custo da Vello é **por DRAW OBJECT**, não por vértice. Cada draw object precisa ser delimitado, binado e
rasterizado.

Os **postage stamps** (doc 47) desenhavam **um `scene.fill()` por ponto**: 42 cards × 96 pontos = **4 032 objetos
por frame**, só de decoração. Mais ~570 dos **dashes** da marcha (13 strokes por fio, em cada fio, todo frame). O
painel inteiro passou de ~400 para ~5 000 objetos — e o frame rate foi junto.

**Encode medido (4 032 pontos):** 2.92 ms por-fill → **1.49 ms** numa path só. E o ganho de CPU é o **menor** dos
dois: o de GPU é a queda de **4 032 → 42** objetos.

## 3. O conserto

- **`ph2d-editor-core::paint_batch`** (módulo novo): `fill_dots` (N quadradinhos, **UM** fill) e `stroke_subpaths`
  (N polilinhas disjuntas, **UM** stroke). Um stamp = 1 objeto. Uma marcha de fio = 1 objeto.
- **Os pontos são QUADRADOS**, e não é concessão: no raio que um stamp usa (1–2 px) um círculo e um quadrado cobrem
  os **mesmos pixels**, e 4 retas achatam de graça onde 4 cúbicas não achatam (2.92 → 1.49 ms).
- **`PREVIEW_POINTS` 96 → 48**: ninguém conta os pontos de uma miniatura.
- **Cards fora da tela não são desenhados.** O clip já os escondia — mas a Vello ainda delimitava e binava cada path
  dentro deles. (Os hit rects já eram clipados, então o invisível continua inclicável.)

**Resultado:** o painel do grafo saiu de **~5 000** para **~250** objetos de desenho por frame.

## 4. E o doc de boot enxugou

> *"deixe só o grafo da chuva. retire os outros."*

O documento de boot agora é **só a neve** (19 nós, 1 sink). As cenas de rig (rubber hose, carne skinada num tentáculo
FABRIK, o goal que respirava) e o card órfão que demonstrava o véu inerte **saíram** — e isso sozinho **corta o
painel pela metade**. Elas vivem no git; **todo nó que usavam segue registrado e testado na própria crate** — o que
morreu foi a cópia do boot, não a cobertura. **Um documento de boot é um demo, não um arquivo.**

## 5. A lição

**"Está lento" é uma pergunta sobre ESCALA, e escala se mede.** A suspeita natural era a simulação (é ela que faz
física a 60 Hz) — e ela custava **0.44 ms**. O culpado era a **decoração**, que ninguém pensa em contar: 96 pontinhos
por card, inocentes um a um, catastróficos como 4 032 objetos de desenho.
