---
name: Communication style — perguntas e formato
description: Como apresentar opções e respostas para o Enio em decisões pontuais.
type: feedback
originSessionId: 3810fc76-ee39-499c-932e-822ab7813c1b
---
Ao pedir decisão ao Enio, apresentar **2-3 opções concretas** com trade-offs explícitos, **recomendação primeiro** (com sufixo "(Recomendado)"), pedir sim/não — não open-ended "o que você acha?".

**Why:** HANDOFF.md L207-210: "Enio aprecia decisão pronta apresentada. Não aprecia 'consulte vibrational alignment com sua visão'." Validado em 2026-05-08 nas perguntas de bootstrap (modo de instalação, sobreposição de git, target rustup) — ele escolheu a opção recomendada nas 3 vezes ou validou alternativa específica.

**How to apply:** usar `AskUserQuestion` com 2-4 opções; primeira opção é a recomendada com label terminando em "(Recomendado)" quando for de fato a melhor escolha técnica; descrição concisa do trade-off em cada opção. Evitar perguntas vagas. Para decisões dentro de Hard Rules + tiebreakers do SKILL, decidir solo sem perguntar.

**Formato de resposta** (não pergunta): markdown estruturado, headers, tabelas, listas; código em blocos com linguagem; pt-BR direto sem floreio. Usar links markdown clicáveis para arquivos (formato VSCode: `[file.md](path/file.md)`).

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
