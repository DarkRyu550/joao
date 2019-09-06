--[[

KEYS[1]: should always be "users:<USERNAME>:password"
KEYS[2]: should always be "users:<USERNAME>:balance"

ARGS[1]: hash of the user's password
ARGS[2]: initial balance of the user

Return values:
     0: Success
     1: User already exists
]]

local passwordKey = KEYS[1]
local balanceKey = KEYS[2]

local password = ARGS[1]
local balance  = ARGS[2]

if redis.call("setnx", passwordKey, password) == 0 then
    return 1
end

redis.call("set", balanceKey, balance)

return 0
