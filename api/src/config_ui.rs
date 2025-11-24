use crate::cli::Cli;
use diesel::Connection;
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{get, post, routes, FromForm, State};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use url::Url;

#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection as DbConnection;

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use diesel::pg::PgConnection as DbConnection;

#[derive(Clone, FromForm)]
struct ConfigForm {
    database_url: String,
    mork_server_url: Option<String>,
    mettakg_api_url: Option<String>,
}

#[derive(Clone)]
pub struct ConfigPageData {
    pub database_url: Option<String>,
    pub mettakg_api_url: Option<String>,
    pub mork_server_url: Option<String>,
    pub error: Option<String>,
}

struct RedirectUrl(Arc<Mutex<Option<String>>>);

fn render_config_page(data: &ConfigPageData) -> RawHtml<String> {
    let db_value = data.database_url.as_deref().unwrap_or("");

    let is_preset = data.error.is_none() && data.database_url.is_some();

    let db_readonly = if is_preset { "readonly" } else { "" };
    let api_value = data.mettakg_api_url.as_deref().unwrap_or("");
    let api_readonly = if is_preset && data.mettakg_api_url.is_some() {
        "readonly"
    } else {
        ""
    };

    let mork_value = data.mork_server_url.as_deref().unwrap_or("");
    let mork_readonly = if is_preset && data.mork_server_url.is_some() {
        "readonly"
    } else {
        ""
    };

    #[cfg(feature = "sqlite")]
    let (db_placeholder, db_hint) = (
        "metta_kg.db or /path/to/database.db",
        "SQLite database file path (e.g., metta_kg.db)",
    );

    #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
    let (db_placeholder, db_hint) = (
        "postgres://user:password@localhost/dbname",
        "PostgreSQL connection string (e.g., postgres://mettakg_user:abc123@localhost/mettakg_db)",
    );

    let error_html = if let Some(err) = &data.error {
        format!(
            r#"
            <div style="background: hsla(0, 70%, 50%, 0.15); border: 1px solid hsla(0, 70%, 50%, 0.3); color: hsl(0, 80%, 70%); padding: 16px; border-radius: 0.5rem; margin-bottom: 24px; font-size: 0.9rem;">
                <strong>Configuration Error:</strong><br>
                {}
            </div>
            "#,
            err
        )
    } else {
        String::new()
    };

    let html = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MeTTa-KG Configuration</title>
        <link href="https://fonts.googleapis.com/css2?family=Geist+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
        <style>
            * {{
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }}
            
            body {{
                font-family: 'Geist Mono', 'Monaco', 'Menlo', monospace;
                background: hsl(0, 0%, 0%);
                color: hsl(0, 0%, 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }}
            
            .container {{
                max-width: 700px;
                width: 100%;
            }}
            
            .header {{
                text-align: center;
                margin-bottom: 40px;
            }}
            
            .logo {{
                font-size: 2.5rem;
                font-weight: 700;
                color: hsl(142, 71%, 45%);
                margin-bottom: 8px;
                letter-spacing: -0.02em;
            }}
            
            .subtitle {{
                color: hsl(0, 0%, 45%);
                font-size: 0.95rem;
            }}
            
            .card {{
                background: hsl(0, 0%, 10%);
                border: 1px solid hsl(0, 0%, 25%);
                border-radius: 0.5rem;
                padding: 32px;
                box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
            }}
            
            .form-group {{
                margin-bottom: 24px;
            }}
            
            .form-group:last-of-type {{
                margin-bottom: 0;
            }}
            
            label {{
                display: block;
                font-weight: 600;
                font-size: 0.875rem;
                margin-bottom: 8px;
                color: hsl(0, 0%, 100%);
            }}
            
            .required {{
                color: hsl(0, 84%, 60%);
                margin-left: 2px;
            }}
            
            input {{
                width: 100%;
                padding: 10px 14px;
                background: hsl(0, 0%, 0%);
                border: 1px solid hsl(0, 0%, 25%);
                border-radius: 0.5rem;
                color: hsl(0, 0%, 100%);
                font-family: 'Geist Mono', monospace;
                font-size: 0.875rem;
                transition: all 0.2s ease;
                outline: none;
            }}
            
            input:focus {{
                border-color: hsl(142, 71%, 45%);
                box-shadow: 0 0 0 3px hsla(142, 71%, 45%, 0.15);
            }}
            
            input[readonly] {{
                background: hsl(0, 0%, 15%);
                color: hsl(0, 0%, 65%);
                cursor: not-allowed;
                opacity: 0.8;
            }}
            
            input[readonly]:focus {{
                border-color: hsl(0, 0%, 25%);
                box-shadow: none;
            }}
            
            input::placeholder {{
                color: hsl(0, 0%, 35%);
            }}
            
            .hint {{
                font-size: 0.8rem;
                color: hsl(0, 0%, 45%);
                margin-top: 6px;
                line-height: 1.4;
            }}
            
            .preset {{
                display: inline-flex;
                align-items: center;
                gap: 6px;
                margin-top: 8px;
                padding: 4px 12px;
                background: hsl(142, 60%, 90%);
                color: hsl(142, 70%, 25%);
                border-radius: 0.5rem;
                font-size: 0.8rem;
                font-weight: 600;
            }}
            
            .preset::before {{
                content: "✓";
                font-weight: bold;
            }}
            
            button {{
                width: 100%;
                padding: 12px 24px;
                background: hsl(142, 71%, 45%);
                color: hsl(0, 0%, 0%);
                border: none;
                border-radius: 0.5rem;
                font-weight: 600;
                font-size: 0.95rem;
                font-family: 'Geist Mono', monospace;
                cursor: pointer;
                transition: all 0.2s ease;
                margin-top: 32px;
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 8px;
            }}
            
            button:hover {{
                background: hsl(142, 71%, 50%);
                transform: translateY(-1px);
                box-shadow: 0 4px 12px hsla(142, 71%, 45%, 0.4);
            }}
            
            button:active {{
                transform: translateY(0);
            }}
            
            .divider {{
                height: 1px;
                background: hsl(0, 0%, 25%);
                margin: 28px 0;
            }}
            
            .section-title {{
                font-size: 0.75rem;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: hsl(0, 0%, 45%);
                margin-bottom: 16px;
            }}
            
            @media (max-width: 640px) {{
                .card {{
                    padding: 24px;
                }}
                
                .logo {{
                    font-size: 2rem;
                }}
            }}
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <div class="logo">MeTTa-KG</div>
                <div class="subtitle">Configure your knowledge graph server</div>
            </div>
            
            {}
            
            <div class="card">
                <form action="/submit" method="post">
                    <div class="section-title">Database Configuration</div>
                    <div class="form-group">
                        <label>
                            Database URL <span class="required">*</span>
                        </label>
                        <input 
                            type="text" 
                            name="database_url" 
                            placeholder="{}" 
                            value="{}" 
                            {} 
                            required
                        >
                        <div class="hint">{}</div>
                        {}
                    </div>
                    
                    <div class="divider"></div>
                    
                    <div class="section-title">Server Configuration</div>
                    <div class="form-group">
                        <label>API URL</label>
                        <input 
                            type="text" 
                            name="mettakg_api_url" 
                            placeholder="http://127.0.0.1:8000" 
                            value="{}" 
                            {}
                        >
                        <div class="hint">URL where the MeTTa-KG API server will listen (default: http://127.0.0.1:8000)</div>
                        {}
                    </div>
                    
                    <div class="form-group">
                        <label>Mork Server URL</label>
                        <input 
                            type="text" 
                            name="mork_server_url" 
                            placeholder="http://127.0.0.1:8001" 
                            value="{}" 
                            {}
                        >
                        <div class="hint">URL for the embedded Mork MeTTa interpreter server (default: http://127.0.0.1:8001)</div>
                        {}
                    </div>
                    
                    <button type="submit">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M5 12h14"/>
                            <path d="m12 5 7 7-7 7"/>
                        </svg>
                        Start Server
                    </button>
                </form>
            </div>
        </div>
    </body>
    </html>
    "#,
        error_html,
        db_placeholder,
        db_value,
        db_readonly,
        db_hint,
        if is_preset && data.database_url.is_some() {
            r#"<div class="preset">Set via CLI argument</div>"#
        } else {
            ""
        },
        api_value,
        api_readonly,
        if is_preset && data.mettakg_api_url.is_some() {
            r#"<div class="preset">Set via CLI argument</div>"#
        } else {
            ""
        },
        mork_value,
        mork_readonly,
        if is_preset && data.mork_server_url.is_some() {
            r#"<div class="preset">Set via CLI argument</div>"#
        } else {
            ""
        }
    );

    RawHtml(html)
}

#[get("/")]
fn config_page(data: &State<ConfigPageData>) -> RawHtml<String> {
    render_config_page(data)
}

fn test_connection(url: &str) -> Result<(), String> {
    DbConnection::establish(url)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[post("/submit", data = "<form>")]
async fn submit_config(
    form: Form<ConfigForm>,
    tx: &State<mpsc::Sender<ConfigForm>>,
    redirect_url: &State<RedirectUrl>,
) -> Result<Redirect, RawHtml<String>> {
    let trimmed_form = ConfigForm {
        database_url: form.database_url.trim().to_string(),
        mork_server_url: form.mork_server_url.as_ref().map(|s| s.trim().to_string()),
        mettakg_api_url: form.mettakg_api_url.as_ref().map(|s| s.trim().to_string()),
    };

    let render_error = |msg: String| {
        let data = ConfigPageData {
            database_url: Some(trimmed_form.database_url.clone()),
            mettakg_api_url: trimmed_form.mettakg_api_url.clone(),
            mork_server_url: trimmed_form.mork_server_url.clone(),
            error: Some(msg),
        };
        render_config_page(&data)
    };

    let api_url_str = trimmed_form
        .mettakg_api_url
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    let mork_url_str = trimmed_form
        .mork_server_url
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8001".to_string());

    let api_url =
        Url::parse(&api_url_str).map_err(|_| render_error("Invalid API URL format".to_string()))?;
    let mork_url = Url::parse(&mork_url_str)
        .map_err(|_| render_error("Invalid Mork Server URL format".to_string()))?;

    let api_port = api_url.port().unwrap_or(8000);
    let mork_port = mork_url.port().unwrap_or(8001);

    if api_port == mork_port {
        return Err(render_error(format!(
            "Port conflict: Both API and Mork server are configured to use port {}. Please choose different ports.",
            api_port
        )));
    }

    fn is_port_available(host: &str, port: u16) -> bool {
        TcpListener::bind((host, port)).is_ok()
    }

    let mork_host = mork_url.host_str().unwrap_or("127.0.0.1");
    if !is_port_available(mork_host, mork_port) {
        return Err(render_error(format!(
            "Port {} on {} is already in use. Please choose a different port for the Mork server.",
            mork_port, mork_host
        )));
    }

    if let Err(e) = test_connection(&trimmed_form.database_url) {
        return Err(render_error(format!(
            "Failed to connect to database: {}",
            e
        )));
    }

    let api_url = trimmed_form
        .mettakg_api_url
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    *redirect_url.0.lock().await = Some(api_url.clone());

    let _ = tx.send(trimmed_form).await;

    Ok(Redirect::to("/waiting"))
}

#[get("/waiting")]
fn waiting_page(redirect_url: &State<RedirectUrl>) -> RawHtml<String> {
    let url = redirect_url
        .0
        .try_lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    let html = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Starting MeTTa-KG...</title>
        <link href="https://fonts.googleapis.com/css2?family=Geist+Mono:wght@400;600&display=swap" rel="stylesheet">
        <style>
            * {{
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }}
            
            body {{
                font-family: 'Geist Mono', monospace;
                background: hsl(0, 0%, 0%);
                color: hsl(0, 0%, 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }}
            
            .container {{
                text-align: center;
                max-width: 500px;
            }}
            
            .spinner {{
                width: 64px;
                height: 64px;
                border: 4px solid hsl(0, 0%, 25%);
                border-top-color: hsl(142, 71%, 45%);
                border-radius: 50%;
                animation: spin 1s linear infinite;
                margin: 0 auto 24px;
            }}
            
            @keyframes spin {{
                to {{ transform: rotate(360deg); }}
            }}
            
            h1 {{
                font-size: 1.5rem;
                font-weight: 600;
                margin-bottom: 12px;
                color: hsl(142, 71%, 45%);
            }}
            
            p {{
                color: hsl(0, 0%, 45%);
                line-height: 1.6;
            }}
            
            .url {{
                color: hsl(142, 71%, 45%);
                font-weight: 600;
            }}
        </style>
        <script>
            const targetUrl = '{}';
            const probeUrl = targetUrl.replace(/\/$/, "") + "/api/tokens";
            
            function checkServer() {{
                fetch(probeUrl)
                    .then(response => {{
                        // If we get ANY response (even 401 Unauthorized), the server is running
                        if (response.status >= 200 && response.status < 600) {{
                            window.location.href = targetUrl;
                        }} else {{
                            setTimeout(checkServer, 1000);
                        }}
                    }})
                    .catch(() => {{
                        setTimeout(checkServer, 1000);
                    }});
            }}
            
            // Start checking after 3 seconds to give the server time to restart
            setTimeout(checkServer, 3000);
        </script>
    </head>
    <body>
        <div class="container">
            <div class="spinner"></div>
            <h1>Configuration Submitted!</h1>
            <p>Starting MeTTa-KG server...</p>
            <p style="margin-top: 8px;">Redirecting to <span class="url">{}</span></p>
            <p style="margin-top: 16px; font-size: 0.85rem;">If not redirected automatically, <a href="{}" style="color: hsl(142, 71%, 45%);">click here</a>.</p>
        </div>
    </body>
    </html>
    "#,
        url, url, url
    );

    RawHtml(html)
}

pub async fn launch_config_server(preset_config: ConfigPageData) -> Cli {
    let (tx, mut rx) = mpsc::channel::<ConfigForm>(1);
    let redirect_url = RedirectUrl(Arc::new(Mutex::new(None)));

    let config_rocket = rocket::build()
        .manage(tx)
        .manage(preset_config)
        .manage(redirect_url)
        .mount("/", routes![config_page, submit_config, waiting_page])
        .configure(rocket::Config {
            address: "127.0.0.1".parse().unwrap(),
            port: 8000,
            ..Default::default()
        });

    let handle = tokio::spawn(async move {
        let _ = config_rocket.launch().await;
    });

    let config_form: ConfigForm = rx.recv().await.expect("Failed to receive config");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    handle.abort();

    Cli {
        database_url: Some(config_form.database_url),
        mork_server_url: config_form.mork_server_url.filter(|s| !s.is_empty()),
        mettakg_api_url: config_form.mettakg_api_url.filter(|s| !s.is_empty()),
    }
}
