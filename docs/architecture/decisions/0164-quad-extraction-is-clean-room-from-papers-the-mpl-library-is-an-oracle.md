# ADR-0164 — A EXTRAÇÃO de malha quad é clean-room dos *papers*; a biblioteca MPL-2.0 é ORÁCULO, não fonte a portar

- **Status:** Accepted
- **Data:** 2026-08-24
- **Linha:** `line/sculpt3d` — papel **E** da [`SKILL_Cleanroom`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)
- **Toca:** `ph2d-gridmap` (o arredondamento inteiro) · uma crate nova de extração · `ph2d-quadflow` (a porta do botão)
- **Não move:** contrato congelado nenhum (CLAUDE.md §6). Nada no produto muda enquanto o interruptor estiver desligado.
- **Espec funcional:** [`SPEC_extracao_de_malha_quad.md`](../../3D/cleanroom/SPEC_extracao_de_malha_quad.md)
- **Triagem e medições:** [`TRIAGEM_quad_remesh.md`](../../3D/cleanroom/TRIAGEM_quad_remesh.md)

## Contexto

A caça por eliminação de 2026-08-23 fechou com um diagnóstico e um endereço: *a distorção
nasce entre o **domínio** e a **superfície**; mesmo com traçado, marcação e domínio perfeitos,
o preenchimento **por patch** fica em `15°` de enviesamento e o oráculo de produção faz `6°`.*
A obra nomeada foi a **extração** — pôr os pontos da grade nas isolinhas inteiras de um mapa
global, em vez de interpolar o interior de cada patch a partir da fronteira.

A triagem de licença de 2026-08-24 (§2 da skill) descobriu que essa obra **já existe** sob
**MPL-2.0** — copyleft **por-arquivo**, licença **já aceite** pelo `deny.toml` desta casa.
Isso abriu um degrau **T0½** (porte fiel, horas-dias) ao lado do **T2** (clean-room, semanas).

⭐⭐⭐ **E a medição do mesmo dia mudou o peso da decisão.** O arnês em
`~/Referencias/directional-bench/` exporta o **nosso** campo cruzado no formato de
intercâmbio da biblioteca e corre a extração dela. Com a régua **por-face** desta casa:

| | campo **deles** | ⭐ campo **NOSSO** | oráculo de produção | ⛔ o nosso F5 hoje |
|---|---|---|---|---|
| enviesamento p50 | `5,0°` | ⭐ **`3,0°`** | `6°` | ⛔ **`27°`** |
| enviesamento máx | `43,3°` | **`29,6°`** | — | — |
| faces com canto pior que 60° | `0` | **`0`** | `0` | ⛔ **`9 159`** |
| aspecto p50 | `1,13` | **`1,06`** | `1,08` | `1,98` |
| ⚠️ aspecto máx | `187` | ⚠️ `3 639` | — | `122,7` |

⇒ **Duas coisas ficaram provadas de uma vez:** a rota da extração **ultrapassa** o oráculo de
produção na grandeza que perseguimos, e **o nosso campo é melhor que o deles** — o F2 estava
ilibado por contagem de singularidades e agora está ilibado por **resultado**.

## Decisão

**A extração é escrita nesta casa, clean-room, a partir dos *papers* públicos.**
**A biblioteca MPL-2.0 fica FORA da árvore, como SEGUNDO ORÁCULO.**

⛔ **Nenhum arquivo dela é traduzido para dentro do repositório.**

## Porquê (cada razão com medição ou mecanismo)

1. ⭐ **Já temos 80% da cadeia, e é nossa.** `ph2d-gridmap` entrega corte em discos, penteado
   com salto de período, solver global e marcação; `ph2d-crossfield` entrega o campo, o índice
   por-vértice e o salto de período. Um porte fiel **descartaria isso** para re-encanar tudo
   sobre as estruturas de malha e a álgebra linear deles.
2. ⭐⭐ **Custo de licença ZERO.** O porte deixaria arquivos **permanentemente públicos** dentro
   do subsistema mais valioso do produto. A MPL-2.0 não contamina o resto — mas *aqueles*
   arquivos ficam abertos para sempre, e a obrigação viaja com cada versão futura deles.
3. ⭐ **As duas peças que faltam estão inteiramente publicadas** — o arredondamento
   misto-inteiro (2009) e a extração (2013) —, e a espec já as re-descreve em vocabulário
   desta casa, com os gates e as barras.
4. ⭐⭐⭐ **A biblioteca permissiva é um oráculo MELHOR que o GPL.** Ela lê o campo por
   **arquivo**, e o formato não é expressão protegida ⇒ **corre sobre a NOSSA malha com o
   NOSSO campo**, e escreve a malha resultante. *Comparar fase a fase, na mesma peça, é mais
   forte que comparar o fim* — e com o oráculo GPL isso nunca esteve disponível.
5. ⚠️ **Medido: a extração dela é lenta na nossa escala.** Segundos numa peça de 2 404
   triângulos; **minutos sem terminar** numa de 6 768. ⇒ portar seria herdar um custo que
   ainda não entendemos, numa implementação que não conseguimos correr sobre os nossos dados.
   *Escrever do *paper* deixa o desenho de desempenho na nossa mão.*
6. ⚠️ **O degrau T0½ continua disponível** se a obra 2 (§6 da espec) se revelar mais cara do
   que a espec prevê. ⛔ **Esta decisão não o queima** — só não o toma primeiro.

## Consequências

- ⭐ **O repositório continua inteiramente proprietário.** Nenhum arquivo publicado.
- A obra parte em duas, e **a primeira é pequena**: o arredondamento **uma-a-uma com
  re-solve** (espec §5) fecha o bloqueador nomeado da `ph2d-gridmap` (resíduo de `0,291` de
  célula nas translações de ciclo).
- ⛔ **Predicado de orientação exacto é obrigatório** (espec §1), com `num-bigint` +
  `num-rational` (MIT/Apache) atrás de um filtro em `f64`.
- ⚠️ **Quem escreve NÃO é a janela que especificou.** Esta janela leu fonte de alvo copyleft
  durante a triagem e está **queimada para o papel I** neste módulo (ledger, §Papel E).
- ⏳ **A auditoria R-pré da espec (§4.2) está PENDENTE** e é condição de abertura da janela I.
- ⭐ O arnês fica: `~/Referencias/directional-bench/` (biblioteca + arnês C++ + exportador de
  campo em Rust + régua em Python). Ele **não entra** no repositório.

## Alternativas rejeitadas

| alternativa | por que não |
|---|---|
| **Porte fiel T0½ da biblioteca MPL-2.0** | arquivos permanentemente públicos no subsistema mais valioso; descarta a cadeia que já temos; herda um custo de desempenho não compreendido (razão 5) |
| **Clean-room T2 do remalhador GPL de produção** | ⛔ semanas, 4 janelas, parede, vassoura e ledger de contaminação — **para chegar a uma qualidade que a rota escolhida já mede como melhor** (`3,0°` contra `6°`) |
| **Continuar a melhorar o preenchimento por patch** | ⛔ **recusado por medição em 2026-08-23**: quatro achatamentos, forma do domínio, poda de patches, subdivisão local e ponto fixo — todos medidos, todos fechados. `16` rondas de relaxação movem a mediana de `27°` para `26°` e pagam `3,4×` as dobras |
| **Perseguir o campo (as linhas neurais de 2024-2026)** | ⛔ melhoram a fase que **já está certa**: o nosso campo mede `3,0°` contra os `5,0°` do campo de referência |
| **Portar a implementação de referência do emparelhamento (Blossom)** | ⛔ **não é software livre** — avaliação e pesquisa apenas, redistribuição proibida, licença comercial à parte. É **T4** |

## ⛔ Recusas MEDIDAS

| recusa | mecanismo | onde |
|---|---|---|
| ⛔ **Não portar a biblioteca MPL-2.0** | obrigação de publicar arquivos no subsistema mais valioso, + descarte da cadeia própria, + custo de desempenho não compreendido | acima, razões 1-2-5 |
| ⛔ **Não abrir clean-room T2 do alvo GPL** | a rota escolhida mede **melhor** que ele, sem parede nenhuma | alternativas |
| ⛔ **Não voltar ao preenchimento por patch** | família fechada por medição em 2026-08-23 | [`PLAN.md`](../../3D/quad-remesh/PLAN.md) §4-tricies..§4-septemetquinquagies |
| ⛔ **Não trocar o nosso campo** | ele mede **melhor** que o da biblioteca de referência | contexto |
