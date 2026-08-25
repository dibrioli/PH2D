---
description: Clean-room de código restrito — papel E (espec), I (implementa), R (revisa PRÉ/PÓS) ou SOLO (uma janela orquestra tudo).
argument-hint: [Papel: E|I|R-PRÉ|R-PÓS|SOLO] [Alvo + licença + fonte local] [Módulo PH2D]
---
Você vai operar o protocolo clean-room no papel `$1`.
Alvo: $2
Módulo PH2D: $3

Leia INTEIRA: docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md
Depois execute o BLOCO do seu papel (§10) com os dados acima.

Regras que não esperam a leitura:
- Papel E: TRIAGEM primeiro (§2) — leia a licença REAL e cace irmão permissivo
  (T0/T1) antes de aceitar o pipeline; checkpoint de PATENTE incondicional
  (§8.1); ledger ANTES da primeira leitura do fonte; TRAVESSIA INTEGRAL do
  fonte (cobertura no ledger) e mineração das dicas dos autores ANTES da
  espec (§3.E + §4.1.11-13); TUDO do alvo (fonte, builds, rascunhos) em
  ~/Referencias/<alvo>/, nunca no repo/tmp/scratchpad; todo artefato
  destinado ao Implementador passa `bash scripts/cleanroom-sweep.sh` antes
  do commit.
- Papel I: você só é o Implementador se ESTA janela nunca conteve o fonte do
  alvo. Se já conteve (mesmo de relance), diga-o AGORA. Passo 0 mecânico do
  BLOCO-I (deny config + conferir o CABEÇALHO da espec — você nunca abre o
  ledger). Suas fontes: espec, papers pelo mapa de leitura, código do PH2D,
  dumps prontos do oráculo. Nada mais.
- Papel R: modo PRÉ audita a espec contra §4.2 ANTES de I abrir (janela que
  não seja a E); modo PÓS roda paridade + sweep total + revisão estrutural e
  fecha o ledger. R vê os dois lados e não escreve código de produto.
- Papel SOLO: você orquestra E e R como SUBAGENTES (contexto isolado, contrato
  de retorno sem expressão do alvo) e implementa VOCÊ MESMA — logo as regras
  do BLOCO-I valem para a sua janela desde a primeira mensagem: nunca abra o
  fonte. Alvo pequeno/médio; obra de dias prefere janelas separadas (§3).

CORRENTE DE HANDOFFS (§10, vale para E, R-PRÉ e I): ao terminar o seu papel,
escreva o bloco do PRÓXIMO papel já preenchido, salve em
docs/<Módulo>/cleanroom/NEXT_<papel>.md e imprima-o INTEIRO no fim da sua
resposta — o Enio só abre a janela nova e cola. O NEXT_I.md passa o
cleanroom-sweep antes de salvo. Um handoff não acrescenta nada além dos
campos do molde.

A parede em uma frase: quem escreve o código do produto nunca teve a expressão
original no contexto. Todo o resto é maximalista; essa linha não se cruza.
