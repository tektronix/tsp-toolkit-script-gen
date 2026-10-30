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
    local numberOfSteps    = 10
    local numberOfSweeps   = 10
    local stepToSweepDelay = 0
    local nplc             = 1
    local source_delay     = 0
    local measure_count    = 1
    
    local bias1 = { slot_id = 1, ch_id = 1 }
    bias1.slot = localnode.slot[bias1.slot_id]
    if bias1.slot.model ~= "MSMU60-2" then error("Expected MSMU60-2 in slot " .. bias1.slot_id .. " but found " .. bias1.slot.model .. ". Please check your configuration.") end
    bias1.ch = bias1.slot.smu[bias1.ch_id]
    bias1.tm = bias1.slot.trigger.model
    bias1.tm_name = "localnode" .. bias1.slot_id .. bias1.ch_id .. " Bias Trigger Model"
    bias1.block_id = bias1.slot_id .. bias1.ch_id
    
    local bias2 = { slot_id = 1, ch_id = 2 }
    bias2.slot = localnode.slot[bias2.slot_id]
    bias2.ch = bias2.slot.smu[bias2.ch_id]
    bias2.tm = bias2.slot.trigger.model
    bias2.tm_name = "localnode" .. bias2.slot_id .. bias2.ch_id .. " Bias Trigger Model"
    bias2.block_id = bias2.slot_id .. bias2.ch_id
    
    local step1 = { slot_id = 2, ch_id = 1 }
    step1.slot = localnode.slot[step1.slot_id]
    if step1.slot.model ~= "MPSU50-2" then error("Expected MPSU50-2 in slot " .. step1.slot_id .. " but found " .. step1.slot.model .. ". Please check your configuration.") end
    step1.ch = step1.slot.psu[step1.ch_id]
    step1.tm = step1.slot.trigger.model
    step1.tm_name = "localnode" .. step1.slot_id .. step1.ch_id .. " Step Trigger Model"
    step1.block_id = step1.slot_id .. step1.ch_id
    
    local sweep1 = { slot_id = 2, ch_id = 2 }
    sweep1.slot = localnode.slot[sweep1.slot_id]
    sweep1.ch = sweep1.slot.psu[sweep1.ch_id]
    sweep1.tm = sweep1.slot.trigger.model
    sweep1.tm_name = "localnode" .. sweep1.slot_id .. sweep1.ch_id .. " Sweep Trigger Model"
    sweep1.block_id = sweep1.slot_id .. sweep1.ch_id
    
    local sweep2 = { slot_id = 3, ch_id = 1 }
    sweep2.slot = localnode.slot[sweep2.slot_id]
    if sweep2.slot.model ~= "MSMU200-2" then error("Expected MSMU200-2 in slot " .. sweep2.slot_id .. " but found " .. sweep2.slot.model .. ". Please check your configuration.") end
    sweep2.ch = sweep2.slot.smu[sweep2.ch_id]
    sweep2.tm = sweep2.slot.trigger.model
    sweep2.tm_name = "localnode" .. sweep2.slot_id .. sweep2.ch_id .. " Sweep Trigger Model"
    sweep2.block_id = sweep2.slot_id .. sweep2.ch_id
    
    local sweep3 = { slot_id = 3, ch_id = 2 }
    sweep3.slot = localnode.slot[sweep3.slot_id]
    sweep3.ch = sweep3.slot.smu[sweep3.ch_id]
    sweep3.tm = sweep3.slot.trigger.model
    sweep3.tm_name = "localnode" .. sweep3.slot_id .. sweep3.ch_id .. " Sweep Trigger Model"
    sweep3.block_id = sweep3.slot_id .. sweep3.ch_id
    
    print("Resetting channels...")
    -- Reset the TSP-Link and timer triggers for each node
    for i = 1, 3 do localnode.tsplink.trigger[i].reset() end
    for i = 1, 4 do localnode.trigger.timer[i].reset() end
    bias1.ch.reset()
    bias2.ch.reset()
    step1.ch.reset()
    sweep1.ch.reset()
    sweep2.ch.reset()
    sweep3.ch.reset()
    
    -- Abort any trigger models that may be running
    -- Only one abort per slot is necessary, so not all channels need to be listed here.
    bias1.tm.abort()
    step1.tm.abort()
    sweep2.tm.abort()
    
    print("Configuring channels...")
    -- bias1
    bias1.ch.sense = bias1.ch.SENSE_2WIRE
    bias1.ch.source.func = bias1.slot.smu.FUNC_DC_VOLTAGE
    bias1.ch.source.levelv = 0
    bias1.ch.source.autorangev = bias1.slot.smu.ON
    bias1.ch.source.limiti = 0.1
    bias1.ch.source.limitv = 20
    bias1.ch.source.delay = source_delay
    bias1.ch.measure.nplc = nplc
    bias1.ch.measure.autorangei = bias1.slot.smu.ON
    bias1.ch.measure.count = measure_count
    bias1.ch.trigger.measure.i(bias1.ch.defbuffer1)
    -- bias2
    bias2.ch.sense = bias2.ch.SENSE_2WIRE
    bias2.ch.source.func = bias2.slot.smu.FUNC_DC_VOLTAGE
    bias2.ch.source.levelv = 0
    bias2.ch.source.autorangev = bias2.slot.smu.ON
    bias2.ch.source.limiti = 0.1
    bias2.ch.source.limitv = 20
    bias2.ch.source.delay = source_delay
    bias2.ch.measure.nplc = nplc
    bias2.ch.measure.count = measure_count
    bias2.ch.trigger.measure.iv(bias2.ch.defbuffer1, bias2.ch.defbuffer2)
    -- step1
    step1.ch.trigger.source.listv({ 1, 2 })
    step1.ch.measure.count = measure_count
    step1.ch.trigger.measure.i(step1.ch.defbuffer1)
    -- sweep1
    sweep1.ch.trigger.source.logv(1e-3, 10, numberOfSweeps)
    sweep1.ch.measure.count = measure_count
    sweep1.ch.trigger.measure.v(sweep1.ch.defbuffer1)
    -- sweep2
    sweep2.ch.sense = sweep2.ch.SENSE_2WIRE
    sweep2.ch.source.func = sweep2.slot.smu.FUNC_DC_VOLTAGE
    sweep2.ch.trigger.source.logv(1e-3, 10, numberOfSweeps)
    sweep2.ch.source.autorangev = sweep2.slot.smu.ON
    sweep2.ch.source.limiti = 0.1
    sweep2.ch.source.limitv = 20
    sweep2.ch.source.delay = source_delay
    sweep2.ch.measure.nplc = nplc
    sweep2.ch.measure.autorangev = sweep2.slot.smu.ON
    sweep2.ch.measure.count = measure_count
    sweep2.ch.trigger.measure.v(sweep2.ch.defbuffer1)
    -- sweep3
    sweep3.ch.sense = sweep3.ch.SENSE_2WIRE
    sweep3.ch.source.func = sweep3.slot.smu.FUNC_DC_VOLTAGE
    sweep3.ch.trigger.source.logv(1e-3, 10, numberOfSweeps)
    sweep3.ch.source.autorangev = sweep3.slot.smu.ON
    sweep3.ch.source.limiti = 0.1
    sweep3.ch.source.limitv = 20
    sweep3.ch.source.delay = source_delay
    sweep3.ch.measure.nplc = nplc
    sweep3.ch.measure.autorangev = sweep3.slot.smu.ON
    sweep3.ch.measure.count = measure_count
    sweep3.ch.trigger.measure.v(sweep3.ch.defbuffer1)
    
    print("Configuring trigger models...")
    -- Events used by the channels to notify each other of source/measure completion
    local bias1_source_event = bias1.tm.EVENT_NOTIFY1
    local bias1_measure_event = bias1.tm.EVENT_NOTIFY2
    local bias2_source_event = bias2.tm.EVENT_NOTIFY3
    local bias2_measure_event = bias2.tm.EVENT_NOTIFY4
    local step1_source_event = step1.tm.EVENT_NOTIFY1
    local step1_measure_event = step1.tm.EVENT_NOTIFY2
    local sweep1_source_event = sweep1.tm.EVENT_NOTIFY3
    local sweep1_measure_event = sweep1.tm.EVENT_NOTIFY4
    local sweep2_source_event = sweep2.tm.EVENT_NOTIFY1
    local sweep2_measure_event = sweep2.tm.EVENT_NOTIFY2
    local sweep3_source_event = sweep3.tm.EVENT_NOTIFY3
    local sweep3_measure_event = sweep3.tm.EVENT_NOTIFY4
    
    -- bias1
    bias1.tm.delete(bias1.tm_name)
    bias1.tm.create(bias1.tm_name)
    bias1.tm.addblock.wait(bias1.tm_name, bias1.block_id .. "_waitall_bias_1", bias2_source_event)
    bias1.tm.addblock.notify(bias1.tm_name, bias1.block_id .. "_bias_notify", bias1_source_event)
    bias1.tm.addblock.wait(bias1.tm_name, bias1.block_id .. "_wait_sweep", sweep1_source_event, bias1.tm.CLEAR_NEVER)
    bias1.tm.addblock.measure(bias1.tm_name, bias1.block_id .. "_bias_measure", bias1.ch_id)
    bias1.tm.addblock.wait(bias1.tm_name, bias1.block_id .. "_waitall_bias_meas_1", bias2_measure_event)
    bias1.tm.addblock.notify(bias1.tm_name, bias1.block_id .. "_bias_meas_notify", bias1_measure_event)
    bias1.tm.addblock.wait(bias1.tm_name, bias1.block_id .. "_bias_wait_sweep_meas", sweep1_measure_event, bias1.tm.CLEAR_NEVER)
    bias1.tm.addblock.wait(bias1.tm_name, bias1.block_id .. "_bias_wait_step_meas", step1_measure_event, bias1.tm.CLEAR_NEVER)
    bias1.tm.addblock.branch.counter(bias1.tm_name, bias1.block_id .. "_loop", bias1.block_id .. "_bias_notify", 2 * numberOfSweeps)
    
    -- bias2
    bias2.tm.delete(bias2.tm_name)
    bias2.tm.create(bias2.tm_name)
    bias2.tm.addblock.notify(bias2.tm_name, bias2.block_id .. "_bias_notify", bias2_source_event)
    bias2.tm.addblock.wait(bias2.tm_name, bias2.block_id .. "_wait_sweep", sweep1_source_event, bias2.tm.CLEAR_NEVER)
    bias2.tm.addblock.measure(bias2.tm_name, bias2.block_id .. "_bias_measure", bias2.ch_id)
    bias2.tm.addblock.notify(bias2.tm_name, bias2.block_id .. "_bias_meas_notify", bias2_measure_event)
    bias2.tm.addblock.wait(bias2.tm_name, bias2.block_id .. "_bias_wait_sweep_meas", sweep1_measure_event, bias2.tm.CLEAR_NEVER)
    bias2.tm.addblock.wait(bias2.tm_name, bias2.block_id .. "_bias_wait_step_meas", step1_measure_event, bias2.tm.CLEAR_NEVER)
    bias2.tm.addblock.branch.counter(bias2.tm_name, bias2.block_id .. "_loop", bias2.block_id .. "_bias_notify", 2 * numberOfSweeps)
    
    -- step1
    step1.tm.delete(step1.tm_name)
    step1.tm.create(step1.tm_name)
    step1.tm.addblock.wait(step1.tm_name, step1.block_id .. "_step_wait_bias", bias1_source_event, step1.tm.CLEAR_NEVER)
    step1.tm.addblock.source.action.step(step1.tm_name, step1.block_id .. "_step_step", step1.ch_id)
    step1.tm.addblock.delay.constant(step1.tm_name, step1.block_id .. "_step_delay_sweep", stepToSweepDelay)
    step1.tm.addblock.notify(step1.tm_name, step1.block_id .. "_step_done", step1_source_event)
    step1.tm.addblock.wait(step1.tm_name, step1.block_id .. "_step_wait_sweep", sweep1_source_event, step1.tm.CLEAR_NEVER)
    step1.tm.addblock.measure(step1.tm_name, step1.block_id .. "_step_measure", step1.ch_id)
    step1.tm.addblock.notify(step1.tm_name, step1.block_id .. "_step_meas done", step1_measure_event)
    step1.tm.addblock.wait(step1.tm_name, step1.block_id .. "_step_wait_sweep_bias_meas1", sweep1_measure_event, step1.tm.CLEAR_NEVER, step1.tm.WAIT_AND, bias1_measure_event)
    step1.tm.addblock.branch.counter(step1.tm_name, step1.block_id .. "_step_inner_loop", step1.block_id .. "_step_wait_sweep", numberOfSweeps)
    step1.tm.addblock.branch.counter(step1.tm_name, step1.block_id .. "_step_outer_loop", step1.block_id .. "_step_wait_bias", 2)
    step1.tm.addblock.notify(step1.tm_name, step1.block_id .. "_final_notify", step1_source_event)
    
    -- sweep1
    sweep1.tm.delete(sweep1.tm_name)
    sweep1.tm.create(sweep1.tm_name)
    sweep1.tm.addblock.wait(sweep1.tm_name, sweep1.block_id .. "_sweep_wait_bias", bias1_source_event, sweep1.tm.CLEAR_NEVER)
    sweep1.tm.addblock.wait(sweep1.tm_name, sweep1.block_id .. "_sweep_wait_step", step1_source_event, sweep1.tm.CLEAR_NEVER)
    sweep1.tm.addblock.source.action.step(sweep1.tm_name, sweep1.block_id .. "_sweep_step", sweep1.ch_id)
    sweep1.tm.addblock.wait(sweep1.tm_name, sweep1.block_id .. "_sweep_waitall_1", sweep2_source_event, sweep1.tm.CLEAR_NEVER, sweep1.tm.WAIT_AND, sweep3_source_event)
    sweep1.tm.addblock.notify(sweep1.tm_name, sweep1.block_id .. "_sweep_done", sweep1_source_event)
    sweep1.tm.addblock.measure(sweep1.tm_name, sweep1.block_id .. "_sweep_measure", sweep1.ch_id)
    sweep1.tm.addblock.wait(sweep1.tm_name, sweep1.block_id .. "_sweep_other_measure_waitall_1", sweep2_measure_event, sweep1.tm.CLEAR_NEVER, sweep1.tm.WAIT_AND, sweep3_measure_event)
    sweep1.tm.addblock.notify(sweep1.tm_name, "localnodesweep_measure_done", sweep1_measure_event)
    sweep1.tm.addblock.wait(sweep1.tm_name, sweep1.block_id .. "_sweep_meas done1", step1_measure_event, sweep1.tm.CLEAR_NEVER, sweep1.tm.WAIT_AND, bias1_measure_event)
    sweep1.tm.addblock.branch.counter(sweep1.tm_name, sweep1.block_id .. "_sweep_inner_loop", sweep1.block_id .. "_sweep_step", numberOfSweeps)
    sweep1.tm.addblock.branch.counter(sweep1.tm_name, sweep1.block_id .. "_sweep_outer_loop", sweep1.block_id .. "_sweep_wait_step", 2)
    
    -- sweep2
    sweep2.tm.delete(sweep2.tm_name)
    sweep2.tm.create(sweep2.tm_name)
    sweep2.tm.addblock.wait(sweep2.tm_name, sweep2.block_id .. "_sweep_wait_bias", bias1_source_event, sweep2.tm.CLEAR_NEVER)
    sweep2.tm.addblock.wait(sweep2.tm_name, sweep2.block_id .. "_sweep_wait_step", step1_source_event, sweep2.tm.CLEAR_NEVER)
    sweep2.tm.addblock.source.action.step(sweep2.tm_name, sweep2.block_id .. "_sweep_step", sweep2.ch_id)
    sweep2.tm.addblock.notify(sweep2.tm_name, sweep2.block_id .. "_sweep_done", sweep2_source_event)
    sweep2.tm.addblock.measure(sweep2.tm_name, sweep2.block_id .. "_sweep_measure", sweep2.ch_id)
    sweep2.tm.addblock.notify(sweep2.tm_name, "localnodesweep_measure_done", sweep2_measure_event)
    sweep2.tm.addblock.wait(sweep2.tm_name, sweep2.block_id .. "_sweep_meas done1", step1_measure_event, sweep2.tm.CLEAR_NEVER, sweep2.tm.WAIT_AND, bias1_measure_event)
    sweep2.tm.addblock.branch.counter(sweep2.tm_name, sweep2.block_id .. "_sweep_inner_loop", sweep2.block_id .. "_sweep_step", numberOfSweeps)
    sweep2.tm.addblock.branch.counter(sweep2.tm_name, sweep2.block_id .. "_sweep_outer_loop", sweep2.block_id .. "_sweep_wait_step", 2)
    
    -- sweep3
    sweep3.tm.delete(sweep3.tm_name)
    sweep3.tm.create(sweep3.tm_name)
    sweep3.tm.addblock.wait(sweep3.tm_name, sweep3.block_id .. "_sweep_wait_bias", bias1_source_event, sweep3.tm.CLEAR_NEVER)
    sweep3.tm.addblock.wait(sweep3.tm_name, sweep3.block_id .. "_sweep_wait_step", step1_source_event, sweep3.tm.CLEAR_NEVER)
    sweep3.tm.addblock.source.action.step(sweep3.tm_name, sweep3.block_id .. "_sweep_step", sweep3.ch_id)
    sweep3.tm.addblock.notify(sweep3.tm_name, sweep3.block_id .. "_sweep_done", sweep3_source_event)
    sweep3.tm.addblock.measure(sweep3.tm_name, sweep3.block_id .. "_sweep_measure", sweep3.ch_id)
    sweep3.tm.addblock.notify(sweep3.tm_name, "localnodesweep_measure_done", sweep3_measure_event)
    sweep3.tm.addblock.wait(sweep3.tm_name, sweep3.block_id .. "_sweep_meas done1", step1_measure_event, sweep3.tm.CLEAR_NEVER, sweep3.tm.WAIT_AND, bias1_measure_event)
    sweep3.tm.addblock.branch.counter(sweep3.tm_name, sweep3.block_id .. "_sweep_inner_loop", sweep3.block_id .. "_sweep_step", numberOfSweeps)
    sweep3.tm.addblock.branch.counter(sweep3.tm_name, sweep3.block_id .. "_sweep_outer_loop", sweep3.block_id .. "_sweep_wait_step", 2)
    
    print("Turning on channels...")
    bias1.ch.source.output = 1
    bias2.ch.source.output = 1
    step1.ch.source.output = 1
    sweep1.ch.source.output = 1
    sweep2.ch.source.output = 1
    sweep3.ch.source.output = 1
    
    print("Initiating trigger model...")
    sweep1.tm.initiate(sweep1.tm_name)
    sweep2.tm.initiate(sweep2.tm_name)
    sweep3.tm.initiate(sweep3.tm_name)
    step1.tm.initiate(step1.tm_name)
    bias1.tm.initiate(bias1.tm_name)
    bias2.tm.initiate(bias2.tm_name)
    
    print("Waiting for trigger model to complete...")
    waitcomplete()
    
    print("Turning off channels...")
    bias1.ch.source.output = 0
    bias2.ch.source.output = 0
    step1.ch.source.output = 0
    sweep1.ch.source.output = 0
    sweep2.ch.source.output = 0
    sweep3.ch.source.output = 0
    waitcomplete()
    
    print("Printing results in CSV table (comma-delimited)...")
    print()
    local results = {
        bias1.ch.defbuffer1,
        bias2.ch.defbuffer1,
        bias2.ch.defbuffer2,
        step1.ch.defbuffer1,
        sweep1.ch.defbuffer1,
        sweep2.ch.defbuffer1,
        sweep3.ch.defbuffer1,
    }
    print("Reading Number,bias[1].CURRENT,bias[2].CURRENT,bias[2].VOLTAGE,step[1].CURRENT,sweep[1].VOLTAGE,sweep[2].VOLTAGE,sweep[3].VOLTAGE")
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
