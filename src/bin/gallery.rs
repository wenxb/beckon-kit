use beckon_kit::{
    WindowConfig, WindowRootHandleExt,
    gallery::{Gallery, Page},
    gpui::{AppContext, Context, Window, WindowOptions, px, size},
    init,
};

fn main() {
    let mut materials = false;
    let mut smoke_test = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--materials" => materials = true,
            "--smoke-test" => {
                materials = true;
                smoke_test = true;
            }
            "--help" | "-h" => {
                println!(
                    "Beckon Kit 组件工作台\n\n\
                     用法：beckon-kit-gallery [--materials] [--smoke-test]\n\n\
                     选项：\n  --materials    直接打开毛玻璃材质页\n  \
                     --smoke-test   更新背景 16 帧后退出\n  -h, --help     显示帮助"
                );
                return;
            }
            _ => {
                eprintln!("未知参数：{argument}");
                std::process::exit(2);
            }
        }
    }
    gpui_platform::application().run(move |cx| {
        init(cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let handle = WindowConfig::new("Beckon Kit · 组件工作台")
            .size(size(px(980.0), px(680.0)))
            .min_size(size(px(420.0), px(380.0)))
            .options(WindowOptions {
                app_id: Some("com.youngwen.beckon.kit".into()),
                ..Default::default()
            })
            .open(cx, |_window, cx| {
                cx.new(|cx| {
                    let mut gallery = Gallery::new(cx);
                    if materials {
                        gallery.show_page(Page::Materials, cx);
                    }
                    gallery
                })
            })
            .expect("打开组件工作台失败");
        if smoke_test {
            handle
                .update_content(cx, |gallery, window, cx| {
                    smoke_frame(gallery, window, cx, 0)
                })
                .expect("安排展示自检失败");
        }
        cx.activate(true);
    });
}

fn smoke_frame(
    gallery: &mut Gallery,
    window: &mut Window,
    cx: &mut Context<Gallery>,
    frame: usize,
) {
    gallery.advance_scene(cx);
    gallery.set_scene_shift(if frame.is_multiple_of(2) { 0.0 } else { 1.0 }, cx);
    cx.on_next_frame(window, move |gallery, window, cx| {
        if frame == 15 {
            eprintln!(
                "beckon-kit-gallery：原生帧循环已完成；实时背景模糊={}；GPU={:?}；装饰={:?}；协商完成={}；窗口尺寸={:?}；client_inset={:?}",
                if window.is_backdrop_blur_supported() {
                    "支持"
                } else {
                    "不支持"
                },
                window.gpu_specs(),
                window.window_decorations(),
                window.window_decorations_negotiated(),
                window.viewport_size(),
                window.client_inset(),
            );
            cx.quit();
        } else {
            smoke_frame(gallery, window, cx, frame + 1);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use beckon_kit::gpui::TestAppContext;

    #[gpui::test]
    fn smoke_loop_updates_the_entity_without_reentering_the_window(cx: &mut TestAppContext) {
        cx.update(init);
        let handle = cx.open_window(size(px(980.0), px(680.0)), |_, cx| Gallery::new(cx));
        handle
            .update(cx, |gallery, window, cx| {
                smoke_frame(gallery, window, cx, 0)
            })
            .unwrap();

        for _ in 0..15 {
            cx.update_window(handle.into(), |_, window, cx| {
                assert_eq!(window.simulate_next_frame(cx), 1);
            })
            .unwrap();
        }
        let gallery = handle.root(cx).unwrap();
        assert_eq!(
            gallery.read_with(cx, |gallery, _| gallery.scene_revision()),
            16
        );
        assert_eq!(
            gallery.read_with(cx, |gallery, _| gallery.scene_shift()),
            1.0
        );
    }
}
