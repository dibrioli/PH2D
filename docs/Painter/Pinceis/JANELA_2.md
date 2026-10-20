# JANELA 2 de 5 — Os motores: o que os pincéis do catálogo pedem e ainda não existe

> Leia ANTES: `00_MISSAO.md`, `ESTADO.md`, `HANDOFF_JANELA_1.md`, `03_MOTORES.md` (a lista de lacunas). Nasce em
> espera: responda «JANELA 2 pronta — aguardo o sinal» e pare até o `<cross-session-message>` com o SINAL.

## Tarefa

Implementar, por ordem de quantos pincéis do catálogo cada uma destrava, as LACUNAS de motor do `03_MOTORES.md`. Cada
lacuna é uma feature de produto completa (CLAUDE.md §0.6, DIRETIVA_IMPLEMENTACAO a cada passo):

1. **Medida contra o oráculo ANTES** (o que o Krita/MyPaint/Rebelle faz na bateria de gestos) — a lei vem da medição.
2. **Aditiva**: os 3 meios antigos e os 31 presets ficam byte-idênticos com ela desligada (prove com as miniaturas
   `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` e a cerca da linha).
3. **No painel** quando é um botão do artista (tokens/i18n, HR-15; a família vive em `crates/ph2d-app-painter`,
   a shell só encolhe), **no ficheiro do preset** (`CAMPOS_DO_PINCEL`/knob do registo) quando é do pincel.
4. **Portão + mutação** (`/pd-mutacao`, red-first) e **custo medido** (ms/quadro a 40 e 200 px; memória).

Candidatas prováveis (confirme no `03_MOTORES.md`): a ponta chata que GIRA com o arrasto (com inércia e memória entre
traços — o Flat Oily/Knife do Rebelle, C38); modos de mistura do pincel (Multiply/Screen/Overlay/Color/Luminosity —
o Marker/Effect); dual brush (forma × textura de outra ponta, Procreate/Photoshop); dinâmica de cor por dab e por
traço; dispersão/contagem/rotação aleatória (spray, folhagem, partículas); smudge/blender que arrasta a cor de baixo
(os borradores de todos os apps); taper real por pressão e velocidade; textura de grão em modo «textured» × «moving»;
e o que o catálogo pedir. As lanes do Jitter alto (C39: o Dabber rasgado) são decisão de lei — resolva-a medindo
contra o Dabber do Rebelle, não por gosto.

Paralelize: um agente `general-purpose` por lacuna independente (cada um com o briefing completo: árvore, regras, o
contrato, a lacuna, a medição do oráculo, o critério de pronto), no máximo 2 com compilação pesada ao mesmo tempo; o
coordenador revê cada diff (`/code-review` ou leitura) antes de aceitar. Commits por lacuna.

## Saída
Cada lacuna: código + portão + mutação + linha no `03_MOTORES.md` (feito, medido, custo). `ESTADO.md` actualizado
(o que ficou por fazer volta à fila com o porquê), `HANDOFF_JANELA_2.md`, SINAL à JANELA 3. Se sobrar tempo e
contexto, comece a fila da JANELA 3 em vez de parar.
