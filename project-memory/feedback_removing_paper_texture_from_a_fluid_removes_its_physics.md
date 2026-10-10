---
name: feedback_removing_paper_texture_from_a_fluid_removes_its_physics
description: Tirar o papel de um motor de aquarela tira-lhe a física (o limiar por célula é o que faz o molhado sangrar) — para «sem desenho de papel» baralhe as alturas por célula, não as alise
metadata:
  type: feedback
---

O dono (10/10): «com Paper: None o papel ainda exerce efeito». Medidas quatro leis sem Kind no Wet Paint:
plano a 1 (tinta +65 %), plano à média 0,5 (o Solid ganha buracos — o limiar come os toques fracos), plano +
lei ESPERADA do depósito e da humidade (tinta certa, mas o molhado deixou de sangrar: −5 contra +12…+21 do
Rebelle; com o relevo do Cold só na água, igual — não era a capilaridade), e as alturas do Cold AO ACASO por
célula: grão fino sem desenho, orla a meia 1,45 (Rebelle 1,6), tinta −1 %, sangra +13. O dono escolheu esta.
Prancha `docs/Painter/NewFeaturesWetPaint/prancha_w1g_sem_papel.png`; tabela no 05 §4b «W1g».

**Why:** num motor portado com limiar por célula (`deposit = stamp − (1 − tooth)·gate`), a heterogeneidade
do papel É a estrutura que a água segue; uma constante (até a «esperança» exacta) muda a dinâmica.
**How to apply:** para tirar o DESENHO de um campo que alimenta uma física de limiar, mantenha a distribuição
(os quantis) e destrua só a correlação espacial; meça a física (sangria, orla) antes do aspeto.
