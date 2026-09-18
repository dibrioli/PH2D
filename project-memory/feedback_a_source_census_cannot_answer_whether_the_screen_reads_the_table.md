---
name: feedback_a_source_census_cannot_answer_whether_the_screen_reads_the_table
description: Trinta censos LEXICAIS não podem provar que uma palavra do ecrã veio da tabela — um rótulo esquecido no pintor pinta-se igual ao que veio dela; a prova é um IDIOMA DE TESTE.
metadata:
  type: feedback
---

Medido em 2026-09-17 (`line/UIUX`, 10.ª fatia do HR-15). Nove fatias moveram **4 986** chaves e
**3 692** sítios para a tabela sob **30** censos. Os trinta abrem o FONTE de uma crate e perguntam
*«há aqui um literal com cara de língua?»* — e a pergunta do HR-15 é *«a palavra que o artista LÊ
saiu da tabela?»*. **Elas divergem no caso que importa:** um rótulo esquecido no pintor pinta-se
**exactamente igual** ao que veio da tabela.

⇒ `PH2D_LANG=teste`, um idioma derivado do inglês por deformação: o que ficar em inglês normal no
ecrã está, por construção, no código. **A primeira fotografia achou `default-scene`** — e a régua
lexical **não o podia ver**: a `is_language` recusa um token nu que não seja Capitalizado nem
GRITADO (para não acusar identificadores), e `default-scene`/`sprites`/`bodies` leem `false` nela.

**Why:** as duas réguas têm pontos cegos **complementares**, e só a segunda olha para o ecrã.
⛔ E dimensionar o ponto cego por uma varredura lexical reproduz o defeito um nível acima: a minha
sonda por «palavra minúscula» devolveu **1 592** achados que eram mensagens de `assert` e nomes de
campo — *uma heurística lexical não separa um RÓTULO minúsculo de um IDENTIFICADOR minúsculo, que é
precisamente a razão de a régua os recusar em bloco*.

**How to apply:** construa o idioma de teste **antes** de declarar o HR-15 fechado, e ancore-o em
três leis (marcador verbatim · chave desconhecida CRUA, senão `tr(k) != k` parte em meio repo ·
inglês byte-idêntico como CONTROLO). ⚠️ **O alongamento não é uma constante** — medido sobre 89 832
pares de 4 editores instalados, um rótulo de ≤6 caracteres DOBRA no p90 e um parágrafo cresce 1,37
([[feedback_a_bar_calibrated_without_the_approved_side_measures_our_own_defects]]).
⛔ **Tire a foto de CONTROLO**: a minha 1.ª leitura culpou a tensão por uma elisão que o ecrã em
inglês já tinha em treze sítios. ⛔ E ele **não** acorda um gate que compara duas chaves com a
mesma palavra — sendo derivado do inglês, as duas deformam-se igual; isso só uma língua AUTORADA
resolve. Irmãs: [[reference_topic_measurement_discipline]] · [[reference_topic_gate_discipline]] ·
[[feedback_a_key_census_that_sees_only_literals_prescribes_deleting_live_labels]]
