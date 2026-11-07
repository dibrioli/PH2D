---
name: project_scene_tab_follows_object_transport_in_bar_2026_11_07
description: "Dono 07/11: aba Scene nova, a aba acompanha o tipo do objeto (Animate fica); menu Run sai, 5 botões no centro da barra"
metadata:
  type: project
---

Dono, 07/11, depois do smoke OK do Animated Sprite (ficha `fechado`): nova aba **Scene** (mover e arrumar, sem ferramenta de desenho), a de fábrica. A aba acompanha o objeto selecionado: Image/Skeleton/PixelSprite/Empty = Scene · Vector = Vector · Painted = Draw · Flip = Flip · Motion = Nodes; em Animate não muda. Selecionar só escolhe: desenhar pede Tab. Clicar numa aba continua valendo até a próxima troca de objeto. Menu **Run** saiu; no centro da barra de menus: Play in a Window · Play Fullscreen · Play in the Editor · Pause · Stop (como a Godot). Stop = o antigo Rewind.

Decisões do smoke: ▶ não grava chave; Frames/s, Cycle e Rules não gravam; aviso de imagem trocada fica salvo até responder; Look dobrado não mostra nada; 17 campos ok se nenhum essencial sumiu.

**Why:** com a Vector salva, o programa abria a ferramenta dela com o Player selecionado e o painel Pixel na frente.

**How to apply:** revê o D3 de [[project_card_ui_reference_order_2026_11_02]] só no «o layout é do utilizador»: o clique na aba vale, mas a troca de objeto manda. A troca não fecha painel aberto pelo artista (só a diferença entre as listas das duas abas). Sem seleção ao abrir, a aba salva continua valendo.
