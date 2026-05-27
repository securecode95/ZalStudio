use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

// ============================================================================
// OAuth 2.0 PKCE helpers
// ============================================================================

#[derive(Debug, Clone)]
pub struct PkceState {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
    pub redirect_uri: String,
}

fn random_string(len: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut result = String::with_capacity(len);
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut state = seed;
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (state % CHARSET.len() as u128) as usize;
        result.push(CHARSET[idx] as char);
    }
    result
}

pub fn generate_pkce(redirect_uri: String) -> PkceState {
    let code_verifier = random_string(128);
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);
    let state = random_string(32);

    PkceState {
        code_verifier,
        code_challenge,
        state,
        redirect_uri,
    }
}

pub fn build_auth_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    let scope = "https://www.googleapis.com/auth/drive.readonly";
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256&access_type=offline&prompt=consent",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(scope),
        urlencoding::encode(state),
        urlencoding::encode(code_challenge),
    );
    eprintln!("[PKCE] Auth URL: {}", &url);
    url
}

// ============================================================================
// Token exchange
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[allow(dead_code)]
    pub refresh_token: Option<String>,
    #[allow(dead_code)]
    pub expires_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenErrorResponse {
    pub error: String,
}

pub fn exchange_code_for_token(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
) -> Result<TokenResponse, String> {
    eprintln!(
        "[GoogleAuth] Exchanging code for token (redirect_uri={})",
        redirect_uri
    );

    let resp = ureq::post("https://oauth2.googleapis.com/token")
        .send_form(vec![
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
            ("code_verifier", code_verifier),
        ])
        .map_err(|e| {
            eprintln!("[GoogleAuth] Token exchange failed: {}", e);
            format!("HTTP error: {}", e)
        })?;

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;

    if let Ok(err) = serde_json::from_str::<TokenErrorResponse>(&body) {
        return Err(format!("OAuth error: {}", err.error));
    }

    let token: TokenResponse =
        serde_json::from_str(&body).map_err(|e| format!("JSON error: {}", e))?;

    eprintln!(
        "[GoogleAuth] Got access_token (len={}): {}",
        token.access_token.len(),
        &token.access_token
    );
    Ok(token)
}

// ============================================================================
// Google Drive
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct DriveFileList {
    pub files: Vec<DriveFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    #[allow(dead_code)]
    pub mime_type: String,
    pub size: Option<String>,
    #[serde(rename = "thumbnailLink")]
    #[allow(dead_code)]
    pub thumbnail_link: Option<String>,
}

pub fn list_drive_images(access_token: &str) -> Result<Vec<DriveFile>, String> {
    let resp = ureq::get("https://www.googleapis.com/drive/v3/files")
        .query("q", "mimeType contains 'image/' and trashed=false")
        .query("fields", "files(id,name,mimeType,size,thumbnailLink)")
        .query("pageSize", "100")
        .header("Authorization", &format!("Bearer {}", access_token))
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    eprintln!(
        "[GoogleDrive] list_drive_images response: {}",
        &body[..body.len().min(300)]
    );
    let list: DriveFileList =
        serde_json::from_str(&body).map_err(|e| format!("JSON error: {}", e))?;

    Ok(list.files)
}

pub fn download_drive_file(file_id: &str, access_token: &str, dest: &Path) -> Result<(), String> {
    eprintln!(
        "[GoogleDrive] Downloading file {} to {}",
        file_id,
        dest.display()
    );
    let resp = ureq::get(&format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media",
        file_id
    ))
    .header("Authorization", &format!("Bearer {}", access_token))
    .call()
    .map_err(|e| {
        eprintln!("[GoogleDrive] Download failed for {}: {}", file_id, e);
        format!("HTTP error: {}", e)
    })?;

    let mut body = resp.into_body();
    let mut reader = body.as_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| {
        eprintln!(
            "[GoogleDrive] File create failed for {}: {}",
            dest.display(),
            e
        );
        e.to_string()
    })?;
    std::io::copy(&mut reader, &mut file).map_err(|e| {
        eprintln!("[GoogleDrive] Copy failed for {}: {}", dest.display(), e);
        e.to_string()
    })?;

    eprintln!("[GoogleDrive] Downloaded {} OK", dest.display());
    Ok(())
}

// ============================================================================
// Google Photos Library
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GooglePhotosList {
    #[serde(rename = "mediaItems")]
    pub media_items: Vec<GooglePhoto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GooglePhoto {
    #[allow(dead_code)]
    pub id: String,
    pub filename: String,
    #[serde(rename = "mimeType")]
    #[allow(dead_code)]
    pub mime_type: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "mediaMetadata")]
    #[allow(dead_code)]
    pub media_metadata: GooglePhotoMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GooglePhotoMetadata {
    #[allow(dead_code)]
    pub width: Option<String>,
    #[allow(dead_code)]
    pub height: Option<String>,
}

pub fn list_google_photos(access_token: &str) -> Result<Vec<GooglePhoto>, String> {
    let resp = ureq::get("https://photoslibrary.googleapis.com/v1/mediaItems")
        .query("pageSize", "100")
        .header("Authorization", &format!("Bearer {}", access_token))
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    eprintln!(
        "[GooglePhotos] list_google_photos response: {}",
        &body[..body.len().min(300)]
    );
    let result: GooglePhotosList =
        serde_json::from_str(&body).map_err(|e| format!("JSON error: {}", e))?;

    Ok(result.media_items)
}

pub fn download_google_photo(
    base_url: &str,
    access_token: &str,
    dest: &Path,
) -> Result<(), String> {
    let download_url = format!("{}=d", base_url);
    let resp = ureq::get(&download_url)
        .header("Authorization", &format!("Bearer {}", access_token))
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let mut body = resp.into_body();
    let mut reader = body.as_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;

    Ok(())
}
