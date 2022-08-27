@echo off
cargo test -- --nocapture --test-threads=1
set /p flag=Build docs?
if %flag%==y cargo doc --open --no-deps --workspace
set /p flag=Create C header?
if %flag%==y cbindgen --config ./cbindgen.toml --output napi.h
set /p flag=Build release?
if %flag%==y cargo build
pause