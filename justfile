set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

# 列出常用命令
default:
    @just --list

# 启动组件展示，参数传给 bin；不套验证资源限制
kit *args:
    cargo run --locked --features gallery --bin beckon-kit-gallery -- "$@"

# 编译检查，包含可选平台、展示与测试入口
check *args:
    python3 scripts/run_limited.py -- cargo check --locked --all-features --all-targets "$@"

# 测试可追加 --test 等 Cargo 参数，测试线程固定为一个
test *args:
    python3 scripts/run_limited.py -- cargo test --locked --all-features "$@" -- --test-threads=1

# 严格 Clippy，覆盖全部 feature 和 target
lint *args:
    python3 scripts/run_limited.py -- cargo clippy --locked --all-features --all-targets "$@" -- -D warnings

# 格式化；只检查用 just fmt -- --check
fmt *args:
    python3 scripts/run_limited.py -- cargo fmt "$@"

# 独立库边界及验证脚本回归
boundaries:
    python3 scripts/run_limited.py -- python3 scripts/check_boundaries.py
    python3 scripts/run_limited.py -- python3 -m unittest discover -s scripts -p 'test_*.py'
