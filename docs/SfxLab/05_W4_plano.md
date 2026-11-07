# W4 «Exploração» — plano

Leitor: a LLM que implementa. Base: `4094f16642` (W3 aprovada pelo dono em 06/11). Plano geral: [01_plano.md](01_plano.md) · tela de referência: [crítica](pesquisa/critica.md) «Explorar» e «Descartar quase de graça». **Estado: plano aprovado pelo dono em 06/11, que delegou as decisões («faça buscando o estado da arte e tome as decisões»); construído; falta o smoke dele.** Sem IA ([memória](../../project-memory/project_sfx_module_no_ai_2026_11_05.md)).

## O que o artista consegue fazer ao fim

1. **Roll (`R`):** troca tudo o que não está travado por um som bem diferente, e toca.
2. **Mutate (`M`):** o mesmo gesto, pequeno; apertar de novo continua andando.
3. **Lock (`L`, cadeado):** trava uma camada inteira ou um campo; mil sorteios não o mudam nem por um bit.
4. **Next (`N`):** joga fora a tentativa e traz outra do mesmo gesto, a partir do mesmo ponto.
5. **Keep (`K`):** marca o som numa lista com estrela, sem diálogo nem nome.
6. **Heard:** tudo o que foi TOCADO fica numa lista gravada com o projeto; clicar ouve, duplo clique traz de volta.
7. **Explore (`Tab`):** o som atual no centro e oito variações em volta, no mesmo volume; `1`–`8` tocam; arrastar para longe muda mais e arrastar de volta devolve; `Enter` faz a variação virar a atual, e nascem oito novas em volta dela.
8. **Ctrl+Z** desfaz qualquer um desses gestos; o som aprovado do jogo nunca muda sozinho.

**Fora da W4:** versões e A/B (W6) · categorias e os quatro controles grandes (W6) · `Make set` (W6) · soltar arquivo (W5) · biblioteca gravada (dono).

## 1. Estado da arte

Base: [ferramentas_e_ux.md](pesquisa/notas/ferramentas_e_ux.md) §Cited Findings, mais a busca de 06/11 abaixo. Blender, After Effects, Illustrator e Rive não têm sorteio de som; deles só vem o idioma `Tab` = modo (Blender, já lei do PH2D).

| Quem | O que faz, e por quê | O que levamos |
|---|---|---|
| Bfxr, jsfxr | `R`, `M`, `L` em uma tecla: o artista julga sons, não botões | As teclas e o laço |
| ChipTone | Randomize total, que «às vezes cria o horror» | Sorteio nunca anda pela faixa inteira (medido abaixo) |
| Synplant 2 | Distância do arrasto = tamanho da mudança; voltar desfaz; replantar faz o galho virar semente | O arrasto reversível e o «vira a atual», sem a planta |
| Evosynth (Yee-King 2016) | 3 552 cruzamentos, 90 salvos: ~2,5 por 100 | Descartar quase de graça |
| McDermott et al. | Avaliar devagar cansa e força populações pequenas | Oito, com triagem por tecla |

**Busca de 06/11, que fechou as quatro decisões** (nada disto é estudo sobre efeito sonoro de jogo; o sinal é de direção):

| Fonte | O que diz | Decisão |
|---|---|---|
| [WaveLab](https://archive.steinberg.help/wavelab_pro/v12/en/wavelab/topics/audio_montage/loudness_matching_c.html), [AB Loudness Match](https://gearspace.com/threads/tbproaudio-ablm-2.1357447/), [Steinberg, som curto](https://forums.steinberg.net/t/feature-request-loudness-meta-normalizer-solutions-for-short-files-samples/1025599) | Nivela só a escuta; mede 400 ms; a medida de arquivo inteiro não serve a som curto | Sorteio nivelado pelo maior momento; `Space` no volume real |
| [Venom](https://community.vcvrack.com/t/randomize-right-click-context-menu/1492/13), [Synerator](https://rekkerd.org/synerator-randomizer-plugin-host-updated-to-v1-1-0/) | Trava por parâmetro salva com o patch; quantidade + trava + desfazer | Travas e Heard no arquivo do projeto |
| [Bfxr](https://www.bfxr.net/) | `R`, `M`, `L` em uma tecla | As teclas; `N` no lugar do `X` da crítica, porque casa com o nome do botão |
| [PLOS One 2024](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0296347) | 9 opções por rodada, 99 avaliações ≈ 40 min; espaço menor rende mais cedo (simulado, som de veículo) | Oito, descarte de uma tecla, sorteio a partir do valor atual |
| [Synplant 2](https://www.musicradar.com/reviews/sonic-charge-synplant-2-review) | Muda mais quanto mais longe da semente; replantar | Arrasto reversível e `Enter` |
| [MUSHRA](https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.1534-1-200301-S!!PDF-E.pdf) | Referência escondida e âncora | §7: a gravação é a referência, o jsfxr a âncora, e os casos repetidos checam a coerência |

Sem profissional no fim da W4, a W5 pode seguir, com o teste marcado antes de congelar a tela.

**Tentado e abandonado:** só um caso tem alguma base: população grande em síntese evolutiva, limitada pelo cansaço de quem avalia (McDermott). Não há registro de abandono do sorteio nem do arrasto. **Recusado por nós, não abandonado por eles:** árvore genealógica e semente como tela (crítica: «tire a árvore e a semente; mostre oito»). **Criticado, sem abandono:** presets sem etiqueta do Synplant. Não achei estudo controlado comparando arrasto, oito opções e sliders em efeito sonoro: quem decide é o ouvido do dono e o teste às cegas.

## 2. Desenho: uma porta por pergunta

| Pergunta | Porta única | Nota |
|---|---|---|
| Como nasce uma variação? | `ph2d_sfx_explore::varia(base, numero, tamanho, travas) -> Recipe`, função pura | Roll e Mutate são a MESMA função em dois tamanhos (crítica). Tamanho 0 devolve a base; o sorteio de cada campo é fixo pela semente e o tamanho só o escala, logo o arrasto volta |
| O que pode variar? | Coluna nova na tabela `Campo`: `variacao` (escala relativa/linear, teto do salto) | Campo novo sem a coluna não compila. Discretos (`OPCOES`: onda, filtro, lei) só no Roll. Duração, cadeado e `approved` nunca |
| Como soa? | `render_cancelavel`, a de sempre | Sem segundo motor |
| Quem rende oito? | `ph2d_sfx_explore::Ninhada`: oito renders paralelos, ticket e cancelamento, entrega conforme fica pronto | `RenderJob` recusado: drena até o mais novo (`job.rs:134`), joga fora sete. Ele segue dono do som atual |
| Que volume tem? | `maior_momento(&[f32])`, janela fixa de 400 ms em `ph2d-audio/src/dsp/loudness.rs` | `integrated_lufs` recusada: divide pelo arquivo, e o silêncio no fim mente (§6) |
| Que volume se ouve? | `escuta_nivelada(pcm, alvo)`: ganho sobre CÓPIA, ANTES de `limitar_escuta` | Ganho depois do limitador furaria o teto. O arquivo e os medidores não mudam |
| Quem é ouvido nivelado? | Só o que nasceu de gesto: `R`, `M`, `N`, Explore, Heard. O `Space` na tela normal toca o volume real | Senão o campo Gain vira controle morto |
| O que foi ouvido? | `SfxLibrary.ouvidos: BTreeMap<nome, Vec<Ouvido>>` (receita, como nasceu, estrela), que entra só por `Sessao::ouviu`, chamada quando a escuta recebe o som | Entra ao TOCAR, sem repetir receita igual. Fora do desfazer de propósito: desfazer não apaga o que se ouviu |
| Desfazer | `Historia` da aba: cada gesto, um passo; o arrasto coalesce (`abre_gesto`) | `Aba::desfaz` já preserva `approved` |
| Teclas | `gestos::tecla`, consumindo `R M L K N 1–8 Tab Enter` | Zero linha na shell: a aba já é dona do teclado |

**Roll parte de onde?** Do **ponto de partida**: o último som escolhido (edição à mão, Keep, Enter no Explore, recuperado do Heard). `R` repetido não anda sem parar até o horror; `M` anda. `N` volta ao ponto de partida do gesto e repete. **Som sem camadas:** `R` pega um som de fábrica por sorteio, para o primeiro clique já soar.

**⛔ Armadilha achada no código:** a semente de cada camada vem de `Recipe::seed` (`layer_seed`, `ph2d-sfx-dsp/src/lib.rs:161`). Roll que mude `seed` muda o ruído da camada TRAVADA. Cura: `Layer.variacao: u64` (0 por omissão, misturada por XOR à semente da receita na chamada de `layer_seed`, e o 0 não muda os hashes dourados); `Recipe::seed` não muda no sorteio, e só as camadas livres trocam a própria.

**⛔ `det-math`:** `exp`/`ln` do `varia` vão por `ph2d-det-math`, e o gerador é o SplitMix64 do motor (hoje `pub(crate)`, `rng.rs`), exposto. O mesmo sorteio dá o mesmo som em qualquer máquina.

**⛔ Adoção:** o `service` adota a receita do mundo que difere da última escrita e zera o histórico (04 §Decisões). Gravar `ouvidos` não pode disparar isso: tem gate.

**Crates:** `ph2d-sfx-explore` (lei, sem tela) · `ph2d-sfx-explore-tela` (pinta a barra, o Heard e a grade, e devolve o que se clica; como o `ph2d-object-picker`, não é painel) · `ph2d-app-sfxlab` só liga a mão (`explorar.rs`). **Teto 2690/2692: a conta soma `tests/` (769 linhas).** Os gates da família foram para `shells/sfxlab-scenes/tests/familia`, sem edição; o tecto da crate não sobe. Apps irmãs só dependem de `ph2d-app-host`: por isso a peça nova não é `ph2d-app-*`.

## 3. Contrato congelado e schema

Não encosta em §6. Corrido em 06/11: `sed -n '/^## §6/,/^## §7/p' CLAUDE.md | grep -c -E "AudioSource2D|VariationSet|PlaySound|SfxLibrary|Recipe"` → **0**; `git grep -lE 'impl\s+(NodeOp|OpResolver|NodeManifest)\b|\bRasterEditTool\b|\bCanvasPaintTool\b|VectorOp\b' -- 'crates/ph2d-sfx-*' crates/ph2d-app-sfxlab crates/ph2d-panel-sfxlab crates/ph2d-sound-set crates/ph2d-audio crates/ph2d-audio-edit crates/ph2d-app-audio | wc -l` → **0**. Repetir no fecho, com as crates novas.

| Mudança | Degrau? | Por quê |
|---|---|---|
| `Layer.variacao`, `Layer.travada`, `Layer.travas`, `SfxLibrary.ouvidos` | Só o blob: `PH2DSFXL` v2 → v3, lendo v1 e v2 | O campo `sfx_lab` do projeto já existe: `PROJECT_SCHEMA` +0 |
| Trava por campo | Guarda `Campo.id` | É `hash_node_id` do nome: renomear um campo soltaria a trava. `LEIS` repete os nomes e `todo_campo_de_camada_tem_lei` reprova o desencontro |

**Teto do Heard (o recurso é o tamanho do arquivo do projeto):** 10 000 por som, o mais velho sem estrela sai primeiro. A 1 por segundo são quase 3 h seguidas no mesmo som. Tabela em §6.

## 4. UI: as quatro condições

Prova de cada linha: `ph2d-ui-testkit`, evento real, efeito observável. Só inglês, tokens, i18n, [receita de cartões](../CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md) e [lei da compactação](../CartoesUI/16_REFERENCIAS_POR_CARTAO.md).

| Controle | Existe? | Pintado e registrado | Clique chega | A sequência leva a |
|---|---|---|---|---|
| Barra Roll · Mutate · Next · Keep · Explore | Não | W4, no topo da aba | W4 | Som novo toca; entra no Heard |
| L na linha de camada | Linha sim (≡ M S) | W4 | W4 | Camada igual em mil sorteios |
| `L` sob o mouse, em campo e em escolha | O cartão não tem ícone por campo: o rótulo diz «(locked)» | W4 | Painel publica o que está sob a mão | Igual em mil sorteios |
| Faixa Heard, estrela, filtro «Kept» | Não | W4, faixa fixa sob a pilha | W4 | Ouvir; duplo clique = atual |
| Grade do Explore, célula, onda, número | Onda sim (`paint.rs`) | W4, no corpo da aba | W4 | `Enter` = atual, oito novas |
| Arrasto na célula, `Change (%)` | Não | W4 | W4 | Muda o tamanho; volta devolve |
| Aviso «Same volume» | Não | W4 | n/a | Diz por que soa diferente do arquivo |
| Medidores | Sim | W3 | n/a | Mostram o ARQUIVO, nunca o nivelado |

`Tab` com campo de texto focado continua do campo. Com a janela do Use in Game aberta, nada disto reage. Com a aba fora de foco, o `Tab` volta ao troca-modo global.

## 5. Gates, vermelho primeiro

| Gate | Fixture que contém o fenômeno |
|---|---|
| `fixado_nao_muda_em_mil_sorteios` | **Camada de RUÍDO travada ao lado de corpo livre**; compara os bytes da camada, não só os campos |
| `filhos_tocam_no_mesmo_volume` | Ninhada de Roll (33,1 dB de diferença medida); ±0,5 dB depois de nivelar, pelo `maior_momento` da saída |
| `nivelar_nao_passa_do_teto_nem_muda_o_arquivo` | Som agudo de pico alto e volume baixo; compara bytes do arquivo |
| `maior_momento_nao_muda_com_silencio_no_fim` | O mesmo som ×1, ×2, ×4 |
| `heard_recupera_o_ouvido_ha_cem_sorteios` | 100 Rolls tocados; recupera o 1.º; hash igual; só o tocado entra |
| `heard_sobrevive_a_salvar_e_abrir` · `gravar_heard_nao_zera_o_desfazer` | Projeto real; o segundo pega a adoção do `service` |
| `desfazer_cobre_o_sorteio` | Roll, Mutate, Enter, recuperar, travar; desfaz tudo, compara PCM; `approved` intacto |
| `sortear_nao_troca_o_som_do_jogo` | Som aprovado + 100 Rolls; o som do jogo igual; barra «Changed» |
| `arrastar_de_volta_devolve_a_base` | Tamanho 0,7 → 0: bits iguais à base |
| `a_mesma_variacao_da_o_mesmo_som` · `refazer_uma_nao_muda_as_outras` | Mesma base e número; receita com três camadas |
| `sorteio_nunca_e_mudo_nem_quebra_o_motor` | As 8 de fábrica × 1 000; nada abaixo de −70 ou não finito; render inteiro cancelável |
| `blob_v2_abre_na_v3` · `hash_dourado_intacto` | Bytes v2 gravados antes da mudança; `conjunto 86ea4a05df14620b` |
| `render_velho_da_ninhada_nunca_entra` | Pior corpo de 120 s, logo seguido de Roll |
| Teclas | `tab_entra_e_sai_e_nao_troca_o_modo`; letras não vazam para o mundo |
| Mutação (agente `mutacao`, 1× no fecho) | Trava ignorada · nivelar depois do limitador · Heard fora do tocar · `variacao` entrando em `seed` |

Censos: `the_shell_only_shrinks` (delta 0), `no_dependency_climbs_a_layer`, `no_magic_numeric`, `TETOS_DAS_CRATES_APP`, tetos de LOC, i18n.

## 6. Smoke e números

Cenas: **`PH2D_SFX_SMOKE=5`** (aba em «Impact», 4 camadas, a camada de ruído travada, objeto com som aprovado para provar que ele não muda) · **`=6`** (a 5 com o Explore aberto, para fotografar). Fotografar cada passo; o som fica com o ouvido do dono, que usa caixas pequenas.

**Medido em 06/11** (sonda descartável `crates/ph2d-sfx-lab/examples/sonda_w4.rs`, apagada; variação simplificada, sem o `varia` real; 8 sons de fábrica; 8 filhos; sementes fixas; mínimo de 7 blocos intercalados; perfil `smoke`; load ~5–9, logo ordem de grandeza). ⚠️ A primeira versão trocava as sementes por bloco e o mínimo escolhia as ninhadas baratas: corrigido.

| Oito variações (ms) | Mutate em fila | Mutate paralelo | Roll em fila | Roll paralelo |
|---|---:|---:|---:|---:|
| Metal Hit, 2 s | 10,7 | 1,6 | 7,3 | 1,8 |
| Impact, 4 camadas, 1,2 s | 10,6 | 1,5 | 9,3 | 1,8 |
| Pior dos 8 de fábrica | 10,7 | 1,6 | 9,3 | 1,8 |

Sons de fábrica cabem até em fila. O paralelo se paga no pior corpo de 120 s (9 370 ms por render, W2): 75 s em fila; é ele que pede ticket e cancelamento. **Não medido:** memória de 8 renders no teto (derivado de 184 MB por render de 8 camadas: ~1,5 GB); o passo 1 mede.

| Como varia | Mudos, de 64 | Maior diferença de volume numa ninhada |
|---|---:|---:|
| 30 % / total pela FAIXA do campo (Mutate / Roll) | **6** / 1 | **63,3** / 52,8 dB |
| ±30 % / até ×4,5 RELATIVO ao valor (Mutate / Roll) | **0** / 0 | **5,3** / 33,1 dB |

Faixa do campo recusada: 30 % de um ataque que vai a 120 s são ±36 s, e o som nem começa. O sorteio relativo é a lei; o tamanho final do Roll passa pelo ouvido do dono.

| Mesmo som, com silêncio no fim | Arquivo inteiro (`integrated_lufs`) | Maior momento |
|---|---:|---:|
| Metal Hit ×1 / ×2 / ×4 | −22,84 / −25,85 / −28,86 | −15,98 nos três |
| Whoosh ×1 / ×2 / ×4 | −22,83 / −25,84 / −28,85 | −19,84 nos três |

| Alvo de escuta | Filhos que passam de −1 dB de pico ao nivelar, de 64 |
|---|---:|
| −14 | 25 |
| −18 | 8 |
| **−23 (proposto)** | **0** |
| −28 | 0 |

Custo por filho: maior momento 0,11 ms; limitador 0,01 ms quando não trabalha. Com 0 de 64 em −23, o limitador quase nunca muda o timbre do que se compara; a amostra é pequena, e o gate cobre o resto.

| Heard, receita de 4 camadas (61–244 bytes por receita de fábrica) | Bytes | Gravar |
|---|---:|---:|
| 1 000 | 235 KB | 0,33 ms |
| 10 000 | 2,4 MB | 3,44 ms |
| 100 000 | 23,8 MB | 38,9 ms |

**Medido em 06/11 com o código final** (`medir_exploracao`, `--ignored`; `varia` real, com medida e limitador; mínimo de 7 blocos; perfil `smoke`; load ~8, logo ordem de grandeza):

| Do pedido à oitava pronta (ms) | Mutate | Roll |
|---|---:|---:|
| Metal Hit, 2 s | 2,43 | 2,72 |
| Impact, 4 camadas, 1,2 s | 2,20 | 2,90 |
| Pior dos 8 de fábrica | 2,43 | 2,90 |

| Outras medidas | |
|---|---|
| Pior corpo sorteado de 120 s | 556 ms até a oitava |
| 8 camadas × 120 s | 450 ms; pico do processo +1 128 MB, abaixo dos ~1,5 GB derivados; cabe na máquina de 8 GiB |
| Heard com 10 000 linhas | Entrar uma: 0,035 ms; copiar para o mundo: 1,38 ms |
| Quadro da tela, CPU, sem GPU | Normal 0,372 ms; Explore 0,310 ms |

O pior caso de 120 s ficou muito abaixo dos 9 370 ms da W2 porque aquele era outro corpo; os 75 s estimados acima não se confirmaram nesta fixture, e o cancelamento continua pelo caso real de W2. **Aberto:** o medidor Loudness segue na medida de arquivo inteiro; ele e o nivelado leem coisas diferentes de propósito até o dono decidir.

## 7. Teste às cegas com profissional (fim da W4, antes de W5)

**Pergunta:** o som feito aqui compete, e o que falta quando não compete: material ou exploração? **Limite honesto:** uma pessoa é sinal, não prova. A régua é escrita ANTES.

**Parte A, às cegas, ~30 min, sem saber do PH2D.** Página offline única (`docs/SfxLab/teste_cegas/`, gerada por script), 12 casos: moeda, pulo, clique de interface, golpe de metal, batida de madeira, whoosh, 2 de cada. Fora: fogo, chuva, água e passos, que dependem de gravação (W5). Cada caso traz A, B, C embaralhados: **nosso** (dono, até 5 min por som) · **família sfxr** (jsfxr, dono, até 2 min, corrido sem ler fonte; licença conferida antes) · **gravação profissional** (biblioteca gratuita com licença conferida). Mesmo formato, mesmo volume (`maior_momento`), −23. Ele ordena do 1.º ao 3.º e responde «usaria no jogo: sim / com ajuste / não». Dois casos repetidos, sem aviso, medem a coerência. A chave fica num arquivo que só abre depois.

**Parte B, à vista, ~20 min, depois da A.** Ele usa a ferramenta em tela compartilhada: «um golpe de espada para o seu jogo, em 10 min». Registro por log: sons ouvidos, tempo até o primeiro Keep, vezes que usou Heard, Lock, Explore. Conversa no fim. Dono fora da sala na A, para não dar pista.

| Resultado (12 casos, preferência par a par) | Leitura | O que se faz |
|---|---|---|
| Ganha do sfxr ≥ 9 e da gravação ≥ 9 | Compete | Segue para W5 |
| Ganha do sfxr ≥ 9, da gravação ≤ 8 | **Falta material** | Biblioteca gravada e W5 sobem na fila; W6–W8 esperam |
| Ganha do sfxr ≤ 8 | Motor ou exploração | Para; com a Parte B acha-se qual, e volta-se antes de W5 |
| Parte B: Keep nenhum ou Heard/Explore ignorados | Exploração | Refaz-se a tela, mesmo com A boa |

Sorte: 9 de 12 ao acaso sai 7,3 % das vezes; 10 de 12, 1,9 %. Casos repetidos incoerentes: leitura vale menos, e dito no relatório. O relatório vai ao dono.

**Do dono:** convidar a pessoa (engenheiro de som de games; sem vínculo com o projeto) · 1h30 para fazer os sons · a aprovação desta régua ANTES · decidir se paga. Sem profissional, W5 fica por decisão dele.

## Ordem

Em série: cada passo usa o anterior; as medições finais vão num lote só (§0.10).

1. Abrir folga no teto (mover gates de tela); `Layer.variacao`, travas, `ouvidos`, blob v3; `varia`, `maior_momento`, `escuta_nivelada`, `Ninhada`, com gates e sem tela.
2. Barra, teclas, Lock, Heard na tela normal; cena 5.
3. Explore, grade, arrasto, `Enter`; cena 6; lote final de medição, fotos.
4. Smoke do dono, `/pd-onda-fechar`.
5. Teste às cegas (§7), com o dono.
