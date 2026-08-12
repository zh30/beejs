# Historical source archives

Moved out of the default compile surface during the runtime optimization sprint:

- `cloud/` — earlier multi-cloud adapter experiments (not referenced from `src/lib.rs`)
- `cloudnative/` — duplicate of concepts in `src/cloud_native/` (feature `cloudnative`)

Keep `src/cloud_native/` as the feature-gated cloud-native module. Do not reintroduce
these archives into the default library surface without a compile + test gate.
