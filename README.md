# Personal API

A secure Rust API service for personal website with two endpoints:
- `GET /api/resume` - Returns a PDF resume
- `POST /contact` - Handles contact form submissions

## Features

- **Security**: Input validation, sanitization, and CORS protection
- **Email Integration**: Automatic email notifications via Brevo (Sendinblue) API
- **Docker**: Containerized with multi-stage build for optimal size
- **Validation**: Comprehensive form validation using the `validator` crate
- **Logging**: Structured logging with tracing
- **Health Check**: `/health` endpoint for monitoring
- **Environment Variables**: Secure configuration via environment variables

## API Endpoints

### GET /api/resume
Returns a PDF file of the resume.

**Response**: PDF file with `Content-Type: application/pdf`

### POST /contact
Accepts contact form data with the following JSON structure:

```json
{
  "email": "user@example.com",
  "firstName": "John",
  "lastName": "Doe", 
  "phoneNumber": "1234567890",
  "message": "Hello, I'm interested in your services."
}
```

**Response**:
```json
{
  "success": true,
  "message": "Thank you for your message. We'll get back to you soon!",
  "id": "unique-contact-id"
}
```

## Environment Setup

### Required Environment Variables

Create a `.env` file in the project root with the following variables:

```bash
# Brevo (Sendinblue) API Configuration
BREVO_API_KEY=your_brevo_api_key_here
BREVO_SENDER_EMAIL=your-email@example.com
BREVO_SENDER_NAME=Your Name

# Optional: Recipient email for contact form submissions
CONTACT_RECIPIENT_EMAIL=contact@example.com
```

### Getting Brevo API Key

1. Sign up for a [Brevo account](https://www.brevo.com/)
2. Go to **SMTP & API** → **API Keys**
3. Create a new API key with email sending permissions
4. Copy the API key to your `.env` file

**Note**: The `.env` file is automatically ignored by Git for security.

## Running the Service

### Using Docker (Recommended)

1. Build the Docker image:
```bash
docker build -t personal-api .
```

2. Run the container:
```bash
docker run -p 3030:3030 personal-api
```

### Using Docker Compose

Make sure your environment variables are set, then:

```bash
# Set environment variables (or use .env file)
export BREVO_API_KEY=your_api_key
export BREVO_SENDER_EMAIL=your@email.com
export BREVO_SENDER_NAME="Your Name"
export CONTACT_RECIPIENT_EMAIL=contact@example.com

docker-compose up --build
```

### Running Locally

1. Install Rust (if not already installed)
2. Run the service:
```bash
cargo run
```

The service will be available at `http://localhost:3030`

## Security Features

- Input validation and sanitization
- **Restricted CORS**: Only allows access from:
  - `http://localhost:3000`, `http://localhost:3001`, `http://localhost:8080`, `http://localhost:8081` (development)
  - `http://127.0.0.1:3000`, `http://127.0.0.1:3001`, `http://127.0.0.1:8080`, `http://127.0.0.1:8081` (development)
  - `https://michaelhenry.me`, `https://www.michaelhenry.me` (production)
- Non-root user in Docker container
- Request logging
- Error handling without information leakage

## Testing the API

### Test the resume endpoint:
```bash
curl -X GET http://localhost:3030/api/resume --output resume.pdf
```

### Test the contact endpoint:
```bash
curl -X POST http://localhost:3030/api/contact \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "firstName": "John",
    "lastName": "Doe",
    "phoneNumber": "1234567890",
    "message": "Test message"
  }'
```

### Health check:
```bash
curl http://localhost:3030/health
```
