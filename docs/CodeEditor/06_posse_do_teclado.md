# CodeEditor — item 7: de quem é o teclado (e o rato) com a cena a correr

> Origem: smoke do dono da cena 6 (20/10): *«a tecla espaço fica em conflito com o jogo pois dá pausa»*. Família
> inteira num bloco (CLAUDE.md §0.10): os scripts, o controlo de jogador e o rato — o mesmo defeito em três sítios.

## 1. O defeito (lido no código, 20/10)

- `key_input` (`shells/desktop/src/input_dispatch/keyboard.rs`) alimenta o jogo (`InputState`) e **segue**: a mesma
  tecla chega a todo atalho do editor (*«o dedo do jogador OBSERVA, nunca consome»*, W3). Espaço = Play/Pause
  (`handlers_teclas_editor.rs`), setas = nudge do vetor, letras = ferramentas, Tab = modo, Delete = apagar…
- O controlo de jogador (`player_input`) lê o MESMO `InputState` ⇒ o mesmo conflito.
- O rato: `on_mouse_input` dá o clique ao jogo (`rato_do_jogo`) e **segue**: com a cena a correr, o clique esquerdo
  seleciona e abre um Translate, o direito acrescenta uma raiz (`right_click_on_canvas`), o do meio faz pan.
- E o inverso: com a cena a correr e o foco num painel (um campo do Inspector), as letras escritas chegam ao jogo.

## 2. Oráculo — o EDITOR do Godot 4.7.2 corrido (MIT; `ferramentas/oraculo_teclado/`, fixtura `saida.txt`)

Corrido numa Xvfb (o XTest chega; na Xwayland do `kwin --virtual` não), teclas e cliques por `xdotool`, o jogo a
registar cada tecla e um batimento de 100 ms. Duas corridas (a 1.ª clicou fora da barra no caso «2D» — corrigido).

| Situação | O jogo recebe | O editor faz |
|---|---|---|
| F5 com o editor em foco | o FOCO (sem clique) | corre o jogo |
| jogo em foco: Espaço, ←, A, Tab, Esc, Delete, 1 | sim | nada |
| jogo em foco: Ctrl+S, Ctrl+Z | sim | (não medido) |
| jogo em foco: F8 (parar) | sim | **pára** — atalho reencaminhado |
| jogo em foco: F9 / F10 (suspender / quadro) | F9 sim | **suspende / 1 quadro** — reencaminhados |
| jogo em foco: F7 (pausa) | sim | **nada** (não é reencaminhado) |
| clique no editor | perde o foco | — |
| editor em foco: Espaço, ← | **nada** | o atalho dele; F7 pausa |
| clique no jogo | o clique + o foco | — |
| modo «2D» da barra: clique / Espaço no jogo | **nem um nem outro** | escolhe objetos |

Unity e Roblox Studio não correm aqui (Windows/macOS) — documentação, não oráculo:
[Unity: atalhos globais não correm com a Game view seleccionada a jogar, por desenho; os de MENU correm sempre](https://issuetracker-mig.prd.it.unity3d.com/issues/shortcutattribute-does-not-work-when-in-play-mode-and-the-game-view-is-selected) ·
[Roblox: Shift+F5 pára, F5 pausa/continua](https://create.roblox.com/docs/studio/shortcuts) ·
[Roblox: «Respect Studio shortcuts when game has focus» (ligado de fábrica) é a causa de um defeito relatado por quem faz jogos](https://devforum.roblox.com/t/zoom-to-shortcut-f-button-sinks-input-in-solo-play/3918064).

## 3. Decisões do dono (20/10)

1. **F5 pausa/continua, Shift+F5 pára e rebobina** — funcionam sempre, também com o jogo a usar o teclado; o jogo
   não as recebe (Roblox).
2. **Moldura na cena** na cor de destaque enquanto o jogo tem o teclado.
3. **Com a cena a correr, o clique na cena é só do jogo** (o «Input» do Godot): não seleciona, não move, não pinta.
4. **A mão da física fica como está** (com a física ligada, agarrar/atirar um corpo continua — e o jogo também recebe
   o clique). Por extensão técnica, as irmãs dela (explosão e atração, a mesma ferramenta de interacção).

## 4. A lei

`o jogo tem o teclado` = **a cena corre** (`Playhead::is_playing`) **∧ a cena está à vista** (nenhuma aba de
código/quadro por cima, nenhuma entrada de texto, **nenhuma paleta, menu, diálogo ou pré-visualização do Morph**) **∧ o
foco é do jogo**. **Tapar a cena LIMPA o foco** (e larga as teclas): destapar não o devolve, só um clique (Godot/Unity).

- **O foco do jogo:** o Play (qualquer porta: botão, Espaço, F5, grafo, cena de smoke) dá-o ao jogo se a cena está à
  vista (Godot: o F5 dá o foco); um clique **na cena** dá-o; um clique **fora** (painel, barra, linha do tempo, aba)
  tira-o. Detectado por quadro (a subida do `is_playing`), nunca por porta — as portas do Play são cinco.
- **Teclas, com o jogo a ter o teclado:** a tecla nua desce **só** no jogo (nenhum atalho do editor a vê);
  um acorde (Ctrl/Alt/Super) é **só** do editor (a lei de sempre: *«os players pulam sozinhos»* com Ctrl+Z);
  F5/Shift+F5 são **só** do transporte; o autorepeat do F5 é ignorado (só a 1.ª pressão alterna).
- **Sem o jogo a ter o teclado:** nenhuma tecla desce no jogo (Godot: editor em foco ⇒ o jogo não recebe), excepto
  no modo de pré-visualização do morph (o modal que É uma pré-visualização de jogo e lê o `ActionState`).
- **O soltar chega sempre ao jogo** (a assimetria de sempre; uma subida de tecla não descida é ignorada pelo
  `KeyboardState`). **Perder a posse larga tudo** (`release_all`): pausar, clicar num painel ou abrir uma aba com
  uma seta presa não deixa o personagem a andar sozinho.
- **Rato, com a cena a correr:** um botão premido **na cena** vai ao jogo e dá-lhe o foco; nenhum ramo do editor
  age, excepto os botões do HUD (são a interface do JOGO) e o laboratório de física (mão, explosão, atração — sem
  selecionar). Premido fora da cena, o editor tem-no (e tira o foco ao jogo). O soltar passa sempre a todos, **excepto** o de um premir só-do-jogo, que é só do jogo (com o Up do HUD). A roda
  fica do editor (o jogo não a lê — declarado no item 6).
- **Moldura:** desenhada exactamente quando `o jogo tem o teclado`.

## 5. Onde

- **Folha pura** `ph2d-app-components::posse_do_jogo` (nova): `Cena`, `FocoDoJogo {quadro, tecla, clique, premido,
  tem_o_teclado}`, `Tecla {desceu, acorde, shift, f5, repete}`, `Transporte`, `Destino` e `destino()` — a decisão inteira
  testável sem janela. A shell: `input_dispatch/posse_do_jogo.rs`; a moldura: `paint_canvas_overlays::paint_game_frame`.
- **Shell (composição):** o observador do `key_input` pergunta à folha (alimenta / consome / transporte); o
  `on_mouse_input` pergunta-lhe pelo clique (um irmão `despacho_clique_jogo.rs`, porque `input_dispatch.rs` está a
  586/600); o `push_input_to_script` corre o `frame` antes dos leitores; F5/Shift+F5 ao lado do Espaço (Espaço e F5
  numa porta só; Shift+F5 = o `TransportCmd::Reset` do botão Rebobinar, pelo barramento); a moldura no overlay.
- O estado `GameFocus` mora no `ComponentsShell` (a catraca `the_app_only_sheds_fields` fica em 172).

## 6. Portões (o vermelho se a propriedade quebrar) — os nomes REAIS

- **Folha** (`crates/ph2d-app-components/src/posse_do_jogo.rs`, 15 testes): `o_play_da_o_teclado_ao_jogo` ·
  `com_uma_aba_por_cima_o_play_nao_da_o_teclado` · `um_clique_fora_tira_o_teclado_e_larga_as_teclas` ·
  `pausar_larga_as_teclas_presas` · `tapar_a_cena_tira_o_foco_e_destapar_nao_o_devolve` ·
  `o_largar_de_um_clique_do_jogo_e_do_jogo` · `a_tecla_do_jogo_entra_no_teclado_dele_e_so_ela` ·
  `com_a_cena_tapada_um_clique_nao_e_so_do_jogo` · `com_a_cena_parada_um_clique_nao_e_do_jogo` ·
  `com_o_jogo_a_tecla_nua_e_so_do_jogo` · `sem_o_jogo_nenhuma_tecla_desce_nele` ·
  `a_previa_do_morph_continua_a_ler_o_teclado` · `um_acorde_e_sempre_do_editor` · `o_soltar_passa_sempre` ·
  `f5_e_shift_f5_sao_do_transporte_e_nunca_do_jogo`.
- **Shell, ordem** (`shells/desktop/tests/it/a_posse_do_teclado_esta_na_ordem.rs`, 4 testes):
  `a_tecla_pergunta_a_posse_antes_de_todo_atalho` · `o_clique_pergunta_a_posse_antes_do_primeiro_ramo_do_editor` ·
  `o_foco_anda_antes_dos_leitores_do_teclado` · `o_espaco_e_o_f5_sao_uma_porta` (inclui Shift+F5 = `Reset`).
- **Re-ancorado:** `the_players_finger_reaches_the_bridge.rs` (os dois testes do bloco antigo, agora por comportamento
  via `destino`) · `morph_arrow_seam_tests.rs` (agulha `self.tecla_do_jogo(`).
- **Conduzido** (app real, Xvfb + `xdotool`): `ferramentas/conduz_posse/corre.sh`, 15/15; fixtura `saida.txt`, fotos 01–05.
- **Mutação:** 15 mutantes + controlo (os 2 sobreviventes ganharam `com_a_cena_tapada_um_clique_nao_e_so_do_jogo` e
  `a_tecla_do_jogo_entra_no_teclado_dele_e_so_ela`).

## 7. Smoke = a cena 6

`PH2D_CODE_SMOKE=6`: Espaço atira e a cena **não** pausa; F5 pausa e continua; Shift+F5 rebobina; clicar num painel
tira a moldura e as setas deixam de mover o círculo; clicar na cena devolve-a; clicar no quadrado não o seleciona.

## 8. Consequências declaradas

- O botão do meio (pan) e o menu do botão direito do canvas, sobre a cena, são do jogo enquanto ela corre (a roda/zoom
  continua do editor) — consequência do «clique só do jogo».
- As teclas encaminhadas pelo rato sobre o grafo do Motion/linha do tempo vão ao jogo enquanto ele tem o foco (o foco é por clique, como no Godot).
- Um quadro de atraso do foco depois do Play (a subida vê-se no início do quadro seguinte).
- Shift não é acorde: Shift+letra vai ao jogo (Shift+A, o menu Add, fica indisponível a jogar).
- Teclas sem nome em `ph2d_input` são engolidas enquanto o jogo tem o teclado.
- Fora de jogo nenhum `keydown` chega ao `InputState` (o jogador não lê as setas ao avançar quadro a quadro com a física armada).
- Uma cena que começa a correr tapada precisa de um clique.
- A mão da física, a explosão e a atração mantêm-se (decisão 4).
