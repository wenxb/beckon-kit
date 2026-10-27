# beckon-kit

基于 GPUI 的独立 Rust UI 组件库，提供主题、窗口外壳、输入控件、列表、图标、画布和实时毛玻璃材质。不依赖 Beckon 业务代码。

## 接入

```toml
[dependencies]
beckon-kit = { git = "https://github.com/wenxb/beckon-kit.git", tag = "v0.1.0", features = ["platform"] }
```

GPUI 补丁已包含在库的 `vendor/` 中，**消费项目不需要配置 patch，也不要另外添加 GPUI 依赖**。类型、宏和平台入口统一从 kit 导入：

```rust,no_run
use beckon_kit::{WindowConfig, components::Button, gpui, platform};
use gpui::{AppContext, Context, IntoElement, Render, Window};

struct Hello;

impl Render for Hello {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Button::new("hello", "Hello, Beckon Kit")
    }
}

fn main() {
    platform::application().run(|cx| {
        beckon_kit::init(cx);
        WindowConfig::new("Hello")
            .open(cx, |_, cx| cx.new(|_| Hello))
            .expect("打开窗口失败");
    });
}
```

- 默认 feature 为空；只使用组件时不必启用 `platform`。
- `platform`：原生应用启动入口。
- `gallery`：可运行的组件展示。
- `test-support`：GPUI 输入、布局和离屏渲染测试支持；测试模块同样导入 `use beckon_kit::gpui;`。

## 本地开发

```sh
git clone https://github.com/wenxb/beckon-kit.git
cd beckon-kit
just kit
```

展示程序支持 `just kit --materials` 和 `just kit --smoke-test`。不使用 just 时：

```sh
cargo run --locked --features gallery --bin beckon-kit-gallery
```

需要经常联动修改时，把应用与 kit 放在同级目录，并在应用中改用本地依赖：

```toml
[dependencies]
beckon-kit = { path = "../beckon-kit", features = ["platform"] }
```

应用自己的 workspace 可把这一条放进 `[workspace.dependencies]`，成员继续通过 `workspace = true` 引用。修改本地 kit 后直接重新构建应用即可。

## 验证

```sh
just check
just test
just test --test text_area --test window_surfaces
just lint
just fmt -- --check
just boundaries
```

工具链由 `rust-toolchain.toml` 固定；构建需要 GPUI 对应的系统库、字体和渲染后端。上述验证命令的资源限制脚本目前要求 Linux、cgroup v2、systemd 用户管理器与 `taskset`，限制为 4 个 Cargo job、8 GiB 内存、禁用 swap；请串行运行。

## 更多资料

- [架构与组件边界](ARCHITECTURE.md)
- [窗口配置与行为](WINDOW_ROOT.md)
- [GPUI 补丁来源、平台限制与维护](ENGINE_PATCH.md)
- API 文档：`cargo doc --no-deps --features platform --open`

许可证：[Apache-2.0](LICENSE)。GPUI 与图标保留各自的许可证声明。当前通过 Git 分发，尚未发布到 crates.io；macOS / Windows 仍缺真实设备验收。
