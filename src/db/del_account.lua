-- del_account.lua: Deletes a new user at a key.
-- Parameters:
--      KEYS[1] - KEYS[8]:email
--      KEYS[2] - KEYS[8]:name
--      KEYS[3] - KEYS[8]:history
--      KEYS[4] - KEYS[8]:cd_lock
--      KEYS[5] - KEYS[8]:keyhash
--      KEYS[6] - KEYS[8]:salt
--      KEYS[7] - KEYS[8]:tokens
--      KEYS[8] - Target key family where the account is stored.
--

if not redis.call("get", KEYS[8]) then
	return "-KeyDoesNotExist"
end

redis.call("del", KEYS[8])
redis.call("del", KEYS[1])
redis.call("del", KEYS[2])
redis.call("del", KEYS[3])
redis.call("del", KEYS[4])
redis.call("del", KEYS[5])
redis.call("del", KEYS[6])
redis.call("del", KEYS[7])

return "+OK"

