use reqwest::Client;
use serde_json::json;

pub struct Email {
    pub api_key: String,
    pub from: String,
    pub base_url: String,
}

impl Email {
    pub async fn send(&self, to: &str, subject: &str, html: &str) -> Result<(), String> {
        if self.api_key.is_empty() {
            // Log but don't fail in development
            println!("Email would be sent to {to}: {subject}");
            return Ok(());
        }

        let res = Client::new()
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "from": self.from,
                "to": to,
                "subject": subject,
                "html": html
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.text().await.unwrap_or_default())
        }
    }

    pub async fn send_verification(&self, to: &str, token: &str) -> Result<(), String> {
        let url = format!("{}/verify-email?token={}", self.base_url, token);
        self.send(
            to,
            "Verify your email - Artist Oil Paints",
            &format!(
                r#"
                <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto;">
                    <h2>Welcome to Artist Oil Paints!</h2>
                    <p>Please verify your email address by clicking the button below:</p>
                    <p style="margin: 30px 0;">
                        <a href="{url}" style="background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px;">
                            Verify Email
                        </a>
                    </p>
                    <p style="color: #666; font-size: 14px;">
                        Or copy this link: <a href="{url}">{url}</a>
                    </p>
                    <p style="color: #666; font-size: 14px;">This link expires in 24 hours.</p>
                </div>
                "#
            ),
        )
        .await
    }

    pub async fn send_password_reset(&self, to: &str, token: &str) -> Result<(), String> {
        let url = format!("{}/reset-password?token={}", self.base_url, token);
        self.send(
            to,
            "Reset your password - Artist Oil Paints",
            &format!(
                r#"
                <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto;">
                    <h2>Password Reset Request</h2>
                    <p>Click the button below to reset your password:</p>
                    <p style="margin: 30px 0;">
                        <a href="{url}" style="background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px;">
                            Reset Password
                        </a>
                    </p>
                    <p style="color: #666; font-size: 14px;">
                        Or copy this link: <a href="{url}">{url}</a>
                    </p>
                    <p style="color: #666; font-size: 14px;">This link expires in 1 hour.</p>
                    <p style="color: #666; font-size: 14px;">If you didn't request this, you can safely ignore this email.</p>
                </div>
                "#
            ),
        )
        .await
    }
}
