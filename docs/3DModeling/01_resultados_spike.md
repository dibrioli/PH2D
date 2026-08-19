# W0 — resultados do spike do campo implícito

**Medido em 2026-08-19**, workstation Linux (32 núcleos), `fidget` **0.5.0**, build `--release`.
Reprodução: `cd spikes/field-spike && cargo run --release` (e `--features jit` para o §6).
⚠️ Todas as tabelas abaixo são **saída do programa**, não escritas de memória.

---

## Veredito: **o caminho se sustenta, e o buraco tem endereço**

| Pergunta da W0 (`03_plano_implicito.md` §6) | Resultado |
|---|---|
| 1. Quina viva | ❌ **REPROVOU** — a aresta é **quantizada à grade** (§2). É o item de trabalho que a W0 existiu para achar |
| 2. Os dois caracteres de fillet | ✅ entregues, e **visivelmente diferentes** (§1) |
| 3. O raio pedido é o entregue? | ✅ **exato: erro 0,00 %**. ⚠️ orgânico: **entrega 75 % do pedido**, sempre (§4) |
| 4. Resolução × tempo × memória | ✅ **35 ms** a 128³, **118 ms** a 256³, pico **105 MiB** (§5) |
| 5. Intérprete × JIT | ✅ **decidido: JIT fica DESLIGADO** — ganho **zero** (§6) |
| 6. Determinismo (HR-5) | ✅ **byte-idêntico** entre corridas (§7) |
| *(extra)* ponte `fidget → ph2d_mesh` | ✅ funciona; o STL saiu pelo exportador da casa (§8) |

⛔ **Nenhum kill-criterion disparou.** O do raio (*"se o raio pedido não for o entregue, PARA"*) é
justamente o que passou com nota máxima. O da quina abre trabalho nomeado, que é o que ele manda:
*"a extração de malha é reescrita por nós ou trocada; **não se afrouxa a barra**"*.

---

## §1 — A imagem (o entregável que o Enio julga)

![comparativo](imagens/w0_comparativo.png)

Da esquerda para a direita: **cubo** (a prova da quina) · **junção de 3 sem arredondar** ·
**arredondamento exato (r = 0,12)** · **arredondamento orgânico (k = 0,12)**.

A peça é a que quebra o Bevel do Blender: três cilindros ortogonais com **vértice triplo** no
centro. Os dois arredondamentos **fecharam o vértice triplo sem falhar** — que era a promessa
central do caminho implícito.

E de perto, que é onde a imagem pode **reprovar** o motor:

![comparativo de perto](imagens/w0_comparativo_zoom.png)

⚠️ **O painel 1 mostra o defeito**: o fio da aresta do cubo sai **serrilhado**. Os painéis 3 e 4
(arredondados) saem limpos — porque ali não há feição viva para capturar.

> ⚠️ **O sombreamento é PLANO de propósito.** Normal interpolada por vértice alisaria a transição e
> faria um canto arredondado passar por canto vivo. A imagem tem de poder reprovar.

---

## §2 — A quina viva: reprovou, e o mecanismo está NOMEADO

Erro da malha **nos vértices** (a malha está sobre a superfície?), profundidade 7, célula 0,01562:

| cena | triângulos | vértices | tempo | erro médio | erro máx | máx em células |
|---|---:|---:|---:|---:|---:|---:|
| cubo | 41.220 | 20.612 | 19,5 ms | 2,40e-7 | 2,50e-7 | **0,000** |
| junção — união dura | 111.380 | 55.692 | 29,0 ms | 8,85e-6 | 4,11e-4 | 0,026 |
| junção — arredondamento exato | 108.428 | 54.216 | 38,7 ms | 1,59e-5 | 9,39e-4 | 0,060 |
| junção — arredondamento orgânico | 107.690 | 53.847 | 57,4 ms | 1,36e-5 | 7,66e-4 | 0,049 |

**A malha está sobre a superfície com precisão excelente** (o cubo a 2,5e-7 — zero para efeitos
práticos). Isso **não** é a mesma pergunta que a quina, e é por isso que há a sonda seguinte.

### §2.1 — A sonda da aresta, e o achado

*"Existe vértice de malha sobre a aresta ideal?"*, fatiando a aresta em faixas de uma célula:

| meia-aresta | prof. | célula | canto médio | canto pior | aresta média | aresta pior | fatias capturadas |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0,45 | 6 | 0,03125 | 0,57 | 0,57 | 0,40 | 0,40 | **0/25** |
| 0,45 | 7 | 0,01562 | 0,99 | 1,13 | 0,80 | 0,80 | **0/49** |
| 0,45 | 8 | 0,00781 | 0,74 | 0,85 | 0,60 | 0,60 | **0/98** |

*(tudo em frações de célula; "capturada" = existe vértice a menos de ¼ de célula da aresta ideal)*

**Refinar não cura**: em unidades de célula o desvio fica onde está. E então a sonda do
**mecanismo** — a mesma medição, variando só **onde a face cai dentro da célula**:

| meia-aresta | face em células | **fração** | canto médio | aresta média | aresta pior | capturadas |
|---:|---:|---:|---:|---:|---:|---:|
| 0,5 | 32,00 | **0,00** | 1,41 | 1,00 | 1,00 | 0/55 |
| 0,25 | 16,00 | **0,00** | 1,41 | 1,00 | 1,00 | 0/28 |
| 0,4703125 | 30,10 | **0,10** | 0,12 | **0,10** | 0,10 | 51/52 |
| 0,4609375 | 29,50 | **0,50** | 0,62 | **0,50** | 0,50 | 0/51 |
| 0,4765625 | 30,50 | **0,50** | 0,62 | **0,50** | 0,50 | 0/52 |
| 0,45 | 28,80 | **0,80** | 0,99 | **0,80** | 0,80 | 0/49 |

### ⚠️ O achado, em uma linha

**O desvio da aresta é IGUAL à fração de célula em que a face cai** — 0,10 → 0,10 · 0,50 → 0,50 ·
0,80 → 0,80. Não é ruído nem aproximação: é **quantização à grade**. O vértice mais próximo da
aresta verdadeira senta na linha da grade, não na aresta; e como células vizinhas quantizam para
lados diferentes, o fio serrilha — que é exatamente o que a imagem de perto mostra.

*Um número que reproduz a variável de entrada com três casas não é uma medição ruidosa: é uma
função. É por isso que este achado é um **mecanismo** e não um sintoma.*

⚠️ **Consequência para o produto, e ela é limitada:** o defeito só aparece onde há **feição viva** —
um cubo, uma união dura. **O caso que este módulo existe para servir (arredondado) não o exibe**, e
os painéis 3 e 4 provam. Mas um cubo é o básico do básico: isto **tem** de ser resolvido.

**Hipóteses para a 2ª tentativa** (⛔ nenhuma verificada ainda — são hipóteses, não conclusões):
o QEF pode estar a ser fixado (*clamped*) aos limites da célula · a deteção de feição pode não estar
a disparar · pode ser opção da implementação e não do algoritmo. A conferência decisiva é comparar a
mesma entrada com o **`libfive`** (o mesmo autor, e a implementação que ele próprio diz ser a mais
testada — `03_plano_implicito.md` §2.2).

---

## §3 — A propriedade de distância

| cena | média | mín | máx | desvio máx | pior ponto |
|---|---:|---:|---:|---:|---|
| cubo | 1,0000 | 0,9998 | 1,0000 | **0,0002** | — |
| união dura | 1,0000 | 0,9514 | 1,0000 | **0,0486** | quina (derivada não existe lá) |
| arredondamento **exato** | 1,0148 | 0,9514 | **1,4142** | **0,4142** | (−0,19, 0,19, −0,19), f = −0,011 |
| arredondamento **orgânico** | 0,9665 | **0,7089** | 1,0000 | **0,2911** | (0,26, 0,25, 0,01), f = 0,003 |

### ⚠️ Isto CORRIGE o plano

O `03_plano_implicito.md` §3.1 previa que **encadear** arredondamentos seria o que degradaria o
campo. **Errado** — medido:

| operador | aplicações | mín | máx | desvio máx |
|---|---:|---:|---:|---:|
| exato | 1 | 0,9514 | 1,4132 | **0,4132** |
| exato | 2 | 0,9514 | 1,4142 | **0,4142** |
| orgânico | 1 | 0,7089 | 1,0000 | **0,2911** |
| orgânico | 2 | 0,7089 | 1,0000 | **0,2911** |

**Uma aplicação já degrada o mesmo que duas.** Encadear não compõe o dano. A degradação é
**local**, onde duas superfícies se tocam quase **tangentes** (`√2` é a assinatura exata de dois
gradientes paralelos somados) — nos cilindros, o ponto em que dois deles se osculam. Não é o filete
transversal, que sai exato (§4).

*A cura que o plano prescrevia (rastrear Lipschitz pela cadeia, re-distanciar) estava a mirar o
alvo errado. **Mecanismo certo, cura errada** — e é por isso que se mede antes de construir.*

---

## §4 — O raio pedido é o raio entregue? **O resultado que valida o caminho**

Sonda analítica (dois meios-espaços, sem malha no meio):

| operador | raio pedido | raio entregue | erro relativo |
|---|---:|---:|---:|
| **exato** | 0,05 | **0,0500** | **0,00 %** |
| **exato** | 0,12 | **0,1200** | **0,00 %** |
| **exato** | 0,25 | **0,2500** | **0,00 %** |
| orgânico | 0,05 | 0,0375 | **25,0 %** |
| orgânico | 0,12 | 0,0900 | **25,0 %** |
| orgânico | 0,25 | 0,1875 | **25,0 %** |

⭐ **O operador exato entrega exatamente o raio pedido.** É a premissa central do módulo — *o raio
do fillet é um parâmetro editável e confiável* — e está **provada**, não assumida.

⚠️ **E uma lei de produto cai daqui:** o `k` do orgânico **não é um raio**. Ele entrega
**exatamente 3/4** do número, em todos os raios testados. Logo: ou o painel o calibra (×4/3) ou o
rotula como outra coisa. ⛔ Expor `k` com a etiqueta "raio" seria mentir ao utilizador de 25 %,
sempre — e o [`feedback_a_label_must_promise_what_the_model_delivers`](../../project-memory/feedback_a_label_must_promise_what_the_model_delivers.md) já registra esta família de erro.

---

## §5 — Resolução × tempo × memória (os números que viram teto)

| profundidade | grade | triângulos | tempo | pico RSS |
|---:|---:|---:|---:|---:|
| 5 | 32³ | 7.116 | **2,7 ms** | 67 MiB |
| 6 | 64³ | 28.814 | **9,9 ms** | 67 MiB |
| 7 | 128³ | 108.428 | **35,1 ms** | 73 MiB |
| 8 | 256³ | 372.540 | **117,7 ms** | 105 MiB |

**Malhar em 64³ custa 10 ms** — cabe num quadro de 60 fps com folga. Em 128³, 35 ms: cabe em
"malha grossa ao mexer, fina ao parar". ⚠️ Isto é **um núcleo, sem GPU, sem JIT**.

**Consequência para o §5.3 do plano:** a marcha de raios em GPU **não é necessária para começar** —
e um segundo avaliador não entra sem número que o justifique.

---

## §6 — Intérprete × JIT: o `unsafe` **não se paga**

| profundidade | intérprete | JIT | ganho |
|---:|---:|---:|---:|
| 5 | 2,7 ms | 3,0 ms | **−11 %** |
| 6 | 9,9 ms | 10,1 ms | **−2 %** |
| 7 | 35,1 ms | 36,8 ms | **−5 %** |
| 8 | 117,7 ms | 119,8 ms | **−2 %** |

⛔ **O JIT fica DESLIGADO.** Ganho zero (dentro do ruído, e do lado errado). O HR-2 exige
justificativa escrita para `unsafe`; aqui não há sequer o que justificar. **O relógio da malhagem
está na travessia do octree e no QEF, não na avaliação do campo** — e é por isso que acelerar o
avaliador não move o resultado.

---

## §7 — Determinismo (HR-5)

Duas corridas na profundidade 6: **byte-idênticas** (14.409 vértices, 28.814 triângulos).
O `chacha20` que aparece no grafo de dependências **não** alcança este caminho.

## §8 — A ponte para a casa

`fidget::mesh::Mesh` → `ph2d_mesh::Mesh::from_parts` → `ph2d_mesh::write_stl`: **funciona**, sem
recusa de validação, nas quatro cenas. O núcleo da W2 está provado antes de a W2 começar.

---

## Decisões forçadas pelos números

1. ⛔ **JIT desligado** (§6) — e a feature fica no `Cargo.toml` só para re-medir.
2. ✅ **O operador EXATO é o default** (§4) — validado a 0,00 %.
3. ⚠️ **O `k` do orgânico não vai à UI como "raio"** (§4) — calibrar ×4/3 ou renomear.
4. ⚠️ **A malhagem é o item de trabalho da W1/W2** (§2) — com o mecanismo já nomeado.
5. ⛔ **Sem GPU por ora** (§5) — 10 ms a 64³ num núcleo não justifica um segundo avaliador.
6. ⛔ **Não rastrear Lipschitz pela cadeia** (§3) — a degradação não vem do encadeamento.

## ⛔ Recusas MEDIDAS

| Recusa | Número que a sustenta |
|---|---|
| Ligar o JIT da `fidget` | ganho **−2 % a −11 %** (§6) |
| Segundo avaliador em GPU antes de precisar | 64³ em **9,9 ms** num núcleo (§5) |
| Re-distanciamento / Lipschitz encadeado como cura | 1 aplicação degrada **igual** a 2 (§3) |
| Expor o `k` do orgânico como "raio" | entrega **75 %** do pedido, sempre (§4) |
| Culpar a resolução pelo serrilhado | refinar **não** muda o desvio em células (§2.1) |
