# W3 «Lab» — a tela do SFX Lab: plano

Leitor: a LLM que implementa. Base: `bbfefaf393` + a aprovação de W2 (06/11). Plano geral: [01_plano.md](01_plano.md) · tela de referência: [crítica](pesquisa/critica.md) «Tela principal» · números do motor: [03_W2_medicao.md](03_W2_medicao.md). Estado: **à espera da aprovação do dono; nada implementado.**

## O que o artista consegue fazer ao fim

1. Abrir um som novo por **File ▸ New Sound** ou por **Create sound** no cartão Sound de um objeto.
2. Ver o som no tempo: o som inteiro em cima e cada camada com a sua onda, sobre uma régua em ms.
3. Acrescentar, apagar, reordenar, mutar e isolar camadas (Noise, Tone, Struck body, Recording de um som do projeto); arrastar uma camada no tempo.
4. Editar a camada escolhida em cartões à direita e ouvir com **Space** ou, ligado, a cada mudança.
5. Ler Peak, Loudness e Spectrum do som atual; o limitador protege os ouvidos sem entrar no arquivo.
6. Desfazer e refazer qualquer edição.
7. **Use in Game:** em até 3 cliques, o som toca no Play. Motor que mudou avisa; nunca troca o som aprovado em silêncio.

**Fora de W3:** Roll/Mutate/Keep (W4) · soltar arquivo, microfone, Split (W5) · versões, categoria, Listen as, A/B, Export em lote (W6) · Game Clip (W7) · curvas por gesto (W8). O Drum Thump (base 95 Hz) que quase sumiu nas caixas pequenas do dono é caso de uso do Listen as, em W6.

## 1. Estado da arte

Sem pesquisa nova: vale a [tabela do plano §1](01_plano.md) e a [crítica](pesquisa/critica.md), com fontes em [ferramentas_e_ux.md](pesquisa/notas/ferramentas_e_ux.md). Blender, After Effects, Illustrator e Rive não sintetizam som; o que se aproveita deles é o idioma de camadas sobre régua (sequenciador do Blender, timeline do After Effects).

| Decisão | Por quê | Recusado ou abandonado |
|---|---|---|
| Pilha sobre régua | Em som curto a pergunta é «o que entra quando» | Grafo como tela principal: esconde o tempo |
| Cartões só da camada escolhida | DopplerPRO é criticado por não caber numa tela (fonte fraca) | Tudo à vista |
| Receita + áudio aprovado guardado | Crackdown 2 e EyePet: gravação é de 1.ª classe; síntese pura não basta (Farnell) | Síntese como solução geral |

## 2. Desenho: uma porta por pergunta

| Pergunta | Porta única | Nota |
|---|---|---|
| Como soa? | `ph2d_sfx_dsp::render`, que passa a delegar para `render_cancelavel(.., &cancelado)` | O cancelamento fica DENTRO da porta: o hash dourado não pode mudar |
| Quando rende? | `RenderJob` em `ph2d-app-sfxlab`: UMA thread com o `RenderCache`, `submit(receita, ticket)` e `poll`, no modelo de `ph2d-code-format/src/worker.rs:25-73` | `Job` (`progress.rs:248`) recusado: uma thread por job e sem cancelamento |
| O que se ouve? | `escuta(&Pcm) -> SampleData`: limitador offline sobre uma CÓPIA, e `AudioEngine::play_preview` | Sem tocar na thread de áudio. O arquivo nunca passa por aqui |
| Quanto mede? | `integrated_lufs` (`ph2d-audio/src/dsp/loudness.rs:50`) e um pico, sobre o buffer SEM limitador | O medidor mostra a verdade do arquivo, inclusive CLIP |
| Qual é o som aprovado? | `aprovar(lib, receita) -> nome`: grava em `ProjectSounds` e guarda `Approved { sound, blake3, engine }` na receita | Único escritor; motor diferente vira aviso |
| Quem liga ao objeto? | `AudioSource2D.sound` ou uma entrada do `SoundSet`, por uma só função de `ph2d-app-sfxlab` | Com o desfazer da cena |
| Desfazer | Fila da família pela `DocumentSurface::undo`, como o Kanban (`ph2d-app-kanban/src/actions.rs:77`), com instantâneos de `Recipe` | Muda o plano §2: ver abaixo |

**Por que não `ProjectState`:** o blob leva as gravações em PCM, e copiá-lo a cada arrasto paga pelo material que não mudou; a `Recipe` tem poucas centenas de bytes e o material é imutável, endereçado por blake3. Kanban e Code já têm a própria fila, e a shell não ganha sítios. `ProjectSounds` não está em `undo.rs`: desfazer o Use in Game devolve a ligação do objeto e deixa o som no projeto, sem efeito no jogo.

**Crates:** `ph2d-panel-sfxlab` (pintura e cartões) · `ph2d-app-sfxlab` (`DocumentSurface`, serviço, `RenderJob`). A aba pinta a pilha; os cartões usam a `paint_card_list` no retângulo direito da área, como o `Tela` da receita de cartões. Custo na shell: dependência, `install`, um gancho por quadro, medidos contra a folga ~136 (`wc -l` do explorador; recontar pelo gate).

**Limitador:** teto −1 dBFS, com antecipação, em `ph2d-sfx-dsp`, sobre `ph2d-det-math`. Como corre offline, não pode estourar por construção.

**Mudo e solo:** `muted` entra no documento (o render pula a camada, sem mudar a semente das outras); solo é só da escuta e nunca entra no arquivo.

## 3. Contrato congelado e schema

Não encosta em §6. Provas corridas em 06/11 nesta árvore: `sed -n '/^## §6/,/^## §7/p' CLAUDE.md | grep -c -E "AudioSource2D|VariationSet|PlaySound|SfxLibrary|Recipe"` → **0**; `git grep -lE 'impl\s+(NodeOp|OpResolver|NodeManifest)\b|\bRasterEditTool\b|\bCanvasPaintTool\b|VectorOp\b' -- 'crates/ph2d-sfx-*' crates/ph2d-sound-set crates/ph2d-audio crates/ph2d-audio-edit crates/ph2d-app-audio | wc -l` → **0**. Repetir no fecho, com as crates novas.

| Mudança | Degrau? | Por quê |
|---|---|---|
| `Approved` e `muted` na `Recipe`/`Layer` | Só o blob: `PH2DSFXL` v1 → v2, lendo v1 | O `PROJECT_SCHEMA` não muda: o campo `sfx_lab` já existe |
| Registo de família | Não | `ph2d-app-sync` regenera; `TETOS_DAS_CRATES_APP` ganha uma linha |

## 4. UI: as quatro condições

Prova de cada linha: `ph2d-ui-testkit`, evento real, efeito observável.

| Controle | Existe? | Pintado e registrado | Clique chega | A sequência leva a |
|---|---|---|---|---|
| New Sound, Create sound | Não | W3 | `family_command` e cartão Sound | Aba aberta; no 2.º, já ligada ao objeto |
| Régua | `label_step` (`ruler.rs:191`), sem o pintor do canvas | W3 | Clique põe o cursor | Play começa ali |
| Onda do Mix e das camadas | Não; `paint.rs` do Audio Editor a conferir | W3 | n/a | Reflete o último render |
| Linha de camada, ≡ M S | Não | W3 | W3 | Som muda; arrastar muda `start` |
| Add layer | Não | W3 | W3 | Camada nova toca |
| Cartões Source, Shape | Cartão sim | W3 | W3 | Editar muda o som |
| Peak, Loudness | Mixer sim, reuso a medir | W3 | n/a | Reflete o buffer sem limitador |
| Spectrum | Não; FFT a conferir (`paint_spectral.rs`) | W3 | n/a | Reflete o buffer |
| Play, Stop, Space, Play on change | Voz sim | W3 | W3 | Ouve o som atual |
| Use in Game | Não | W3 | W3 | **Som toca no Play** |
| Aviso de motor mudado | Não | W3 | n/a | Aprovado continua tocando igual |

Atalhos: conferir `Space` e `M`/`S` contra os globais antes de usar. Lei da compactação e micro-pesquisa Figma/Rive por cartão ([16](../CartoesUI/16_REFERENCIAS_POR_CARTAO.md)). Só inglês, tokens e i18n.

**Corrige de passagem:** Preview mudo em objeto sem `Transform` (`audio_2d.rs:58-61` devolve `None`; `fase_inspector_commits_audio.rs:24` descarta o `false`): é o mesmo cartão por onde o artista entra. A escuta não depende de posição.

## 5. Gates, vermelho primeiro

| Gate | Fixture que contém o fenômeno |
|---|---|
| `limitador_nunca_passa_do_teto_na_escuta_e_nao_entra_no_arquivo` | Ressonância com ganho que passa de 1,0; compara os bytes do arquivo |
| `cancelar_nao_muda_um_bit` e os hashes dourados intactos (`conjunto 86ea4a05df14620b`) | As 9 receitas, com e sem cancelamento anterior |
| `render_velho_nunca_substitui_o_novo` | Pior corpo de 120 s (9,4 s), logo seguido de uma edição barata |
| `editar_nao_bloqueia_a_tela` | O mesmo pior corpo; o `submit` volta sem esperar |
| `desfazer_devolve_a_receita_e_o_som` | Sequência de edição, undo, redo; compara hash do PCM |
| `blob_v1_abre_na_v2` | Bytes v1 gravados agora, antes da mudança |
| `mudo_nao_muda_a_semente_das_outras` · `solo_nao_entra_no_arquivo` | Receita de 3 camadas |
| `aprovado_nao_muda_quando_o_motor_muda` | Versão simulada; aviso aparece e o som fica igual |
| `tres_cliques_ate_ouvir_no_jogo` | Objeto sem som; Create sound, Use in Game, Play |
| `preview_sem_transform_toca` | Objeto sem `Transform` |
| Mutação nos gates do limitador, ticket, `muted` e Use in Game | Agente `mutacao`, 1× no fecho |

Censos e portões por nome: receita de cartões §Portões, `the_shell_only_shrinks`, tetos de LOC, `no_magic_numeric`.

## 6. Smoke e números

Cena nova `PH2D_SFX_SMOKE=3`: a aba aberta em «Metal Hit», com um objeto sem som pronto para o Use in Game, moedas e Audio Listener. Fotografar cada passo.

**Medido em 06/11** (sonda W2, mínimo de 7 rodadas intercaladas, perfil `smoke`, load ~16, logo ordem de grandeza): típica 2 s 7,41 ms; pior 2 s 24,63 ms; editar um corpo 3,30 a 4,14 ms; só o ganho 0,10 ms. De W2: pior corpo de 120 s 9 370 ms e gravação a +24 semitons 32 ms, os dois casos que justificam a thread.

**A medir antes da mensagem ao dono:** latência da edição até o som novo tocável, tempo por quadro da tela com 8 e com 64 camadas, custo do limitador e do pico/espectro por render. O teto de 30 ms continua proposto: o ouvido do dono o confirma aqui.

## Decisões de construção (lidas no código, 06/11; valem sobre o §2)

- **Cartões = painel de verdade, na VAGA DO INSPECTOR** (decisão da janela principal, passo 4; revoga a coluna reservada dentro da aba, recusada pela foto: Inspector aberto ao lado de uma faixa vazia). Nenhuma `DocumentSurface` pinta `CardCtx` (exige `PaintCtx`: `panel/cards.rs:149`). ⇒ `ph2d-panel-sfxlab`, `DEFAULT_SLOT` RightTop, `DEFAULT_VISIBLE=false`. Com aba do som ativa o Inspector sai e o painel entra; ao sair, o Inspector volta como estava, aberto ou fechado (`ponte::vaga`). Gates em `ph2d-panel-sfxlab/tests/it/vaga.rs`.
- **Medidores:** coluna fina à direita da pilha, DENTRO da aba, pintada pela surface (`medidor::pinta`). Calculados no `RenderJob` sobre o buffer sem limitador: pico, `integrated_lufs` e espectro pela porta `ph2d-audio-spectral` (ADR-0115). O cálculo mora na crate do painel porque a da família está no tecto de LOC.
- **Nome da camada:** o tipo, numerado quando repete, por ordem de id: reordenar não renomeia. Derivado na tela, sem campo no documento.
- **Recording:** o cartão Source escolhe o material (`Widget::Dropdown`). O `Take` não tem nome: a tela diz «Recording 1 (1.00 s)». A Add layer continua pegando o primeiro.
- **Sem camada escolhida:** clicar fora de tudo na aba (a faixa do Mix, por exemplo) tira a seleção, e os cartões passam ao Sound (Duration, Seed).
- **Campo que não se aplica não se pinta** (`ler` devolve `None`): ruído branco sem Hz nem Q; Stretch só na lei Stretched; FM só na onda FM. Curva sem Attack, Decay e Close: frase, e o botão explícito «Use Attack, Decay and Close». **Esc e arrastar de volta** devolvem o valor exato, não o arredondado da caixa (`ponte::aplica`).
- **Seed na tela:** os 32 bits de baixo, porque o campo é `f64` e a semente tem 64; os de cima ficam.
- **Painel ↔ família:** como o mixer: snapshot `thread_local` na crate do painel (família publica antes de pintar) e fila de intenções que a família drena no `service`. O painel depende de `ph2d-sfx-recipe`; a família depende do painel.
- **Tabela única de campos:** `Campo` (id, chave de rótulo, unidade, faixa, ler, escrever) gera `populate`, `prepare`, cartões e despacho. Campo que a tabela não tem não existe em lugar nenhum.
- **Mundo e som:** a surface não recebe `World`. A shell chama `ph2d_app_sfxlab::service(hero, world, escuta)` uma vez por imagem, ao lado de `ph2d_app_kanban::service`. `escuta` é o trait `Escuta` da família (`toca(SampleData)`, `para()`), implementado para `AudioEngine` via `AudioSystem::scene_parts().1`; o teste usa um falso. A família guarda a cópia de trabalho e escreve na `SfxLibrary` do mundo no `service`; a receita do mundo que difere da última escrita (projeto aberto) é adotada e o histórico zera.
- **`RenderJob`:** uma thread por aba aberta, dona do `RenderCache`; `AtomicU64` com o ticket mais novo é o `cancelado`; a fila drena até o pedido mais novo; `Drop` cancela e faz join. Devolve `Pcm`, onda fixa por camada e do mix (min/max por coluna; a tela só reamostra), pico e LUFS sem limitador. `RenderCache` ganha um leitor do buffer por camada. Material em `Arc<Takes>`.
- **Onda:** própria, offline, sem `PeakCache` (o Audio Editor pede `SampleData`, e ela vai por camada com `start`). **Espectro:** pela porta FFT `ph2d-audio-spectral` (ADR-0115), na thread do render.
- **Abrir de novo:** `family_commands` lista um «Open Sound ▸ <nome>» por receita da biblioteca sem aba, além de «New Sound».
- **Desfazer:** instantâneos de `Recipe`, com um gesto contínuo (arrasto, scrub) coalescendo em um passo; teto de 1 000 passos, com a medição do tamanho ao lado.

## Ordem

1. **Feito** (`174859cd01`). Porta do motor: cancelamento, limitador, blob v2, `muted`, `Approved` (sem tela; gates).
2. Família: aba, `RenderJob`, escuta, desfazer, cena 3.
3. Tela: régua, ondas, pilha, gestos.
4. Cartões e medidores.
5. Use in Game, Create sound, Preview sem `Transform`.
6. Medir, fotografar, smoke do dono, `/pd-onda-fechar`.

Cada passo muda o seguinte, por isso em série; as medições vão num lote só (§0.10). Janela nova entre passos se o contexto passar de 300 mil.
