---
name: a-promise-that-justifies-a-decision-must-have-a-reader
description: "Quando a justificação de uma escolha estrita é «e o utilizador é avisado», vá VER quem lê o aviso — sem leitor, paga-se o preço da regra e não se entrega o benefício"
metadata:
  type: feedback
---

**Medido por auditoria na fonte de dados do Motion, 2026-08-30.**

O leitor de tabelas diverge do *Import CSV* do Blender de propósito: ele infere o tipo da
coluna **do primeiro valor**, e nós recusamos a coluna inteira se **qualquer** célula não
converter. A divergência está certa — com a regra dele, uma coluna cujo 1.º valor calha ser
`1990` e o resto é texto entra como uma coluna de **zeros**, sem aviso.

E eu justifiquei-a assim, no doc-comment: *«a nossa salta-a e **NOMEIA-A**»*, com o campo
`Table::skipped` ao lado e a nota *«⚠️ Nunca em silêncio: o painel mostra isto.»*

⛔ **A auditoria foi ver quem lia o `skipped`: ninguém.** Nem painel, nem toast, nem log —
zero ocorrências fora da própria crate. ⇒ o preço da regra estrita era pago inteiro (999
números + uma célula má = coluna inteira apagada) e o benefício **não existia**.

**Why:** uma regra mais estrita que a referência só é melhor *se o utilizador souber por que
não vê o dado*. Sem o canal, ela é estritamente pior: a referência dá um número errado, a
nossa não dá nada — e as duas são igualmente silenciosas.

**How to apply:** sempre que escrever *«…e o artista é avisado»*, *«…e isso é reportado»*,
*«…e o painel mostra»* como a razão de uma escolha, **grepe o campo nesse instante**. Se
nenhum consumidor o lê, ou constrói o consumidor no mesmo commit, ou a justificação cai e a
escolha tem de se defender sozinha. Relacionado:
[[feedback_a_dead_knob_has_two_species_no_probe_catches]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] ·
[[feedback_a_tool_is_adopted_only_when_a_written_step_names_it]].


## 2.ª ocorrência, no mesmo dia — a promessa que um REVERT deixou para trás

**L-System, 2026-08-30.** Escrevi `turtle::draws` (a pergunta estreita: *o que é um osso?*)
e o doc dela dizia, para justificar a existência: *«mais estreito que o `draws_or_marks`, e a
diferença tem consumidor: a família de crescimento pergunta se sobrou algum módulo VELHO que
desenha»*. Depois **revertí** a mudança em `grows_by_refining` (para desfazer outra coisa) e
**o doc ficou**. ⇒ durante um bloco inteiro havia uma função com um doc a nomear um leitor que
ela não tinha, e o defeito que ela dizia curar continuava vivo para qualquer gramática que o
artista escrevesse.

⭐ **O que o revelou não foi ler o doc — foi o `git diff --stat`:** o ficheiro do leitor
alegado **não estava na lista dos modificados**. E ao aplicar a cura a sério, o clippy
respondeu a pergunta seguinte de graça: `draws_or_marks` ficou **sem nenhum leitor** ⇒ era
lixo, e a cura de um órfão é apagá-lo.

**How to apply:** um revert não é local — ele pode deixar órfã a JUSTIFICAÇÃO que outro
ficheiro já escreveu. Ao reverter, grepe o símbolo revertido nos docs; e antes de commitar,
leia o `--stat` a perguntar *cada afirmação deste diff tem o ficheiro dela aqui?*
