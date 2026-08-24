═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL R — REVISOR                (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Modo: PRÉ · Módulo: 3D (quad remesh) · Alvo: as RESTRIÇÕES LINEARES
Espec a auditar: docs/3D/cleanroom/SPEC_restricoes_por_eliminacao.md
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md

Você é o REVISOR em modo PRÉ. Audita a espec contra o §4.2 — pseudo-
código espelhado, wording de manual, nomes internos, tabela verbatim —
e atesta no cabeçalho dela ("auditada contra §4.2 por R em <data>").
⛔ SEM esse atestado, a janela I não abre.

Leia: SKILL_Cleanroom §3.R + §4.2 + §7.1.

⛔⛔ ESTA AUDITORIA NÃO É A DE ROTINA — leia o porquê antes de começar
═══════════════════════════════════════════════════════════════════

O E desta espec é a janela `49c94a84-e903-48a9-bd7f-b14685d71061`, que
foi o R-pós da obra anterior — e que por isso LEU, em 2026-08-24:

  · o laço de arredondamento de uma implementação de referência
    (MPL-2.0), incluindo a MONTAGEM DE RESTRIÇÕES dela;
  · o esboço (nomes de função e comentários, zero corpo) de um módulo
    de extracção da mesma biblioteca.

Registado em LEDGER §R-pós.3. ⇒ **O risco desta espec não é o risco de
sempre.** O de sempre é o E filtrar mal a travessia; aqui é
convergência de EXPRESSÃO a entrar pelo próprio E, num ponto em que
ele viu a resposta.

⭐ A espec foi escrita com uma contra-medida explícita: ela diz o que
tem de ser VERDADE e qual é a LEI publicada, e recusa-se a dizer por
que estrutura de dados, por que decomposição e em que ORDEM se monta o
sistema (§2.3, último parágrafo). ⚠️ Isso é uma AFIRMAÇÃO do E sobre o
próprio trabalho — e é exactamente o que você existe para não aceitar
de graça.

O QUE AUDITAR, POR ORDEM DE RISCO
═══════════════════════════════════════════════════════════════════

1. ⛔⛔ **§1 e §2.3 — a lei da eliminação.** É aqui que a convergência
   entraria. Confira contra o *paper* de 2009 (`~/Referencias/papers/
   miq2009.txt`, §2.1 final) que a lei é PUBLICADA, e confira contra a
   espec que ela é dita como REQUISITO e não como receita. ⚠️ Sinal de
   alarme: qualquer menção a estrutura de matriz, ordem de eliminação,
   decomposição, factorização ou permutação. Nada disso deve estar lá.

2. ⛔ **§3.1 — a detecção de feição.** A lei vem do §3 do mesmo *paper*
   (regiões parabólicas, anisotropia relativa das curvaturas
   principais, faixa de raios, escolha da leitura estável). Confira:
   (a) que está RE-DESCRITA e não transcrita; (b) ⭐ que os QUATRO
   coeficientes concretos do *paper* NÃO foram copiados — a espec manda
   medi-los, de propósito, e isso é a prova de que ela descende do
   *paper* e não de uma implementação.

3. **§0 e §2.1 — as tabelas.** Toda tabela tem de ser medição NOSSA.
   Confira a proveniência de cada uma (o `chain_info`, a sonda
   `the_rounding_ladder_sweeps_its_two_constants`, o doc do
   `SEAM_WEIGHT`) — nenhuma pode vir do alvo.

4. **§5 — os gates.** Cada barra tem de ser derivada e dizer de onde
   vem. ⚠️ O gate nº1 é o coração da obra; confira que ele exige ZERO e
   não «pequeno», e que pede prova de mutação.

5. **Sweep (§7.1)** sobre a espec e sobre este handoff, com CONTROLE
   POSITIVO antes de acreditar num verde. ⚠️ O script vive só no
   primário: invoque-o pelo caminho ABSOLUTO.
   (Já corrido pelo E sobre a espec: ✓ limpo, exit 0.)

6. **Produza o `NEXT_I_eliminacao.md`** — o bloco colável da janela I,
   com o Passo 0 já corrigido: ⚠️⚠️ o `deny` do BLOCO-I tem de cobrir
   **Bash**, e não só `Read`. Medido: a janela I anterior fez 177
   chamadas `Bash` e ZERO `Read` — a parede dela aguentou por
   disciplina, não por mecanismo (LEDGER, "Papel I", achado de molde).

O QUE O I VAI PRECISAR, E QUE JÁ EXISTE
═══════════════════════════════════════════════════════════════════
· A cadeia inteira, com o instrumento ponta-a-ponta:
    cargo run --release -p ph2d-quadextract --example chain_info -- \
      /home/enio/Documentos/Projetos/ph2d-quadbench/corpus/sculpt_wrinkled.obj
  ⭐ Ele aceita um `.obj` do corpus desde 2026-08-24, e imprime as duas
  colunas que esta obra move (resíduo da costura · forma por-face).
· Os mapas de referência verificados em `fixtures/`.
· A base do fork: `line/quadextract` (que descende de `line/sculpt3d`),
  ⛔ NÃO `main` — a pasta `cleanroom/` não existe no `main`.

⚠️ E DUAS COISAS QUE NÃO SÃO DESTA OBRA MAS PASSAM MUDAS
═══════════════════════════════════════════════════════════════════
· A colisão do número de ADR `0164`, escrita por duas linhas com
  títulos diferentes (LEDGER §R-pós.7.3).
· `scripts/cleanroom-sweep.sh` continua NÃO RASTREADO — a prova deste
  ledger não é reproduzível noutra máquina (LEDGER §R-pós.7.1).
═══════════════════════════════════════════════════════════════════
