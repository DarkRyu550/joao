-- new_account.lua: Creates a new user at a key.
-- Parameters:
--      KEYS[1] - KEYS[1]:name
--      KEYS[2] - KEYS[1]:email
--      KEYS[3] - KEYS[1]:keyhash
--      KEYS[4] - KEYS[1]:salt
--      KEYS[5] - KEYS[1]:cd_lock
--      KEYS[6] - Target key family where the user will be stored.
--
-- 		ARGV[1] - Starting balance.
--      ARGV[2] - User's email account.
--      ARGV[3] - User's real, full name.
--      ARGV[4] - User's keyhash.
--      ARGV[5] - Salt value used for the keyhash.
--

if redis.call("get", KEYS[6]) then
	return "-KeyExists"
end

redis.call("set", KEYS[6], ARGV[1])
redis.call("set", KEYS[2], ARGV[2])
redis.call("set", KEYS[1], ARGV[3])
redis.call("set", KEYS[3], ARGV[4])
redis.call("set", KEYS[4], ARGV[5])

if redis.call("get", KEYS[5]) then
	redis.call("del", KEYS[5])
end

return "+OK"

