# BUGS da Luz 2D — os que enganavam

> Um bug entra aqui quando o sintoma apontava para OUTRO lugar. O mecanismo, a medição e o gate
> que o guarda; o resto vive no handoff da onda.

## 1. «Com a luz ligada o app custa a maximizar e a encaixar na tela» (11/10)

- **O sintoma enganava:** parecia o compositor, a janela ou o layout do editor. Era a luz: a GRADE
  dela segue o tamanho da janela, e cada tamanho novo recriava o `Solver` inteiro — o maximizar anima
  o tamanho e o recriava a CADA quadro.
- **Medido** (o caminho do produto, `LuzDoQuadro::preparar` + `gravar`, maximizar de 12 quadros de
  1024×768 a 1920×1040, cena 3): ~1 850–2 100 ms por quadro que muda a grade contra 3–8 ms com o
  tamanho fixo; alocar texturas e buffers ~10 ms; o resto é compilar as pipelines da C3 («c3 fina»
  390 + 730 ms nas duas variantes, «c3 rebote» 120 + 210 ms). Recriar só o módulo com os layouts
  guardados saía rápido — o caro é compilar TUDO de novo (módulo, layouts e pipelines).
- **Por que nenhum gate o via:** todas as réguas da luz medem o tempo de PLACA (os carimbos) de um
  solver já montado, num tamanho fixo; o custo de nascer é de CPU e nunca entrou em nenhuma conta.
- **A cura:** `ph2d_light2d_gpu::Fabrica` — os módulos, os layouts e as pipelines guardados pela
  descrição inteira (o `Debug` do descritor traz a identidade dos objetos que referencia), partilhados
  pelos solvers de uma mesma `LuzDoQuadro` (`Solver::novo_com`). Uma grade nova só realoca. O zoom a
  atravessar oitavas e a troca de nível (a mesma família: grade nova) reaproveitam o módulo e os
  layouts — 2–10 ms medidos.
- **Gate:** `redimensionar_a_janela_nao_recompila_a_luz` (`tests/it/redimensionar.rs`): num
  maximizar, os solvers das grades novas compilam ZERO pipelines (antes 190 em 10 grades). Mutações
  4/4: sem a fábrica no editor; o módulo sem cache; o cômputo sem cache; a fábrica a esquecer a cada
  chamada — esta cegava a régua enquanto o contador vivia dentro do cache (ele passou para fora).
