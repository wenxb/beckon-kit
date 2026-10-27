# gpui-pre-linux 0.3.9 本地补丁

来源：crates.io `gpui-pre-linux` 0.3.9，对应 Zed 提交
`93d745668805029476e6d7fbb19652116fed51de`，许可证为 Apache-2.0。
仅保留发布包的 `Cargo.toml`、`LICENSE-APACHE` 和 `src/`。
工作区通过 `[patch.crates-io]` 使用此副本；不要修改 Cargo 下载缓存。

## 改动

`src/linux/wayland/window.rs`：将顶层窗口的 `app_id`、标题设置移到
`get_toplevel` 之后、GPU 初始化之前。Mesa 在初始化渲染器时会同步 Wayland
请求；身份设置晚于同步会让 GNOME 先创建 `window:…` 的未知应用，再替换成
Beckon。窗口最大尺寸仍在渲染器建立后设置，其他平台、弹窗和 layer-shell
分支保持上游实现。

## 验证与升级

在仓库根目录执行：

```sh
cargo test -p beckon-ui --features test-support
cargo build -p beckon-app
/usr/bin/python3 scripts/check_wayland_identity.py --binary /mnt/hdd/target/beckon/debug/beckon
```

最后一项需要当前 GNOME/Wayland 会话、PyGObject 的 Gio/AT-SPI 和已安装的
Beckon `.desktop`。它检查首次启动、隐藏后重开时的身份请求顺序和 Dock 名称，
结束后退出测试进程，日志保存在临时目录。

升级 GPUI 时先对照新的窗口创建顺序；若上游已在 GPU 初始化前设置身份，
移除工作区补丁及此目录，再运行上述验证。保留上游文件布局以便核对差异。
