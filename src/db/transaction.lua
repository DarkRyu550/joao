local USER0_HISTORY = KEYS[1] .. ":history"
local USER1_HISTORY = KEYS[2] .. ":history"
local USER0_COOLER  = KEYS[1] .. ":cd_lock"
local USER1_COOLER  = KEYS[2] .. ":cd_lock"

local srcValue  = redis.call("get", KEYS[1])
local destValue = redis.call("get", KEYS[2])

if not srcValue then
    return 2
end
if not destValue then
    return 3
end

if redis.call("get", USER0_COOLER) or redis.call("get", USER1_COOLER) then
	return 4
end

local amt    = tonumber(ARGV[1])
local srcBal = tonumber(srcValue)

-- TODO: Use a proper, string-based comparation here. Fuck floating point.
if amt <= srcBal then
    redis.call("decrby", KEYS[1], ARGV[1])
    redis.call("incrby", KEYS[2], ARGV[1])

	-- Record the transaction for both of them.
	local record = {}
	record.from    = KEYS[1]
	record.to      = KEYS[2]
	record.ammount = ARGV[1]
	local json_record = cjson:encode(record)

	redis.call("lpush", USER0_HISTORY, json_record)
	redis.call("lpush", USER1_HISTORY, json_record)

	-- Activate cooldown for both of them
	redis.call("set", USER0_COOLER, "1")
	redis.call("set", USER1_COOLER, "1")

	redis.call("expire", USER0_COOLER, "2")
	redis.call("expire", USER1_COOLER, "2")

    return 0
else
    return 1
end
