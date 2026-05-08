--!strict
-- C1 spike fixture: hello world Luau script.
-- Loaded once by hello_runner.rs; `update(frame)` invoked 60 times.

local frame_count = 0

print("hello from luau — runtime initialized")

function update(frame: number)
    frame_count += 1
    if frame_count % 10 == 0 then
        print(string.format("frame %d (count=%d)", frame, frame_count))
    end
end
