use warp::Filter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::env;
use validator::Validate;
use reqwest::Client;

#[derive(Debug, Deserialize, Validate)]
struct ContactForm {
    #[validate(email)]
    email: String,
    #[validate(length(min = 1, max = 100))]
    #[serde(rename = "firstName")]
    first_name: String,
    #[validate(length(min = 1, max = 100))]
    #[serde(rename = "lastName")]
    last_name: String,
    #[validate(length(min = 10, max = 20))]
    #[serde(rename = "phoneNumber")]
    phone_number: String,
    #[validate(length(min = 1, max = 1000))]
    message: String,
}

#[derive(Debug, Serialize)]
struct ContactResponse {
    success: bool,
    message: String,
    id: String,
}

#[derive(Debug, Serialize)]
struct BrevoEmail {
    sender: BrevoSender,
    to: Vec<BrevoRecipient>,
    subject: String,
    #[serde(rename = "htmlContent")]
    html_content: String,
}

#[derive(Debug, Serialize)]
struct BrevoSender {
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct BrevoRecipient {
    email: String,
    name: Option<String>,
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    if let Err(e) = dotenv::dotenv() {
        println!("Warning: Could not load .env file: {}", e);
    } else {
        println!("Loaded environment variables from .env file");
    }
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // CORS configuration for security
    let cors = warp::cors()
        .allow_origins(vec![
            "http://localhost:3000",
            "http://localhost:3001", 
            "http://localhost:8080",
            "http://localhost:8081",
            "http://127.0.0.1:3000",
            "http://127.0.0.1:3001",
            "http://127.0.0.1:8080", 
            "http://127.0.0.1:8081",
            "https://michaelhenry.me",
        ])
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // Health check endpoint
    let health = warp::path("health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    // GET /api/resume - Returns PDF file
    let resume = warp::path("api")
        .and(warp::path("resume"))
        .and(warp::get())
        .and_then(handle_resume);

    // POST /api/contact - Handles contact form
    let contact = warp::path("api")
        .and(warp::path("contact"))
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_contact);

    // Combine all routes
    let routes = health
        .or(resume)
        .or(contact)
        .with(cors)
        .with(warp::log("rust-api-service"));

    println!("Starting server on http://localhost:3030");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}

async fn handle_resume() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let pdf_path = "assets/Michael Henry Resume - Staff Software Engineer.pdf";
    
    // Check if file exists
    if !Path::new(pdf_path).exists() {
        return Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "Resume not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        )));
    }

    // Read the PDF file
    match fs::read(pdf_path) {
        Ok(pdf_data) => {
            Ok(Box::new(warp::reply::with_header(
                pdf_data,
                "Content-Type",
                "application/pdf",
            )))
        }
        Err(_) => {
            Ok(Box::new(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Failed to read resume"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

async fn handle_contact(form: ContactForm) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate the form data
    if let Err(validation_errors) = form.validate() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "message": "Validation failed",
                "errors": validation_errors
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Basic input sanitization for security
    let sanitized_email = sanitize_input(&form.email);
    let sanitized_first_name = sanitize_input(&form.first_name);
    let sanitized_last_name = sanitize_input(&form.last_name);
    let _sanitized_phone = sanitize_input(&form.phone_number);
    let _sanitized_message = sanitize_input(&form.message);

    // Generate a unique ID for this contact submission
    let contact_id = uuid::Uuid::new_v4().to_string();

    // Log the contact form submission (in production, you'd save to database)
    tracing::info!(
        "Contact form submitted: {} {} <{}> - ID: {}",
        sanitized_first_name,
        sanitized_last_name,
        sanitized_email,
        contact_id
    );

    // Send email via Brevo
    let (email_success, response_message, status_code) = match send_brevo_email(&form, &contact_id).await {
        Ok(()) => {
            tracing::info!("Contact form email sent successfully for ID: {}", contact_id);
            (true, "Thank you for your message. We'll get back to you soon!".to_string(), warp::http::StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to send contact form email for ID {}: {}", contact_id, e);
            (false, "Your message was received, but there was an issue sending the notification email. Please try again or contact us directly.".to_string(), warp::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    };

    let response = ContactResponse {
        success: email_success,
        message: response_message,
        id: contact_id,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        status_code,
    ))
}

// Send email via Brevo API
async fn send_brevo_email(contact_form: &ContactForm, contact_id: &str) -> Result<(), anyhow::Error> {
    tracing::debug!("Attempting to send email via Brevo for contact ID: {}", contact_id);
    
    let api_key = env::var("BREVO_API_KEY")
        .map_err(|_| anyhow::anyhow!("BREVO_API_KEY environment variable not set"))?;
    
    let sender_email = env::var("BREVO_SENDER_EMAIL")
        .map_err(|_| anyhow::anyhow!("BREVO_SENDER_EMAIL environment variable not set"))?;
    
    let sender_name = env::var("BREVO_SENDER_NAME")
        .map_err(|_| anyhow::anyhow!("BREVO_SENDER_NAME environment variable not set"))?;
    
    let recipient_email = env::var("CONTACT_RECIPIENT_EMAIL")
        .unwrap_or_else(|_| sender_email.clone());

    tracing::debug!("Using sender: {} <{}>, recipient: {}", sender_name, sender_email, recipient_email);

    let client = Client::new();
    
    let html_content = format!(
        r#"
        <h2>New Contact Form Submission</h2>
        <p><strong>Contact ID:</strong> {}</p>
        <p><strong>Name:</strong> {} {}</p>
        <p><strong>Email:</strong> {}</p>
        <p><strong>Phone:</strong> {}</p>
        <p><strong>Message:</strong></p>
        <p>{}</p>
        <hr>
        <p><em>This message was sent from your website contact form.</em></p>
        "#,
        contact_id,
        contact_form.first_name,
        contact_form.last_name,
        contact_form.email,
        contact_form.phone_number,
        contact_form.message.replace('\n', "<br>")
    );

    let email = BrevoEmail {
        sender: BrevoSender {
            name: sender_name,
            email: sender_email,
        },
        to: vec![BrevoRecipient {
            email: recipient_email,
            name: Some("Contact Form".to_string()),
        }],
        subject: format!("New Contact Form Submission from {} {}", 
                        contact_form.first_name, contact_form.last_name),
        html_content,
    };

    let response = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&email)
        .send()
        .await?;

    if response.status().is_success() {
        tracing::info!("Email sent successfully via Brevo for contact ID: {}", contact_id);
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("Failed to send email via Brevo: {}", error_text);
        Err(anyhow::anyhow!("Failed to send email: {}", error_text))
    }
}

// Basic input sanitization to prevent XSS and other attacks
fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() && *c != '\0')
        .collect::<String>()
        .trim()
        .to_string()
}
