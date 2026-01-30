const CSS: &str = r#"
    #transparent-window {
        background-color: transparent;
    }
    #dock-container {
        background-color: rgba(30, 30, 30, 0.85);
        border-radius: 16px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        padding: 8px;
        margin-bottom: 10px;
        transition: transform 0.3s ease-in-out, opacity 0.3s ease-in-out;
    }
    #dock-container.dock-hidden {
        transform: translateY(100px);
        opacity: 0;
    }
    .dock-item {
        background: transparent;
        border-radius: 12px;
        padding: 4px;
        transition: all 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    }
    .dock-item:hover {
        background-color: rgba(255, 255, 255, 0.2);
        transform: scale(1.1);
    }
    .active-window {
        background-color: rgba(255, 255, 255, 0.15);
        box-shadow: inset 0 -2px 0 0 rgba(100, 200, 255, 0.8);
    }
    .dock-search {
        background-color: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 4px 8px;
        color: white;
        min-height: 32px;
    }
    .dock-search:focus {
        background-color: rgba(255, 255, 255, 0.15);
        border-color: rgba(100, 200, 255, 0.5);
    }
    .dock-search image {
        color: rgba(255, 255, 255, 0.6);
    }

    /* App Grid Window */
    #app-grid-window {
        background-color: rgba(25, 25, 25, 0.95);
        border-radius: 16px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .app-grid-container {
        background-color: transparent;
    }

    .app-grid-search {
        background-color: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 8px 12px;
        color: white;
        min-height: 36px;
        margin-bottom: 12px;
    }

    .app-grid-search:focus {
        background-color: rgba(255, 255, 255, 0.15);
        border-color: rgba(100, 200, 255, 0.5);
    }

    .app-grid-flow {
        background-color: transparent;
    }

    .app-grid-item {
        padding: 8px;
        border-radius: 12px;
        min-width: 80px;
        min-height: 100px;
    }

    .app-grid-button-item {
        background: transparent;
        border-radius: 12px;
        padding: 8px;
        transition: all 0.15s ease-out;
    }

    .app-grid-button-item:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }

    .app-grid-button-item:active {
        background-color: rgba(255, 255, 255, 0.15);
        transform: scale(0.95);
    }

    .app-grid-icon {
        margin-bottom: 4px;
    }

    .app-grid-label {
        color: rgba(255, 255, 255, 0.9);
        font-size: 11px;
        text-align: center;
    }

    .app-grid-button {
        padding: 4px;
    }
"#;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("No display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
