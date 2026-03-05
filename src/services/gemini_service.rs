use anyhow::Result;
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking_config: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
struct SystemInstruction {
    parts: Vec<TextPart>,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    thinking_level: String,
}

#[derive(Debug, Serialize)]
struct Content {
    role: String,
    parts: Vec<TextPart>,
}

#[derive(Debug, Serialize)]
struct TextPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: Option<String>,
    #[serde(default)]
    thought: Option<bool>,
}

#[derive(Clone)]
pub struct GeminiService {
    api_key: String,
    model: String,
    client: Arc<reqwest::Client>,
}

impl GeminiService {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client
            ::builder()
            .timeout(std::time::Duration::from_secs(300))
            .pool_max_idle_per_host(5)
            .build()
            .expect("Failed to build Gemini HTTP client");

        Self {
            api_key,
            model,
            client: Arc::new(client),
        }
    }

    /// Base URL for the Gemini generateContent endpoint.
    fn endpoint(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model,
            self.api_key
        )
    }

    /// Send a simple text prompt and get back the text response.
    pub async fn generate_text(&self, prompt: &str) -> Result<String> {
        self.generate_with_system(prompt, None, 1.0, 8192).await
    }

    /// Send a prompt with an optional system instruction.
    pub async fn generate_with_system(
        &self,
        prompt: &str,
        system: Option<&str>,
        temperature: f32,
        max_tokens: u32
    ) -> Result<String> {
        let body = GeminiRequest {
            contents: vec![Content {
                role: "user".into(),
                parts: vec![TextPart { text: prompt.into() }],
            }],
            system_instruction: system.map(|s| SystemInstruction {
                parts: vec![TextPart { text: s.into() }],
            }),
            generation_config: Some(GenerationConfig {
                temperature,
                max_output_tokens: max_tokens,
            }),
            thinking_config: Some(ThinkingConfig {
                thinking_level: "low".into(),
            }),
        };

        let response = self.client.post(&self.endpoint()).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error {}: {}", status, text);
        }

        let resp: GeminiResponse = response.json().await?;

        Self::extract_text(&resp)
    }

    /// Send a multi-turn conversation history + new user message.
    pub async fn generate_chat_response(
        &self,
        system_prompt: &str,
        history: &[(String, String)], // (role, content) pairs
        user_message: &str,
        temperature: f32
    ) -> Result<String> {
        let mut contents: Vec<Content> = history
            .iter()
            .map(|(role, content)| Content {
                role: role.clone(),
                parts: vec![TextPart { text: content.clone() }],
            })
            .collect();

        contents.push(Content {
            role: "user".into(),
            parts: vec![TextPart {
                text: user_message.into(),
            }],
        });

        let body = GeminiRequest {
            contents,
            system_instruction: Some(SystemInstruction {
                parts: vec![TextPart {
                    text: system_prompt.into(),
                }],
            }),
            generation_config: Some(GenerationConfig {
                temperature,
                max_output_tokens: 4096,
            }),
            thinking_config: Some(ThinkingConfig {
                thinking_level: "low".into(),
            }),
        };

        let response = self.client.post(&self.endpoint()).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error {}: {}", status, text);
        }

        let resp: GeminiResponse = response.json().await?;

        Self::extract_text(&resp)
    }

    /// Extract the actual text response, skipping any thinking parts.
    fn extract_text(resp: &GeminiResponse) -> Result<String> {
        resp.candidates
            .first()
            .and_then(|c| {
                c.content.parts
                    .iter()
                    .filter(|p| !p.thought.unwrap_or(false))
                    .find_map(|p| p.text.clone())
            })
            .ok_or_else(|| anyhow::anyhow!("Empty response from Gemini"))
    }

    /// Auto-generate a short chat title from the first user message.
    pub async fn generate_chat_title(&self, first_message: &str) -> Result<String> {
        let prompt =
            format!(r#"Generate a concise chat title (max 5 words, no quotes) for a mental health companion conversation starting with:
"{}"
Return ONLY the title text."#, first_message);

        let raw = self.generate_text(&prompt).await?;
        let title = raw.trim().trim_matches('"').chars().take(60).collect::<String>();
        Ok(if title.is_empty() { "New Chat".into() } else { title })
    }

    /// Analyze mood data and produce a mental health analytics summary.
    pub async fn analyze_mood_data(
        &self,
        user_name: &str,
        mood_entries_json: &str,
        baseline_json: &str,
        period: &str
    ) -> Result<String> {
        let prompt = format!(
            r#"You are a mental health analytics AI. Analyze the following data for user "{user_name}" over the {period} period.

BASELINE MENTAL CHARACTERISTICS:
{baseline}

MOOD ENTRIES (chronological):
{mood}

Provide a comprehensive JSON analytics report with this exact structure:
{{
  "summary": "2-3 paragraph narrative summary of the user's mental health during this period",
  "insights": "Key patterns, triggers, and behavioral observations as a detailed paragraph",
  "recommendations": "Specific, actionable recommendations for improving mental health as a numbered list",
  "overall_mood_trend": "improving|stable|declining",
  "risk_level": "low|moderate|high|severe",
  "avg_mood_score": <number 1-10>,
  "key_triggers": ["list", "of", "identified", "triggers"],
  "positive_patterns": ["list of positive behaviors observed"],
  "areas_of_concern": ["list of concerning patterns if any"]
}}

Be empathetic, evidence-based, and constructive. Do not diagnose. Focus on patterns and practical support.
Return ONLY valid JSON."#,
            user_name = user_name,
            period = period,
            baseline = baseline_json,
            mood = mood_entries_json
        );

        self.generate_with_system(
            &prompt,
            Some(
                "You are a professional mental health analytics assistant. Always respond with valid JSON only."
            ),
            0.4,
            4096
        ).await
    }
}
