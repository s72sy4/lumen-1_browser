use crate::settings::Language;
use gtk::prelude::*;
use image::GenericImageView;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct Palette {
    bg: (u8, u8, u8),
    surface: (u8, u8, u8),
    fg: (u8, u8, u8),
    accent: (u8, u8, u8),
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    let m = |x: u8, y: u8| ((x as f32 * (1.0 - amount)) + (y as f32 * amount)).round() as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

fn clamp(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

fn dominant_color(path: &str) -> Option<(u8, u8, u8)> {
    let image = image::open(path).ok()?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return None;
    }
    let step = ((width as u64 * height as u64) / 20_000).max(1) as u32;
    let mut buckets: HashMap<(u8, u8, u8), (u32, u32)> = HashMap::new();
    for y in (0..height).step_by(step as usize) {
        for x in (0..width).step_by(step as usize) {
            let pixel = image.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let key = (r & 0xe0, g & 0xe0, b & 0xe0);
            let saturation = (r.max(g).max(b) as u32).saturating_sub(r.min(g).min(b) as u32);
            let entry = buckets.entry(key).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += saturation + 1;
        }
    }
    buckets
        .into_iter()
        .max_by_key(|(_, (count, saturation))| saturation * count)
        .map(|((r, g, b), _)| {
            (
                r.saturating_add(16),
                g.saturating_add(16),
                b.saturating_add(16),
            )
        })
}

fn palette(wallpaper: Option<&str>) -> Palette {
    let accent = wallpaper
        .and_then(dominant_color)
        .map(|(r, g, b)| {
            let max = r.max(g).max(b).max(70) as f32;
            let scale = 205.0 / max;
            (
                clamp(r as f32 * scale),
                clamp(g as f32 * scale),
                clamp(b as f32 * scale),
            )
        })
        .unwrap_or((137, 140, 251));

    Palette {
        bg: mix((18, 18, 22), accent, 0.05),
        surface: mix((29, 29, 35), accent, 0.10),
        fg: (236, 236, 241),
        accent,
    }
}

fn css(p: Palette) -> String {
    let rgb = |c: (u8, u8, u8)| format!("{}, {}, {}", c.0, c.1, c.2);
    let rgba = |c: (u8, u8, u8), a: f32| format!("rgba({}, {}, {}, {})", c.0, c.1, c.2, a);
    let fg = format!("rgb({})", rgb(p.fg));
    let muted = rgba(p.fg, 0.6);
    let accent = format!("rgb({})", rgb(p.accent));

    format!(
        r#"
window {{
    background-color: {bg_solid};
    color: {fg};
}}
#lumen-1_browser-window {{
    background-color: {bg_solid};
}}
#lumen-1_browser-header {{
    background-color: {surface_solid};
    border-bottom: 1px solid {border};
    padding: 2px 6px;
}}
#lumen-1_browser-entry {{
    background-color: {field};
    color: {fg};
    border: 1px solid {border};
    border-radius: 18px;
    padding: 8px 16px;
    font-size: 14px;
    transition: background-color 180ms ease, border-color 180ms ease, box-shadow 180ms ease;
}}
#lumen-1_browser-entry:focus {{
    border-color: {accent};
    background-color: {field_focus};
    box-shadow: 0 0 0 3px {accent_glow};
}}
#lumen-1_browser-button {{
    background-color: transparent;
    color: {fg};
    border: none;
    border-radius: 12px;
    padding: 7px 11px;
    min-width: 18px;
    transition: background-color 140ms ease, color 140ms ease;
}}
#lumen-1_browser-button:hover {{
    background-color: {hover};
    color: {accent};
}}
#lumen-1_browser-button:active {{
    background-color: {accent_glow};
}}
#lumen-1_browser-tab-list {{
    background-color: transparent;
}}
#lumen-1_browser-tab-list row {{
    background-color: transparent;
    color: {fg};
    border-radius: 10px;
    padding: 10px 12px;
    margin: 2px 6px;
    border-left: 2px solid transparent;
    transition: background-color 150ms ease, color 150ms ease, border-color 150ms ease;
}}
#lumen-1_browser-tab-list row:hover {{
    background-color: {hover};
}}
#lumen-1_browser-tab-list row:selected {{
    background-color: {accent_glow};
    color: {accent};
    border-left: 2px solid {accent};
}}
#lumen-1_browser-tab-close {{
    min-width: 0;
    padding: 0 5px;
    border-radius: 6px;
    font-size: 12px;
    background-color: transparent;
}}
#lumen-1_browser-tab-close:hover {{
    background-color: {accent_glow};
    color: {accent};
}}
#lumen-1_browser-tab-list row label {{
    font-size: 13px;
    font-weight: 500;
}}
#lumen-1_browser-panel {{
    background-color: {surface_solid};
    border-right: 1px solid {border};
    padding: 8px 4px;
}}
#lumen-1_browser-panel-revealer,
#lumen-1_browser-settings-revealer {{
    background-color: transparent;
}}
#lumen-1_browser-progress {{
    min-height: 2px;
    background-color: transparent;
}}
#lumen-1_browser-progress trough {{
    background-color: transparent;
    border: none;
    min-height: 2px;
}}
#lumen-1_browser-progress progress {{
    background-color: {accent};
    border-radius: 1px;
    min-height: 2px;
}}
#lumen-1_browser-private-badge {{
    color: {accent};
    background-color: {accent_glow};
    border-radius: 8px;
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 700;
}}
#lumen-1_browser-clock {{
    color: {muted};
    font-size: 13px;
    font-weight: 600;
    padding: 0 8px;
}}
#lumen-1_browser-settings {{
    background-color: {bg_solid};
}}
#settings-title {{
    font-size: 26px;
    font-weight: 700;
}}
.settings-label {{
    font-weight: 600;
    color: {muted};
}}
.settings-section {{
    font-size: 15px;
    font-weight: 700;
    color: {fg};
}}
.settings-hint {{
    color: {muted};
    font-size: 12px;
}}
.history-entry {{
    font-size: 12px;
}}
#lumen-1_browser-settings-sidebar {{
    background-color: {surface_solid};
    border-radius: 12px;
    padding: 6px;
}}
#lumen-1_browser-settings-sidebar button {{
    background-color: transparent;
    color: {fg};
    border-radius: 10px;
    padding: 8px 10px;
    transition: background-color 160ms ease, color 160ms ease;
}}
#lumen-1_browser-settings-sidebar button:hover {{
    background-color: {hover};
}}
#lumen-1_browser-settings-sidebar button:checked {{
    background-color: {accent_glow};
    color: {accent};
}}
#lumen-wallpaper-grid flowboxchild {{
    background-color: transparent;
    padding: 0;
    border: none;
}}
.wallpaper-card {{
    border: 2px solid transparent;
    border-radius: 12px;
    padding: 4px;
    background-color: {field};
    transition: border-color 180ms ease, background-color 180ms ease;
}}
.wallpaper-card:hover {{
    background-color: {hover};
    border-color: {accent};
}}
.wallpaper-card:checked {{
    border-color: {accent};
    background-color: {accent_glow};
}}
.wallpaper-card image {{
    border-radius: 8px;
}}
#lumen-1_browser-settings button {{
    border-radius: 10px;
    padding: 7px 13px;
    transition: background-color 150ms ease;
}}
#lumen-1_browser-credits-card {{
    background-color: {surface_solid};
    border: 1px solid {border};
    border-radius: 18px;
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.28);
}}
#credits-icon {{
    margin-bottom: 4px;
}}
#credits-name {{
    font-size: 24px;
    font-weight: 700;
    color: {fg};
}}
#credits-by {{
    font-size: 13px;
    color: {muted};
}}
#credits-stack {{
    font-size: 11px;
    color: {muted};
    letter-spacing: 1px;
    background-color: {accent_glow};
    border-radius: 10px;
    padding: 4px 10px;
}}
.credits-link {{
    color: {accent};
    border-radius: 10px;
    padding: 6px 10px;
    background-color: {field};
}}
#lumen-1_browser-settings entry {{
    border-radius: 10px;
    padding: 6px 10px;
}}
#lumen-1_browser-settings check {{
    border-radius: 4px;
}}
#lumen-1_browser-settings check:checked {{
    background-color: {accent};
    color: {bg_solid};
}}
"#,
        bg_solid = format!("rgb({})", rgb(p.bg)),
        surface_solid = format!("rgb({})", rgb(p.surface)),
        field = rgba(p.fg, 0.06),
        field_focus = rgba(p.accent, 0.10),
        border = rgba(p.fg, 0.12),
        hover = rgba(p.fg, 0.10),
        accent_glow = rgba(p.accent, 0.20),
        fg = fg,
        muted = muted,
        accent = accent,
    )
}

pub fn apply(wallpaper: Option<&str>) {
    if let Some(screen) = gdk::Screen::default() {
        let provider = gtk::CssProvider::new();
        if let Err(error) = provider.load_from_data(css(palette(wallpaper)).as_bytes()) {
            eprintln!("theme css error: {error}");
        }
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn wallpaper_data_uri(path: &str) -> Option<String> {
    let extension = std::path::Path::new(path)
        .extension()?
        .to_string_lossy()
        .to_ascii_lowercase();
    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return None,
    };
    let bytes = std::fs::read(path).ok()?;
    use base64::Engine;
    Some(format!(
        "data:{};base64,{}",
        mime,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

pub fn start_page_html(wallpaper: Option<&str>, transparent: bool, language: Language) -> String {
    let p = palette(wallpaper);
    let hex = |c: (u8, u8, u8)| format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2);
    let scrim = format!("rgba({}, {}, {}, 0.55)", p.bg.0, p.bg.1, p.bg.2);
    let glow = format!("rgba({}, {}, {}, 0.16)", p.accent.0, p.accent.1, p.accent.2);
    let solid = hex(p.bg);
    let background = if transparent {
        "none".to_string()
    } else {
        match wallpaper.and_then(wallpaper_data_uri) {
            Some(uri) => format!(
                "radial-gradient(1100px 760px at 12% 8%, {glow}, transparent 60%), linear-gradient(0deg, {scrim}, {scrim}), url(\"{uri}\")",
                glow = glow,
                scrim = scrim,
                uri = uri
            ),
            None => format!(
                "radial-gradient(1100px 760px at 12% 8%, {glow}, transparent 60%)",
                glow = glow
            ),
        }
    };
    let (days, months) = match language {
        Language::Deutsch => (
            r#"["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"]"#,
            r#"["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"]"#,
        ),
        Language::English => (
            r#"["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]"#,
            r#"["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]"#,
        ),
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
html, body {{ height: 100%; margin: 0; }}
body {{
    background-color: {solid};
    background-image: {background};
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    color: {fg};
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    padding: 42px 48px;
    box-sizing: border-box;
    overflow: hidden;
    animation: pageIn 520ms ease both;
}}
.widget {{
    text-align: left;
    animation: rise 720ms cubic-bezier(.2, .8, .2, 1) both;
}}
.clock {{
    font-size: 76px;
    font-weight: 700;
    letter-spacing: -1px;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    text-shadow: 0 10px 34px rgba(0, 0, 0, 0.22);
}}
.date {{
    margin-top: 12px;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 2px;
    text-transform: uppercase;
    color: {muted};
}}
@keyframes pageIn {{
    from {{ opacity: 0; }}
    to {{ opacity: 1; }}
}}
@keyframes rise {{
    from {{ opacity: 0; transform: translateY(16px); }}
    to {{ opacity: 1; transform: translateY(0); }}
}}
@keyframes fadeIn {{
    from {{ opacity: 0; }}
    to {{ opacity: 0.75; }}
}}
</style>
</head>
<body>
<div class="widget">
<div class="clock" id="clock"></div>
<div class="date" id="date"></div>
</div>
<script>
var days = {days};
var months = {months};
function tick() {{
    var now = new Date();
    var h = String(now.getHours()).padStart(2, "0");
    var m = String(now.getMinutes()).padStart(2, "0");
    document.getElementById("clock").textContent = h + ":" + m;
    document.getElementById("date").textContent = days[now.getDay()] + ", " + now.getDate() + ". " + months[now.getMonth()] + " " + now.getFullYear();
}}
tick();
setInterval(tick, 10000);
</script>
</body>
</html>"#,
        solid = solid,
        background = background,
        fg = hex(p.fg),
        muted = hex(mix(p.fg, p.bg, 0.42)),
        days = days,
        months = months,
    )
}
