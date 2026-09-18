---
name: feedback_the_test_language_cannot_tell_a_leak_from_a_declared_decision
description: O idioma de teste responde «esta palavra está no código?» e mostra IGUAL uma fuga de HR-15 e uma isenção já decidida com mecanismo — triar antes de migrar, senão desfaz-se trabalho pago.
metadata:
  type: feedback
---

Medido em 2026-09-17 (`line/UIUX`, 11.ª fatia do HR-15). O dono correu `PH2D_LANG=teste` e reportou
**sete** bolsos de inglês. Triados: **três eram fugas reais** (146 rótulos, curados) e **quatro eram
decisões já tomadas, cada uma com o mecanismo escrito no código** — os nomes de objecto na
Hierarquia (*«é conteúdo, não chrome: o artista renomeia»*), o painel Authored UI (*«sem `ph2d-i18n`,
e a ausência é a decisão»* — o texto é o que o artista autorou) e as duas BANCADAS (Widget Gallery e
Widget Lab, cujos rótulos são os nomes dos nossos próprios widgets).

**Why:** o instrumento mede uma propriedade **sintáctica** — *esta palavra está escrita no código?* —
e a resposta é **sim** nas sete. Ele não sabe distinguir *ninguém migrou isto* de *alguém decidiu
não migrar isto e escreveu porquê*. ⇒ migrar a lista inteira desfaria quatro decisões medidas; e
ignorá-la deixaria 146 rótulos crus. *Um instrumento que devolve a lista certa ainda pode induzir a
leitura errada.*

**How to apply:** antes de tocar num bolso que a foto acusa, **procure a isenção**: `NOT_LANGUAGE` /
`ISENTOS` no gate da crate, e o `Cargo.toml` dela (a ausência de `ph2d-i18n` costuma trazer a razão
por escrito). Se houver argumento, **reporte-o ao dono em vez de o desfazer** — ele decide (§0.8).
⭐ E as fugas reais têm uma FORMA que se reconhece: *um censo cuja crate não é DONA do texto que ela
pinta fica verde sobre texto cru* — as três desta fatia eram motores (`ph2d-sculpt3d`, `ph2d-tokens`)
pintados por painéis a **zero** literais, e nenhum deles está na lista de crates que a régua varre.
⚠️ E uma chave montada por `format!("{PREFIXO}nome")` é invisível aos DOIS censos (o lexical lê o
modelo como língua; o de chaves lê as que estão escritas) — escreva-a inteira e gateie o prefixo.
Irmãs: [[feedback_a_source_census_cannot_answer_whether_the_screen_reads_the_table]] ·
[[reference_topic_gate_discipline]] · [[reference_topic_measurement_discipline]]
