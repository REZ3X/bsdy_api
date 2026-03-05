use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport,
    AsyncTransport,
    Message,
    Tokio1Executor,
};

use crate::{ config::BrevoConfig, error::{ AppError, Result } };

#[derive(Clone)]
pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    from_email: String,
    from_name: String,
    app_name: String,
    frontend_url: String,
}

impl EmailService {
    pub fn new(brevo: &BrevoConfig, app_name: &str, frontend_url: &str) -> Self {
        Self {
            smtp_host: brevo.smtp_host.clone(),
            smtp_port: brevo.smtp_port,
            smtp_user: brevo.smtp_user.clone(),
            smtp_pass: brevo.smtp_pass.clone(),
            from_email: brevo.from_email.clone(),
            from_name: brevo.from_name.clone(),
            app_name: app_name.to_string(),
            frontend_url: frontend_url.to_string(),
        }
    }

    fn build_mailer(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(self.smtp_user.clone(), self.smtp_pass.clone());
        AsyncSmtpTransport::<Tokio1Executor>
            ::starttls_relay(&self.smtp_host)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("SMTP relay error: {}", e)))?
            .port(self.smtp_port)
            .credentials(creds)
            .build()
            .pipe_ok()
    }

    async fn send(&self, email: Message) -> Result<()> {
        let mailer = self.build_mailer()?;
        mailer.send(email).await.map_err(|e| {
            tracing::error!("Failed to send email: {}", e);
            AppError::InternalError(anyhow::anyhow!("Failed to send email: {}", e))
        })?;
        Ok(())
    }

    /// Send email verification link.
    pub async fn send_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str
    ) -> Result<()> {
        let verification_url = format!("{}/auth/verify-email?token={}", self.frontend_url, token);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
<title>Verify Your Email</title>
<style>
  body {{ font-family: 'Segoe UI', Tahoma, sans-serif; background:#f0f4ff; margin:0; padding:0; }}
  .wrap {{ max-width:600px; margin:40px auto; }}
  .card {{ background:#fff; border-radius:24px; padding:48px 40px; box-shadow:0 8px 40px rgba(100,120,255,0.10); }}
  h1 {{ color:#2d2d2d; font-size:26px; margin-top:0; }}
  p {{ color:#555; line-height:1.7; font-size:15px; }}
  .btn {{ display:inline-block; background:linear-gradient(135deg,#6366f1,#8b5cf6); color:#fff!important;
          padding:14px 36px; border-radius:50px; font-weight:700; text-decoration:none;
          font-size:15px; margin:24px 0; box-shadow:0 4px 16px rgba(99,102,241,0.35); }}
  .link {{ color:#6366f1; word-break:break-all; font-size:13px; }}
  .footer {{ text-align:center; color:#aaa; font-size:12px; margin-top:32px; }}
</style>
</head>
<body>
<div class="wrap">
  <div class="card">
    <h1>Welcome to Blessedly</h1>
    <p>Hi <strong>{name}</strong>,</p>
    <p>Thank you for joining us. Click the button below to verify your email and activate your account.</p>
    <div style="text-align:center">
      <a href="{url}" class="btn">Verify Email Address</a>
    </div>
    <p>Or copy this link into your browser:</p>
    <p><a href="{url}" class="link">{url}</a></p>
    <hr style="border:none;border-top:1px solid #eee;margin:28px 0"/>
    <p style="font-size:13px;color:#888">This link expires in 24 hours.<br>
    If you didn't create an account, you can ignore this email.</p>
  </div>
  <div class="footer">&copy; 2026 {app}. All rights reserved.</div>
</div>
</body>
</html>"#,
            app = self.app_name,
            name = to_name,
            url = verification_url
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse().unwrap())
            .to(format!("{} <{}>", to_name, to_email).parse().unwrap())
            .subject(format!("Verify your {} account", self.app_name))
            .header(ContentType::TEXT_HTML)
            .body(html)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Email build error: {}", e)))?;

        self.send(email).await?;
        tracing::info!("Verification email sent to {}", to_email);
        Ok(())
    }

    /// Send weekly/monthly mental health report email.
    pub async fn send_report_email(
        &self,
        to_email: &str,
        to_name: &str,
        report_type: &str,
        period_start: &str,
        period_end: &str,
        summary: &str,
        recommendations: &str,
        mood_trend: &str,
        avg_mood: Option<f32>,
        risk_level: &str
    ) -> Result<()> {
        let trend_label = match mood_trend {
            "improving" => "&#9650;",
            "declining" => "&#9660;",
            _ => "&#8594;",
        };

        let risk_color = match risk_level {
            "low" => "#22c55e",
            "moderate" => "#f59e0b",
            "high" => "#ef4444",
            "severe" => "#991b1b",
            _ => "#6b7280",
        };

        let mood_block = if let Some(mood) = avg_mood {
            format!(
                r#"<div style="text-align:center;margin:20px 0">
                <span style="font-size:40px;font-weight:800;color:#6366f1">{:.1}</span>
                <span style="font-size:18px;color:#888">/10</span>
                <p style="margin:4px 0;color:#888;font-size:13px">Average Mood Score</p>
            </div>"#,
                mood
            )
        } else {
            String::new()
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<title>{app} {rtype} Report</title>
<style>
  body {{ font-family: 'Segoe UI', Tahoma, sans-serif; background:#f0f4ff; margin:0;padding:0; }}
  .wrap {{ max-width:600px;margin:40px auto; }}
  .card {{ background:#fff;border-radius:24px;padding:48px 40px;box-shadow:0 8px 40px rgba(100,120,255,.10); }}
  h1 {{ color:#2d2d2d;font-size:24px;margin-top:0; }}
  h2 {{ color:#4b5563;font-size:18px;margin-top:32px; }}
  p {{ color:#555;line-height:1.7;font-size:15px; }}
  .badge {{ display:inline-block;padding:4px 14px;border-radius:50px;font-size:13px;font-weight:700; }}
  .section {{ background:#f8fafc;border-radius:16px;padding:20px;margin:16px 0; }}
  .footer {{ text-align:center;color:#aaa;font-size:12px;margin-top:32px; }}
</style>
</head>
<body>
<div class="wrap">
<div class="card">
  <h1>Your {rtype} Mental Health Report</h1>
  <p style="color:#6b7280;font-size:14px">{start} – {end} &nbsp;|&nbsp; <strong>{name}</strong></p>

  {mood_block}

  <div style="display:flex;gap:12px;flex-wrap:wrap;margin:16px 0">
    <span class="badge" style="background:#ede9fe;color:#6d28d9">{trend_label} Mood {trend}</span>
    <span class="badge" style="background:#fee2e2;color:{risk_color}">{risk} Risk</span>
  </div>

  <h2>Summary</h2>
  <div class="section"><p style="margin:0">{summary}</p></div>

  <h2>Recommendations</h2>
  <div class="section"><p style="margin:0">{recs}</p></div>

  <hr style="border:none;border-top:1px solid #eee;margin:28px 0"/>
  <p style="font-size:13px;color:#888">This report was generated automatically by {app}.<br>
  You can disable automated reports in your account settings.</p>
</div>
<div class="footer">&copy; 2026 {app}. All rights reserved.</div>
</div>
</body>
</html>"#,
            app = self.app_name,
            rtype = report_type,
            name = to_name,
            start = period_start,
            end = period_end,
            mood_block = mood_block,
            trend_label = trend_label,
            trend = mood_trend,
            risk_color = risk_color,
            risk = risk_level,
            summary = summary.replace('\n', "<br>"),
            recs = recommendations.replace('\n', "<br>")
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse().unwrap())
            .to(format!("{} <{}>", to_name, to_email).parse().unwrap())
            .subject(
                format!("{} {} Mental Health Report – {}", self.app_name, report_type, period_end)
            )
            .header(ContentType::TEXT_HTML)
            .body(html)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Email build error: {}", e)))?;

        self.send(email).await?;
        tracing::info!("Report email sent to {}", to_email);
        Ok(())
    }

    /// Send crisis alert email when severe risk is detected.
    pub async fn send_crisis_alert(&self, to_email: &str, to_name: &str) -> Result<()> {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"/><title>We're Here For You</title>
<style>
  body{{font-family:'Segoe UI',sans-serif;background:#fff0f0;margin:0;padding:0}}
  .wrap{{max-width:600px;margin:40px auto}}
  .card{{background:#fff;border-radius:24px;padding:48px 40px;box-shadow:0 8px 40px rgba(255,80,80,.12)}}
  h1{{color:#dc2626;font-size:24px;margin-top:0}}
  p{{color:#555;line-height:1.7;font-size:15px}}
  .hotline{{background:#fef2f2;border-left:4px solid #ef4444;padding:16px 20px;border-radius:0 12px 12px 0;margin:20px 0}}
  .footer{{text-align:center;color:#aaa;font-size:12px;margin-top:32px}}
</style></head>
<body>
<div class="wrap"><div class="card">
  <h1>We Care About You, {name}</h1>
  <p>We've noticed some patterns in your recent check-ins that suggest you may be going through a really difficult time.</p>
  <p>You are not alone. Please reach out to a mental health professional or crisis support line:</p>
  <div class="hotline">
    <strong>Into The Light Indonesia:</strong> 119 ext 8<br/>
    <strong>International Association for Suicide Prevention:</strong> <a href="https://www.iasp.info/resources/Crisis_Centres/">Find a crisis center</a>
  </div>
  <p>Our AI companion is here to listen, but for severe distress, please reach out to a qualified human professional immediately.</p>
  <p>Take care of yourself.</p>
</div>
<div class="footer">&copy; 2026 {app}. All rights reserved.</div>
</div>
</body>
</html>"#,
            name = to_name,
            app = self.app_name
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse().unwrap())
            .to(format!("{} <{}>", to_name, to_email).parse().unwrap())
            .subject(format!("An important message from {}", self.app_name))
            .header(ContentType::TEXT_HTML)
            .body(html)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Email build error: {}", e)))?;

        self.send(email).await?;
        tracing::warn!("Crisis alert email sent to {}", to_email);
        Ok(())
    }
}

trait PipeOk: Sized {
    fn pipe_ok(self) -> crate::error::Result<Self> {
        Ok(self)
    }
}
impl PipeOk for AsyncSmtpTransport<Tokio1Executor> {}
