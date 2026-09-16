---
name: measuring-only-smoothness-approves-a-global-blur-as-a-rig
description: Uma família de réguas que uma resposta DEGENERADA ganha por construção aprova a degenerada — no skinning, medir só suavidade shipou um borrão global a chamar-se rig
metadata:
  type: feedback
---

⛔⛔⛔ **Medido 2026-09-15, `line/Vector`, pele de imagem.** Três réguas julgavam a lei de pesos do
esqueleto — **faceta** (quanto o afim de cada triângulo erra o campo), **esticão** (quanto uma
aresta estica) e **círculo** (quanto uma circunferência desenhada sai fora de redondo). As três
medem **SUAVIDADE**, e por elas a melhor lei do corpus era, de longe, um *bump* euclidiano com o
alcance de cada osso subido a `2,0` — `0,40 px` de faceta contra `6,78` da mesma lei no alcance de
fábrica. Foi essa a que shipou, e o dono aprovou-a no smoke.

⭐ **Ela não era um rig: era um borrão.** Um alcance de `2,0` cobre a arte inteira, logo **todo osso
pesa em todo ponto** — e isso é suavíssimo pela mesma razão que o desfoque é suave. A régua que
faltava era a **LOCALIDADE**: *rodar só a PONTA da corrente e medir quanto a arte da RAIZ se mexe.*

| lei | faceta | esticão | círculo | **VAZAMENTO** |
|---|---:|---:|---:|---:|
| euclidiana local | `6,78 px` | `2,234` | `1,2923` | `0,00 px` |
| euclidiana `strength = 2,0` (a que shipou) | **`0,40 px`** | **`1,267`** | **`1,1491`** | ⛔ **`26,51 px`** |
| padrão-ouro (BBW) | `1,27 px` | `1,492` | `1,2913` | ⭐ `0,78 px` |

**A forma geral, que é o que vale guardar:** quando **toda** a família de réguas de um subsistema
pode ser ganha por uma resposta **degenerada** (borrar tudo · devolver constante · não fazer nada),
a família não é um juízo — ela é um convite. ⇒ *antes de escolher entre N leis, pergunte qual
resposta trivial ganha nas réguas que tem, e construa a que a reprova.*

⚠️ **E o sintoma é invisível no smoke**: o que está no ecrã parado é suave, e o defeito só aparece
quando alguém mexe **numa** ponta. Um dono aprova a foto; a régua tem de apanhar o resto.

⚠️ **A régua nova também mentiu à primeira** e por uma razão que se repete: ela escolhia a «ponta»
pelo maior `to_bits` em vez de perguntar ao produto (`chain_ends`), rodava a **raiz**, e leu
`154 px` de vazamento **sobre a lei que não pode vazar**. *O controlo que desmascara uma régua nova
é o caso em que ela tem de ler zero.*

⭐⭐⭐ **E a MESMA jornada apanhou a família DUAS vezes mais, o que é o que a torna uma lei e não um
caso.** A régua seguinte — *quanta arte vira do avesso ao dobrar* — é ganha pela **RIGIDEZ**: arte
presa a UM osso só não pode inverter-se porque **não se deforma**. Medido, a lei que ganhava aquela
coluna deixava **`37,6 %` da arte rigidamente pregada a um osso** contra `8,1 %` do padrão-ouro.
⇒ *toda régua de «quanto isto se estraga» tem de vir com o censo de «quanto isto sequer se mexe»*,
senão ela premeia não fazer nada — e a resposta degenerada muda de nome (borrar · enrijecer) sem
mudar de natureza.

⛔⛔ **E uma quarta, que é de outra espécie e vale por si: eu amostrei a CONDIÇÃO DE FRONTEIRA e
chamei-lhe solução.** O perfil dos pesos foi lido ao longo da linha onde eles estão **presos**
(`w = 1` por construção), e eu quase registei *«o padrão-ouro é uma função escada»* sobre um campo
que fora do eixo lê `0,98 → 0,92 → 0,76 → 0,59 → 0,38 → 0,12`. *Numa solução com restrições, a régua
tem de amostrar onde o problema é LIVRE — no resto ela devolve o que eu escrevi.*

Ver [[a-bar-calibrated-without-the-approved-side-measures-our-own-defects]] (a barra que mede os
nossos próprios defeitos) e [[feature-worse-than-not-existing]] (a linha de controlo «não fazer
nada»). O caso inteiro: `docs/Skeleton/handoffs/HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15.md` §17.
