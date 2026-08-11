# Release versioning

Never package or publish an installer from changed source with a version that has already been released. Before creating a distributable build, increment the version consistently in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`; the generated installer filename and GitHub release tag must use that new version. Do not overwrite or replace an existing release artifact with different application contents under the same version.
