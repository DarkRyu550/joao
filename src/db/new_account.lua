-- new_account.lua: Creates a new user at a key.
-- Parameters:
--      KEYS[1] - user:name
--      KEYS[2] - user:email
--      KEYS[3] - user:keyhash
--      KEYS[4] - user:salt
--      KEYS[5] - user:cd_lock
--      KEYS[6] - user:balance
--      KEYS[7] - uid_table
--      KEYS[8] - user:username
--
-- 		ARGV[1] - Starting balance.
--      ARGV[2] - User's email account.
--      ARGV[3] - User's real, full name.
--      ARGV[4] - User's keyhash.
--      ARGV[5] - Salt value used for the keyhash.
--      ARGV[6] - Username.
--      ARGV[7] - Userhash.
--

if redis.call("hexists", KEYS[7], ARGV[6]) == 1 then
	return "-KeyExists"
end

if    redis.call("exists", KEYS[6]) == 1 
   or redis.call("exists", KEYS[4]) == 1
   or redis.call("exists", KEYS[3]) == 1
   or redis.call("exists", KEYS[2]) == 1
   or redis.call("exists", KEYS[1]) == 1 then

	return "-Retry"
end

redis.call("set", KEYS[6], ARGV[1])
redis.call("set", KEYS[2], ARGV[2])
redis.call("set", KEYS[1], ARGV[3])
redis.call("set", KEYS[3], ARGV[4])
redis.call("set", KEYS[4], ARGV[5])
redis.call("set", KEYS[8], ARGV[6])

if redis.call("get", KEYS[5]) then
	redis.call("del", KEYS[5])
end

redis.call("hset", KEYS[7], ARGV[6], ARGV[7])

return "+OK"

