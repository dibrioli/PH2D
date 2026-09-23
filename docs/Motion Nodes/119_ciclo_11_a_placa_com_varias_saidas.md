# 119 — CICLO 11: a placa com VÁRIAS saídas (2026-09-23)

> Ordem do dono: *«siga»*, depois do smoke `=15` aprovado. A fila ([doc 103](103_dinamica_dos_ciclos.md)
> §5) tem o **10** com as waves técnicas fechadas (a medição dele está no [doc 117](117_o_que_falta_2026-09-22.md)
> §2: não sobra dívida) e o **11** aberto. O [doc 117](117_o_que_falta_2026-09-22.md) §5–§6 nomeia o
> alvo único e medido do 11: **a cerca de ÂMBITO que manda para a CPU todo grafo com mais de uma
> saída** (`motion.sinks.len() != 1` no `cook_gpu`), com o titular corrigido — *não «83 cenas
> presas», e sim «uma cerca que hoje prende uma cena medida (`=107`, `32 762` linhas) e todo
> documento grande que o artista venha a fazer»*.
>
> ⚠️ Este ciclo **não tem tutorial** (é optimização, como o 10 — doc 116 §1).

## §1 — O que já existe (medido no código, não lembrado)

| peça | estado | onde |
|---|---|---|
| o lowering escreve num DESLOCAMENTO de um buffer partilhado | ✅ **inerte** (o único chamador passa `0`) | `GpuCook::encode_lowering`, commit `b4637ecd2` |
| a MISTURA de um sink chega à pipeline por RUN | ✅ | `GpuTexRun::blend` + o `set_pipeline` DENTRO do laço de runs (`a_mistura_do_device_chega_ao_pixel`) |
| a AMOSTRAGEM de um sink chega à pipeline | ⛔ **NÃO** — o desenho do buffer liga `material_bg(texture_id, 0)`: o `Filter` do sink é ignorado na rota da placa | §3, **W1** |
| o planeador conhece MAIS de um sink | ⛔ não — `plan_driven(.., sink)` parte de UMA raiz, e o `cook` lê o sink como a última etapa | **W2** |

## §2 — As waves

| wave | o quê | prova |
|---|---|---|
| **W1** | a AMOSTRAGEM do sink chega ao pixel na rota da placa (`GpuTexRun::sampling`) | pixel: uma imagem `PRETO \| BRANCO` ampliada, com e sem `Nearest` |
| **W2** | o PLANO da UNIÃO: N sinks, os nós partilhados encenados UMA vez (um nó com laço `pre` não pode avançar duas vezes por tique) | o plano de dois sinks contra os dois planos de um |
| **W3** | o COZIMENTO de N sinks: a reserva do total, N lowerings em deslocamentos, os runs de cada sink concatenados | paridade CPU × placa num grafo de dois sinks |
| **W4** | a ORDEM entre sinks: a rota da CPU ordena as linhas de TODOS os sinks por textura; a da placa desenha na ordem do buffer — medir e escrever a lei | um gate que as duas rotas partilham |
| **W5** | a medição (`=107` e o censo de rota) + o smoke do dono | o relógio, a máquina calma |

## §3 — W1 FECHADA (2026-09-23): o `Filter` do sink na rota da placa

**O defeito, achado a ler o desenho do buffer para desenhar a W3:** o laço dos runs do `gpu_extra`
resolvia a textura por `material_bg(r.texture_id, 0)` — com o comentário *«Motion instances carry the
default sampler (word 43 = 0)»*, que é falso desde que o lowering passou a escrever o `sampling` do
sink na palavra 43 (doc 89 folha 17). ⇒ **o `Filter` de uma saída cozida na placa não fazia nada**: a
mesma cena era nítida pela CPU e borrada pela placa, que é a rota de omissão. É a espécie do `CLAUDE.md`
§5.0 *«o consumidor que PROJECTA o valor fora»*, e o molde da cura é o da mistura, um campo ao lado.

- `GpuTexRun::sampling` (a MESMA chave da `RenderInstance`), escrito pelo `texture_runs_from_boundary`
  a partir do estilo do sink; um sink com amostragem ≠ de fábrica emite um run EXPLÍCITO mesmo quando
  toda a textura é o átlas — a partição vazia é *«o átlas, em `Mix`, com o sampler do projecto»*.
- O desenho garante o grupo de amostragem de cada run da placa (átlas **e** individuais), pela
  MESMA varredura que já o fazia para os runs da cena, e liga-o por `material_bg(texture_id, sampling)`.

**Gates:**

| gate | onde | o que afirma |
|---|---|---|
| `o_filtro_do_sink_chega_ao_pixel_pela_rota_da_placa` | `ph2d-render/tests/it/a_amostragem_do_device_chega_ao_pixel.rs` | o xadrez 2×2 do irmão da CPU, desenhado pelo **buffer** da placa: `Nearest` lê o texel preto e `Linear` clareia-o. A instância leva **sempre** a chave de fábrica, para o gate medir o RUN e não um acaso |
| `a_amostragem_do_sink_viaja_e_tira_o_atlas_do_ramo_vazio` | `ph2d-gpu-cook/src/tex_runs.rs` | todo run leva a chave do sink, e uma cena toda de átlas com `Nearest` emite o run explícito — com o CONTROLO (`sampling = 0` continua vazio, byte a byte) |

**Mutação 4 de 4 a sangrar** (com controlo sobre o próprio filtro — um filtro que casa zero testes lê-se `FILTRO VAZIO`, nunca «sobreviveu»): o laço dos runs a ligar o sampler `0` · a varredura que garante os grupos a não ver os runs da placa (o run é **saltado** e o pixel fica no fundo) · só a mistura a tirar o átlas do ramo vazio · o run sem a chave.

## §4 — W2 FECHADA (2026-09-23): o plano da UNIÃO

**A lei:** as N saídas partilham **uma caminhada** do planeador (`plan_driven_many`), logo um nó
que duas saídas lêem é encenado **uma vez**. «N planos de um sink cozinhados lado a lado» correria
esse nó N vezes por tique — e num nó que alimenta um `pre` isso é **avançar a simulação N vezes**.

- **Com UMA saída a união É o plano de sempre, byte a byte**: a `plan_driven` delega nela, o que
  põe todos os gates de paridade da crate a guardar o caso de uma saída.
- `GpuPlan::sinks` — as saídas ENCENADAS, pela ordem pedida. ⚠️ Uma saída que a placa não pode
  encenar **não está lá**: fica em `boundaries` como `(sink, 0)`. *Pedir uma saída não é garantir
  que ela chega à placa*, e a W3 compara as duas listas antes de escolher a rota.
- A cerca do sufixo (`suffix_changes_count`) pergunta a **cada** saída — a partição de texturas de
  uma desalinha-se pelo sufixo dela, e a 1.ª redacção perguntava só ao último estágio.
- ⚠️ **A ordem das saídas pode mudar o plano, e é declarado:** um `pre` só se aceita se a fonte já
  foi encenada, logo um nó da saída B que lê o laço da A encena-se se A foi caminhada antes, e é
  fronteira se não (e aí o recuo devolve o laço à CPU). Os dois são correctos; o chamador passa a
  ordem do DOCUMENTO, que é estável.
- ⚠️ **O recuo do laço é da UNIÃO e é conservador, declarado com gate:** um laço em A e uma
  fronteira temporal em B devolvem o laço de A à CPU, porque o recuo pergunta *«há fronteira não
  estática?»* sobre o plano inteiro — a mesma pergunta que o plano de um sink já faz sem perguntar se
  a fronteira alcança o laço. Se o recuo aprender alcançabilidade, o gate reprova à vista.
- Os pontos de entrada saíram do `plan.rs` (no tecto de 700) para o irmão `plan_uniao.rs`, pela
  costura que lá estava: *o que um nó é* fica, *que saídas se pedem e quando o laço recua* sai.

**Gates** (`ph2d-gpu-cook/tests/it/plan_da_uniao.rs`, sem device): com uma saída a união é o plano
de sempre (incluindo o sink que a placa não encena) · duas cadeias independentes dão a união dos
dois planos, topológica e pela ordem pedida · **o laço partilhado é encenado uma vez** (com o
controlo: os dois planos de um sink contêm-no duas vezes) · a saída que a placa não encena fica
fora das `sinks` · o recuo é da união e o re-plano mantém as duas saídas · a cerca do sufixo
pergunta a cada saída, pelas duas ordens.

**Mutação 5 de 5 a sangrar** — ⚠️ a 5.ª (re-planear o recuo só com a primeira saída)
**sobreviveu primeiro**: o gate do recuo afirmava só que o laço saía, e ficava verde com a saída B
apagada do plano. Hoje afirma as duas saídas e as duas fronteiras.

## §5 — W3, metade da PLACA (2026-09-23): `cook_many`

`GpuCook::cook_many(.., styles)` — um estilo por `GpuPlan::sinks`, na mesma ordem — baixa as
saídas **uma a seguir à outra no MESMO buffer**; o `cook` de sempre delega nele com um estilo só
(⇒ os gates de paridade da crate guardam o caso de uma saída, e correm verdes: 301 + 50, com a
única vermelha a ser a pré-existente `value_slope`, `1,05e-4`, a mesma de sempre). A baixa saiu do
`lib.rs` (a 6 linhas do tecto) para `saidas.rs`.

- ⛔⛔ **A reserva do total vem ANTES da 1.ª escrita** — crescer o buffer substitui-o e apagaria as
  saídas já escritas, em silêncio.
- ⚠️ **O deslocamento de cada saída é o que FOI escrito**, não a soma das contagens: a lei do dono
  (`so_com_forma`) cala uma saída sem forma, e somar a contagem dela deixaria lixo entre as vizinhas.
- ⚠️ **Uma vaga de uniform por baixa**: a 1.ª na de sempre, as seguintes para lá da faixa das
  compactações — duas escritas na mesma vaga chegam ambas à última (`write_buffer` é à submissão).
- ⭐ **A partição cobre o buffer inteiro:** se nenhuma saída pede um run fica vazia (o caminho de
  sempre); se uma pede, todas passam a ter — a de partição vazia ganha o run de átlas em `Mix`,
  senão o ramo dos runs do desenho a saltava inteira.
- ⭐ **Cada saída lê as texturas da SUA fronteira** (`GpuPlan::lineage_boundary`, a mesma caminhada
  da porta 0 da cerca do sufixo): com uma saída a lei de sempre (a fronteira acha-se pelo
  comprimento), com várias duas fronteiras do mesmo comprimento seriam indistinguíveis.
- `GpuCookError::SinkStyleMismatch` (append-only): estilos que não batem com as saídas encenadas.

**Gates** (`gpu_cpu_parity_varias_saidas.rs`, adaptador real): duas saídas (a 2.ª em `Add`) dão o
buffer da CPU linha a linha e a partição cobre as duas faixas · uma saída calada no meio não deixa
buraco · cada saída lê as texturas da sua fronteira (duas fronteiras do MESMO comprimento).
Mais os dois de unidade da junção das partições. **Mutação 5 de 5 a sangrar** — ⚠️ a 4.ª redacção
da mutação do deslocamento mexia na partição e não no deslocamento, e «sobreviveu»: *uma mutação
que não toca a propriedade lê-se como um gate cego*; reescrita, sangra.

⏳ **Por ligar no produto** (a ponte ainda recusa mais de uma saída) — e antes de a ligar, o W4:
**a rota da CPU ordena as linhas do Motion por `(sub_order, texture_id, sampling)` de forma
estável, e a placa desenha pela ordem do buffer.** Com saídas de texturas ou filtros diferentes, a
sobreposição mudaria de rota para rota.

## §6 — W4 FECHADA (2026-09-23): a ORDEM entre saídas

⭐ **A lei é a da CPU, e a placa passa a obedecer-lhe:** o `sort_render_order` da CPU é um sort
ESTÁVEL por `(âncora de recorte, z_order, sub_order, texture_id, sampling)`, e a placa desenha os
runs pela ordem da lista ⇒ `saidas::ordenar_como_a_cpu` ordena os runs, de forma estável, por
`(texture_id, sampling)` quando nenhuma saída pede `Draw Order: Stream`.

- ⛔⛔ **Isto corrige também um defeito PRÉ-EXISTENTE de UMA saída:** com texturas misturadas a
  placa desenhava `[7, 9, 7]` onde a CPU desenha `[7, 7, 9]` — a sobreposição mudava de rota
  para rota sem nenhum multi-sink no meio.
- ⛔ **`stream_order` com mais de uma saída NÃO é reproduzível** (a CPU entrelaça as linhas das
  saídas por índice; a placa só sabe ordenar faixas) ⇒ `GpuCookError::OrdemEntreSaidas` no motor
  e `RECUSA_ORDEM_ENTRE_SAIDAS` na ponte, com a porta pura `ordem_reproduzivel` lida pelos dois.
- Gates (`gpu_cpu_parity_varias_saidas.rs`): a placa desenha pela ordem da CPU · a ordem por linha
  em duas saídas é recusada. ⚠️ A régua da ordem compara POSIÇÕES entre motores com `EPS` (ruído de
  ULP) e o CONTROLO compara a CPU consigo mesma. A mutação que apagava a recusa **sobreviveu** até
  existir o gate dela.

## §7 — W3, metade da PONTE (2026-09-23): a cerca do multi-sink SAIU

`cook_gpu` planeia a UNIÃO (`plan_driven_many(.., &motion.sinks, ..)`), recusa se o plano deixou
alguma saída como fronteira (`RECUSA_SAIDA_FORA_DA_PLACA`) e coze com um estilo por saída
(`cook_many(.., &estilos)`, as TRÊS chamadas). ⚠️ **A `=107` (o interruptor preguiçoso) só corria
na CPU por ter uma segunda saída** — a âncora que a cena pendurava para isso saiu, e ela passa a
**pedir** a CPU pelo nome (`MotionState::cpu_pedida`), com a razão dita no log de rota: ela ensina
um modo SÓ da CPU (*Skip Unused Inputs*).

⭐⭐⭐ **E a varredura que a torna honesta** (`as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da`,
`#[ignore]`, adaptador real): corre a MESMA ponte do quadro sobre **toda cena de demo com mais de
uma saída**, em estados GÉMEOS, e compara a coluna `P` de cada saída (placa contra as tomadas da
bomba) e a ordem do desenho (runs contra o `sort_render_order`). **Resultado: 59 cenas julgadas,
`19 798` posições comparadas, 22 recusadas pela ponte com a razão nomeada.** Piso nas duas
grandezas (`≥ 20` cenas, `≥ 1 000` posições). ⚠️ A barra de cada saída sai do que a corrente DELA
tem e é a que o gate irmão já DECLAROU — tabela `Custom` (`EPS_REL_LUT`, medido `6,65e-4`),
inundação do Voronoi (a banda do `full_relax`, relativa ao lado), o resto `1e-3` absoluto.

## §8 — O que a varredura ACHOU: TRÊS defeitos de produto que a cerca escondia

⛔⛔⛔ **Nenhum dos três é do multi-sink** — a cerca mandava para a CPU os únicos documentos que os
exercitavam, e o artista podia chegar a cada um com UMA saída.

**(a) A cache de pipelines ignorava a VARIANTE do kernel.** A chave era `(tipo, presença)`, e um
nó que escolhe o kernel por param (`variant_by_param`: o `channel` do `motion.noise`, do
`oscillator`, do `drive`) compila um módulo POR VARIANTE ⇒ duas variantes com as mesmas colunas
davam a mesma chave e a segunda recebia o pipeline da primeira. Medido: dois ruídos `Y`/`XY` no
mesmo plano → o segundo a **`0,335`** da CPU; e, com UMA saída, **trocar o canal no painel com a
cena na placa não mudava nada** (a cache persiste entre quadros; `0,26`). Cura: `PipelineKey` com a
identidade dos `&'static` da variante (`O(1)`, ponteiros iguais ⇒ conteúdo igual), e enum porque o
mapa é partilhado com os mapas de redução (hash de conteúdo). Gates:
`a_variante_entra_na_chave_do_pipeline` (as duas ordens no mesmo plano · a troca entre quadros).

**(b) A Voronoi da placa não tinha a MÉTRICA.** O `metric` (Euclidiana/Manhattan/Chebyshev, doc 89
folha 01) chegou à CPU e não à inundação: um Voronoi em Chebyshev saía **redondo pela placa e
quadrado pela CPU**. Cura: `GpuAlgorithm::LloydVoronoi::metric_param` (campo apendado), o valor na
vaga livre do uniform e a MESMA distância no passo da inundação; a escada dos valores é uma, com
`const assert!` no nó. Medido: a atribuição texel a texel diverge **`0`** da CPU nas duas métricas
novas e um passo de Lloyd fica a `1e-6` (a mesma barra do gate da Euclidiana). Gate
`the_metric_reaches_the_device`, com o CONTROLO primeiro (as métricas têm de mudar `≥ 5 %` dos
donos na CPU, senão a fixtura não contém o fenómeno — medido `9,4 %`).

**(c) O L-System em `Branches` saía como QUADRADOS.** A fita é um `geometry_id`, que a placa não
desenha, e a cerca da forma viva perguntava pelo TIPO (o `source.shape`, o `source.text`); o
L-System só é forma viva num modo. A `=108` desenhava cinco quadrados onde a CPU desenha cinco
plantas, e qualquer `lsystem → move → output` do artista já caía nisto. ⛔ **Não se marcou o tipo:**
a bandeira de tipo tem outros dois leitores (a lei da aparência e a das fontes de posições), para
quem um L-System em `Lines` é uma fonte de POSIÇÕES. Cura: canal de registo NOVO e apendado
(`register_live_vector_source_when` / `emits_live_vector`, a pergunta de INSTÂNCIA), o predicado é
do NÓ (`ls::desenha_ramos`), e a regra do modo passou a ser UMA porta (`ls::geometria_e_ramos`) —
ela vivia escrita à mão na shell, e a cerca seria a terceira cópia. A ponte recusa ANTES de
planear (`forma::desenha_forma_condicional`, com os params pela escada inteira). Gates sem placa:
`a_fita_do_lsystem_fica_na_cpu_e_os_segmentos_nao` · `a_bandeira_de_tipo_do_lsystem_nao_mudou`, e
o de ORDEM das recusas (`the_gpu_cook_recusal_placement`) ganhou a agulha.

**Mutação 6 de 6 a sangrar** (a chave sem a variante · o nó sem a condição · a métrica fora do
uniform · o shader sem Chebyshev · a ponte sem a cerca · o predicado sempre falso), com controlo de
filtro vazio no arnês. Portão: `nextest-impacted` `20 171 / 20 174` com os três vermelhos
resolvidos — dois censos que a wave acordou (a razão nova da rota é texto de diagnóstico, isenta
pelo mesmo mecanismo das irmãs; a cerca lê a escada sem cunhar chave ⇒ lista `LEITORES` no censo
das membranas, com a metade que a impede de ser licença) e a flake conhecida
`the_cost_of_sampling_a_path_is_flat_in_its_anchors` (3 de 3 verde sozinha a `load 76`–`85`, zero
diff na crate).

⏳ **Falta a W5:** a medição (o relógio da `=107` e o censo de rota refeito com as curas) e o smoke
do dono.
