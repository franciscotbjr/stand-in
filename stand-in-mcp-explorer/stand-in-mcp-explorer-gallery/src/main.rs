mod args;
mod sections;
mod shell;

use args::Args;
use gpui::{AppContext, size};
use gpui_component::{Root, ThemeMode};
use shell::GalleryShell;
use stand_in_mcp_explorer_ds::assets::DsAssets;
use stand_in_mcp_explorer_ds::theme;

fn main() {
    let args = Args::from_env();

    let app = gpui_platform::application().with_assets(DsAssets);
    app.run(move |cx| {
        stand_in_mcp_explorer_ds::init(cx);
        let theme_mode = match args.mode.as_str() {
            "light" => ThemeMode::Light,
            _ => ThemeMode::Dark,
        };
        theme::apply_theme(theme_mode, cx);

        let bounds = gpui::Bounds::centered(None, size(gpui::px(1280.), gpui::px(820.)), cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| GalleryShell::new(args, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("failed to open gallery window");
        })
        .detach();
    });
}
