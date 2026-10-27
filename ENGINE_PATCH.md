# 同版本 GPUI 引擎补丁

## 来源与范围

源码归 [wenxb/beckon-kit](https://github.com/wenxb/beckon-kit) 独立仓库维护。
发布包为 crates.io `gpui-pre* = 0.3.9`，上游快照为 Zed
`93d745668805029476e6d7fbb19652116fed51de`。八个 vendor crate 保留 Apache-2.0
许可证和原 package/version；只引入文本源码与清单，没有复制图片/字体资源。

这是实时毛玻璃与窗口根容器所需的同版本引擎扩展，不切换 Bezel fork，也不移植任何 Bezel 实现。
Bezel 仅参考公开材质行为文档；`beckon-kit` 的组件、状态和新增模糊模块独立设计。

| 本地 crate | 本次职责 |
| --- | --- |
| `gpui-pre` | scene 原语/屏障/重放；Window API；默认 false 的平台能力；test-support |
| `gpui-pre-wgpu` | 中间场景、区域 halo、高斯滤波与合成；透明离屏测试入口 |
| `gpui-pre-linux` | X11/Wayland window 查询各自 renderer 的实际能力 |
| `gpui-pre-apple` / `gpui-pre-windows` | 处理新增 PrimitiveBatch 的兼容分支；不广告模糊支持 |
| `gpui-pre-platform` | 同版本平台入口，连接库内各平台后端 |
| `gpui-pre-macos` / `gpui-pre-web` | 同版本平台装配依赖，保持引擎类型身份一致 |

`gpui-pre-linux` 原有 Wayland 身份初始化修复是前序工作，升级时不得误删。
窗口扩展另补运行时最小尺寸、Wayland 状态变化通知与输入区域能力区分。

## 窗口裁剪与装配扩展

- `RoundedClip` / `Window::with_rounded_clip`：不改变布局的身份化分组裁剪；同时约束
  应用内命中测试。缓存 paint operations 与 deferred draws 保留原始作用域。
- WGPU 的 `backdrop/clip.rs` / `clip.wgsl` 在组开始保存背景，结束时只应用一次圆角覆盖；
  WGPU 已实现的文字、图片、路径、quad 与场景模糊共用此屏障，不分别累积边缘 alpha。
  模糊仍在未裁剪场景中采样 halo；外投影由 Root 在裁剪外绘制。WGPU 上游本身不绘制
  嵌入视频 surface，本补丁没有新增该后端。
- 仅存在圆角组时分配额外纹理，按尺寸 / 设备复用；嵌套最多 8 层，裁剪纹理总预算
  256 MiB，表面改变或窗口销毁时释放。原生窗框不启用整窗组裁剪。
- Metal / DirectX 暂不支持精确分组裁剪；平台能力保持 false，Kit 拒绝强制客户端路径。
- `App::try_open_window` 使用可失败的首次根实体构造；构造失败清理窗口身份和原生表面。
  原有 `App::open_window` 委托此入口，保持原签名。
- `Window::set_window_min_size` 接收含 client inset 的逻辑尺寸；X11 转设备像素，
  Wayland 转为不含 inset 的 xdg geometry 尺寸。Kit 仅在 Linux 客户端动态协调此能力。
- TestWindow 只记录移动、缩放、最小化、zoom、菜单、最小尺寸和 inset 请求，不伪造
  系统回复；测试显式注入 decoration / tiling / fullscreen / maximized 状态。
- `Window::set_native_caption_buttons_visible` 在 macOS 主线程隐藏原生窗控，保留原生
  外框、阴影与缩放；Windows 的透明标题栏和 Linux 客户端装饰不另显示系统按钮。
- test-support 为带稳定 id 的 div 记录完成帧布局与可访问性事实，缓存 paint replay
  同步重放记录；宿主测试不再依赖旧组件库的观测层。普通构建不保存这些测试记录。
- 宿主通过 kit 的 `gpui` 重导出和模块内同名别名使用宏；无需修改宏包或新增引擎身份。

尚未经过真实 macOS / Windows 设备验收；补丁不把默认 Server decoration 当成系统
圆角半径证据，不承诺任意原生阴影参数。

## 透明描边的抗锯齿合成

WGPU / Metal / DirectX 描边内缘先将覆盖率乘到描边 alpha，再做 source-over。
不能在直通 RGB 与透明黑之间直接插值后再乘 alpha，否则空心描边会在圆角内侧出现黑缝。
两层 alpha 都为零时返回透明黑，避免 `0 / 0`。Linux 通过 WGPU 离屏像素测试验证
四角透明外部、灰白内缘和不透明内容；Metal / DirectX 同步公式，尚未在原生设备验证。

## 控件弹簧时钟

`SpringAnimationElement` 使用 `cx.background_executor().now()` 与 GPUI 调度器共用时钟。
运行时仍是平台单调时钟；test-support 的 `advance_clock` 可以确定性驱动弹簧，
无需 sleep，也不会把真实时间误差当作控件运动。弹簧积分、速度连续性和减少动态效果
逻辑仍使用 GPUI 原有实现。当前 Kit 控件已改用有限 ease-out，共用调度器时钟与原生
动画帧请求，不依赖这段弹簧实现；该补丁保留为引擎能力，不修改其他调用方。

## 维护边界

- 普通根 package 的 `Cargo.toml` 与八个 vendor 包的相对 path 依赖是接线点；库内和消费者均无 `[patch.crates-io]`，不修改 Cargo registry 缓存。
- `scene/`、`window/backdrop.rs`、`app/test_context/backdrop.rs`、WGPU 的 `backdrop/`
  以及 `wgpu_renderer/frame.rs` / `headless.rs` 为拆分后的聚焦模块，均不超过 600 行。
- 上游遗留大文件仍保持上游主体；不为行数口号重构无关平台代码。
- 普通场景不建模糊 pipeline/纹理，仍走直接渲染。模糊资源只在需要时创建并复用。
- 基本 GPU 测试不依赖资产。因为没有 vendored 图片/字体，完整上游 examples/test
  不能默认视为可运行；验收使用 kit 的真实 GPUI test-support / headless integration。

## 升级清单

1. GPUI / 平台包同时升级，对照发布包文本差异；保留原许可证，不换成组件库 fork。
2. 检查所有 PrimitiveBatch 消费者的完整 match，以及每个平台的能力声明。
3. 重跑场景屏障/重放测试，尤其父 batching layer 内的 halo 邻居和嵌套 order。
4. 重跑真实 GPU：当前帧更新、线性滤波、透明 alpha、圆角/裁剪、小数 DPI、大 sigma、resize。
5. 重跑缓存 glass + 真实字体/控件、gallery 输入及既有业务消费方编译。
6. 验证资源上限脚本、边界检查、格式和 Clippy；不能把测试跳过当成通过。

```sh
just test
```

平台能力缺失仍实心降级。尚无真实 Metal/DirectX/Web 模糊或帧耗时基准，不能据此宣称
已经在架构/性能上全面超过其他库。
