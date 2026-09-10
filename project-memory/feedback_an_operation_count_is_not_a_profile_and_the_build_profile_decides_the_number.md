---
name: feedback-an-operation-count-is-not-a-profile-and-the-build-profile-decides-the-number
description: Contar chamadas nao diz onde o relogio esta, e uma cura medida no perfil de build errado le-se 5x menor — o tecto pode ser o opt-level, nao o algoritmo
metadata:
  type: feedback
---

⛔⛔⛔ **Duas armadilhas que se apanham juntas: contar OPERAÇÕES não é um perfil, e o NÚMERO que uma
cura mede depende do perfil de build em que se mede.**

Medido 2026-09-10 (`line/3DModeling`, W147), a caçar o tecto do censo do modelador SDF:

1. Contei as avaliações do `worst_gradient`: o **teste de banda** era `79 %` delas (`474 552` de
   ~`600 000`). Certo — e o perfil confirmou-o depois (`92,3 %` do tempo).
2. Construí a varredura em **lote**. Isolada: **`2,4×`–`2,8×`**. Ligada ao censo: **`5 %`**.
3. Hipótese: *«uma vizinha 32-way paralela está a esfomear-lhe a banda de memória»*. **REFUTADA** —
   sozinho o teste dá a mesma razão (`44,3 → 41,9` = `5,4 %`; acompanhado `72,0 → 68,6` = `4,8 %`).
4. ⭐⭐⭐ **A sonda de perfil deu a resposta: o mesmo trabalho custa `3,85 s` em `--release` e `44 s`
   em `dev`.** A crate não constava da lista `[profile.dev.package.*] opt-level = 2` do `Cargo.toml`
   da raiz — lista que **já existia**, criada para o Painter e o DSP de áudio, com a justificação
   escrita ao lado: *«at opt-0 they are 15-25x slower»*.

Resultado de quatro linhas de `Cargo.toml`: teste `44,3 s → 5,7 s`; suíte de 3 crates
**`372,3 s → 57,1 s`** (`6,5×`); o teste mais longo `307,2 s → 43,4 s`.

⭐⭐ **E aí a cura do passo 2 passou a valer `14 %`** (`5,7 → 4,9 s`) em vez de `5 %`: a `opt-0` a
descodificação que o lote amortiza está afogada em código não-optimizado. ⛔ *Se eu tivesse decidido
pelo `5 %`, tinha deitado fora uma cura boa **e** deixado o tecto de pé.*

**Why:** o sinal que denuncia tudo isto é **o modelo discordar do relógio**. O meu modelo de custo,
com os ns/ponto já medidos, previa `0,55 s` para as 58 formas; o teste levava `44 s`. *Um desacordo
de `80×` entre o modelo e o relógio É o achado* — eu li-o como «o modelo é grosseiro» e fui optimizar
na mesma.

**How to apply:**
- Antes de optimizar, **meça o TOTAL** e confronte-o com o que o modelo prevê. Se discordarem por mais
  de ~2×, **pare e perfile** — a diferença é onde o trabalho de facto está.
- Ao medir uma cura, **diga em que perfil a mediu**, e meça-a nos dois se o consumidor corre em `dev`
  (os testes correm). Uma cura pode ser rejeitada por ser medida no perfil errado.
- ⭐ **Antes de atacar um teste lento neste repo, veja se a crate está na lista de
  `[profile.dev.package.*]` com `opt-level = 2`** — a lista existe, tem precedente escrito, e uma
  crate de aritmética por-ponto que não esteja lá paga `10×`–`25×`.

Ver [[reference-topic-measurement-discipline]] ·
[[feedback-a-grid-never-lands-on-a-measure-zero-set-so-it-reports-it-clean]] ·
[[feedback-a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another]]
