# HANDOFF DE INTEGRAÇÃO — `line/3DModeling` · O CHÃO QUE SÓ RECEBE (2026-09-16)

**Branch:** `line/3DModeling` · **Base:** `git merge-base main HEAD`
**Ordem do dono:** *«adie essas pendências e deixe documentado… vamos voltar à implementação do
render»* → e, perguntado com as quatro saídas na mesa, **«chão invisível»**.

⚠️ **É o segundo handoff da mesma linha no mesmo dia.** O primeiro é o
[`…_O_ARCO_2026-09-16.md`](HANDOFF_INTEGRACAO_line_3DModeling_O_ARCO_2026-09-16.md) (o arco no perfil);
este é a `W4` do plano do render. Os dois são do mesmo ramo e integram juntos.

---

## §1 — O que mudou, em uma frase

A peça no modo **Render** deixou de flutuar: ela pousa num **chão invisível** que não se desenha e só
recebe — a sombra das lâmpadas e o escurecimento de contacto do céu. A `W4` do
[`03_o_plano.md`](../../Render3d/03_o_plano.md) fecha, com a régua dela corrida (`0`, `1` e `10 cm` do
chão dão três sombras diferentes). Mecanismo, medições e recusas:
[`docs/Render3d/07_o_chao_que_so_recebe.md`](../../Render3d/07_o_chao_que_so_recebe.md).

## §2 — ⚠️ CONTADORES PARTILHADOS: nenhum

`PROJECT_SCHEMA`, `FIELD_DOC_VERSION`, `VEC_SCENE_SCHEMA` e os registos de componentes **não se
mexem** — o chão é estado de VISTA, derivado do documento, e não viaja no arquivo. Zero contratos
congelados, zero ADR, zero pacote externo.

## §3 — Os ficheiros

| ficheiro | o quê |
|---|---|
| `ph2d-field-render/src/ground.rs` (**novo**) | as leis: `Ground::hit`, `lowest_point`, `ground_sky` (o céu do chão), `catcher_surface`, `ground_points` |
| `ph2d-field-render/src/shadow.rs` | `shadow_pass_on` — a sombra nos pixels de fundo, com a **cerca alargada** (§5 do doc 07); `Shadows::ground` |
| `ph2d-field-render/src/shade_render.rs` | o `catcher` (a razão), o fundo sombreado e o factor da BORDA |
| `ph2d-field-render/src/occlusion.rs` | o refinamento publica o céu do chão que o passe da sombra calculou |
| `ph2d-field-render/src/march.rs` | `EXHAUSTED_HERE` — o contador de raios largados **por thread** (§6) |
| `ph2d-field-gpu/src/trace_wgsl.rs` · `trace.rs` · `trace_uniforme.rs` · `trace_to_cpu.rs` | o chão no uniforme, os canais do chão na marcha, e de que altura eles são |
| `ph2d-field-gpu/src/trace_lampadas.rs` (**novo**) | os dois tectos das lâmpadas, movidos por TETO DE LINHAS (o `trace.rs` estava em `700/700`) |
| `ph2d-field-gpu/src/paint.rs` | o `fator_do_chao`, o fundo sombreado e o fundo da borda, no dispositivo |
| `ph2d-app-field3d/src/floor.rs` (**novo**) | a âncora: lida uma vez, quando o Render LIGA |
| `ph2d-app-field3d/src/{smoke_state,smoke,view,smoke_draw,smoke_draw_thread,gpu_frame}.rs` | o campo `floor`, a classificação de VISTA/cache, e o chão a viajar no pedido |

## §4 — ⚠️ CINCO coisas que uma leitura rápida do diff entende ao contrário

1. **O chão NÃO é geometria.** Ele não entra na marcha da câmera nem na de sombra: existe só onde um
   raio de câmera falha a peça. Uma peça que o atravesse continua a ver-se inteira — *um chão que
   tapasse coisas deixava de ser invisível*.
2. **O céu do chão não usa os cones da peça, e isso não é uma segunda lei por preguiça:** os `48`
   cones fixos num recetor PLANO desenham **anéis** (`17` extremos numa linha, contra `1` da
   referência convergida). A lei do chão é a oclusão por campo de distância, **ajustada contra
   `2 048` cones** e validada numa cena que não entrou no ajuste.
3. **A cerca alargada é do CHÃO, e a da peça continua a ser a bola simples.** Ali a bola é o que
   impede o estimador de ler a superfície de onde o raio saiu (a acne da esfera, `05` §27.5); aqui um
   ponto do chão não está sobre superfície nenhuma, e a bola simples cortava a penumbra numa elipse
   dura.
4. **A âncora é CACHE, não vista** — e é de propósito: ela é lida do documento quando o Render liga.
   Levantar a peça **não** levanta o chão (é isso que a régua da `W4` pede); voltar a ligar o Render
   pousa-o de novo.
5. **`Ground` vazio é o caminho de sempre, ao BYTE.** Sem chão, `shadow_pass_on(…, None)` é o
   `shadow_pass` de sempre e o pintor copia os bytes do fundo — e longe da peça, **com** chão, a razão
   é exactamente `1` e os bytes são os mesmos.

## §5 — ⚠️ TRÊS premissas minhas que a medição derrubou

- **«A caixa da peça serve para pousar o chão.»** Num cilindro inclinado ela desce `0,02` abaixo da
  borda (gate `num_cilindro_inclinado_a_caixa_desce_abaixo_da_peca_e_a_busca_nao`). ⇒ o ponto mais
  baixo procura-se com o próprio traçador, a olhar de baixo — e **dois** olhares não bastavam: numa
  quina o ponto ficava `2,9e-4` acima dela.
- **«A cerca `o.y > altura` do raio do chão é a definição.»** A mutação que a apagou **SOBREVIVEU**:
  ela é implicada pelo `t ≤ 0` (um raio que desce, vindo de baixo, nunca chega ao plano). ⇒ a cerca
  **saiu**. *Uma cerca que não muda nenhuma resposta é ruído no código.*
- **«O máximo do alfa nos pixels de fundo mede a sombra do chão.»** Ele media também a **silhueta** —
  um pixel de borda leva tinta por cobertura. Com a oclusão do chão ignorada o gate ainda lia `128`
  (a verdade é `224`) e a mutação passava. ⇒ a régua exclui as bordas e mede **duas** coisas (quão
  escuro, e quanto chão). Doc 07 §9.

## §6 — ⛔ Um gate de CONTAGEM que passou a reprovar sob `cargo test`, e a cura

O `a_shape_with_both_recesses_draws_whole_and_strands_no_ray` zera e lê o `march::EXHAUSTED`, que é
**do processo**: sob `cargo test` (um processo por binário, testes em threads) ele conta também os
raios que os vizinhos esgotam — e os gates do chão passaram a traçar ao lado dele. Sob `nextest` (um
processo por teste) ele sempre passou. ⇒ o gate traça numa **pool de UMA thread** e lê um contador
`thread_local` novo (`EXHAUSTED_HERE`), e fica certo nos dois executores.

⚠️ **E fica NOMEADO um vizinho maior, que não é desta wave:** sob `cargo test`, a suíte
`ph2d-field-render --test it` reprova entre `0` e `6` gates de CONTAGEM (tape/march/cache budget), com
o conjunto a mudar entre corridas — a assinatura de estado partilhado. Sob `nextest` os `93` passam.
*O executor que conta é o `nextest`; quem usar o `cargo test` naquela suíte tem de o saber.*

⚠️ **E um SEGUNDO vizinho, que a bateria de PLACA desta wave acusou e que não é dela:** o
`preview::device_tests::com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` reprovou na
bateria (`62` de `63`). ⛔ **Não é o chão, e há duas provas:** ele passa **`None`** no chão à
`gpu_frame::paint` (o quadro de MOVIMENTO não conhece esta wave), e com a CPU a `82`–`91 %` ociosa ele
passa **3 de 3** (`12`, `11`, `11` de `18`) — que é o `11 de 18` que a
[`05` §43.9](../../Render3d/05_o_modo_render_do_modelador.md) já lhe tinha registado antes desta
linha existir.

⛔⛔ **O crivo NÃO é o `loadavg`**, e isto custou-me três corridas: a `load 7,5` ele lê `5`, `2` e `3`
de `18` e reprova, porque aquela é uma média de **um minuto a decair** (a de cinco lia `32` no mesmo
instante). A grandeza é a **ociosidade da CPU**, que o próprio gate imprime desde a §43.9 — *uma
régua que desmente uma flake tem de medir a grandeza que a flake segue, e não a que é fácil de ler.*

## §6-bis — ⚠️ O `fmt` desta linha estava VERMELHO em dois ficheiros que ela própria escreveu

O `cargo fmt --all` deste fecho mexeu em **dois** ficheiros fora das crates do campo, e os dois vêm
de commits **desta linha** (`02554bfae` e `b29972e57`):
`crates/ph2d-gpu/examples/field_march_ceiling.rs` e
`shells/desktop/src/render_loop/hierarchy_duplicate_routing_tests.rs`.

⇒ *um fecho que corre `cargo fmt -p <as minhas crates>` é cego ao que a linha escreveu numa crate
**vizinha** ou na shell* — a mesma forma que o `CLAUDE.md` §5 já regista para os gates de arquitectura
(*«um portão que só corre o que a linha editou é cego a todo espelho»*), agora no formatador. A cura
entra neste commit; a lei é **`cargo fmt --all`** ao fechar, e conferir o que ele tocou fora do
esperado em vez de o descartar.

## §7 — As medições

| | |
|---|---|
| paridade CPU × dispositivo do chão | `100,000 %` dos canais a `≤1` nível, **pior `0`** (`192×108`, `12 878` bytes mudados pelo chão) |
| o céu do chão contra `2 048` cones | esfera e cubo (ajuste) `0,044` · **cruz (fora do ajuste) `0,028`**, pior `0,114` |
| anéis | lei `212` extremos · `48` cones `904` (o gate exige `4×` menos) |
| a cerca alargada | pior salto longe da peça `0,968 → 0,093`; saltos `> 0,2`: `87 → 0` |
| o ponto mais baixo | esfera e cubo-na-quina a `< 1e-4` da verdade |
| **o chão no DISPOSITIVO** (o caminho do produto) | `1080p`: **`+0,43 ms`** sobre `10,52` (`+4,1 %`) · `640×360`: abaixo do ruído. A CPU paga **`+86,3 ms`** pela mesma resposta |
| a imagem do produto, com e sem chão | **`0`** pixels da PEÇA mudam · `95 140` de fundo ganham a sombra (alfa médio `0,8 → 24,7`, maior salto `216`) |

⚠️ **As duas linhas do dispositivo foram medidas com a CPU a `66`–`92 %` OCIOSA**, mínimo de 5
corridas intercaladas. ⛔ Com a máquina ocupada o mesmo `1080p` lê `41,2 ms` — **`3,4×` inflado** —, e
o `loadavg` **não** serve de crivo: ele dizia `7,5` (média de um minuto, a decair) com a de cinco
minutos ainda em `32`. Mecanismo e tabela: [`07` §7-ter](../../Render3d/07_o_chao_que_so_recebe.md).

## §8 — O smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Na barra da área 3D, o pulldown **Shading** (o chip fechado lê `Matcap`) → **Render**. A peça passa a
ter sombra no chão e escurecimento de contacto; movê-la para cima com a seta do gizmo afasta a sombra
e amolece-a.

## §9 — ⏳ O que fica aberto

- o chão **não devolve** luz à peça (a luz indirecta é a `W5`, e o chão será o primeiro recetor dela);
- um só chão, **horizontal**, e a altura é da cena;
- sem lâmpada nenhuma o dispositivo recusa o quadro (cerca anterior a esta wave) e o chão é pintado
  pela CPU;
- um botão de **«pousar outra vez»** — hoje re-ancora-se ligando o Render de novo.
