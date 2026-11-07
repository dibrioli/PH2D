# Degraus `172 → 176` da escada do `PROJECT_SCHEMA`, arquivados verbatim

Saíram de `shells/desktop/src/project_schema.rs` em 2026-11-07 (`line/GameEngineFuncional02`, onda 5): pagam a catraca da shell, e tratam do 3D retirado (ADR-0179).

```text
/// # `172 → 173` — O CATAVENTO: a malha que um sprite mantem VIVA, e a POSE 3D dela
///
/// O `ph2d_ecs::Mesh3D` entra no registo (a rota B do
/// `docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`): a malha deixa de ser assada uma vez e
/// passa a rasterizar POR QUADRO para o G-buffer, de modo que **virar o objecto faz a luz
/// acompanhar**.
///
/// ⭐⭐⭐ **A POSE mora no componente e nao no `Transform`, e isso e' uma MEDICAO.** Aquele tem
/// `rotation: f32` e exprime **apenas** a rotacao no plano do ecra' — e a §5.0 do catavento mediu
/// que *essa* a rota A ja' da' **exactamente**: uma rotacao `R` em torno do eixo da vista leva as
/// normais a `R·n` e a imagem a `R·imagem`, duas operacoes 2D sobre o plano ja' assado (`0,00°` de
/// desacordo, contra `28,58°` de um plano fixo). Fora do plano nao ha' operacao 2D nenhuma
/// (`31,69°`) ⇒ *a rotacao que justifica esta rota e' exactamente a que FALTA aquele campo*.
/// Tabelas: `docs/Render3d/17_a_rota_b_o_catavento.md` §1.5.
///
/// ⛔⛔ **A PRESENCA da componente e' a decisao, e nao um `live: bool`:** o `02.2` diz que a escolha
/// entre as duas rotas e' *«uma propriedade do objeto»*, e ter ou nao ter **e'** essa propriedade.
/// Um campo ao lado seria um segundo sitio a dizê-lo, e os dois divergiriam no primeiro dia em
/// que alguem escrevesse um sem o outro.
///
/// ⛔⛔ **E nao ha' `MeshShading`**, apesar de o `02.2` o desenhar: a
/// `ph2d_form_donation::lei_da_luz::material_da_forma()` **nao recebe argumentos** — o material e'
/// GLOBAL — e a escolha de sombreamento que ja' e' por objecto **e ja' e' gravada** e' a `Lei` do
/// degrau `161`. *Seriam quatro knobs sem consumidor*, que e' o defeito que esta casa caca com
/// censo.
///
/// ⛔ **Sem degrau de migracao** (decisao do Enio, 26/08): um v161 e' recusado em voz alta.
/// ⚠️ **A tripla NAO ve^ este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `173 → 174` — **o catavento GIRA** (`Mesh3D::spin`, 2026-09-21)
///
/// Um campo `f32` no componente do degrau anterior: **voltas por segundo**. ⚠️ **O postcard e'
/// POSICIONAL**, logo um ficheiro gravado em `162` (sem o campo) lido em `163` sairia errado **em
/// silencio** — e e' por isso que um campo num componente que ja' viaja custa um degrau, mesmo no
/// dia seguinte ao que o criou.
///
/// ⛔⛔ **O angulo efectivo e' DERIVADO por quadro e nunca escrito de volta** (`Mesh3D::yaw_em`):
/// um componente registado reescrito a 60 Hz seria **um passo de `Ctrl+Z` por quadro**, que e' o
/// defeito que o `preview_drive` desta casa existe para impedir. ⇒ o `yaw` gravado continua a ser
/// o que o artista AUTOROU, e o giro compoe-se com ele na leitura.
///
/// ⚠️ Com `spin = 0` a peca fica parada no `yaw` autorado, **ao bit** — e ha' gate.
///
/// # `174 → 175` — **o que se ve^ e' o que se assa** (`BakedFormDocument::recorte`, 2026-09-21)
///
/// ⛔⛔ **Report do dono, com foto:** *«O Bake nao e' feito projetando o objeto 3d exatamente como
/// o posiciono sobre a sprite e tem perspectiva, posicao e escala diferente do que eu coloquei»* —
/// e, a seguir, *«o bake 3d recebe zoom e fica como na imagem: um fundo deslocado do objeto 3d»*.
///
/// A porta de assar rasterizava a malha no alvo INTEIRO, logo a peca ocupava, dentro dos texels do
/// sprite, a mesma fraccao que ocupava **da altura do VIEWPORT** — e o sprite e' um rectangulo
/// *dentro* dele. O erro de escala e' exactamente `altura da vista ÷ altura do sprite no ecra'`, e
/// o de posicao e' a distancia entre os dois centros.
///
/// ⭐ A cura e' um frustum **fora do eixo** (`ph2d_mesh_render::ViewRegion`), e o campo novo e' a
/// memoria dele: `aspect` da vista + o rectangulo do sprite em FRACCAO dela.
///
/// ⛔⛔ **Ele TEM de viajar no arquivo por causa da rota B:** um catavento re-rasteriza a forma
/// **por quadro**, e sem esta memoria ele voltaria a encher o sprite no primeiro quadro em que o
/// relogio andasse — a peca SALTAVA ao reabrir o projecto.
///
/// ⚠️ **`None` num documento anterior**, e a ausencia e' a resposta: ate' aqui a forma era sempre
/// rasterizada com a vista inteira, logo um ficheiro velho reabre **sem uma linha de diferenca**.
/// ⛔ Ainda assim o degrau existe, e o motivo e' o de sempre: *o postcard e' POSICIONAL*.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `175 → 176` — **a materia da peca** (`BakedFormDocument::materia_da_forma`, 2026-09-21)
///
/// ⛔⛔ **Report do dono, com foto e uma seta:** *«O algoritmo que vc criou tem esse fundo branco
/// na sprite transparente. logo que roda o objeto o fundo aparece. OU seja: parece que vc criou
/// uma mascara.»*
///
/// ⭐⭐ **O que ele viu NAO era uma mascara — era uma SILHUETA CONGELADA.** Quando o sprite chega
/// **inteiramente transparente**, o bake veste-o da forma (branco, com o alfa da COBERTURA) e
/// grava isso no `base`, que e' o que o arquivo guarda. A rota B re-rasteriza a forma **por
/// quadro**, logo a luz e as normais seguem a peca a virar… e o alfa NAO, porque ele vinha de um
/// `base` do PRIMEIRO gesto. *A peca rodava por baixo do recorte dela propria.*
///
/// ⭐ A cura e' um facto **por OBJECTO** (`vestidos > 0` no assar): com ele o albedo e' o neutro e
/// o alfa e' a cobertura DESTE quadro. ⛔ Uma regra **por texel** (*«onde o base e' transparente,
/// usa o alfa da forma»*) parece equivalente e nao e': num sprite com arte DESENHADA ela encheria
/// de branco toda a volta do desenho sempre que a malha fosse maior do que ele.
///
/// ⚠️ **`false` num documento anterior**, e ele e' o valor CERTO para todos: ate' aqui nenhum bake
/// sabia disto, e um sprite com arte nunca o quer. ⛔ O degrau existe pelo motivo de sempre — *o
/// postcard e' POSICIONAL*.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
```
