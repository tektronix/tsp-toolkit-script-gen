--==========================================================================
-- Runs the IV Characterization sweep as configured in the IV
-- characterization script generation tool.
--
-- WARNING: Edits made to this file will be lost if changes are made in
--          the IV Characterization Script Generation tool. Be sure to
--          rename this script before making code changes to prevent
--          accidental overwrites.
--==========================================================================
function Sweep()
    local numberOfSweeps   = 3
    local aperture         = 1e-4
    local source_delay     = 2e-3
    local measure_delay    = 5e-3
    local measure_count    = 1
    
    local sweep1 = { slot_id = 1, ch_id = 1 }
    sweep1.slot = localnode.slot[sweep1.slot_id]
    if sweep1.slot.model ~= "MSMU60-2" then error("Expected MSMU60-2 in slot " .. sweep1.slot_id .. " but found " .. sweep1.slot.model .. ". Please check your configuration.") end
    sweep1.ch = sweep1.slot.smu[sweep1.ch_id]
    sweep1.tm = sweep1.slot.trigger.model
    sweep1.tm_name = "localnode" .. sweep1.slot_id .. sweep1.ch_id .. " Sweep Trigger Model"
    sweep1.block_id = sweep1.slot_id .. sweep1.ch_id
    
    print("Resetting channels...")
    -- Reset the TSP-Link and timer triggers for each node
    for i = 1, 3 do localnode.tsplink.trigger[i].reset() end
    for i = 1, 4 do localnode.trigger.timer[i].reset() end
    sweep1.ch.reset()
    
    -- Abort any trigger models that may be running
    -- Only one abort per slot is necessary, so not all channels need to be listed here.
    sweep1.tm.abort()
    
    print("Configuring channels...")
    -- sweep1
    sweep1.ch.sense = sweep1.ch.SENSE_2WIRE
    sweep1.ch.source.func = sweep1.slot.smu.FUNC_DC_VOLTAGE
    sweep1.ch.trigger.source.listv({ 0.5, 1, 1.5 })
    sweep1.ch.source.autorangev = sweep1.slot.smu.ON
    sweep1.ch.source.limiti = 0.1
    sweep1.ch.source.limitv = 20
    sweep1.ch.source.delay = source_delay
    sweep1.ch.measure.aperture = aperture
    sweep1.ch.measure.count = measure_count
    sweep1.ch.measure.delay = measure_delay
    sweep1.ch.trigger.measure.iv(sweep1.ch.defbuffer1, sweep1.ch.defbuffer2)
    
    print("Configuring trigger models...")
    -- Events used by the channels to notify each other of source/measure completion
    local sweep1_source_event = sweep1.tm.EVENT_NOTIFY1
    local sweep1_measure_event = sweep1.tm.EVENT_NOTIFY2
    
    -- sweep1
    sweep1.tm.delete(sweep1.tm_name)
    sweep1.tm.create(sweep1.tm_name)
    sweep1.tm.addblock.source.action.step(sweep1.tm_name, sweep1.block_id .. "_sweep_step", sweep1.ch_id)
    sweep1.tm.addblock.notify(sweep1.tm_name, sweep1.block_id .. "_sweep_done", sweep1_source_event)
    sweep1.tm.addblock.measure(sweep1.tm_name, sweep1.block_id .. "_sweep_measure", sweep1.ch_id)
    sweep1.tm.addblock.notify(sweep1.tm_name, "localnodesweep_measure_done", sweep1_measure_event)
    sweep1.tm.addblock.branch.counter(sweep1.tm_name, sweep1.block_id .. "_sweep_inner_loop", sweep1.block_id .. "_sweep_step", 3)
    
    print("Turning on channels...")
    sweep1.ch.source.output = 1
    
    print("Initiating trigger model...")
    sweep1.tm.initiate(sweep1.tm_name)
    
    print("Waiting for trigger model to complete...")
    waitcomplete()
    
    print("Turning off channels...")
    sweep1.ch.source.output = 0
    waitcomplete()
    
    print("Printing results in CSV table (comma-delimited)...")
    print()
    local results = {
        sweep1.ch.defbuffer1,
        sweep1.ch.defbuffer2,
    }
    print("Reading Number,sweep[1].CURRENT,sweep[1].VOLTAGE")
    for row = 1, results[1].n do
        local line = tostring(row)
        for _, buffer in ipairs(results) do
            line = line .. "," .. buffer.readings[row]
        end
        print(line)
    end
    
    print()
    print("Complete")
end

Sweep()
