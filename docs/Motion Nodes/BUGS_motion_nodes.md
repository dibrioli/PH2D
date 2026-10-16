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
| [20](#bug-20--a-118-pânicava-na-placa-o-trail-acusado-era-inocente-o-anel-mat4-do-slit-scan-lido-pelos-cartões) | **A `=118` crasha na placa ao quadro 12** (`attr.rs:456`, `144` contra `36`) | `motion.trail` (acusado, **inocente**) + a leitura dos cartões (`ph2d-gpu-cook` `tap.rs`) | ✅ **CURADO** (aguarda smoke, tutorial 07) | 2026-10-15 |
---

## Bug #20 — a `=118` pânicava na placa: o Trail acusado era inocente, o anel `Mat4` do Slit Scan lido pelos cartões

**Sintoma.** `PH2D_GPU_COOK_DEMO=118` morre ao quadro 12 do app (`column length must equal stream
element count`, `144` contra `36`); com `PH2D_GPU_COOK=0` corre. O `144 = 4 · 36` apontava para o
`Trail` (ecos empilhados) — falso.

**Mecanismo (backtrace do app, perfil `smoke` com `line-tables`).** `dispatch` → `readout::take_tap` →
`GpuCook::tap_sem_espera` → `le_tap` (`tap.rs`). O `Slit Scan` no dispositivo guarda o anel em quatro
colunas `ss_ring*` de `Dim::Mat4` (16 números por elemento, `438bd1a49f`); o `codifica_tap` amostrava
TODAS as colunas do stream e o `le_tap` decodificava toda dim que não fosse `Scalar/Vec2/Vec3` pelo
braço `_ => Vec4` (`3608e67a47`): `36 × 16 / 4 = 144` vec4 num `Stream` de `36`. É do `main`
(medido: o gate vermelho no `0aab2c1cb6`, com o mesmo `144/36`), não da linha.

**Por que nenhum gate via.** Os gates da `=118` cozem pela CPU; os de placa das cenas (`=140`, `=142`)
são por cena e não há varredura das cenas de ciclo na placa; e o pânico vive na LEITURA dos cartões,
fora da porta do quadro — um varrimento só por `o_quadro` ficou VERDE (medido) até ler como o `dispatch`.
A paridade do slit-scan lê o anel pela `read_column` (debug), nunca pelo tap.

**Cura.** O tap só encomenda colunas que um `Column` exprime (`cabe_numa_coluna`); o `le_tap` fica
exaustivo. Gates: `toda_cena_de_ciclo_coze_pela_placa` (as 23 cenas de ciclo, `placa:`) e a leitura
dos cartões na paridade do slit-scan (`ph2d-gpu-cook` `tests/it`), vermelha com a cura revertida.

**Lição.** Um braço `_ =>` num decodificador de dims é uma resposta para o tipo que ainda não nasceu
— e ele nasceu (`Mat4`) num kernel sem que o leitor soubesse.

**✅ As formas — CURADO em `c6187829cb` (15/10).** *Origem:* a cena nasceu em `fb55eab867` (16/09)
e a lei «só com forma» ligou-se em `72cdfa6f71` (19/09) — desde então os seis panos eram o gizmo de
pontos nas duas rotas: nenhum laranja, nenhum arco-íris, rastos ilegíveis. As figuras do tutorial
07 nunca o mostraram porque são DADOS do motor (a coluna `tint`), não fotos. *Cura:* cada pano é
`Grid → Move → Duplicator(Shape)` — um quadrado verde de semi-lado `0,05` (o lado `0,1` de sempre),
saturado para o `Tail Hue Shift` ter matiz — e os cartões de aparência vêm DEPOIS do carimbo.
*Família* (sonda `sonda_formas_das_cenas_de_ciclo`, 23 cenas): só a `=118` tinha sinks com cartão de
aparência sem forma; sem forma nenhuma mas sem prometer cor ficam `=111`, `=112`, `=113`, `=116` e
4 dos 6 panos da `=119` (§0.8 pede formas em todo smoke — fora deste bloco). *Gates:*
`toda_cena_de_ciclo_que_pinta_aparencia_desenha_formas` (pelo grafo, piso + controlo `=111`) e
`todo_anuncio_que_promete_cor_tem_formas_em_todo_pano` (pelo texto). Fotos CPU e placa: laranja,
arco-íris, rastos verdes em arco, a onda de baixo a andar nas duas.

**Os marcadores parados na placa — FECHADO em `de58233ef3`** (visto na foto da cura, 15/10, 12 s de
Play: na PLACA o gizmo ficava na pose de repouso, sem os arcos do rasto nem a onda de baixo).

*Mecanismo (medido).* Na rota inteira na placa a bomba não marcha, e as tomadas dos gizmos
(`pump.tap_streams()`, a porta dos QUATRO: pontos, warp, colisor, pivô) vinham do
`MotionCookPump::cook_taps_only` (`motion_bridge_gpu.rs`, o braço `FullyGpu`) — a CPU a cozer o sink
no playhead sobre um estado que **nunca marchou**. Sem estado dá certo; com estado não: sonda
`grelha 3×3 → oscilador → motion.trail → saída`, `72` pontos na CPU contra `9` na placa (o rasto sem
ecos); `grelha → oscilador` igual nas duas. ⚠️ A suspeita de que `=111`/`=112`/`=116` o reproduziam
caiu: são sem estado, e mediram placa × CPU = `0` antes da cura (a `=118` era a única cena com
`Trail`/`Slit Scan` sem formas). Origem: `main` (o `cook_taps_only` de 08/09 e as tomadas do gizmo de
19/09; a linha não toca nesses ficheiros desde `0aab2c1cb6`).

*Por que nenhum gate via.* `the_fully_gpu_route_cooks_the_taps_it_never_marched` mede UM cozimento no
tique `0` e pergunta se a tomada EXISTE — nunca se ela anda nem se é a da placa; os de paridade cozem
uma rota de cada vez.

*Cura.* A leitura sem espera (`ph2d-gpu-cook` `tap_voo`) ganha TOMADAS por nome, cada uma com o seu
tecto (`TECTO_DA_TOMADA` = `MAX_PONTOS`, o mesmo passo `i·count/n` do gizmo), codificadas no
`submit` do cozimento; a ponte pede-as antes e põe-nas na bomba — **um quadro atrás**, como os
cartões. Só um nó que a placa não encenou volta à CPU. **Família, o mesmo mecanismo** (ler a CPU
num quadro da placa): o cartão (`readout::stamp`), as colunas (`columns::at`) e o valor dirigido
(`params::stream::driven_value`) liam o memo da CPU PRIMEIRO — e ele não está vazio, guarda o último
quadro da CPU; agora a leitura da placa ganha. A sonda do canvas (`edit::sample_probe`) já lia a placa.

*Preço (release, `32 768` linhas no sink, três variantes intercaladas no mesmo processo, mínimo de 9
blocos, `load ~22`):* leitura da placa `0,229 ms` · sem gizmo `0,193 ms` · a CPU antiga `0,257 ms` por
quadro — a cura custa `+0,036 ms` e é mais barata que o que substitui. Lê `4 096` linhas por quadro.

*Gates.* `os_marcadores_andam_na_placa_como_na_cpu` (`placa:`; com o `cook_taps_only` de volta fica
vermelho, `9` contra `27` pontos), `num_quadro_da_placa_o_cartao_le_a_placa_e_nao_o_memo_velho` e
`a_leitura_da_placa_ganha_do_memo_velho_da_cpu` (três mutações, as três sangram). Foto: `=111` pela
placa, esperas 8 e 12 — os pontos mudam de pose.
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

