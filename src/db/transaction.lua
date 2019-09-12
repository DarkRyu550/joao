--[[
    TODO: fix this fucking pile of garbage
]]

local USER0_BALANCE   = KEYS[1]
local USER0_HISTORY   = KEYS[2]
local USER0_COOLDOWN  = KEYS[3]
local USER1_BALANCE   = KEYS[4]
local USER1_HISTORY   = KEYS[5]
local USER1_COOLDOWN  = KEYS[6]

local AMOUNT          = ARGV[1]
local USER0_USERNAME  = ARGV[2]
local USER1_USERNAME  = ARGV[3]

local srcValue  = redis.call("get", USER0_BALANCE)
local destValue = redis.call("get", USER1_BALANCE)

if not srcValue then
    return 2
end
if not destValue then
    return 3
end

if redis.call("get", USER0_COOLDOWN) or redis.call("get", USER1_COOLDOWN) then
	return 4
end

local amt    = tonumber(AMOUNT)
local srcBal = tonumber(srcValue)

-- TODO: Use a proper, string-based comparation here. Fuck floating point.
if amt <= srcBal then
    redis.call("decrby", USER0_BALANCE, AMOUNT)
    redis.call("incrby", USER1_BALANCE, AMOUNT)

	-- Record the transaction for both of them.
	local record = {}
	record.from    = USER0_USERNAME
	record.to      = USER1_USERNAME
	record.amount  = amt
	local json_record = cjson.encode(record)

	redis.call("lpush", USER0_HISTORY, json_record)
	redis.call("lpush", USER1_HISTORY, json_record)

	-- Activate cooldown for both of them
	redis.call("set", USER0_COOLDOWN, "1")
	redis.call("set", USER1_COOLDOWN, "1")

	redis.call("expire", USER0_COOLDOWN, "2")
	redis.call("expire", USER1_COOLDOWN, "2")

    return 0
else
    return 1
end
