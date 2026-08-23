---
name: feedback_a_perfect_correlation_across_the_whole_corpus_is_still_n_samples
description: Uma correlação perfeita sobre TODAS as fixturas continua a ser N amostras — não construa a guarda sobre ela sem derivar o mecanismo, e quando o mecanismo não existe pergunte à FASE SEGUINTE
metadata:
  type: feedback
---

Uma tabela em que o predicado acerta em **todas** as fixturas parece prova. Não é: se o
corpus tem quatro peças, é **uma correlação sobre quatro amostras**, e «100 % das
amostras» soa a certeza porque o denominador desapareceu. ⛔ **Derive o mecanismo antes
de construir a guarda sobre ele** — e se não conseguir, use a guarda cara que não
precisa de mecanismo nenhum.

**Why:** medido no quad remesh (2026-08-23). Uma poda de arcos deixava a orelha
`Infeasible` no F4. A tabela era limpa:

| fixtura podada | `Σ lados` | o F4 |
|---|---|---|
| esfera lisa | 20 (par) | resolve |
| enrugada | 18 (par) | resolve |
| gancho | 40 (par) | resolve |
| ⛔ orelha | **19 (ímpar)** | ⛔ `Infeasible` |

Deduzi: *«cada lado é partilhado por dois patches, logo `Σ` é par; ímpar ⇒ algum arco
tem o mesmo patch dos dois lados»*, construí a guarda da auto-adjacência e escrevi a
tabela no doc como se fosse mecanismo. ⛔ **A orelha continuou `Infeasible`, ainda com
`Σ = 19`.** O argumento era falso: um *lado* é um agrupamento **por-patch** de arcos, e
os dois vizinhos podem agrupar a mesma fronteira em números de lados diferentes — a soma
não tem de ser par. ⇒ *a correlação perfeita era coincidência de quatro amostras, e a
premissa «cada lado é partilhado por dois» não descrevia esta estrutura.*

⭐ **A guarda que funcionou não tem predicado nenhum: correr a FASE SEGUINTE sobre o
resultado de teste e aceitar só se ela aceitar.** Cara — uma resolução por candidato —
e certa. *Nenhum predicado local o soube prever, e essa é a resposta, não um obstáculo.*

**How to apply:**
1. Antes de escrever a guarda, escreva o **mecanismo** numa frase e pergunte de que
   propriedade estrutural ele depende. Se a frase usa «logo» sobre uma definição que não
   verificou (*«um lado é…»*), verifique a definição primeiro.
2. Conte as amostras em voz alta. «Acerta em todas» com `N = 4` é `N = 4`.
3. ⭐ Quando o efeito que quer evitar é *«a fase seguinte recusa»*, a guarda **é** a fase
   seguinte. Não invente um proxy barato para ela — o proxy é o que falha em silêncio.
4. Se já escreveu a tabela num doc como mecanismo, **corrija o doc**, não só o código: a
   tabela sobrevive ao código e é lida como estabelecida.

Irmãs: [[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_n_sources_need_the_cross_check_not_n_self_checks]] ·
[[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]] ·
[[feedback_two_good_hypotheses_failing_refutes_the_family_not_the_two]]
