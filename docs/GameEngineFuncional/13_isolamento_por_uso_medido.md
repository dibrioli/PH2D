# 13 — Isolamento por uso: o que o dono viu e o que a Unity faz (medido, 26/10)

Depois do smoke da onda 12 e de comparar três engines com o MESMO nível (projetos em
`/home/enio/Documentos/Projetos/Comparacao_Engines/{godot,unity,gdevelop}`, a nossa arte em `…/arte`; os três
funcionam — dono, 26/10), o dono concluiu: *«Talvez não seja o desenho da UI o problema. Mas sim a falta de
clareza em cada seção, a falta de isolamento por uso.»* A decisão sobre o modelo Godot (componentes como nós)
fica em aberto; o custo medido está no fim.

## Os três defeitos que o dono apontou (causa lida no código, ainda NÃO reproduzida em teste)

1. **Platform High mostra «Texture & Atlas: H Frames 3 · V Frames 1 · Atlas».** O azulejo de fábrica guarda-se
   como 3 cópias lado a lado (`FactoryArt.grid [3,1]`, `tile: true`, célula do meio — o remédio do fio do mip,
   `ph2d-render/src/atlas/fabrica.rs`) e esse truque interno aparece ao artista como 3 quadros.
2. **Arrastar «Platform» dos Assets para o canvas dá outra coisa** (Individual, 1×1, aparência diferente):
   `shells/desktop/src/asset_drop_apply.rs::drop_spawn_image` COPIA os píxeis (`acquire_individual`) da imagem do
   AssetDb — que para a fábrica é a tira 3×1 — e não passa pela porta `image_choice::choose_library_image`.
   Duas portas para a mesma imagem.
3. **Trocar a Image de Platform High por Flag** mantém as proporções da plataforma (lei 4 da onda 12: sprite com
   *Repeat* sem `ScrollRepeat` guarda tamanho e repetições), aplica o corte 4×1 da bandeira parado num quadro, e
   não há como configurar a bandeira animada (o objeto não tem `SpriteAnimations`; as secções contradizem-se).

## A Unity, medida (Unity 6000.7.0b1 do dono, batchmode numa CÓPIA; script e logs em
`/home/enio/Documentos/Projetos/Comparacao_Engines/unity_experimentos/` — `Assets/Editor/Experimentos.cs`,
`exp.log`, linhas `[EXP]`)

| caso | resultado |
|---|---|
| Simple, escala 2, trocar a imagem | o tamanho SEGUE a imagem nova, a escala fica (plataforma 2×1 → bandeira 2×4 → herói 2×2) |
| Tiled size 3×0,5 → bandeira / montes | o `size` FICA; ladrilhos ao tamanho nativo da nova ⇒ bandeira cortada (3×0,25 ladrilhos), montes um recorte; Continuous = Adaptive; Tiled→Simple reescreve a escala |
| Sliced com bordas → imagem sem bordas | estica a imagem inteira, SEM aviso; as bordas são da imagem |
| re-cortar a folha (4→2 colunas) no importer | todos os objetos mudam; quem usava o quadro 3 fica VAZIO sem erro; a animação perde chaves |
| Animator a tocar | a animação SOBRESCREVE o campo Sprite (simulado: SampleAnimation / AnimationMode / Animator.Update) |
| BoxCollider2D Auto Tiling | inconclusivo em batchmode |
| Sprite Atlas V2 | a escolha vive no ASSET do atlas (lista de imagens/pastas, Include in Build, padding 4, Tight); nem a imagem nem o objeto têm campo de atlas; o renderer não muda |
| Pixels Per Unit no importer | todos os objetos que usam a imagem mudam |

**Onde mora cada escolha na Unity:** objeto = qual imagem/quadro, Draw Mode, size, flip, cor · imagem (importer)
= corte em quadros, PPU, bordas 9-slice, wrap, pivô, Extrude Edges (o remédio do fio de mip DENTRO da imagem) ·
atlas = que imagens se juntam · animação = sequência e tempos, manda no campo Sprite.

## Decisões do dono (26/10)

- ⛔ **A estratégia Atlas/Individual FICA no objeto como está** (secção Texture & Atlas). Não a mover para a imagem
  nem escondê-la.
- O modelo Godot (componentes como nós): adiado; os custos medidos — (A) só apresentação 2–3 ondas, risco baixo;
  (B) nós reais para Sprite/Colisão/Câmara/Som 4–6 ondas, PROJECT_SCHEMA + migração; (C) tudo é nó 10+ ondas —
  e o histórico (o modelo Unity foi escolhido em 24/08 sob o argumento de que o Bevy está mais perto da Unity; o
  dono diz que a escolha foi induzida pela LLM).

## A onda 13 (proposta, a confirmar com o dono)

Isolar por USO sem mexer na estratégia: (1) o corte em quadros e o tamanho nativo passam a ser da IMAGEM na
biblioteca (definido uma vez; a fábrica já os tem em `FactoryArt`); (2) o objeto mostra só o que o seu uso pede —
imagem parada, repetida (bloco/fundo) ou animada — com a estratégia Atlas/Individual na secção de sempre; (3) UMA
porta para meter imagem (linha Image, seletor, queda na ranhura, queda no canvas, script); (4) o truque das 3
cópias deixa de aparecer como H Frames 3 (remédio do mip dentro da imagem, como o Extrude Edges da Unity); (5)
imagem animada num bloco repetido ⇒ o PH2D AVISA e oferece tornar o objeto animado (a Unity corta em silêncio).
