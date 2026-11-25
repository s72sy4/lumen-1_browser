use webkit2gtk::{
    UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserStyleLevel,
    UserStyleSheet,
};

const FILTER_CSS: &str = r#"
div[id^="ad-"], div[class*="advertisement"], div[class*="ad-banner"],
iframe[src*="ads"], iframe[title*="advertisement"], ins.adsbygoogle,
div[id*="google_ads"], div[class*="sponsor"], aside[aria-label*="Anzeige" i] {
    display: none !important;
}
.ytp-ad-overlay-container, .ytp-ad-text, .ytd-player-legacy-desktop-watch-ads-renderer,
ytd-promoted-sparkles-web-renderer, ytd-display-ad-renderer, ytd-ad-slot-renderer,
ytd-in-feed-ad-layout-renderer, ytd-banner-promo-renderer, ytd-statement-banner-renderer,
ytd-companion-slot-renderer, #player-ads, #masthead-ad, #related ytd-ad-slot-renderer,
ytd-rich-item-renderer:has(ytd-ad-slot-renderer) {
    display: none !important;
}
"#;

const YOUTUBE_CLEANER_JS: &str = r#"
(function () {
    if (window.__lumenYtCleaner) return;
    window.__lumenYtCleaner = true;

    var style = document.createElement('style');
    style.textContent = [
        '.ytp-ad-overlay-container, .ytp-ad-module, .ytp-ad-player-overlay-layout,',
        '.ytp-ad-progress-list, .ytp-suggested-action, .ytp-featured-product,',
        'ytd-display-ad-renderer, ytd-ad-slot-renderer, ytd-in-feed-ad-layout-renderer,',
        'ytd-promoted-sparkles-web-renderer, ytd-banner-promo-renderer,',
        'ytd-statement-banner-renderer, ytd-companion-slot-renderer,',
        '#player-ads, #masthead-ad, #related ytd-ad-slot-renderer,',
        'ytd-rich-item-renderer:has(ytd-ad-slot-renderer) {',
        '  display: none !important;',
        '}'
    ].join('\n');
    (document.head || document.documentElement).appendChild(style);

    function clean() {
        var player = document.querySelector('#movie_player');
        var adShowing = player && player.classList.contains('ad-showing');
        if (!adShowing) return;

        var video = document.querySelector('video');
        if (video) {
            video.muted = true;
            try {
                if (isFinite(video.duration) && video.duration > 0) {
                    video.currentTime = video.duration;
                }
            } catch (e) {}
        }

        var skip = document.querySelector(
            '.ytp-ad-skip-button, .ytp-skip-ad-button, .ytp-ad-skip-button-modern, ' +
            '.ytp-ad-skip-button-slot button, button.ytp-ad-skip-button'
        );
        if (skip) skip.click();

        var close = document.querySelector('.ytp-ad-overlay-close-button');
        if (close) close.click();
    }

    clean();
    setInterval(clean, 350);

    var observer = new MutationObserver(clean);
    observer.observe(document.documentElement, { childList: true, subtree: true });
})();
"#;

pub fn user_content_manager(adblock_enabled: bool) -> UserContentManager {
    let manager = UserContentManager::new();
    if adblock_enabled {
        let sheet = UserStyleSheet::new(
            FILTER_CSS,
            UserContentInjectedFrames::AllFrames,
            UserStyleLevel::Author,
            &[],
            &[],
        );
        manager.add_style_sheet(&sheet);
    }
    manager
}

pub fn youtube_cleaner_js() -> &'static str {
    YOUTUBE_CLEANER_JS
}
