# 07 — Auditoria de arrumação do cartão Ray Sensor (dono 15/11)

> Ordem do dono (15/11, com captura do cartão em cartões: «Layer» ao lado do «Reach», «Sees» depois dos botões
> «Rename layers… / Open in tab…»): *«Layers e Sees deveriam ficar numa seção própria mais próximos. Auditoria de
> arrumação para organizar cada coisa no seu lugar no cartão ray».*
> Molde: o Inspector do `RayCast2D` do Godot, de cima para baixo por PERGUNTA. Vale nos cartões e em `PH2D_CARDS=0`.
> Só muda o LUGAR: ids, eventos e canais iguais. Gate: `seam_ray_arrumacao::o_cartao_ray_esta_arrumado_por_pergunta`.

## Linha a linha

| Linha | Pergunta que responde | Onde estava | Onde está agora | Porquê |
|---|---|---|---|---|
| Queixa («Sees nothing right now», «This ray is off»…) | Porque é que ele não vê? | 1.ª, solta | 1.ª, solta | quem não vê nada mexer não quer afinar nada antes disto |
| Leitura («Sees X, at Y») | O que vê agora? | solta, após a queixa | igual | a razão de a secção existir; leitura viva, não controlo |
| Relógio parado | A leitura é de agora? | solta | igual | contexto da leitura |
| Enabled | Está ligado? | 1.ª linha de controlo (cartões: ao lado do Angle) | igual | Godot/Unity: ligar antes de afinar |
| Angle | Para onde olha? | após Enabled | igual (escondido com mira) | — |
| Turns with facing | Vira com o personagem? | após Angle | igual (escondido com mira) | é do Angle |
| Aim at | Para onde olha (alvo)? | após Angle | igual | a outra resposta a «para onde» |
| Reach | Até onde? | cartões: **ao lado da Layer** | cartões: ao lado da **Shape**; clássico: empilhado | «quão longe» e «com que forma» são a mesma pergunta (como olha); a Layer é outra (com quem colide) |
| Shape | Com que forma? | linha própria, após Reach\|Layer | ao lado do Reach | idem |
| Thickness | Quão grosso (Thick)? | por baixo da Shape | igual | só a forma activa mostra número |
| Cone | Quão aberto (Cone)? | por baixo da Shape | igual | idem |
| Layer | Em que camada está? | cartões: ao lado do Reach; clássico: após Reach | **grupo «Layers»**, 1.ª linha | a camada e o que ela vê são uma pergunta só |
| Sees (+ Add layer) | Que camadas vê? | cartões: **depois** dos botões, longe da Layer | grupo «Layers», logo **por baixo da Layer** | a queixa do dono: Layer e Sees juntos |
| «Applies to everything on …» | Isto vale para quem? | por baixo do Sees | igual (dentro do grupo) | é a frase do Sees |
| Rename layers… | Como chamo as camadas? | cartões: **entre** Layer e Sees | grupo «Layers», depois do Sees | ação sobre o que se acabou de ler, não interrompe a pergunta |
| Open in tab… | Onde vejo a matriz toda? | idem | idem | idem |
| Rename (campos + Done) | — | no lugar do Sees e dos botões | igual, dentro do grupo | modo, sem mudança |
| Filter (Ignore my body, Ignore my other parts, Ignore up to, Also ignore, Only see, Tag, Walls block, Hit triggers, Starts inside, See all hits) | O que conta como visto? | grupo «Filter», fechado | igual, após «Layers» | sem mudança por dentro |
| Signals (On Enter, On Exit) | A quem conta? | grupo «Signals», último | igual | sem mudança |

O grupo «Layers» é novo (`INSP_RAY_FOLD_LAYERS`, `ph2d_editor_core::ids`), nasce ABERTO, com dica
(`panel.inspector.ray.layers_tip`); fechado mostra o resumo «Default · sees Default, Ground» (`layers_summary`,
`layers_sees_nothing`). Código: `crates/ph2d-panel-inspector/src/sections/ray_camadas.rs`.

## Outros achados

- **O losango entre «Reach» e «Layer» (a seta do dono).** É o losango do canal do **Reach** (Auto Key / Timeline).
  Medido na foto `26_arrumado.png`: cada losango de uma linha a dois fica no canto superior DIREITO do próprio campo,
  alinhado à borda direita da caixa dele — o mesmo em Enabled\|Angle e em todo o app em cartões. Pertence ao campo
  dele; o que o fazia ler como do vizinho era o rótulo «Layer» colado logo a seguir no vão de coluna. Agora o vizinho
  é «Shape», da mesma pergunta. Nada mudado no losango (é o padrão da casa, `paint_label_row`).
- Coluna do rótulo: a `Seccao` continua medida pela secção INTEIRA (com a Layer dentro), por isso no clássico a coluna
  não salta ao abrir/fechar «Layers».
- Espaçamento: o grupo pede a folga em cima como Filter e Signals (`grupo_dobravel`); nada sobreposto nem cortado nas
  três fotos. Cortes com «…» só os da casa no clássico («None: the Angle…», «wall_ah…»).
- Fotos: `fotos_07/26_arrumado.png`, `26_arrumado_cards_off.png`, `26_arrumado_cone.png`.
