# P11 — Slices: o plano (07/11)

> Linha `line/PixelArt`, HEAD `9b0d1e8a11`, base `652b506b58`, 2 commits à frente do `main` (P10c, smoke OK do dono).
> Plano do módulo: [02_plano_modulo.md §P11](02_plano_modulo.md). Estudo: [01_estudo_aseprite.md §3](01_estudo_aseprite.md). Molde: [P10_plano.md](P10_plano.md).
> Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md).
> Medido: [P11_oraculos_ferramenta.md](P11_oraculos_ferramenta.md) (T1–T8) · [P11_oraculos_arquivo.md](P11_oraculos_arquivo.md) (F1–F5).
> **Estado: plano escrito, decisões do dono no §9 (07/11); P11a em curso.**

## 0. O que entra

| Grupo | Peças | No Aseprite |
|---|---|---|
| **M — Modelo** | `Slice` no documento, formato gravado, desfazer | documento |
| **F — Ferramenta** | `Shift+C`, criar, selecionar, mover, redimensionar, duplicar, apagar | Slice Tool |
| **N — 9-slice** | linhas e cantos arrastáveis | ✔ arrastam |
| **P — Pivot** | marca arrastável | ❌ **além do Aseprite** (T4) |
| **K — Keys** | key por quadro, dura até a seguinte; esconder | ✔ |
| **C — Cartão** | janela flutuante SLICES no lugar do diálogo | Slice Properties |
| **X — Saída** | `.ase` ler e gravar; `meta.slices`; `Split slices`; `Area` | `--split-slices`, `Area:` |

**Fora, de propósito:** propriedades tipadas de user data (P12; o texto e a cor entram) · slices viram colliders (P14) · `--batch` do binário (adiado desde a P9).

## 1. Conferido no código

| Afirmação | Conferido |
|---|---|
| O plano diz «Smoke 12» | ❌ `smoke.rs:13`, `max_level: 23` ⇒ **cena 24** |
| O documento tem slices | ❌ nenhum tipo. Análogo: `Tag`, `sprite.rs:266`; `Sprite.tags`, `sprite.rs:294` |
| O `.ase` tem slices | ⚠️ `file.rs:295` só conta; `ase.rs:349` avisa `Aviso::Slices`; `write.rs` não grava; não há `0x2020` em lado nenhum |
| O JSON tem slices | ⚠️ `OpcoesJson.slices` em `sheet.rs:592`; `ph2d-sprite-sheet/src/aseprite.rs:289` escreve sempre lista vazia |
| Opção de folha análoga | `split_grid`, `sheet.rs:48,290`; checkboxes em `cards_folha.rs:226-238` |
| Numeração de quadros | `frames.rs`: `insert` :72, `delete` :152, `move_to` :185, `move_block` :200, `move_cels` :287, `reverse` :366 |
| Geometria | `canvas.rs`: `rotate_canvas` :163, `flip_canvas` :182, `sprite_size` :207, `canvas_size` :245, `crop` :269, `crop_to_selection` :283, `trim` :290 |
| Formato | `codec.rs:34`, `MINOR = 4`; campo desconhecido é saltado ⇒ MINOR 5, campo novo no sprite, sem `PROJECT_SCHEMA` |
| Ferramenta | `PixelTool`, `ph2d-editor-core/src/pixel_panel/tools.rs:9,44,96,130`; `C`/`Shift+C` sem dono nas tabelas de `input_teclas.rs`; ⚠️ `selecao::key` por ler |
| Tecto `ph2d-app-pixel` | 11 404 de 11 415 (`find … | wc -l`): **folga ~11**. A catraca conta por `the_shell_only_shrinks` |

## 2. O oráculo: o que respondeu e o que não pode responder

| Oráculo (licença por artefacto) | Responde | Estado |
|---|---|---|
| **Aseprite Trial 1.3.18.6** (EULA, só corrido) | Ferramenta, keys, geometria (ponteiro real, leitura no diálogo) · **`meta.slices` por `-b … --list-slices --data`** · leitura dos nossos `.ase` | ✔ corrido |
| LibreSprite 1.3 (GPL-2.0) | **Não tem slices**: `Unsupported chunk type 8226`, 0 slices ao regravar | ⛔ descartado |
| Pixelorama (`strings`) | Sem slices: só `Smart Slice` de importar folha | ⛔ descartado |
| 3 `.aseprite` reais (54 slices) | Chunk no quadro 0; cor em user data 54/54; 53 sem flags, 1 com 9-slice | ✔ parse nosso = JSON do Trial |

### ⛔ O que o Trial não deixa medir

O Trial recusa gravar imagem (`Save operation is not supported in trial version.`, exit 0). **Logo, nenhum píxel e nenhum nome de ficheiro de `--split-slices`, `--slice` ou `Area: Slice` foi visto.** Em particular: se cada quadro corta na key desse quadro, se quadros escondidos saem, o que faz fora do canvas, como se chamam os ficheiros. Para estes, a P11 tem **lei própria testada por propriedade** (§4.5), e ⛔ não dizemos «paridade» neles. A única referência é documentação (`part-{slice}.png`) e duas issues públicas. Um ficheiro real com várias keys, pivot ou escondida também não há: essas leis vêm das fixturas nossas lidas pelo Trial.

## 3. O que o plano antigo dizia e a medição corrigiu

- «9-slice e pivot arrastáveis (pedido da comunidade)»: **o 9-slice o Aseprite já arrasta** (linhas e cantos, T4). Só o **pivot** é pedido não atendido. Na origem da nota, o estudo juntou os dois.
- «export `--split-slices`»: **a janela Export Sprite Sheet do Aseprite não tem Split Slices**; só `Meta: Slices`. O corte é a opção `Area:` do Export File, e o flag de linha de comando. O PH2D não tem linha de comando ⇒ decisão §9.3.

## 4. As leis

### 4.1 Medidas, que copiamos

| Lei | Fonte |
|---|---|
| A key vale até a seguinte; antes da 1.ª vale a 1.ª; key além do fim vale para todos | F3 |
| `w=0` **ou** `h=0` esconde até a seguinte; o nome continua nas listas | F3 |
| Sem clamp ao ler: fora do canvas, centro e pivot fora da caixa, nomes gémeos, tudo sobrevive; só o desenho corta | F3 |
| Criar: recorta no canvas; normaliza; clique não cria; `Slice N` monótono; cor `#0000FF`; nasce sem seleção | T1 |
| Arrasto sobre uma slice nunca cria: move-a e seleciona | T1 |
| Clique substitui; `Shift`+arrasto é marquise; `Esc` limpa; `All`/`None`; `Ctrl+A` é dos pixels | T2 |
| 8 alças; passar da oposta trava em 1×1 | T2 |
| 9-slice ao ligar: margem 1 em todos os lados. Pivot ao ligar: `0,0` | T3 |
| Linha do centro passada da oposta trava com largura 1 | T4 |
| Redimensionar a caixa: `x,y` do centro ficam, a largura leva a diferença; o que não cabe desliga o 9-slice | T4 |
| Mover num quadro sem key cria key lá; `OK` sem mudança não cria | T5 |
| Canvas Size e Trim transladam a caixa; centro e pivot ficam; negativos sobrevivem; Crop não apaga | T7 |
| Sprite Size escala tudo; posição e pivot truncam para zero; cada borda trunca e o tamanho é a diferença | T7, G1 |
| JSON: `name, color, data, keys[frame, bounds, center, pivot]`; `#rrggbbaa`; centro zero não sai; `slices` na **ordem inversa à do ficheiro**; quadros absolutos, indiferente a `--tag`/trim | F5 |
| Ordem do chunk: 0x2022 seguido de 0x2020 com a cor, no quadro 0 depois das camadas | F2 |

### 4.2 Medidas, que NÃO copiamos (defeitos)

| Visto | Nós |
|---|---|
| Desfazer deixa a key criada para trás (T5): a edição seguinte não chega ao quadro | Desfazer volta **tudo** (ADR-0196) |
| Slice sem cor em user data é **invisível** no canvas (F3) | Desenha na cor padrão; ao gravar, escreve a cor |
| `--slice nope` não dá erro; `Output File` diz «Generated» sem gravar (F1, F4) | Erro dito |
| Slice fantasma `0,0,0,0` faz `--split-slices` não sair nada (issue #2283, documentado, não corrido) | Salta só esse quadro |
| Largura `0` no campo vira 1 sem dizer | Igual, com o campo a mostrar 1 |

### 4.3 Decisões do dono que mudam a lei

Ver §9: a slice segue a arte (§4.4); nome na tela, marcas na linha do tempo, remover a mudança de um quadro e `Transform` entram; `Split slices` e `Area` entram os dois.

### 4.4 Quadros e geometria com as slices a seguir a arte (§9.1)

- **Lei:** depois da operação, em cada quadro, a slice cai sobre a mesma arte em que caía antes. Propriedade: compõe-se a caixa antes e depois num mapa de marcas.
- Inserir: keys depois do ponto andam 1; o quadro novo herda. Apagar quadro com key: se o seguinte não tem key própria, ela passa a ele; a última key apagada desaparece e, se era a única, a slice fica com key no quadro seguinte. Mover e inverter: remapeia por quadro, materializando primeiro o valor efetivo de cada quadro; compacta keys iguais vizinhas.
- Flip e Rotate Canvas: caixa, centro e pivot transformados. Flip H: `x' = W-x-w`; centro `cx' = w-cx-cw`; pivot `px' = w-1-px`, porque o pivot é o píxel (T4: desenhado no píxel `bounds+pivot`). Rotações pela mesma regra.
- Moram em `FrameSet` e `CanvasSet` (campo novo `slices`), para nenhum chamador poder esquecer e o inverso sair de graça.

### 4.5 Saída sem paridade possível (§2)

Testadas por propriedade, sobre fixturas que separam as hipóteses:

- `Split slices` e `Area: Slice`: cada quadro corta **na key desse quadro**; quadro escondido não sai; recorte fora do canvas corta na interseção, e a célula nula não sai.
- Nome do item: `{title} ({slice}) {frame}`, com o mesmo padrão editável da P9. Gémeos ganham sufixo numérico, nunca se sobrepõem.
- `meta.slices` continua em quadros absolutos e coordenadas do sprite, porque foi o medido.

## 5. O desenho técnico

| Peça | Mora em | Porquê |
|---|---|---|
| `Slice`, `SliceKey` (`bounds`, `center: Option`, `pivot: Option`), `key_at(frame)`, remapeamentos, transformações | `ph2d-pixel-core/src/slice.rs` + `codec/slices.rs` | documento; sem dependências; MINOR 5, campo novo |
| `Op::Slices` (vetor inteiro + inverso, como as tags); campo `slices` em `FrameSet`, `CanvasSet` | `op.rs` | undo por sprite |
| Gesto puro (`press/drag/release → Option<Op>`), alças, marcas, hit test em píxeis de ecrã | **crate nova `ph2d-pixel-slices`** | fora do tecto; testa-se sem app |
| Desenho do contorno, 9-slice, pivot, alças | `ph2d-pixel-render/src/overlay.rs` | onde já está a seleção; alça de tamanho fixo no ecrã |
| Janela SLICES (lista + campos), em cartões | **crate nova `ph2d-panel-pixel-slices`**, molde `ph2d-panel-pixel-tiles` | cartão compacto, sem diálogo |
| Ligação da ferramenta | `ph2d-app-pixel` (input, estado), só composição | ⚠️ **folga 11**: mover código antes de ligar; nunca subir o número |
| `.ase`: chunk 0x2022, user data 0x2020 pela regra do «último lido» | `ph2d-aseprite/src/{file,write}.rs`, `ph2d-pixel-io/src/ase.rs` | `Aviso::Slices` sai; propriedades tipadas viram aviso novo |
| JSON | `ph2d-sprite-sheet/src/aseprite.rs`, `ph2d-pixel-io/src/sheet.rs` | estende o escritor, nunca um segundo |
| `split_slices`, `Area` | `sheet.rs`, `exporta.rs`, `ph2d-panel-pixel-export` | precedente `split_grid` |

Limitação do formato, declarada: as flags do chunk valem para a slice inteira. Key sem centro grava centro zero (o Trial lê como sem centro, F3); key sem pivot, numa slice em que outra tem, grava a sentinela `ph2d_aseprite::NO_PIVOT` (`i32::MIN, i32::MIN`), que lemos de volta como sem pivot: `0,0` é um pivot real e não pode marcar ausência. Gravar e reabrir é ponto fixo do documento e do JSON (`a_slice_with_a_pivot_on_some_keys_only_survives_save_and_reopen`). O Trial aceita pivot fora do canvas sem clamp e não o desenha (`weird.badpivot`, F3): lá a key fica sem marca visível, como a nossa.

## 6. As provas

- **O oráculo em laço:** o nosso `.ase` passado por `aseprite-trial -b --list-slices --data` dá o mesmo `meta.slices` que o nosso exportador, byte a byte no ramo `slices`. Gravado como fixtura com cabeçalho; a ferramenta refaz.
- As fixturas `oracle_slices_ferramenta.json` e `oracle_slices_arquivo.json` viram testes de lei (T3–T7, F3, F5), salvo as divergências dos §4.2 e §4.4, com a medida do Trial ao lado.
- **Ponteiro real:** ferramenta (criar, mover, alças, centro, pivot, seleção) em `shells/pixel-scenes/tests/it/slices.rs`; janela SLICES em `the_pixel_slices_answers_a_real_pointer`, molde `the_pixel_tiles_answers_a_real_pointer`.
- Formato: `a_minor_4_file…` lê o antigo, fixtura real, e o `an_unknown_field` continua a valer.
- Mutações por agente, e sobrevivente lido primeiro como fixtura que não separa as hipóteses.
- Testes por nome. Crate inteira só onde a regra pede (formato, crate nova, registo de painel). Suíte só na integração.

## 7. Ondas

| Onda | Entrega | Ponto de parada |
|---|---|---|
| **P11a** | modelo, `Op`, formato, leis de quadro e geometria, `.ase`, JSON, `Split slices`, `Area` | testes + laço do oráculo; sem tela |
| **P11b** | ferramenta, overlay, 9-slice, pivot, keys, janela SLICES | ponteiro real; **cena 24 + fotos + smoke do dono** |

P11a e P11b fecham juntas no mesmo smoke: uma slice só se vê com as duas. Os cartões de exportação entram na P11b.

## 8. Cena 24

Sprite pequeno, 6 quadros, que se entende à primeira vista: um **botão** de jogo (cantos, borda, miolo: o 9-slice faz sentido) e um **boneco** a andar, com pivot nos pés e key a seguir o boneco de quadro em quadro. Slices `button` e `hero` já feitas, ferramenta ligada, janela SLICES aberta. Arte de código, sem terceiros. Fotografada antes de ir ao dono.

## 9. Decisões do dono

Respondidas pelo dono em 07/11.

1. **A slice segue a arte ou fica parada?** O dono delegou: «veja o que mais os artistas gostam». **Decidido: segue a arte, nos quadros e na geometria** (§4.4).
   - Quadros: pedido explícito e antigo. [aseprite#1469](https://github.com/aseprite/aseprite/issues/1469) «Slices don't get properly adjusted when adding/deleting frames»; [#3142](https://github.com/aseprite/aseprite/issues/3142) (remapear ao mover, apagar, criar); [fórum 5329](https://community.aseprite.org/t/problem-with-slices-when-adding-removing-frames/5329) (hitbox por quadro que se baralha ao inserir).
   - Virar e girar: **nenhum pedido achado**, nem contra nem a favor. Segue a arte por coerência com a mesma lei. ⚠️ É inferência nossa, não pedido de artista: o smoke confirma.
2. **Extras: os quatro.** Nome na tela quando selecionada · marcas na linha do tempo · remover a mudança de um quadro (botão direito) · `Transform`, levar os pixels junto, desligado por omissão.
3. **Corte na exportação: nos dois lugares.** `Split slices` no Export Sheet e `Area` (Canvas ou o nome da slice) no Export As.

## 10. Rodada 2 de medição

Buracos de T1–T8 que mudavam lei; série, de propósito, porque nasceram da leitura da rodada 1 (CLAUDE.md §0.10). Resultado em [P11_oraculos_ferramenta.md](P11_oraculos_ferramenta.md), G1–G9.

- **Leis que entram:** apagar tira a slice inteira, em qualquer quadro (G2) · arrastar e apagar valem para toda a seleção, 1 desfazer (G3) · nome e cor valem para todas as keys (G6) · o pivot não muda ao redimensionar, mesmo ficando fora (G4) · `Transform` mexe só na célula da camada ativa no quadro atual, e redimensionar escala os pixels (G5).
- **Decisão nossa, por o Trial não fechar a regra:** arrastar uma borda com várias selecionadas aplica o mesmo delta a todas. Valor mixed no cartão aparece vazio, e digitar aplica a todas.
- ⚠️ **Anomalia não explicada (G4):** com key só no quadro 0, mover no quadro 1 também mudou o 0 no Trial, mas com keys em 0, 2, 4 mover no 3 criou key. O fórum relata o mesmo como mudança da 1.3.18 ([tópico 28638](https://community.aseprite.org/t/slice-behavior-change-in-aseprite/28638)). **Nós: editar sempre cria key só naquele quadro.** A lei é uma só, e a marca na linha do tempo e «remover a mudança» a tornam visível. A mostrar ao dono no smoke.
- **Por medir, sem mudar lei:** multi-seleção com caixa + números (`G3c`), Flip com seleção, cliques com outra ferramenta.
