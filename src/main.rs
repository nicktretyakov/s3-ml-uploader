use aws_config::Region;
use aws_sdk_s3::Client;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client as ReqwestClient, Method};
// Use s3 crate with the correct imports
use s3::{bucket::Bucket, creds::Credentials as S3Credentials, region::Region as S3Region};
use sha2::{Digest, Sha256}; // Add Digest trait
use std::{env, path::Path, sync::Arc};
use tokio::{fs, task};

// ML model for file type prediction
mod ml;
use ml::FileTypePredictor;

// Region provider implementation based on the attached file
struct RegionProvider {
    region: String,
}

impl RegionProvider {
    fn new(region: &str) -> Self {
        Self {
            region: region.to_string(),
        }
    }

    async fn region(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.region.clone())
    }
}

/// AWS S3 client creation
async fn create_aws_client() -> Client {
    let region = Region::new("us-east-1");

    // Use defaults() instead of from_env() to avoid deprecation warning
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .load()
        .await;

    Client::new(&config)
}

/// S3 compatible client (e.g., MinIO)
fn create_s3_client() -> Bucket {
    let credentials = S3Credentials::new(
        Some(&env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string())),
        Some(&env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string())),
        None,
        None,
        None,
    )
    .unwrap();

    let region = S3Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
    };

    Bucket::new(
        &env::var("S3_BUCKET").unwrap_or_else(|_| "minio-bucket".to_string()),
        region,
        credentials,
    )
    .unwrap()
}

/// Direct file upload via HTTP request with AWS V4 signature
async fn upload_via_http(file_path: &str, bucket: &str, key: &str) -> Result<(), reqwest::Error> {
    let client = ReqwestClient::new();
    let file_content = fs::read(file_path).await.unwrap();

    let access_key = env::var("AWS_ACCESS_KEY").unwrap_or_else(|_| "your-access-key".to_string());
    let secret_key = env::var("AWS_SECRET_KEY").unwrap_or_else(|_| "your-secret-key".to_string());
    let region = "us-east-1";
    let host = format!("{}.s3.amazonaws.com", bucket);
    let url = format!("https://{}/{}", host, key);
    let date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let scope = format!("{}/{}/s3/aws4_request", &date[..8], region);

    // Create a SHA-256 hash of the file content
    // Fix the digest usage
    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let content_hash = hex::encode(hasher.finalize());

    let string_to_sign = format!("AWS4-HMAC-SHA256\n{}\n{}\n{}", date, scope, content_hash);

    // Create the signing key
    let mut hmac =
        Hmac::<Sha256>::new_from_slice(format!("AWS4{}", secret_key).as_bytes()).unwrap();
    hmac.update(date[..8].as_bytes());
    let date_key = hmac.finalize().into_bytes();

    let mut hmac = Hmac::<Sha256>::new_from_slice(&date_key).unwrap();
    hmac.update(region.as_bytes());
    let region_key = hmac.finalize().into_bytes();

    let mut hmac = Hmac::<Sha256>::new_from_slice(&region_key).unwrap();
    hmac.update(b"s3");
    let service_key = hmac.finalize().into_bytes();

    let mut hmac = Hmac::<Sha256>::new_from_slice(&service_key).unwrap();
    hmac.update(b"aws4_request");
    let signing_key = hmac.finalize().into_bytes();

    // Sign the string to sign
    let mut hmac = Hmac::<Sha256>::new_from_slice(&signing_key).unwrap();
    hmac.update(string_to_sign.as_bytes());
    let signature = hex::encode(hmac.finalize().into_bytes());

    let authorization_header = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={}",
        access_key, scope, signature
    );

    let res = client
        .request(Method::PUT, &url)
        .header("Authorization", authorization_header)
        .header("x-amz-date", &date)
        .header("x-amz-content-sha256", &content_hash)
        .header("Content-Length", file_content.len())
        .body(file_content)
        .send()
        .await?;

    println!("Uploaded via HTTP: {} (Status: {})", key, res.status());
    Ok(())
}

/// File upload to AWS S3 using the AWS SDK
async fn upload_to_aws_s3(client: Arc<Client>, file_path: &str, bucket: &str, key: &str) {
    let file_content = fs::read(file_path).await.unwrap();

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(file_content.into())
        .send()
        .await
        .unwrap();

    println!("Uploaded to AWS S3: {}", key);
}

/// File upload to MinIO
async fn upload_to_minio(bucket: &Bucket, file_path: &str, key: &str) {
    let file_content = fs::read(file_path).await.unwrap();

    bucket.put_object(key, &file_content).await.unwrap();
    println!("Uploaded to MinIO: {}", key);
}

/// Download file from AWS S3
async fn download_from_aws_s3(client: Arc<Client>, bucket: &str, key: &str, output_path: &str) {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .unwrap();

    let data = resp.body.collect().await.unwrap().into_bytes();
    fs::write(output_path, data).await.unwrap();

    println!("Downloaded from AWS S3: {} -> {}", key, output_path);
}

/// Download file from MinIO
async fn download_from_minio(bucket: &Bucket, key: &str, output_path: &str) {
    let (data, _) = bucket.get_object(key).await.unwrap();
    fs::write(output_path, data).await.unwrap();

    println!("Downloaded from MinIO: {} -> {}", key, output_path);
}

/// Process file with ML model before upload
async fn process_file_with_ml(file_path: &str) -> String {
    let file_content = fs::read(file_path).await.unwrap();

    // Initialize ML model
    let predictor = FileTypePredictor::new();

    // Predict file type and get appropriate storage location
    let file_type = predictor.predict(&file_content);
    println!("ML model predicted file type: {}", file_type);

    // Return appropriate key based on file type
    format!(
        "{}/{}",
        file_type,
        Path::new(file_path).file_name().unwrap().to_str().unwrap()
    )
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    println!("Starting S3 ML File Uploader");

    // Create clients
    let aws_client = Arc::new(create_aws_client().await);
    let minio_bucket = create_s3_client();

    // Define files to upload
    let files = vec!["file1.txt", "file2.txt", "file3.txt"];
    let aws_bucket = env::var("AWS_BUCKET").unwrap_or_else(|_| "aws-bucket".to_string());

    // Process files in parallel with ML analysis
    let mut handles = Vec::new();

    for file in files {
        let aws_client = Arc::clone(&aws_client);
        let minio_bucket = minio_bucket.clone();
        let file_str = file.to_string();
        let aws_bucket_str = aws_bucket.clone();

        let handle = task::spawn(async move {
            // Process file with ML to determine appropriate storage location
            let ml_key = process_file_with_ml(&file_str).await;

            // Upload to AWS S3
            let aws_handle = task::spawn(async move {
                upload_to_aws_s3(aws_client, &file_str, &aws_bucket_str, &ml_key).await;
            });

            // Upload to MinIO
            let minio_handle = task::spawn(async move {
                upload_to_minio(&minio_bucket, &file_str, &ml_key).await;
            });

            // Upload via direct HTTP
            let http_handle = task::spawn(async move {
                upload_via_http(&file_str, &aws_bucket_str, &ml_key)
                    .await
                    .unwrap();
            });

            // Wait for all uploads to complete
            aws_handle.await.unwrap();
            minio_handle.await.unwrap();
            http_handle.await.unwrap();

            println!("All uploads completed for file: {}", file_str);
        });

        handles.push(handle);
    }

    // Wait for all file processing to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("All files processed and uploaded successfully!");
}
