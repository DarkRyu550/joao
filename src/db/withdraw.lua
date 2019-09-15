--[[
    KEYS[1]: user balance
    ARGV[1]: amount to withdraw
]]

local amt = tonumber(ARGV[1])
local value = tonumber(redis.call("get", KEYS[1]))

if amt <= value then
    redis.call("decrby", KEYS[1], ARGV[1])
    return 0
else 
    return 1
end
