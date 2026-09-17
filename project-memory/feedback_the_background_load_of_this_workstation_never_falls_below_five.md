---
name: feedback-the-background-load-of-this-workstation-never-falls-below-five
description: "A carga de FUNDO desta workstation é ~7 sem ninguém compilar, logo «esperar por load < 5» nunca chega — o instrumento que sobrevive é o MÍNIMO de N corridas"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-13T23:43:03.427Z
---

Medido em 2026-09-13 (W4 do plano `docs/Skeleton/03`): uma espera por `load < 4` esgotou **20 min** e
acabou por medir a **`8,01`** — e não havia `cargo` nem `rustc` nenhum a correr. A carga é o
**fundo** da máquina: o VS Code, o `rust-analyzer`, o `sccache` e as OUTRAS sessões do Claude.

⇒ a lei do `CLAUDE.md` §5.0 (*«nenhuma leitura de relógio vale nada acima de `load ~5`»*) é
verdadeira e, tomada como *«espere pela calma»*, é inexequível nesta máquina.

**Why:** uma MÉDIA sob contenção mede o vizinho, não o código — e foi por isso que a 1.ª tabela desta
wave (médias a `load 9,9` e `12,6`) teve de ser deitada fora.

**How to apply:** meça o **MÍNIMO de N corridas** (o custo quando o escalonador deu o núcleo) e
imprima a **mediana ao lado** — a distância entre as duas diz quanto a máquina estava a roubar. Com
`load 3,7`–`3,9` o mínimo e a mediana ficaram a `<1 %` uma da outra, o que é a assinatura de uma
leitura boa. Construa ANTES de esperar (`--no-run`), senão o próprio build é a carga
([[feedback-a-wait-for-calm-loop-fires-when-your-own-run-starts]]), e imprima o `/proc/loadavg` ao
lado de cada tabela.

## ⛔⛔ E o `loadavg` MENTE logo a seguir a uma corrida pesada (2026-09-16)

Ele é uma média de **um minuto a decair**: minutos depois de uma bateria acabar ele lia **`7,5`** com
a média de cinco minutos ainda em **`32`** e a CPU longe de ociosa. Esperei por `load < 9`, corri um
gate **3 vezes**, vi-o reprovar as três, e quase o dei como regressão do meu código — ele passou
**3 de 3** assim que a CPU chegou a `82`–`92 %` **ociosa**.

No mesmo instante, o mesmo relógio de dispositivo lia `41,2 ms` e passou a `10,5` — **`3,4×`** de
inflação sobre a leitura que eu ia escrever num doc.

**Why:** um gate de recurso segue a **contenção real**, e o `loadavg` é um filtro passa-baixo dela.
Três reprovas seguidas *parecem* prova de defeito e são só três amostras do mesmo crivo errado.

**How to apply:** espere pela **OCIOSIDADE da CPU** (`vmstat 1 3 | tail -1 | awk '{print $15}'`,
duas amostras `≥ 85 %`), não por um número de `loadavg`; e imprima a ociosidade ao lado de cada
corrida de confirmação. ⚠️ Antes de acusar o seu diff, **procure a grandeza que o gate já imprime
sobre si mesmo** — o `com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` imprime a
ociosidade e tinha a assinatura dele registada num doc (`docs/Render3d/05` §43.9: *«`2 de 18` com a
CPU a `0 %` ociosa, `11 de 18` com ela livre»*) desde antes da wave que eu estava a culpar.
