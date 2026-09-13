---
name: a-sweep-in-one-language-does-not-prove-filtering-in-another
description: O sweep da parede clean-room fechou VERDE sobre uma espec que traduzia prosa do alvo — a vassoura estava em inglês e a espec em português
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/sculpt3d`, obra dos pincéis que faltam): o
`scripts/cleanroom-sweep.sh` deu `✓ limpo, exit 0` sobre uma espec que o R-pré
reprovou com **quatro achados substanciais** — quatro frases eram **tradução
frásica** de prosa do alvo. A vassoura tinha 112 entradas, todas na língua do
alvo; a espec é escrita em português.

**Why:** a vassoura casa por SUBSTRING. Ela prova *«ninguém colou»* e nada mais —
e a forma de contaminação que o §4.2 da SKILL de fato proíbe (wording de
manual/comentário, pseudo-código espelhado) sobrevive a qualquer tradução. ⇒ um
sweep verde sobre um artefacto escrito noutra língua é um **negativo sem
população**: o instrumento não tinha como acusar. É a mesma espécie do censo que
varre zero e fica verde ([[the-orphan-and-the-double-declaration-are-one-audit]]),
um nível acima: aqui a população existe e o padrão é que não a alcança.

**How to apply:** toda vassoura cobre as línguas em que os artefatos são
ESCRITOS, não só a do alvo — as entradas de PROSA entram também traduzidas (em
base64, como as outras), e o ledger regista que ela passou a cobrir as duas. E a
leitura que fica: **o sweep nunca substituiu o R-pré** — ele apanha a colagem, e
quem apanha a tradução é um humano-agente independente a ler os dois lados
([[one-feature-one-line-minimum-windows]]). Quando os dois discordarem, o verde
do instrumento é o suspeito.
