# ADR-0194 — O HUD arruma IRMÃOS com o Cassowary; a âncora ao pai e a raiz continuam procedurais

- **Status:** Accepted — provisório (linha `line/BlenderConstraints`; o integrador reconta o número). O produto (a
  pontuação cresce e a moeda fica ao lado; a tela estreita encolhe o número antes de invadir o minimapa; o arranjo é
  uma ferramenta da barra esquerda) é do DONO (27/10); o desenho técnico é delegado.
- **Data:** 2026-10-27
- **Linha:** `line/BlenderConstraints` (W5.5)
- **Amparo:** [desenho §1.3, §2, §3.3, §4](../../BlenderConstraints/06_W5_resto.md) · [oráculo](../../BlenderConstraints/ferramentas/oraculo_kiwisolver_hud.py) · [fixture](../../../crates/ph2d-hud-layout/fixtures/kiwisolver_1_5_1_hud.json)

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 27/10 contra o máximo do `main`/worktrees (`0193`). Quem integrar reconta-o.

## Contexto

`ph2d-ecs/src/vec_anchors.rs` (03/08) e `ph2d-hud/src/lib.rs` recusavam o Cassowary por ARGUMENTO, não por medição:
*«o caso relacional já tem resposta — o align/distribute»*. A premissa não se confirma (06 §1.3): o align/distribute é
um gesto de UM passo, só no Quadro; não há âncora a IRMÃO; o texto da pontuação muda em runtime e nada mede a sua
largura por quadro. A cerca continua certa para o que cobre — âncora ao PAI e a raiz do HUD, aritmética fechada — e
não cobre irmãos nem desigualdades («encolhe antes de invadir»), que é onde um solver é a resposta.

## Decisão

1. **Crate `ph2d-hud-layout`, sem ECS**, sobre `kasuari 0.4.13` (Cassowary incremental, fork mantido pelo ratatui;
   MIT OR Apache-2.0; `default-features` = só `std`). `Arranjo::new(canvas, elementos, relações)` monta o sistema uma
   vez; `atualiza(canvas, conteúdos)` muda só as *edit variables* (canvas e larguras medidas) e devolve as caixas; `atualiza_elementos` também (ver W5.5 §1).
   Relações: `AoLado` · `Centrado` · `NaoInvade` (desigualdade) · `LarguraDoConteudo` (`w == conteúdo` com a força do
   par e `w <= conteúdo` `REQUIRED`), com força `Requerida/Forte/Media/Fraca`.
2. **O sistema implícito** (doc do módulo; o oráculo monta o MESMO): cada elemento com relação fica onde foi desenhado
   com pesos ABAIXO de `WEAK`, `h` `STRONG`, `w,h >= 0` `REQUIRED`, dentro do canvas `STRONG`; edições a `1e9`.
3. **Desempate medido:** com os quatro stays a `WEAK` (a primeira espec), 10 dos 20 casos do oráculo têm ótimo NÃO
   único — o kiwisolver dá outra resposta com a ordem de inserção trocada (a moeda e a pontuação empatam em quem anda).
   Pesos `PESOS = [0,125; 0,3125; 0,5; 0,6875]`; a atribuição final por três degraus está em W5.5 §2 (20/20 únicos).
4. **O mesmo bit:** o `kasuari` guarda o quadro em `hashbrown` com semente aleatória por processo (foldhash), logo o
   caminho do simplex muda de instância para instância; e as edições somam deltas. Medido: ida e volta `61,3 → 377,9 →
   61,3` devolvia `w` a 2 ulp. Cura: toda entrada passa por uma grelha diádica de `1/256` px — com coeficientes `±1`/`½`
   as contas são EXATAS e qualquer caminho dá o mesmo bit (gate `the_layout_is_the_same_bit_every_frame`: 100 quadros,
   10 idas e voltas, 50 instâncias).
5. **Um solver por componente conexo** das relações: os grupos não partilham variáveis, o ótimo é o mesmo (o oráculo
   resolve tudo num só solver e casa), e o custo deixa de ser quadrático no HUD inteiro. Sugerir o valor já sugerido
   não chama o solver. Quem não tem relação devolve a sua caixa ao bit e não paga nada.
6. **Nunca pânico:** índice fora da lista ou `a == b` ⇒ relação ignorada; `REQUIRED` que o solver recusa ⇒ fica de fora
   (a primeira inserida ganha).
7. Corre DEPOIS de `VecAnchors` e de `ph2d-hud::place`, só sobre os irmãos com relação. A largura medida do rótulo por
   quadro existe desde W5.5 (§5 abaixo).

## Oráculo (corrido, nunca lido)

`python-kiwisolver 1.5.1` (BSD-3-Clause, `pacman -Qi python-kiwisolver`), o Cassowary de referência, monta o mesmo
sistema por script e grava `kiwisolver_1_5_1_hud.json` (cabeçalho com versão, licença, comando e data). 20 casos em 6
famílias (ao lado, pilha, centrado, não invade, largura com força, requerida impossível), vários quadros cada (o
caminho incremental também é régua). Gate `the_layout_matches_kiwisolver_case_by_case`, tolerância `1e-9` absoluta —
o ótimo é único (gravado por caso) e os números são diádicos; o resultado medido casa ao bit.

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| `GtkConstraintLayout` (GTK 4.24.1) como motor ou oráculo | LGPL-2.1+; o kiwisolver basta e corre sem ecrã. |
| Aritmética à mão por relação (como a âncora ao pai) | Não exprime desigualdades nem forças: «encolhe antes de invadir» é um ótimo, não uma fórmula. |
| `cassowary` (crate original) | Sem manutenção; o `kasuari` é o fork com correções, usado pelo ratatui. |
| Stays todos `WEAK` (a primeira espec) | Medido: 10/20 casos com ótimo não único — a resposta dependeria da ordem de hash. |
| Confiar no simplex para o mesmo bit sem grelha | Medido: 2 ulp na ida e volta; e a ordem do quadro do `kasuari` é aleatória por processo. |
| Um solver para o HUD inteiro | Quadrático no número de linhas; os grupos independentes têm o mesmo ótimo. |

## Preço

- **Desigualdades `REQUIRED` contraditórias não são detetadas** pelo Cassowary (kiwi e kasuari): medido com `AoLado` +
  `NaoInvade` opostos, o kiwisolver aceita e viola outra requerida (`w = −8`), com resposta que muda com a ordem.
  Só a contradição de IGUALDADES é recusada. Fica fora da fixture; a ferramenta da barra esquerda impede o par (W5.5 §7, gate `the_solver_silently_accepts_opposite_required_sides`).
- A grelha de `1/256` px desloca uma caixa com relação até `1/512` px do desenho (invisível; quem não tem relação fica
  ao bit).
- Um grupo que não cabe no canvas reparte a violação entre restrições `STRONG` iguais — ótimo não único; o gate do
  mesmo bit cobre a mesma instância, o oráculo só cobre casos que cabem.
- Custo (27/10, `--profile smoke`, mínimo de 3, `load ~9` — relógio indicativo): 1 000 elementos em 500 pares:
  `new` 8,3 ms · `atualiza` com os 1 000 conteúdos a mudar 0,78 ms · sem mudança 0,14 ms. Um GRUPO único cresce
  ~n³ no `new` (cadeia de 50 / 100 / 200: 33 ms / 0,30 s / 2,95 s; `atualiza` 0,24 / 0,97 / 5,1 ms) — toda contenção
  no canvas partilha as 4 variáveis dele e as linhas do quadro ficam densas. Um HUD real tem grupos de poucos
  elementos; o tecto medido é `TECTO_GRUPO = 64` (W5.5 §8).

## Decisões da W5.5 (27/10/2026)

Cada uma com a razão medida. Onde contradiz o texto acima, vale esta secção.

1. **Stays como *edit variables*, com a força do próprio stay** (`Arranjo::atualiza_elementos`): as caixas mudam por quadro (âncoras, rótulos recozidos) sem remontar o solver. Gate `moving_boxes_give_the_bits_of_a_fresh_build` (igual a uma montagem nova quando o ótimo é único).
2. **Três degraus de stay de posição.** Seguidor puro (`a` de alguma relação, `b` de nenhuma) `PESOS[0]`/`[1]`; seguidor E âncora (meio de uma pilha) `PESOS[2]`/`[3]`; âncora pura (nunca `a`) posição `STRONG` (largura `PESOS[3]`). Medido: a 0,5 o minimapa era empurrado à borda do canvas antes de a pontuação encolher; a `STRONG` no meio da pilha o oráculo teve um caso não único (três linhas empilhadas) ⇒ três degraus, oráculo 20/20 únicos.
3. **Só o TEXTO muda de tamanho:** elemento sem `LarguraDoConteudo` guarda `w` a `STRONG`. Medido na foto da cena 11: a moeda achatava-se antes de a pontuação encolher.
4. **`PARADA = 2/GRELHA`:** a soma de três números da grelha difere da soma arredondada até 1,5 célula (a moeda «andava» 0,0023 com folga 0,4); abaixo de `PARADA` o passe não publica nada. A ferramenta grava a `folga` já na grelha (`na_grelha`).
5. **Largura do rótulo = a caixa VIVA dele** (recozido com o texto derivado antes do passe, `hud_label_live`), não uma 2.ª modelagem por `unwrapped_block_width`: uma só medida para a caixa que as âncoras e o desenho usam (lados da tinta ≠ avanço). Desvio deliberado do plano do handoff W5.4.
6. **Rótulo derivado cresce do ALINHAMENTO** (`ph2d_vec_text::alinha::ao_autorado`): esquerda → para a direita, direita → para a esquerda, centro como antes ao bit. O placar preso à direita da cena HUD antiga passou a alinhar à direita.
7. **Ferramenta «HUD Layout»** (`DrawMode::HudLayout` na coluna Vector; ids `vector.mode.hud` / `vector.hud.*`): arrastar uma seta elemento→irmão cria `AoLado` no eixo dominante com a folga ATUAL (ligar não move nada); a barra da seta escolhida: Ao lado / Não invadir / Centrar através / Apagar; texto que liga ganha `LarguraDoConteudo(Media)`. `UiArranjo::junta` substitui o mesmo par no mesmo eixo, recusa círculo de lados (`ph2d_hud_layout::contradiz`: o Cassowary aceita desigualdades `REQUIRED` opostas e viola uma em silêncio, gate `the_solver_silently_accepts_opposite_required_sides`) e recusa grupo maior que `TECTO_GRUPO`.
8. **`TECTO_GRUPO = 64`** (recurso: tempo de montar o solver de um grupo ao editar uma seta; o quadro vivo só atualiza). `kill_w5::w5_hud_group_cost_by_size`, `--profile smoke`, mínimo de 7×10, `load ~96`, indicativo:

   | Cadeia | 8 | 16 | 32 | 64 | 128 |
   |---|---|---|---|---|---|
   | `new` (ms) | 0,31 | 0,97 | 2,8 | 12 | 86 |
   | `atualiza` (µs) | 7 | 12 | 25 | 55 | 95 |

   64 cabe num quadro de 60 Hz; 128 travaria ~5 quadros.
9. **Critério de morte** (mesma rodada intercalada, `--profile smoke`, mínimo de 7×40, `load 96`): Valor 0,468 ms · behaviours 0,404 ms (quadro − pilha base de 1 000 `Limit`) · HUD 0,456 ms por 1 000 (500 pares, toda caixa e todo conteúdo a mudar por quadro). Uma 1.ª rodada a `load 140` leu 0,53/0,44/0,63: o que o intercalado garante é a razão HUD/Valor ≈ 1,0–1,2.
10. **Dados:** componente `UiArranjo` no canvas (`ph2d-ecs::hud_arranjo`, relações `Relacao<StableId>` — `Relacao` é genérica no id), registado, descritor «HUD Arrangement» (maquinaria), `PROJECT_SCHEMA` 196→197. O passe `ph2d_app_components::hud_arranjo` corre depois de `anchor_canvases` no `LayoutLive.recook` da shell, só no preview, e publica um instantâneo por quadro (`Visto`) de onde a ferramenta desenha. Cena 11 «Painel do jogo» (`PH2D_CONSTRAINT_SMOKE=11`).
