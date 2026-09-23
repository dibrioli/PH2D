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
