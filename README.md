# tsp-toolkit-script-gen

The rust application to generate tsp scripts

**Open Script** regenerates the script and waits until generation and the file
write succeed before opening the editor. **Save As** first asks for a destination
file, then generates the script directly into it and opens it once the write
succeeds. Generation or write failures are shown in the generator instead of
opening a file. Save As keeps the session's original output path; canceling the
dialog generates nothing.

Generation requires a configured session output path or an explicit Save As
destination. If neither is available, generation fails with an error; no default
output location is used.

Configuration edits and channel reallocation regenerate the script in the
background. Their data updates are returned immediately; if the background
generation or write fails, the error is shown in the generator afterwards.

## Channel operation messages

The backend accepts these comma-separated channel operations, where `role` is
`bias`, `step`, or `sweep`:

- `add,role`
- `remove,role,address,model`
- `update,role,oldAddress,newAddress,oldModel,newModel`

Addresses remain TSP addresses. Including the module model distinguishes retained
invalid assignments from replacement modules at the same address. Reassignment
requires a valid destination not already in use by another assignment, resets
model-specific settings, and preserves the channel name and UUID.

Remove/update messages require explicit module models; address-only messages are
not accepted. Rejected operations and malformed operation messages preserve the
backend's existing configuration and return it with an error status in an
`evaluated_response`, allowing the UI to restore its state and display the error.
The bias, step, and sweep UI dropdowns use address-and-model identities, display
`address (model)`, retain invalid current assignments, and exclude occupied
alternatives. Changes and removals send both address and model; channel edits and
device validity lookups also match both fields.

## Role assignments

A configuration supports at most one stepper channel; no stepper is also valid.
Bias and sweep channels have no role-specific count limit and may be assigned
across any available nodes, including splitting a single role across mainframes.
Multiple roles may share a mainframe. Each assignment still requires a valid,
available device; a physical channel cannot be assigned to multiple roles.

The existing UI hides Add Step when a stepper exists.
