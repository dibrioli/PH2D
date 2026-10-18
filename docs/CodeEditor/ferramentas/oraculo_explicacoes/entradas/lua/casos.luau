--!strict
local M = {}
M.amplitude = 1
local function soma(a: number, b: number): number
	return a + b
end
local function dobro(a: number): number
	return a * 2
end
function M.update(self: typeof(M), dt: number)
	pirnt("x")
	local y = self.ampiltude * 2
	local v = vector.create(1, 2, 3)
	local z = v.w
	local s = soma(1)
	local d = dobro("texto")
	local nao_usada = 3
	local n: any = nil
	local q = nil
	q()
	print(y, z, s, d)
end
return M
