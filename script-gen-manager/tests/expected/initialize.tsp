-- The local functions are scoped at the project level, so that we can keep
-- track of the reading buffers throughout the test.
local systemNodes = {}
local systemSmus = {}
local systemSmuReadingBufferIndexes = {}
local appendMode = 1

local gOverrunDetected = false
--==========================================================================
-- This function tests for overruns in any of the system smus' trigger models.
-- It returns an error to the application that will be displayed in a
-- dialog box if an overrun occurs.
--==========================================================================
local _Overruncheck = function()
    --==========================================================================
    -- This function test the results of the value in the smu's trigger overrun
    -- status register. -- It returns an error if an overrun occurs.
    --
    -- parameter   overrun     A bit pattern that contains the smu trigger overrun
    --                         status register result
    --==========================================================================
    local TestOverrunBits = function(overrun, selNode, whichSmu)
        local whichNode = [[localnode.]]
        if (localnode.tsplink ~= nil) then
            if (selNode.tsplink.node ~= localnode.tsplink.node) then
                whichNode = [[node[]] .. selNode.tsplink.node .. [[].]]
            end
        end
        whichNode = whichNode .. whichSmu .. [[ (]] .. selNode.model .. [[)]]
        if (bit.test(overrun, 2)) then
            gOverrunDetected = true
            return("arm overrun on " .. whichNode .. "{eol}")
        elseif (bit.test(overrun, 3)) then
            gOverrunDetected = true
            return("source overrun on " .. whichNode .. "{eol}")
        elseif (bit.test(overrun, 4)) then
            gOverrunDetected = true
            return("measure overrun on " .. whichNode .. "{eol}")
        elseif (bit.test(overrun, 5)) then
            gOverrunDetected = true
        return("end pulse overrun on " .. whichNode .. "{eol}")
    else
        return("")
    end
end

-- Report an overrun error only once.
if(gOverrunDetected == false) then
    local allSmusInOverrun = [[]]
    
    for i, selNode in ipairs(systemNodes) do
        for j, selSmu in ipairs(systemSmus[i]) do
            if(j == 1) then
                local overrun = selNode.status.operation.instrument.smua.trigger_overrun.event
                if (overrun > 0) then
                    allSmusInOverrun = allSmusInOverrun .. TestOverrunBits(overrun, selNode, [[smua]])
                end
            elseif(j == 2) then
                local overrun = selNode.status.operation.instrument.smub.trigger_overrun.event
                if (overrun > 0) then
                    allSmusInOverrun = allSmusInOverrun .. TestOverrunBits(overrun, selNode, [[smub]])
                end
            end
        end
    end
    
    if (gOverrunDetected == true) then
        print("[{error}]Script stopped due to:{eol}" .. allSmusInOverrun)
    end
end
end

local gComplianceDetected = false
--==========================================================================
-- Checks smus for voltage or current compliance.
-- If a compliance event occurs, the message dialog box is presented to the
-- user.
--==========================================================================
local _ComplianceCheck = function()
    -- Report a compliance error only once.
    if(gComplianceDetected == false) then
        local allSmusInCompliance = [[]]
        
        for i, selNode in ipairs(systemNodes) do
            for j, selSmu in ipairs(systemSmus[i]) do
                local compDetect = selNode.status.measurement.instrument.event
                if(compDetect > 0) then
                    -- Compose and show error message
                    local whichNode = [[localnode.]]
                    if (localnode.tsplink ~= nill) then
                        if (selNode.tsplink.node ~= localnode.tsplink.node) then
                            whichNode = [[node[]] .. selNode.tsplink.node .. [[].]]
                        end
                    end
                    local whichSmu = [[]]
                    if(compDetect == 2) then
                        whichSmu = whichNode .. [[smua]]
                    elseif (compDetect == 4) then
                        whichSmu = whichNode .. [[smub]]
                    elseif (compDetect == 6) then
                        whichSmu = whichNode .. [[smua and ]] .. whichNode .. [[smub]]
                    end
                    whichSmu = whichSmu .. [[ (]] .. selNode.model .. [[){eol}]]
                    allSmusInCompliance = allSmusInCompliance .. whichSmu
                    
                    -- Set gComplianceDetected so we don't report the error again.
                    gComplianceDetected = true
                end
            end
        end
        
        if (gComplianceDetected == true) then
            print("[{info}]Compliance detected on:{eol}" .. allSmusInCompliance ..
            "{eol}Click the Advanced button and check the Source Limit setting.")
        end
    end
end

local gTestAborted = false
--==========================================================================
-- This function waits up to a specified period for the sweeps to complete.
-- If the delay parameter is -1, then the timeout is infinite. It returns true
-- if all sweeping actions are complete. Otherwise, the return value is false.
-- Note: If gTestAborted is true, this function returns false immediately.
--
-- parameter  interval     Maximum time to wait (in seconds).
--==========================================================================
local _WaitForComplete = function(interval)
    local pollInterval = 0.1
    
    --==========================================================================
    -- Checks the system smus for any that are still sweeping.
    --==========================================================================
    local IsSweepingComplete = function()
        for i, selNode in ipairs(systemNodes) do
            for j, selSmu in ipairs(systemSmus[i]) do
                if (j == 1) then
                    local statcond = selNode.status.operation.instrument.smua.condition
                    if (bit.test(statcond,4)) then
                        return false
                    end
                elseif (j == 2) then
                    local statcond = selNode.status.operation.instrument.smub.condition
                    if (bit.test(statcond,4)) then
                        return false
                    end
                end
            end
        end
        
        return true
    end
    
    local notimeout = false
    if (interval < 0) then
        notimeout = true
        interval = 1
    end
    
    while(interval > 0) do
        -- Check the abort flag
        if (gTestAborted == true) then
            return false
        end
        
        -- Check for errors
        if (errorqueue.count > 0) then
            return false
        end
        
        -- Check for overruns and compliance
        _Overruncheck()
        _ComplianceCheck()
        
        if (IsSweepingComplete() == true) then
            return true
        end
        
        delay(pollInterval)
        
        if (notimeout == false) then
            interval = interval - pollInterval
        end
    end
    
    if (IsSweepingComplete() == true) then
        return true
    end
    
    return false
end
local maxNodes = 64   -- Maximum possible nodes in TSP link system.

local projectSetup =  -- Instrument configuration for the project.
local currentSetup = {} -- Current instrument configuration.

local errorTag = "[{error}]"
local errorMessage = {[[Instrument in project configuration not found.]], [[Instrument configuration at does not match.]]}
local errorNo = 0

--==========================================================================
-- Configures the status model to detect voltage or current compliance.
--==========================================================================
local ConfigStatusModel = function()
    for i, selNode in ipairs(systemNodes) do
        selNode.status.reset()
        for j, selSmu in ipairs(systemSmus[i]) do
            -- The selSmu.source.compliance call will force an update
            -- the condition register of the measurement.instrument.smuX to
            -- update
            local dum = selSmu.source.compliance
            if(j == 1) then
                -- Clear the measurement.instrument.smua event register by reading its value
                dum = selNode.status.measurement.instrument.smua.event
                -- Now configure the status model to detect voltage or current compliance
                selNode.status.measurement.instrument.smua.enable = 3
                selNode.status.measurement.instrument.smua.ptr = 3
                selNode.status.measurement.instrument.enable = 6
                selNode.status.measurement.instrument.ptr = 6
            elseif(j == 2) then
                -- Clear the measurement.instrument.smub event register by reading its value
                dum = selNode.status.measurement.instrument.smub.event
                -- Now configure the status model to detect voltage or current compliance
                selNode.status.measurement.instrument.smub.enable = 3
                selNode.status.measurement.instrument.smub.ptr = 3
            end
        end
    end
end

--==========================================================================
-- Determines the current system configuration.
--==========================================================================
local GetSetup = function ()
    local masterNode = 1
    if(localnode.tsplink ~= nil) then
        masterNode = tsplink.node
    end
    
    systemSmus = {}
    systemNodes = {}
    systemNodes[1] = node[masterNode]
    
    currentSetup[1] = {}
    currentSetup[1][1] = node[masterNode]
    currentSetup[1][2] = node[masterNode].model
    currentSetup[1][3] = node[masterNode].revision
    
    systemSmus[1] = {}
    systemSmus[1][1] = node[masterNode].smua
    
    if(node[masterNode].smub ~= nil) then
        systemSmus[1][2] = node[masterNode].smub
    end
    
    if(localnode.tsplink ~= nil) then
        local j = 2
        for i = 1, maxNodes do
            if((tsplink.node ~= i) and (node[i] ~= nil)) then
                systemNodes[j] = node[i]
                currentSetup[j] = {}
                currentSetup[j][1] = node[i]
                currentSetup[j][2] = node[i].model
                currentSetup[j][3] = node[i].revision
                
                systemSmus[j] = {}
                systemSmus[j][1] = node[i].smua
                
                if(node[i].smub ~= nil) then
                    systemSmus[j][2] = node[i].smub
                end
                j = j +1
            end
        end
    end
end

GetSetup()

-- Check the project setup versus the configuration setup.
-- If there is a difference, report an error.
local numpInstruments = table.getn(projectSetup)
local numcInstruments = table.getn(currentSetup)

for i=1, numpInstruments do
    local pfields = table.getn(projectSetup[i])
    
    errorNo = 1
    for j=1,numcInstruments do
        -- Compare nodes.
        if(projectSetup[i][1] ==  currentSetup[j][1]) then
            errorNo = 0
            
            for k=2, pfields do
                if(projectSetup[i][k] ~= currentSetup[j][k]) then
                    errorNo = 2
                end
            end
            break
        end
    end
    
    if (errorNo > 0) then
        break
    end
end

if (errorNo > 0) then
    error(errorMessage[errorNo])
    print(errorTag .. errorMessage[errorNo])
end

--==========================================================================
-- Initializes all system readings buffers by clearing and setting them to
-- append mode. An array, sysSmuReadingBufferIndexes, is used to keep track of the
-- data stored to each of the reading buffers.
--==========================================================================
local InitalizeReadingBuffers = function()
    systemSmuReadingBufferIndexes = {}
    for i, selNode in ipairs(systemSmus) do
        systemSmuReadingBufferIndexes[i] = {}
        
        for j, selSmu in ipairs(selNode) do
            -- Create an array for the smu.
            systemSmuReadingBufferIndexes[i][j] = {}
            
            local snvBuffers =  {selSmu.nvbuffer1, selSmu.nvbuffer2}
            for k, selBuffer in ipairs(snvBuffers) do
                -- Create an array for nvbuffers 1 and 2.
                systemSmuReadingBufferIndexes[i][j][k] = {}
                
                local sysSmuReadingBufferIndexes = systemSmuReadingBufferIndexes[i][j][k]
                sysSmuReadingBufferIndexes["start"] = 0
                sysSmuReadingBufferIndexes["stop"] = 0
                
                selBuffer.clear()
                selBuffer.appendmode = 1
                selBuffer.collecttimestamps = 1
                selBuffer.collectsourcevalues = 1
                selBuffer.timestampresolution = 1e-6
                if (selBuffer.fillmode ~= nil) then
                    selBuffer.fillmode = selSmu.FILL_ONCE
                end
            end
            
        end
    end
end

InitalizeReadingBuffers()
ConfigStatusModel()
