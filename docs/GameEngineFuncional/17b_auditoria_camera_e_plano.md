# 17b — Auditoria da câmera no canvas (2 lentes, 28/10) e plano de cura

Origem: smoke do dono sobre a onda 17 ([plano](17_camera_no_canvas_plano.md) ·
[handoff](handoffs/HANDOFF_CONTINUACAO_2026-10-27_17_CAMERA.md)). Texto técnico, para a próxima LLM.

## Pedido do dono (28/10)

Dobrar o ícone da câmera no canvas · ícone correcto na Hierarchy · propriedade no painel para o tamanho do
ícone · «no Run in Editor a câmera ficou parada, é correcto?» · «não consigo mover a câmera no canvas nem
modificar o offset» · auditoria completa, ver Godot e Unity e fazer melhor.

## Achados (mecanismo · repro · gate que faltava)

**A1 ALTA — o miolo («Dentro») nunca agarra numa cena real.** `camera_no_canvas.rs:35-53` passa
`outro_por_baixo = pick_objects_at(...)` ≠ câmera; o Sky (40×16 m, `smoke.rs:365,465`) e os montes cobrem a
moldura inteira ⇒ `alca_em` recusa (`moldura.rs:142-143`); o Down cai na seleção e seleciona o fundo.
Repro: `PH2D_GAME_SMOKE=4`, Camera, arrastar dentro da moldura. Gate verde porque
`clicar_dentro_da_moldura_nao_muda_a_selecao` recebe o bool à mão e a fixture não tem fundo nem Follow; a foto
`camera_zoom` escreve `height_world=5` directo (`smoke_foto.rs:179`) — nenhum gate nem foto passa pelo ponteiro.

**A2 ALTA — câmera com Follow: a moldura não anda com o Transform.** A moldura é a vista (`moldura.rs:61`) e,
em edição, o rebobinar corre a cada quadro (`fase_fabrica_e_morte.rs:309`; 292 linhas `[rebobinar]` no log) e
tira o `CameraRuntime` (`rewind_runtime.rs:190`) ⇒ a câmera renasce no alvo (`camera_2d.rs:198-204`). O ícone
anda, a moldura fica. Sem gate com Follow.

**A3 ALTA — limites: a 2.ª escrita leva o min ANTIGO.** `camera_no_canvas.rs:89-90` manda `LimitMin` e
`LimitMax` no MESMO quadro; `apply_camera_edit` (`camera_inspector.rs:242`) lê `CameraLimits` do mundo e
enfileira o componente inteiro (`:253`); a fila é diferida (`ph2d-inspector-ordering/src/lib.rs:189`). Borda
esquerda, de baixo e canto baixo-esquerda mortos; o centro só estica o max. (Inferência: a fila só drena depois
do laço `fase_inspector_commits.rs:255-271` — confirmar com teste.) Gate verde:
`as_alcas_dos_limites_nunca_invertem` testa só o `arrastar` puro.

**A4 ALTA (visível) — Hierarchy com ícone errado.** `hero_bridge.rs:152-156` escolhe o ícone do tipo com
`vec_path.is_some() ? Vector : Sprite`; câmera, luz e vazio mostram Sprite. `IconId::Camera` só existe no selo
do componente (`hier_rows.rs:86`). Gate verde: `o_icone_da_hierarquia_traz_o_cartao` cobre o selo, não o ícone
do tipo.

**A5 MÉDIA — Look Through ligado: realimentação.** `camera_frame_move` converte o ponteiro por `gfx.camera`,
que a pré-visualização reescreve a cada quadro (`fase_game_camera.rs:100-108`) ⇒ o miolo foge, o zoom fica
instável. `camera_frame_down` não olha `game_camera_preview`.

**A6 MÉDIA — dois campos «Offset» com o mesmo nome** (`component_catalog.rs:106` GameCamera, `:164` Follow).
O do Follow entra na mira e é apertado pelos limites (`camera_2d.rs:245-251`): no smoke 4 alvo+offset
(−8; 2,5) é preso em (−5,89; 3,00) ⇒ qualquer offset do Follow com x ≤ 2,11 e y ≤ 1,5 dá a mesma moldura. O da
câmera soma depois da cerca (`:263-265`) — devia mover; não confirmado por clique. Gate:
`the_camera_section_is_alive` só verifica que o Offset se pinta; só o Dolly tem teste
`ValueChanged`→barramento.

**A7 MÉDIA — «o canto mantém o centro» é falso com limites.** `arrastar` (`moldura.rs`, Canto) usa o centro
congelado no Down, mas o centro real é re-apertado a cada quadro contra a nova meia-largura
(`camera_2d.rs:247/267`) ⇒ a moldura escorrega e o canto foge do ponteiro. Na vista por omissão só um canto
está no ecrã. Gate verde: `o_canto_so_muda_a_altura_e_mantem_o_centro` nunca corre `camera_2d::update`.

**A8 MÉDIA — Play no editor.** Moldura e rótulo continuam pintados por cima do jogo (`fase_selection_highlight`
só os salta no palco ecrã cheio) e as alças continuam agarráveis (`camera_frame_down` só olha o palco) ⇒ um
arrasto escreve no documento a meio da corrida.

**A9 BAIXA-MÉDIA — ícone e dedo.** Ícone 22 px pintado (`moldura_tinta.rs:20`) × dedo = anel do vazio, raio
24 px que escala com o objecto (`group_gizmo_view.rs:53,123-130`); câmera escondida: `molduras()` ignora
`Visibility` (ícone pintado, não clicável); com a câmera selecionada o gizmo genérico do vazio desenha uma caixa
~48 px à volta.

**A10 BAIXA.** Rótulo cortado à esquerda (foto: «…nera · 1422 × 800 px»); sem hover/cursor nas alças; arrasto
não grava chave com Auto Key; mudar o formato não tem desfazer (`ProjectSettings` fora do `ProjectState`, como
a unidade); «na CERCA» alterna no log em quadros com `ticks==0` (anterior à onda).

## (a) «Run in Editor: a câmera ficou parada, é correcto?»

A câmera ANDA. Medido no palco editor com `PH2D_SIGNAL_LOG=1`: centro de (−5,89; 3,00) a (9,89; 3,00), preso nos
dois limites. Parece parada porque y fica preso em 3,00 a corrida inteira (limites com 10 m de altura, vista
8 m, alvo 2,5 < 3) e x só sai do limite depois de o Player passar ~−5,89 (os primeiros ~2,1 m). A onda alargou
essa zona em ~1,5 m ao corrigir o aspecto (canvas 1,41 → 16:9). **Não verificado:** as setas reais do dono com
o Player selecionado — repetir com `PH2D_SIGNAL_LOG=1`.

## Verificado consistente

Um só aspecto (`GameFormat::aspect`) alimenta moldura, Look through, DestroyOutside, aviso do Inspector e
janela · em ecrã cheio tarjas e vista usam a mesma caixa · formato gravado lido antes da janela abrir · rótulo
em px correcto · dropdown/Size/Frame do Inspector pintados, populados e clicáveis (popover registado por
último, `HitIndex::hit` em reverso) · o Up chega antes das guardas `on_canvas`.

## Godot 4 e Unity (pesquisa 28/10)

- **Godot:** sem ícone no canvas, só 3 rectângulos (ecrã magenta, limites amarelo, drag margin ciano), activa a
  3 px; idioma = câmera FILHA do player; no editor smoothing e drag margin ficam desligados
  (`is_part_of_edited_scene`) ⇒ moldura no player + offset, presa aos limites (a mesma sensação de «presa»);
  offset só número; limites com editor de arrasto (2025, undo, botão «Snap limits to viewport»); o editor não
  segue o jogo (Game tab: override de câmera). Pedidos abertos: miniatura (#7627), centrar no jogo (#8970).
- **Unity:** ícone com tamanho GLOBAL (Gizmos ▸ slider, 3D Icons); a Hierarchy mostra o ícone do 1.º
  componente (Camera); Cinemachine com Follow: a vcam não se arrasta («move o alvo, não a câmera»), edição em
  modo de edição instável (Solo); Position Composer: offset, dead/soft zone, damping editados por guias na Game
  view; a Scene view não segue o jogo (Cameras overlay = miniatura); Pixel Perfect «Run In Edit Mode».

## Plano da cura (onda 17b) — melhor que os dois

1. **Ícone:** o dobro (44 px), em píxeis de ecrã; propriedade «Icon Size» no painel da câmera, do PROJECTO,
   junto do formato no blob `jogo` → `JOGO_VERSAO` 2 (lê o 1), passos largos; o dedo = o rectângulo do ícone
   (não o anel); respeitar `Visibility`; sem a caixa do gizmo genérico por cima. (A9)
2. **Hierarchy:** ícone pelo TIPO do objecto (Camera, Light, vazio…) em `hero_bridge.rs:152`, com um selo
   «segue <alvo>» quando há Follow. Gate tipo→ícone. (A4)
3. **Miolo:** Down dentro da moldura da câmera selecionada fica PENDENTE; arrastar > limiar = mover a câmera;
   soltar sem mexer = o clique segue para a seleção (a lição da Godot sem perder o gesto). Gate pelo ponteiro
   (`App::pointer_button`) com o Sky por baixo. (A1)
4. **Câmera com Follow:** arrastar o miolo edita o OFFSET DO FOLLOW (a moldura anda em relação ao alvo, ao vivo
   no Inspector) e desenha-se uma linha tracejada do ícone ao alvo com o nome; dica uma vez: «segue Player:
   arrastar muda o Offset». Sem Follow: move a pose. (A2)
5. **Limites:** UMA edição `CameraFieldEdit::Limits{min,max}` (nunca duas no mesmo quadro). Gate: a edição pelo
   caminho da fila deixa os dois lados. (A3)
6. **Canto:** a altura a partir do centro VISTO no quadro (o re-apertado); o canto oposto fica no sítio quando a
   cerca empurra; gate que corre `camera_2d::update` com limites. (A7)
7. **Look Through ligado:** a conversão do ponteiro congela a câmera do editor no Down (ou as alças desligam-se
   com aviso). **Play no editor:** alças desligadas, moldura ténue sem rótulo. (A5, A8)
8. **Nomes:** «Offset» da câmera e do Follow com rótulos distintos (ex.: «View Offset» / «Follow Offset»); gate
   `ValueChanged`→barramento para os dois e um que mede a moldura a mexer com Follow+limites em `ticks==0`. (A6)
9. **Rótulo** dentro do canvas (encosta à borda visível); cursor/hover nas alças. (A10)
10. **Fica para a onda 18** (já decidido): zona morta pintada, miniatura, escurecer fora. Juntar «a vista do
    editor segue a câmera do jogo» (pedido de Godot e Unity) como opção.

## Estado (28/10)

Curado na onda 17b: [handoff](handoffs/HANDOFF_CONTINUACAO_2026-10-28_17b_CAMERA.md).
