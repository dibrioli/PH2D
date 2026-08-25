---
description: Clean-room de código restrito — UMA linha, UMA janela; E/R são subagentes; RETOMADA assume a MESMA linha.
argument-hint: [Modo: LINHA|RETOMADA] [Alvo + licença + fonte local] [Módulo PH2D]
---
Você vai operar o protocolo clean-room no modo `$1` (vazio = LINHA).
Alvo: $2
Módulo PH2D: $3

Leia INTEIRA: docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md
Depois execute o bloco do seu modo (§10) com os dados acima:

- Modo LINHA (o padrão — BLOCO-LINHA): você é a ÚNICA janela desta feature.
  Abre line/<módulo> (MODELO_ABERTURA_LINHA, sem parar no "aguardo a tarefa"),
  opera sob as regras do Implementador DESDE JÁ (§3.I — você NUNCA abre o
  fonte do alvo), despacha os papéis E, R-PRÉ e R-PÓS como SUBAGENTES
  (missões do §10, com o CONTRATO DE RETORNO verbatim) e implementa VOCÊ
  MESMA. Triagem devolveu T0/T0½ (porta permissiva)? O clean-room acabou:
  porte fiel, você mesma, com atribuição. Patente viva? PARE e reporte ao
  Enio.
- Modo RETOMADA (BLOCO-RETOMADA): a linha JÁ existe e a janela anterior
  PAROU. FASE 0 do MODELO_TROCA_DE_AGENTE_NA_LINHA (cd/pwd/branch — você
  começa na árvore ERRADA), passo 0 do I, confira o CABEÇALHO da espec
  (você nunca abre o ledger) e continue do passo em que a anterior parou.
  Motivo = incidente §6? O código pós-exposição está em QUARENTENA.

Regras que não esperam a leitura:
- UMA feature = UMA linha; UMA janela por vez. Contexto enchendo, incidente
  §6 ou fim de jornada → preencha o BLOCO-RETOMADA, rode o sweep sobre ele,
  salve em docs/<Módulo>/cleanroom/RETOMADA_<alvo>.md, imprima-o INTEIRO e
  PARE. Nunca abra uma segunda linha nem deixe duas janelas na mesma.
- A parede: quem escreve o código do produto nunca teve a expressão original
  no contexto. Os subagentes E/R leem o fonte; você nunca. Report de
  subagente com expressão do alvo = incidente §6 (sua janela queima como I;
  a retomada continua da espec — tudo durável já vive em disco).
- Todo artefato que cruza a parede passa `bash scripts/cleanroom-sweep.sh`
  antes do commit/entrega — o report do subagente-E incluído.
- Fechamento: gate batched + handoff normal da casa (sem mecanismo interno
  do alvo — só nome + link p/ cleanroom/); NÃO integra, NÃO pusha (§0.7).
