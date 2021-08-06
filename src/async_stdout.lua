hook.Add("PlayerInitialSpawn", "async_stdout", function()
	hook.Remove("PlayerInitialSpawn", "async_stdout")
	require("async_stdout")
end)