# P12 — Ergonomia: o plano (08/11)

> Linha `line/PixelArt`, HEAD `ec50512797`, base `652b506b58`, 32 commits à frente do `main` (P11, smoke OK do dono 08/11).
> Plano do módulo: [02_plano_modulo.md §P12](02_plano_modulo.md). Estudo: [01_estudo_aseprite.md §6](01_estudo_aseprite.md). Molde: [P11_plano.md](P11_plano.md).
> Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md).
> Medido: [P12_oraculos.md](P12_oraculos.md) (O1–O22).
> **Estado: plano escrito; decisões do dono no §9 por responder. Nenhum código.**

## 0. O que entra

| Grupo | Peças | No Aseprite |
|---|---|---|
| **A — Atalhos** | tabela de fábrica igual à do Aseprite (com as exceções do dono), janela `Keyboard Shortcuts` editável, roda do rato | Edit ▸ Keyboard Shortcuts (O1–O9) |
| **R — Run Command** | procurar qualquer comando pelo nome e corrê-lo | `Ctrl+Space` (O10) |
| **H — Undo History** | lista dos passos com nome, clicar volta lá | Edit ▸ Undo History (O11–O14) |
| **S — Recuperação** | gravar cópia de segurança sozinho; oferecer de volta depois de um fecho à força | Files ▸ Recover Files (O15; O16 não medível) |
| **U — User data** | cor + texto (+ propriedades, §9.5) em sprite, camada, cel, tag, tileset; `.ase` ida e volta | User Data (O17) |
| **P — Preferências** | grade, grade de píxel, xadrez, onion skin; para este documento / para os novos | Grid, Background, onion (O18–O21) |

**Fora, de propósito:** layouts de workspace e painéis dockáveis (é a UI do app inteiro) · tema · extensões · `--batch` (adiado desde a P9).

## 1. Conferido no código

| Afirmação | Conferido |
|---|---|
| O plano diz «Smoke 13» | ❌ `smoke.rs:13`, `max_level: 24` ⇒ **cena 25** |
| Atalhos editáveis | ❌ só `match` fixos: `ph2d-app-pixel/src/input_teclas.rs` (`key_with` ~:96), ferramentas em `ph2d-editor-core/src/pixel_panel/tools.rs:134-155`; app: `shells/desktop/src/keymap.rs:13`. Nenhum ficheiro de teclas, nenhuma janela. `ph2d-input::InputMap` (`map.rs:42`) é do jogador, não do editor |
| Procurar comando | ⚠️ **existe a paleta global `Ctrl+K`** (`shells/desktop/src/global_palette_input.rs`, `editor-core/src/screens/hero/global_palette.rs`): rail, painéis e menus. Não lista comandos do Pixel |
| Undo History | ❌ sem lista em lado nenhum. Pixel: `History` por sprite (`ph2d-pixel-core/src/history.rs:13`, tecto 64 MiB `:9`); `Op` sem nome (`op.rs:14`) |
| Autosave / recuperação | ❌ no desktop e no Pixel. Só scripts Luau (`ph2d-app-code/src/recovery.rs`). O Pixel grava dentro do `.ph2dproj` (`persist.rs:19`, chamado em `project_save.rs:159-162`) |
| Preferências | ✔ `~/.ph2d/prefs.txt` (`shells/desktop/src/prefs.rs`, `chave=valor`). ❌ **nenhuma do Pixel persiste**: grade `state.rs:110` (bool, cor = token `Border`, sem tamanho), onion `compose.rs:89-109` (1/1, base 68, step 28), xadrez `pixel-render/src/view.rs:15-20` (16, cores fixas) — tudo de sessão |
| User data | ⚠️ só texto de slice (`slice.rs:45`). Leitor `.ase` lê texto+cor e **descarta** propriedades (`ph2d-aseprite/src/file.rs:475-493`, «P12» em `:60`); escritor só de slice (`write.rs:166-175`) |
| Conflitos de tecla hoje | `G` grade do app (deixado passar, `tools.rs:132`) · **`F2`**: rename global (`keymap.rs:24`) × loop do Pixel (`input_teclas.rs:181`), por medir quem ganha · `Ctrl+K` paleta do app × Preferences do Aseprite |
| Tecto `ph2d-app-pixel` | 11 412 de 11 415: **folga 3**. Tudo novo em crate |

## 2. O oráculo

| Oráculo | Responde | Estado |
|---|---|---|
| **Aseprite Trial 1.3.18.6** (EULA, só corrido; Xvfb `:77`, `HOME` limpo) | atalhos, busca, conflito, Reset, roda; Run Command; Undo History; opções de recuperação; User Data; Grid/Background/Timeline/onion | ✔ O1–O22 |
| Tabela de fábrica | o `Export` do diálogo só grava o **delta** (161 bytes de fábrica, O2). A tabela veio do ficheiro de DADOS instalado (`data/gui.xml`, lista de teclas, não código) e foi conferida por amostra no diálogo. ⚠️ Antes de a usar em teste, **conferir linha a linha no diálogo** (rodada 2, §10) | ⚠️ |

### ⛔ O que o Trial não deixa medir

**A recuperação depois do crash (O16):** com o período em 10 s, `kill -9`, duas vezes: nenhum ficheiro de sessão e nada oferecido ao relançar. O Trial não grava dados de recuperação. ⇒ o fluxo «Recover Lost Sprites» e o formato no disco **não têm paridade possível**; a lei é nossa (§4.3), testada por propriedade.

## 3. O que o plano antigo dizia e a medição corrigiu

- «Atalhos iguais aos do Aseprite»: as exceções do dono tiram ao Aseprite `G` (balde) e `Shift+G` (Gradient), e o `P` / `Shift+P` dele (Frame Properties / Layer Properties) ficam sem tecla ⇒ §4.2.
- «Run Command»: o app **já tem** uma paleta (`Ctrl+K`). Não se faz outra (§9.2).
- «Preferências do módulo (grade, onion, xadrez)»: no Aseprite o onion **não** está nas Preferences, abre do botão direito no botão de onion (O21).

## 4. As leis

### 4.1 Medidas, que copiamos

| Lei | Fonte |
|---|---|
| Tabela de fábrica (ferramentas, momentâneas, modificadores, menus, zoom `1..6`, quadros, camadas, pincel) | O-tabela |
| Uma acção pode ter várias teclas; a mesma tecla em contextos diferentes não é conflito | O7 |
| Busca por parte do nome, em todas as secções, agrupada | O4 |
| Roda: sem modificador = zoom, `Ctrl` = tamanho do pincel, `Ctrl+Shift` = quadro, `Alt`/`Alt+Shift` = cor da paleta, `Shift` = rolar na horizontal | O9 |
| Run Command: busca aproximada (`flp` → Flip), tecla à direita, indisponível a cinzento | O10 |
| Undo History: nome do passo, memória no título, clicar = desfazer até lá, acção nova **corta** o resto; uma história por sprite; «ir ao quadro/camada mudados» ligado | O11–O14 |
| Recuperação ligada de fábrica, a cada **2 min**; guarda **1 semana** | O15 |
| User data = cor (transparente por omissão) + texto, botão ao lado do nome | O17 |
| Grade 16×16, opacidade 160 auto; grade de píxel; xadrez 16×16 com «Apply Zoom»; escopo «este documento / novos documentos» | O18–O19 |
| Onion: opacidade 27 %, passo 11 %, atrás do sprite, laço na tag, «Set as Defaults» | O21 |

### 4.2 Medidas, que NÃO copiamos (defeitos ou choques)

| Aseprite | Nós | Porquê |
|---|---|---|
| `G` balde · `Shift+G` Gradient | `P` · `Shift+P` | ordem do dono (07/11); `G` é a grade em todo o app |
| `Ctrl+'` grade | `G` **e** `Ctrl+'` | o `G` do app manda; `Ctrl+'` fica como 2.ª tecla para quem vem do Aseprite |
| `P` Frame Properties · `Shift+P`/`F2` Layer Properties | sem `P`/`Shift+P`; `F2` = Layer Properties (rename, como no app) | o `P` é do balde; `F2` já é renomear no app inteiro (§9.1) |
| `Ctrl+K` Preferences | `Ctrl+K` = paleta do app | é do app inteiro (§9.2) |
| Tecla já usada é **roubada sem perguntar** (O6) | pergunta: «`Shift+S` is used by Snap to Grid. Replace?» | perder uma tecla em silêncio é o defeito |
| `Reset` só de tudo (O8) | também por acção | pedido óbvio, custo zero |
| `Enter` na caixa de captura vira tecla (O5) | igual: a captura grava qualquer tecla; confirma-se no botão | é como se atribui `Enter` |

### 4.3 Recuperação: lei nossa (sem paridade, §2)

Cópia de segurança do documento a cada N minutos **só se mudou** desde a última, numa pasta própria fora do projeto, com escrita atómica (ficheiro novo + renomear). Ao abrir depois de um fecho à força, uma janela oferece cada cópia com nome, hora e miniatura: `Restore` · `Discard` · `Later`. Fecho normal apaga as cópias. Guardadas 1 semana. Prova: matar o processo em cada ponto da escrita e reabrir dá sempre ou a cópia anterior ou a nova, nunca um ficheiro partido.

**Feito (08/11):** a janela é a do app, em cartões (`ph2d_recovery::card`: um cartão por sessão com nome, «Today 14:32» e tamanho, `Restore` · `Discard`; rodapé `Later`; `Esc` = `Later`; modal, come o ponteiro e o teclado); depois de Open/Restore os bytes abertos são a linha de base (a primeira hora não regrava igual); fechar com trabalho por gravar (um restaurado inclui-se) pergunta `Save` · `Don't Save` · `Cancel` na mesma janela.

**Medido (08/11, `a_serializacao_de_um_projeto_grande_custa`, `--profile smoke`, load ~6–11):** 16 Pixel Sprites 512² RGBA × 8 quadros = 128 MiB — codec do Pixel **84–85 ms**, uma cópia (o `postcard` do `ProjectFile`) **22–23 ms**, impressão SipHash **33 ms** (mínimo de 3, duas corridas): ≈ 140 ms no fio da janela, ≫ 8 ms. Decidido: a hora só serializa se houve entrada do artista desde a última (`Recovery::touched`, alimentado pelo `had_input` do `post_frame_undo`); com o app parado custa zero. Aberto: a edição remota do Quadro sem entrada local espera a próxima entrada.

**Sem soluço a meio do gesto (08/11):** (a) a hora vencida espera a PAUSA do artista — nenhum botão premido e, ou o quadro calmo depois do fim de um traço, ou `IDLE` = 1 s sem entrada (`Recovery::idle`); prazo duro de 2 × o intervalo, depois grava na mesma (`Autosave::overdue`; prova `a_hora_espera_a_pausa_do_artista_com_prazo_duro`). (b) A impressão SipHash e a decisão «mudou?» vão para o fio da escrita: o fio da janela só ENTREGA o `Vec` (`Autosave::offer`; o resultado vem em `finish`). Medido no MESMO processo, intercalado, mínimo de 5, `--profile smoke`, load ~6 (`o_fio_da_janela_so_entrega_os_bytes`):

| 128 MiB, no fio da janela | antes (impressão + entrega) | depois (só entrega) |
|---|---|---|
| mínimo de 5, corrida 1 | 32,5 ms | 35 µs |
| mínimo de 5, corrida 2 | 34,1 ms | 43 µs |

(c) **Codec incremental e serialização fora do fio da janela (08/11, feito).** A revisão do documento mora no `History` (`ph2d-pixel-core::history`, `History::revision`): um valor do PROCESSO (nunca se repete entre documentos nem depois de um `install`), novo em todo commit/undo/redo/`go_to` que aplica e em `History::touch`; recusado ou nada a desfazer não muda. Como prefs, próximo id, último export, user data e bandeiras de camada podem mudar SEM passo (o `set_prefs`, o `reserve_id`, a contínua do `intervalo`), a chave da cache (`codec::EntryCache`, por `Doc` em `ph2d-app-pixel`) é dupla: a revisão para as células + os bytes do registo SEM células (pequeno), comparados inteiros — nada gravável escapa. `codec::write_entry` + `write_library_from_entries` (o formato já era a sequência de entradas: bytes iguais; `write_library` passa a ser isso). Fio da cópia: a shell parte `project_bytes` em `project_file()` (o `ProjectFile` com o blob do Pixel por juntar + `persist::entries`, `Arc` por entrada) e `serialize`; a cópia usa `Recovery::offer_with`, cujo fecho junta o blob e corre o `postcard` NO FIO DA ESCRITA; o Save junta no fio da janela, os MESMOS bytes. Provas por nome: `a_revisao_muda_em_todo_passo_e_nao_num_passo_vazio`, `o_que_muda_sem_passo_tambem_reescreve_a_entrada`, `a_cache_da_os_mesmos_bytes_que_a_escrita_inteira` (6 × 60 edições aleatórias, 3 documentos, cache = escrita inteira), `a_entrada_guardada_e_a_escrita_inteira_a_cada_edicao` (app-pixel), `a_serializacao_corre_no_fio_da_escrita` (recovery).

**Medido (08/11, `o_codec_reaproveita_os_documentos_que_nao_mudaram`, `--profile smoke`, MESMO processo, três variantes intercaladas em rotação, mínimo de 5, load ~10–20):** 16 sprites 512² RGBA × 8 quadros = 128 MiB, um editado entre cópias.

| 128 MiB, um editado | antes (clona + `write_library`) | cache + junção (o Save) | só entradas (fio da janela da cópia) |
|---|---|---|---|
| corrida 1 | 76,1 ms | 23,3 ms | 1,12 ms |
| corrida 2 | 74,0 ms | 24,0 ms | 1,08 ms |

O `postcard` do `ProjectFile` (~23 ms) sai do fio da janela na cópia (corre no fecho do `offer_with`); no Save continua. Fica no fio da janela da cópia: o `capture_project` e os outros módulos (não medidos aqui) + ~1 ms do documento editado. Preço: a cache guarda uma cópia dos bytes gravados por documento (128 MiB a mais de RAM num projecto de 128 MiB); `EntryCache::clear` larga os que deixaram de estar vivos.

Settings ▸ Recovery (Off, 10 s … 30 min, ✓ 2 min) escreve `recovery.minutes` (Off = 0) e vale já (`Recovery::follow`). O hover por baixo da janela modal também não passa (`o_hover_por_baixo_da_janela_nao_chega_ao_editor`).

## 5. O desenho técnico (provisório até o §9)

- **`ph2d-pixel-keys`** (crate nova): a tabela `Acção → [Tecla+contexto]` como dados; a de fábrica com teste contra a O-tabela; o delta do utilizador em `~/.ph2d/pixel-keys.txt` (só o que mudou, como o Aseprite); o despacho de `input_teclas.rs` passa a perguntar à tabela (o `match` sai da `ph2d-app-pixel`, que encolhe).
- **`ph2d-panel-pixel-keys`**: janela `Keyboard Shortcuts` em cartões (busca, secções, `Add`, `×`, conflito, `Reset`).
- **Run Command**: os comandos do Pixel registam-se na paleta global (nome + tecla da tabela + disponível?).
- **`ph2d-pixel-core`**: `Op` ganha nome (`label()`); `History` expõe a lista e «ir a n». **`ph2d-panel-pixel-history`**: a janela.
- **Recuperação**: §9.4 decide o escopo; o escritor atómico e a varredura ao arrancar numa crate própria.
- **User data**: campo `UserData { color, text, props }` em sprite, camada, cel, tag, tileset; formato MINOR +1; `.ase` lê e grava `0x2020` depois de cada dono (propriedades tipadas incluídas).
- **Preferências**: `PixelPrefs` (grade, grade de píxel, xadrez, onion) no documento + cópia «para novos» em `~/.ph2d/prefs.txt`. O onion reusa `compose::Onion` na unidade 0–255: `base 68` = os 27 % do O21, `step 28` = os 11 % (mais `loop_tag` ligado e `current_layer_only`, ainda sem leitor).
- **Feito (dados, 08/11)**: `UserData`/`Owner`/`Op::UserData` e `PixelPrefs` no núcleo, formato MINOR 6, `.ase` `0x2020`+`0x2008` lido e escrito, Trial lê (`crates/ph2d-pixel-io/tests/fixtures/user_data/`). A cor de uma tag no `.ase` é `Tag::color`. Aberto: a `Slice` continua com `color`/`data` próprios (os chamadores de fora do núcleo: `ph2d-pixel-slices`, `ph2d-pixel-cenas`, `ph2d-app-pixel`, `shells/pixel-scenes`); as propriedades de uma slice ainda só avisam.

## 6. As provas

Por nome: tabela de fábrica = O-tabela (cada linha) · conflito pergunta e não rouba · delta grava e relê só o mudado · cada comando do Pixel acessível pela paleta · Undo History «ir a n» = n desfazeres e acção nova corta · `.ase` com user data em todos os donos lido pelo Trial (`--list-layers --list-tags --data`) e relido igual · recuperação por propriedade (§4.3) · preferências viajam no projeto. Mutação nas leis do conflito e da recuperação.

## 7. Ondas

| Onda | Entrega | Ponto de parada |
|---|---|---|
| **P12a** | tabela de teclas + despacho, janela de atalhos, Run Command, Undo History | ponteiro real |
| **P12b** | recuperação, user data + `.ase`, preferências | testes + Trial lê os nossos `.ase`; **cena 25 + fotos + smoke do dono** |

Fecham juntas no mesmo smoke.

## 8. Cena 25

Sprite pequeno com 3 camadas e 4 quadros já com história (6–8 passos de ferramentas diferentes), um atalho já trocado pelo «utilizador», grade 8×8 colorida e onion ligado. Abertas: Undo History e a janela de atalhos. Arte de código, sem terceiros. Fotografada antes de ir ao dono.

## 9. Decisões do dono

Respondidas pelo dono em 08/11: **«faça conforme sugere»**, as seis recomendações, com uma ordem a mais:
**«cuide de não criar conflitos entre atalhos, pois o app tem múltiplos módulos usando múltiplos atalhos»** (§9.7).

1. **`F2` = renomear a camada** (como no app). O laço da animação perde a tecla; fica no Run Command e no botão.
2. **Run Command = a paleta do app (`Ctrl+K`)** com os comandos do Pixel e a tecla de cada um. ⛔ **Sem `Ctrl+Space`**: o censo (08/11) achou-o GLOBAL (zen, `input_handlers.rs`), e o Pixel hoje engole-o por acidente (`input::space`); a §9.7 manda. Cura: o Pixel devolve-o ao app.
3. **Trocar atalhos: só no Pixel agora**, janela `Keyboard Shortcuts`. O desenho (tabela como dados) serve ao app inteiro depois.
4. **Recuperação do projeto inteiro**: cópia a cada 2 min se mudou; ao reabrir, `Restore` / `Discard` com a hora.
5. **User data = cor + texto + tabela `name = value` editável na tela** (além do Aseprite), em sprite, camada, cel, tag, tileset; `.ase` ida e volta.
6. **Grade, xadrez, onion: em cada documento + «Set as Defaults» para os novos**, como o Aseprite.
7. **Sem conflito com o resto do app.** Lei: uma tecla do Pixel nunca pode ser uma tecla GLOBAL do app (as que valem em qualquer aba) salvo as passagens declaradas (o Pixel deixa-a ao app: `G`, `F2`, `Ctrl+K`…). Prova por nome: censo das teclas globais × tabela do Pixel (fábrica **e** a do utilizador: a janela recusa atribuir uma tecla global, dizendo de quem é).
8. **Smoke 1 do dono (08/11), trocar um atalho num passo só:** «clicar no atalho, o próprio campo é limpo e aguarda o novo atalho; não precisa do `+` nem de 2 etapas». Revoga o `Add` + `Confirm` do Aseprite (O5). Defeitos do mesmo smoke: `Don't Save` ao fechar trava o app · UNDO HISTORY não estreita · janelas novas sem `X` e fora do menu Window · a tecla capturada aparece uma atrasada.

## 10. Rodada 2 de medição

**Feita 08/11** ([P12_oraculos.md](P12_oraculos.md) «Rodada 2», R1–R6). Leis que entram: tabela de fábrica corrigida (✎R1; `Esc` = cancelar a operação, `Ctrl+Z` também no contexto Transformation, setas e `F2` no contexto Normal) · história **linear** (o non-linear do Aseprite achata a lista e o `Ctrl+Z` anda pela lista, não pelo pai: R2, não copiamos) · o tecto de memória larga os mais velhos em silêncio e guarda sempre o mais novo (R3 = o nosso `History` hoje) · onion: 1 antes e 1 depois, colchetes arrastáveis sobre os números da timeline, guardado no documento (R4) · Cel e Tag: cor + texto (R5) · importar teclas SOMA ao que há (R6).

Pedido original: conferir a tabela de fábrica linha a linha no diálogo · non-linear history ligado · limite de memória a valer · quantos quadros de onion antes/depois e o gesto que os põe · user data de Cel e Tag · importar um ficheiro de teclas.
