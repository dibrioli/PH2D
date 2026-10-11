# Bugs do módulo Motion Nodes — registro + soluções

> **Índice** — partido em 2026-10-11 (`doc-split`, sha256: nenhuma linha perdida). Fica aqui a tabela-resumo
> e o #19 (fechado, smoke do dono 11/10); os fechados vivem verbatim em `archive/BUGS_motion_nodes/` (≤ 40 KB cada).
> Não há tabela «Recusas MEDIDAS» neste doc.

| Bug | Onde |
|---|---|
| Bug #1–#4 | [`parte_1`](archive/BUGS_motion_nodes/parte_1.md) |
| Bug #5–#9 | [`parte_2`](archive/BUGS_motion_nodes/parte_2.md) |
| Bug #10–#18 | [`parte_3`](archive/BUGS_motion_nodes/parte_3.md) |
| Bug #19 (ABERTO) | **neste doc** (vivo) |


> Log vivo dos bugs **não-triviais** do sistema de nós (sintoma → causa-raiz → hipóteses que
> caíram → medição → solução → lições). O objetivo não é listar todo fix (o git já faz isso),
> mas registrar os bugs cuja **causa enganava** — aqueles em que a aparência, ou o nó que o
> report acusou, levou o diagnóstico para a pista errada. Cada entrada termina em **lições
> generalizáveis**, para o próximo agente não repetir o erro.
>
> Estado por-wave: os handoffs em [`handoffs/`](handoffs/). O catálogo por família:
> [`89_conferencia/`](89_conferencia/).

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](archive/BUGS_motion_nodes/parte_1.md#bug-1--o-nó-acusado-estava-inocente-o-campo-gateia-o-pulso-e-não-gateia-a-memória) | **"Box inconsistente"** — marcar Invert e desmarcar não devolve o quadro inicial | `field.box` (acusado, **inocente**) + `pulse.counter`/`pulse.sample_hold` (a memória) | ✅ **Fechado — smoke aprovado** (porta `reset`) | 2026-08-10 |
| [2](archive/BUGS_motion_nodes/parte_1.md#bug-2--o-glow-e-a-forma-vivem-nos-dois-lados-do-tonemap-e-o-lod-troca-o-lado) | **"Glow não funciona com shape"** | `fx.glow` (acusado, **inocente**) + a partição de render sprite ⁄ vetor | ✅ **CURADO** (aguarda smoke) — o passe passou a ler a CAMADA | 2026-08-20 |
| [3](archive/BUGS_motion_nodes/parte_1.md#bug-3--o-diagnoser-sabia-e-ninguem-perguntava) | **"Todas as peças paradas"** (cena `=71`, banda 6) | uma cena com um fio a menos — e o INSTRUMENTO que ninguém invocava | ✅ **CURADO** (aguarda smoke) — mais um falso positivo pré-existente do diagnoser | 2026-08-20 |
| [4](archive/BUGS_motion_nodes/parte_1.md#bug-4--o-multiply-não-desobedecia-à-alfa-ele-a-invertia-e-o-gate-media-o-único-ponto-em-que-os-modos-concordam) | **"Shadow multiply não obedece o alpha"** (cena `=84`) | `fx.drop_shadow` (acusado, **inocente**) + o par de fatores do `Multiply` em `ph2d-render` | ✅ **FECHADO — smoke aprovado** (cena `=84`, linha ALFA) — resposta invertida, num gate verde há anos | 2026-08-23 |
| [5](archive/BUGS_motion_nodes/parte_2.md#bug-5--o-editor-de-curva-era-oferecido-numa-onda-que-não-o-lê--e-o-censo-que-o-teria-apanhado-não-podia-vê-lo) | **"Wave curve dos osciladores não está funcionando"** (com foto) | `motion.oscillator` (o MOTOR, **inocente**) + a tabela de gates dele — e mais DOIS nós pelo mesmo mecanismo | ✅ **CURADO** (aguarda smoke, cena `=94`) — o editor aparecia em toda onda e só era lido na `Custom` | 2026-08-24 |
| [11](archive/BUGS_motion_nodes/parte_3.md#bug-11--a-seta-diferente-era-a-única-certa-a-família-do-rig-escreve-o-ângulo-local-na-coluna-que-o-desenho-lê-como-mundo) | **"Em skeleton o último objeto tem direção diferente"** (com foto) | a família do **RIG** (7 nós, 6 cópias do `fk.rs`) — e a seta acusada é a ÚNICA certa | ✅ **CURADO** (aguarda smoke) — `rot` passa a levar o MUNDO, `lrot` o local; 6 provas de mutação; ✅ as cópias numa porta (05/10) | 2026-09-19 |
| [12](archive/BUGS_motion_nodes/parte_3.md#bug-12--o-fio-agia-num-ramo-e-não-no-outro-e-o-cartão-mentia-nos-dois) | **"Number no Strength do Vortex não tem efeito"** + **"Number não aceita negativos"** | o ramo HÍBRIDO do `cook_gpu` (não entregava os valores do fio) + o CARTÃO (desenhava o override) + a faixa `0..40` das forças com sentido | ✅ **CURADO** (aguarda smoke, cena `=128`) | 2026-10-03 |
| [13](archive/BUGS_motion_nodes/parte_3.md#bug-13--o-grafo-abria-com-a-parte-de-cima-escondida-o-enquadramento-media-cartões-e-desenhava-pílulas) | **"O grafo abre com a parte de cima escondida"** | o enquadramento automático do painel do grafo (a tela, acusada, **inocente**) | ✅ **CURADO** | 2026-10-03 |

| [18](archive/BUGS_motion_nodes/parte_3.md#bug-18--um-toque-que-treme-virava-arrasto-no-grafo-e-um-campo-legível-não-respondia) | **"Apertar botões em todo módulo é impreciso e inconsistente"** (da barra de baixo aos campos dentro dos nós) | o despacho da superfície do grafo (`ph2d-editor-core`) + o cartão (`ph2d-panel-motion-graph`) | ✅ FECHADO (smoke do dono 10/10) | 2026-10-10 |
| [19](#bug-19--o-ângulo-do-glow-aparece-só-onde-não-faz-nada) | **O ângulo do *streak* do Glow aparece só onde não faz nada** (achado do agente, ciclo 20, sem report) | `fx.glow` (`PARAM_GATES`) + `motion_bridge_params_visible.rs` | ✅ **FECHADO** — smoke do dono 11/10 (cena `=70`), espécie `ParamGateAway` | 2026-10-10 |
---

## Bug #19 — o ângulo do Glow aparece só onde não faz nada

### O que se vê (achado lendo o código, 10/10, sem report do dono)

O `Streak Angle` do `fx.glow` aparece com o `Anamorphic` em `1` (o halo redondo, onde o ângulo é
a identidade) e **some** com o `Anamorphic` em `≥ 1,5` ou `< 0,5` — exactamente onde ele morde.

### Mecanismo

`ph2d-node-fx-glow/src/lib.rs` `PARAM_GATES = [ParamGate { param: "angle", when: "stretch",
values: &[1] }]`, e o doc-comment ao lado diz *«o `1` é o único valor de `stretch` que ESCONDE o
ângulo»*. Mas um `ParamGate` **mostra** o param quando `round(when) ∈ values`
(`ph2d-node-registry/src/ui.rs` `ParamGate`: *«The allowed `when` values … for `param` to
appear»*; `motion_bridge_params_visible.rs::shows`: esconde se `!values.contains(round(v))`). ⇒ a
semântica está invertida. E é um gate de MODO (`mode_hides`): cruzar `1,5` com o ângulo ligado
por fio é a travessia em que o painel trata a row como *«não existe»*.

### A cura (11/10)

O ponto inerte de uma grandeza contínua é uma pergunta que nenhuma das três espécies sabia fazer
(os gates combinam por E, então dois `Above` não dão «≠ 1»). Quarta espécie no registo:
`ParamGateAway { param, when, from }` (`ph2d-node-registry/src/ui.rs`), com a lei num método só
(`shows(v) = v != from`) — a desigualdade EXACTA, a mesma do braço literal do núcleo
(`ph2d-bloom` `upsample_basis`: `s == 1.0`). Contínua, logo nunca `mode_hides`. O `fx.glow`
regista-a; os quatro leitores de espécies a conhecem (`Visibility::shows`, a sonda de alcance,
a sonda dos escondidos, a nota do tutorial). Fechar sem gate (saída b) deixaria um controlo morto
no valor de omissão.

**Gates** (`motion_bridge_params_visible_tests.rs`): o painel e o NÚCLEO concordam em
`stretch ∈ {min; 0,5; 0,95; 1; 1,05; 2; max}` — o oráculo é a base da tenda calculada com dois
ângulos, não a tabela (1 escondido, 6 visíveis) — e o censo de toda espécie de ponto do catálogo
(esconde no ponto, mostra nos dois extremos, ponto dentro da faixa). A cena `=70` já dizia ao dono
*«ele só aparece quando o Anamorphic sai de 1»*: até hoje ela ensinava o contrário do que se via.

### Lição

**Um doc-comment que descreve o que um gate ESCONDE ao lado de uma estrutura que declara o que ele
MOSTRA é a mesma família de «a nota descreve onde a tradução aconteceria»** — a prosa e o dado
dizem o contrário, e nenhum teste perguntou ao painel.

