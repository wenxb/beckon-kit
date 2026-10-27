## 独立组件库边界

- 本 crate 是标准、宿主无关的组件库；通用组件、材质、展示、测试与文档放在这里，业务接入与业务 UI 留在宿主。
- 独立仓库为 https://github.com/wenxb/beckon-kit；根目录是普通 Cargo package `beckon-kit` 0.1.0、edition 2024、Apache-2.0，不是宿主 workspace 成员。八个同版本 vendor 包通过相对 path 依赖连接；库和消费者都不能依赖 `[patch.crates-io]` 接线。
- 禁止依赖 `gpui-kit`、`gpui-base`、`gpui-component`、`bezel` 或任何 Beckon 业务 crate。
- 组件及视觉设计原创；可以读底层 GPUI API 和参考库的公开行为文档，不复制其他组件库实现。
- 一个 GPUI 引擎身份；宿主通过 `beckon_kit::gpui` 使用类型、trait 和宏，通过 opt-in `platform` feature 使用 `beckon_kit::platform`，不直接声明 GPUI 或平台包依赖。升级 `gpui-pre` 与平台包必须同版本并重新验证，不能静默引入第二套引擎。
- 简单组件采用受控值 + 请求回调；内部状态只保存焦点、按压、拖动等交互状态。
- 令牌在 `theme/`，行为在 `interaction/` 或 `text/`，材质后端在 `material/`，组件在 `components/`。
- 不把布局、文本造型、Unicode 分词、图片滤波重新实现成第二套基础设施。
- 每个文件不超过 600 行；注释说清约束，不引用外部文档代替解释。

## API 设计

- 以调用端易用性（API ergonomics）验收：先写最小用法和常见定制示例，再设计内部实现。
- 合理默认、流式配置、复杂能力按需展开；不要让调用者手动装配内部层次或平台分支。
- 复用 GPUI 类型、实体、句柄和回调惯例；避免同义入口、隐藏失效和大型配置泛型链。
- 不支持的能力明确报告；便捷接口不能以默默忽略参数换取表面简洁。

## 宿主接入与样式定制

- 缺少通用功能或必要的样式扩展点时补齐 kit，不能以此为由省略宿主需求、降低效果或在宿主另写一套同类控件。
- 宿主专属的配色、尺寸、布局与组合留在宿主；不能据此新增业务命名的组件、变体、令牌或修改库的默认设计。
- 优先增强可复用的定制能力：主题令牌、GPUI `Styled` / `StyleRefinement`、内容组合与确有通用语义的变体；不要为单一页面堆布尔开关。
- 同类控件共享 kit 的行为、语义与设计规则；自定义外观不能破坏稳定身份、受控状态、键盘、焦点、禁用、IME 或减少动态效果。
- 新增定制入口必须给出默认与覆盖示例，并测试覆盖确实生效、默认不变且交互契约不退化；未实现的覆盖不能静默忽略。

## 验证

- 新交互必须增加 GPUI `test-support` 输入测试；不能用直接调用回调代替点击/按键。
- 所有交互控件有稳定身份、语义标签、禁用处理和键盘路径。
- 材质必须明确来源；无后端、区域不可用、减少透明度时用实心降级。
- 不在 `render` / `paint` 中卷积、读屏或同步等待；图片准备与 GPU 身份跨帧复用。
- Linux 验证用 `scripts/run_limited.py -- <command>`：默认 4 个 Cargo job、最多 8 CPU、8 GiB 硬上限、禁用 swap；不得并发重构建。
- 在本仓根运行 `just check`、`just test`、`just lint`、`just fmt -- --check` 和 `just boundaries`；这些命令均经过资源限制脚本，不得并发运行。
- 定向测试用 `just test --test <name>`；测试默认启用全部 feature、单线程执行，定向测试优先，小修不要反复重跑全套。
- `just boundaries` 同时运行 `scripts/check_boundaries.py` 和 `python3 -m unittest discover -s scripts -p 'test_*.py'`，仅检查独立库；宿主依赖边界由宿主仓库维护。
- 未经用户重新允许，不读取或展示图片；视觉核查使用布局/绘制断言和离屏渲染测试。
