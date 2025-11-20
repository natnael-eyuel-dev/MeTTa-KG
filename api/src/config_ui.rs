use crate::cli::Cli;
use rocket::{get, post, routes, FromForm, State};
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, FromForm)]
struct ConfigForm {
    database_url: String,
    mettakg_frontend_url: Option<String>,
    mork_server_url: Option<String>,
    mettakg_api_url: Option<String>,
}

#[derive(Clone)]
pub struct ConfigPageData {
    pub database_url: Option<String>,
    pub mettakg_api_url: Option<String>,
    pub mork_server_url: Option<String>,
    pub mettakg_frontend_url: Option<String>,
}

struct RedirectUrl(Arc<Mutex<Option<String>>>);

#[get("/")]
fn config_page(data: &State<ConfigPageData>) -> RawHtml<String> {
    let db_value = data.database_url.as_deref().unwrap_or("");
    let db_readonly = if data.database_url.is_some() { "readonly" } else { "" };
    
    let api_value = data.mettakg_api_url.as_deref().unwrap_or("");
    let api_readonly = if data.mettakg_api_url.is_some() { "readonly" } else { "" };
    
    let mork_value = data.mork_server_url.as_deref().unwrap_or("");
    let mork_readonly = if data.mork_server_url.is_some() { "readonly" } else { "" };
    
    let frontend_value = data.mettakg_frontend_url.as_deref().unwrap_or("");
    let frontend_readonly = if data.mettakg_frontend_url.is_some() { "readonly" } else { "" };

    let html = format!(r#"
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
                            placeholder="postgres://user:password@localhost/dbname" 
                            value="{}" 
                            {} 
                            required
                        >
                        <div class="hint">PostgreSQL connection string (e.g., postgres://mettakg_user:abc123@localhost/mettakg_db)</div>
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
                    
                    <div class="form-group">
                        <label>Frontend URL</label>
                        <input 
                            type="text" 
                            name="mettakg_frontend_url" 
                            placeholder="http://127.0.0.1:3000" 
                            value="{}" 
                            {}
                        >
                        <div class="hint">URL for the frontend dev server if running separately (default: http://127.0.0.1:3000)</div>
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
        db_value, db_readonly, 
        if data.database_url.is_some() { r#"<div class="preset">Set via CLI argument</div>"# } else { "" },
        api_value, api_readonly,
        if data.mettakg_api_url.is_some() { r#"<div class="preset">Set via CLI argument</div>"# } else { "" },
        mork_value, mork_readonly,
        if data.mork_server_url.is_some() { r#"<div class="preset">Set via CLI argument</div>"# } else { "" },
        frontend_value, frontend_readonly,
        if data.mettakg_frontend_url.is_some() { r#"<div class="preset">Set via CLI argument</div>"# } else { "" }
    );

    RawHtml(html)
}

#[post("/submit", data = "<form>")]
async fn submit_config(
    form: Form<ConfigForm>, 
    tx: &State<mpsc::Sender<ConfigForm>>,
    redirect_url: &State<RedirectUrl>
) -> Redirect {
    let trimmed_form = ConfigForm {
        database_url: form.database_url.trim().to_string(),
        mettakg_frontend_url: form.mettakg_frontend_url.as_ref().map(|s| s.trim().to_string()),
        mork_server_url: form.mork_server_url.as_ref().map(|s| s.trim().to_string()),
        mettakg_api_url: form.mettakg_api_url.as_ref().map(|s| s.trim().to_string()),
    };
    
    let api_url = trimmed_form.mettakg_api_url.clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());
    
    *redirect_url.0.lock().await = Some(api_url.clone());
    
    let _ = tx.send(trimmed_form).await;
    
    Redirect::to("/waiting")
}

#[get("/waiting")]
fn waiting_page(redirect_url: &State<RedirectUrl>) -> RawHtml<String> {
    let url = redirect_url.0.try_lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());
    
    let html = format!(r#"
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
            // Poll the API URL to check if it's ready, then redirect
            const apiUrl = '{}';
            let attempts = 0;
            const maxAttempts = 30; // Try for 30 seconds
            
            function checkServer() {{
                attempts++;
                fetch(apiUrl + '/health')
                    .then(response => {{
                        if (response.ok) {{
                            window.location.href = apiUrl;
                        }} else if (attempts < maxAttempts) {{
                            setTimeout(checkServer, 1000);
                        }}
                    }})
                    .catch(() => {{
                        if (attempts < maxAttempts) {{
                            setTimeout(checkServer, 1000);
                        }} else {{
                            // After 30 seconds, just redirect anyway
                            window.location.href = apiUrl;
                        }}
                    }});
            }}
            
            // Start checking after 2 seconds
            setTimeout(checkServer, 2000);
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
    "#, url, url, url);
    
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
            port: 8080,
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
        mettakg_frontend_url: config_form.mettakg_frontend_url.filter(|s| !s.is_empty()),
        mork_server_url: config_form.mork_server_url.filter(|s| !s.is_empty()),
        mettakg_api_url: config_form.mettakg_api_url.filter(|s| !s.is_empty()),
    }
}