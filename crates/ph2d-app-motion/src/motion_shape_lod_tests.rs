//! Os gates do LOD da forma (report do Enio, 2026-09-21). A lei é pura: uma decisão sobre
//! contagem e tamanho, e uma partição. ⚠️ **O assar continua a ser GPU e continua fora daqui** —
//! o que estes gates defendem é *quem* vira tile e *quando ela não pode*.

use super::*;
use crate::motion_shape_bake::ShapeTile;

/// Um quadrado de `1 × 1` na origem — a geometria dos gates. ⚠️ A FORMA não importa para esta lei
/// (o que decide é a CAIXA dela), e um quadrado torna a aritmética conferível à mão.
fn quadrado() -> ph2d_vec_scene::VecPath {
    ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5])
}

fn store_com_quadrado() -> (VecPathStore, u32) {
    let mut s = VecPathStore::default();
    let gid = s.push(quadrado());
    (s, gid)
}

/// `n` cópias da geometria `gid`, todas com a mesma pose unitária.
fn copias(gid: u32, n: usize, size: [f32; 2]) -> Vec<VectorInstance> {
    (0..n)
        .map(|i| VectorInstance {
            geometry_id: gid,
            texture_id: 0,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            premultiplied: 0.0,
            #[expect(clippy::cast_precision_loss, reason = "índice de cópia num gate")]
            world_pos: [i as f32, 0.0],
            size,
            basis: [1.0, 0.0, 0.0, 1.0],
            tint: [1.0; 4],
            anchor: [0.0, 0.0],
        })
        .collect()
}

/// **A forma PEQUENA e NUMEROSA vira tile** — o caso do report.
///
/// A câmara com escala `2` dá ao quadrado unitário `2 px` de lado, que cabe na barra de
/// [`LADO_MAXIMO_PX`]; as `11` cópias passam o joelho de `10`.
#[test]
fn uma_forma_pequena_e_numerosa_e_escolhida() {
    let (s, gid) = store_com_quadrado();
    let insts = copias(gid, 11, [1.0, 1.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true);
    assert!(
        quer.contains(&gid),
        "2 px de lado e 11 cópias têm de entrar"
    );
}

/// **O CONTROLO do tamanho: a mesma cena, a mesma contagem, a câmara AFASTADA de menos.** Com
/// escala `8` o quadrado mede `8 px` — o dobro da barra — e a tile erraria `9,9` níveis.
///
/// ⚠️ Sem este gate, `LADO_MAXIMO_PX` podia ir a infinito e nada reprovava: *uma barra que nunca
/// recusa ninguém não é uma barra.*
#[test]
fn a_mesma_forma_grande_no_ecra_fica_crisp() {
    let (s, gid) = store_com_quadrado();
    let insts = copias(gid, 11, [1.0, 1.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(8.0), 10, true);
    assert!(
        quer.is_empty(),
        "8 px de lado é o dobro da barra — fica crisp"
    );
}

/// **O CONTROLO da contagem**: pequena mas SOZINHA não paga um assado.
#[test]
fn poucas_copias_ficam_crisp() {
    let (s, gid) = store_com_quadrado();
    let insts = copias(gid, 10, [1.0, 1.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true);
    assert!(quer.is_empty(), "o joelho é ESTRITO: 10 não passa 10");
}

/// ⭐⭐⭐ **A MAIOR cópia é que decide, e não a primeira nem a média.**
///
/// `200` cópias minúsculas e **UMA** grande: a decisão é por GEOMETRIA, logo a tile iria com a
/// grande também e o artista veria exactamente aquela borrada. ⚠️ Uma régua que lesse a PRIMEIRA
/// cópia passaria este caso (a grande é a última), e uma que lesse a MÉDIA também.
#[test]
fn a_maior_copia_e_que_decide() {
    let (s, gid) = store_com_quadrado();
    let mut insts = copias(gid, 200, [1.0, 1.0]);
    // ⚠️⚠️ **A grande vai para o MEIO, e a posição é load-bearing.** Em ÚLTIMO, uma régua que
    // guardasse simplesmente a cópia mais recente (em vez do máximo) passaria este gate — foi
    // uma mutação SOBREVIVENTE que o disse. Em primeiro, passaria a que guarda a primeira.
    insts.insert(
        100,
        VectorInstance {
            size: [40.0, 40.0], // 80 px de lado sob a câmara de escala 2
            ..insts[0]
        },
    );
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true);
    assert!(
        quer.is_empty(),
        "uma cópia grande entre 200 pequenas segura a geometria inteira no crisp"
    );
}

/// **SEM TILE a forma fica crisp** — a cerca que o irmão já tem, e pela mesma razão: o assado
/// termina no quadro SEGUINTE, e uma tile em falta não pode apagar a forma.
#[test]
fn sem_tile_assada_nada_se_move() {
    let (s, gid) = store_com_quadrado();
    let mut insts = copias(gid, 11, [1.0, 1.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true);
    let mut quads = Vec::new();
    let bake = ShapeBake::default();
    let movidas = aplica_lod_de_forma(&mut quads, &mut insts, &bake, &quer);
    assert_eq!(movidas, 0, "sem tile, zero movidas");
    assert_eq!(insts.len(), 11, "e as onze continuam a desenhar-se");
    assert!(quads.is_empty());
}

/// **COM tile, as cópias mudam de lista** — e o que sobra em `vector_instances` é o que fica
/// crisp. ⚠️ As duas metades: o que SAIU e o que FICOU. Contar só uma deixaria passar uma
/// partição que duplicasse as instâncias em vez de as mover.
#[test]
fn com_tile_as_copias_viram_quads() {
    let (s, gid) = store_com_quadrado();
    let mut insts = copias(gid, 11, [1.0, 1.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true);
    let mut bake = ShapeBake::default();
    bake.seed_for_test(
        gid,
        ShapeTile {
            texture_id: 7,
            world_size: [1.0, 1.0],
            local_center: [0.0, 0.0],
        },
    );
    let mut quads = Vec::new();
    let movidas = aplica_lod_de_forma(&mut quads, &mut insts, &bake, &quer);
    assert_eq!(movidas, 11, "as onze viram quads");
    assert!(insts.is_empty(), "e nenhuma fica no caminho crisp");
    assert!(
        quads.iter().all(|q| q.texture_id == 7),
        "e o quad amostra a tile assada"
    );
    // ⭐⭐⭐ **E O TINT VIAJA** — a tile de um primitivo é assada em BRANCO
    // (`draw_path_standalone` passa `[1,1,1,1]`) e a cor da cópia é o `tint`. Sem esta metade, o
    // LOD trocava a cor de toda cópia ao armar, e o sintoma seria «as estrelas ficaram brancas ao
    // afastar» — uma queixa que ninguém ligaria ao LOD.
    assert!(
        quads.iter().all(|q| q.tint == [1.0; 4]),
        "o tint da instância tem de chegar ao quad"
    );
}

/// **Uma geometria que o LOD não quer NÃO é tocada, mesmo com tile assada.**
///
/// ⚠️ É o CONTROLO da partição: sem ele, `aplica_lod_de_forma` podia ignorar o conjunto `quer` e
/// mover tudo o que tivesse tile — e como o assador de glow assa formas por outra razão, isso
/// levaria à tile formas que o artista está a ver GRANDES.
#[test]
fn o_que_o_lod_nao_quer_nao_e_tocado() {
    let (s, gid) = store_com_quadrado();
    let mut insts = copias(gid, 11, [1.0, 1.0]);
    let _ = &s;
    let mut bake = ShapeBake::default();
    bake.seed_for_test(
        gid,
        ShapeTile {
            texture_id: 7,
            world_size: [1.0, 1.0],
            local_center: [0.0, 0.0],
        },
    );
    let mut quads = Vec::new();
    // ⚠️⚠️ **O conjunto pedido não pode ser VAZIO aqui.** Com `quer` vazio a função sai pela
    // guarda de cima e a cerca que este gate existe para medir **nunca corre** — a mutação que a
    // apagava sobrevivia. Ele leva OUTRA geometria: o LOD quer alguém, e não esta.
    let outra: std::collections::BTreeSet<u32> = [gid + 1].into_iter().collect();
    let movidas = aplica_lod_de_forma(&mut quads, &mut insts, &bake, &outra);
    assert_eq!(movidas, 0, "a geometria que o LOD não pediu não se move");
    assert_eq!(insts.len(), 11);
}

/// ⭐⭐⭐ **A tile que o LOD está a usar NÃO é largada pelo despejo** — o gate do defeito pior que
/// esta wave podia ter escrito.
///
/// Depois da partição as cópias vivem em `instances` como quads, e o `live` de sempre (os
/// `geometry_id` das `vector_instances`) **já não as vê**. Com a cena PARADA o cozimento devolve
/// cedo e as duas listas persistem ⇒ sem esta lei o despejo largava a textura **e os quads
/// passavam a amostrar um slot livre**.
///
/// ⚠️ **As duas metades**: com o quad presente o gid está vivo; SEM ele o gid sai. A primeira
/// sozinha passaria numa lei que devolvesse sempre tudo, e essa nunca largaria uma tile — que é o
/// OOM do quadro 19706 que este assador já pagou.
#[test]
fn a_tile_que_o_lod_desenha_nao_e_largada() {
    let (s, gid) = store_com_quadrado();
    let mut insts = copias(gid, 11, [1.0, 1.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true);
    let mut bake = ShapeBake::default();
    bake.seed_for_test(
        gid,
        ShapeTile {
            texture_id: 7,
            world_size: [1.0, 1.0],
            local_center: [0.0, 0.0],
        },
    );
    let mut quads = Vec::new();
    assert_eq!(
        aplica_lod_de_forma(&mut quads, &mut insts, &bake, &quer),
        11
    );
    assert!(
        insts.is_empty(),
        "controlo: a partição esvaziou o lado crisp"
    );

    let vivas = vivas_com_o_lod(&vivos(&insts), &quads, &bake);
    assert!(
        vivas.contains(&gid),
        "⛔ o gid tem de continuar VIVO: os quads ainda amostram a tile dele"
    );
    // E a metade que impede a lei de virar «nunca larga nada»: sem os quads, o gid sai.
    let sem_quads = vivas_com_o_lod(&vivos(&insts), &[], &bake);
    assert!(
        !sem_quads.contains(&gid),
        "⛔ sem ninguém a desenhar a tile, ela TEM de poder ser largada (o OOM de 2026-08-21)"
    );
}

/// **E o lado crisp continua a contar** — uma geometria que o LOD não levou está viva pelo caminho
/// de sempre. ⚠️ Sem esta metade, a lei nova podia ter SUBSTITUÍDO o `live` em vez de o UNIR, e o
/// despejo largaria a tile de uma forma que está a ser desenhada crisp com glow.
#[test]
fn o_lado_crisp_continua_a_contar_como_vivo() {
    let (s, gid) = store_com_quadrado();
    let insts = copias(gid, 3, [1.0, 1.0]);
    let _ = &s;
    let bake = ShapeBake::default();
    let vivas = vivas_com_o_lod(&vivos(&insts), &[], &bake);
    assert!(vivas.contains(&gid), "quem desenha crisp está vivo");
}

/// ⭐⭐ **A partição usa o [`tile_quad`] e NÃO o `vector_instance_as_tile` ingénuo** — e este gate
/// existe porque os dois são indistinguíveis na fixtura fácil.
///
/// A caixa de uma forma paramétrica não é centrada na origem local dela (uma seta, um arco), e o
/// ingénuo copiaria `world_pos`/`size` da instância: o quad mediria `1 × 1` e ficaria no sítio
/// errado. ⚠️ **Com `local_center = [0, 0]` e `world_size = [1, 1]` as duas rotas dão o MESMO
/// resultado** — é por isso que a fixtura tem de ser excêntrica, senão o gate fica verde sobre a
/// troca que ele existe para impedir.
#[test]
fn a_particao_honra_a_ancora_da_forma() {
    let (s, gid) = store_com_quadrado();
    let mut insts = copias(gid, 11, [2.0, 2.0]);
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(1.0), 10, true);
    assert!(quer.contains(&gid), "controlo: 2 px de lado entra");
    let mut bake = ShapeBake::default();
    bake.seed_for_test(
        gid,
        ShapeTile {
            texture_id: 7,
            world_size: [3.0, 5.0],
            local_center: [10.0, -4.0],
        },
    );
    let mut quads = Vec::new();
    assert_eq!(
        aplica_lod_de_forma(&mut quads, &mut insts, &bake, &quer),
        11
    );
    // O tamanho sai do TILE × a escala da instância (`3 × 2`, `5 × 2`), nunca do `vi.size`.
    assert_eq!(quads[0].size, [6.0, 10.0], "o tamanho vem do tile");
    // E o centro do bbox atravessa a BASE: `local_center × size` somado ao `world_pos` (que é
    // `[0, 0]` na primeira cópia) ⇒ `[20, -8]`.
    assert_eq!(quads[0].world_pos, [20.0, -8.0], "a âncora é a do bbox");
}

/// ⭐⭐⭐ **Uma forma RODADA ocupa a DIAGONAL dela, e é isso que a soma das duas colunas mede.**
///
/// Um quadrado de `4 px` de lado a `45°` desenha-se numa caixa de `5,66 px` — acima da barra, logo
/// ele tem de ficar crisp. ⚠️ Uma régua que tomasse o MAIOR dos dois termos em vez da soma leria
/// `2,83 px` e mandava-o à tile: foi uma mutação SOBREVIVENTE que nomeou este buraco, e **nenhuma
/// das outras fixturas tem base rodada**, que é exactamente onde os dois diferem.
#[test]
fn uma_forma_rodada_ocupa_a_diagonal_dela() {
    let (s, gid) = store_com_quadrado();
    let d = std::f32::consts::FRAC_1_SQRT_2;
    let insts: Vec<VectorInstance> = copias(gid, 11, [4.0, 4.0])
        .into_iter()
        .map(|vi| VectorInstance {
            basis: [d, d, -d, d], // 45°
            ..vi
        })
        .collect();
    let quer = geometrias_para_lod_com(&insts, &s, Affine::scale(1.0), 10, true);
    assert!(
        quer.is_empty(),
        "5,66 px de diagonal está acima da barra de 4 — fica crisp"
    );
    // O CONTROLO: a MESMA forma, o MESMO tamanho, SEM rotação — ela cabe e entra.
    let direitas = copias(gid, 11, [4.0, 4.0]);
    assert!(
        geometrias_para_lod_com(&direitas, &s, Affine::scale(1.0), 10, true).contains(&gid),
        "controlo: sem rotação, 4 px cabe na barra"
    );
}

/// **A lei da porta de bissecção, nas três células** — ausente · `"0"` · outra coisa.
///
/// ⚠️ Ela é uma função de um `Option<&str>` e não uma leitura do processo, de propósito: a
/// pergunta *«qual é o caminho de OMISSÃO?»* é sobre o PRODUTO, e um gate que lesse o ambiente
/// responderia sobre a máquina em que corre.
#[test]
fn a_porta_de_bisseccao_diz_o_que_a_variavel_significa() {
    assert!(lod_por(None), "ausente ⇒ LIGADO (é o caminho de omissão)");
    assert!(!lod_por(Some("0")), "`0` ⇒ desligado");
    assert!(!lod_por(Some(" 0 ")), "e com espaços à volta também");
    assert!(lod_por(Some("1")), "qualquer outra coisa ⇒ ligado");
}

/// **E desligada, o LOD não quer NINGUÉM** — mesmo na cena que ele quereria.
///
/// ⚠️ O CONTROLO está dentro: a MESMA entrada com a rota ligada tem de escolher a geometria. Sem
/// ele, uma lei que devolvesse sempre vazio passaria esta metade.
#[test]
fn desligada_a_porta_nao_quer_ninguem() {
    let (s, gid) = store_com_quadrado();
    let insts = copias(gid, 11, [1.0, 1.0]);
    assert!(
        geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, false).is_empty(),
        "desligada, ninguém vira tile"
    );
    assert!(
        geometrias_para_lod_com(&insts, &s, Affine::scale(2.0), 10, true).contains(&gid),
        "controlo: ligada, esta é exactamente a cena que ela quer"
    );
}

/// O conjunto vivo do lado CRISP — o que a fase constrói antes da partição e passa à lei.
fn vivos(insts: &[VectorInstance]) -> std::collections::BTreeSet<u32> {
    insts.iter().map(|vi| vi.geometry_id).collect()
}
