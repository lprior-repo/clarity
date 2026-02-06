use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{self, filter::LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Create a new Axum router with CSS serving
    let app = Router::new()
        .route("/", get(root))
        .route("/assets/responsive.css", get(serve_css));

    // Bind to the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 4123));
    let listener = TcpListener::bind(addr).await?;

    println!("Server starting on http://{}", addr);

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Clarity - A modern fullstack Dioxus application with responsive design">
    <meta name="theme-color" content="#1976d2">
    <title>Clarity Application</title>
    <style>
        /* Inline critical CSS for above-the-fold content */
        *, *::before, *::after {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        html {
            font-size: 100%;
            scroll-behavior: smooth;
            -webkit-text-size-adjust: 100%;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            line-height: 1.5;
            color: #212121;
            background-color: #ffffff;
            min-height: 100vh;
            overflow-x: hidden;
        }

        .container {
            width: 100%;
            max-width: 1440px;
            margin-left: auto;
            margin-right: auto;
            padding-left: 0.75rem;
            padding-right: 0.75rem;
        }

        .hero {
            padding: 2.5rem 0.75rem;
            text-align: center;
        }

        @media (min-width: 768px) {
            .hero {
                padding: 2.5rem 2rem;
            }
        }

        h1 {
            font-size: clamp(2.25rem, 1.75rem + 2.5vw, 3rem);
            font-weight: 700;
            line-height: 1.25;
            margin-bottom: 1rem;
            overflow-wrap: break-word;
        }

        h2 {
            font-size: clamp(1.875rem, 1.5rem + 1.875vw, 2.5rem);
            font-weight: 700;
            line-height: 1.25;
            margin-bottom: 1rem;
            overflow-wrap: break-word;
        }

        p {
            font-size: clamp(1rem, 0.9rem + 0.5vw, 1.125rem);
            line-height: 1.625;
            margin-bottom: 1rem;
            overflow-wrap: break-word;
        }

        .skip-to-content {
            position: absolute;
            top: -40px;
            left: 0;
            background: #1976d2;
            color: white;
            padding: 0.5rem 1rem;
            text-decoration: none;
            z-index: 1070;
        }

        .skip-to-content:focus {
            top: 0;
        }

        @media (prefers-reduced-motion: reduce) {
            *, *::before, *::after {
                animation-duration: 0.01ms !important;
                transition-duration: 0.01ms !important;
                scroll-behavior: auto !important;
            }
        }
    </style>
    <link rel="stylesheet" href="/assets/responsive.css">
</head>
<body>
    <div class="container">
        <a href="#main-content" class="skip-to-content">Skip to main content</a>
        <header>
            <h1>Clarity Application</h1>
        </header>
        <main id="main-content">
            <section class="hero">
                <h2>Welcome to the Modern Fullstack Dioxus Application!</h2>
                <p>
                    This application demonstrates responsive design principles with:
                </p>
                <ul>
                    <li>Mobile-first approach</li>
                    <li>Fluid typography using clamp()</li>
                    <li>Flexible grid and flexbox layouts</li>
                    <li>Touch-friendly interface (44x44px minimum)</li>
                    <li>Dark mode support via prefers-color-scheme</li>
                    <li>Reduced motion support for accessibility</li>
                    <li>Semantic HTML for screen readers</li>
                </ul>
                <p>
                    Resize your browser window or view on different devices to see the responsive design in action!
                </p>
            </section>
        </main>
        <footer>
            <p>Built with Rust, Axum, and Dioxus</p>
        </footer>
    </div>

    <script>
        // Check for reduced motion preference
        const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
        document.documentElement.classList.toggle('reduced-motion', prefersReducedMotion.matches);

        // Optional: Dark mode toggle (can be enhanced later)
        const prefersDarkMode = window.matchMedia('(prefers-color-scheme: dark)');
        document.documentElement.classList.toggle('dark', prefersDarkMode.matches);
    </script>
</body>
</html>
    "#)
}

/// Serve the responsive CSS file with proper content type
async fn serve_css() -> impl IntoResponse {
    let css_path = "../clarity-client/assets/responsive.css";

    match tokio::fs::read_to_string(css_path).await {
        Ok(css_content) => {
            let headers = [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")];
            (headers, css_content).into_response()
        }
        Err(_) => {
            let error_msg = "CSS file not found";
            let headers = [(axum::http::header::CONTENT_TYPE, "text/plain")];
            (axum::http::StatusCode::NOT_FOUND, headers, error_msg).into_response()
        }
    }
}
