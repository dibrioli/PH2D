# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-12 — F13: O MODO FORMA SELECCIONA, A *RESIZE BOX* GUARDA O TRAÇO, O *VIEWED*, A CAIXA BAIXA E O BICO NA PLACA DO MOTION

> Fecha **F9 + F10 + F11 + F12 + F13 juntas** (nenhuma está no `main`). O que a F9–F12 mudaram,
> mediram e deixaram aberto: [handoff da F12](HANDOFF_INTEGRACAO_line_UIUX_2026-10-12_F12_SETAS.md)
> (que cita os da F11, F10 e F9). Este cobre a F13. Pedido do dono palavra por palavra e o escopo
> A–E: [`22_prompt…`](../vetor_nova_ui/22_prompt_F13_selecionar_no_modo_forma_e_as_respostas.md).
> Janela intermédia: [continuação F13](HANDOFF_CONTINUACAO_line_UIUX_2026-10-12_F13.md).

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, sobre o `main` `b1dd77e477` (o `main` não andou
  desde o rebase de 12/10; 0 commits atrás).
- **`PROJECT_SCHEMA` NÃO mudou** — nenhuma migração. Nenhum contrato congelado tocado.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

A entrada UI/UX: «(F12)» → «o modo Forma que selecciona (F13)», smoke `…6..20`, «Último:» → este
handoff.

## §1 — O que mudou (F13)

| commit | o quê |
|---|---|
| `95da41c20c` · `21b4235311` | **o Polígono pelo relatório do dono** («Cutout estranho, Smooth invertido»): o *Cutout* em esquadria até ao centro do arredondado (`corner_style::normals_meet`), o *Smooth* com `1` = o redondo (`rounded_at_one`) sem apagar o raio do canto, o tipo do canto mantido no tecto do raio (`corner_live`, a tangente do segmento de origem). Doc 15 §3b |
| `f5b37f04c0` | **A — seleccionar no modo Forma**: o clique que acerta numa forma DESARMA (`after_the_shape_release`: `shape_armed = false`, mesmo sem a selecção mudar) e as alças dela voltam; no modo Forma os NÓS nunca se desenham (`hides_the_nodes`: `mode == Shape ‖ …`) |
| `63157a4bf9` | a cena NOVA `=20` (`vector_mode_seleciona`: estrela + elipse, a paleta arma um polígono, clique no meio da estrela, arma de novo, clique dentro da elipse; `PH2D_CENA20_FICA=0` pára na estrela); gate H4 — a forma sem `VecShape` é seleccionada sem alças e sem nós |
| `110a308292` · `c8c1219a79` | **B — a *Resize Box* MANTÉM a caneta** (o *Scale Strokes & Effects* do Illustrator desligado): `ph2d_app_vec::vec_frame_pen` (`keep_the_pen`, `resize_keeping_the_pen`), chamado pelo `vec_frame_resize::apply` da shell; a receita de uma forma viva segue a razão da GEOMETRIA. Doc 17 §2 |
| `d27c1ae575` | **C** — o ⇕ do *Viewed* (cubo, cone, pirâmide) sobe para «From above»: `HandleDesc::down` (`arrow_down`), só ele inverte |
| `669475ed5a` | **D** — caixa baixa: com `h/4 < park_at` a fila de ⇕ passa à meia altura, a começar um `RAIL_PITCH` por fora da borda (a alça do meio fica na borda) |
| `6403559be4` | **E** — a placa da GPU do Motion corta o bico além do limite (SVG 2 `miter-clip`): `corte_da_esquadria` em `shape.wgsl`, `contorno.wgsl` e no espelho `contorno_cpu.rs`; o alcance da esquadria passa a `√(limite² + 1)`; `PH2D_MITER_CLIP=0` chega à placa pelo SINAL do `limite_esquadria`. Doc 19 §5 (fim) |

## §2 — Medido e decidido (técnico, delegado)

- **A, as hipóteses do prompt:** H1 e H2 confirmadas VERMELHAS pela cadeia do produto antes da cura
  (`shape_mode_handles_tests.rs`); H3 FALSA (o `path_at` já conta o preenchimento: o clique no meio
  de uma estrela cheia selecciona-a); H4: sem `VecShape` não há alças (nada a mexer), os nós ficam
  escondidos (no modo Forma nada os arrasta), a caixa é a da selecção.
- **B, a lei:** a geometria escala o que FALTA à caixa visual (a margem do traço fica de cada lado) e o
  pivô é o ponto que deixa as DUAS bordas exactas. Onde a margem muda com a razão (o bico em esquadria
  de uma estrela esticada só num eixo), uma secante sobre o erro da largura visual: **medido 5–6
  passes** à tolerância `1e-6` da caixa (4 não chegam; o ponto fixo simples precisava de dezenas) ⇒
  `PASSES = 6`. Ficam com a caneta: a largura, os traços da pilha, o ladrilho da estampa do traço, o
  deslocamento e a dilatação das camadas. A escala LIVRE (sem *Resize Box*) não muda (`√|det|`).
- **D, a conta:** o quadrado da alça tem 12 px, o disco de acerto do ⇕ 24 px ⇒ tocam-se quando a
  distância vertical `h/4 < 6 + 12 = PARK_PX` (= `park_at`) — o limiar É a constante que já existia.
  Por fora, o 1.º ⇕ fica a 27 px da borda (o disco começa a 15 px; o quadrado acaba a 6 px).
- **E, o alcance:** as quinas do corte ficam a `√(d² + r²)` do vértice na ida-e-volta (`d = limite·r`)
  — `√17 ≈ 4,12` meias larguras com o limite 4, acima dos `4` de antes. Mudou no shader
  (`alcance_da_peca`) e no `eixo.rs` (o bloco e a extensão), a mesma função.
- **E, a régua dos gates de GPU:** o `pelo_vello` de `tests/it/paridade_com_o_vello.rs` desenhava a
  referência com o `stroke` CRU do Vello (que chanfra). Com o corte, `o_passe_desenha_o_que_o_vello_desenha`
  e `o_tracejado_esticado_desenha_o_que_o_vello_desenha` divergiam; com `PH2D_MITER_CLIP=0` os dois
  passavam ⇒ a diferença era SÓ o bico. Cura: a referência CHAMA o `stroke_uniform` da casa (a lei
  que o render da cena usa). 15/15 na placa.

## §3 — Foundational tocado e contratos

- `ph2d-tool-vector`: `HandleDesc` ganhou o campo `down` (público; os dois construtores literais da
  tabela escrevem-no). `ph2d-vec-render::miter_clip` reexporta `enabled` (a chave do corte para a
  placa). `ph2d-shape-gpu`: nenhuma mudança de layout (`EixoItem` 72 bytes, `Record` 352).
- A shell: `vec_frame_resize.rs` (+14 linhas: o campo `world_scale` e a chamada); a lei vive em
  `ph2d-app-vec`. `the_shell_only_shrinks` com folga.

## §4 — O que só a árvore combinada pega

- A `line/Vector` (se integrar antes): toca o `vec_frame_resize`/a *Resize Box*? Re-correr os gates
  `vec_frame_resize` e o `a_resize_with_the_stroke_in_the_box_keeps_the_pen_and_both_visual_edges`.
- Qualquer linha do Motion que mexa no `shape.wgsl`/`contorno.wgsl`: os gates de GPU do
  `ph2d-shape-gpu` são `#[ignore]` (precisam da placa) — correr `PH2D_GPU=1 … --test it -- --ignored`.

## §5 — Provas de mutação

⟨por preencher⟩

## §6 — Smoke (do dono)

O relatório ao dono (passos numerados) vai na conversa; as fotos da cena 20:
`/var/tmp/ph2d_F13_cena20/1_estrela.png` (a estrela seleccionada pelo clique: alças verdes, nenhum
nó) e `2_elipse.png` (a selecção passou à elipse).

## §7 — O que fica ABERTO

1. **Para o dono decidir (achados da auditoria do Polígono, doc 15 §3b, sem cura):** o *Cutout* no
   tecto deixa lascas de espessura zero (o rectângulo 2:1 fica um quadrado com dois «bigodes»); a
   seta dobrada com *Tail* > *Head width* perde as farpas (o OOXML prende a haste à cabeça); o *Check*
   com peso ≥ 0,5 mostra um degrau no topo do braço curto. Vão no relatório do smoke como perguntas.
2. D com TRAÇO GROSSO: a fila «por fora» mede-se da borda da CURVA; as alças da caixa vivem na caixa
   VISUAL. Com meia largura de traço ≥ 9 px no ecrã o 1.º ⇕ por fora encosta na alça do meio (medido
   pela conta: 27 − 12 − 6 = 9). O `handle_grip` não vê o traço (só o cozinhado) — curar pede passar a
   margem visual ao `view` de `shape_handles_live`.
3. A integração das DUAS linhas do Vetor (esta e a `line/Vector`) — por ordem do dono.

## §8 — Portão, binário e `agent-loop-profile.sh`

⟨por preencher⟩
