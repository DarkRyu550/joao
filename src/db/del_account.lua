-- del_account.lua: Deletes a new user at a key.
-- Parameters:
--      KEYS[1]  - user:email
--      KEYS[2]  - user:name
--      KEYS[3]  - user:history
--      KEYS[4]  - user:cd_lock
--      KEYS[5]  - user:keyhash
--      KEYS[6]  - user:salt
--      KEYS[7]  - user:tokens
--      KEYS[8]  - user:balance
--      KEYS[9]  - user:username
--      KEYS[10] - uid_table
--

if not redis.call("get", KEYS[8]) then
	return "-KeyDoesNotExist"
end

local username = redis.call("get", KEYS[9])
redis.call("hdel", KEYS[10], username)

redis.call("del", KEYS[9])
redis.call("del", KEYS[8])
redis.call("del", KEYS[1])
redis.call("del", KEYS[2])
redis.call("del", KEYS[3])
redis.call("del", KEYS[4])
redis.call("del", KEYS[5])
redis.call("del", KEYS[6])
redis.call("del", KEYS[7])

return "+OK"

