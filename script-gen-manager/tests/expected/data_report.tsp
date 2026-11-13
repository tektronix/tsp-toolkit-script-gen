local buffers = {} -- Array of reading buffers to return
local bufferNames = {} -- Array of reading buffer names
local bufferSmuNames = {} -- Array of SMU names for each reading buffer
local expectedCount = {} -- Array containing the expected number of return values.
local waitInterval = 1  -- Interval between message transfers
local maxToReturn = 100 -- Maximum reading buffer points between message transfers.

-- The following are special tokens used internally by the data report function
local DATA_REPORT = "(({{Data}}))"
local SWEEPSTART = "{SWEEP-START}"
local START = "{START}"
local NAME = "{NAME}"
local EXPECTED_COUNT = "{EXPECTED-COUNT}"
local PTS_IN_BUFF = "{PTS-IN-BUFF}"
local PTS_RETURNED =  "{PTS-RETURNED}"
local BASE_TIME_STAMP = "{BASE-TIME-STAMP}"
local READINGS = "{READINGS}"
local TIMESTAMPS = "{TIMESTAMPS}"
local SRCVALS = "{SRCVALS}"
local END = "{END}"
local COMPLETE = "{COMPLETE}"

local errorTag = "[{error}]"
local dataIndexes = {} -- Data transfer index for each reading buffer
local done = true  -- True when data transfer from all of reading buffers is completed
local dataComplete = true  -- True when all of reading buffers have reached their expected counts

--==========================================================================
-- This function determines the ending points that will be in the buffer
-- at the time data collection takes place.
--==========================================================================
local GetStopPoints = function()
    for i, selBuffer in ipairs(buffers) do
        for j, selSysNode in ipairs(systemSmus) do
            for k, selSmu in ipairs(selSysNode) do
                if (selSmu.nvbuffer1 == selBuffer) then
                    expectedCount[i] = systemSmuReadingBufferIndexes[j][k][1]["stop"]
                elseif (selSmu.nvbuffer2 == selBuffer) then
                    expectedCount[i] = systemSmuReadingBufferIndexes[j][k][2]["stop"]
                end
            end
        end
    end
end

--==========================================================================
-- Checks if the SMU for selBuffer is sweeping.
--==========================================================================
local IsSweeping = function(selBuffer)
    for i, selNode in ipairs(systemNodes) do
        for j, selSmu in ipairs(systemSmus[i]) do
            if (j == 1) then
                if ((selSmu.nvbuffer1 == selBuffer) or (selSmu.nvbuffer2 == selBuffer)) then
                    local statcond = selNode.status.operation.instrument.smua.condition
                    return (bit.test(statcond,4))  -- Check Bit B3, Sweeping (SWE)
                end
            elseif (j == 2) then
                if ((selSmu.nvbuffer1 == selBuffer) or (selSmu.nvbuffer2 == selBuffer)) then
                    local statcond = selNode.status.operation.instrument.smub.condition
                    return (bit.test(statcond,4))  -- Check Bit B3, Sweeping (SWE)
                end
            end
        end
    end
    return false
end

--==========================================================================
-- Clears the reading buffers and resets the array that contains the
-- reading buffer storage start/ stop points.
--==========================================================================
local ClearReturnedBuffers = function()
    for i, selBuffer in ipairs(buffers) do
        selBuffer.clear()
        
        local smuBufferIndex = nil
        for j, selSysNode in ipairs(systemSmus) do
            for k, selSmu in ipairs(selSysNode) do
                if (selSmu.nvbuffer1 == selBuffer) then
                    smuBufferIndex = systemSmuReadingBufferIndexes[j][k][1]
                elseif (selSmu.nvbuffer2 == selBuffer) then
                    smuBufferIndex = systemSmuReadingBufferIndexes[j][k][2]
                end
            end
        end
        if(smuBufferIndex ~= nil) then
            smuBufferIndex["start"] = 0
            smuBufferIndex["stop"] = 0
        end
    end
end

GetStopPoints()

-- Initialize dataIndex. This array is used to keep track of the points returned to
-- the application.

for i, v in ipairs(buffers) do
    dataIndexes[i] = 1
end

-- The following print statements turns on data report feature within the application.
print(DATA_REPORT)
print(SWEEPSTART)

repeat
    done = true
    dataComplete = true
    for i, selBuffer in ipairs(buffers) do
        -- Check for smu trigger model overruns.
        _Overruncheck()
        
        -- Check for smu compliance.
        _ComplianceCheck()
        
        -- Caution: Check for stillSweeping BEFORE getting numPointsStored
        local stillSweeping = IsSweeping(selBuffer)
        local numPointsStored = selBuffer.n
        
        -- Check for sweep completion before data is complete (i.e. something bad happened)
        if ((numPointsStored < expectedCount[i]) and (stillSweeping == false)) then
            gTestAborted = true
        end
        
        dataComplete = dataComplete and (numPointsStored >= expectedCount[i])
        done = done and (dataIndexes[i] > expectedCount[i])
        
        if(dataIndexes[i] <= numPointsStored) then
            local points = numPointsStored - dataIndexes[i] + 1
            
            if(points > maxToReturn) then
                points = maxToReturn
            end
            
            local retString = string.format("%s%s%s%d%s%d%s", NAME,  bufferNames[i], PTS_IN_BUFF, numPointsStored, PTS_RETURNED, points, START)
            
            if(dataIndexes[i] == 1) then
                retString = retString .. string.format("%s%d%s%f", EXPECTED_COUNT, expectedCount[i], BASE_TIME_STAMP, selBuffer.basetimestamp)
            end
            
            local start = dataIndexes[i]
            local stop = start + points -1
            
            local readings =""
            
            for j=start, stop do
                if(j == start) then
                    readings = readings .. string.format("%s%e", READINGS, selBuffer.readings[j])
                else
                    readings = readings .. string.format(",%e", selBuffer.readings[j])
                end
            end
            
            local timestamps = ""
            if (selBuffer.collecttimestamps == 1) then
                for j=start, stop do
                    if(j == start) then
                        timestamps = timestamps .. string.format("%s%e", TIMESTAMPS, selBuffer.timestamps[j])
                    else
                        timestamps = timestamps .. string.format(",%e", selBuffer.timestamps[j])
                    end
                end
            end
            
            local srcValues = ""
            if (selBuffer.collectsourcevalues == 1) then
                for j=start, stop do
                    if(j == start) then
                        timestamps = timestamps .. string.format("%s%e", SRCVALS, selBuffer.sourcevalues[j])
                    else
                        timestamps = timestamps .. string.format(",%e", selBuffer.sourcevalues[j])
                    end
                end
            end
            
            dataIndexes[i] = stop +1
            
            retString = retString .. readings .. timestamps .. srcValues .. END
            print(retString)
        end
    end
    
    if(dataComplete == false) then
        if (gTestAborted == true) then
            done = true  -- Stop waiting for data completion because it will never happen
        else
            delay(waitInterval)
        end
    end
until (done == true)

print(COMPLETE .. "\n(({{}}))")

-- The application may not have selected all the buffers being used in the test, so
-- we need to make sure the overlapped operations are complete before clearing the buffers.
local sweepCompleted = _WaitForComplete(-1)

if (sweepCompleted == true) then
    ClearReturnedBuffers()
end
