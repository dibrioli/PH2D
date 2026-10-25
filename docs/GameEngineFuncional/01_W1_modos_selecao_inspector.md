# W1 — os modos têm nomes coerentes, trocar de objecto sai do modo, o Inspector segue a selecção

> Linha `line/GameEngineFuncional` (aberta 24/10). Pedido do dono, 24/10, antes de aproximar a engine da
> Godot: *«garantir que quando um objeto é selecionado, seja na hierarquia ou seja no canvas, o painel
> correto seja aberto no inspector automaticamente […] mesmo no modo paint ou edit, deve ser possível
> escolher outro objeto […] se um objeto estiver sobre o outro, não deve ser possível selecionar o que
> está abaixo […] ao mudar de objeto abre o modo Object e seu painel […] trocar o nome no modo paint da
> imagem para Draw como Flip e fazer o mesmo para Vector (edit vira draw)»*.

## O que muda (e o que REVOGA)

| # | Lei nova | Revoga |
|---|---|---|
| 1 | **Os nomes:** Image = Object · **Draw** · Edit; Flip = Object · Draw · Edit; Vector = Object · **Draw**. `ObjectMode::Paint` sai do vocabulário (nenhum tipo o declara — D6). | spec/06 §3.4 (Image ▸ Paint, Vector ▸ Edit) |
| 2 | **Escolher OUTRO objecto num modo de criação** (clique no canvas, linha da Hierarquia, laço, intervalo) **selecciona-o e volta a Object**, seja do mesmo tipo ou de outro. A selecção fica a que o artista fez. Desseleccionar (clique no vazio) continua a NÃO sair (05/10). As partes DE DENTRO do objecto (os ossos do esqueleto) continuam a não sair. | escolha 4 de 03/10 (*«não trocar»*, o cadeado) e o «Edit do TIPO» de 05/10 para o clique (outra forma do vetor já não fica no modo) |
| 3 | **Num modo de criação o clique no canvas pega SÓ o objecto de cima** — o 1.º da porta única `pick_objects_at` (vetor › Flip › imagens, a ordem de composição), sem o ciclo de sobrepostos nem a preferência pela selecção. O modo Object fica como está. | — |
| 4 | **O Inspector segue a selecção:** em Object, quando o objecto activo muda (por qualquer porta) ou um modo acaba, o Inspector abre (se fechado) e vem à FRENTE do seu encaixe. Num modo, o painel do módulo já vem à frente (a ponte da ferramenta). | — (não existia: 10 cenas de smoke forçavam `bump_panel_z(INSP_PANEL)` à mão) |
| 5 | **A aba de cima pede o modo de UM tipo:** `CanvasOwner::Mode(kind, mode)` / `ModeRequest::Open(kind, mode)` — com o `Draw` partilhado por três tipos, a aba *Flip* abriria o Painter numa imagem activa. | — |

## Decisões técnicas (delegadas)

- A porta de selecção deixa de recusar: `mode_drive::refused` → `mode_drive::chosen`, que marca
  `ModeState::release` quando o alvo é outro objecto (lei pura `object_mode::switches`). O quadro consome a
  marca no passo 2, ANTES da rede `still_holds`, e larga o módulo sem repor a selecção (o `Step::Leave`
  repõe-na ao trancado — seria desfazer a escolha do artista).
- Pintar por baixo de outro objecto continua a pintar a imagem em edição: a ferramenta reclama o clique
  dentro da pegada ANTES do pick (`painter_canvas_down`). A lei 3 é sobre o que a SELECÇÃO pega.
- Com uma ferramenta de desenho na mão (pincel do Flip, caneta do vetor) o clique desenha — é dela; a
  troca pelo canvas acontece onde o clique chega ao pick (fora da pegada do Painter, a ferramenta Select do
  vetor/Flip) e sempre pela Hierarquia.

## Gates (red-first)

- `object_mode_tests`: `switches` (outro objecto · o próprio · parte de dentro · parte de um modo do tipo ·
  limpar) e `resolve(Open(kind, mode))` com o tipo errado.
- `mode_drive_tests`: no modo Draw de uma imagem, escolher outra imagem → Object e a selecção é a nova;
  no Draw do vetor, outra forma → Object; um osso no Edit do esqueleto → fica; o Inspector vem à frente.
- Seam com o Hero real: clique na linha da Hierarquia durante um modo → Object + Inspector à frente.
