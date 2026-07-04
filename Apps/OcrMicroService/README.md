# OcrMicroService

A small OCR microservice written in Rust.

The service exposes an HTTP endpoint that accepts image uploads using `multipart/form-data`, runs Tesseract OCR, and returns the recognized text or hOCR output as JSON.

## Features

- HTTP OCR endpoint
- Multipart file upload
- Supports plain text OCR output
- Supports hOCR output
- Configurable Tesseract command
- Can use either native Tesseract or Docker-based Tesseract
- Includes a Docker-based Linux build workflow
- Simple deployment using `scp`

## Requirements

### Local development

To build and run locally, you need:

- Rust
- Cargo
- Tesseract OCR, if using the native Tesseract config

### Docker-based Linux build

To build a Debian-compatible Linux executable from any machine with Docker:

- Docker

The build container uses Debian Bookworm, matching Debian 12.

### Server runtime

For the recommended native deployment on Debian 12, the server needs:

- Debian 12
- Tesseract OCR
- The required Tesseract language packages

Install the default English Tesseract package with:

```bash
sudo apt update
sudo apt install -y tesseract-ocr tesseract-ocr-eng
```

Install additional language packages as needed, for example:

```bash
sudo apt install -y tesseract-ocr-deu tesseract-ocr-fra
```

## Configuration

The service expects a TOML configuration file describing how Tesseract should be executed.

If no `--config` argument is provided, the service looks for:

```text
tesseract.toml
```

in the same directory as the executable.

You can also pass the config path explicitly:

```bash
./OcrMicroService --config ./config_examples/tesseract_native.toml
```

### Native Tesseract configuration

Use this when Tesseract is installed directly on the host:

```toml
[tesseract]
command = "tesseract"
args = [
    "{image_path}",
    "stdout",
    "-l",
    "{language}",
    "{config_path}",
]
```

### Docker-based Tesseract configuration

Use this when you want the service to call Tesseract through Docker:

```toml
[tesseract]
command = "docker"
args = [
    "run",
    "--rm",
    "-v",
    "{config_path}:/tmp/config",
    "-v",
    "{image_path}:/tmp/img.jpg",
    "jitesoft/tesseract-ocr",
    "/tmp/img.jpg",
    "stdout",
    "-l",
    "{language}",
    "/tmp/config",
]
```

Note that this requires Docker to be available wherever the service is running.

## Build

### Build locally with Cargo

```bash
cargo build --release
```

The executable will be available at:

```text
target/release/OcrMicroService
```

### Build a Linux executable using Docker

This is the recommended build method if you want to produce an executable suitable for a Debian 12 server.

Run:

```bash
chmod +x scripts/build-linux.sh
./scripts/build-linux.sh
```

The built executable will be copied to:

```text
dist/OcrMicroService
```

## Run locally

Using the native Tesseract config:

```bash
./target/release/OcrMicroService --config ./config_examples/tesseract_native.toml
```

Or, if you built using the Docker build script:

```bash
./dist/OcrMicroService --config ./config_examples/tesseract_native.toml
```

The service listens on:

```text
http://127.0.0.1:3000
```

## API

### POST `/ocr/tesseract`

Runs OCR on one or more uploaded image files.

The request must use `multipart/form-data`.

Supported form fields:

| Field | Required | Description |
| --- | --- | --- |
| `files` | Yes | One or more image files to process |
| `language` | No | Tesseract language code. Defaults to `eng` |
| `config` | No | Output mode. Supported values: `text` and `hocr`. Defaults to `text` |

Example response:

```json
{
  "results": [
    {
      "upload_name": "example.png",
      "temp_path": "/tmp/example.png",
      "hocr": null,
      "text": "Recognized text here\n",
      "error": null
    }
  ]
}
```

## Testing with curl

Start the service first:

```bash
./dist/OcrMicroService --config ./config_examples/tesseract_native.toml
```

In another terminal, send an image file to the OCR endpoint.

### Text output

```bash
curl -X POST http://127.0.0.1:3000/ocr/tesseract \
  -F "language=eng" \
  -F "config=text" \
  -F "files=@/path/to/image.png"
```

Example using a local file named `sample.png`:

```bash
curl -X POST http://127.0.0.1:3000/ocr/tesseract \
  -F "language=eng" \
  -F "config=text" \
  -F "files=@sample.png"
```

### hOCR output

```bash
curl -X POST http://127.0.0.1:3000/ocr/tesseract \
  -F "language=eng" \
  -F "config=hocr" \
  -F "files=@sample.png"
```

### Multiple files

```bash
curl -X POST http://127.0.0.1:3000/ocr/tesseract \
  -F "language=eng" \
  -F "config=text" \
  -F "files=@page-1.png" \
  -F "files=@page-2.png"
```

### Pretty-print the JSON response

If `jq` is installed:

```bash
curl -s -X POST http://127.0.0.1:3000/ocr/tesseract \
  -F "language=eng" \
  -F "config=text" \
  -F "files=@sample.png" | jq
```

### Save hOCR output to a file

The response is JSON, so the hOCR content is inside the `results[0].hocr` field.

With `jq`:

```bash
curl -s -X POST http://127.0.0.1:3000/ocr/tesseract \
  -F "language=eng" \
  -F "config=hocr" \
  -F "files=@sample.png" \
  | jq -r '.results[0].hocr' > output.hocr
```

## Deploy to a Debian 12 server

Build the Linux executable:

```bash
./scripts/build-linux.sh
```

Deploy using the provided SCP script:

```bash
chmod +x scripts/deploy-scp.sh
./scripts/deploy-scp.sh user@example.com /opt/ocr-microservice
```

This copies:

- `dist/OcrMicroService`
- `config_examples/tesseract_native.toml` as `tesseract.toml`

to the target directory.

On the server, make sure Tesseract is installed:

```bash
sudo apt update
sudo apt install -y tesseract-ocr tesseract-ocr-eng
```

Then run:

```bash
/opt/ocr-microservice/OcrMicroService --config /opt/ocr-microservice/tesseract.toml
```

## Running as a service

For production use, it is recommended to run the executable using `systemd`, a process manager, or another service supervisor.

A typical installation directory could be:

```text
/opt/ocr-microservice
```

The executable can be started with:

```bash
/opt/ocr-microservice/OcrMicroService --config /opt/ocr-microservice/tesseract.toml
```

Systemd unit example: `nano ocr-microservice.service`

```
[Unit]
Description=OCR Microservice
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/ocr-microservice
ExecStart=/opt/ocr-microservice/OcrMicroService --config /opt/ocr-microservice/tesseract.toml
Restart=always
RestartSec=5
User=ocr
Group=ocr

[Install]
WantedBy=multi-user.target
```

Prepare with:

```bash
sudo useradd --system --home /opt/ocr-microservice --shell /usr/sbin/nologin ocr
sudo chown -R ocr:ocr /opt/ocr-microservice

sudo cp ocr-microservice.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now ocr-microservice
sudo systemctl status ocr-microservice
```

## Notes

- Uploaded files are temporarily written to `/tmp`.
- The default OCR language is `eng`.
- The default output config is `text`.
- For hOCR output, use `config=hocr`.
- If using native Tesseract, the required language data must be installed on the server.
- If using Docker-based Tesseract, Docker must be available on the machine running the service.
```