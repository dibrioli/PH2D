# ADR-0176 — O modo Render do modelador desenha MALHAS com luz de jogo; o campo continua a FONTE

- **Status:** Aceito (2026-10-02, ordem do dono).
  ⚠️ **O número 0176 é provisório:** foi lido de `ls docs/architecture/decisions` (último = 0175) na
  base `1ad60a1ce`. O integrador **reconta** na integração.
- **Data:** 2026-10-02
- **Linha:** `line/3DModeling`
- **Cofre do módulo:** [`docs/3DModeling/`](../../3DModeling/README.md)
- **Handoff da wave:** [`HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md`](../../3DModeling/handoffs/HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md)
- **Emenda:** [ADR-0161](0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)
  — a cláusula *«o que o artista vê é o campo traçado»* deixa de valer **só para o modo Render**.
  O Matcap continua traçado; o campo continua a fonte de edição.

## O problema

O dono, sobre o Render traçado: *«ainda não está em tempo real como uma game engine»*. Quatro
sintomas, todos **só no Render** (o Matcap está bom):

1. borrado ao mexer · 2. engasga ao girar · 3. demora ao editar · 4. granulado que some com o tempo.

Alvo dito: nível **Fortnite / Plants vs Zombies**, e **a engine tem de rodar em MOBILE**.

Decisões do dono (a ordem é dele, não derivada):

1. Ao entrar no Render os objetos **viram malhas**. Objetos que se sobrepõem numa booleana
   **FUNDEM-SE**; peça solta (não toca ninguém) = **objeto separado**.
2. O **chão** (só recebe sombra) fica.
3. O Render traçado **sai** quando o rápido for aprovado. **Mover no Render move DE VERDADE**
   (escreve o `FieldPose`); peça que nasceu de corte/espelho fica **travada, com frase**.

## A decisão

- No modo **Render**, a imagem é uma **malha extraída do campo** e desenhada por um desenhista de
  jogo (forward, numa passada, mapa de sombra, oclusão assada). A malha é **derivada, de vista, não
  gravada**: o campo é a fonte, `PROJECT_SCHEMA` **intacto**.
- O ponto de entrada de dados é o mesmo (`FieldDoc`); o que muda é o que a vista ensina ao ecrã.
- `PH2D_FIELD_SHADING=render` abre já no Render. ~~`PH2D_FIELD_RENDER_TRACADO=1` volta ao
  traçado~~ — **saiu em 03/10** com o Render traçado (ver abaixo).

## Estado da arte (o que cada um faz, e o que isso nos deixa)

| Quem | O que faz | Serve a nós |
|---|---|---|
| Oculus Medium / Substance 3D Modeler | edita SDF, **exibe triângulos gerados** | ⭐ precedente exato |
| Dreams | tentou polígonos e raymarch, ficou com *splats*; mas edita **AO VIVO durante o jogo** | aqui a edição é no Matcap; o Render só mostra |
| Blender | Cycles progressivo × Eevee raster; *Remesh Sharp* = dual contouring | mesma divisão: lento-fiel × rápido |
| Godot Mobile / Unreal mobile (Fortnite móvel sem Lumen/Nanite) / URP | forward numa passada, mapa de sombra, oclusão assada, MSAA | é o orçamento que o mobile paga |

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| **Manter o traçado otimizado** | já otimizado ao limite em set/2026 (oclusão no tempo, resolução dinâmica) e **continua a acumular entre quadros** (borrado/granulado são a natureza dele); não roda em WebGL2 (precisa compute + storage) |
| **Malha por `fidget::mesh`** | recusada em W0/W19: quina serrilhada e face dobrada |
| **Render novo + traçado como «final»** | dono: só o rápido interessa |

## Consequências

- Crate nova **`ph2d-mesh-forward`**: `Features::empty()` + `Limits::downlevel_webgl2_defaults`;
  **texturas em vez de storage**; **6 pipelines compilados UMA vez** (nada compila ao editar).
- **Style e Bloom foram portados em 02/10** ao desenhista de jogo: o Bloom pela lei `ph2d_bloom::wgsl`
  em passes de desenho; o Style por `ph2d_style::wgsl` por pixel, com a curvatura assada por vértice.
  Ver [handoff O_BRILHO_E_O_ESTILO](../../3DModeling/handoffs/HANDOFF_line_3DModeling_O_BRILHO_E_O_ESTILO_2026-10-02.md).
- **O céu fotográfico (HDRI) entrou em 03/10** (crate `ph2d-sky`): substitui a parte sem caixa do
  céu; a luz-chave com sombra fica. Ver [handoff O_CEU_DE_VERDADE](../../3DModeling/handoffs/HANDOFF_line_3DModeling_O_CEU_DE_VERDADE_2026-10-03.md).
- **O sol do HDRI é a luz-chave desde 03/10**, com a sombra na direcção dele (mapa de faces de trás,
  PCSS em 3 níveis), medida contra o Cycles. Ver [handoff O_SOL_E_A_SOMBRA](../../3DModeling/handoffs/HANDOFF_line_3DModeling_O_SOL_E_A_SOMBRA_2026-10-03.md).
- **Kill-criterion escrito ANTES de medir:** quadro **≤ 8 ms** a 1080p e entrada no Render **≤ 1 s**.
  Medido: quadro **1,2–2,4 ms**, entrada **0,03–0,44 s** (a de 0,44 s a `load` 6–20 — re-medir calmo).
- A malha tem de **concordar com o campo** (pose, partição, material): a lei vive nos gates do
  handoff §4, não aqui.
- ⭐ **O Render traçado SAIU em 2026-10-03** (ordem do dono: *«lembre-se que buscamos o padrão
  Unreal/Fortnite ou Plants vs Zombies. Pode apagar o render antigo»*): o sombreamento de CPU
  (`shade_render` e a sua luz, sombra, oclusão, ricochete, chão, SSS e sondas), o pintor de material
  da placa (`ph2d-field-gpu::paint*`, céu no tempo, sondas) e a luz/céu/chão do shader da marcha.
  Sem placa, o modo Render mostra o **Matcap**; nenhum caminho de produto chega ao traçado (gate
  `render_sem_tracado_tests`). O Matcap continua **traçado** (ADR-0161). Ver o
  [handoff O_RENDER_ANTIGO_SAI](../../3DModeling/handoffs/HANDOFF_line_3DModeling_O_RENDER_ANTIGO_SAI_2026-10-03.md).
