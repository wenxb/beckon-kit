# beckon-kit

Beckon 自己的独立 GPUI 组件库。**不是 GPUI Kit 换皮，也不是业务 `beckon-ui` 的搬迁。**

组件、状态边界、语义、令牌和材质组合均在这里独立设计；只使用现有 GPUI 引擎及
通用的 Unicode / 图像算法库。没有 `gpui-kit` / `gpui-base` / `gpui-component` /
`bezel` 或业务 crate 的依赖。

源码仓库：[wenxb/beckon-kit](https://github.com/wenxb/beckon-kit)。仓库根是普通 Cargo
package `beckon-kit` **0.1.0**，Rust **edition 2024**，许可证 **Apache-2.0**；不是
Beckon workspace 成员，也不是 virtual workspace。八个同版本 `gpui-pre* = 0.3.9`
vendor 包由库内相对 path 依赖连接，随 Git 源码一同取得；消费者不需要任何
`[patch.crates-io]`，也不直接声明另一套 GPUI 依赖。来源与维护范围见
[ENGINE_PATCH.md](ENGINE_PATCH.md)。

## 标准组件与宿主定制的边界

kit 提供宿主无关的标准组件和一致的设计、行为契约，不收纳应用页面或业务专用皮肤。
接入时缺少通用功能，就补标准组件；缺少外观表达能力，就增强通用的样式定制接口。
不能因为 kit 暂时缺能力而省略需求、降低效果，或在宿主保留另一套同类控件实现。

- **库负责**：通用组件行为与语义、合理默认设计、主题令牌、样式扩展点和内容组合。
- **宿主负责**：业务配色、特定尺寸、页面布局与组件组合；通过 kit 的公开能力定制，
  不把“某应用搜索框”等专用样式反向写进库默认值、变体或令牌。
- **定制方式**：优先使用主题令牌、GPUI `Styled` / `StyleRefinement` 与内容组合；
  只有存在独立、通用语义时才增加变体，不能为某一页面堆叠专用开关。
- **不变契约**：外观可变，稳定身份、受控状态、键盘与焦点、禁用、IME、减少动态效果
  等行为不能退化。同类控件继续共享 kit 的实现与设计规则。
- **临时禁用**：Button / Checkbox / Toggle 保留已有焦点句柄，但不响应鼠标或键盘激活，也不进入 Tab 顺序；恢复后可直接继续键盘操作。
- **新增接口验收**：提供默认及覆盖示例，测试覆盖生效、默认不变且交互仍正确；
  未实现或不支持的覆盖明确报告，不能默默忽略。

开发时遵守 [AGENTS.md](AGENTS.md)；具体分层与准入规则见
[ARCHITECTURE.md](ARCHITECTURE.md)。

## 宿主依赖入口

宿主不直接声明 GPUI 或 GPUI 平台包；由 kit 固定底层版本并重导出同一套类型、trait 与宏。
不额外包一层 `Window` / `Entity`，所以样式和回调仍遵循 GPUI 的惯例。

```toml
[dependencies]
beckon-kit = { git = "https://github.com/wenxb/beckon-kit", tag = "v0.1.0", features = ["platform"] }

[dev-dependencies]
beckon-kit = { git = "https://github.com/wenxb/beckon-kit", tag = "v0.1.0", features = ["test-support"] }
```

```rust,ignore
use beckon_kit::{gpui, platform};
use gpui::{Context, IntoElement, Render, Window, div, prelude::*};

// gpui 是 kit 的重导出；宏展开也使用这个别名，无需额外 gpui 依赖。
gpui::actions!(host, [Confirm]);
// platform::application() 由启动宿主调用，组件本身不启动事件循环。
```

仅使用组件时不启用 `platform`；`gallery` 与 `test-support` 按需开启平台能力。
测试用 `#[gpui::test]`，相应模块同样导入 `use beckon_kit::gpui;`。

## 运行与验证

需要 Rust 工具链、[just](https://just.systems/) 和 Python 3；Linux 检查还需要
cgroup v2、systemd 用户管理器及 `taskset`。平台构建与 gallery 运行仍需 GPUI 对应的
系统库、字体和可用的 GPU / 软件渲染后端。

```sh
git clone https://github.com/wenxb/beckon-kit.git
cd beckon-kit
just kit
just kit --materials # 直接进入实时毛玻璃展示页；窗口页提供外框设置
just check
just test
just test --test text_area --test window_surfaces # 定向测试
just lint
just fmt -- --check
just boundaries # 依赖/文件边界与验证脚本测试
# 不使用 just 时，在 clone 根目录运行：
cargo run --locked --features gallery --bin beckon-kit-gallery
python3 scripts/run_limited.py -- cargo test --locked --all-features -- --test-threads=1
python3 scripts/run_limited.py -- cargo clippy --locked --all-features --all-targets -- -D warnings
python3 scripts/run_limited.py -- python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/run_limited.py -- python3 scripts/check_boundaries.py
```

`gallery` 是同 crate 的 opt-in 中文组件文档：固定左侧导航，右侧独立滚动内容。
分为颜色、图标、基础控件、虚拟列表、画布、材质、窗口七页；移除宣传大标题、版本徽章与全局状态页脚，
操作结果留在对应示例中。窄窗口缩减侧栏与内容边距，不隐藏组件。保留明暗主题和密度操作；
右上角使用 Lucide 图标按钮切换明暗主题与紧凑布局，保留中文悬停提示、可访问名称和键盘路径。
紧凑布局通过柔和选中底色与辅助技术切换状态表达。原有控件身份保持不变。
库默认不链接平台启动代码。默认测试使用 GPUI `test-support`；完整验证另包含真实字体
造型与 GPU 离屏数值断言，不读取图片文件、不生成或展示截图。

`just kit` 直接调用 Cargo，不经过资源限制脚本；`check` / `test` / `lint` / `fmt` /
`boundaries` 均经过 `scripts/run_limited.py`。Linux 验证用的脚本
使用独立 systemd 用户资源组：默认四个 Cargo job、最多八个 CPU、
内存硬上限 8 GiB、禁用 swap，并输出整个进程树的内存峰值。测试仍默认单线程，
用 `--memory-mib` 可降低内存上限。小修先跑受影响的定向测试，收尾时再做完整回归。
超限会使验证失败；
缺少 cgroup v2 / 用户 systemd 时也失败，不静默退回无限制运行。
使用固定 scope 身份拒绝重复启动；不要同时运行多个重构建。
`--smoke-test` 在原生窗口中更新背景 16 帧后退出，输出能力和 GPU 文本信息；不截图。

## 已有基础

- `Button`：Primary / Secondary / Outline / Soft / Quiet / Link / Danger；文字或纯图标模式，尺寸、禁用、忙碌、按压取消、键盘激活及受控切换状态。
- `Checkbox` / `Toggle`：受控选择值、Mixed 状态、键盘路径与语义；复选框使用 info 蓝，开关使用 success 绿；外部已有字段标题时可隐藏重复文字，保留无障碍名称。
- `SegmentedControl` / `Tabs`：分段 / 胶囊 / 下划线三种 `TabVariant`，一个 Tab 停靠点、箭头循环、跳过禁用选项；选择组与页面标签保留不同辅助技术语义。
- `Slider`：统一量化域、键盘步进、越界拖动、抬起清理与辅助技术动作。
- `TextField`：默认单行编辑实体；`set_multiline(true, cx)` 开启软折行和自动高度，随换行、宽度变化、删除即时增高 / 收缩；空值按占位文字测量，至少保留一行。通过 `set_style` 的 `h(...)` 固定视口，或 `max_h(...)` 限制自动增高，溢出仍可垂直滚动；没有内部滚动时滚轮交给父容器。支持上下移动与跨行选择，Enter 换行、Ctrl/Cmd+Enter 提交。共用 UTF-16 IME、grapheme 编辑、剪贴板、撤销、禁用/只读与多行无障碍语义；Plain / Outline、公开样式覆盖、密码遮罩与禁止剪贴板导出不变。切回单行会把换行规范为空格并清空历史，开启多行不重置已有草稿/光标。
- `Select`：受控单选、禁用选项、箭头导航、回车确认、Esc / 点击外部取消。
- `VirtualList`：基于 GPUI `uniform_list`，只构建测量行与可见范围；公开 `Styled` 与原生滚动句柄接口，行内容、固定高度、交互和可访问名称由调用端提供。
- `Canvas`：不可变 `Arc<canvas::Scene>` 驱动矩形 / 箭头 / 原生单行文字，统一缩放和平移、组裁剪、后画优先命中；窗口级主指针捕获保留按下目标 / 起点，丢失按钮取消。焦点、标签与文字替代可用；文字编辑复用独立 TextField。
- `Markdown`：CommonMark / GFM 文本、列表、代码与链接；链接默认不执行，HTML 按文本显示，图片只显示替代文字。
- `ProgressBar` / `Badge`：进度使用 info；徽标支持 Neutral / Info / Success / Warning / Danger。
- `Icon` / `IconName`：39 个官方 Lucide SVG 与 1 个 Feather GitHub 图标，默认 16px、继承前景色，支持自定义字节或宿主资源路径。
- `Surface`：实心或磨砂材质，圆角、明暗、分层、能力失败与降低透明度的实心降级。
- `ThemeScope` / `Root`：局部主题、字号/密度、语义令牌和统一 Tab 路径。

输入框可用高度因布局取整或样式覆盖小于一行时，IME 候选框锚点固定在视口顶部，保留真实行高；单行与多行共用此边界处理。

### 中性结构与语义颜色

不设品牌色：浅色主题主要操作使用深墨色，深色主题仅反转其明暗对比，不引入色相。
`primary / on_primary / primary_hover / primary_pressed` 专用于主要操作；
`info / on_info / info_hover / info_pressed` 用于勾选、标签选中、进度、滑杆和链接；
`selection` 与 `focus` 同属 info。`window_border` 独立控制外框的低对比边缘，不影响控件边界和焦点环。`success / on_success` 用于开关开启与成功，
`warning` 表达注意，`danger / on_danger` 表达危险；页面背景、文字、边界保持中性灰。
原 `accent* / on_accent` 令牌被明确语义替代，不保留混用品牌色与状态色的入口。
所有颜色仍可由宿主修改，明暗主题的文字与状态前景有对比度回归测试。

## 接入方式

库不启动平台或事件循环，不初始化宿主服务，不决定系统外观。嵌入现有窗口时先调用
`init(cx)`，再把组件放到 `Root` 内；需要局部主题时用 `ThemeScope`。
完整窗口用 `WindowConfig::open`，内部自动装配轻量 Root，无需手动叠加。

应用级主题只需一处安装；已打开/后续打开的默认窗口、缓存控件和默认外框同步生效：

```rust,no_run
use beckon_kit::theme::{Appearance, Theme};
use beckon_kit::gpui::App;

fn use_dark_mode(cx: &mut App) {
    let mut theme = Theme::new(Appearance::Dark);
    theme.colors.window_border = beckon_kit::gpui::rgb(0x343434).into();
    theme.install(cx);
}
```

省略 `WindowConfig::theme` 即跟随全局；显式 `.theme(...)` / `WindowRoot::set_theme(...)`
固定单个窗口，`ThemeScope` 覆盖子树，不影响兄弟或其他窗口。`Theme::global(cx)` 读取应用默认值，
`Theme::current(cx)` 读取渲染中的最近覆盖；事件回调不要读取后者。系统明暗监听和偏好持久化由宿主负责。
外框默认值随主题更新，显式修改的圆角、边框或投影保持不变。

字段标题已经在控件外部时：
```rust,no_run
use beckon_kit::components::Toggle;
let labelled = Toggle::new("notifications", "通知", true); // 默认显示文字
let indicator = Toggle::new("notifications-compact", "通知", true).label_visible(false);
// 只隐藏可见文字；读屏名称、点击、键盘和焦点路径不变。Checkbox 支持相同配置。
```

```rust,no_run
use beckon_kit::{Root, components::{Button, ButtonVariant}, theme::{Appearance, Theme, ThemeScope}};
use beckon_kit::gpui::{App, ClickEvent, IntoElement, Window, prelude::*};

fn specimen() -> impl IntoElement {
    ThemeScope::new(
        Theme::new(Appearance::Dark),
        Root::new("root").child(
            Button::new("run", "运行")
                .variant(ButtonVariant::Primary)
                .on_click(|_: &ClickEvent, _: &mut Window, _: &mut App| {}),
        ),
    )
}
```

回调参数用引用，可直接接 GPUI `cx.listener(...)`。简单控件的 `on_change` 是
**请求新值**，宿主负责更新自己的值并 `cx.notify()`；控件不会维护第二份业务状态。
文本编辑器需要文档、选择和 IME 的一致生命周期，因此使用 `Entity<TextField>`。
程序性 `set_value` 不伪装成用户输入事件。

### 编辑器定制与受控选择

```rust,no_run
use beckon_kit::components::{Select, SelectOption, TextField, TextFieldVariant};
use beckon_kit::gpui::{Context, StyleRefinement, Styled, px, relative};

fn plain_editor(cx: &mut Context<TextField>) -> TextField {
    let mut field = TextField::new("query", "搜索", cx); // 默认 Outline
    field.set_variant(TextFieldVariant::Plain, cx);
    field.set_style(
        StyleRefinement::default().text_size(px(24.0)).line_height(relative(1.25)),
        cx,
    );
    field
}

let select = Select::new(
    "mode", "显示模式",
    [SelectOption::new("list", "列表"), SelectOption::new("grid", "网格")],
    Some("list".into()),
).on_change(|value, window, cx| { /* 更新宿主持有的选中值 */ });
```

`set_style` 替换整份覆盖；传入 `StyleRefinement::default()` 恢复默认样式。
`set_masked(true, cx)` 按 grapheme 绘制遮罩并隐藏辅助技术明文值，阻止复制和剪切；
原始文本仍由编辑实体与平台 IME 协议持有，不承诺内存加密。
组件局部键盘上下文优先于祖先快捷键；宿主特有的结果导航应显式接管，不能覆盖 IME 预编辑。

### 固定行高虚拟列表

```rust,no_run
use beckon_kit::components::VirtualList;
use beckon_kit::gpui::{UniformListScrollHandle, ScrollStrategy, div, prelude::*, px};

// 句柄由拥有视图的状态持有，不要每帧新建；元素 id 也要保持稳定。
let scroll = UniformListScrollHandle::new();
let list = VirtualList::new("rows", 10_000, |visible, _, _| {
    visible.map(|index| div().h(px(32.0)).child(format!("Row {index}"))).collect::<Vec<_>>()
}); // 默认宽度 100%，高度上限 320px，行高必须一致。
let styled = VirtualList::new("styled-rows", 10_000, |visible, _, _| {
    visible.map(|index| div().h(px(32.0)).child(format!("Row {index}"))).collect::<Vec<_>>()
}).label("记录").track_scroll(&scroll).h(px(192.0));
scroll.scroll_to_item(5000, ScrollStrategy::Center);
```

`VirtualList` 不复制集合；闭包捕获共享快照，在请求的范围内按需构建行。GPUI 会额外构建一行
用于测量，不能在行构建器中触发业务事件。行文本可用 `truncate()` 省略，但完整文本应保留在
行的 `aria_label`；滚动范围与点击身份由稳定 id / 原生句柄维持。协议宿主仍独立限制每表 500 行。

### 声明式画布

使用 `components::Canvas::new(id, Arc<components::canvas::Scene>)`，通过 `.on_pointer(...)` 接收
`PointerEvent { phase, position, origin, target }`。viewport 宽高为逻辑像素；根变换为
`viewport = scene * scale + (x, y)`，指针事件回到 scene 坐标。组变换只影响绘制 / 命中，
不改变事件坐标系。`Node::{Rect, Arrow, Text, Group}` 按顺序绘制，视口及组 clip 同时裁剪。
场景 id 稳定且 `Arc` 身份 / 字体未变时复用箭头细分和文字造型；调用方不要每帧重新分配场景。
同 id 场景替换保留捕获，Up 可在视口外，主按钮丢失产生 Cancel；移除画布释放捕获状态。
`.focus_handle(...)` 复用宿主焦点，`.label(...)` 提供名称，场景文字加入替代说明。
Canvas 不拥有文字编辑器，调用方用 TextField 编辑数据后替换 scene。独立使用者负责校验场景；
插件的非有限值、组合变换、几何和总量限额由协议宿主准入，不在 kit 引入业务依赖。

### 按钮与标签变体

```rust,no_run
use beckon_kit::components::{Button, ButtonVariant, Segment, TabVariant, Tabs};

let action = Button::new("save", "保存").variant(ButtonVariant::Soft);
let tabs = Tabs::new(
    "view", "结果视图",
    [Segment::new("list", "列表"), Segment::new("grid", "网格")],
    "list",
)
.variant(TabVariant::Underline)
.on_change(|selected, window, cx| { /* 宿主切换对应内容 */ });
```

`Tabs` 用于页面导航，内容由宿主渲染；`SegmentedControl` 用于同一内容内的选项切换。
两者共用 `TabVariant::{Segmented, Pills, Underline}`；选中状态由底色或下划线明确呈现，
hover 仅调整文字，不增加背景。按钮的 `Outline` 使用 1px 描边，其余常规变体不描边；
键盘焦点通过独立焦点环呈现，不影响布局。

`Button::content(element)` 可替换可见文字内容（例如分段显示组合键），`Button::new` 的文字仍作为无障碍名称；焦点、键盘激活与禁用行为不变。

### 图标

```rust,no_run
use beckon_kit::components::{Button, Icon, IconName};
use beckon_kit::gpui::{Styled, px, rgb};

let search = Icon::new(IconName::Search); // 16px，继承父级文字颜色
let info = Icon::new(IconName::Info).size(px(20.0)).text_color(rgb(0x2563eb));
let action = Button::new("add", "添加").leading(Icon::new(IconName::Plus));
let theme = Button::icon("theme", "切换到浅色模式", Icon::new(IconName::Sun));
let custom = Icon::data(IconName::Copy.svg()); // 也可传入自己的 SVG 字节
let asset = Icon::path("application/export.svg"); // 宿主 AssetSource 提供
```

内置图标使用 `include_bytes!`，无资源注册、运行时文件或网络请求。仅嵌入当前小目录，
不拉入完整图标包或第二套 GPUI。`Icon::data` 拷贝到共享存储，clone 共享字节；
最终绘制与缓存由 GPUI SVG 渲染器管理，不承诺整条渲染链零复制。
`Icon` 是装饰性元素，不另设焦点与点击逻辑，可访问名称由父级按钮 / 导航项提供。
仅参考 GPUI Kit 的公开使用文档，组件实现与布局独立设计；Lucide 来源版本及 ISC / MIT
许可见 `assets/lucide/`。

`Button::icon(id, label, icon)` 隐藏文字并提供方形点击区域，复用文字按钮的尺寸、变体、
禁用 / 忙碌与键盘行为。`label` 必填，用于可访问名称与 GPUI 原生悬停提示；提示保留按钮
所在的局部主题，不依赖全局默认主题。`.toggled(bool)` 声明受控切换状态，选中外观由宿主
选择按钮变体表达；点击仍只发出请求。

### 控件动效

`Toggle` 的滑钮 / 底色、`Checkbox` 的底色 / 勾选淡入淡出 / Mixed 形态，以及 `Slider`
的点击 / 键盘位置与按压反馈使用 320ms 的 cubic ease-out（`1 - (1 - t)³`）。
起步即响应，前四分之一时间完成约 58% 位移，随后减速并准时结束；不再使用慢起步、
贴近目标的弹簧拖尾，也没有回弹。底色、滑钮与描边通过 RGB 通道混合，避免 HSL 色相旋转
在中间帧引入额外颜色；颜色与透明度保持合法范围，滑块轨道端点仍然受限。
连续操作从当前显示值接续；反向立即响应，不继承旧方向的惯性。首次挂载不播入场动画，
静止后不请求动画帧。展示区的开关 / 复选框整行及 tabs 不添加 hover 或 active 底色，
tabs 也不额外添加按下变色；选中状态与键盘焦点保留。

滑块实际拖动时直接跟随指针，抓取滑钮保留抓取偏移，释放后不惯性漂移数值；动效只影响
表现，不延迟 `on_change` 或辅助技术值。宿主通过 GPUI `cx.set_reduce_motion(true)`
启用减少动态效果时，三类控件直接显示目标状态。现有 Gallery 控件自动使用这些动效。

## 窗口根容器

```rust,no_run
use beckon_kit::{WindowConfig, WindowRoot, WindowRootHandleExt, window::FrameMode};
use beckon_kit::gpui::{App, AppContext, Context, IntoElement, Render, Window, WindowHandle, div, px, size};
struct Gallery;
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement { div() }
}
fn open(cx: &mut App) -> beckon_kit::gpui::Result<WindowHandle<WindowRoot<Gallery>>> {
    let handle = WindowConfig::new("组件工作台")
        .size(size(px(980.0), px(680.0)))
        .min_size(size(px(420.0), px(380.0)))
        .client_style(|frame| frame.radius(px(16.0)))
        .open(cx, |_, cx| cx.new(|_| Gallery))?;
    let content = handle.content(cx)?;
    handle.update_content(cx, |gallery, window, cx| { let _ = (gallery, window, cx); })?;
    Ok(handle)
}
```

默认 `Adaptive + Integrated`：Linux 请求客户端装饰，macOS / Windows 保留原生外框。
`size` / `min_size` 指可见客户区（含自定义标题区，不含阴影带）。原生 bounds 逃生口
通过 `options(WindowOptions { .. })` 提供，不能与 `size` 混用；外壳管理字段冲突会报错。
`prepare(cx)` 返回 `PreparedWindow { options, root }`，可直接组合 GPUI
`try_open_window` 和可失败的 `WindowRoot::mount(root, window, cx, content_factory)`。
挂载必须发生在开窗构造闭包内，重复装饰所有者会立即拒绝。

`frame_status()` 报告请求、实际归属、客户区几何、系统状态和降级原因；通过 GPUI
`cx.subscribe` 订阅 `WindowFrameStatus`。系统管理不等于测得系统半径。
`set_title`、`set_theme`、`set_client_style` 在实体更新阶段调用；样式校验失败保持旧值。
动态投影调整同步协调客户区与缩放最小值。结构性装饰、标题区模式是创建属性。

自绘标题区的最小化 / 最大化 / 关闭使用全圆形 Lucide 图标按钮；最大化回复后，方框切换为
叠放方框并标注“还原”。按钮保留中文悬停提示、可访问名称、Tab / Enter / 空格路径，
平台或配置不支持时禁用。Windows 最大化按钮保留 Snap Layout 悬停区域；macOS 保留原生按钮。

`DragRegion` 只提供显式空白移动区；按钮放在它旁边。关闭按钮、原生关闭与
`WindowRoot::request_close` 共用 `on_close` 策略（Allow / Cancel / Pending）；待确认用
同一 `CloseRequest` 调用 `resume_close`，过期 / 跨窗口身份无效。`CloseWindow` action
及 macOS Cmd-W / 其他平台 Ctrl-W 也走此策略。Root 不退出应用。
`set_titlebar_content(Some(view.into()), cx)` 将工具控件放入融合标题区，和原生按钮保留区、
显式拖动空白区分开。`set_overlay(Some(view.into()), cx)` 提供受同一客户区裁剪的窗内浮层
slot；调用者管理浮层自身的焦点、关闭和定位，不引入全局 overlay 管理器。

Linux WGPU 使用整组 GPU 圆角覆盖，阴影不在内容裁剪范围；布局、矩形 mask、命中测试、
缓存与 deferred 绘制保留各自语义。Metal / DirectX 尚未实现精确客户端外框，强制 Client
在创建前报错。Adaptive 的 `TitlebarMode::None` 保留原生外框、隐藏原生窗控，适合由宿主
组合关闭钮的面板；macOS 通过 AppKit 隐藏 traffic lights，Windows 沿用透明标题区。
原生融合及无标题区路径仍需在真实 macOS / Windows 设备验收，不承诺任意原生圆角和阴影。

插件等自绘客户区可用 `FrameMode::None + TitlebarMode::None`：不加外框、标题区、底色或
阴影带，配置冲突直接报错。普通无框窗口不包含置顶或独占输入语义。
`WindowConfig::overlay(cx)?` 在 Linux Wayland 请求真实 layer-shell Overlay 层、全输出锚定和
Exclusive 键盘；不支持的后端或合成器明确失败，不回退成普通全屏。窗口仍经同一个
`WindowRoot` 挂载，内容状态、退出策略由调用者拥有；这不是窗内 `set_overlay` slot。

## 毛玻璃的准确边界

参考目标是 [Bezel 的磨砂材质](https://github.com/crabtalk/bezel/blob/main/www/docs/material.md)，
不包括折射、镜头边缘或 Liquid Glass。只阅读了其公开材质文档，没有搬运其组件实现。

**默认材质已经是实时场景背景模糊，不需要提供图片。** 卡片采样同一窗口、当前帧中
先于它绘制的真实控件、文字和色块；背景变化或移动时重新采样，随后绘制的前景保持清晰。
没有折射、放大、坐标扭曲或液态边缘。它不是对桌面、其他窗口的读屏模糊。

在保持 `gpui-pre = 0.3.9` 的前提下补充底层能力：kit 的八个 vendor 包通过
库内相对 path 依赖使用同版本引擎；库与消费者都不使用 `[patch.crates-io]`，保留一个
GPUI 身份。组件与渲染补丁均没有引入 Bezel 代码或其 fork。

| 后端 | 当前行为 |
| --- | --- |
| Linux WGPU（X11 / Wayland） | 实时场景采样与 GPU 高斯滤波；离屏数值测试、两种原生窗口 16 帧 smoke 均通过 |
| Apple Metal / Windows DirectX | 新绘制类型的兼容分支；尚无场景模糊实现，Surface 实心降级 |
| Web / 其他平台 | 未接通能力时实心降级，不承诺支持 |

未支持、参数无效、减少透明度或提高对比度时使用实心材质；不是只降低 alpha 假装模糊。

```rust,no_run
use beckon_kit::{components::Surface, theme::Theme, gpui::{App, px}};

fn frosted() -> Surface {
    Surface::new("inspector").frosted()
}

fn configure_material(cx: &mut App) {
    let mut theme = Theme::global(cx).as_ref().clone();
    theme.frosted.blur_sigma = px(18.0);
    theme.frosted.tint_opacity = 0.94;
    theme.install(cx); // 已打开、缓存及后续新建的默认窗口统一更新
}
```

按钮使用同一个方法，不需要额外容器；其他内容组合到 `Surface`，背景模糊、文字和交互保持独立层级。

```rust,no_run
use beckon_kit::components::{Button, Icon, IconName};

fn close() -> Button {
    Button::icon("close", "关闭", Icon::new(IconName::X))
        .tooltip(false) // 只去掉悬停提示，保留无障碍名称
        .frosted()
}
```

`Button` 与 `Surface` 的 `.frosted()` 只开启材质，统一读取主题 `Theme::frosted`，
不再在组件调用处选择档位。`FrostedMaterial` 明确使用 `blur_sigma`（模糊强度）与
`tint_opacity`（底色不透明度），没有与明暗主题混淆的 `Light` / `Dense` 名称。
默认 sigma 为 16 个逻辑像素，底色不透明度随亮 / 暗主题为 0.95 / 0.91；DPI 只在绘制边界转换一次。
显式窗口主题及 `ThemeScope` 仍是局部覆盖，不受全局安装影响。

`Button::frosted` 保留变体的语义文字颜色；`bg` 不覆盖材质，底色使用主题 `colors.surface`，
圆角、边框与阴影仍用 `Styled`。没有模糊后端、参数无效、减少透明度或提高对比度时
按实心降级，交互和焦点不受影响。sigma 合法区间为 `(0, 64]`，不透明度为 `[0, 1]`；
更低的不透明度需重新验证文字对比度。

低层 `Surface::backdrop(SceneBackdrop::new(...))` 仍可显式替换采样来源/强度；
普通组件无需装配后端，避免绕过统一主题。

### 宿主控件外观覆盖

默认 `Toggle` 仍为 34×20 指示器，默认分段控件仍使用 kit 的令牌。宿主设计值通过
现有 `Styled` 与下面的子部件扩展点覆盖，不需要重写交互：

```rust,no_run
use beckon_kit::{components::{Toggle, Segment, SegmentedControl}, gpui::{StyleRefinement, Styled, px, size, rgb}};

fn controls() {
    let _default = Toggle::new("default", "Enabled", false);
    let _custom = Toggle::new("custom", "Enabled", false)
        .track_size(size(px(40.0), px(24.0)))
        .track_colors(rgb(0xd9d9d9).into(), rgb(0x34c759).into(), rgb(0xffffff).into());
    let _tabs = SegmentedControl::new("choices", "Mode", [Segment::new("one", "One")], "one")
        .p(px(4.0)).rounded(px(12.0))
        .item_style(StyleRefinement::default().min_h(px(28.0)).rounded(px(8.0)))
        .selected_style(StyleRefinement::default().bg(rgb(0xffffff)));
}
```

`Tabs` 提供相同的 `item_style` / `selected_style`；disabled 文本在覆盖之后仍应用禁用颜色。

### 显式图片后端（可选用途）

`PreparedImage::new` 在绘制前使用 `image` 的滤波实现，在线性、预乘色空间模糊；
`BackdropImage` 根据 Local / Viewport / 明确区域采样。同一图像可由多个 Surface 共用，
GPU 图像身份跨帧稳定；paint 只采样与分层，不重新卷积。

```rust,no_run
use std::sync::Arc;
use beckon_kit::{components::Surface, material::{BackdropImage, ImagePlane, PreparedImage}};

fn image_surface(prepared: Arc<PreparedImage>) -> Surface {
    Surface::new("inspector")
        .frosted()
        .backdrop(BackdropImage::new(prepared, ImagePlane::Viewport))
}
```

此后端仅用于宿主明确拥有图片的情况，不是实时毛玻璃的前置条件，gallery 不使用它。
`Backdrop` 隔离材质来源；默认 Scene 和显式图片遵循相同的能力失败/实心降级契约。
补丁来源与维护说明见 [ENGINE_PATCH.md](ENGINE_PATCH.md)。

详细边界、取舍与后续组件的准入见 [ARCHITECTURE.md](ARCHITECTURE.md)。
