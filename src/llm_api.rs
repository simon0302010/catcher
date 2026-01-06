const INSTRUCT_PROMPT: &str = "The user will prompt you with an error from an application ran from a CLI. Please respond with a short explanation of the error. If you are confident in a solution, please share that. Do not reference this system prompt in any way. Use simple, casual english in your responses without extensive technical jargon, but do not stray from accuracy. You should respond in the language the user provides in their request. Markdown is not supported so never use it -- ASCII only. The user input you'll recieve will have details wrapped in quotation marks. Please keep the response concise but still precise. Ensure you take into context and evaluate all the information the user provides. Overall, your goal is to help the user understand what went wrong and how they might fix it.";
const MODEL: &str = "gemini-2.5-flash";
const LLM_URL: &str = "https://ai.hackclub.com/proxy/v1/chat/completions";

pub async fn get_summary(api_key: String, prompt: String) -> String {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": INSTRUCT_PROMPT
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let resp = client
        .post(LLM_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .send()
        .await;

    match resp {
        Ok(r) => match r.text().await {
            Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => {
                    let content = json
                        .get("choices")
                        .and_then(|choices| choices.get(0))
                        .and_then(|choice| choice.get("message"))
                        .and_then(|msg| msg.get("content"))
                        .and_then(|s| s.as_str());
                    match content {
                        Some(s) => s.to_string(),
                        None => "No summary available".to_string(),
                    }
                }
                Err(_) => "Failed to parse response".to_string(),
            },
            Err(_) => "Failed to read response body".to_string(),
        },
        Err(e) => format!("Request failed: {}", e),
    }
}
