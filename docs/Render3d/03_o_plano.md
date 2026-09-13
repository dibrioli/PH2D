# 03 — O plano: oito waves, cada uma com o que se mede

⚠️ **A ordem é por SALTO VISUAL POR UNIDADE DE TRABALHO**, e não por dificuldade nem por vontade.
Cada wave acaba num **smoke** (a lei da casa) e traz a régua que a julga.

⛔ **Nada aqui está construído.** Este é o plano que o estudo produziu.

## W1 — A gestão de cor (o fundamento barato)

Espaço de trabalho **linear** declarado, **exposição** autorada (em *stops*), e um tonemapper a
sério (**AgX** ou **ACES**), com a saída em sRGB.

- **Porquê primeiro:** é a diferença entre «parece de 2005» e «parece moderno», e **tudo o que vem
  depois mede-se errado sem ela** — um bloom sobre valores que não são HDR é um borrão.
- **Já temos:** `tonemap.wgsl` e `bloom.wgsl` no `ph2d-render`.
  ⛔ **Remedido em 13/09 ([`04`](04_a_remedicao_contra_a_arvore.md) §1):** o tonemap está em
  **bypass** e o `game_rt` é **partilhado com a arte 2D**; o modelador pinta **matcap** fora do HDR.
  ⇒ a `W1` sozinha não tem consumidor honesto, e a primeira fatia que se vê é o modo *Render* do
  modelador com `W1` + `W2`/`W3` mínimas juntas (`04` §4).
- **Falta:** o espaço de trabalho **declarado**, a exposição como número do artista, e a escolha do
  tonemapper como chip.
- **Régua:** uma rampa de luminância de `0` a `16` tem de sair monótona e sem clipping colorido; e o
  branco de `1,0` tem de cair no mesmo pixel antes e depois de mudar a exposição em `+1` e `−1`.
- **Smoke:** a mesma peça com exposição `−2`, `0`, `+2` lado a lado.

## W2 — A superfície OpenPBR

Uma crate `ph2d-material` com os **41 números** do `open_pbr_surface`, e o shader **gerado** do
padrão (ver `00` §3).

- **A 1.ª medição da wave é a ponte para WGSL**, e ela tem três candidatos por medir: **Slang → SPIR-V**,
  **GLSL → `naga`**, ou um `GenWgsl` próprio. ⚠️ *Escolher sem medir os três é o erro que o §0 proíbe.*
  ✅ **MEDIDA em 13/09 ([`04`](04_a_remedicao_contra_a_arvore.md) §2):** `WgslShaderGenerator` →
  `naga` `glsl-in` valida (`5 598` linhas); a rota por SPIR-V reprova no fragmento; o `GenWgsl`
  próprio não é preciso. ⚠️ Validar não é sombrear igual — o `MaterialXView` continua a ser a régua.
- **Lobos mínimos para o alvo:** base difusa · especular GGX com **compensação de energia** ·
  metalness · **coat** · emissão. O `transmission` e o `fuzz` entram depois.
- ⭐ **O `subsurface` já existe** (`sss.rs` + tabela pré-integrada) e passa a ser **uma entrada do
  OpenPBR** em vez de um sistema ao lado.
- **Régua:** o *furnace test* — sob um ambiente branco uniforme, uma esfera de `base_color = 1` e
  qualquer rugosidade tem de devolver branco. *É o teste que apanha energia perdida, e não perdoa.*
- **Oráculo:** o `MaterialXView` renderiza o mesmo material; comparam-se as imagens **por passo**.

## W3 — O céu como FONTE de luz (IBL)

Ambiente pré-filtrado: irradiância difusa + especular por rugosidade + a BRDF integrada.

- **Porquê agora:** é o que põe cor no lado escuro sem o lavar, e é o que faz o metal existir.
- **Substitui** o `env_ambient` constante do `ph2d-light`, que é a razão de a peça de hoje flutuar.
- **Régua:** a mesma esfera contra o mesmo ambiente, comparada com o `MaterialXView`.

## W4 — Sombras que POUSAM o objecto

Cascatas para o sol + **endurecimento no contacto**.

- ⚠️ **SSAO não é isto**, e o `01` §4 explica porquê — nós já temos SSAO e o objecto continua a
  flutuar.
- **Régua:** um objecto a `0`, `1` e `10` cm do chão tem de dar três sombras diferentes.

## W5 — ⭐⭐⭐ A luz indirecta, traçada contra o NOSSO campo

A wave que decide se a engine é bonita, e a que só nós podemos fazer assim (ver `02` §5.1).

- **Candidato principal:** **cascatas de radiância** com sondas esparsas — sem ruído, logo sem
  denoiser, que é o que preserva um look de cores chapadas.
- ⭐ **O traçado percorre o campo de distância verdadeiro** (`ph2d-field-eval`), não um proxy.
- ⛔ **O preço está medido e é o risco da wave:** o quadro de movimento do modelador custa `26,7 ms`
  contra um orçamento de `16,7`, e a marcha é `80 %` disso. *A wave começa por medir quanto de GI
  cabe, e o resultado pode ser «cozida e não em tempo real» — que é uma resposta legítima.*
- **Régua:** a caixa de Cornell. Ela tem resposta conhecida e não deixa mentir.

## W6 — A AUTORIA: o grafo MaterialX no módulo de nós

O artista vê **um nó** com a foto que o dono mandou. Quem quiser mais, abre o grafo — e o grafo é
**MaterialX**, logo entra e sai do Blender, do Houdini e do Substance.

- ⭐ O módulo de nós desta casa já tem cartão com params, paleta com busca, e a lei do *nenhum knob
  morto*. **A ligação é a tabela de 807 nodedefs como DADO**, que é o padrão que já deu o único
  painel `42/42` limpo do repositório.
- **Régua:** um `.mtlx` do Blender abre aqui e devolve os mesmos pixels que o `MaterialXView`.

## W7 — O pós

Bloom sobre HDR verdadeiro · profundidade de campo · anti-aliasing temporal.

## W8 — ⭐ A camada de ESTILO (a que faz *aquele* jogo)

Os botões para mentir de propósito, por cima de um pipeline honesto: **rim light** autorada, tinta
por **curvatura**, **grade de cor por zona**, saturação da luz indirecta, contorno onde o artista o
pedir.

- ⚠️ **É a última de propósito.** Estilo sobre um pipeline sem `W1` e `W5` é o protótipo que o
  `01` §4 nomeia.

---

## A ordem, num parágrafo

**`W1` e `W2` são o fundamento e são baratas.** `W3` e `W4` fazem o objecto existir no espaço. `W5`
é a wave grande e é a nossa vantagem estrutural. `W6` é o que responde ao *«intuitivo para
artistas»*. `W7` e `W8` são o acabamento — e o `W8` é o que faz a engine parecer-se com o alvo do
dono em vez de parecer-se com toda a gente.

## ⛔ O que este plano NÃO faz, e porquê

| ausência | razão |
|---|---|
| Nanite / geometria virtualizada | resolve um problema que não temos (`02` §3) |
| ReSTIR / hardware de raios | a Unreal chegou primeiro e melhor; e produz ruído, que é o inimigo deste look |
| um modelo de material próprio | o padrão está no disco e gera o código (`00`) |
| cel-shading como ponto de partida | é a camada `8`, não a `1` |
