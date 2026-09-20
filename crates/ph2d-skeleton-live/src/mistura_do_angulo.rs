//! ⭐⭐⭐ **QUAL LEI DE ÂNGULO O PRODUTO USA** — a porta de bissecção da wave de 2026-09-20.
//!
//! ⚠️ **Ficheiro próprio por tecto de LOC** (o [`crate::skin_live`] foi a `714` contra `700`), e o
//! corte é por RESPONSABILIDADE: aquele re-coze a forma a cada quadro e este responde a UMA
//! pergunta, *«que lei mistura as rotações?»*. ⛔ Nunca por uma entrada no `FILE_OVERAGE_OK`.

/// ⭐⭐⭐ **A LEI DO ÂNGULO QUE O PRODUTO USA** — `PH2D_SKIN_ANGULO=1` liga a média DESDOBRADA.
///
/// Ela nasce **DESLIGADA**, e a lei da casa é essa: *tudo o que é novo shipa desligado*. Com ela a
/// `0` (ou ausente) a saída é **byte-idêntica** à de sempre, e há gate a afirmá-lo.
///
/// # ⛔⛔⛔ O que ela compra — e ela é MUITO menor do que a 1.ª leitura dizia
///
/// | na barra da cena do dono | hoje (`Circulo`) | com ela (`Desdobrado`) |
/// |---|---:|---:|
/// | a aresta de DENTRO do cotovelo, a `90°` | `0,0496` | **`0,2524`** (`5,1×`) |
/// | a quina numa aresta que em repouso é RECTA, a `90°` | `180°` (bico) | **`67,5°`** |
/// | o BICO (`θ̄′·r = 1`) | `93°` | **`121°`** |
/// | a forma a DOBRAR sobre si mesma | `93°` | **`108°`** |
/// | a aresta visivelmente esmagada (`< 25 %`) | `77°` | **`91°`** |
/// | a área | `100,02 %` | `100,03 %` |
///
/// ⛔⛔⛔ **E AGORA A COLUNA QUE FALTAVA — quanto ela move o DESENHO:**
///
/// | dobra | p50 | MÁX | % da ESPESSURA (`1,0`) |
/// |---|---:|---:|---:|
/// | `45°` | `0,0000` | `0,0049` | **`0,5 %`** |
/// | `70°` | `0,0000` | `0,0192` | **`1,9 %`** |
/// | `90°` | `0,0000` | `0,0426` | **`4,3 %`** |
/// | `110°` | `0,0000` | `0,0820` | `8,2 %` |
/// | `150°` | `0,0000` | `0,2504` | `25,0 %` |
///
/// ⚠️⚠️ **As duas tabelas são verdade ao mesmo tempo, e a lição é de RÉGUA.** O `5,1×` é um MÍNIMO
/// sobre o contorno — um extremo **LOCAL** —, e um segmento de `500` pode multiplicar-se por cinco
/// sem a forma mudar de aspecto: desenhadas lado a lado, as duas leis ficam **quase uma em cima da
/// outra** (`diag_b_desenha_as_duas_leis`, e o `p50 = 0,0000` diz que METADE do desenho não se
/// mexe). *É a família do `edge_max` cego ao quad fino, do outro lado: uma régua LOCAL não diz o
/// TAMANHO do que se vê.* ⛔ Eu reportei o `5,1×` ao dono antes de desenhar, e foi a imagem que me
/// corrigiu.
///
/// ⛔⛔ **E ACIMA DE `108°` ELA É PIOR** na mesma régua (`110°`: `0,0868` contra `0,3030`;
/// `130°`: `0,0642` contra `0,4367`). Ali as duas já dobraram sobre si mesmas, logo nenhuma está
/// certa — mas *a janela em que ela é uma melhoria inequívoca é `93°`–`108°`*, quinze graus, mais
/// uma melhoria modesta em tudo o que está abaixo.
///
/// # ⏳ A pergunta que fica ABERTA, e ela é maior que esta porta
///
/// ⛔ **O padrão-ouro tem o MESMO vinco** (auditoria de 2026-09-20), logo *nenhuma mistura por
/// ponto de duas rotações rígidas em torno de uma junta partilhada o evita* — a aresta de dentro
/// de um cotovelo que dobra TEM de encurtar. ⇒ a alavanca de fundo não é a lei do ângulo; é
/// **quanto da dobra cabe em cada junta**, e este app já sabe reparti-la por sub-ossos
/// ([`ph2d_skeleton::SkinBone::sub`], o *bendy bone* que o painel chama «Curve Handles»).
///
/// ⚠️⚠️ **E isso NÃO está medido.** Eu escrevi a sonda e ela leu `estic 1,0000` em toda a linha,
/// com `1 979` de `2 001` pontos a misturar dois ossos — um resultado que não sei explicar, logo
/// não é uma medição. *Uma régua que não vejo funcionar não entra num doc-comment*; a sonda foi
/// apagada e a pergunta fica aqui, com o mecanismo, para quem a for medir a sério não recomeçar
/// do zero.
///
/// ⚠️ **Lida UMA VEZ por processo** ([`std::sync::OnceLock`]) e no SÍTIO que constrói a pele — um
/// `var_os` por ponto seria uma syscall dentro do laço do desenho, e a lei viaja daí para baixo
/// **na própria [`Skin`]**, nunca num estado global mutável.
///
/// ⛔ **O caminho do DISPOSITIVO recusa-se a desenhar com ela** ([`crate::skin_image_gpu`]): o
/// shader implementa a média em círculo e mais nenhuma, logo a alternativa seria uma imagem
/// **errada** em vez de uma imagem lenta.
#[must_use]
pub fn mistura_do_ambiente() -> ph2d_skeleton::MisturaDoAngulo {
    use ph2d_skeleton::MisturaDoAngulo;
    static ESCOLHA: std::sync::OnceLock<MisturaDoAngulo> = std::sync::OnceLock::new();
    *ESCOLHA.get_or_init(|| {
        if std::env::var("PH2D_SKIN_ANGULO").as_deref() == Ok("1") {
            MisturaDoAngulo::Desdobrado
        } else {
            MisturaDoAngulo::Circulo
        }
    })
}
