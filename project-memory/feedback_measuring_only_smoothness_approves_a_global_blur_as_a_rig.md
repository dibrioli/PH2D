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

Ver [[a-bar-calibrated-without-the-approved-side-measures-our-own-defects]] (a barra que mede os
nossos próprios defeitos) e [[feature-worse-than-not-existing]] (a linha de controlo «não fazer
nada»). O caso inteiro: `docs/Skeleton/handoffs/HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15.md` §17.
