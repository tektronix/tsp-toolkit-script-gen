# tsp-toolkit-script-gen
The rust application to generate tsp scripts 

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
Model-aware UI selection and save/recall migration are separate follow-up work.
