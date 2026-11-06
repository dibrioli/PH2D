# SFX Lab — plano da linha `line/SfxLab`

Módulo de criação de efeitos sonoros para games, **sem IA** (dono, 05/11). «SFX Lab» é nome provisório.
Base: [crítica de design de áudio](pesquisa/critica.md) (recomendações aprovadas pelo dono em 05/11) e
[pesquisa](pesquisa/relatorio.md). **Estado: W0 e W1 com smoke do dono aprovado ([handoff](handoffs/HANDOFF_CONTINUACAO_W0_W1_2026-11-05.md)); W2 feita, à espera do ouvido do dono ([handoff](handoffs/HANDOFF_CONTINUACAO_W2_2026-11-05.md), [números](03_W2_medicao.md)); [handoff de integração](handoffs/HANDOFF_INTEGRACAO_line_SfxLab_2026-11-05.md) escrito, integração NÃO ordenada. Motor: [ADR-0198](../architecture/decisions/0198-o-motor-do-sfx-lab-e-kernels-proprios-sobre-det-math.md).**

## 1. Estado da arte

Blender/AE/Illustrator/Rive não fazem isto: a referência aqui são ferramentas de áudio
([tabela e fontes](pesquisa/notas/ferramentas_e_ux.md)).

| Quem | O que faz, e por quê | O que levamos |
|---|---|---|
| jsfxr / ChipTone | Som em segundos; o artista julga sons, não botões | Sortear · mutar · fixar; recusamos o teto retrô |
| Synplant 2 | Distância do arrasto = tamanho da mutação | A mecânica, sem a planta |
| GameSynth / DSP Motion | Gesto comanda o som | Curvas gravadas sem botão, só para o que se move |
| Unreal MetaSounds | Grafo de som dentro do motor | Prova que «dentro do motor» já existe: o diferencial tem de ser outro |
| gdfxr | Receita de ~100 bytes como asset | Receita como fonte; aqui com áudio aprovado guardado |
| Wwise / FMOD | Conjuntos, regra de disparo, parâmetro do jogo | Regra de disparo, sem ligação direta |
| Krotos | Gravação contínua, performance | Gravar sempre |

**Tentado e abandonado (evidência fina; a ausência de lançamento não prova abandono):**

| Caso | O que aconteceu | Lição |
|---|---|---|
| Síntese pura como solução geral | Farnell cobriu fogo, água e armas e admite que a qualidade não era alta; o EyePet da Sony modelou o trato vocal e usou 634 formas de onda de gravações | Gravação própria é de primeira classe |
| Híbrido Crackdown 2 | Modelo + resíduo «indistinguishable» em boa parte dos casos | É o caminho que funcionou |
| SOUL | Sem push desde 2022 | Não apostar numa linguagem externa de DSP |
| Knyst, Glicol | Parecem dormentes | Dependência de grafo em Rust é escassa |
| GameSynth | Última versão vista: 2023 | Espaço aberto; não prova nada sobre o produto |
| SoundSeed, Krotos Concept | Estado não confirmado | Não citar como fato |

Oráculos (§0.9), só correndo e só depois da triagem por artefato: `ffmpeg` (`ebur128`) para
loudness e `sox` (`GPL-2.0-only`, executado sem linkar) para pico. Família sfxr como comparação de
ouvido no teste às cegas, sem ler fonte.

## 2. Desenho: uma porta por pergunta

| Pergunta | Porta única | Estado |
|---|---|---|
| Qual amostra toca agora? | `SceneAudio::play` (`crates/ph2d-app-audio/src/scene.rs:99`). Autoplay (`audio_2d.rs:124`) e `PlaySound` (`audio_2d.rs:167`) já passam por ela: o sorteio mora só aqui | Lido |
| Qual é o conjunto do objeto? | Tipo novo `SoundSet` em crate baixa sem FFT, com **nomes de `ProjectSounds`**, nunca caminhos. `VariationSet` (`ph2d-audio-edit/src/variation.rs:124`, com `path`) passa a ser vista dele | Proposta |
| Como soa uma receita? | `render(receita, semente, versão_do_motor) -> PCM`. **A prévia toca o buffer renderizado por essa mesma função**: prévia e arquivo não têm como divergir | Proposta; W0 mede |
| Qual é o som aprovado? | Entrada em `ProjectSounds` + hash blake3 guardado na receita. Motor mudou ⇒ aviso, nunca troca silenciosa | Proposta |
| Quanto mede o volume? | Uma função de loudness (`ph2d-audio/src/dsp`), usada na escuta e na conferência | Conferir em W0 |
| Que efeitos existem? | `ph2d_audio_edit::Effect`: uma lista, nos dois lugares | Lido |
| Desfazer | `ProjectState` por diff, como o resto do editor | Lido |

**Crates (drop-crate, ADR-0075):** `ph2d-sound-set` (tipo, sorteio) · `ph2d-sfx-recipe` (documento: camadas, fontes, envelopes, semente, versão) · `ph2d-sfx-dsp` (kernels + `render`) · `ph2d-panel-sfxlab` (cartões) · `ph2d-app-sfxlab` (aba, serviço). Sem segundo motor de reprodução: o buffer entra no `ph2d-audio` como qualquer amostra. Tela: pilha de camadas sobre régua de tempo, cartões à direita, sem grafo.

**Riscos já vistos no código:**

- `ph2d-audio` declara-se isento da HR-5. O caminho canônico não pode herdar isso. O rack `Effect` entra em `render`, então **a garantia entre máquinas não vale para receita com efeito** enquanto o rack não passar por `ph2d-det-math`. W0 audita. A garantia «o som aprovado nunca muda» não depende disso: o áudio aprovado fica guardado.
- `ph2d-audio-edit` é opcional em `ph2d-app-audio` (`Cargo.toml:26`) e puxa FFT: por isso `SoundSet` não pode morar nela.
- FunDSP semeia a fase pela posição do nó e usa `libm`. Candidata, não decidida: W0 compara com kernels próprios.
- `deny.toml` recusa dependência git (`:232-234`); FunDSP vem do crates.io e não está no `Cargo.lock` hoje.
- O microfone não existe: zero `build_input_stream` no repositório.
- A catraca `the_shell_only_shrinks` conta o que a aba acrescenta na shell; a cura é mover, nunca subir o número.

## 3. Contrato congelado e schema

**Não encosta em §6.** Provas (05/11, nesta worktree):

```bash
sed -n '/^## §6/,/^## §7/p' CLAUDE.md | grep -c -E "AudioSource2D|VariationSet|PlaySound"   # → 0
git grep -lE 'impl\s+(NodeOp|OpResolver|NodeManifest)\b|\bRasterEditTool\b|\bCanvasPaintTool\b|VectorOp\b' -- \
  crates/ph2d-audio crates/ph2d-audio-edit crates/ph2d-app-audio \
  crates/ph2d-ecs/src/audio_2d.rs crates/ph2d-ecs/src/audio_2d_library.rs | wc -l            # → 0
```

A aba não implementa `Tool` nem `NodeOp`, e o desenho não usa `ph2d-nodegraph`. Repetir os dois greps no fecho.

**Schema:**

| Mudança | Degrau? | Por quê |
|---|---|---|
| `SoundSet` como componente novo | Não | `ComponentBlob` é chaveado por `blake3(nome)`: arquivo antigo só não tem o blob (`registry.rs:489-491`). Binário antigo recusa arquivo novo com o tipo (`:515-517`), como deve |
| Receitas no projeto (W2) | **Sim, um** | Campo novo do arquivo, como `kanbans` (`project.rs:194`). Conta-se com `scripts/schema-recount.py`, nunca à mão; `v216` é só a foto de hoje |
| Marcador em quadro (W1) | A decidir ao ler o formato | Preferir componente irmão; se for campo posicional, degrau |

Censos que precisam do componente novo: `ph2d-component-desc/src/catalog/audio.rs`, `ph2d-app-components/src/property_laws.rs`, `ph2d-app-macros/src/mundo/portas.rs`. Próximo ADR livre hoje: 0198 (contar de novo ao escrever).

## 4. UI: as quatro condições

Sítios-modelo da aba: Kanban (`init.rs:378`, `fase_pointer_subjects.rs:38`, `project_save.rs:77`, `project_load.rs:96`, `project_catalogs.rs:77`) e `DocTab` (`doc_surface.rs:20`). Antes de W3, ler a [receita de cartões](../CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md) e DIRETRIZ §3.B.

| Controle | Componente existe? | Pintado e registrado | Clique chega ao barramento | A sequência leva a |
|---|---|---|---|---|
| Cartão Sound: lista de sons, Pick | Cartão sim; lista não | W1 | W1 | Sorteio audível no Play |
| Create sound | Não | W3 | W3 | Aba aberta já ligada ao objeto |
| Pilha de camadas, régua, onda | Não; base na tira do Flip a conferir | W3 | W3 | Editar a camada muda o som |
| Pico, volume | Mixer sim, reuso a confirmar | W3 | n/a | Reflete o buffer atual |
| Espectro ao vivo | Não | W3 | n/a | Contorno da referência em W6 |
| Roll, Mutate, Lock, Keep, Next | Não | W4 | W4 | Lista Heard, Keep vira versão |
| Explore (`Tab`) | Não | W4 | W4 | Filho vira Current |
| Soltar gravação, Split at hits, Record | Não | W5 | W5 | Parte vira camada |
| A/B, Listen as, Repeat Test, Balance | Não | W6 | W6 | Decisão sobre o conjunto |
| Use in Game | Não | W3 (mínimo), W6 | W3 | **Som toca no Play** |
| Export | Exportação sim; lote não | W6 | W6 | Arquivos com nome do modelo |
| Game Clip | Não | W7 | W7 | Loop com disparos reais |

Prova de cada linha: teste `ph2d-ui-testkit` que dirige o evento real e afirma o efeito observável. Atalhos conferidos contra os globais antes de W3. Texto só em inglês; tokens e i18n (HR-15).

## 5. Waves, gates e fixtures

Gates red-first; cada fixture contém o fenômeno. O som precisa da régua **e** do ouvido do dono.

| Wave | Entrega | Gate vermelho primeiro · fixture | Kill-criterion |
|---|---|---|---|
| **W0 Medição** (uma rodada, §0.10) | ADR do motor; FunDSP × kernels próprios; custo de `render`; bytes entre máquinas; auditoria do rack | Hash do PCM: mesma receita em AMD (aqui), Intel e ARM (Mac/CI). Fixture com ressonância longa, ruído semeado e efeito | Prévia acima de 30 ms (valor proposto, a confirmar no ouvido) para receita de 8 camadas ⇒ cache por camada antes de W3. Hash diverge ⇒ promessa só por máquina, dita ao dono |
| **W1 Caminho até o jogo** | Vários sons por objeto; Random/Sequence/Shuffle/no-repeat; altura e volume por disparo; marcador em quadro; Autoplay verificado | `mil_disparos_nunca_repetem_o_ultimo`; `autoplay_espera_o_play`; `marcador_dispara_uma_vez_por_ciclo`, com loop que cruza o quadro exatamente na volta; projeto antigo abre | Só sai com o caminho sobre `SceneAudio::play`; uma segunda porta reprova |
| **W2 Receita** | Documento, kernels, `render`; camadas de ruído, tom, corpo vibrante, gravação; gravação no projeto | Hash dourado por receita de fábrica; `aliasing` medido com tom agudo varrido; NaN não entra em estado; semente não muda quando o vizinho é editado | Corpo vibrante que o dono reprove no ouvido volta ao desenho antes de W3 |
| **W3 Tela** | Aba, camadas, cartões, medidores, **limitador de proteção**, New sound, Use in Game mínimo | `limitador_nunca_passa_do_teto_na_escuta_e_nao_entra_no_arquivo`, com fixture de ressonância que explode; seam tests da §4; **3 cliques até ouvir no jogo** | Cartões seguem a régua do dono; nada grudado |
| **W4 Exploração** | Sortear, mutar, fixar, oito filhos, mesmo volume, Keep/Next, Heard | `fixado_nao_muda_em_mil_sorteios`; `filhos_tocam_no_mesmo_volume`; `heard_recupera_o_ouvido_ha_cem_sorteios`; `desfazer_cobre_o_sorteio` | **Teste às cegas com profissional aqui.** Se ele não escolher o nosso, resolver material e exploração antes de W5–W8 |
| **W5 Material próprio** | Soltar arquivo, microfone, Split at hits, taxa alta preservada, curva de volume extraída, transformações fortes | `descer_duas_oitavas_preserva_o_que_a_taxa_alta_guarda`, com fixture de 96 kHz com conteúdo acima de 24 kHz; `split_acha_os_golpes` em tomada real; microfone sem alocar na thread de áudio | Qualidade de esticar e mudar altura reprovada ⇒ avaliar kernel antes de seguir |
| **W6 Entrega** | Equilíbrio do conjunto, Repeat Test, A/B no mesmo volume, Listen as, sala de escuta fora do arquivo, categorias do projeto, versões, modelo de nome, exportação em lote e por camada, conferência | Loudness contra `ffmpeg ebur128` em fixture; `sala_de_escuta_nao_entra_no_arquivo`; `variacao_fora_da_curva_e_apontada`; **aprovado não muda quando o motor muda** (versão simulada) | Nenhum |
| **W7 No jogo** | Test in Game (troca no próximo disparo); Game Clip; três forças assadas escolhidas pelo contato | `troca_vale_no_proximo_disparo_sem_clique`; `clip_redispara_na_mesma_ordem_e_forca`; `forca_escolhe_a_camada` | Custo do Game Clip ainda desconhecido: mede-se antes. Imagem cara demais ⇒ sai só com os disparos e os outros sons |
| **W8 Gesto e ao vivo** | Desenhar o som (curvas), Follows, som calculado ao vivo para motor e vento, sem `editor` no jogo (HR-7) | `curva_gravada_e_editavel`; `voz_ao_vivo_nao_aloca` | Sem ganho audível sobre o assado ⇒ não entra |

**Fora:** pintura de som, metáfora de planta, voz ajustando receita, grafo como tela principal, ligação com Wwise/FMOD, plugin, IA. **Depende do dono, sem bloquear código:** biblioteca gravada própria (até lá, material gravado do próprio usuário e nada de presets fingindo qualidade) e o convite ao profissional.

Integração a cada 1–2 waves, só por ordem do dono (W1 vale por si).

## 6. Smoke

**Números medidos** ([handoff W0+W1](handoffs/HANDOFF_CONTINUACAO_W0_W1_2026-11-05.md)): W0 — receita típica de 2 s a 9,6 ms de kernels; pior caso 38,8 ms (> 30 ms propostos, cache por camada antes de W3); hash da típica `e262c4b2a017544c` igual em x86-64, x86-64-v3 e wasm32, Intel e ARM NÃO PROVADO. W1 — mil disparos, Shuffle + No repeat 334·333·333 (0 repetidos), Random + No repeat 327·340·333 (0), Random 327·341·332 (328 repetidos), Sequence 334·333·333 (0); cartão: 64 linhas 1557 µs, 32 marcadores 873 µs. Nenhum som ouvido. Regra: em cada wave a sonda headless corre antes da mensagem ao dono, e os números saem dela. Único número existente, só como ordem de grandeza: 32 ressonâncias a 0,066 % de um núcleo ([notas](pesquisa/notas/tecnicas_de_sintese.md)), Ryzen 9950X sob carga, sem efeitos.

Não existe sonda offline de áudio; ela nasce em W0 (`PH2D_SFX_SMOKE` conta-se no roteador novo). Primeira cena, W1: objeto com três sons bem diferentes, moedas que disparam e Audio Listener já posto. A sonda conta qual amostra saiu em mil disparos; a mensagem traz essa contagem. O dono confere com o ouvido.
