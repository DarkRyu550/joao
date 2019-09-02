local amt = tonumber(ARGV[1])

local srcValue = redis.call("get", KEYS[1])
local destValue = redis.call("get", KEYS[2])

if not srcValue then
    return 2
end
if not destValue then
    return 3
end

local srcBal = tonumber(srcValue)

if amt <= srcBal then
    redis.call("set", KEYS[1], srcBal - amt)
    local destBal = tonumber(destValue)
    redis.call("set", KEYS[2], destBal + amt)
    return 0
else
    return 1
end
