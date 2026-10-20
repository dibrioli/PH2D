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
código/quadro por cima) **∧ o foco é do jogo**.

- **O foco do jogo:** o Play (qualquer porta: botão, Espaço, F5, grafo, cena de smoke) dá-o ao jogo se a cena está à
  vista (Godot: o F5 dá o foco); um clique **na cena** dá-o; um clique **fora** (painel, barra, linha do tempo, aba)
  tira-o. Detectado por quadro (a subida do `is_playing`), nunca por porta — as portas do Play são cinco.
- **Teclas, com o jogo a ter o teclado:** a tecla nua desce **só** no jogo (nenhum atalho do editor a vê);
  um acorde (Ctrl/Alt/Super) é **só** do editor (a lei de sempre: *«os players pulam sozinhos»* com Ctrl+Z);
  F5/Shift+F5 são **só** do transporte.
- **Sem o jogo a ter o teclado:** nenhuma tecla desce no jogo (Godot: editor em foco ⇒ o jogo não recebe), excepto
  no modo de pré-visualização do morph (o modal que É uma pré-visualização de jogo e lê o `ActionState`).
- **O soltar chega sempre ao jogo** (a assimetria de sempre; uma subida de tecla não descida é ignorada pelo
  `KeyboardState`). **Perder a posse larga tudo** (`release_all`): pausar, clicar num painel ou abrir uma aba com
  uma seta presa não deixa o personagem a andar sozinho.
- **Rato, com a cena a correr:** um botão premido **na cena** vai ao jogo e dá-lhe o foco; nenhum ramo do editor
  age, excepto os botões do HUD (são a interface do JOGO) e o laboratório de física (mão, explosão, atração — sem
  selecionar). Premido fora da cena, o editor tem-no (e tira o foco ao jogo). O soltar passa sempre a todos. A roda
  fica do editor (o jogo não a lê — declarado no item 6).
- **Moldura:** desenhada exactamente quando `o jogo tem o teclado`.

## 5. Onde

- **Folha pura** `ph2d-app-components::posse` (nova): `GameFocus { frame, press, has_keyboard }` e
  `route_key(…) -> KeyRoute { feed, consume, transport }` — a decisão inteira testável sem janela.
- **Shell (composição):** o observador do `key_input` pergunta à folha (alimenta / consome / transporte); o
  `on_mouse_input` pergunta-lhe pelo clique (um irmão `despacho_clique_jogo.rs`, porque `input_dispatch.rs` está a
  586/600); o `push_input_to_script` corre o `frame` antes dos leitores; F5/Shift+F5 ao lado do Espaço (Espaço e F5
  numa porta só; Shift+F5 = o `TransportCmd::Reset` do botão Rebobinar, pelo barramento); a moldura no overlay.
- O estado `GameFocus` mora no `ComponentsShell` (a catraca `the_app_only_sheds_fields` fica em 172).

## 6. Portões (o vermelho se a propriedade quebrar)

- Folha: `o_play_da_o_teclado_ao_jogo` · `um_clique_fora_tira_o_teclado_e_larga_as_teclas` ·
  `pausar_larga_as_teclas_presas` · `com_o_jogo_a_tecla_nua_e_so_do_jogo` · `um_acorde_e_sempre_do_editor` ·
  `f5_e_shift_f5_nunca_chegam_ao_jogo` · `sem_o_jogo_nenhuma_tecla_desce_nele` · `o_soltar_passa_sempre` ·
  `o_morph_continua_a_ler_o_teclado` · `com_uma_aba_por_cima_o_jogo_nao_tem_o_teclado`.
- Shell (conduzido pelo `key_input`/`on_mouse_input` reais): `o_espaco_com_o_jogo_nao_pausa_a_cena` ·
  `f5_pausa_e_continua_com_o_jogo_a_ter_o_teclado` · `shift_f5_para_e_rebobina` ·
  `um_clique_na_cena_a_correr_nao_seleciona` · `a_mao_da_fisica_continua_a_pegar` · `a_moldura_so_com_o_teclado_do_jogo`.
- Fonte: o observador consulta a folha ANTES de todo atalho (posição na cadeia, como `the_bind_beats_every_editor_shortcut`).

## 7. Smoke = a cena 6

`PH2D_CODE_SMOKE=6`: Espaço atira e a cena **não** pausa; F5 pausa e continua; Shift+F5 rebobina; clicar num painel
tira a moldura e as setas deixam de mover o círculo; clicar na cena devolve-a; clicar no quadrado não o seleciona.
