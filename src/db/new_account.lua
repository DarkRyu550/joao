-- new_user.lua: Creates a new user at a key.
-- Parameters:
--      KEYS[1] - Target key family where the user will be stored.
--                Think of this parameter as a user hash, this is the actual
--                value that identifies this user.
--
-- 		ARGS[1] - Starting balance.
--      ARGS[2] - User's email account.
--      ARGS[3] - User's real, full name.
--

local USER_NAME    = KEYS[1] .. ":name"
local USER_EMAIL   = KEYS[1] .. ":email"
local USER_COOLER  = KEYS[1] .. ":cd_lock"
local USER_HISTORY = KEYS[1] .. ":history"

-- Make sure we're not overwriting anyone
if redis.call("get", KEYS[1]) then
	return "-KeyExists"
end

if 

redis.call("set", KEYS[1], "1")

if ARGS[1] then
	redis.call("set", USER_NAME, ARGS[1])
end

return "+OK"

