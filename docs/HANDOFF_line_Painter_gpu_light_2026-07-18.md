# HANDOFF — a LUZ do impasto roda na GPU (2026-07-18)

**Linha:** `line/Painter` (Modo L). **Estado:** fechado, **pendente smoke do Enio**. NÃO integrado (aguarda
ordem explícita). Continuação de [`HANDOFF_line_Painter_inflate_closing_2026-07-18.md`](HANDOFF_line_Painter_inflate_closing_2026-07-18.md).

## O que estava errado (e era muito maior que a luz)

`gpu_eligible` desistia no instante em que `impasto_visible()` — então **um documento com QUALQUER relevo
compositava a pilha inteira na CPU**: cada blend mode, cada ajuste, cada camada, todo frame. A luz era o
*motivo* e a luz é a parte barata.

E era pior no caso mais comum: a CPU também recusa seu caminho zero-composite quando há relevo
(`runtime.rs:223` — aquela pista devolve o `Arc` cru de `canvas_rgba`, e a luz não pode escrever nos
PIXELS do artista). Ou seja, **um desenho de uma camada só, esculpido — o jeito mais ordinário de usar
Impasto — pagava composite completo + luz completa na CPU a cada frame sujo**, e a GPU estava proibida de
ajudar.

## O desenho

**Passe pós-composite, não um `LayerOp`.** A nota diferida pedia um `LayerOp` novo; está errada. A luz é
**espacialmente não-local** (diferenças centrais) e roda **uma vez no fim** — o próprio compositor
argumenta (`layer_compositor/mod.rs:154-168`) que não-local pertence ao pass-graph segmentado, não ao laço
per-pixel. Como passe pós-composite ele não toca `LayerOp`, `flatten_layer_ops`, `validate_op_list`,
segmentação nem bind groups do compositor: superfície muito menor, e o encaixe é o MESMO da CPU
(`composite -> luz -> overlay`).

**Só a ÓPTICA porta. O fold não.** Este é o orçamento de risco inteiro:

| fica na CPU (canônico, uma implementação) | vai pro shader |
|---|---|
| quais camadas, em que z-order, `Add`/`Level`, `impasto_depth`, traço vivo, **teto de vidro** | normal por diferença central |
| cobertura (`max` entre camadas) | 4 lâmpadas, difuso/especular relativos |
| material (fold `over` a partir de `NEUTRAL`) | `wrapped_ndl`, LUT, `channel`, `light_pixel` |

Um shader que re-derivasse o fold seria uma segunda resposta a *"como camadas de tinta se empilham"*, e as
duas divergiriam no único lugar onde ninguém lê um número: uma screenshot. O port é limitado, puro, e
pinado contra a função que ele porta.

**O LUT especular SOBE, não é recomputado** — `powf` é o único transcendental do modelo e assim ele nunca
roda no dispositivo. É a razão principal de a paridade ter saído exata.

### Arquivos
- **`crates/ph2d-render/src/impasto_light.rs`** (NOVO, 644): `ImpastoLightPass` (irmão do `PreviewPremul`)
  + `ImpastoLightInput::check` — a porta ÚNICA de validação de forma (os gates exercem o mesmo predicado
  que o `run` aplica).
- **`crates/ph2d-render/src/shaders/impasto_light.wgsl`** (NOVO, 227).
- **`crates/ph2d-tool-painter/src/tool/paint/impasto_gpu.rs`** (NOVO, 268): materializa os 3 planos
  chamando o sampler DA PRÓPRIA luz (`ReliefFields`), nunca uma cópia do fold.
- **`impasto_shade.rs`**: `Rig::export_lamps` (o rotor de 1° fica na CPU — um `sin`/`cos` no shader seria
  um 2º rotor) + `force_chromatic` (`cfg(test)`).
- **`impasto_light.rs`**: `apply_impasto_light` virou **`pub`** — é a passagem canônica contra a qual a GPU
  é reconciliada. Visibilidades `pub(super)` para o materializador irmão.
- **`material.rs`**: `SpecLut::table()`.
- **`painter_gpu_preview.rs`**: `compose_light_premul` (extraída, porta única que o gate e2e dirige) + o
  portão.

## Uma decisão de projeto que vale reler antes de mexer

**A luz despacha a tela INTEIRA, mesmo quando o composite atualizou só uma região.** Ela tem sua PRÓPRIA
textura persistente, então a frescura dela não pode ser herdada do retângulo que o compositor tocou: um
frame com o relevo escondido (planos `None`, compositor atualiza uma região, luz não roda) seguido de um
frame parcial iluminado carregaria pixels de antes daquela atualização, no canto onde ninguém olhava. Hoje
custa **zero** (todo despacho já é tela cheia) e continua correto se uma pista parcial for adicionada.

## A política de paridade — e a correção que ela traz

A nota diferida dizia *"reconciliado bit-a-bit"*. **Isso não é a política deste projeto** e persegui-la
seria caçar fantasma: o compositor declara (`mod.rs:24-31`) que a saída de runtime NÃO é bit-idêntica entre
backends (um backend pode contrair `a*b+c` em FMA). O template real é: **literais bit-idênticos por gate
CPU-only** + **acordo de runtime dentro de um épsilon documentado** por gate `#[ignore]` com GPU, sempre
contra o **kernel canônico**.

**Medido nesta RTX: `worst delta 0`, `0 de 16384 bytes diferem`, nos 5 materiais** (neutro / glossy / metal
/ waxy / metal+wax). Melhor que o orçado — porque o LUT sobe pronto e o store é quantizado
explicitamente (`floor(v*255+0.5)`, o half-away do Rust) em vez de deixado à conversão unorm.

⚠️ **O gate mede DUAS coisas, e a segunda foi paga por uma mutação:** um limite de MAGNITUDE sozinho é
insuficiente. Tirar o `+0.5` do `quantise` — virar arredondamento em truncamento, exatamente a divergência
que a função existe pra abolir — move **2375 de 16384 bytes por UM nível** e passava tranquilo sob um
limite de 2. Então há também `MAX_DIFFERING_BYTES = 16`: *quão longe* alguém foi (ULP/FMA) e *quantos*
foram (erro sistemático) são perguntas diferentes.

## Gates

**`ph2d-render/tests/impasto_light_gpu.rs`** (`--ignored`, GPU real):
- `gpu_impasto_light_matches_the_cpu_pass` — 5 materiais, contra `apply_impasto_light`.
- `the_shader_leaves_flat_paint_byte_identical` / `the_shader_does_not_touch_bare_paper` — `assert_eq!`.

**`shells/desktop/.../painter_preview_handoff_tests.rs`** (`--ignored`, GPU real):
- **`the_gpu_producer_shows_what_the_cpu_producer_shows`** (NOVO) — **o gate a que este port responde**:
  produto real (`try_drive` -> elegibilidade -> flatten -> compositor -> luz -> premul -> slot), readback
  do dispositivo, comparado ao OUTRO produtor. Byte-idêntico. *Não podia ser escrito antes de hoje: não
  existia frame GPU com relevo pra ler de volta.*

**CPU-only** (rodam em qualquer runner): literais do shader vs constantes Rust · `inverseSqrt` proibido ·
forma mal-formada recusada sem dispositivo · os planos SÃO o fold da luz texel a texel · os planos desistem
onde a CPU desiste · `the_achromatic_fast_lane_is_the_coloured_one_to_the_bit` (**a licença** do shader de
caminho único) · 3 gates de elegibilidade no shell.

### Mutações: 6 rodadas, 5 sangram
AMBIENT · sinal da normal · tint metálico · peso de cobertura · `quantise` sem `+0.5` (só depois do gate
contador) — todas RED. E a mutação do AMBIENT também derruba o gate e2e (2961 bytes, 4 níveis).

⚠️ **1 sobrevivente, POR PROJETO e documentado no gate:** neutralizar o early-out de tinta plana deixa tudo
verde — porque com inclinação zero `N·L = L.z` para toda lâmpada, o difuso é igual ao divisor termo a termo
e a razão é exatamente 1. **O contrato é honrado duas vezes** (pelo ramo e pela aritmética), então nenhuma
mutação única o mata — a lição `feedback_layered_defenses_need_per_layer_gates`. Quem apagar o early-out
por asseio perde velocidade e nada mais.

## O que MUDOU de comportamento (leia antes de smokar)

**Uma coisa só:** `gpu_eligible` deixou de recusar relevo. A recusa trivial agora é
`preview_is_trivial_stack() && !impasto_visible()` — espelhando `runtime.rs:223` deliberada e
visivelmente, porque **as duas pistas têm de concordar** ou o trabalho vai pra pior delas.

⚠️ **Gate alterado, e a razão importa:** `the_screen_survives_the_gpu_to_cpu_producer_handoff` usava
`impasto_show` como alavanca pra virar elegibilidade, afirmando *"the GPU compositor cannot light relief"*.
Ele **não testava a luz** — testava a dança de handoff (re-seed do slot, `arc_token`, pista parcial). A
alavanca morreu; o propósito não. Alavanca nova = **máscara de camada** (que o `flatten_for_gpu` segue não
representando). O relevo agora aparece no lado GPU da dança em vez do lado CPU, e é a única coisa que mudou
no que ele prova.

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && cargo run -p ph2d-host-desktop --release
```
1. Pinte traços grossos com impasto (falloff já nasce Sphere) e confira que **o relevo continua com a mesma
   aparência de sempre** — o port é byte-idêntico, então qualquer diferença visível é bug.
2. Adicione uma 2ª camada, ajustes, blend modes: a pilha inteira agora compõe na GPU **com** relevo. Era o
   caso que caía todo na CPU.
3. Esconda/mostre a luz (o olho do Impasto) e mexa nas lâmpadas: deve responder ao vivo.
4. Adicione uma **máscara** de camada: volta pra CPU (não-representável) e o desenho não pode piscar nem
   mudar.

## Verificação
tool-painter 715 · render 153 + 3 GPU novos · shell 693 · editor-core 753 + arch/LOC · workspace
`check --all-targets` limpo · clippy 0 · fmt aplicado. Todas as pistas `--ignored` de GPU verdes.

**Vermelhos que NÃO são desta linha** (medidos, não argumentados): `write_mobile_to_disk` (sonda manual de
áudio, exige `PROBE_OUT`) · `watercolor_app_params_incremental_matches_full_{diluted,mixer_on}` (Δ2 stale
num caminho que este trabalho não toca) · `sculpt_perf_kill_criterion` — INFLATE mede **7,77 ms @2048 /
8,15 @4096** contra o kill 8, encostado no limite; ver §Perf.

## Perf — o vermelho do sculpt NAO e desta linha (medido, nao argumentado)

`sculpt_perf_kill_criterion` falha no INFLATE. **Medi contra o HEAD shipado (`ba03ed84`) num worktree
separado, alternando as duas arvores com a maquina assentada:**

| rodada | `line/Painter` | HEAD `ba03ed84` |
|---|---|---|
| 1 | 7,73 ms | 7,72 ms |
| 2 | 7,71 ms | 7,70 ms |

**Identicos dentro de 0,03 ms** — este trabalho nao toca o caminho do sculpt e nao custa nada nele.

O que o exercicio expos e outra coisa, e vale registrar: **o gate e marginal e a PRIMEIRA execucao depois
de um build mente**. A mesma maquina, mesmo commit, leu 9,55 ms (HEAD, primeira rodada quieta), 8,54 (sob
carga) e 7,7 (assentada) — o kill e 8, entao o veredicto depende do estado do cache/boost, nao do codigo.
Isso e uma fragilidade real do gate, **herdada**, e o conserto (aquecer antes de medir, ou dar margem
honesta) pertence a quem possui o orcamento de perf do sculpt. Reescrever o numero pra ficar verde seria a
armadilha que a memoria `feedback_frozen_bar_check_the_arithmetic_before_gaming_it` descreve.

## Adendo (mesmo dia): o REACH BOUND do Inflate

O vermelho do sculpt acima me levou a medir a bola, e ela tinha trabalho morto de sobra.

**O achado, contado:** o laço da bola visita todo o disco (~800 offsets a ρ=16) por texel. Mas uma fonte
só contribui onde `a_p² > dq` — e no fixture do kill, **35% dos texels tinham `A(q) = 0`** (nenhuma fonte
no disco inteiro podia contribuir) e apenas **32% dos 73M taps** eram úteis.

**O conserto:** percorrer o disco em ordem crescente de `dq` e parar em `dq >= A(q)²`, onde `A(q)` é o
maior `amount` na caixa do texel. É **exato, não heurística** — todo offset além desse ponto reprova o
mesmo teste in-ball que o laço antigo rodava. `A(q)` sai de um `box_max` separável O(área) (van
Herk/Gil-Werman), com caixa QUADRADA de propósito: ela contém o disco, então o limite é conservador e o
resultado não muda um bit.

**Três detalhes que decidiram:**
- **A ordenação é ESTÁVEL.** O vencedor em `sbuf` é o PRIMEIRO offset a atingir o máximo, e num platô
  plano de `amount` uniforme todas as fontes à mesma distância empatam ao float. Ordem instável = matéria
  vindo de outra fonte = outra COR. (`slice::sort_by` é estável por contrato da stdlib.)
- **O `box_max` tem de ser varrido POR LINHA.** A versão óbvia (uma coluna por vez, com alocação por
  linha) custava **0,85 ms dos 4,3 da bola** — um quinto do orçamento que a função existe pra proteger.
- **Caminho interior separado:** para texels longe da borda o índice da fonte é `qi + off` — sem
  aritmética de coordenada e sem teste de limite, decidido uma vez por texel em vez de 4 comparações por
  tap.

**Ganho, medido de forma determinística: o passeio admite 20,4% do disco** num traço com falloff (4,9×
menos), e **zero** em vizinhança não-tocada. ⚠️ **Gateado por CONTAGEM, não por cronômetro** — ver abaixo.

⚠️ **NÃO ajuda o Filter Layer**, e isso é honesto: ele preenche `amount` uniformemente, `A(q) = 1` em todo
lugar e o corte nunca dispara. Medido lado a lado que também **não regride** (9,77 ms com a mudança vs
10,08/10,02 sem).

Gates: `the_reach_bound_is_exact_the_ball_is_byte_identical_without_it` (oráculo = o kernel shipado COM a
otimização removida; compara altura E argmax, 3 depths, fixture com faixa morta + rampa + saturação +
platô) · `the_reach_bound_admits_only_the_offsets_that_could_contribute` (a contagem). Mutações: limite
frouxo (`amount` do próprio texel) RED · `box_max` sempre 1.0 RED. **Uma NÃO sangra e está registrada no
gate:** trocar por `sort_unstable_by` fica verde — pdqsort coincide neste input; a propriedade repousa na
garantia da stdlib, não no fixture, e caçar um input onde diverge pinaria o algoritmo de ordenação.

### ⚠️ Por que os gates de perf aqui são CONTADOS e não cronometrados

Esta máquina degradou ~3× ao longo da sessão: o kernel **INALTERADO** mediu **10 ms** onde a doc deste
mesmo arquivo registra **3 ms**. E o kill criterion tem outliers de 40+ ms num único move (page-faults de
buffers de 16 MB tocados dentro da janela medida), que sozinhos movem a média em 2 ms. Nenhum número de
wall-clock que eu produzi hoje é atribuível com confiança — por isso o gate novo conta offsets, que é
propriedade da aritmética e igual em toda máquina. O `sculpt_perf_kill_criterion` continua marginal e
flaky; não o reescrevi (não é meu orçamento, e mexer no número seria justamente a armadilha).

## Aberto
- **Cache com chave de versão para os planos.** Hoje são materializados a cada frame de preview GPU (que só
  acontece em `take_preview_dirty`). É deliberado: uma versão teria de rastrear TODA entrada do fold
  (planos por camada, `impasto_depth`, `impasto_composite`, visibilidade de grupo, traço vivo, material do
  pincel) e **o modo de falha de esquecer uma é uma luz velha que ninguém vê que é velha**. Se aparecer no
  profile, o conserto vem COM um gate que prove a invalidação em cada uma dessas entradas.
- Pista parcial para a luz (hoje despacha tela cheia, de propósito — ver acima).
- O resto do backlog do §5 do Painter, inalterado (borda do Inflate, cura do banco, Conserve p/
  Flatten/Fill, perf do Deform não gateada, relevo do papel).
