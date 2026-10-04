// ⭐ doc 121 §9.15 (d) — **o prefixo do `cs_varre` por SUBGRUPO.** Só entra no módulo onde o dispositivo
// tem `Features::SUBGROUP` (`pass.rs`); sem ela, o `cs_varre` de memória de grupo. A soma é de INTEIROS:
// a mesma cobertura, bit a bit, por outra ordem.
//
// O pixel de cada fio é posto pela pista do subgrupo (`subgroup_id · subgroup_size + pista`), para que uma
// célula de `PIXELS_DA_CELULA` px sejam pistas CONTÍGUAS do mesmo subgrupo quando ele tem `≥ 32` — e a
// soma inclusiva do subgrupo só precisa de tirar a da pista antes do início da célula. Num subgrupo
// `< 32` a célula atravessa vários: soma-se o total dos anteriores da mesma célula (memória de grupo,
// uma barreira, e o laço não corre com subgrupos de `32` ou mais).
var<workgroup> totais_do_subgrupo: array<vec3<i32>, 64>;

@compute @workgroup_size(64)
fn cs_varre_sg(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(num_workgroups) nwg: vec3<u32>,
    @builtin(subgroup_id) sid: u32,
    @builtin(subgroup_size) tam: u32,
    @builtin(subgroup_invocation_id) pista: u32,
) {
    let li = sid * tam + pista;
    let g = wid.x * 64u + li + wid.y * nwg.x * 64u;
    let viva = g / PIXELS_DA_CELULA < celulas_em_uso();
    let v = depositos_do_pixel(g, viva);
    var s = subgroupInclusiveAdd(v);
    let inicio = pista - pista % PIXELS_DA_CELULA;
    let antes = subgroupShuffle(s, max(inicio, 1u) - 1u);
    s = select(s, s - antes, inicio > 0u);
    let total = subgroupAdd(v);
    if pista == 0u {
        totais_do_subgrupo[sid] = total;
    }
    workgroupBarrier();
    if tam < PIXELS_DA_CELULA {
        let por_celula = PIXELS_DA_CELULA / tam;
        for (var k = sid - sid % por_celula; k < sid; k += 1u) {
            s += totais_do_subgrupo[k];
        }
    }
    if viva {
        acaba_o_pixel(g, s);
    }
}
