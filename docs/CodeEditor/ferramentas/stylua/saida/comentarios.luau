--!strict
--!native
-- @hook on_hit
-- cabeçalho: ação, ênfase — e emoji ⭐
local   x :number=1   -- @hook inline
--[[ bloco
   multi  linha  ]]
type T = { read Foo: number, write Bar: string }
type A = { read number }
const K = 3
-- stylua: ignore
local   m  =  { 1,2,
   3 }
function on_hit(self,   other)   -- @hook on_hit
    --@param other
    return   x+K
end
local s = 'aspas simples'
local t = "com \"escape\""
