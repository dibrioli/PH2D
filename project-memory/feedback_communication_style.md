---
name: Communication style — perguntas e formato
description: Como apresentar opções e respostas para o Enio em decisões pontuais.
type: feedback
originSessionId: 3810fc76-ee39-499c-932e-822ab7813c1b
modified: 2026-09-25T01:28:45.622Z
---
Ao pedir decisão ao Enio, apresentar **2-3 opções concretas** com trade-offs explícitos, **recomendação primeiro** (com sufixo "(Recomendado)"), pedir sim/não — não open-ended "o que você acha?".

**Why:** HANDOFF.md L207-210: "Enio aprecia decisão pronta apresentada. Não aprecia 'consulte vibrational alignment com sua visão'." Validado em 2026-05-08 nas perguntas de bootstrap (modo de instalação, sobreposição de git, target rustup) — ele escolheu a opção recomendada nas 3 vezes ou validou alternativa específica.

**How to apply:** usar `AskUserQuestion` com 2-4 opções; primeira opção é a recomendada com label terminando em "(Recomendado)" quando for de fato a melhor escolha técnica; descrição concisa do trade-off em cada opção. Evitar perguntas vagas. Para decisões dentro de Hard Rules + tiebreakers do SKILL, decidir solo sem perguntar.

**Formato de resposta** (não pergunta): pt-BR direto sem floreio; código em blocos com linguagem.

⛔ **A LÍNGUA é pt-BR SEMPRE ao falar com ele** — reforçado pelo Enio em 2026-09-24 (*«Ao falar
comigo, fale em PT-BR»*) depois de uma resposta minha sair em inglês. Vale também para os passos
do smoke; o que fica em inglês são só os rótulos que aparecem na tela.

⛔⛔ **A CONVERSA é SEMPRE em pt-BR — o que é em inglês é só a UI do APP.** Em 2026-09-24 (line/3DModeling,
depois de uma compactação) respondi ao dono em inglês várias mensagens seguidas, e ele corrigiu: *«próxima
vez fale comigo em PT-BR»*. A causa provável foi misturar a regra *«nada de PT-BR no app»* com a língua da
conversa. ⇒ texto para o Enio (respostas, relatos de smoke, notas entre ferramentas) = pt-BR; strings do
app = inglês; docs/handoffs/commits = a língua que o repo já usa (pt-BR). Vale também logo a seguir a uma
compactação: o resumo não carrega o idioma, a regra carrega.

⚠️ **Esta linha dizia também "headers, tabelas" e "links markdown para arquivos", e isso ficou
para trás da correção de 2026-08-18 abaixo** — sobreviveu na mesma frase, a prescrever o
oposto. Para o **ENIO**: nada de nome de arquivo nem link de código, a menos que ele peça
(`CLAUDE.md §0.8`). Tabela só quando ela for a coisa mais **clara** para um leigo (uma tabela
de 12 colunas com nomes de crate não é). Para **handoff/ADR/gate/commit**, cujo leitor é a
próxima LLM: markdown denso, tabelas e links continuam certos e exigidos.

⚠️ **CORRIGIDO em 2026-08-18 — esta linha dizia "densidade alta", e o Enio pediu o OPOSTO:**

> *"Quero respostas sucintas, muito claras aos humanos leigos, falando apenas do essencial, com
> smokes em etapas descritas para quem está aprendendo do assunto."* · *"muitas das ferramentas
> que estamos implementando eu ainda não conheço e devo aprender a usá-las nos testes"*

Ele é o **DONO do produto, não um engenheiro acompanhando o desenvolvimento** — e o smoke é onde
ele **aprende a ferramenta que acabamos de construir**. Então: **curto, só o essencial, sem
jargão** (nada de gate, schema, crate, ADR, contagem de mutação, nome de arquivo/função a menos
que ele pergunte), dizendo **o que ele consegue FAZER agora** em vez do que foi construído.

⚠️ Vale para a resposta **AO ENIO**. Handoff, ADR, doc-comment, gate e mensagem de commit
continuam técnicos e densos — o leitor deles é a próxima LLM. A lei mora no `CLAUDE.md §0.8`
(sempre carregado); aqui fica só a correção, para esta memória não seguir prescrevendo o
contrário. Smoke em passos: [[feedback_ready_to_smoke_example]].
