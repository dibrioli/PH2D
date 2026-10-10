# CONTINUAÇÃO — `line/MiroClone` W10 «As pontas da W9» (briefing, 2026-10-10)

> Para a janela que ASSUME a linha (`/pd-linha-assumir`, rota «linha reaberta» do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)):
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. W5…W9 continuam por integrar na mesma branch (integra-se tudo junto, só por ordem do
> dono); a W10 vai por cima.

## 1. Onde a linha está

- **W9 fechada e com smoke do dono OK (10/10)** — [handoff de integração W9](HANDOFF_INTEGRACAO_line_MiroClone_W9_2026-10-10.md)
  (o portão, a auditoria, a mutação, a régua das rotas, e o §8-bis: o traço à mão das setas curvas
  curado depois do 1.º smoke). Estudo: [`../07_fidelidade_excalidraw.md`](../07_fidelidade_excalidraw.md)
  (§2 as leis, §3 as recusas — incluindo as duas do fecho: o círculo tangente ao bico e o miolo ⇒ `Center`).
- O dono escolheu (10/10), entre «juntar ao main», «quadro ao vivo (Etapa 2)» e isto: **terminar as três
  pontas abertas da W9** (handoff W9 §9). Não abra outras frentes.

## 2. O que fazer (sem voltar a perguntar) — num bloco (CLAUDE.md §0.10)

1. **O 0.18.1 depois de MEXER uma forma** (auditoria W9, lente 1, média). Hoje o escritor grava o legado
   `{focus: 0, gap: g}` em toda a órbita/centro e `{focus: 0, gap: 1}` no ponto fixo (`ph2d-board-excalidraw/src/write.rs`,
   `binding`); ao abrir as duas versões mostram as pontas onde as escrevemos (gate
   `xv3_both_versions_keep_the_tips_we_wrote`), mas o 0.18.1, ao re-ligar depois de o artista mexer uma
   forma, mira ao CENTRO: as pontas em órbita fora do centro saltam **9,5–36,9 px** (medido em
   `saidas/w9_X_nosso.0181.json`, passo `empurrar`). Cura: **MEDIR primeiro a lei do `focus` do 0.18.1**
   — uma varredura no oráculo (`focus` −0,9…0,9 × rectângulo/elipse/losango × ângulo da seta × `gap`),
   num ficheiro de casos À PARTE (como o `w9_nosso_casos.json`: o `w9_casos.json` tem o sha nas 220 saídas),
   corrida pelo `corre_fidelidade.mjs --versao 0181`; já há dois pontos medidos (`B_legado_focus05_gap10` e
   `_diag`). Depois o escritor grava o `focus` que faz o 0.18.1 mirar pelo NOSSO foco. ⛔ O oráculo é
   CORRIDO, nunca lido (CLAUDE.md §0.9). O `inside` não existe no 0.18.1: meça o que o `gap 0` faz lá antes
   de escolher. Gate: o `empurrar` do 0.18.1 sobre o nosso ficheiro deixa as pontas a ≤ 0,05 (ou a lei
   exacta onde o 0.18.1 não a consegue exprimir, com o número medido ao lado).
2. **O ponto do realce** ao arrastar uma ponta (auditoria W9, lente 2, baixa): o overlay marca o FOCO
   (`ph2d-board-edit/src/wire.rs`, `Gesture::target` → `Target.fixed`), mas a ponta pára a `g` dali (na
   recta, onde a linha da outra ponta entra na faixa). Primeiro **meça no oráculo o que o Excalidraw `next`
   mostra** enquanto se arrasta uma ponta sobre uma forma (Playwright, `corre_fidelidade.mjs` com passos de
   rato e uma captura a meio do arrasto); depois o nosso realce mostra o MESMO (o ponto onde a ponta vai
   parar, ou o contorno realçado — o que ele fizer). Gate pelo overlay contra a rota.
3. **A letra do texto importado segue o botão «Rascunho ↔ Final»** — ✅ **DECIDIDO pelo dono (10/10):
   *«Troca junto com o botão»***. Ao ABRIR o `.excalidraw` o texto continua com a letra do ficheiro
   (`Look::letter` = `Hand`/`Sans`, M8); quando o artista aperta o botão (o quadro inteiro ou a selecção,
   `ph2d-board-edit` `toggle_sketch`), os elementos que ele troca passam a `Letter::Auto` (a letra do modo) —
   no MESMO passo de desfazer. A volta ao `.excalidraw` escreve a letra do modo (`font_family`). Gate no
   editor (o botão real) e uma lição na cena 11 ou uma linha na lição 1 (a cena não pode ensinar o
   contrário — §5.0).

Protocolo: cada lei com o teste VERMELHO visto antes da cura; o oráculo com cabeçalho e determinismo (duas
corridas iguais); fechar com `/pd-linha-fechar` (portão batched 1×, auditoria de 2 lentes, prova de
mutação, cena fotografada, handoff de integração W10 que SUBSTITUI o da W9, CLAUDE.md §5.1). Não integra,
não pusha. Falar com o dono SEMPRE em PT-BR, curto e sem jargão.

## 3. Lembretes medidos nesta linha (W9)

- O `cargo-test-narrow.sh` conta um erro de compilação como «0 testes»; o `cargo check --all-targets` com
  `CARGO_BUILD_WARNINGS=deny` à parte.
- O guarda de recursos recusa um `heredoc` com `cargo` dentro: escreva o script com a ferramenta de escrita
  e corra-o pela `scripts/ph2d-run.sh`. O `zsh` não parte uma variável em palavras: `git add
  --pathspec-from-file`.
- O tecto de 700 linhas conta os `*_tests.rs` (não o `tests.rs`): quatro ficheiros passaram-no na W9 —
  corte por responsabilidade ANTES do portão.
- Um campo novo no `Style` cresce CADA `Element` (o clippy `large_enum_variant` no `BoardOp` apanhou-o):
  meça o `size_of` antes; o raro vai para o `Org`.
- O oráculo: a página `www/` reconstrói-se pela secção «Página» do `instala.sh`; ao re-correr o esquema da
  W8 devolva o `w8_sondas.json` (só sorteados mudam).
- Uma tolerância na normal de uma aresta estica para `tol/sen θ` numa recta rasante (memória, família da Régua).
