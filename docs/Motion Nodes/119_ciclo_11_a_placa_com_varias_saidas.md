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
