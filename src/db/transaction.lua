local srcValue  = redis.call("get", KEYS[1])
local destValue = redis.call("get", KEYS[2])

if not srcValue then
    return 2
end
if not destValue then
    return 3
end

local amt    = tonumber(ARGV[1])
local srcBal = tonumber(srcValue)

if amt <= srcBal then
    redis.call("decrby", KEYS[1], amt)
    redis.call("incrby", KEYS[2], amt)
    return 0
else
    return 1
end
