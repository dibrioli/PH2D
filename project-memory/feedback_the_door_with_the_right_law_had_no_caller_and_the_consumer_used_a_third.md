---
name: feedback-the-door-with-the-right-law-had-no-caller-and-the-consumer-used-a-third
description: "Duas portas com a lei certa e ZERO chamadores de produto, enquanto o consumidor real mapeava por uma terceira — uma porta sem chamador é indistinguível de uma lei ausente"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-14T18:56:27.003Z
---

Medido em 2026-09-14 (W10 do plano `docs/Skeleton/03`). A fila do módulo dizia *«a UV do pintor FORA
da malha responde pela lei do quad de repouso»* — um limite pequeno e nomeado. O item estava **mal
endereçado**, e a medição mudou o tamanho do defeito:

* `sprite_world_to_uv` e `sprite_world_to_uv_unclamped` conheciam a malha desde a wave anterior e
  tinham **ZERO chamadores de produto** (só testes e o `pub use`);
* o Painter mapeia o ponteiro por **outra** porta — o afim do quad de repouso —, que não sabe o que é
  uma malha;
* ⇒ numa arte deformada a pincelada caía deslocada **em toda a arte**, não só fora dela.

**Why:** ao curar um subsistema, escrevem-se as portas certas e gateiam-se — e a wave dá-se por
fechada porque *a lei existe*. Mas **nenhuma sonda deste repo pergunta se uma porta tem chamador**
(já está escrito no §5 para o `dock_columns::close`, órfão depois de um gesto sair). Uma porta sem
consumidor e uma lei que não existe produzem exactamente o mesmo app.

⚠️ E o item da fila descrevia o resíduo (*«fora da malha»*) porque quem o escreveu olhou para a porta
que tinha acabado de curar — não para quem o produto de facto chama.

**How to apply:** quando uma wave publica uma porta que substitui uma lei, o censo de fecho é
`grep` pelo **nome dela fora dos testes**. Zero chamadores ⇒ a wave não fechou: ou falta ligar o
consumidor, ou ele usa outra porta — e é essa que tem de ser encontrada e nomeada. E ao herdar um
item de lista aberta, meça **quem chama**, não o que a porta faz. Ver
[[feedback-a-wrong-id-in-a-list-is-not-a-visible-defect-until-its-reader-is-read]] e
[[feedback-a-new-gesture-inherits-the-enemies-of-the-old-one]].
