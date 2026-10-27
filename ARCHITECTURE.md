# 独立组件库设计

## 目标与非目标

- 目标：自己的设计语言、组件行为、材质和验证，在独立仓库 [beckon-kit](https://github.com/wenxb/beckon-kit) 内演进；根目录是普通 Cargo package `0.1.0`（edition 2024、Apache-2.0），不属于宿主 workspace。
- 非目标：把业务 UI 搬进库、变更插件协议、从零重造渲染引擎、宣称已覆盖完整桌面 UI。宿主可以接入 kit，库仍保持独立。
- “架构更好”要靠明确边界与测试验证；不靠组件数量、层数或对其他库的优劣口号。

## 依赖与分层

```text
gallery（opt-in bin、标本与状态）
    ↓
components（原创表现、组合与可访问语义）
    ↓
interaction / text（输入生命周期、受控意图、编辑协议）
    ↓
theme / material（语义令牌、局部环境、后端能力）
    ↓
GPUI engine / image / unicode-segmentation / markdown
```

宿主只依赖 kit：类型、trait 与宏由 `beckon_kit::gpui` 重导出，平台启动由 opt-in `platform` feature 下的 `beckon_kit::platform` 提供；测试通过 `test-support` feature 启用同一引擎的测试能力，不另声明 GPUI 依赖。

`Root` 只装配共同的键盘路径和文本环境。主题不是服务定位器，不能藏剪贴板、宿主或
业务搜索状态。`ThemeScope` 用绘制生命周期限定环境，退出必恢复，不污染兄弟节点
和其他窗口；事件回调不读临时局部环境，应捕获必要数据。

GPUI、平台包严格使用同一发布版本。平台启动代码由 opt-in `platform` feature
引入；gallery / test-support 按需启用它。以后业务接入只在应用层新增依赖，库仍然不知道 `Item`、插件、仓储与窗口生命周期。
实时模糊是用户确认的引擎层扩展例外：在 `vendor/gpui-pre*` 补充场景原语、
能力检测和 WGPU 实现；八个 vendor 包使用库内相对 path 依赖保留一个引擎，
库和消费者均不需要 `[patch.crates-io]`，不修改业务组件。

## 状态模型

| 类型 | 状态所有者 | 变化路径 |
| --- | --- | --- |
| 按钮 | 宿主决定动作与忙碌 | 一次激活请求 |
| 复选 / 开关 / 分段 / 滑杆 | 宿主 | 新值请求 → 更新宿主 → notify |
| 焦点 / 按压 / 拖动 | 控件的 keyed state | GPUI 输入生命周期 |
| 文本 / 选择 / IME / undo | `Entity<TextField>` | GPUI 原生输入协议 → 编辑 → emit |
| 画布场景 / 几何 | 调用方的不可变 `Arc<canvas::Scene>` | 新场景 → 原生路径 / 文字编译；同身份复用 |
| 应用默认主题 | GPUI `Global` 中的主题环境 | `Theme::install` → 刷新所有窗口 |
| 窗口 / 子树主题覆盖 | `WindowConfig::theme` / `ThemeScope` | 显式覆盖优先，不反向改全局 |

复用 GPUI 的 `Global` / `Entity` / `observe_global`，不再维护平行 Store。
业务共享状态由宿主定义；全局状态读取不隐式建立响应依赖，主题传播由 kit 统一负责。

简单控件不能偷偷持有第二份 value；否则程序性变更、外部状态与控件显示会脱节。
文本编辑是例外，不用一个 String 回调草率模拟编辑器。其纯文本核心与输入适配/绘制
分开；UTF-8 是储存单位，UTF-16 是平台协议单位，grapheme 是用户删除与移动单位。
Word 与 grapheme 边界使用成熟 Unicode 库，不写字节删字符的替代品。

保持稳定控件/选项 id。默认焦点使用 GPUI keyed state，显式句柄可由宿主持有。
Tab 停靠信息写在 FocusHandle 上；只给显式 `track_focus` 的 div 设置 tab_index 不够。
按钮提交与按压反馈分离，焦点变化或移出后释放不应误激活其他控件。
画布捕获按下目标与起点，窗口级 Move / Up 允许移出视口；丢失按钮 Cancel，移除释放状态。
kit 统一绘制 / 命中变换与裁剪，不拥有标注工具或文字编辑器；编辑复用 TextField。

## 样式与材质

- Colors 区分中性 `primary` 与 `info / success / warning / danger` 状态，不提供统一品牌 accent。
  主要操作用墨色，默认激活用 info，开关用 success；尺寸、字号与密度集中定义。
- `Icon` 复用 GPUI SVG 和 Styled，不自造矢量渲染器。内置小型 Lucide 目录带固定来源与许可；
  自定义资源由字节或宿主 AssetSource 提供。图标装饰与父级交互 / 可访问名称分离。
- `Theme::new` 返回可替换默认值；宿主可以直接调整令牌，不注册全局有副作用的样式插件。
- `Styled` / `StyleRefinement` 用于布局与有意的局部覆盖；Surface 的底色由 Material 管理，`.bg(...)` 不代替材质。
- 宿主需求先区分通用功能与专属外观：前者补标准组件，后者补可复用的定制接口，配色、尺寸与组合留在宿主；不能新增业务专用变体或改变库默认设计来迁就某一页面。
- kit 缺能力不构成省略需求、降低效果或在宿主重复实现控件的理由。定制不能绕过组件行为与语义；新增接口同时验证默认、覆盖和真实输入路径。
- Surface 的背景和前景占不同绘制顺序；不能指望 GPUI 同层 quads/text/images 的批处理顺序。
- Canvas 按叶子隔离绘制顺序，即使嵌在 Surface 的批处理层内，后声明的矩形仍覆盖之前的路径与文字；GPU 回归同时验证变换后的组裁剪。
- 模糊来源与能力是明确类型。失败/区域不完整/降低透明度/提高对比度均实心降级。
- 小字的默认对比度测试覆盖明暗实心以及磨砂在极黑/极白背景的最差组合。
- 图像准备不在 paint 中；准备结果不可变、可后台生成、可共享，缓存身份稳定。
- 控件动效用有限 ease-out，表现值存于 GPUI keyed state，由 `request_animation_frame`
  与调度器时钟驱动；可中断且不延迟业务状态，尊重 `reduce_motion`，不另开定时器或空闲循环。

## 实时毛玻璃渲染契约

`Surface::frosted` 默认选择 `SceneBackdrop`。材质配置与组件无 GPU 对象，Window 负责
逻辑像素 → 物理像素、祖先 opacity 和 content mask，renderer 负责 GPU 资源。

1. `BackdropBlur` 是全局绘制屏障，即使位于已有 batching layer 中也不能复用旧 order。
   它后面的邻居即使位于高斯 halo 内，也不能提前进入被采样的画面。
2. 含模糊的场景先画进可采样中间目标；每个屏障只复制当前目标的必要区域和约 3σ halo。
3. 面积预滤波不跳过源像素，避免大半径、条纹/文字和小数 DPI 产生稀疏采样 alias。
   线性预乘 RGBA16Float 上进行水平/垂直高斯，随后重建合成。
4. 用 snapshot 与滤波结果插值替换覆盖区域；不能 source-over 再叠一次背景导致 alpha 加深。
   圆角和 content mask 只裁切输出，不把必要的采样 halo 剪掉。
5. tint、边界高光和子控件在屏障后绘制，前景不模糊；多个/嵌套材质按当前顺序读取场景。
6. 缓存只重放绘制操作，不缓存背景截图；未变的 glass 子树仍采样当前帧背景。
7. 无模糊场景保留原直接渲染路径。模糊 pipeline 延迟建立，纹理跨帧复用；调整大小/
   设备恢复使纹理失效。每个区域使用独立不可变 uniform，不能复用被队列写成最终值的参数。

runtime paint/render 不 CPU 回读、不捕获桌面、不等待 GPU、不 CPU 卷积。
离屏测试中的 CPU 回读仅用于数值断言；不写文件、不读图、不向 CLI 输出图像。
当前 Linux WGPU 支持；Metal/DirectX 的兼容 match 不等于已实现模糊。

## 开发准入

新增组件先明确：状态归属、键盘行为、语义/标签、禁用、布局极值、材料组合。
实现和展示标本同时进入这个 crate。行为用 `#[gpui::test]` 与真实输入分发；布局用
debug bounds；材质用绘制顺序/alpha/圆角断言和真实 GPU 离屏路径。

默认测试不能要求桌面会话或系统字体；完整 gallery 验证需要正常的 GPU/软件渲染后端
与平台字体。不要把 GPU 初始化失败静默跳过当成通过。当前用户禁止读图，保持纯文本验证。

检查入口：`just boundaries` 守住禁用依赖、单一引擎、vendor 连接和 600 行文件上限，
同时运行验证脚本测试；`just check` / `just test` / `just lint` / `just fmt -- --check`
分别检查编译、行为与渲染路径、严格 Clippy 和格式。
这些命令通过 `scripts/run_limited.py` 串行执行，限制 Linux 上整个子进程树的内存，
包括 rustc/linker/GPU 测试；不能只用 `--test-threads=1` 忽略编译和链接的并行内存。

## 尚未交付的边界

- `WindowRoot` 首版已实现原生 / 客户端装饰、几何和 Linux WGPU 圆角裁剪，见
  [WINDOW_ROOT.md](WINDOW_ROOT.md)；macOS / Windows 真实设备验收与精确客户端外框仍未交付。
  轻量 `Root` 继续只负责内容环境与焦点路径。
- `Select` 已有单窗口 keyed popup、GPUI 定位和局部键盘路径；通用 menu / popover /
  dialog / toast 的 overlay 系统仍未交付，需要统一焦点恢复与关闭策略，不能各自新增全局监听。纯图标按钮的简单标签提示
  直接复用 GPUI tooltip 的生命周期与定位，不作为独立浮层系统。
- 多行 `TextField` 已覆盖普通表单的软折行、滚动、选择与 IME。代码高亮和大文档仍未交付：
  不把 `TextBuffer` 扩成自造 IDE，先评估文本数据结构与 parser，保留组件与编辑核心边界。
- Metal / DirectX / Web 场景模糊与跨平台真实设备验证，及大规模重叠材质的性能基准。
- 完整屏幕阅读器验收：组件已有 roles / labels / values / disabled / busy，仍需要平台辅助
  技术会话验证，不能用截图测试替代。
- 展示页为侧栏导航与右侧组件文档；右上角主题 / 密度操作复用 Button 的纯图标模式，
  保留中文标签、键盘与受控选中语义。宿主接入另行授权，不反向引入业务依赖。
