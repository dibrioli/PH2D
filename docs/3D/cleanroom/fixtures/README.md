# Fixtures — mapas de grade inteira de REFERÊNCIA, e o verificador deles

⭐⭐ **Estes arquivos são o INSUMO da extração** ([`SPEC_extracao_de_malha_quad.md`](../SPEC_extracao_de_malha_quad.md)
§2–§6). Com eles, a extração pode ser construída e gateada **sozinha**, sem esperar que o
arredondamento inteiro (espec §5) esteja pronto.

## Proveniência (§5 da SKILL_Cleanroom — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malha de entrada** | ⭐ **nossa** — `ph2d-quadbench/corpus/`, triangulada por nós |
| **Campo direccional** | ⭐ **nosso** — `ph2d-crossfield`, exportado pelo arnês `rustfield` |
| **Quem calculou o mapa** | uma implementação independente sob **MPL-2.0**, corrida **fora da árvore** como oráculo ([ADR-0164](../../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)) |
| **Estatuto legal** | ⭐ **dados.** Saída de programa não é coberta pela licença do programa — é texto de licença, não opinião (SKILL_Cleanroom §1.1) |
| **Regenerar** | `~/Referencias/directional-bench/` — o arnês, o exportador de campo e o modo `so-mapa` |
| **Data** | 2026-08-24 |

⛔ **Não há aqui nenhum asset de terceiros.** A malha da jarra usada nas primeiras medições
é **exemplo empacotado com a biblioteca** e por isso **NÃO** entrou (§1.5.3).

## As peças, e por que estas duas

| arquivo | peça | arestas interiores | ⭐ costuras (rotação ≠ 0) | por que ela está aqui |
|---|---|---|---|---|
| `sculpt_hooked.mapa.gz` | gancho orgânico, fechado, 6 768 triângulos | 10 151 | **247** | o caso realista |
| `torus_64x32.mapa.gz` | toro, **género 1**, 4 096 triângulos | 6 143 | **138** | ⭐ gate nº10 da espec: `χ = 0`, e é a peça que já expôs uma perda de asa nesta linha |

⚠️ **Uma peça sem costura não pode gatear a máquina de transição.** A primeira medição saiu
sobre um mapa com **`{0: 3535}`** — zero costuras — e teria deixado passar uma extração que
ignorasse transições por completo. *Fixture só prova o que contém.*

⛔ **Não há fixture com BORDO, e não é esquecimento:** a integração do oráculo **cai**
(SIGSEGV) em malha com bordo, medido em duas peças. ⇒ **o gate nº9 da espec não terá oráculo**
— a nossa extração tem de o resolver sem gabarito.

## O formato

Texto, uma directiva por linha, **vocabulário do domínio**:

```
malha <nV> <nF>
v <x> <y> <z>            # a superfície, em R³
f <a> <b> <c>            # triângulos, índices base 0
canto <face> <k> <u> <v> # a imagem do canto k (0..2) daquela face, no domínio
```

⚠️ **A imagem é POR CANTO, não por vértice** — é isso que permite a cada triângulo ter a sua
carta, e é de comparar as duas imagens de uma aresta partilhada que sai a função de transição
(espec §2.2).

## O verificador — ⭐ é o gate nº4 da espec, executável

```
cd /home/enio/Documentos/Projetos/PH2D && \
  gzip -dc docs/3D/cleanroom/fixtures/torus_64x32.mapa.gz > /tmp/t.mapa && \
  python3 docs/3D/cleanroom/fixtures/verifica_mapa.py /tmp/t.mapa
```

Ele deriva a transição de **cada** aresta interior e imprime o resíduo da rotação e o da
translação, com percentis **e a contagem ao lado**.

⭐ **Medido nestas duas peças:** resíduo de translação **máximo `3,55e-15`** e de rotação
**máximo `4,65e-14`** ⇒ são mapas de grade inteira **a ponto flutuante**.

⚠️ **Ele foi provado por DOIS controlos positivos**, não só por ficar verde:
- deslocar **uma** face por `(+0,37 · −0,21)` ⇒ o resíduo de translação passa a **`3,700e-01`** e ele reprova;
- rodar **uma** face `90°` ⇒ a distribuição passa de `{0: N}` a `{0: N−3, 3: 3}` e ele **continua** a aprovar (é transição legítima).
