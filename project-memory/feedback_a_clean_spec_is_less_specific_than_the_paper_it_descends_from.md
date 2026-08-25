---
name: feedback_a_clean_spec_is_less_specific_than_the_paper_it_descends_from
description: Para provar que uma espec clean-room descende da literatura e não de código lido, meça a ESPECIFICIDADE dela contra a fonte — quem traduz código herda as constantes, quem descreve herda a lei
metadata:
  type: feedback
---

Auditando (papel R-pré) a espec da extração de malha quad em 2026-08-24, a pergunta em
aberto era se a secção §5 descendia do *paper* (lícito) ou da implementação alheia que a
janela E admitia ter visto (= a rota de porte que o ADR-0164 rejeitara). **A prova decisiva
não foi de conteúdo, foi de especificidade:** o *paper* publica a tolerância concreta do
laço; **a espec não a copia** — fala em «tolerância» e «tecto» e manda MEDIR a fracção que
fica no primeiro degrau.

**Why:** uma tradução de código **herda as constantes** (elas estão lá, e copiá-las é o
caminho de menor esforço); uma descrição derivada da literatura **herda a lei** e deixa os
números para quem medir. ⇒ *ser MENOS específica que a fonte citada é evidência positiva de
descrição; ser MAIS específica é a bandeira vermelha.* O mesmo sinal apanha o contrário: um
detalhe que **nem o paper nem a espec** dão e que «veio» é o tripwire de recall.

**How to apply:**
1. ⛔ **Não audite a espec contra a sua memória do algoritmo** — extraia o texto da fonte
   citada (`pdftotext -layout`) e faça `grep`. É o que torna o veredito refutável, e foi o
   que transformou «suspeita» em «§4.2 verde» em minutos: o glossário que parecia nome
   interno ocorre 40/42/13/14/55 vezes no *paper*, e o procedimento detalhado é o
   `Algorithm 1` **publicado**.
2. Depois compare **grão a grão**: constantes, tolerâncias, tamanhos de tabela, nomes de
   campo. Cada item em que a espec é mais vaga que a fonte é um voto a favor dela.
3. ⚠️ Vale para além do clean-room — é a régua de *«esta nota veio de medir ou de copiar?»*.

Irmãs: [[feedback_the_oracle_writes_its_intermediate_stages_compare_phase_by_phase]] ·
[[reference_topic_oracle_discipline]] · [[feedback_stale_comment_and_dead_code_lie]]
